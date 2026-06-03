use lara_db::connection::config::{
    DatabaseConfig, MongoConfig, SqlConfig, SqliteConfig,
};

/// Resolve the database connection.
///
/// Resolution order:
///   1. If `DATABASE_URL` is set, the driver is inferred from its **scheme/host**
///      (e.g. `postgres://…`, `mysql://…`, `mongodb://…`, `sqlite://…`) and the
///      whole config is parsed from the URL.
///   2. Otherwise fall back to the `DB_CONNECTION` switch + individual `DB_*` vars.
pub fn default_connection() -> DatabaseConfig {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        if !url.trim().is_empty() {
            return from_url(&url);
        }
    }
    let driver = std::env::var("DB_CONNECTION").unwrap_or_else(|_| "sqlite".to_string());
    connection_for(&driver)
}

/// Infer the driver from a connection URL's scheme and build its config.
pub fn from_url(url: &str) -> DatabaseConfig {
    let (scheme, rest) = match url.split_once("://") {
        Some((s, r)) => (s.to_lowercase(), r),
        None => return sqlite_connection_with(url), // bare path → sqlite file
    };

    match scheme.as_str() {
        "postgres" | "postgresql" | "pgsql" => DatabaseConfig::Postgres(parse_sql_url(rest, 5432)),
        "mysql" | "mariadb"                 => DatabaseConfig::Mysql(parse_sql_url(rest, 3306)),
        "mongodb" | "mongodb+srv"           => parse_mongo_url(url, rest),
        "sqlite" | "file"                   => sqlite_connection_with(rest.trim_start_matches('/')),
        _                                    => sqlite_connection_with(rest),
    }
}

/// Build a `DatabaseConfig` for a named driver (from discrete `DB_*` vars).
pub fn connection_for(driver: &str) -> DatabaseConfig {
    match driver.to_lowercase().as_str() {
        "postgres" | "postgresql" | "pgsql" => DatabaseConfig::Postgres(sql_config(5432, "postgres")),
        "mysql" | "mariadb"                 => DatabaseConfig::Mysql(sql_config(3306, "root")),
        "mongodb" | "mongo"                 => mongodb_connection(),
        _                                    => sqlite_connection(),
    }
}

// ── URL parsers ─────────────────────────────────────────────────────────────

/// Parse `user:pass@host:port/dbname?params` into a `SqlConfig`.
fn parse_sql_url(rest: &str, default_port: u16) -> SqlConfig {
    // Split credentials from the host part (last '@' separates them).
    let (creds, host_part) = match rest.rsplit_once('@') {
        Some((c, h)) => (c, h),
        None => ("", rest),
    };
    let (username, password) = match creds.split_once(':') {
        Some((u, p)) => (u.to_string(), p.to_string()),
        None if !creds.is_empty() => (creds.to_string(), String::new()),
        None => ("postgres".to_string(), String::new()),
    };

    // host[:port]/dbname?params
    let (host_port, db_and_query) = match host_part.split_once('/') {
        Some((hp, db)) => (hp, db),
        None => (host_part, ""),
    };
    let database = db_and_query
        .split(['?', '&'])
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or("lara")
        .to_string();

    let (host, port) = match host_port.split_once(':') {
        Some((h, p)) => (h.to_string(), p.parse().unwrap_or(default_port)),
        None => (host_port.to_string(), default_port),
    };

    let ssl = db_and_query.contains("sslmode=require") || db_and_query.contains("ssl=true");

    SqlConfig {
        host: if host.is_empty() { "localhost".into() } else { host },
        port,
        database,
        username,
        password,
        max_connections: env_u32("DB_MAX_CONNECTIONS", 20),
        min_connections: env_u32("DB_MIN_CONNECTIONS", 2),
        connect_timeout_secs: env_u64("DB_CONNECT_TIMEOUT", 30),
        ssl,
    }
}

