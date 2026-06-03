use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use lara_db::ModelTrait;
use lara_router::Validated;
use serde::Deserialize;
use serde_json::json;

use crate::app::http::middleware::{ensure_role, AuthUser};
use crate::app::http::requests::{StoreUserRequest, UpdateUserRequest};
use crate::bootstrap::app::AppState;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default = "page1")] pub page: u64,
    #[serde(default = "per15")] pub per_page: u64,
}
fn page1() -> u64 { 1 }
fn per15() -> u64 { 15 }

/// GET /api/users
pub async fn index(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListQuery>,
) -> impl IntoResponse {
    match state.users.paginate(q.page, q.per_page).await {
        Ok(p) => Json(json!({
            "data": p.data.iter().filter_map(|u| u.to_json_public().ok()).collect::<Vec<_>>(),
            "meta": {
                "total": p.total, "per_page": p.per_page,
                "current_page": p.current_page, "last_page": p.last_page,
            }
        })).into_response(),
        Err(e) => server_error(e),
    }
}

/// GET /api/users/:id
pub async fn show(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match state.users.find(id).await {
        Ok(user) => Json(json!({ "data": user.to_json_public().unwrap_or_default() })).into_response(),
        Err(_) => not_found("user"),
    }
}

/// POST /api/users  (admin only)
pub async fn store(
    State(state): State<Arc<AppState>>,
    AuthUser(actor): AuthUser,
    Validated(body): Validated<StoreUserRequest>,
) -> impl IntoResponse {
    if let Err(e) = ensure_role(&actor, "admin").await {
        return forbidden(&e.to_string());
    }
    match state.users.create(body.name, body.email, body.password).await {
        Ok(user) => (
            StatusCode::CREATED,
            Json(json!({ "data": user.to_json_public().unwrap_or_default() })),
        ).into_response(),
        Err(e) => unprocessable(e),
    }
}

/// PUT /api/users/:id  (admin only)
pub async fn update(
    State(state): State<Arc<AppState>>,
    AuthUser(actor): AuthUser,
    Path(id): Path<i64>,
    Validated(body): Validated<UpdateUserRequest>,
) -> impl IntoResponse {
    if let Err(e) = ensure_role(&actor, "admin").await {
        return forbidden(&e.to_string());
    }
    match state.users.update(id, body.name, body.email, body.status).await {
        Ok(user) => Json(json!({ "data": user.to_json_public().unwrap_or_default() })).into_response(),
        Err(e) => unprocessable(e),
    }
}

/// DELETE /api/users/:id  (admin only)
pub async fn destroy(
    State(state): State<Arc<AppState>>,
    AuthUser(actor): AuthUser,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    if let Err(e) = ensure_role(&actor, "admin").await {
        return forbidden(&e.to_string());
    }
    match state.users.delete(id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => unprocessable(e),
    }
}

// ── response helpers ──────────────────────────────────────────────────────────

fn server_error(e: anyhow::Error) -> axum::response::Response {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": e.to_string() }))).into_response()
}
fn unprocessable(e: anyhow::Error) -> axum::response::Response {
    (StatusCode::UNPROCESSABLE_ENTITY, Json(json!({ "message": e.to_string() }))).into_response()
}
fn not_found(what: &str) -> axum::response::Response {
    (StatusCode::NOT_FOUND, Json(json!({ "message": format!("{} not found", what) }))).into_response()
}
fn forbidden(msg: &str) -> axum::response::Response {
    (StatusCode::FORBIDDEN, Json(json!({ "message": msg }))).into_response()
}
