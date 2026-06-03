use lara_db::ModelTrait;
use lara_derive::Model;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use super::role::Role;

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[lara(table = "permissions", primary_key = "id", soft_deletes)]
pub struct Permission {
    pub id: Option<i64>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Permission {
    /// Roles that have this permission (many-to-many through `permission_role`).
    pub fn roles(&self) -> lara_db::BelongsToMany<Permission, Role> {
        self.belongs_to_many::<Role>("permission_role", "permission_id", "role_id", None)
    }
}
