use async_trait::async_trait;
use std::sync::Arc;

use lara_queue::{drivers::{database::DatabaseDriver, sync::SyncDriver}, QueueManager};
use lara_scheduler::{Schedule, Scheduler};

use crate::config::queue::QueueDriver;
use crate::app::jobs::CleanupJob;
use super::{Builder, ServiceProvider};

/// Configures the queue manager and schedules recurring jobs.
pub struct QueueServiceProvider;

#[async_trait]
impl ServiceProvider for QueueServiceProvider {
    fn name(&self) -> &'static str { "QueueServiceProvider" }

    async fn register(&self, _builder: &mut Builder) -> anyhow::Result<()> {
        // The QueueManager is built per-driver; in a full app it would be a singleton.
        Ok(())
    }

    async fn boot(&self, builder: &Builder) -> anyhow::Result<()> {
        // Build the queue manager from config.
        let _manager = match builder.queue_config.driver {
            QueueDriver::Database => QueueManager::new(DatabaseDriver::new()),
            _                     => QueueManager::new(SyncDriver),
        };

        // Define the schedule (cleanup nightly at 02:00). The scheduler runs
        // in its own task spawned from `main` / the `schedule:run` command.
        let scheduler = Scheduler::new().call(
            "nightly-cleanup",
            Schedule::DailyAt { hour: 2, minute: 0 },
            || async {
                CleanupJob { older_than_days: 30 };
                tracing::info!("scheduled CleanupJob queued");
                Ok(())
            },
        );
        // Spawn the scheduler loop in the background.
        tokio::spawn(scheduler.run());

        tracing::info!("queue configured + scheduler started");
        let _ = Arc::new(()); // keep imports tidy
        Ok(())
    }
}
