use std::net::SocketAddr;
use axum::Router;
use tokio::net::TcpListener;

use crate::{
    config::Config,
    container::{Container, SharedContainer, make_container},
    error::{CoreError, Result},
    service_provider::{ServiceProvider, ProviderRegistry},
};

/// The central Lara application.
pub struct Application {
    container: SharedContainer,
    config: Config,
    providers: ProviderRegistry,
    router: Router,
    booted: bool,
}

impl Application {
    pub fn new() -> Self {
        Self {
            container: make_container(),
            config: Config::new(),
            providers: ProviderRegistry::new(),
            router: Router::new(),
            booted: false,
        }
    }

    /// Register a configuration value.
    pub fn configure(mut self, key: &str, value: impl Into<serde_json::Value>) -> Self {
        self.config.set(key, value);
        self
    }

    /// Load configuration from a JSON file.
    pub fn configure_from_file(mut self, path: &str) -> Result<Self> {
        let cfg = Config::from_json_file(path)?;
        self.config.merge(cfg);
        Ok(self)
    }

    /// Register a service provider.
    pub fn register(mut self, provider: impl ServiceProvider + 'static) -> Self {
        self.providers.register_provider(Box::new(provider));
        self
    }

    /// Set the Axum router.
    pub fn with_router(mut self, router: Router) -> Self {
        self.router = router;
        self
    }

    /// Merge routes into the existing router.
    pub fn merge_router(mut self, router: Router) -> Self {
        self.router = self.router.merge(router);
        self
    }

    /// Run the register→boot lifecycle for all providers.
    pub async fn boot(&mut self) -> Result<()> {
        if self.booted {
            return Err(CoreError::Application("Application already booted".into()));
        }

        let mut container = self.container.write().await;
        self.providers.run_register(&mut container, &self.config).await?;
        drop(container);

        let container = self.container.read().await;
        self.providers.run_boot(&container, &self.config).await?;
        drop(container);

        self.booted = true;
        Ok(())
    }

    /// Bind the application's shared container into the Axum router state and start serving.
    pub async fn serve(mut self, addr: SocketAddr) -> Result<()> {
        if !self.booted {
            self.boot().await?;
        }

        // The application router is already built with its own state;
        // the shared container is available via extension if needed.
        let router = self.router;

        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| CoreError::Application(e.to_string()))?;

        tracing::info!("Lara listening on {}", addr);

        axum::serve(listener, router)
            .await
            .map_err(|e| CoreError::Application(e.to_string()))
    }

    /// Access the shared container (after booting).
    pub fn container(&self) -> SharedContainer {
        self.container.clone()
    }

    /// Access configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}
