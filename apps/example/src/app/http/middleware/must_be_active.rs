use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use serde_json::json;

use super::authenticate::AuthUser;

/// Middleware: reject requests from users whose status is not `active`.
/// Must run *after* [`authenticate`](super::authenticate::authenticate).
pub async fn must_be_active(req: Request, next: Next) -> Response {
    if let Some(AuthUser(user)) = req.extensions().get::<AuthUser>() {
        if user.status != "active" {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({ "message": format!("Your account is {}.", user.status) })),
            ).into_response();
        }
    }
    next.run(req).await
}
