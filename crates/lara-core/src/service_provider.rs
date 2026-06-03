use async_trait::async_trait;
use crate::{container::Container, config::Config, error::Result};

/// Every Lara package exposes itself through a ServiceProvider.
/// Providers are registered in two phases:
///   1. `register` — bind things into the container (no cross-provider deps yet).
///   2. `boot`     — all providers are registered; safe to call other bindings.
#[async_trait]
pub trait ServiceProvider: Send + Sync {
    /// Name used for diagnostics.
    fn name(&self) -> &'static str;

    /// Bind services into the container.
    async fn register(&self, container: &mut Container, config: &Config) -> Result<()>;

    /// Called after all providers have been registered.
    async fn boot(&self, container: &Container, config: &Config) -> Result<()> {
        let _ = (container, config);
        Ok(())
    }

    /// If `true` this provider is deferred and only booted when its services are first resolved.
    fn is_deferred(&self) -> bool {
        false
    }

    /// The abstract keys this deferred provider provides (used when `is_deferred` is true).
    fn provides(&self) -> Vec<&'static str> {
        vec![]
    }
}

/// Collect providers and drive the register→boot lifecycle.
pub struct ProviderRegistry {
    providers: Vec<Box<dyn ServiceProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self { providers: Vec::new() }
    }

    pub fn register_provider(&mut self, provider: Box<dyn ServiceProvider>) {
        self.providers.push(provider);
    }

    pub async fn run_register(&self, container: &mut Container, config: &Config) -> Result<()> {
        for p in &self.providers {
            if !p.is_deferred() {
                p.register(container, config).await?;
            }
        }
        Ok(())
    }

    pub async fn run_boot(&self, container: &Container, config: &Config) -> Result<()> {
        for p in &self.providers {
            if !p.is_deferred() {
                p.boot(container, config).await?;
            }
        }
        Ok(())
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
