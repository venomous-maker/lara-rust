use std::sync::Arc;
use async_trait::async_trait;

use lara_cache::{FileCache, MemoryCache, RateLimiter, SharedCache};

use crate::config::cache::CacheDriver;
use super::{Builder, ServiceProvider};

/// Binds the cache store and the rate limiter as singletons.
pub struct CacheServiceProvider;

#[async_trait]
impl ServiceProvider for CacheServiceProvider {
    fn name(&self) -> &'static str { "CacheServiceProvider" }

    async fn register(&self, builder: &mut Builder) -> anyhow::Result<()> {
        let cache: SharedCache = match builder.cache_config.driver {
            CacheDriver::File  => Arc::new(FileCache::new(&builder.cache_config.file_path)),
            // Redis driver omitted here for brevity; falls back to memory.
            _                  => Arc::new(MemoryCache::new()),
        };
        builder.cache = Some(cache);
        builder.limiter = Some(Arc::new(RateLimiter::new()));
        Ok(())
    }
}
