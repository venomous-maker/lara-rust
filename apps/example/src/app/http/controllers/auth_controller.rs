use std::sync::Arc;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use lara_db::ModelTrait;
use lara_router::Validated;
use serde_json::json;

use crate::app::http::middleware::AuthUser;
use crate::app::http::requests::{LoginRequest, RegisterRequest};
use crate::bootstrap::app::AppState;

/// POST /api/auth/register
pub async fn register(
    State(state): State<Arc<AppState>>,
    Validated(body): Validated<RegisterRequest>,
) -> impl IntoResponse {
    match state.auth.register(body.name, body.email, body.password).await {
        Ok(result) => (
            StatusCode::CREATED,
            Json(json!({
                "user": result.user.to_json_public().unwrap_or_default(),
                "token": result.token,
            })),
        ).into_response(),
        Err(e) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({ "message": e.to_string() })),
        ).into_response(),
    }
}

/// POST /api/auth/login
pub async fn login(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Validated(body): Validated<LoginRequest>,
) -> impl IntoResponse {
    let ip = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    match state.auth.login(&body.email, &body.password, ip).await {
        Ok(result) => Json(json!({
            "user": result.user.to_json_public().unwrap_or_default(),
            "token": result.token,
        })).into_response(),
        Err(e) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "message": e.to_string() })),
        ).into_response(),
    }
}

/// GET /api/auth/me  (auth required)
pub async fn me(AuthUser(user): AuthUser) -> impl IntoResponse {
    Json(json!({ "user": user.to_json_public().unwrap_or_default() }))
}
