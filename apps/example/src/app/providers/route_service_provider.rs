use async_trait::async_trait;
use super::{Builder, ServiceProvider};

/// In Laravel this loads route files. In Lara Rust the Axum router is assembled
/// in [`crate::routes`] and mounted in `main`; this provider documents that and
/// can apply route-wide configuration (rate-limit defaults, prefixes, etc).
pub struct RouteServiceProvider;

#[async_trait]
impl ServiceProvider for RouteServiceProvider {
    fn name(&self) -> &'static str { "RouteServiceProvider" }

    async fn register(&self, _builder: &mut Builder) -> anyhow::Result<()> {
        Ok(())
    }

    async fn boot(&self, _builder: &Builder) -> anyhow::Result<()> {
        tracing::info!("routes: web + /api mounted by the HTTP kernel");
        Ok(())
    }
}
