use lara_db::ModelTrait;
use lara_derive::Model;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use super::user::User;

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[lara(table = "user_profiles", primary_key = "id")]
pub struct UserProfile {
    pub id: Option<i64>,
    pub user_id: i64,
    pub bio: Option<String>,
    pub avatar: Option<String>,
    pub website: Option<String>,
    pub twitter: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl UserProfile {
    pub fn user(&self) -> lara_db::BelongsTo<UserProfile, User> {
        self.belongs_to::<User>("user_id", None)
    }
}
