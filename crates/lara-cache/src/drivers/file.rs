use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use anyhow::Result;
use crate::Cache;

#[derive(Serialize, Deserialize)]
struct FileEntry {
    value: Value,
    expires_at: Option<u64>,  // unix timestamp
}

pub struct FileCache {
    dir: PathBuf,
}

impl FileCache {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        let dir = dir.into();
        std::fs::create_dir_all(&dir).ok();
        Self { dir }
    }

    fn path(&self, key: &str) -> PathBuf {
        let hashed = format!("{:x}", md5_simple(key));
        self.dir.join(format!("{}.json", hashed))
    }
}

fn md5_simple(s: &str) -> u64 {
    let mut h: u64 = 14695981039346656037;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    h
}

fn now_ts() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

#[async_trait]
impl Cache for FileCache {
    async fn get(&self, key: &str) -> Result<Option<Value>> {
        let path = self.path(key);
        if !path.exists() { return Ok(None); }
        let raw = tokio::fs::read(&path).await?;
        let entry: FileEntry = serde_json::from_slice(&raw)?;
        if let Some(exp) = entry.expires_at {
            if now_ts() > exp {
                tokio::fs::remove_file(&path).await.ok();
                return Ok(None);
            }
        }
        Ok(Some(entry.value))
    }

    async fn set(&self, key: &str, value: Value, ttl: Option<Duration>) -> Result<()> {
        let entry = FileEntry {
            value,
            expires_at: ttl.map(|d| now_ts() + d.as_secs()),
        };
        let raw = serde_json::to_vec(&entry)?;
        tokio::fs::write(self.path(key), raw).await?;
        Ok(())
    }

    async fn has(&self, key: &str) -> Result<bool> {
        Ok(self.get(key).await?.is_some())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        tokio::fs::remove_file(self.path(key)).await.ok();
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        let mut dir = tokio::fs::read_dir(&self.dir).await?;
        while let Some(entry) = dir.next_entry().await? {
            tokio::fs::remove_file(entry.path()).await.ok();
        }
        Ok(())
    }
}
