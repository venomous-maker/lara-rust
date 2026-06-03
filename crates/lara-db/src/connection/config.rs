use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "driver", rename_all = "lowercase")]
pub enum DatabaseConfig {
    Postgres(SqlConfig),
    Mysql(SqlConfig),
    Sqlite(SqliteConfig),
    Mongodb(MongoConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqlConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    #[serde(default = "default_pool_size")]
    pub max_connections: u32,
    #[serde(default = "default_pool_size")]
    pub min_connections: u32,
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout_secs: u64,
    #[serde(default)]
    pub ssl: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqliteConfig {
    pub path: String,
    #[serde(default = "default_pool_size")]
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MongoConfig {
    pub uri: String,
    pub database: String,
    #[serde(default = "default_pool_size")]
    pub max_pool_size: u32,
}

impl SqlConfig {
    pub fn postgres_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}{}",
            self.username,
            self.password,
            self.host,
            self.port,
            self.database,
            if self.ssl { "?sslmode=require" } else { "" },
        )
    }

    pub fn mysql_url(&self) -> String {
        format!(
            "mysql://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database,
        )
    }
}

fn default_pool_size() -> u32 { 10 }
fn default_connect_timeout() -> u64 { 30 }
