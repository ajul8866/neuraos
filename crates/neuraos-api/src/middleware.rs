// neuraos-api/src/middleware.rs
// Axum middleware: auth, rate-limiting, request-id

use axum::{
    extract::Request,
    http::{HeaderMap, HeaderValue, header},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

/// Inject a X-Request-Id header on every request
pub async fn request_id(mut req: Request, next: Next) -> Response {
    let id = Uuid::new_v4().to_string();
    req.headers_mut().insert(
        "x-request-id",
        HeaderValue::from_str(&id).unwrap_or_else(|_| HeaderValue::from_static("unknown")),
    );
    let mut res = next.run(req).await;
    res.headers_mut().insert(
        "x-request-id",
        HeaderValue::from_str(&id).unwrap_or_else(|_| HeaderValue::from_static("unknown")),
    );
    res
}

/// Basic Bearer token extractor (stub — wire to ShieldAuthManager in production)
pub async fn require_auth(req: Request, next: Next) -> Result<Response, crate::ApiError> {
    let auth = req.headers().get(header::AUTHORIZATION);
    match auth {
        Some(v) if v.to_str().unwrap_or("").starts_with("Bearer ") => Ok(next.run(req).await),
        _ => Err(crate::ApiError::Unauthorized),
    }
}
