use std::sync::Arc;
use axum::{
    middleware::from_fn_with_state,
    routing::{delete, get, post, put},
    Router,
};

use crate::app::http::controllers::{
    auth_controller, permission_controller, role_controller, user_controller,
};
use crate::app::http::middleware::{authenticate, must_be_active};
use crate::bootstrap::app::AppState;

/// Build the `/api` routes.
///
/// * **public** — registration & login (no auth).
/// * **protected** — everything else, wrapped by `authenticate` → `must_be_active`.
pub fn api_routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    let public = Router::new()
        .route("/auth/register", post(auth_controller::register))
        .route("/auth/login", post(auth_controller::login));

    let protected = Router::new()
        // auth
        .route("/auth/me", get(auth_controller::me))
        // users
        .route("/users", get(user_controller::index).post(user_controller::store))
        .route(
            "/users/{id}",
            get(user_controller::show)
                .put(user_controller::update)
                .delete(user_controller::destroy),
        )
        // roles
        .route("/roles", get(role_controller::index).post(role_controller::store))
        .route("/roles/{id}", delete(role_controller::destroy))
        .route("/roles/{id}/permissions", put(role_controller::sync_permissions))
        // permissions
        .route("/permissions", get(permission_controller::index))
        .route("/permissions/{id}", get(permission_controller::show))
        // middleware: authenticate (outermost) → must_be_active
        .route_layer(from_fn_with_state(state.clone(), must_be_active))
        .route_layer(from_fn_with_state(state.clone(), authenticate));

    public.merge(protected)
}
