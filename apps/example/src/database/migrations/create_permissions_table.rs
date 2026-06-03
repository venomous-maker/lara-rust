use async_trait::async_trait;
use lara_db::{error::Result, migrations::Migration, schema::Schema};

pub struct CreatePermissionsTable;

#[async_trait]
impl Migration for CreatePermissionsTable {
    fn name(&self) -> &'static str { "2024_01_01_000004_create_permissions_table" }

    async fn up(&self) -> Result<()> {
        Schema::create("permissions", |t| {
            t.id();
            t.string("name", 100);
            t.string("slug", 100).unique();
            t.text("description").nullable();
            t.timestamps();
            t.soft_deletes();
        }).await?;

        // pivot: permission_role
        Schema::create("permission_role", |t| {
            t.unsigned_big_integer("permission_id");
            t.unsigned_big_integer("role_id");
            t.unique_index(&["permission_id", "role_id"]);
        }).await
    }

    async fn down(&self) -> Result<()> {
        Schema::drop("permission_role").await?;
        Schema::drop("permissions").await
    }
}
