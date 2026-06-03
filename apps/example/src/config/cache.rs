use std::env;

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
        let driver = match env::var("CACHE_DRIVER").as_deref() {
            Ok("file")  => CacheDriver::File,
            Ok("redis") => CacheDriver::Redis,
            _           => CacheDriver::Memory,
        };
        Self {
            driver,
            prefix: env::var("CACHE_PREFIX").unwrap_or_else(|_| "lara_".into()),
            redis_url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into()),
            file_path: env::var("CACHE_PATH").unwrap_or_else(|_| "storage/cache".into()),
            default_ttl_secs: 3600,
        }
    }
}
