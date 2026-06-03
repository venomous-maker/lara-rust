use anyhow::{anyhow, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use lara_db::{ModelTrait, Paginator};
use lara_events::SharedDispatcher;

use crate::app::events::UserStatusChanged;
use crate::app::models::{Role, User};

/// Handles user persistence and role management.
///
/// **Dependency injection:** receives the shared [`EventDispatcher`] at
/// construction time so it can emit domain events.
pub struct UserService {
    events: SharedDispatcher,
}

impl UserService {
    pub fn new(events: SharedDispatcher) -> Self {
        Self { events }
    }

    pub async fn paginate(&self, page: u64, per_page: u64) -> Result<Paginator<User>> {
        User::query()
            .order_by_desc("created_at")
            .paginate(per_page, page)
            .await
            .map_err(|e| anyhow!("query failed: {}", e))
    }

    pub async fn find(&self, id: i64) -> Result<User> {
        User::find_or_fail(id)
            .await
            .map_err(|_| anyhow!("user #{} not found", id))
    }

    pub async fn email_exists(&self, email: &str) -> Result<bool> {
        User::query()
            .where_eq("email", email)
            .exists()
            .await
            .map_err(|e| anyhow!("query failed: {}", e))
    }

    /// Create a user with a securely hashed password.
    pub async fn create(&self, name: String, email: String, password: String) -> Result<User> {
        if self.email_exists(&email).await? {
            return Err(anyhow!("a user with this email already exists"));
        }
        let hashed = Self::hash_password(&password)?;
        User::create(User {
            name,
            email,
            password: hashed,
            status: "active".into(),
            ..Default::default()
        })
        .await
        .map_err(|e| anyhow!("create failed: {}", e))
    }

    pub async fn update(
        &self,
        id: i64,
        name: Option<String>,
        email: Option<String>,
        status: Option<String>,
    ) -> Result<User> {
        let mut user = self.find(id).await?;
        if let Some(n) = name { user.name = n; }
        if let Some(e) = email { user.email = e; }
        if let Some(s) = status {
            if s != user.status {
                let old = user.status.clone();
                user.status = s.clone();
                self.events.dispatch(UserStatusChanged {
                    user_id: id,
                    old_status: old,
                    new_status: s,
                }).await;
            }
        }
        user.save().await.map_err(|e| anyhow!("update failed: {}", e))?;
        Ok(user)
    }

    pub async fn delete(&self, id: i64) -> Result<()> {
        let user = self.find(id).await?;
        user.delete().await.map_err(|e| anyhow!("delete failed: {}", e))
    }

    pub async fn assign_role(&self, user: &User, role_slug: &str) -> Result<()> {
        let role = Role::query()
            .where_eq("slug", role_slug)
            .first_or_fail()
            .await
            .map_err(|_| anyhow!("role `{}` not found", role_slug))?;
        let role_id = role.primary_key_value().map_err(|e| anyhow!("{}", e))?;
        user.roles()
            .attach(&[role_id])
            .await
            .map_err(|e| anyhow!("attach role failed: {}", e))
    }

    pub async fn roles_of(&self, user: &User) -> Result<Vec<Role>> {
        user.roles().get().await.map_err(|e| anyhow!("{}", e))
    }

    // ── password helpers (also used by AuthService) ──────────────────────────

    pub fn hash_password(plain: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        Ok(Argon2::default()
            .hash_password(plain.as_bytes(), &salt)
            .map_err(|e| anyhow!("hash error: {}", e))?
            .to_string())
    }

    pub fn verify_password(plain: &str, hash: &str) -> bool {
        match PasswordHash::new(hash) {
            Ok(parsed) => Argon2::default()
                .verify_password(plain.as_bytes(), &parsed)
                .is_ok(),
            Err(_) => false,
        }
    }
}
