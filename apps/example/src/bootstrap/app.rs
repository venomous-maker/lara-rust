use std::sync::Arc;

use lara_cache::{RateLimiter, SharedCache};
use lara_db::Db;
use lara_events::SharedDispatcher;
use lara_mail::Mailer;

use crate::config::{self, AppConfig};
use crate::app::providers;
use crate::app::services::{
    AuthService, FileService, PermissionService, RoleService, TokenService, UserService,
};

/// The immutable, shared application state injected into every Axum handler
/// via `State<Arc<AppState>>`. It is the fully-resolved DI container: all
/// singletons have been wired by the service providers before this is built.
#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub cache: SharedCache,
    pub events: SharedDispatcher,
    pub limiter: Arc<RateLimiter>,

    // Domain service singletons
    pub auth: Arc<AuthService>,
    pub users: Arc<UserService>,
    pub roles: Arc<RoleService>,
    pub permissions: Arc<PermissionService>,
    pub files: Arc<FileService>,
    pub tokens: Arc<TokenService>,
}

impl AppState {
    /// Boot the whole application:
    ///   1. connect the database + mail (global facades),
    ///   2. run every service provider (register → boot),
    ///   3. freeze the resolved singletons into an `AppState`.
    pub async fn boot() -> anyhow::Result<Arc<Self>> {
        // ── Global facades ────────────────────────────────────────────────────
        Db::configure(config::default_connection())
            .await
            .map_err(|e| anyhow::anyhow!("database connection failed: {}", e))?;

        Mailer::configure(config::mail::mail_config())
            .map_err(|e| anyhow::anyhow!("mail config failed: {}", e))?;

        // ── Service providers (DI wiring) ─────────────────────────────────────
        let mut builder = providers::Builder::new();
        let stack = providers::providers();
        providers::run(&mut builder, &stack).await?;

        // ── Freeze into immutable state ───────────────────────────────────────
        let state = AppState {
            config: builder.app_config.clone(),
            cache: builder.cache.expect("cache bound"),
            events: builder.events.expect("events bound"),
            limiter: builder.limiter.expect("limiter bound"),
            auth: builder.auth.expect("auth bound"),
            users: builder.users.expect("users bound"),
            roles: builder.roles.expect("roles bound"),
            permissions: builder.permissions.expect("permissions bound"),
            files: builder.files.expect("files bound"),
            tokens: builder.tokens.expect("tokens bound"),
        };

        tracing::info!("application booted: {}", state.config.name);
        Ok(Arc::new(state))
    }
}
