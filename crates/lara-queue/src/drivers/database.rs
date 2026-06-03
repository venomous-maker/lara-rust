use async_trait::async_trait;
use crate::job::JobPayload;
use super::QueueDriver;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::VecDeque;

/// In-process database-backed queue (uses an in-memory store for now;
/// production would persist to the `jobs` table via sqlx).
pub struct DatabaseDriver {
    queue: Arc<Mutex<VecDeque<JobPayload>>>,
    failed: Arc<Mutex<Vec<(JobPayload, String)>>>,
}

impl DatabaseDriver {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            failed: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl QueueDriver for DatabaseDriver {
    async fn push(&self, payload: JobPayload) -> Result<String> {
        let id = payload.id.clone();
        self.queue.lock().await.push_back(payload);
        Ok(id)
    }

    async fn pop(&self, queue: &str) -> Result<Option<JobPayload>> {
        let mut q = self.queue.lock().await;
        let pos = q.iter().position(|p| p.queue == queue);
        Ok(pos.map(|i| q.remove(i).unwrap()))
    }

    async fn ack(&self, _payload: &JobPayload) -> Result<()> { Ok(()) }

    async fn fail(&self, payload: &JobPayload, error: &str) -> Result<()> {
        self.failed.lock().await.push((payload.clone(), error.to_string()));
        Ok(())
    }

    async fn release(&self, payload: &JobPayload, _delay: u64) -> Result<()> {
        let mut p = payload.clone();
        p.attempts += 1;
        self.queue.lock().await.push_back(p);
        Ok(())
    }

    fn driver_name(&self) -> &'static str { "database" }
}
