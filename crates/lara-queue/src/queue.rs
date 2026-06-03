use std::sync::Arc;
use crate::{
    drivers::QueueDriver,
    job::{Job, JobPayload},
};
use anyhow::Result;

/// Thin façade over a `QueueDriver`.
pub struct Queue {
    driver: Arc<dyn QueueDriver>,
}

impl Queue {
    pub fn new(driver: impl QueueDriver + 'static) -> Self {
        Self { driver: Arc::new(driver) }
    }

    /// Push a job onto the queue.
    pub async fn push<J: Job>(&self, job: &J) -> Result<String> {
        let payload = JobPayload::new(job)?;
        self.driver.push(payload).await
    }

    /// Push with a delay.
    pub async fn later<J: Job>(&self, job: &J, delay_secs: u64) -> Result<String> {
        let mut payload = JobPayload::new(job)?;
        payload.available_at = chrono::Utc::now().timestamp() + delay_secs as i64;
        self.driver.push(payload).await
    }

    /// Pop the next available job.
    pub async fn pop(&self, queue_name: &str) -> Result<Option<JobPayload>> {
        self.driver.pop(queue_name).await
    }

    pub async fn ack(&self, payload: &JobPayload) -> Result<()> {
        self.driver.ack(payload).await
    }

    pub async fn fail(&self, payload: &JobPayload, error: &str) -> Result<()> {
        self.driver.fail(payload, error).await
    }

    pub async fn release(&self, payload: &JobPayload, delay_secs: u64) -> Result<()> {
        self.driver.release(payload, delay_secs).await
    }

    pub fn driver_name(&self) -> &'static str {
        self.driver.driver_name()
    }
}

/// Multi-queue manager.
pub struct QueueManager {
    default_queue: Queue,
    named_queues: std::collections::HashMap<String, Queue>,
}

impl QueueManager {
    pub fn new(default_driver: impl QueueDriver + 'static) -> Self {
        Self {
            default_queue: Queue::new(default_driver),
            named_queues: std::collections::HashMap::new(),
        }
    }

    pub fn add_queue(mut self, name: &str, driver: impl QueueDriver + 'static) -> Self {
        self.named_queues.insert(name.to_string(), Queue::new(driver));
        self
    }

    pub fn queue(&self, name: &str) -> &Queue {
        self.named_queues.get(name).unwrap_or(&self.default_queue)
    }

    pub fn default(&self) -> &Queue { &self.default_queue }

    /// Dispatch a job — uses the queue declared by `J::queue_name()`.
    pub async fn dispatch<J: Job>(&self, job: J) -> Result<String> {
        let queue = self.queue(J::queue_name());
        // In sync mode (SyncDriver), execute immediately
        if queue.driver.driver_name() == "sync" {
            job.handle().await?;
            return Ok("sync".to_string());
        }
        queue.push(&job).await
    }
}
