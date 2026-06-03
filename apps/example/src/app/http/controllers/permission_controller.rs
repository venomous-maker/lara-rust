use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::bootstrap::app::AppState;

/// GET /api/permissions
pub async fn index(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.permissions.all().await {
        Ok(perms) => Json(json!({ "data": perms })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": e.to_string() }))).into_response(),
    }
}

/// GET /api/permissions/:id
pub async fn show(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match state.permissions.find(id).await {
        Ok(p) => Json(json!({ "data": p })).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, Json(json!({ "message": "permission not found" }))).into_response(),
    }
}
