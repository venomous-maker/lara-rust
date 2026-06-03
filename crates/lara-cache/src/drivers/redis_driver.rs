// Redis cache driver — enabled via the `redis-driver` feature flag.
use async_trait::async_trait;
use serde_json::Value;
use std::{sync::Arc, time::Duration};
use anyhow::Result;
use crate::Cache;

pub struct RedisCache {
    pool: deadpool_redis::Pool,
}

impl RedisCache {
    pub async fn new(url: &str) -> Result<Self> {
        let cfg = deadpool_redis::Config::from_url(url);
        let pool = cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))?;
        Ok(Self { pool })
    }
}

#[async_trait]
impl Cache for RedisCache {
    async fn get(&self, key: &str) -> Result<Option<Value>> {
        let mut conn = self.pool.get().await?;
        let raw: Option<String> = redis::cmd("GET")
            .arg(key)
            .query_async(&mut *conn)
            .await?;
        Ok(raw.and_then(|s| serde_json::from_str(&s).ok()))
    }

    async fn set(&self, key: &str, value: Value, ttl: Option<Duration>) -> Result<()> {
        let mut conn = self.pool.get().await?;
        let serialized = serde_json::to_string(&value)?;
        if let Some(d) = ttl {
            redis::cmd("SET")
                .arg(key)
                .arg(&serialized)
                .arg("EX")
                .arg(d.as_secs())
                .query_async::<()>(&mut *conn)
                .await?;
        } else {
            redis::cmd("SET")
                .arg(key)
                .arg(&serialized)
                .query_async::<()>(&mut *conn)
                .await?;
        }
        Ok(())
    }

    async fn has(&self, key: &str) -> Result<bool> {
        let mut conn = self.pool.get().await?;
        let exists: i64 = redis::cmd("EXISTS").arg(key).query_async(&mut *conn).await?;
        Ok(exists > 0)
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let mut conn = self.pool.get().await?;
        redis::cmd("DEL").arg(key).query_async::<()>(&mut *conn).await?;
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        let mut conn = self.pool.get().await?;
        redis::cmd("FLUSHDB").query_async::<()>(&mut *conn).await?;
        Ok(())
    }
}