/// Build a Mongo config from a full URI; the database is the URL path segment.
fn parse_mongo_url(full_url: &str, rest: &str) -> DatabaseConfig {
    let after_host = rest.split_once('/').map(|(_, db)| db).unwrap_or("");
    let database = after_host
        .split(['?', '&'])
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or("lara")
        .to_string();

    DatabaseConfig::Mongodb(MongoConfig {
        uri: full_url.to_string(),
        database,
        ..mongo_options()
    })
}

// ── discrete-var builders (fallback) ────────────────────────────────────────

fn sqlite_connection() -> DatabaseConfig {
    let path = std::env::var("DB_PATH").unwrap_or_else(|_| "database/app.db".to_string());
    sqlite_connection_with(&path)
}

fn sqlite_connection_with(path: &str) -> DatabaseConfig {
    DatabaseConfig::Sqlite(SqliteConfig {
        path: if path.is_empty() { "database/app.db".into() } else { path.to_string() },
        max_connections: env_u32("DB_MAX_CONNECTIONS", 5),
    })
}

fn mongodb_connection() -> DatabaseConfig {
    DatabaseConfig::Mongodb(MongoConfig {
        uri: std::env::var("DB_URI")
            .or_else(|_| std::env::var("MONGO_URI"))
            .unwrap_or_else(|_| {
                format!(
                    "mongodb://{}:{}",
                    std::env::var("DB_HOST").unwrap_or_else(|_| "127.0.0.1".into()),
                    env_u16("DB_PORT", 27017),
                )
            }),
        database: std::env::var("DB_DATABASE").unwrap_or_else(|_| "lara".to_string()),
        ..mongo_options()
    })
}

/// Shared MongoDB options read from the environment (auth, replica set, topology).
/// Mirrors the vest `db.config.ts` knobs.
fn mongo_options() -> MongoConfig {
    MongoConfig {
        uri: String::new(),
        database: String::new(),
        max_pool_size: env_u32("DB_MAX_CONNECTIONS", 20),
        min_pool_size: std::env::var("DB_MIN_CONNECTIONS").ok().and_then(|v| v.parse().ok()),
        // Authentication (optional)
        username: std::env::var("DB_USERNAME").ok().filter(|s| !s.is_empty()),
        password: std::env::var("DB_PASSWORD").ok().filter(|s| !s.is_empty()),
        auth_source: std::env::var("MONGO_AUTH_SOURCE").ok(),
        // Replica set / topology
        replica_set: std::env::var("MONGO_REPLICA_SET").ok().filter(|s| !s.is_empty()),
        direct_connection: std::env::var("MONGO_DIRECT_CONNECTION").ok().map(|v| v == "true"),
        retry_writes: std::env::var("MONGO_RETRY_WRITES").ok().map(|v| v == "true"),
        server_selection_timeout_ms: std::env::var("MONGO_SERVER_SELECTION_TIMEOUT_MS")
            .ok().and_then(|v| v.parse().ok()).or(Some(10_000)),
    }
}

fn sql_config(default_port: u16, default_user: &str) -> SqlConfig {
    SqlConfig {
        host: std::env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string()),
        port: env_u16("DB_PORT", default_port),
        database: std::env::var("DB_DATABASE").unwrap_or_else(|_| "lara".to_string()),
        username: std::env::var("DB_USERNAME").unwrap_or_else(|_| default_user.to_string()),
        password: std::env::var("DB_PASSWORD").unwrap_or_default(),
        max_connections: env_u32("DB_MAX_CONNECTIONS", 20),
        min_connections: env_u32("DB_MIN_CONNECTIONS", 2),
        connect_timeout_secs: env_u64("DB_CONNECT_TIMEOUT", 30),
        ssl: std::env::var("DB_SSL").map(|v| v == "true").unwrap_or(false),
    }
}

// ── tiny env helpers ────────────────────────────────────────────────────────

fn env_u16(key: &str, default: u16) -> u16 {
    std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}
fn env_u32(key: &str, default: u32) -> u32 {
    std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}
fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}
