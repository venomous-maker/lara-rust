use lara_core::env::{env, env_or};

#[derive(Debug, Clone)]
pub enum CacheDriver {
    Memory,
    File,
    Redis,
}

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub driver: CacheDriver,
    pub prefix: String,
    pub redis_url: String,
    pub file_path: String,
    pub default_ttl_secs: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        let driver = match env("CACHE_DRIVER").as_deref() {
            Some("file")  => CacheDriver::File,
            Some("redis") => CacheDriver::Redis,
            _             => CacheDriver::Memory,
        };
        Self {
            driver,
            prefix: env_or("CACHE_PREFIX", "lara_"),
            redis_url: env_or("REDIS_URL", "redis://127.0.0.1:6379"),
            file_path: env_or("CACHE_PATH", "storage/cache"),
            default_ttl_secs: 3600,
        }
    }
}
