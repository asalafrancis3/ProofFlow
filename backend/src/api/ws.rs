//! WebSocket handler — structured error codes (#1092).
//!
//! All error frames emitted over the WebSocket connection use `WsErrorFrame`
//! with a `WsErrorCode` variant so that clients can match on a stable,
//! machine-readable code instead of parsing free-text messages.
//!
//! See `crate::errors::ws_errors` for the full code table and wire format.

use actix_web::{web, HttpRequest, HttpResponse};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::errors::ws_errors::{WsErrorCode, WsErrorFrame};
use crate::services::api::ApiBuilder;

// ── Message types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum WsMessage {
    Subscribe { channel: String },
    Unsubscribe { channel: String },
    Event { channel: String, data: serde_json::Value },
    Authenticate { token: String },
    AuthSuccess,
    /// Structured auth error — use `WsErrorFrame` for wire format.
    AuthError { code: WsErrorCode, message: String },
    Pong,
    Ping,
    /// Structured error — use `WsErrorFrame` for wire format.
    Error { code: WsErrorCode, message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsAuthRequest {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsSubscribeRequest {
    pub channel: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsUnsubscribeRequest {
    pub channel: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub connection_id: String,
    pub user_id: Option<String>,
    pub subscribed_channels: Vec<String>,
    pub connected_at: String,
    pub last_heartbeat: String,
}

// ── Connection manager ────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct WsConnectionManager {
    pub shutdown_flag: Arc<AtomicBool>,
}

impl WsConnectionManager {
    pub fn new() -> Self {
        Self {
            shutdown_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_shutting_down(&self) -> bool {
        self.shutdown_flag.load(Ordering::Relaxed)
    }

    pub fn initiate_shutdown(&self) {
        self.shutdown_flag.store(true, Ordering::Relaxed);
        info!("WebSocket server shutdown initiated");
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Serialise a `WsErrorFrame` to a JSON string for sending over the socket.
fn error_frame(code: WsErrorCode) -> String {
    WsErrorFrame::new(code).to_json()
}

/// Serialise a `WsErrorFrame` with a custom message.
fn error_frame_msg(code: WsErrorCode, message: &str) -> String {
    WsErrorFrame::with_message(code, message).to_json()
}

// ── Handler ───────────────────────────────────────────────────────────────────

pub async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    manager: web::Data<WsConnectionManager>,
) -> Result<HttpResponse, actix_web::Error> {
    if manager.is_shutting_down() {
        return Ok(
            HttpResponse::ServiceUnavailable().json(ApiBuilder::error_response::<String>("server_shutting_down", "server shutting down", 503))
        );
    }

    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;

    let connection_id = uuid::Uuid::new_v4().to_string();
    info!(connection_id = %connection_id, "WebSocket connection established");

    let mut authenticated = false;
    let mut user_id: Option<String> = None;
    let mut subscribed_channels: Vec<String> = Vec::new();
    let connected_at = chrono::Utc::now().to_rfc3339();
    let mut last_heartbeat = std::time::Instant::now();

    actix_web::rt::spawn(async move {
        let heartbeat_interval = std::time::Duration::from_secs(10);
        let mut interval = tokio::time::interval(heartbeat_interval);
        let shutdown_flag = manager.shutdown_flag.clone();

        loop {
            tokio::select! {
                Some(msg) = msg_stream.next() => {
                    match msg {
                        Ok(actix_ws::Message::Text(text)) => {
                            match serde_json::from_str::<WsMessage>(&text) {
                                Ok(WsMessage::Authenticate { token }) => {
                                    // Minimal length check — replace with real JWT validation.
                                    if token.len() >= 8 {
                                        authenticated = true;
                                        user_id = Some(format!("user_{}", &token[..8]));
                                        info!(user_id = %user_id.as_ref().unwrap(), "WebSocket client authenticated");
                                        let _ = session
                                            .text(serde_json::to_string(&WsMessage::AuthSuccess).unwrap())
                                            .await;
                                    } else {
                                        warn!("WebSocket authentication failed: invalid token");
                                        let _ = session
                                            .text(error_frame(WsErrorCode::AuthInvalidToken))
                                            .await;
                                    }
                                }
                                Ok(WsMessage::Subscribe { channel }) => {
                                    if !authenticated {
                                        let _ = session
                                            .text(error_frame(WsErrorCode::AuthTokenRequired))
                                            .await;
                                        continue;
                                    }
                                    if !subscribed_channels.contains(&channel) {
                                        subscribed_channels.push(channel.clone());
                                        info!(channel = %channel, "Client subscribed to channel");
                                    }
                                    let _ = session
                                        .text(
                                            serde_json::to_string(&serde_json::json!({
                                                "type": "subscribed",
                                                "channel": channel
                                            }))
                                            .unwrap(),
                                        )
                                        .await;
                                }
                                Ok(WsMessage::Unsubscribe { channel }) => {
                                    subscribed_channels.retain(|c| c != &channel);
                                    let _ = session
                                        .text(
                                            serde_json::to_string(&serde_json::json!({
                                                "type": "unsubscribed",
                                                "channel": channel
                                            }))
                                            .unwrap(),
                                        )
                                        .await;
                                }
                                Ok(WsMessage::Ping) | Ok(WsMessage::Pong) => {
                                    last_heartbeat = std::time::Instant::now();
                                    let _ = session
                                        .text(serde_json::to_string(&WsMessage::Pong).unwrap())
                                        .await;
                                }
                                // Unrecognised message type — structured error.
                                Ok(_) => {
                                    let _ = session
                                        .text(error_frame(WsErrorCode::MessageUnknownType))
                                        .await;
                                }
                                // Parse failure — structured error.
                                Err(_) => {
                                    let _ = session
                                        .text(error_frame(WsErrorCode::MessageParseError))
                                        .await;
                                }
                            }
                        }
                        Ok(actix_ws::Message::Ping(bytes)) => {
                            last_heartbeat = std::time::Instant::now();
                            let _ = session.pong(&bytes).await;
                        }
                        Ok(actix_ws::Message::Pong(_)) => {
                            last_heartbeat = std::time::Instant::now();
                        }
                        Ok(actix_ws::Message::Close(_)) => {
                            info!(connection_id = %connection_id, "WebSocket connection closed by client");
                            break;
                        }
                        Err(e) => {
                            error!(error = %e, "WebSocket protocol error");
                            break;
                        }
                        _ => {}
                    }
                }
                _ = interval.tick() => {
                    let _ = session.ping(b"").await;

                    if last_heartbeat.elapsed() > std::time::Duration::from_secs(30) {
                        warn!(connection_id = %connection_id, "Client heartbeat timeout, closing connection");
                        let _ = session
                            .text(error_frame(WsErrorCode::ServerHeartbeatTimeout))
                            .await;
                        break;
                    }

                    if shutdown_flag.load(Ordering::Relaxed) {
                        info!(connection_id = %connection_id, "Server shutting down, closing WebSocket");
                        let _ = session
                            .text(error_frame(WsErrorCode::ServerShuttingDown))
                            .await;
                        break;
                    }
                }
            }
        }

        info!(connection_id = %connection_id, "WebSocket connection closed");
    });

    Ok(response)
}

pub async fn ws_health() -> HttpResponse {
    HttpResponse::Ok().json(ApiBuilder::success_response(serde_json::json!({
        "status": "healthy",
        "protocol": "WebSocket",
        "version": "1.0.0"
    })))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── WsMessage serialisation ───────────────────────────────────────────────

    #[actix_web::test]
    async fn subscribe_message_serialises_correctly() {
        let msg = WsMessage::Subscribe {
            channel: "waste:updates".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("subscribe"));

        let deserialized: WsMessage = serde_json::from_str(&json).unwrap();
        match deserialized {
            WsMessage::Subscribe { channel } => assert_eq!(channel, "waste:updates"),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn auth_message_roundtrips() {
        let msg = WsMessage::Authenticate {
            token: "test_token_123".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: WsMessage = serde_json::from_str(&json).unwrap();
        match deserialized {
            WsMessage::Authenticate { token } => assert_eq!(token, "test_token_123"),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn event_message_roundtrips() {
        let msg = WsMessage::Event {
            channel: "test".to_string(),
            data: serde_json::json!({"key": "value"}),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: WsMessage = serde_json::from_str(&json).unwrap();
        match deserialized {
            WsMessage::Event { channel, data } => {
                assert_eq!(channel, "test");
                assert_eq!(data, serde_json::json!({"key": "value"}));
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn connection_info_roundtrips() {
        let info = ConnectionInfo {
            connection_id: "conn-1".to_string(),
            user_id: Some("user-1".to_string()),
            subscribed_channels: vec!["channel-1".to_string()],
            connected_at: "2024-01-01T00:00:00Z".to_string(),
            last_heartbeat: "2024-01-01T00:00:10Z".to_string(),
        };
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: ConnectionInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.connection_id, "conn-1");
        assert_eq!(deserialized.user_id, Some("user-1".to_string()));
    }

    // ── Structured error frame helpers ────────────────────────────────────────

    #[test]
    fn error_frame_helper_produces_valid_json_with_code() {
        let json = error_frame(WsErrorCode::AuthTokenRequired);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["type"], "error");
        assert_eq!(parsed["payload"]["code"], "auth.token_required");
        assert!(parsed["payload"]["message"].is_string());
    }

    #[test]
    fn error_frame_msg_helper_overrides_message() {
        let json = error_frame_msg(WsErrorCode::ChannelNotFound, "channel 'events' not found");
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["payload"]["code"], "channel.not_found");
        assert_eq!(parsed["payload"]["message"], "channel 'events' not found");
    }

    #[test]
    fn auth_invalid_token_error_frame_has_correct_code() {
        let json = error_frame(WsErrorCode::AuthInvalidToken);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["payload"]["code"], "auth.invalid_token");
    }

    #[test]
    fn server_shutting_down_error_frame_has_correct_code() {
        let json = error_frame(WsErrorCode::ServerShuttingDown);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["payload"]["code"], "server.shutting_down");
    }

    #[test]
    fn heartbeat_timeout_error_frame_has_correct_code() {
        let json = error_frame(WsErrorCode::ServerHeartbeatTimeout);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["payload"]["code"], "server.heartbeat_timeout");
    }

    #[test]
    fn parse_error_frame_has_correct_code() {
        let json = error_frame(WsErrorCode::MessageParseError);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["payload"]["code"], "message.parse_error");
    }

    #[test]
    fn unknown_type_error_frame_has_correct_code() {
        let json = error_frame(WsErrorCode::MessageUnknownType);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["payload"]["code"], "message.unknown_type");
    }
}
