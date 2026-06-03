use chrono::{DateTime, Utc};
use crate::model::Model;

/// Extension for models with `created_at` / `updated_at`.
pub trait Timestamps: Model {
    fn created_at(&self) -> Option<DateTime<Utc>>;
    fn updated_at(&self) -> Option<DateTime<Utc>>;
}
