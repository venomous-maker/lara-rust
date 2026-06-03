/// Global connection manager — holds named connections so that
/// models never need a `&DbConnection` argument.
///
/// Initialise once at application startup:
/// ```
/// lara_db::Db::configure(DatabaseConfig::Sqlite(...)).await?;
/// // or
/// lara_db::Db::configure_named("replica", config).await?;
/// ```
use std::{
    collections::HashMap,
    sync::{Arc, OnceLock, RwLock},
};

use crate::{
    connection::{connect, config::DatabaseConfig, DbConnection},
    error::{DbError, Result},
};

static MANAGER: OnceLock<Arc<DbManager>> = OnceLock::new();

struct DbManager {
    default: RwLock<Option<DbConnection>>,
    named: RwLock<HashMap<String, DbConnection>>,
}

impl DbManager {
    fn new() -> Self {
        Self {
            default: RwLock::new(None),
            named: RwLock::new(HashMap::new()),
        }
    }
}

fn manager() -> &'static Arc<DbManager> {
    MANAGER.get_or_init(|| Arc::new(DbManager::new()))
}

/// The `Db` façade — static methods only.
pub struct Db;

impl Db {
    /// Set (and connect) the default connection.
    pub async fn configure(cfg: DatabaseConfig) -> Result<()> {
        let conn = connect(&cfg).await?;
        let mgr = manager();
        *mgr.default.write().unwrap() = Some(conn);
        Ok(())
    }

    /// Add a named connection (e.g. `"read"`, `"write"`, `"analytics"`).
    pub async fn configure_named(name: &str, cfg: DatabaseConfig) -> Result<()> {
        let conn = connect(&cfg).await?;
        manager()
            .named
            .write()
            .unwrap()
            .insert(name.to_string(), conn);
        Ok(())
    }

    /// Register an already-built driver as the default connection.
    pub fn set_connection(conn: DbConnection) {
        *manager().default.write().unwrap() = Some(conn);
    }

    /// Get the default connection (panics if not configured).
    pub fn connection() -> DbConnection {
        manager()
            .default
            .read()
            .unwrap()
            .clone()
            .expect("No default database connection. Call `Db::configure()` at startup.")
    }

    /// Get a named connection.
    pub fn connection_named(name: &str) -> Result<DbConnection> {
        manager()
            .named
            .read()
            .unwrap()
            .get(name)
            .cloned()
            .ok_or_else(|| DbError::Connection(format!("No connection named `{}`", name)))
    }

    /// Execute a raw SQL statement on the default connection.
    pub async fn statement(sql: &str) -> Result<()> {
        let db = Self::connection();
        db.execute(crate::connection::CompiledQuery {
            sql: sql.to_string(),
            params: vec![],
        })
        .await?;
        Ok(())
    }

    /// Execute a raw SELECT on the default connection and return JSON rows.
    pub async fn select(sql: &str, params: Vec<crate::value::Value>) -> Result<Vec<serde_json::Value>> {
        let db = Self::connection();
        db.fetch_all(crate::connection::CompiledQuery {
            sql: sql.to_string(),
            params,
        })
        .await
    }

    /// Convenience: run a closure inside a manual transaction on the default connection.
    /// Uses `BEGIN` / `COMMIT` / `ROLLBACK` raw SQL (SQL databases only).
    pub async fn transaction<F, T, Fut>(f: F) -> Result<T>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T>> + Send,
        T: Send,
    {
        let db = Self::connection();
        db.execute(crate::connection::CompiledQuery { sql: "BEGIN".into(), params: vec![] }).await?;
        match f().await {
            Ok(v) => {
                db.execute(crate::connection::CompiledQuery { sql: "COMMIT".into(), params: vec![] }).await?;
                Ok(v)
            }
            Err(e) => {
                db.execute(crate::connection::CompiledQuery { sql: "ROLLBACK".into(), params: vec![] }).await.ok();
                Err(e)
            }
        }
    }
}
