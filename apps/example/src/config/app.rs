use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub name: String,
    pub env: String,
    pub debug: bool,
    pub url: String,
    pub port: u16,
    pub timezone: String,
    pub key: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: env::var("APP_NAME").unwrap_or_else(|_| "Lara App".into()),
            env:  env::var("APP_ENV").unwrap_or_else(|_| "local".into()),
            debug: env::var("APP_DEBUG").map(|v| v == "true").unwrap_or(true),
            url:  env::var("APP_URL").unwrap_or_else(|_| "http://localhost:3000".into()),
            port: env::var("APP_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(3000),
            timezone: env::var("APP_TIMEZONE").unwrap_or_else(|_| "UTC".into()),
            key: env::var("APP_KEY").unwrap_or_else(|_| "secret-change-me-in-production".into()),
        }
    }
}
