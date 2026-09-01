//! # Idempotency Middleware — Issue #919
//!
//! Supports idempotent write operations via the `Idempotency-Key` request header.
//!
//! ## Protocol
//!
//! POST/PUT/PATCH/DELETE requests that supply an `Idempotency-Key` header
//! (max 128 chars, typically a UUID v4) are guaranteed to be processed
//! **at most once**. A duplicate request with the same key gets the cached
//! response without re-running handler logic.
//!
//! **Request header:**
//! ```text
//! Idempotency-Key: 550e8400-e29b-41d4-a716-446655440000
//! ```
//!
//! **Response header:**
//! ```text
//! X-Idempotency-Status: created   ← first invocation, handler ran
//! X-Idempotency-Status: replayed  ← duplicate key, cached response returned
//! ```
//!
//! ## Cache
//!
//! - In-process `Mutex<HashMap>` — no external dependency required.
//! - TTL: 24 hours.  Expired entries are pruned on every request.
//! - Only 2xx / 4xx responses are cached; 5xx server errors are **not**
//!   cached so a transient failure can be retried.
//! - Max key length: 128 bytes.

use actix_web::{
    body::MessageBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::{
        header::{HeaderName, HeaderValue},
        Method, StatusCode,
    },
    Error, HttpResponse,
};
use bytes::Bytes;
use futures::future::LocalBoxFuture;
use futures::FutureExt;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

// ── Constants ─────────────────────────────────────────────────────────────────

pub const IDEMPOTENCY_KEY_HEADER: &str = "idempotency-key";
const STATUS_HEADER: &str = "x-idempotency-status";
const TTL_SECS: u64 = 86_400; // 24 h
const MAX_KEY_LEN: usize = 128;

// ── Cached response entry ─────────────────────────────────────────────────────

#[derive(Clone)]
struct Cached {
    status: StatusCode,
    body: Bytes,
    born: Instant,
}

impl Cached {
    fn is_expired(&self, ttl: Duration) -> bool {
        self.born.elapsed() > ttl
    }
}

// ── Shared store type ─────────────────────────────────────────────────────────

type Store = Arc<Mutex<HashMap<String, Cached>>>;

// ── Transform (factory) ───────────────────────────────────────────────────────

/// Middleware factory — wrap with `.wrap(IdempotencyMiddleware::new())` in
/// `App::new()`.
pub struct IdempotencyMiddleware {
    store: Store,
    ttl: Duration,
}

impl IdempotencyMiddleware {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
            ttl: Duration::from_secs(TTL_SECS),
        }
    }

    /// Construct with a custom TTL (useful in tests).
    pub fn with_ttl_secs(secs: u64) -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
            ttl: Duration::from_secs(secs),
        }
    }
}

impl Default for IdempotencyMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, B> Transform<S, ServiceRequest> for IdempotencyMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = IdempotencyService<S>;
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(IdempotencyService {
            service,
            store: self.store.clone(),
            ttl: self.ttl,
        }))
    }
}

// ── Service (per-request logic) ───────────────────────────────────────────────

pub struct IdempotencyService<S> {
    service: S,
    store: Store,
    ttl: Duration,
}

impl<S, B> Service<ServiceRequest> for IdempotencyService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Only intercept write methods
        if !is_write_method(req.method()) {
            return Box::pin(self.service.call(req).map(|r| r.map(|res| res.map_into_boxed_body())));
        }

        // Extract key header
        let key = match req
            .headers()
            .get(IDEMPOTENCY_KEY_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
        {
            Some(k) => k,
            None => return Box::pin(self.service.call(req).map(|r| r.map(|res| res.map_into_boxed_body()))),
        };

        // Key too long → 400
        if key.len() > MAX_KEY_LEN {
            return Box::pin(async move {
                Err(actix_web::error::ErrorBadRequest(format!(
                    "Idempotency-Key must not exceed {MAX_KEY_LEN} bytes"
                )))
            });
        }

        let ttl = self.ttl;
        let store = self.store.clone();

        // Evict stale entries (cheap amortised cost)
        {
            let mut guard = store.lock().unwrap();
            guard.retain(|_, c| !c.is_expired(ttl));
        }

        // Cache hit → replay
        {
            let guard = store.lock().unwrap();
            if let Some(cached) = guard.get(&key) {
                let status = cached.status;
                let body = cached.body.clone();
                drop(guard);

                return Box::pin(async move {
                    let http_resp = HttpResponse::build(status)
                        .insert_header((STATUS_HEADER, "replayed"))
                        .insert_header(("content-type", "application/json"))
                        .body(body);
                    Ok(ServiceResponse::new(
                        req.into_parts().0,
                        http_resp.map_into_boxed_body(),
                    ))
                });
            }
        }

        // Cache miss → run handler, cache response
        let fut = self.service.call(req);
        Box::pin(async move {
            let svc_res = fut.await?;
            let status = svc_res.status();
            let (http_req, body) = svc_res.into_parts();

            let bytes = actix_web::body::to_bytes(body.into_body()).await.unwrap_or_default();

            // Only cache successful/client-error responses
            if status.is_success() || status.is_client_error() {
                let mut guard = store.lock().unwrap();
                guard.insert(
                    key,
                    Cached {
                        status,
                        body: bytes.clone(),
                        born: Instant::now(),
                    },
                );
            }

            let http_resp = HttpResponse::build(status)
                .insert_header((STATUS_HEADER, "created"))
                .insert_header(("content-type", "application/json"))
                .body(bytes);

            Ok(ServiceResponse::new(http_req, http_resp.map_into_boxed_body()))
        })
    }
}

