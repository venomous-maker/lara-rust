use std::env;

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
            jwt_secret: env::var("JWT_SECRET").unwrap_or_else(|_| "super-secret-jwt-key".into()),
            jwt_ttl_hours: env::var("JWT_TTL").ok().and_then(|v| v.parse().ok()).unwrap_or(24),
            refresh_ttl_hours: env::var("JWT_REFRESH_TTL").ok().and_then(|v| v.parse().ok()).unwrap_or(168),
            bcrypt_rounds: env::var("BCRYPT_ROUNDS").ok().and_then(|v| v.parse().ok()).unwrap_or(12),
        }
    }
}
