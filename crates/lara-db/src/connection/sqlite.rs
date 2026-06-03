use async_trait::async_trait;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool, Row as SqlxRow};
use serde_json::Value as JsonValue;

use super::{CompiledQuery, Driver, ExecResult, Grammar};
use crate::{
    error::Result,
    connection::config::SqliteConfig,
    value::Value,
};

pub struct SqliteDriver {
    pool: SqlitePool,
}

impl SqliteDriver {
    pub async fn connect(cfg: &SqliteConfig) -> Result<Self> {
        let url = if cfg.path == ":memory:" {
            "sqlite::memory:".to_string()
        } else {
            format!("sqlite://{}", cfg.path)
        };
        let pool = SqlitePoolOptions::new()
            .max_connections(cfg.max_connections)
            .connect(&url)
            .await?;
        Ok(Self { pool })
    }

    fn bind_query<'q>(
        sql: &'q str,
        params: &'q [Value],
    ) -> sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>> {
        let mut q = sqlx::query(sql);
        for p in params {
            q = bind_value_sq(q, p);
        }
        q
    }
}

fn bind_value_sq<'q>(
    q: sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>>,
    v: &'q Value,
) -> sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>> {
    match v {
        Value::Null     => q.bind(Option::<String>::None),
        Value::Bool(b)  => q.bind(*b as i64),
        Value::Int(n)   => q.bind(*n),
        Value::Float(f) => q.bind(*f),
        Value::Text(s)  => q.bind(s.as_str()),
        Value::Bytes(b) => q.bind(b.as_slice()),
        Value::Json(j)  => q.bind(j.to_string()),
    }
}

fn row_to_json(row: &sqlx::sqlite::SqliteRow) -> JsonValue {
    use sqlx::Column;
    let mut map = serde_json::Map::new();
    for col in row.columns() {
        let name = col.name().to_string();
        let val = sq_col_to_json(row, col.ordinal());
        map.insert(name, val);
    }
    JsonValue::Object(map)
}

fn sq_col_to_json(row: &sqlx::sqlite::SqliteRow, idx: usize) -> JsonValue {
    if let Ok(v) = row.try_get::<Option<i64>, _>(idx) {
        return v.map(|n| JsonValue::Number(n.into())).unwrap_or(JsonValue::Null);
    }
    if let Ok(v) = row.try_get::<Option<f64>, _>(idx) {
        return v
            .and_then(|f| serde_json::Number::from_f64(f).map(JsonValue::Number))
            .unwrap_or(JsonValue::Null);
    }
    if let Ok(v) = row.try_get::<Option<String>, _>(idx) {
        return v.map(JsonValue::String).unwrap_or(JsonValue::Null);
    }
    JsonValue::Null
}

#[async_trait]
impl Driver for SqliteDriver {
    async fn execute(&self, q: CompiledQuery) -> Result<ExecResult> {
        let result = Self::bind_query(&q.sql, &q.params)
            .execute(&self.pool)
            .await?;
        Ok(ExecResult {
            rows_affected: result.rows_affected(),
            last_insert_id: Some(result.last_insert_rowid()),
        })
    }

    async fn fetch_all(&self, q: CompiledQuery) -> Result<Vec<JsonValue>> {
        let rows = Self::bind_query(&q.sql, &q.params)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.iter().map(row_to_json).collect())
    }

    async fn fetch_one(&self, q: CompiledQuery) -> Result<Option<JsonValue>> {
        let row = Self::bind_query(&q.sql, &q.params)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.as_ref().map(row_to_json))
    }

    fn grammar(&self) -> Grammar { Grammar::Sqlite }
    fn driver_name(&self) -> &'static str { "sqlite" }
    fn as_any(&self) -> &dyn std::any::Any { self }
}
