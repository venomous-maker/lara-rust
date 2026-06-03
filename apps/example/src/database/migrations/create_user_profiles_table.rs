use async_trait::async_trait;
use lara_db::{error::Result, migrations::Migration, schema::Schema};

pub struct CreateUserProfilesTable;

#[async_trait]
impl Migration for CreateUserProfilesTable {
    fn name(&self) -> &'static str { "2024_01_01_000003_create_user_profiles_table" }

    async fn up(&self) -> Result<()> {
        Schema::create("user_profiles", |t| {
            t.id();
            t.foreign_id("user_id").unique();
            t.text("bio").nullable();
            t.string("avatar", 500).nullable();
            t.string("website", 255).nullable();
            t.string("twitter", 100).nullable();
            t.timestamps();
        }).await
    }

    async fn down(&self) -> Result<()> {
        Schema::drop("user_profiles").await
    }
}
