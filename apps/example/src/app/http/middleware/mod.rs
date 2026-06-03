pub mod authenticate;
pub mod must_be_active;
pub mod throttle;

pub use authenticate::{authenticate, AuthUser};
pub use must_be_active::must_be_active;

use anyhow::{anyhow, Result};
use crate::app::models::User;

/// Authorization helper: assert the user holds the given role (by slug).
/// Used inside controllers in place of a parameterized `role:` middleware.
pub async fn ensure_role(user: &User, role_slug: &str) -> Result<()> {
    let has = user
        .roles()
        .query()
        .where_eq("slug", role_slug)
        .exists()
        .await
        .map_err(|e| anyhow!("role check failed: {}", e))?;
    if has {
        Ok(())
    } else {
        Err(anyhow!("requires `{}` role", role_slug))
    }
}
