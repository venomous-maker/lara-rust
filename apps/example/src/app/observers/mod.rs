//! Model observers.
//!
//! Observers react to a model's lifecycle events. `lara-db` exposes the
//! [`ModelObserver`](lara_db::model::events::ModelObserver) trait; an app
//! registers an observer to run side-effects on create/update/delete.

use lara_db::model::events::ModelObserver;
use serde_json::Value;

/// Observes [`User`](crate::app::models::User) lifecycle events.
pub struct UserObserver;

impl ModelObserver for UserObserver {
    fn creating(&self, data: &Value) -> bool {
        tracing::debug!("UserObserver::creating {}", data.get("email").and_then(|v| v.as_str()).unwrap_or("?"));
        true // return false to abort the create
    }

    fn created(&self, data: &Value) {
        tracing::info!("UserObserver::created user #{}", data.get("id").and_then(|v| v.as_i64()).unwrap_or(0));
    }

    fn deleting(&self, data: &Value) -> bool {
        tracing::warn!("UserObserver::deleting user #{}", data.get("id").and_then(|v| v.as_i64()).unwrap_or(0));
        true
    }
}
