// Redis queue driver — enabled via the `redis-driver` feature flag.
use async_trait::async_trait;
use crate::job::JobPayload;
use super::QueueDriver;
use anyhow::Result;

pub struct RedisQueueDriver {
    pool: deadpool_redis::Pool,
}

impl RedisQueueDriver {
    pub async fn new(url: &str) -> Result<Self> {
        let cfg = deadpool_redis::Config::from_url(url);
        let pool = cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))?;
        Ok(Self { pool })
    }
}

#[async_trait]
impl QueueDriver for RedisQueueDriver {
    async fn push(&self, payload: JobPayload) -> Result<String> {
        let mut conn = self.pool.get().await?;
        let id = payload.id.clone();
        let data = serde_json::to_string(&payload)?;
        redis::cmd("RPUSH")
            .arg(&payload.queue)
            .arg(&data)
            .query_async::<()>(&mut *conn)
            .await?;
        Ok(id)
    }

    async fn pop(&self, queue: &str) -> Result<Option<JobPayload>> {
        let mut conn = self.pool.get().await?;
        let result: Option<(String, String)> = redis::cmd("BLPOP")
            .arg(queue)
            .arg(0)
            .query_async(&mut *conn)
            .await?;
        Ok(result.and_then(|(_, data)| serde_json::from_str(&data).ok()))
    }

    async fn ack(&self, _payload: &JobPayload) -> Result<()> { Ok(()) }

    async fn fail(&self, payload: &JobPayload, error: &str) -> Result<()> {
        let mut conn = self.pool.get().await?;
        let mut p = payload.clone();
        let data = serde_json::json!({ "payload": p, "error": error });
        redis::cmd("RPUSH")
            .arg("failed")
            .arg(data.to_string())
            .query_async::<()>(&mut *conn)
            .await?;
        Ok(())
    }

    async fn release(&self, payload: &JobPayload, delay_secs: u64) -> Result<()> {
        // Re-push with a delay by using a ZADD with timestamp score
        let mut conn = self.pool.get().await?;
        let score = (chrono::Utc::now().timestamp() + delay_secs as i64) as f64;
        let data = serde_json::to_string(payload)?;
        redis::cmd("ZADD")
            .arg(format!("{}_delayed", payload.queue))
            .arg(score)
            .arg(&data)
            .query_async::<()>(&mut *conn)
            .await?;
        Ok(())
    }

    fn driver_name(&self) -> &'static str { "redis" }
}
