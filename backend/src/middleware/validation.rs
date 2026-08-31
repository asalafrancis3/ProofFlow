use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    body::EitherBody,
    Error, HttpResponse,
};
use futures::future::LocalBoxFuture;

const MAX_BODY_BYTES: usize = 1_048_576; // 1 MB

pub struct ValidationMiddleware;

impl<S, B> Transform<S, ServiceRequest> for ValidationMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = ValidationMiddlewareService<S>;
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(ValidationMiddlewareService { service }))
    }
}

pub struct ValidationMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for ValidationMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let method = req.method().clone();
        let needs_content_type = matches!(method.as_str(), "POST" | "PUT" | "PATCH");

        if needs_content_type {
            let content_type = req
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");

            if !content_type.starts_with("application/json") {
                let (req, _) = req.into_parts();
                let response = HttpResponse::UnsupportedMediaType()
                    .json(serde_json::json!({ "error": "Content-Type must be application/json" }));
                return Box::pin(async move { Ok(ServiceResponse::new(req, response).map_into_right_body()) });
            }

            if let Some(length) = req
                .headers()
                .get("content-length")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<usize>().ok())
            {
                if length > MAX_BODY_BYTES {
                    let (req, _) = req.into_parts();
                    let response = HttpResponse::PayloadTooLarge()
                        .json(serde_json::json!({ "error": "Request body must not exceed 1MB" }));
                    return Box::pin(async move { Ok(ServiceResponse::new(req, response).map_into_right_body()) });
                }
            }
        }

        let fut = self.service.call(req);
        Box::pin(async move { fut.await.map(|res| res.map_into_left_body()) })
    }
}
