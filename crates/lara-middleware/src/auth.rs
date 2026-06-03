use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtClaims {
    pub sub: String,    // user id
    pub exp: usize,
    pub iat: usize,
}

#[derive(Clone)]
pub struct AuthLayer {
    secret: Arc<String>,
}

impl AuthLayer {
    pub fn new(secret: impl Into<String>) -> Self {
        Self { secret: Arc::new(secret.into()) }
    }

    pub async fn jwt_auth(
        State(layer): State<AuthLayer>,
        mut req: Request,
        next: Next,
    ) -> Response {
        let token = req
            .headers()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(|t| t.to_string());

        let Some(token) = token else {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "No bearer token provided"})),
            ).into_response();
        };

        let key = DecodingKey::from_secret(layer.secret.as_bytes());
        let mut validation = Validation::new(Algorithm::HS256);

        match decode::<JwtClaims>(&token, &key, &validation) {
            Ok(data) => {
                req.extensions_mut().insert(data.claims);
                next.run(req).await
            }
            Err(e) => (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": format!("Invalid token: {}", e)})),
            ).into_response(),
        }
    }
}
