use async_trait::async_trait;
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use anyhow::Result;
use crate::Cache;

struct Entry {
    value: Value,
    expires_at: Option<Instant>,
}

impl Entry {
    fn is_expired(&self) -> bool {
        self.expires_at.map(|e| Instant::now() > e).unwrap_or(false)
    }
}

pub struct MemoryCache {
    store: Arc<RwLock<HashMap<String, Entry>>>,
}

impl MemoryCache {
    pub fn new() -> Self {
        Self { store: Arc::new(RwLock::new(HashMap::new())) }
    }
}

impl Default for MemoryCache {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl Cache for MemoryCache {
    async fn get(&self, key: &str) -> Result<Option<Value>> {
        let store = self.store.read().await;
        if let Some(e) = store.get(key) {
            if e.is_expired() { return Ok(None); }
            return Ok(Some(e.value.clone()));
        }
        Ok(None)
    }

    async fn set(&self, key: &str, value: Value, ttl: Option<Duration>) -> Result<()> {
        let mut store = self.store.write().await;
        store.insert(key.to_string(), Entry {
            value,
            expires_at: ttl.map(|d| Instant::now() + d),
        });
        Ok(())
    }

    async fn has(&self, key: &str) -> Result<bool> {
        Ok(self.get(key).await?.is_some())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        self.store.write().await.remove(key);
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.store.write().await.clear();
        Ok(())
    }
}
