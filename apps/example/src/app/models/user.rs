use lara_db::{ModelTrait, QueryBuilder};
use lara_derive::Model;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use super::role::Role;
use super::user_profile::UserProfile;

#[derive(Debug, Clone, Default, Serialize, Deserialize, Model)]
#[lara(table = "users", primary_key = "id", soft_deletes)]
pub struct User {
    pub id: Option<i64>,
    pub name: String,
    pub email: String,
    #[lara(hidden)]
    pub password: String,
    pub status: String,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl User {
    pub fn roles(&self) -> lara_db::BelongsToMany<User, Role> {
        self.belongs_to_many::<Role>("role_user", "user_id", "role_id", None)
    }

    pub fn profile(&self) -> lara_db::HasOne<User, UserProfile> {
        self.has_one::<UserProfile>("user_id", None)
    }

    /// Scope: only active users.
    pub fn active() -> QueryBuilder<User> {
        User::query().where_eq("status", "active")
    }

    /// Scope: email-verified users.
    pub fn verified() -> QueryBuilder<User> {
        User::query().where_not_null("email_verified_at")
    }

    /// Full-text search (name LIKE % or email =).
    pub fn search(term: &str) -> QueryBuilder<User> {
        User::query()
            .where_like("name", format!("%{}%", term))
            .or_where_eq("email", term)
    }

    pub fn is_active(&self) -> bool { self.status == "active" }
}
