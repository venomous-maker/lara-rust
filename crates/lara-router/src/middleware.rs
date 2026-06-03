use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use serde_json::json;

/// Log every incoming request.
pub async fn log_request(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let uri    = req.uri().clone();
    tracing::info!("{} {}", method, uri);
    let resp = next.run(req).await;
    tracing::info!("{} {} -> {}", method, uri, resp.status());
    resp
}

/// Reject requests where the body exceeds a byte limit.
pub async fn body_limit(
    req: Request,
    next: Next,
) -> Response {
    // Actual body limiting is handled by tower_http::limit::RequestBodyLimitLayer
    next.run(req).await
}

/// Naive JSON-only middleware: returns 415 for non-JSON content-type on POST/PUT/PATCH.
pub async fn require_json(req: Request, next: Next) -> Response {
    use axum::http::Method;
    let needs_body = matches!(req.method(), &Method::POST | &Method::PUT | &Method::PATCH);
    if needs_body {
        let ct = req.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if !ct.contains("application/json") {
            return (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                Json(json!({"error": "Content-Type must be application/json"})),
            ).into_response();
        }
    }
    next.run(req).await
}
