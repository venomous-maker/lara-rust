use async_trait::async_trait;
use lara_events::make_dispatcher;

use crate::app::listeners;
use super::{Builder, ServiceProvider};

/// Registers the event dispatcher and wires every listener.
pub struct EventServiceProvider;

#[async_trait]
impl ServiceProvider for EventServiceProvider {
    fn name(&self) -> &'static str { "EventServiceProvider" }

    async fn register(&self, builder: &mut Builder) -> anyhow::Result<()> {
        // Singleton dispatcher.
        builder.events = Some(make_dispatcher());
        Ok(())
    }

    async fn boot(&self, builder: &Builder) -> anyhow::Result<()> {
        // Now that the dispatcher exists, attach all listeners.
        let dispatcher = builder.events();
        listeners::register(&dispatcher).await;
        tracing::info!("event listeners registered");
        Ok(())
    }
}
