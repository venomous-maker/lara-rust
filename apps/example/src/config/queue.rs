use std::env;

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
        let driver = match env::var("QUEUE_DRIVER").as_deref() {
            Ok("database") => QueueDriver::Database,
            Ok("redis")    => QueueDriver::Redis,
            _              => QueueDriver::Sync,
        };
        Self {
            driver,
            redis_url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into()),
            default_queue: env::var("QUEUE_NAME").unwrap_or_else(|_| "default".into()),
            retry_after_secs: 90,
            max_tries: 3,
        }
    }
}
