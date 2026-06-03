use anyhow::{anyhow, Result};
use lara_db::{ModelTrait, Paginator};

use crate::app::models::File;

/// File metadata persistence and disk cleanup.
pub struct FileService {
    storage_dir: String,
}

impl FileService {
    pub fn new(storage_dir: impl Into<String>) -> Self {
        Self { storage_dir: storage_dir.into() }
    }

    pub async fn paginate(&self, page: u64, per_page: u64) -> Result<Paginator<File>> {
        File::query()
            .order_by_desc("created_at")
            .paginate(per_page, page)
            .await
            .map_err(|e| anyhow!("{}", e))
    }

    pub async fn record(
        &self,
        user_id: Option<i64>,
        original_name: String,
        stored_name: String,
        mime_type: String,
        size: i64,
    ) -> Result<File> {
        let path = format!("{}/{}", self.storage_dir, stored_name);
        File::create(File {
            user_id,
            original_name,
            stored_name,
            path,
            mime_type,
            size,
            ..Default::default()
        })
        .await
        .map_err(|e| anyhow!("record failed: {}", e))
    }

    pub async fn find(&self, id: i64) -> Result<File> {
        File::find_or_fail(id).await.map_err(|_| anyhow!("file #{} not found", id))
    }

    /// Soft-delete the DB record and remove the file from disk (best-effort).
    pub async fn delete(&self, id: i64) -> Result<()> {
        let file = self.find(id).await?;
        let _ = tokio::fs::remove_file(&file.path).await;
        file.delete().await.map_err(|e| anyhow!("{}", e))
    }

    pub fn storage_dir(&self) -> &str {
        &self.storage_dir
    }
}
