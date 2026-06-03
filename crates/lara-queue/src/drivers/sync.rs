use async_trait::async_trait;
use crate::job::JobPayload;
use super::QueueDriver;
use anyhow::Result;

/// Executes jobs synchronously (useful for testing / development).
pub struct SyncDriver;

#[async_trait]
impl QueueDriver for SyncDriver {
    async fn push(&self, payload: JobPayload) -> Result<String> {
        // In sync mode we immediately "complete" — actual execution is done
        // by QueueManager::dispatch which calls handle() directly.
        Ok(payload.id)
    }

    async fn pop(&self, _queue: &str) -> Result<Option<JobPayload>> {
        Ok(None)
    }

    async fn ack(&self, _payload: &JobPayload) -> Result<()> { Ok(()) }
    async fn fail(&self, _payload: &JobPayload, _error: &str) -> Result<()> { Ok(()) }
    async fn release(&self, _payload: &JobPayload, _delay: u64) -> Result<()> { Ok(()) }
    fn driver_name(&self) -> &'static str { "sync" }
}
