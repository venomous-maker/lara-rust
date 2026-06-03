use lara_core::env::{env_bool, env_or, env_or_parse};

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
            name: env_or("APP_NAME", "Lara App"),
            env:  env_or("APP_ENV", "local"),
            debug: env_bool("APP_DEBUG", true),
            url:  env_or("APP_URL", "http://localhost:3000"),
            port: env_or_parse("APP_PORT", 3000),
            timezone: env_or("APP_TIMEZONE", "UTC"),
            key: env_or("APP_KEY", "secret-change-me-in-production"),
        }
    }
}
