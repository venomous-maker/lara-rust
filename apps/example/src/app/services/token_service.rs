use anyhow::{anyhow, Result};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::config::AuthConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id
    pub email: String,
    pub exp: usize,
    pub iat: usize,
}

/// Issues and verifies JWTs. A singleton — depends only on `AuthConfig`.
pub struct TokenService {
    config: AuthConfig,
}

impl TokenService {
    pub fn new(config: AuthConfig) -> Self {
        Self { config }
    }

    pub fn issue(&self, user_id: i64, email: &str) -> Result<String> {
        let now = chrono::Utc::now();
        let exp = now + chrono::Duration::hours(self.config.jwt_ttl_hours as i64);
        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            iat: now.timestamp() as usize,
            exp: exp.timestamp() as usize,
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
        )
        .map_err(|e| anyhow!("token encode failed: {}", e))
    }

    pub fn verify(&self, token: &str) -> Result<Claims> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|e| anyhow!("token invalid: {}", e))
    }
}
