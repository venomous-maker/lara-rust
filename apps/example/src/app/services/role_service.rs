use anyhow::{anyhow, Result};
use lara_db::{ModelTrait, Value};

use crate::app::models::{Permission, Role};

/// Role CRUD and permission synchronization. No external dependencies.
pub struct RoleService;

impl RoleService {
    pub fn new() -> Self { Self }

    pub async fn all(&self) -> Result<Vec<Role>> {
        Role::all().await.map_err(|e| anyhow!("{}", e))
    }

    pub async fn find(&self, id: i64) -> Result<Role> {
        Role::find_or_fail(id).await.map_err(|_| anyhow!("role #{} not found", id))
    }

    pub async fn create(&self, name: String, slug: String, description: Option<String>) -> Result<Role> {
        Role::create(Role {
            id: None,
            name,
            slug,
            description,
            created_at: None,
            updated_at: None,
            deleted_at: None,
        })
        .await
        .map_err(|e| anyhow!("create failed: {}", e))
    }

    pub async fn delete(&self, id: i64) -> Result<()> {
        let role = self.find(id).await?;
        role.delete().await.map_err(|e| anyhow!("{}", e))
    }

    /// Replace the role's permission set with the given permission IDs.
    pub async fn sync_permissions(&self, role_id: i64, permission_ids: &[i64]) -> Result<()> {
        let role = self.find(role_id).await?;
        let ids: Vec<Value> = permission_ids.iter().map(|id| Value::Int(*id)).collect();
        role.permissions()
            .sync(&ids)
            .await
            .map_err(|e| anyhow!("sync failed: {}", e))
    }

    pub async fn permissions_of(&self, role: &Role) -> Result<Vec<Permission>> {
        role.permissions().get().await.map_err(|e| anyhow!("{}", e))
    }
}

impl Default for RoleService {
    fn default() -> Self { Self::new() }
}
