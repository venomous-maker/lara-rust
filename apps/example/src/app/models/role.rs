use lara_db::ModelTrait;
use lara_derive::Model;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use super::user::User;
use super::permission::Permission;

#[derive(Debug, Clone, Default, Serialize, Deserialize, Model)]
#[lara(table = "roles", primary_key = "id", soft_deletes)]
pub struct Role {
    pub id: Option<i64>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Role {
    /// Users that have this role.
    pub fn users(&self) -> lara_db::BelongsToMany<Role, User> {
        self.belongs_to_many::<User>("role_user", "role_id", "user_id", None)
    }

    /// Permissions granted to this role.
    pub fn permissions(&self) -> lara_db::BelongsToMany<Role, Permission> {
        self.belongs_to_many::<Permission>("permission_role", "role_id", "permission_id", None)
    }
}
