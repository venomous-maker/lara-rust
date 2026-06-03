mod app;
mod bootstrap;
mod config;
mod database {
    pub mod migrations;
    pub mod seeders;
}
mod routes;

use async_trait::async_trait;
use lara_console::{Args, Command, CommandMeta, Kernel};

use app::console::commands::{PermissionsListCommand, PermissionsSyncCommand};
use bootstrap::app::AppState;
use database::{migrations::all_migrations, seeders::seed};

// ── Built-in framework commands ────────────────────────────────────────────────

struct MigrateCommand;
impl CommandMeta for MigrateCommand {
    fn command_name() -> &'static str { "migrate" }
    fn command_description() -> &'static str { "Run all pending database migrations" }
}
#[async_trait]
impl Command for MigrateCommand {
    async fn handle(&self, _args: &Args) -> anyhow::Result<()> {
        all_migrations().run().await.map_err(|e| anyhow::anyhow!("{}", e))?;
        println!("Migrations complete.");
        Ok(())
    }
}

struct MigrateRollback;
impl CommandMeta for MigrateRollback {
    fn command_name() -> &'static str { "migrate:rollback" }
    fn command_description() -> &'static str { "Roll back the last batch" }
}
#[async_trait]
impl Command for MigrateRollback {
    async fn handle(&self, _args: &Args) -> anyhow::Result<()> {
        all_migrations().rollback().await.map_err(|e| anyhow::anyhow!("{}", e))?;
        println!("Rolled back.");
        Ok(())
    }
}

struct DbSeed;
impl CommandMeta for DbSeed {
    fn command_name() -> &'static str { "db:seed" }
    fn command_description() -> &'static str { "Seed the database with initial data" }
}
#[async_trait]
impl Command for DbSeed {
    async fn handle(&self, _args: &Args) -> anyhow::Result<()> {
        seed().await.map_err(|e| anyhow::anyhow!("{}", e))?;
        println!("Seeded.");
        Ok(())
    }
}

// ── Entry point ─────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    // Boot the app (configures DB, mail, and all service providers / singletons).
    let _state = AppState::boot().await?;

    Kernel::new()
        .register(MigrateCommand)
        .register(MigrateRollback)
        .register(DbSeed)
        .register(PermissionsListCommand)
        .register(PermissionsSyncCommand)
        .handle()
        .await
}
