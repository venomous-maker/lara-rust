use std::sync::Arc;
use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};
use serde_json::json;

use crate::bootstrap::app::AppState;

pub fn web_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(welcome))
        .route("/health", get(health))
}

async fn welcome(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(json!({
        "app": state.config.name,
        "env": state.config.env,
        "message": "Welcome to Lara Rust 🦀",
    }))
}

async fn health() -> impl IntoResponse {
    Json(json!({ "status": "ok", "version": env!("CARGO_PKG_VERSION") }))
}
