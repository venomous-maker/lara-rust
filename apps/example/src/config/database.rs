use lara_db::connection::config::{DatabaseConfig, SqlConfig, SqliteConfig};

pub fn default_connection() -> DatabaseConfig {
    // For the example we default to SQLite so it runs without any external server.
    // Override via environment variables in production.
    let path = std::env::var("DB_PATH").unwrap_or_else(|_| "database/app.db".to_string());
    DatabaseConfig::Sqlite(SqliteConfig {
        path,
        max_connections: 5,
    })
}

pub fn postgres_connection() -> DatabaseConfig {
    DatabaseConfig::Postgres(SqlConfig {
        host:     std::env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string()),
        port:     std::env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string()).parse().unwrap_or(5432),
        database: std::env::var("DB_DATABASE").unwrap_or_else(|_| "lara".to_string()),
        username: std::env::var("DB_USERNAME").unwrap_or_else(|_| "postgres".to_string()),
        password: std::env::var("DB_PASSWORD").unwrap_or_default(),
        max_connections: 20,
        min_connections: 2,
        connect_timeout_secs: 30,
        ssl: false,
    })
}

pub fn mysql_connection() -> DatabaseConfig {
    DatabaseConfig::Mysql(SqlConfig {
        host:     std::env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string()),
        port:     std::env::var("DB_PORT").unwrap_or_else(|_| "3306".to_string()).parse().unwrap_or(3306),
        database: std::env::var("DB_DATABASE").unwrap_or_else(|_| "lara".to_string()),
        username: std::env::var("DB_USERNAME").unwrap_or_else(|_| "root".to_string()),
        password: std::env::var("DB_PASSWORD").unwrap_or_default(),
        max_connections: 20,
        min_connections: 2,
        connect_timeout_secs: 30,
        ssl: false,
    })
}