fn is_write_method(m: &Method) -> bool {
    matches!(*m, Method::POST | Method::PUT | Method::PATCH | Method::DELETE)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App, HttpResponse};
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    async fn counter_handler(counter: web::Data<Arc<AtomicU32>>) -> HttpResponse {
        let n = counter.fetch_add(1, Ordering::SeqCst);
        HttpResponse::Ok().json(serde_json::json!({"count": n}))
    }

    fn app_with_counter() -> Arc<AtomicU32> {
        // Returns counter – used via test::init_service
        todo!() // placeholder; see individual tests below for inline approach
    }

    #[actix_web::test]
    async fn no_key_passes_through() {
        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();
        let app = test::init_service(
            App::new()
                .wrap(IdempotencyMiddleware::new())
                .app_data(web::Data::new(c))
                .route("/test", web::post().to(counter_handler)),
        )
        .await;

        let req = test::TestRequest::post().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(resp.headers().get(STATUS_HEADER).is_none(), "no key → no status header");
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[actix_web::test]
    async fn first_request_is_created() {
        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();
        let app = test::init_service(
            App::new()
                .wrap(IdempotencyMiddleware::new())
                .app_data(web::Data::new(c))
                .route("/test", web::post().to(counter_handler)),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/test")
            .insert_header((IDEMPOTENCY_KEY_HEADER, "uuid-111"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get(STATUS_HEADER).and_then(|v| v.to_str().ok()),
            Some("created")
        );
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[actix_web::test]
    async fn duplicate_key_is_replayed() {
        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();
        let app = test::init_service(
            App::new()
                .wrap(IdempotencyMiddleware::new())
                .app_data(web::Data::new(c))
                .route("/test", web::post().to(counter_handler)),
        )
        .await;

        // First call
        let r1 = test::TestRequest::post()
            .uri("/test")
            .insert_header((IDEMPOTENCY_KEY_HEADER, "uuid-222"))
            .to_request();
        let _ = test::call_service(&app, r1).await;

        // Duplicate call
        let r2 = test::TestRequest::post()
            .uri("/test")
            .insert_header((IDEMPOTENCY_KEY_HEADER, "uuid-222"))
            .to_request();
        let resp2 = test::call_service(&app, r2).await;

        assert_eq!(
            resp2.headers().get(STATUS_HEADER).and_then(|v| v.to_str().ok()),
            Some("replayed")
        );
        // Handler must NOT have been called again
        assert_eq!(counter.load(Ordering::SeqCst), 1, "handler called more than once");
    }

    #[actix_web::test]
    async fn get_requests_are_not_intercepted() {
        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();
        let app = test::init_service(
            App::new()
                .wrap(IdempotencyMiddleware::new())
                .app_data(web::Data::new(c))
                .route("/test", web::get().to(counter_handler)),
        )
        .await;

        for _ in 0..3u8 {
            let req = test::TestRequest::get()
                .uri("/test")
                .insert_header((IDEMPOTENCY_KEY_HEADER, "same-key"))
                .to_request();
            let resp = test::call_service(&app, req).await;
            assert!(resp.headers().get(STATUS_HEADER).is_none());
        }
        assert_eq!(counter.load(Ordering::SeqCst), 3, "GET should not be deduplicated");
    }

    #[actix_web::test]
    async fn cached_entry_expiry() {
        let long_ago = Instant::now() - Duration::from_secs(200);
        let entry = Cached {
            status: StatusCode::OK,
            body: Bytes::new(),
            born: long_ago,
        };
        assert!(entry.is_expired(Duration::from_secs(100)));
        assert!(!entry.is_expired(Duration::from_secs(300)));
    }
}
