pub mod config;
pub mod postgres;
pub mod mysql;
pub mod sqlite;
#[cfg(feature = "mongodb")]
pub mod mongodb;

use std::sync::Arc;
use async_trait::async_trait;
use serde_json::Value as JsonValue;

use crate::error::Result;
use config::DatabaseConfig;

#[derive(Debug, Clone)]
pub struct ExecResult {
    pub rows_affected: u64,
    pub last_insert_id: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct CompiledQuery {
    pub sql: String,
    pub params: Vec<crate::value::Value>,
}

/// MongoDB query spec — passed to `mongo_*` driver methods.
#[derive(Debug, Clone, Default)]
pub struct MongoQuery {
    /// Collection name (= table name for models).
    pub collection: String,
    /// JSON filter document (converted from WhereClause).
    pub filter: JsonValue,
    /// JSON sort spec (e.g. `{"name": 1, "age": -1}`).
    pub sort: Option<JsonValue>,
    pub limit: Option<i64>,
    pub skip:  Option<u64>,
    /// Projection (fields to include/exclude).
    pub projection: Option<JsonValue>,
}

/// Low-level database driver — object-safe.
#[async_trait]
pub trait Driver: Send + Sync {
    // ── SQL path ──────────────────────────────────────────────────────────────

    async fn execute(&self, query: CompiledQuery) -> Result<ExecResult>;
    async fn fetch_all(&self, query: CompiledQuery) -> Result<Vec<JsonValue>>;
    async fn fetch_one(&self, query: CompiledQuery) -> Result<Option<JsonValue>>;

    // ── MongoDB path (default = UnsupportedOperation for SQL drivers) ─────────

    async fn mongo_find_all(&self, q: MongoQuery) -> Result<Vec<JsonValue>> {
        let _ = q;
        Err(crate::error::DbError::UnsupportedOperation(
            "This driver does not support MongoDB queries".into(),
        ))
    }

    async fn mongo_find_one(&self, q: MongoQuery) -> Result<Option<JsonValue>> {
        let _ = q;
        Err(crate::error::DbError::UnsupportedOperation(
            "This driver does not support MongoDB queries".into(),
        ))
    }

    async fn mongo_insert(&self, collection: &str, doc: JsonValue) -> Result<String> {
        let _ = (collection, doc);
        Err(crate::error::DbError::UnsupportedOperation(
            "This driver does not support MongoDB insert".into(),
        ))
    }

    async fn mongo_update(
        &self,
        collection: &str,
        filter: JsonValue,
        update: JsonValue,
    ) -> Result<u64> {
        let _ = (collection, filter, update);
        Err(crate::error::DbError::UnsupportedOperation(
            "This driver does not support MongoDB update".into(),
        ))
    }

    async fn mongo_delete(&self, collection: &str, filter: JsonValue) -> Result<u64> {
        let _ = (collection, filter);
        Err(crate::error::DbError::UnsupportedOperation(
            "This driver does not support MongoDB delete".into(),
        ))
    }

    async fn mongo_count(&self, collection: &str, filter: JsonValue) -> Result<u64> {
        let _ = (collection, filter);
        Err(crate::error::DbError::UnsupportedOperation(
            "This driver does not support MongoDB count".into(),
        ))
    }

    // ── Meta ──────────────────────────────────────────────────────────────────

    fn grammar(&self) -> Grammar;
    fn driver_name(&self) -> &'static str;
    fn is_mongodb(&self) -> bool { self.grammar() == Grammar::Mongodb }

    /// Downcast hook — lets callers reach a concrete driver (e.g. `MongoDriver`)
    /// for driver-specific features such as MongoDB sessions/transactions.
    fn as_any(&self) -> &dyn std::any::Any;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Grammar {
    Postgres,
    Mysql,
    Sqlite,
    Mongodb,
}

impl Grammar {
    pub fn placeholder(&self, index: usize) -> String {
        match self {
            Grammar::Postgres => format!("${}", index),
            Grammar::Mysql | Grammar::Sqlite => "?".to_string(),
            Grammar::Mongodb => String::new(),
        }
    }

    pub fn quote_ident(&self, ident: &str) -> String {
        match self {
            Grammar::Postgres | Grammar::Sqlite => format!("\"{}\"", ident),
            Grammar::Mysql => format!("`{}`", ident),
            Grammar::Mongodb => ident.to_string(),
        }
    }

    pub fn is_sql(&self) -> bool { !matches!(self, Grammar::Mongodb) }
}

pub type DbConnection = Arc<dyn Driver>;

pub async fn connect(cfg: &DatabaseConfig) -> Result<DbConnection> {
    match cfg {
        DatabaseConfig::Postgres(c) => {
            Ok(Arc::new(postgres::PostgresDriver::connect(c).await?))
        }
        DatabaseConfig::Mysql(c) => {
            Ok(Arc::new(mysql::MySqlDriver::connect(c).await?))
        }
        DatabaseConfig::Sqlite(c) => {
            Ok(Arc::new(sqlite::SqliteDriver::connect(c).await?))
        }
        #[cfg(feature = "mongodb")]
        DatabaseConfig::Mongodb(c) => {
            Ok(Arc::new(mongodb::MongoDriver::connect(c).await?))
        }
        #[cfg(not(feature = "mongodb"))]
        DatabaseConfig::Mongodb(_) => Err(crate::error::DbError::UnsupportedOperation(
            "MongoDB requires the `mongodb` feature".into(),
        )),
    }
}
