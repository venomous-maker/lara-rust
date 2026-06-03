use lara_db::ModelTrait;
use lara_derive::Model;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use super::user::User;

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[lara(table = "files", primary_key = "id", soft_deletes)]
pub struct File {
    pub id: Option<i64>,
    pub user_id: Option<i64>,
    pub original_name: String,
    pub stored_name: String,
    pub path: String,
    pub mime_type: String,
    pub size: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl File {
    /// The user who uploaded this file.
    pub fn owner(&self) -> lara_db::BelongsTo<File, User> {
        self.belongs_to::<User>("user_id", None)
    }

    pub fn human_size(&self) -> String {
        let bytes = self.size as f64;
        const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes;
        let mut unit = 0;
        while size >= 1024.0 && unit < UNITS.len() - 1 {
            size /= 1024.0;
            unit += 1;
        }
        format!("{:.2} {}", size, UNITS[unit])
    }
}
