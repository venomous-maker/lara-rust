use std::sync::Arc;
use anyhow::{anyhow, Result};
use lara_db::ModelTrait;
use lara_events::SharedDispatcher;

use crate::app::events::{UserLoggedIn, UserRegistered};
use crate::app::models::User;
use crate::app::services::{token_service::TokenService, user_service::UserService};

/// Authentication flows: register, login, identity.
///
/// **Dependency injection:** composes three other singletons —
/// [`UserService`], [`TokenService`], and the shared [`EventDispatcher`].
pub struct AuthService {
    users: Arc<UserService>,
    tokens: Arc<TokenService>,
    events: SharedDispatcher,
}

#[derive(Debug)]
pub struct AuthResult {
    pub user: User,
    pub token: String,
}

impl AuthService {
    pub fn new(
        users: Arc<UserService>,
        tokens: Arc<TokenService>,
        events: SharedDispatcher,
    ) -> Self {
        Self { users, tokens, events }
    }

    /// Register a new user, emit `UserRegistered`, and return an auth token.
    pub async fn register(&self, name: String, email: String, password: String) -> Result<AuthResult> {
        let user = self.users.create(name, email, password).await?;
        let id = user.id.unwrap_or_default();

        // Assign the default "user" role (best-effort).
        let _ = self.users.assign_role(&user, "user").await;

        self.events.dispatch(UserRegistered {
            user_id: id,
            name: user.name.clone(),
            email: user.email.clone(),
        }).await;

        let token = self.tokens.issue(id, &user.email)?;
        Ok(AuthResult { user, token })
    }

    /// Authenticate by email + password, emit `UserLoggedIn`, return a token.
    pub async fn login(&self, email: &str, password: &str, ip: String) -> Result<AuthResult> {
        let user = User::query()
            .where_eq("email", email)
            .first()
            .await
            .map_err(|e| anyhow!("query failed: {}", e))?
            .ok_or_else(|| anyhow!("invalid credentials"))?;

        if !UserService::verify_password(password, &user.password) {
            return Err(anyhow!("invalid credentials"));
        }
        if user.status != "active" {
            return Err(anyhow!("account is {}", user.status));
        }

        let id = user.id.unwrap_or_default();
        self.events.dispatch(UserLoggedIn {
            user_id: id,
            email: user.email.clone(),
            ip,
        }).await;

        let token = self.tokens.issue(id, &user.email)?;
        Ok(AuthResult { user, token })
    }

    /// Resolve the authenticated user from a bearer token.
    pub async fn me(&self, token: &str) -> Result<User> {
        let claims = self.tokens.verify(token)?;
        let id: i64 = claims.sub.parse().map_err(|_| anyhow!("bad subject"))?;
        self.users.find(id).await
    }

    pub fn tokens(&self) -> &Arc<TokenService> {
        &self.tokens
    }
}
