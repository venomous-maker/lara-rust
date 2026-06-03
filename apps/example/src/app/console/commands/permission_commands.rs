use async_trait::async_trait;
use lara_console::{Args, Command, CommandMeta};
use lara_db::ModelTrait;

use crate::app::models::Permission;

/// `artisan permissions:list` — print all permissions.
pub struct PermissionsListCommand;

impl CommandMeta for PermissionsListCommand {
    fn command_name() -> &'static str { "permissions:list" }
    fn command_description() -> &'static str { "List all permissions" }
}

#[async_trait]
impl Command for PermissionsListCommand {
    async fn handle(&self, _args: &Args) -> anyhow::Result<()> {
        let perms = Permission::all().await.map_err(|e| anyhow::anyhow!("{}", e))?;
        if perms.is_empty() {
            println!("No permissions found. Run `artisan db:seed` first.");
            return Ok(());
        }
        println!("{:<5} {:<28} {}", "ID", "SLUG", "NAME");
        for p in perms {
            println!("{:<5} {:<28} {}", p.id.unwrap_or_default(), p.slug, p.name);
        }
        Ok(())
    }
}

/// `artisan permissions:sync` — ensure the canonical permission set exists.
pub struct PermissionsSyncCommand;

impl CommandMeta for PermissionsSyncCommand {
    fn command_name() -> &'static str { "permissions:sync" }
    fn command_description() -> &'static str { "Create any missing canonical permissions" }
}

#[async_trait]
impl Command for PermissionsSyncCommand {
    async fn handle(&self, _args: &Args) -> anyhow::Result<()> {
        const CANON: &[(&str, &str)] = &[
            ("users.view", "View users"),
            ("users.create", "Create users"),
            ("users.update", "Update users"),
            ("users.delete", "Delete users"),
            ("roles.manage", "Manage roles"),
        ];
        let mut created = 0;
        for (slug, name) in CANON {
            let exists = Permission::query()
                .where_eq("slug", *slug)
                .exists()
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if !exists {
                Permission::create(Permission {
                    name: name.to_string(),
                    slug: slug.to_string(),
                    ..Default::default()
                })
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
                created += 1;
            }
        }
        println!("permissions synced ({} created)", created);
        Ok(())
    }
}
