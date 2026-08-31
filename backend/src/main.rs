mod api;
mod cache;
mod compliance;
mod config;
mod container;
mod crypto;
mod errors;
mod middleware;
mod redis;
mod rpc;
mod search;
mod security;
mod services;
mod validation;

use actix_cors::Cors;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, ResponseError};
use services::WebhookManager;
// Removed unused imports: RecommendationEngine, NFTManager, ChainAbstraction (#906)
use api::{
    archival as archival_api, audit, compliance_api, contracts, export, search as search_api, signing_api,
    verification, ws,
};
use cache::{Cache, CacheInvalidationManager};
use middleware::{
    CsrfMiddleware, IdempotencyMiddleware, RateLimitConfig, RateLimitMiddleware, RequestIdMiddleware,
    ValidationMiddleware,
};
use rpc::{StellarRpcClient, StellarRpcConfig};
// analytics API removed — legacy unregistered routes cleaned up (#906)
use container::AppContainer;
use errors::AppError;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let use_json = std::env::var("LOG_FORMAT")
        .map(|v| v.to_lowercase() == "json")
        .unwrap_or(false);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    if use_json {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().json().with_current_span(true))
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().pretty())
            .init();
    }

    info!(service = "backend", "Starting Scavenger Backend Server on 0.0.0.0:8080");

    // Build the DI container (#914) — all services are constructed here.
    let container = AppContainer::from_env().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    // Destructure what the closures below need so we can `.clone()` into each
    // worker thread without keeping the whole container alive.
    let email_service = container.email.clone();
    let notification_service = container.notification.clone();
    let reporting_service = container.reporting.clone();
    let storage_service = container.storage.clone();
    let webhook_manager = container.webhook.clone();
    let cache = container.cache.clone();
    let cache_invalidation = container.cache_invalidation.clone();
    let audit_service = container.audit.clone();
    let verification_service = container.verification.clone();
    let search_client = container.search.clone();
    let archival_service = container.archival.clone();
    let stellar_client = container.stellar.clone();

    info!(
        horizon = %stellar_client.horizon_url(),
        contract = %stellar_client.contract_id(),
        "Stellar RPC client ready"
    );
    info!("All services initialized via AppContainer (archival, cache, audit, websocket, verification)");

    let rate_limit_config = RateLimitConfig::default();
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let redis_client = redis::RedisClient::new(&redis_url).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to connect to Redis, rate limiting will use in-memory state");
        redis::RedisClient::new("redis://localhost:6379").unwrap()
    });
    let ws_manager = ws::WsConnectionManager::new();
    let csrf_secret = std::env::var("CSRF_SECRET").unwrap_or_else(|_| "change-me-in-production".to_string());
    let allowed_origins = std::env::var("ALLOWED_ORIGINS").unwrap_or_else(|_| "http://localhost:3000".to_string());

    HttpServer::new(move || {
        let cors = {
            let mut builder = Cors::default();
            for origin in allowed_origins.split(',').map(str::trim) {
                builder = builder.allowed_origin(origin);
            }
            builder
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                .allowed_headers(vec![
                    actix_web::http::header::AUTHORIZATION,
                    actix_web::http::header::CONTENT_TYPE,
                    actix_web::http::header::HeaderName::from_static("x-csrf-token"),
                    actix_web::http::header::HeaderName::from_static("x-session-id"),
                    // #919: idempotency key header
                    actix_web::http::header::HeaderName::from_static("idempotency-key"),
                ])
                .max_age(3600)
        };

        App::new()
            .wrap(cors)
            // ── Middleware ordering (documented per #1094) ────────────────────
            // Order matters: actix-web applies middleware in *reverse* registration
            // order on the way in (outermost first) and *forward* order on the way
            // out.  The sequence below guarantees that `RequestIdMiddleware` is the
            // very first middleware to run on the inbound path, so that a request ID
            // is present in the request extensions before any downstream middleware
            // (rate-limit, CSRF, validation, idempotency) can short-circuit with an
            // early error response.
            //
            // Inbound execution order (first → last):
            //   1. RequestIdMiddleware    — assign / echo x-request-id
            //   2. ValidationMiddleware  — reject malformed Content-Type / payloads
            //   3. RateLimitMiddleware   — reject over-quota requests (429)
            //   4. CsrfMiddleware        — reject CSRF violations (403)
            //   5. IdempotencyMiddleware — deduplicate write operations
            //   (application handlers)
            //
            // Because actix-web wraps in reverse, RequestIdMiddleware must be
            // registered *last* in the `.wrap()` chain so it executes *first*.
            .wrap(IdempotencyMiddleware::new())
            .wrap(RateLimitMiddleware::new(rate_limit_config.clone(), web::Data::new(redis_client.clone())))
            .wrap(ValidationMiddleware)
            .wrap(RequestIdMiddleware)
            .app_data(web::Data::new(email_service.clone()))
            .app_data(web::Data::new(notification_service.clone()))
            .app_data(web::Data::new(reporting_service.clone()))
            .app_data(web::Data::new(storage_service.clone()))
            .app_data(web::Data::new(webhook_manager.clone()))
            .app_data(web::Data::new(cache.clone()))
            .app_data(web::Data::new(cache_invalidation.clone()))
            .app_data(web::Data::new(audit_service.clone()))
            .app_data(web::Data::new(verification_service.clone()))
            .app_data(web::Data::new(ws_manager.clone()))
            .app_data(web::Data::new(search_client.clone()))
            .app_data(web::Data::new(archival_service.clone()))
            .app_data(web::Data::new(stellar_client.clone()))
            .app_data(web::JsonConfig::default().error_handler(json_error_handler))
            .default_service(web::route().to(not_found))
            // Health
            .route("/health", web::get().to(health_check))
            // Contract Queries (Task 1)
            .route("/api/v1/contracts/wastes", web::get().to(contracts::list_wastes))
            .route("/api/v1/contracts/wastes/{id}", web::get().to(contracts::get_waste))
            .route(
                "/api/v1/contracts/participants",
                web::get().to(contracts::list_participants),
            )
            .route(
                "/api/v1/contracts/participants/{id}",
                web::get().to(contracts::get_participant),
            )
            .route("/api/v1/contracts/stats", web::get().to(contracts::get_contract_stats))
            .route("/api/v1/contracts/info", web::get().to(contracts::get_contract_info))
            .route(
                "/api/v1/cache/invalidate/waste/{id}",
                web::post().to(contracts::invalidate_waste_cache),
            )
            .route(
                "/api/v1/cache/invalidate/all",
                web::post().to(contracts::invalidate_all_cache),
            )
            .route("/api/v1/cache/metrics", web::get().to(contracts::cache_metrics))
            // WebSocket (Task 2)
            .route("/ws", web::get().to(ws::ws_handler))
            .route("/ws/health", web::get().to(ws::ws_health))
            // Export (Task 3)
            .route("/api/v1/exports", web::post().to(export::export_data))
            .route("/api/v1/exports", web::get().to(export::list_exports))
            .route("/api/v1/exports/{id}/download", web::get().to(export::download_export))
            .route("/api/v1/exports/{id}/email", web::post().to(export::send_export_email))
            .route(
                "/api/v1/exports/scheduled",
                web::post().to(export::create_scheduled_export),
            )
            .route(
                "/api/v1/exports/scheduled",
                web::get().to(export::list_scheduled_exports),
            )
            .route(
                "/api/v1/exports/scheduled/{id}",
                web::delete().to(export::delete_scheduled_export),
            )
            // Audit (Task 4)
            .route("/api/v1/audit/logs", web::get().to(audit::list_audit_logs))
            .route("/api/v1/audit/logs/{id}", web::get().to(audit::get_audit_entry))
            .route("/api/v1/audit/summary", web::get().to(audit::get_audit_summary))
            .route("/api/v1/audit/report", web::post().to(audit::generate_audit_report))
            .route("/api/v1/audit/export", web::get().to(audit::export_audit_logs))
            .route("/api/v1/audit/alerts", web::post().to(audit::create_alert_rule))
            .route("/api/v1/audit/alerts", web::get().to(audit::list_alert_rules))
            .route("/api/v1/audit/retention", web::get().to(audit::get_retention_policy))
            .route("/api/v1/audit/retention", web::put().to(audit::update_retention_policy))
            .route("/api/v1/audit/purge", web::post().to(audit::purge_old_logs))
            // Verification (Task 5)
            .route(
                "/api/v1/verification/start",
                web::post().to(verification::start_verification),
            )
            .route(
                "/api/v1/verification/{participant_id}/status",
                web::get().to(verification::get_verification_status),
            )
            .route(
                "/api/v1/verification/document",
                web::post().to(verification::submit_document),
            )
            .route(
                "/api/v1/verification/document/{doc_id}/verify",
                web::post().to(verification::verify_document),
            )
            .route(
                "/api/v1/verification/checklist",
                web::post().to(verification::submit_checklist),
            )
            .route(
                "/api/v1/verification/pending-reviews",
                web::get().to(verification::get_pending_reviews),
            )
            .route(
                "/api/v1/verification/approve",
                web::post().to(verification::approve_participant),
            )
            .route(
                "/api/v1/verification/reject",
                web::post().to(verification::reject_participant),
            )
            .route(
                "/api/v1/verification/{participant_id}/retry",
                web::post().to(verification::retry_verification),
            )
            // Compliance Monitoring (Task 6)
            .route(
                "/api/v1/compliance/checklists",
                web::get().to(compliance_api::list_checklists),
            )
            .route(
                "/api/v1/compliance/checklists",
                web::post().to(compliance_api::create_checklist),
            )
            .route(
                "/api/v1/compliance/check",
                web::post().to(compliance_api::run_compliance_check),
            )
            .route(
                "/api/v1/compliance/alerts",
                web::get().to(compliance_api::list_compliance_alerts),
            )
            .route(
                "/api/v1/compliance/alert-rules",
                web::post().to(compliance_api::create_alert_rule),
            )
            .route(
                "/api/v1/compliance/alert-rules",
                web::get().to(compliance_api::list_alert_rules),
            )
            .route(
                "/api/v1/compliance/audit-trail",
                web::get().to(compliance_api::get_audit_trail),
            )
            .route(
                "/api/v1/compliance/report",
                web::post().to(compliance_api::generate_compliance_report),
            )
            // Transaction Signing (#1090)
            // sign/multisig/revoke/events/revocations removed — superseded by
            // packages/scavenger-sdk/src/signing.ts client-side signing.
            // Only stateless verify and documentation remain server-side.
            .route("/api/v1/signing/verify", web::post().to(signing_api::verify_signature))
            .route(
                "/api/v1/signing/documentation",
                web::get().to(signing_api::get_documentation),
            )
            // Search (Task 8)
            .route("/api/v1/search", web::get().to(search_api::search))
            .route("/api/v1/search/suggest", web::get().to(search_api::suggest))
            .route("/api/v1/search/config", web::get().to(search_api::get_search_config))
            // Archival (Task 9)
            .route("/api/v1/archival/policies", web::post().to(archival_api::create_policy))
            .route("/api/v1/archival/policies", web::get().to(archival_api::list_policies))
            .route(
                "/api/v1/archival/policies/{id}",
                web::get().to(archival_api::get_policy),
            )
            .route(
                "/api/v1/archival/policies/{id}",
                web::put().to(archival_api::update_policy),
            )
            .route(
                "/api/v1/archival/policies/{id}",
                web::delete().to(archival_api::delete_policy),
            )
            .route("/api/v1/archival/archives", web::get().to(archival_api::query_archives))
            .route("/api/v1/archival/archives", web::post().to(archival_api::archive_data))
            .route(
                "/api/v1/archival/archives/{id}",
                web::delete().to(archival_api::delete_archive),
            )
            .route(
                "/api/v1/archival/archives/{id}/restore",
                web::post().to(archival_api::restore_data),
            )
            .route("/api/v1/archival/stats", web::get().to(archival_api::get_statistics))
            .route("/api/v1/archival/jobs", web::get().to(archival_api::list_jobs))
            .route("/api/v1/archival/jobs/{id}", web::get().to(archival_api::get_job))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

/// Central fallback for unmatched routes — keeps 404s in the same
/// unified JSON error envelope as handler-returned `AppError`s.
async fn not_found(req: HttpRequest) -> HttpResponse {
    AppError::NotFound {
        resource: "route",
        id: req.path().to_string(),
    }
    .error_response()
}

/// Central fallback for malformed JSON request bodies — keeps 400s in
/// the same unified JSON error envelope instead of actix's plaintext default.
fn json_error_handler(err: actix_web::error::JsonPayloadError, _req: &HttpRequest) -> actix_web::Error {
    let response = AppError::BadRequest(err.to_string()).error_response();
    actix_web::error::InternalError::from_response(err, response).into()
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "1.0.0",
        "services": ["contracts", "websocket", "export", "audit", "cache"]
    }))
}
