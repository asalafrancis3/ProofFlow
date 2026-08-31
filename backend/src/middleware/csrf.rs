use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    body::EitherBody,
    Error, HttpResponse,
};
use futures::future::LocalBoxFuture;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct CsrfMiddleware {
    pub secret: String,
}

impl CsrfMiddleware {
    pub fn new(secret: impl Into<String>) -> Self {
        Self { secret: secret.into() }
    }
}

/// Generates a CSRF token tied to a session ID and secret.
pub fn generate_token(session_id: &str, secret: &str) -> String {
    let mut h = DefaultHasher::new();
    format!("{}:{}", session_id, secret).hash(&mut h);
    hex::encode(h.finish().to_le_bytes())
}

/// Validates a CSRF token.
pub fn validate_token(token: &str, session_id: &str, secret: &str) -> bool {
    token == generate_token(session_id, secret)
}

impl<S, B> Transform<S, ServiceRequest> for CsrfMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = CsrfMiddlewareService<S>;
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(CsrfMiddlewareService {
            service,
            secret: self.secret.clone(),
        }))
    }
}

pub struct CsrfMiddlewareService<S> {
    service: S,
    secret: String,
}

impl<S, B> Service<ServiceRequest> for CsrfMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let method = req.method().as_str();
        let is_state_changing = matches!(method, "POST" | "PUT" | "PATCH" | "DELETE");

        if is_state_changing {
            let token = req
                .headers()
                .get("x-csrf-token")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            let session_id = req
                .headers()
                .get("x-session-id")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("default");

            if !validate_token(token, session_id, &self.secret) {
                let (req, _) = req.into_parts();
                let response =
                    HttpResponse::Forbidden().json(serde_json::json!({ "error": "Invalid or missing CSRF token" }));
                return Box::pin(async move { Ok(ServiceResponse::new(req, response).map_into_right_body()) });
            }
        }

        let fut = self.service.call(req);
        Box::pin(async move { fut.await.map(|res| res.map_into_left_body()) })
    }
}
