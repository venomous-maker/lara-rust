pub mod sync;
pub mod database;
#[cfg(feature = "redis-driver")]
pub mod redis_driver;

use async_trait::async_trait;
use crate::job::JobPayload;
use anyhow::Result;

#[async_trait]
pub trait QueueDriver: Send + Sync {
    async fn push(&self, payload: JobPayload) -> Result<String>;
    async fn pop(&self, queue: &str) -> Result<Option<JobPayload>>;
    async fn ack(&self, payload: &JobPayload) -> Result<()>;
    async fn fail(&self, payload: &JobPayload, error: &str) -> Result<()>;
    async fn release(&self, payload: &JobPayload, delay_secs: u64) -> Result<()>;
    fn driver_name(&self) -> &'static str;
}
