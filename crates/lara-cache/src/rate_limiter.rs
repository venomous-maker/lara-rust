use std::{collections::HashMap, sync::Arc, time::{Duration, Instant}};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
struct Window {
    count: u64,
    window_start: Instant,
}

/// Simple in-process rate limiter (sliding window).
pub struct RateLimiter {
    windows: Arc<Mutex<HashMap<String, Window>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self { windows: Arc::new(Mutex::new(HashMap::new())) }
    }

    /// Returns `true` if the key is within the allowed limit.
    pub async fn attempt(&self, key: &str, max_attempts: u64, decay: Duration) -> bool {
        let mut windows = self.windows.lock().await;
        let now = Instant::now();
        let window = windows.entry(key.to_string()).or_insert(Window {
            count: 0,
            window_start: now,
        });

        if now.duration_since(window.window_start) >= decay {
            window.count = 0;
            window.window_start = now;
        }

        if window.count < max_attempts {
            window.count += 1;
            true
        } else {
            false
        }
    }

    /// Remaining attempts for a key in the current window.
    pub async fn remaining(&self, key: &str, max_attempts: u64, decay: Duration) -> u64 {
        let mut windows = self.windows.lock().await;
        let now = Instant::now();
        let window = windows.entry(key.to_string()).or_insert(Window {
            count: 0,
            window_start: now,
        });
        if now.duration_since(window.window_start) >= decay {
            return max_attempts;
        }
        max_attempts.saturating_sub(window.count)
    }

    pub async fn clear(&self, key: &str) {
        self.windows.lock().await.remove(key);
    }

    pub async fn too_many_attempts(&self, key: &str, max_attempts: u64, decay: Duration) -> bool {
        !self.attempt(key, max_attempts, decay).await
    }
}

impl Default for RateLimiter {
    fn default() -> Self { Self::new() }
}
