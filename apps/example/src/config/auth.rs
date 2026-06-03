use lara_core::env::{env_or, env_or_parse};

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_ttl_hours: u64,
    pub refresh_ttl_hours: u64,
    pub bcrypt_rounds: u32,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: env_or("JWT_SECRET", "super-secret-jwt-key"),
            jwt_ttl_hours: env_or_parse("JWT_TTL", 24),
            refresh_ttl_hours: env_or_parse("JWT_REFRESH_TTL", 168),
            bcrypt_rounds: env_or_parse("BCRYPT_ROUNDS", 12),
        }
    }
}
