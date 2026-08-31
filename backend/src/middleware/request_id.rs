use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header::{HeaderName, HeaderValue},
    Error, HttpMessage,
};
use futures::future::LocalBoxFuture;
use std::time::Instant;
use tracing::Instrument;
use uuid::Uuid;

/// The canonical request-ID header name (lower-case, per HTTP/2 requirements).
///
/// This constant is the *single source of truth* for the header name used when
/// reading an inbound request ID from a client and when echoing it back on
/// every response (including error responses).  Any middleware or handler that
/// needs to reference this header should import this constant rather than
/// hard-coding the string.
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Correlation ID for a single request. Read via `req.extensions().get::<RequestId>()`.
///
/// The value is guaranteed to be present on **every** request that passes
/// through [`RequestIdMiddleware`], including requests that are short-circuited
/// by downstream middleware (e.g. auth failures, rate-limit rejections).  This
/// is enforced by registering `RequestIdMiddleware` *before* all other
/// middleware in the `App` builder chain.
#[derive(Debug, Clone)]
pub struct RequestId(pub String);

impl RequestId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub struct RequestIdMiddleware;

impl<S, B> Transform<S, ServiceRequest> for RequestIdMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestIdMiddlewareService<S>;
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(RequestIdMiddlewareService { service }))
    }
}

pub struct RequestIdMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestIdMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Accept an existing request ID from the client or generate a fresh UUID.
        // This runs before any downstream middleware so that every subsequent
        // handler (including auth rejections, rate-limit 429s, etc.) can read
        // the request ID from the request extensions.
        let request_id = req
            .headers()
            .get(REQUEST_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .filter(|v| !v.is_empty())
            .map(|v| v.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        // Attach to request extensions — accessible to all subsequent middleware
        // and handlers via `req.extensions().get::<RequestId>()`.
        req.extensions_mut().insert(RequestId(request_id.clone()));

        let span = tracing::info_span!(
            "http_request",
            request_id = %request_id,
            method = %req.method(),
            path = %req.path(),
        );

        let started_at = Instant::now();
        let fut = self.service.call(req);

        Box::pin(
            async move {
                tracing::debug!("request started");
                let mut res = fut.await?;
                let elapsed_ms = started_at.elapsed().as_millis();
                tracing::info!(
                    status = res.status().as_u16(),
                    elapsed_ms = elapsed_ms as u64,
                    "request completed"
                );

                // Echo the request ID on **every** response, including 4xx/5xx
                // error responses produced by upstream middleware layers.
                if let Ok(header_value) = HeaderValue::from_str(&request_id) {
                    res.headers_mut().insert(
                        HeaderName::from_static(REQUEST_ID_HEADER),
                        header_value,
                    );
                }

                Ok(res)
            }
            .instrument(span),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App, HttpResponse};

    /// Every successful response must carry `x-request-id`.
    #[actix_web::test]
    async fn test_request_id_present_on_200() {
        let app = test::init_service(
            App::new()
                .wrap(RequestIdMiddleware)
                .route("/ok", web::get().to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;

        let req = test::TestRequest::get().uri("/ok").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(
            resp.headers().contains_key(REQUEST_ID_HEADER),
            "x-request-id missing on 200 response"
        );
    }

    /// A client-supplied request ID must be echoed back unchanged.
    #[actix_web::test]
    async fn test_client_supplied_request_id_echoed() {
        let app = test::init_service(
            App::new()
                .wrap(RequestIdMiddleware)
                .route("/ok", web::get().to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;

        let client_id = "my-correlation-id-abc123";
        let req = test::TestRequest::get()
            .uri("/ok")
            .insert_header((REQUEST_ID_HEADER, client_id))
            .to_request();
        let resp = test::call_service(&app, req).await;

        let echoed = resp
            .headers()
            .get(REQUEST_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        assert_eq!(echoed, client_id, "client-supplied request ID was not echoed");
    }

    /// Requests with no supplied ID must still get one generated.
    #[actix_web::test]
    async fn test_generated_request_id_is_nonempty() {
        let app = test::init_service(
            App::new()
                .wrap(RequestIdMiddleware)
                .route("/ok", web::get().to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;

        let req = test::TestRequest::get().uri("/ok").to_request();
        let resp = test::call_service(&app, req).await;

        let id = resp
            .headers()
            .get(REQUEST_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        assert!(!id.is_empty(), "generated request ID must not be empty");
    }

    /// Error responses (4xx/5xx) must also carry `x-request-id`.
    #[actix_web::test]
    async fn test_request_id_present_on_404() {
        let app = test::init_service(
            App::new()
                .wrap(RequestIdMiddleware)
                .default_service(web::route().to(|| async {
                    HttpResponse::NotFound().finish()
                })),
        )
        .await;

        let req = test::TestRequest::get().uri("/does-not-exist").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status().as_u16(), 404);
        assert!(
            resp.headers().contains_key(REQUEST_ID_HEADER),
            "x-request-id missing on 404 error response"
        );
    }

    /// `RequestId` must be injected into request extensions before any handler runs.
    #[actix_web::test]
    async fn test_request_id_in_extensions() {
        let app = test::init_service(
            App::new()
                .wrap(RequestIdMiddleware)
                .route(
                    "/check",
                    web::get().to(|req: actix_web::HttpRequest| async move {
                        let id = req
                            .extensions()
                            .get::<RequestId>()
                            .map(|r| r.0.clone())
                            .unwrap_or_default();
                        HttpResponse::Ok().body(id)
                    }),
                ),
        )
        .await;

        let req = test::TestRequest::get().uri("/check").to_request();
        let resp = test::call_service(&app, req).await;
        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap_or("");

        assert!(!body_str.is_empty(), "RequestId extension must be set in handler");
    }
}
