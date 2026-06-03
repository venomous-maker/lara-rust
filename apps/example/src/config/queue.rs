use lara_core::env::{env, env_or};

#[derive(Debug, Clone)]
pub enum QueueDriver {
    Sync,
    Database,
    Redis,
}

#[derive(Debug, Clone)]
pub struct QueueConfig {
    pub driver: QueueDriver,
    pub redis_url: String,
    pub default_queue: String,
    pub retry_after_secs: u64,
    pub max_tries: u32,
}

impl Default for QueueConfig {
    fn default() -> Self {
        let driver = match env("QUEUE_DRIVER").as_deref() {
            Some("database") => QueueDriver::Database,
            Some("redis")    => QueueDriver::Redis,
            _                => QueueDriver::Sync,
        };
        Self {
            driver,
            redis_url: env_or("REDIS_URL", "redis://127.0.0.1:6379"),
            default_queue: env_or("QUEUE_NAME", "default"),
            retry_after_secs: 90,
            max_tries: 3,
        }
    }
}
