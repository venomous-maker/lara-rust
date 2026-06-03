use async_trait::async_trait;
use sqlx::{postgres::PgPoolOptions, PgPool, Row as SqlxRow};
use serde_json::Value as JsonValue;
use std::time::Duration;

use super::{CompiledQuery, Driver, ExecResult, Grammar};
use crate::{
    error::Result,
    connection::config::SqlConfig,
    value::Value,
};

pub struct PostgresDriver {
    pool: PgPool,
}

impl PostgresDriver {
    pub async fn connect(cfg: &SqlConfig) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(cfg.max_connections)
            .min_connections(cfg.min_connections)
            .acquire_timeout(Duration::from_secs(cfg.connect_timeout_secs))
            .connect(&cfg.postgres_url())
            .await?;
        Ok(Self { pool })
    }
}

fn pg_row_to_json(row: &sqlx::postgres::PgRow) -> JsonValue {
    use sqlx::Column;
    let mut map = serde_json::Map::new();
    for col in row.columns() {
        let name = col.name().to_string();
        let val = pg_col_to_json(row, col.ordinal());
        map.insert(name, val);
    }
    JsonValue::Object(map)
}

fn pg_col_to_json(row: &sqlx::postgres::PgRow, idx: usize) -> JsonValue {
    // Try each type in order — sqlx's typed getters are unambiguous
    if let Ok(v) = row.try_get::<Option<i64>, _>(idx) {
        return v.map(|n| JsonValue::Number(n.into())).unwrap_or(JsonValue::Null);
    }
    if let Ok(v) = row.try_get::<Option<f64>, _>(idx) {
        return v
            .and_then(|f| serde_json::Number::from_f64(f).map(JsonValue::Number))
            .unwrap_or(JsonValue::Null);
    }
    if let Ok(v) = row.try_get::<Option<bool>, _>(idx) {
        return v.map(JsonValue::Bool).unwrap_or(JsonValue::Null);
    }
    if let Ok(v) = row.try_get::<Option<String>, _>(idx) {
        return v.map(JsonValue::String).unwrap_or(JsonValue::Null);
    }
    JsonValue::Null
}

fn bind_pg<'q>(
    sql: &'q str,
    params: &'q [Value],
) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
    let mut q = sqlx::query(sql);
    for p in params {
        q = match p {
            Value::Null     => q.bind(Option::<String>::None),
            Value::Bool(b)  => q.bind(*b),
            Value::Int(n)   => q.bind(*n),
            Value::Float(f) => q.bind(*f),
            Value::Text(s)  => q.bind(s.as_str()),
            Value::Bytes(b) => q.bind(b.as_slice()),
            Value::Json(j)  => q.bind(sqlx::types::Json(j)),
        };
    }
    q
}

#[async_trait]
impl Driver for PostgresDriver {
    async fn execute(&self, q: CompiledQuery) -> Result<ExecResult> {
        let result = bind_pg(&q.sql, &q.params)
            .execute(&self.pool)
            .await?;
        Ok(ExecResult { rows_affected: result.rows_affected(), last_insert_id: None })
    }

    async fn fetch_all(&self, q: CompiledQuery) -> Result<Vec<JsonValue>> {
        let rows = bind_pg(&q.sql, &q.params)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.iter().map(pg_row_to_json).collect())
    }

    async fn fetch_one(&self, q: CompiledQuery) -> Result<Option<JsonValue>> {
        let row = bind_pg(&q.sql, &q.params)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.as_ref().map(pg_row_to_json))
    }

    fn grammar(&self) -> Grammar { Grammar::Postgres }
    fn driver_name(&self) -> &'static str { "postgres" }
    fn as_any(&self) -> &dyn std::any::Any { self }
}
