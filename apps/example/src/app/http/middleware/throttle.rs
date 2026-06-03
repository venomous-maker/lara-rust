use std::sync::Arc;
use std::time::Duration;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use serde_json::json;

use crate::bootstrap::app::AppState;

/// Middleware: limit each client IP to 60 requests / minute using the shared
/// [`RateLimiter`] singleton.
pub async fn throttle(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Response {
    let ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let allowed = state
        .limiter
        .attempt(&format!("http:{}", ip), 60, Duration::from_secs(60))
        .await;

    if allowed {
        next.run(req).await
    } else {
        (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({ "message": "Too Many Attempts." })),
        ).into_response()
    }
}
