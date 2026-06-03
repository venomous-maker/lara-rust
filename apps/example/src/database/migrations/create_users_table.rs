use async_trait::async_trait;
use lara_db::{error::Result, migrations::Migration, schema::Schema};

pub struct CreateUsersTable;

#[async_trait]
impl Migration for CreateUsersTable {
    fn name(&self) -> &'static str { "2024_01_01_000001_create_users_table" }

    async fn up(&self) -> Result<()> {
        Schema::create("users", |t| {
            t.id();
            t.string("name", 255);
            t.string("email", 255).unique();
            t.string("password", 255);
            t.string("status", 20).default("'active'");
            t.date_time("email_verified_at").nullable();
            t.timestamps();
            t.soft_deletes();
            t.unique_index(&["email"]);
        }).await
    }

    async fn down(&self) -> Result<()> {
        Schema::drop("users").await
    }
}
