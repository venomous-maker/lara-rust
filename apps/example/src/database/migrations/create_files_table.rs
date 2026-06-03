use async_trait::async_trait;
use lara_db::{error::Result, migrations::Migration, schema::Schema};

pub struct CreateFilesTable;

#[async_trait]
impl Migration for CreateFilesTable {
    fn name(&self) -> &'static str { "2024_01_01_000005_create_files_table" }

    async fn up(&self) -> Result<()> {
        Schema::create("files", |t| {
            t.id();
            t.foreign_id("user_id").nullable();
            t.string("original_name", 255);
            t.string("stored_name", 255);
            t.string("path", 500);
            t.string("mime_type", 100);
            t.big_integer("size");
            t.timestamps();
            t.soft_deletes();
        }).await
    }

    async fn down(&self) -> Result<()> {
        Schema::drop("files").await
    }
}
