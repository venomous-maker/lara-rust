use std::sync::Arc;
use async_trait::async_trait;

use crate::app::services::{
    AuthService, FileService, PermissionService, RoleService, TokenService, UserService,
};
use super::{Builder, ServiceProvider};

/// Binds the application's domain services as singletons.
///
/// This is where **dependency injection** happens: services are constructed
/// in dependency order, each receiving the singletons it needs.
pub struct AppServiceProvider;

#[async_trait]
impl ServiceProvider for AppServiceProvider {
    fn name(&self) -> &'static str { "AppServiceProvider" }

    async fn register(&self, builder: &mut Builder) -> anyhow::Result<()> {
        let events = builder.events();

        // Leaf services (no service dependencies).
        let tokens = Arc::new(TokenService::new(builder.auth_config.clone()));
        let users  = Arc::new(UserService::new(events.clone()));
        let roles  = Arc::new(RoleService::new());
        let permissions = Arc::new(PermissionService::new());
        let files  = Arc::new(FileService::new("storage/uploads"));

        // Composite service — injected with the three singletons it depends on.
        let auth = Arc::new(AuthService::new(users.clone(), tokens.clone(), events.clone()));

        builder.tokens = Some(tokens);
        builder.users = Some(users);
        builder.roles = Some(roles);
        builder.permissions = Some(permissions);
        builder.files = Some(files);
        builder.auth = Some(auth);

        Ok(())
    }

    async fn boot(&self, _builder: &Builder) -> anyhow::Result<()> {
        tracing::info!("domain services bound (auth, users, roles, permissions, files)");
        Ok(())
    }
}
