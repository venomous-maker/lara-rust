use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use lara_db::ModelTrait;
use lara_router::Validated;
use serde_json::json;

use crate::app::http::middleware::{ensure_role, AuthUser};
use crate::app::http::requests::{StoreRoleRequest, SyncPermissionsRequest};
use crate::bootstrap::app::AppState;

/// GET /api/roles
pub async fn index(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.roles.all().await {
        Ok(roles) => Json(json!({ "data": roles })).into_response(),
        Err(e) => err(e),
    }
}

/// POST /api/roles  (admin only)
pub async fn store(
    State(state): State<Arc<AppState>>,
    AuthUser(actor): AuthUser,
    Validated(body): Validated<StoreRoleRequest>,
) -> impl IntoResponse {
    if let Err(e) = ensure_role(&actor, "admin").await {
        return forbidden(&e.to_string());
    }
    match state.roles.create(body.name, body.slug, body.description).await {
        Ok(role) => (StatusCode::CREATED, Json(json!({ "data": role }))).into_response(),
        Err(e) => err(e),
    }
}

/// PUT /api/roles/:id/permissions  (admin only)
pub async fn sync_permissions(
    State(state): State<Arc<AppState>>,
    AuthUser(actor): AuthUser,
    Path(id): Path<i64>,
    Validated(body): Validated<SyncPermissionsRequest>,
) -> impl IntoResponse {
    if let Err(e) = ensure_role(&actor, "admin").await {
        return forbidden(&e.to_string());
    }
    match state.roles.sync_permissions(id, &body.permission_ids).await {
        Ok(_) => Json(json!({ "message": "permissions synced" })).into_response(),
        Err(e) => err(e),
    }
}

/// DELETE /api/roles/:id  (admin only)
pub async fn destroy(
    State(state): State<Arc<AppState>>,
    AuthUser(actor): AuthUser,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    if let Err(e) = ensure_role(&actor, "admin").await {
        return forbidden(&e.to_string());
    }
    match state.roles.delete(id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => err(e),
    }
}

fn err(e: anyhow::Error) -> axum::response::Response {
    (StatusCode::UNPROCESSABLE_ENTITY, Json(json!({ "message": e.to_string() }))).into_response()
}
fn forbidden(msg: &str) -> axum::response::Response {
    (StatusCode::FORBIDDEN, Json(json!({ "message": msg }))).into_response()
}
