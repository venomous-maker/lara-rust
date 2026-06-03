pub mod drivers;
pub mod rate_limiter;

use async_trait::async_trait;
use serde_json::Value;
use std::{sync::Arc, time::Duration};
use anyhow::Result;

// ── Cache trait (dyn-compatible: no generics in methods) ─────────────────────

#[async_trait]
pub trait Cache: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Value>>;
    async fn set(&self, key: &str, value: Value, ttl: Option<Duration>) -> Result<()>;
    async fn has(&self, key: &str) -> Result<bool>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn clear(&self) -> Result<()>;
}

/// Convenience free function: get or compute + cache.
pub async fn get_or_set<F, Fut>(
    cache: &dyn Cache,
    key: &str,
    ttl: Option<Duration>,
    f: F,
) -> Result<Value>
where
    F: FnOnce() -> Fut + Send,
    Fut: std::future::Future<Output = Result<Value>> + Send,
{
    if let Some(v) = cache.get(key).await? {
        return Ok(v);
    }
    let v = f().await?;
    cache.set(key, v.clone(), ttl).await?;
    Ok(v)
}

pub type SharedCache = Arc<dyn Cache>;

pub use drivers::memory::MemoryCache;
pub use drivers::file::FileCache;
pub use rate_limiter::RateLimiter;
