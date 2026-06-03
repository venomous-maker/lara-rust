use lara_db::{error::Result, ModelTrait, Value};

use crate::app::models::{Permission, Role, User};
use crate::app::services::UserService;

/// Seed the database with roles, permissions, and demo users.
pub async fn seed() -> Result<()> {
    seed_permissions().await?;
    seed_roles().await?;
    seed_users().await?;
    tracing::info!("database seeded");
    Ok(())
}

async fn seed_permissions() -> Result<()> {
    const PERMS: &[(&str, &str)] = &[
        ("users.view", "View users"),
        ("users.create", "Create users"),
        ("users.update", "Update users"),
        ("users.delete", "Delete users"),
        ("roles.manage", "Manage roles"),
    ];
    for (slug, name) in PERMS {
        if !Permission::query().where_eq("slug", *slug).exists().await? {
            Permission::create(Permission {
                id: None,
                name: name.to_string(),
                slug: slug.to_string(),
                description: None,
                created_at: None,
                updated_at: None,
                deleted_at: None,
            }).await?;
        }
    }
    Ok(())
}

async fn seed_roles() -> Result<()> {
    for (slug, name) in [("admin", "Administrator"), ("user", "User")] {
        if !Role::query().where_eq("slug", slug).exists().await? {
            Role::create(Role {
                id: None,
                name: name.to_string(),
                slug: slug.to_string(),
                description: None,
                created_at: None,
                updated_at: None,
                deleted_at: None,
            }).await?;
        }
    }

    // Grant every permission to the admin role.
    if let Ok(admin) = Role::query().where_eq("slug", "admin").first_or_fail().await {
        let perm_ids: Vec<Value> = Permission::all()
            .await?
            .into_iter()
            .filter_map(|p| p.id.map(Value::Int))
            .collect();
        admin.permissions().sync(&perm_ids).await?;
    }
    Ok(())
}

async fn seed_users() -> Result<()> {
    let seeds = [
        ("Admin", "admin@example.com", "admin"),
        ("Demo User", "user@example.com", "user"),
    ];

    for (name, email, role_slug) in seeds {
        if User::query().where_eq("email", email).exists().await? {
            continue;
        }
        // `password` for both demo accounts.
        let hashed = UserService::hash_password("password")
            .map_err(|e| lara_db::DbError::Other(e.to_string()))?;

        let user = User::create(User {
            id: None,
            name: name.to_string(),
            email: email.to_string(),
            password: hashed,
            status: "active".to_string(),
            email_verified_at: None,
            created_at: None,
            updated_at: None,
            deleted_at: None,
        }).await?;

        if let Ok(role) = Role::query().where_eq("slug", role_slug).first_or_fail().await {
            if let Some(role_id) = role.id {
                user.roles().attach(&[Value::Int(role_id)]).await?;
            }
        }
    }
    Ok(())
}
