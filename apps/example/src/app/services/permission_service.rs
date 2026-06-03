use anyhow::{anyhow, Result};
use lara_db::ModelTrait;

use crate::app::models::Permission;

/// Read-only access to permissions.
pub struct PermissionService;

impl PermissionService {
    pub fn new() -> Self { Self }

    pub async fn all(&self) -> Result<Vec<Permission>> {
        Permission::query()
            .order_by_asc("slug")
            .get()
            .await
            .map_err(|e| anyhow!("{}", e))
    }

    pub async fn find(&self, id: i64) -> Result<Permission> {
        Permission::find_or_fail(id)
            .await
            .map_err(|_| anyhow!("permission #{} not found", id))
    }
}

impl Default for PermissionService {
    fn default() -> Self { Self::new() }
}
