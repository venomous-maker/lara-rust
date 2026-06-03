use async_trait::async_trait;
use lara_db::{error::Result, migrations::Migration, schema::Schema};

pub struct CreateRolesTable;

#[async_trait]
impl Migration for CreateRolesTable {
    fn name(&self) -> &'static str { "2024_01_01_000002_create_roles_table" }

    async fn up(&self) -> Result<()> {
        Schema::create("roles", |t| {
            t.id();
            t.string("name", 100);
            t.string("slug", 100).unique();
            t.text("description").nullable();
            t.timestamps();
        }).await?;
        Schema::create("role_user", |t| {
            t.unsigned_big_integer("user_id");
            t.unsigned_big_integer("role_id");
            t.unique_index(&["user_id", "role_id"]);
        }).await
    }

    async fn down(&self) -> Result<()> {
        Schema::drop("role_user").await?;
        Schema::drop("roles").await
    }
}
