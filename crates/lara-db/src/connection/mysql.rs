use async_trait::async_trait;
use sqlx::{mysql::MySqlPoolOptions, MySqlPool, Row as SqlxRow};
use serde_json::Value as JsonValue;
use std::time::Duration;

use super::{CompiledQuery, Driver, ExecResult, Grammar};
use crate::{
    error::Result,
    connection::config::SqlConfig,
    value::Value,
};

pub struct MySqlDriver {
    pool: MySqlPool,
}

impl MySqlDriver {
    pub async fn connect(cfg: &SqlConfig) -> Result<Self> {
        let pool = MySqlPoolOptions::new()
            .max_connections(cfg.max_connections)
            .min_connections(cfg.min_connections)
            .acquire_timeout(Duration::from_secs(cfg.connect_timeout_secs))
            .connect(&cfg.mysql_url())
            .await?;
        Ok(Self { pool })
    }

    fn bind_query<'q>(
        sql: &'q str,
        params: &'q [Value],
    ) -> sqlx::query::Query<'q, sqlx::MySql, sqlx::mysql::MySqlArguments> {
        let mut q = sqlx::query(sql);
        for p in params {
            q = bind_value_my(q, p);
        }
        q
    }
}

fn bind_value_my<'q>(
    q: sqlx::query::Query<'q, sqlx::MySql, sqlx::mysql::MySqlArguments>,
    v: &'q Value,
) -> sqlx::query::Query<'q, sqlx::MySql, sqlx::mysql::MySqlArguments> {
    match v {
        Value::Null     => q.bind(Option::<String>::None),
        Value::Bool(b)  => q.bind(b),
        Value::Int(n)   => q.bind(*n),
        Value::Float(f) => q.bind(*f),
        Value::Text(s)  => q.bind(s.as_str()),
        Value::Bytes(b) => q.bind(b.as_slice()),
        Value::Json(j)  => q.bind(j.to_string()),
    }
}

fn row_to_json(row: &sqlx::mysql::MySqlRow) -> JsonValue {
    use sqlx::Column;
    let mut map = serde_json::Map::new();
    for col in row.columns() {
        let name = col.name().to_string();
        let val = mysql_col_to_json(row, col.ordinal());
        map.insert(name, val);
    }
    JsonValue::Object(map)
}

fn mysql_col_to_json(row: &sqlx::mysql::MySqlRow, idx: usize) -> JsonValue {
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

#[async_trait]
impl Driver for MySqlDriver {
    async fn execute(&self, q: CompiledQuery) -> Result<ExecResult> {
        let result = Self::bind_query(&q.sql, &q.params)
            .execute(&self.pool)
            .await?;
        Ok(ExecResult {
            rows_affected: result.rows_affected(),
            last_insert_id: Some(result.last_insert_id() as i64),
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

    fn grammar(&self) -> Grammar { Grammar::Mysql }
    fn driver_name(&self) -> &'static str { "mysql" }
}
