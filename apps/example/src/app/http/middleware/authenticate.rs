use std::sync::Arc;
use axum::{
    extract::{FromRequestParts, Request, State},
    http::{request::Parts, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::app::models::User;
use crate::bootstrap::app::AppState;

/// The authenticated user, placed into request extensions by [`authenticate`].
#[derive(Clone)]
pub struct AuthUser(pub Arc<User>);

/// Middleware: verify the `Authorization: Bearer <jwt>` header and load the user.
///
/// Apply with `route_layer(from_fn_with_state(state.clone(), authenticate))`.
pub async fn authenticate(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Response {
    let token = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::to_string);

    let Some(token) = token else {
        return unauthorized("missing bearer token");
    };

    match state.auth.me(&token).await {
        Ok(user) => {
            req.extensions_mut().insert(AuthUser(Arc::new(user)));
            next.run(req).await
        }
        Err(e) => unauthorized(&e.to_string()),
    }
}

fn unauthorized(msg: &str) -> Response {
    (StatusCode::UNAUTHORIZED, Json(json!({ "message": "Unauthenticated.", "error": msg }))).into_response()
}

/// Extractor so handlers can simply take `user: AuthUser`.
impl<S: Send + Sync> FromRequestParts<S> for AuthUser {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or_else(|| unauthorized("not authenticated"))
    }
}
