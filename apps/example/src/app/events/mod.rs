use lara_events::Event;
use serde::{Deserialize, Serialize};

/// Fired right after a user successfully registers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRegistered {
    pub user_id: i64,
    pub name: String,
    pub email: String,
}
impl Event for UserRegistered {}

/// Fired when a user logs in.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLoggedIn {
    pub user_id: i64,
    pub email: String,
    pub ip: String,
}
impl Event for UserLoggedIn {}

/// Fired when a user logs out.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLoggedOut {
    pub user_id: i64,
}
impl Event for UserLoggedOut {}

/// Fired when a password reset is requested.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetRequested {
    pub user_id: i64,
    pub email: String,
    pub token: String,
}
impl Event for PasswordResetRequested {}

/// Fired when a user's status changes (active/inactive/banned).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatusChanged {
    pub user_id: i64,
    pub old_status: String,
    pub new_status: String,
}
impl Event for UserStatusChanged {}
