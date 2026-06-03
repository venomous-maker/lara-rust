//! # Service Providers
//!
//! Providers are the central place where the application is bootstrapped.
//! Each provider has two phases:
//!
//! * [`register`](ServiceProvider::register) — bind singletons into the [`Builder`].
//!   Never resolve another provider's bindings here.
//! * [`boot`](ServiceProvider::boot) — everything is registered; safe to use other
//!   singletons (wire listeners, schedule jobs, etc).
//!
//! The [`Builder`] *is* the dependency-injection container: providers populate its
//! typed slots, and once all providers have run it is frozen into the immutable
//! [`AppState`](crate::bootstrap::app::AppState) that every HTTP handler receives.

pub mod app_service_provider;
pub mod cache_service_provider;
pub mod event_service_provider;
pub mod queue_service_provider;
pub mod route_service_provider;

use std::sync::Arc;
use async_trait::async_trait;

use lara_cache::{RateLimiter, SharedCache};
use lara_events::SharedDispatcher;

use crate::config::{AppConfig, AuthConfig, CacheConfig, QueueConfig};
use crate::app::services::{
    AuthService, FileService, PermissionService, RoleService, TokenService, UserService,
};

/// The DI container assembled during bootstrap.
/// Typed slots are filled by providers in dependency order.
pub struct Builder {
    // Configuration (available to every provider)
    pub app_config: AppConfig,
    pub auth_config: AuthConfig,
    pub cache_config: CacheConfig,
    pub queue_config: QueueConfig,

    // Singletons (filled by providers)
    pub cache: Option<SharedCache>,
    pub events: Option<SharedDispatcher>,
    pub limiter: Option<Arc<RateLimiter>>,
    pub tokens: Option<Arc<TokenService>>,
    pub users: Option<Arc<UserService>>,
    pub auth: Option<Arc<AuthService>>,
    pub roles: Option<Arc<RoleService>>,
    pub permissions: Option<Arc<PermissionService>>,
    pub files: Option<Arc<FileService>>,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            app_config: AppConfig::default(),
            auth_config: AuthConfig::default(),
            cache_config: CacheConfig::default(),
            queue_config: QueueConfig::default(),
            cache: None,
            events: None,
            limiter: None,
            tokens: None,
            users: None,
            auth: None,
            roles: None,
            permissions: None,
            files: None,
        }
    }

    // Typed resolvers — panic with a clear message if a provider forgot to bind.
    pub fn events(&self) -> SharedDispatcher {
        self.events.clone().expect("EventServiceProvider must register the dispatcher")
    }
    pub fn cache(&self) -> SharedCache {
        self.cache.clone().expect("CacheServiceProvider must register the cache")
    }
    pub fn tokens(&self) -> Arc<TokenService> {
        self.tokens.clone().expect("AppServiceProvider must register TokenService")
    }
    pub fn users(&self) -> Arc<UserService> {
        self.users.clone().expect("AppServiceProvider must register UserService")
    }
}

impl Default for Builder {
    fn default() -> Self { Self::new() }
}

/// A service provider — registers + boots a slice of the application.
#[async_trait]
pub trait ServiceProvider: Send + Sync {
    fn name(&self) -> &'static str;

    /// Bind singletons into the builder.
    async fn register(&self, builder: &mut Builder) -> anyhow::Result<()>;

    /// Post-registration wiring (listeners, schedules, …).
    async fn boot(&self, _builder: &Builder) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Run every provider through register → boot, in declaration order.
pub async fn run(builder: &mut Builder, providers: &[Box<dyn ServiceProvider>]) -> anyhow::Result<()> {
    for p in providers {
        tracing::debug!("registering provider: {}", p.name());
        p.register(builder).await?;
    }
    for p in providers {
        tracing::debug!("booting provider: {}", p.name());
        p.boot(builder).await?;
    }
    Ok(())
}

/// The ordered provider stack for this application.
pub fn providers() -> Vec<Box<dyn ServiceProvider>> {
    vec![
        Box::new(cache_service_provider::CacheServiceProvider),
        Box::new(event_service_provider::EventServiceProvider),
        Box::new(app_service_provider::AppServiceProvider),
        Box::new(queue_service_provider::QueueServiceProvider),
        Box::new(route_service_provider::RouteServiceProvider),
    ]
}
