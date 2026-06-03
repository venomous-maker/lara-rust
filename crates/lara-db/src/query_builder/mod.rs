pub mod joins;
pub mod mongo_filter;
pub mod pagination;
pub mod where_clause;

use std::{collections::HashMap, marker::PhantomData};
use serde_json::Value as JsonValue;

use crate::{
    connection::{CompiledQuery, Grammar, MongoQuery},
    db::Db,
    error::{DbError, Result},
    model::Model,
    value::Value,
};
use joins::{Join, JoinType};
use mongo_filter::clauses_to_filter;
use pagination::Paginator;
use where_clause::{Boolean, HavingClause, Order, OrderByClause, WhereClause};

// ── QueryBuilder ──────────────────────────────────────────────────────────────

/// Fluent SQL/NoSQL query builder.  All execution methods use the **global connection**
/// (`Db::connection()`). No db parameter is ever required by the caller.
pub struct QueryBuilder<M: Model> {
    table: String,
    primary_key: String,
    select: Vec<String>,
    distinct: bool,
    where_clauses: Vec<WhereClause>,
    joins: Vec<Join>,
    order_by: Vec<OrderByClause>,
    group_by: Vec<String>,
    having: Vec<HavingClause>,
    limit: Option<u64>,
    offset: Option<u64>,
    with_trashed: bool,
    only_trashed: bool,
    eager_loads: Vec<String>,
    _phantom: PhantomData<M>,
}

impl<M: Model> QueryBuilder<M> {
    pub fn new(table: &str, primary_key: &str) -> Self {
        Self {
            table: table.to_string(),
            primary_key: primary_key.to_string(),
            select: vec!["*".to_string()],
            distinct: false,
            where_clauses: Vec::new(),
            joins: Vec::new(),
            order_by: Vec::new(),
            group_by: Vec::new(),
            having: Vec::new(),
            limit: None,
            offset: None,
            with_trashed: false,
            only_trashed: false,
            eager_loads: Vec::new(),
            _phantom: PhantomData,
        }
    }

    // ── SELECT ────────────────────────────────────────────────────────────────

    pub fn select<I, S>(mut self, cols: I) -> Self
    where I: IntoIterator<Item = S>, S: Into<String> {
        self.select = cols.into_iter().map(|c| c.into()).collect(); self
    }

    pub fn add_select(mut self, col: impl Into<String>) -> Self {
        if self.select == ["*"] { self.select.clear(); }
        self.select.push(col.into()); self
    }

    pub fn select_raw(mut self, expr: impl Into<String>) -> Self {
        self.select = vec![expr.into()]; self
    }

    pub fn distinct(mut self) -> Self { self.distinct = true; self }

    // ── WHERE ─────────────────────────────────────────────────────────────────

    pub fn where_eq(self, col: &str, val: impl Into<Value>) -> Self {
        self.add_where(col, "=", val.into(), Boolean::And)
    }
    pub fn where_ne(self, col: &str, val: impl Into<Value>) -> Self {
        self.add_where(col, "!=", val.into(), Boolean::And)
    }
    pub fn where_gt(self, col: &str, val: impl Into<Value>) -> Self {
        self.add_where(col, ">", val.into(), Boolean::And)
    }
    pub fn where_gte(self, col: &str, val: impl Into<Value>) -> Self {
        self.add_where(col, ">=", val.into(), Boolean::And)
    }
    pub fn where_lt(self, col: &str, val: impl Into<Value>) -> Self {
        self.add_where(col, "<", val.into(), Boolean::And)
    }
    pub fn where_lte(self, col: &str, val: impl Into<Value>) -> Self {
        self.add_where(col, "<=", val.into(), Boolean::And)
    }
    pub fn where_like(self, col: &str, pattern: impl Into<String>) -> Self {
        self.add_where(col, "LIKE", Value::Text(pattern.into()), Boolean::And)
    }
    pub fn where_not_like(self, col: &str, pattern: impl Into<String>) -> Self {
        self.add_where(col, "NOT LIKE", Value::Text(pattern.into()), Boolean::And)
    }
    pub fn or_where_eq(self, col: &str, val: impl Into<Value>) -> Self {
        self.add_where(col, "=", val.into(), Boolean::Or)
    }
    pub fn or_where_ne(self, col: &str, val: impl Into<Value>) -> Self {
        self.add_where(col, "!=", val.into(), Boolean::Or)
    }

    fn add_where(mut self, col: &str, op: &str, val: Value, boolean: Boolean) -> Self {
        self.where_clauses.push(WhereClause::Basic {
            column: col.to_string(), op: op.to_string(), value: val, boolean,
        });
        self
    }

    pub fn where_in<I: IntoIterator<Item = V>, V: Into<Value>>(mut self, col: &str, vals: I) -> Self {
        self.where_clauses.push(WhereClause::In {
            column: col.to_string(),
            values: vals.into_iter().map(|v| v.into()).collect(),
            negated: false, boolean: Boolean::And,
        });
        self
    }

    pub fn where_not_in<I: IntoIterator<Item = V>, V: Into<Value>>(mut self, col: &str, vals: I) -> Self {
        self.where_clauses.push(WhereClause::In {
            column: col.to_string(),
            values: vals.into_iter().map(|v| v.into()).collect(),
            negated: true, boolean: Boolean::And,
        });
        self
    }

    pub fn where_between(mut self, col: &str, min: impl Into<Value>, max: impl Into<Value>) -> Self {
        self.where_clauses.push(WhereClause::Between {
            column: col.to_string(), min: min.into(), max: max.into(), negated: false, boolean: Boolean::And,
        });
        self
    }

    pub fn where_not_between(mut self, col: &str, min: impl Into<Value>, max: impl Into<Value>) -> Self {
        self.where_clauses.push(WhereClause::Between {
            column: col.to_string(), min: min.into(), max: max.into(), negated: true, boolean: Boolean::And,
        });
        self
    }

    pub fn where_null(mut self, col: &str) -> Self {
        self.where_clauses.push(WhereClause::Null { column: col.to_string(), not_null: false, boolean: Boolean::And });
        self
    }

    pub fn where_not_null(mut self, col: &str) -> Self {
        self.where_clauses.push(WhereClause::Null { column: col.to_string(), not_null: true, boolean: Boolean::And });
        self
    }

    pub fn where_raw(mut self, sql: impl Into<String>, params: Vec<Value>) -> Self {
        self.where_clauses.push(WhereClause::Raw { sql: sql.into(), params, boolean: Boolean::And });
        self
    }

    pub fn or_where_raw(mut self, sql: impl Into<String>, params: Vec<Value>) -> Self {
        self.where_clauses.push(WhereClause::Raw { sql: sql.into(), params, boolean: Boolean::Or });
        self
    }

    pub fn where_group(mut self, f: impl FnOnce(QueryBuilder<M>) -> QueryBuilder<M>) -> Self {
        let inner = f(QueryBuilder::new(&self.table.clone(), &self.primary_key.clone()));
        if !inner.where_clauses.is_empty() {
            self.where_clauses.push(WhereClause::Nested { clauses: inner.where_clauses, boolean: Boolean::And });
        }
        self
    }

    // ── Soft deletes ──────────────────────────────────────────────────────────

    pub fn with_trashed(mut self) -> Self  { self.with_trashed = true; self }
    pub fn only_trashed(mut self) -> Self  { self.only_trashed = true; self }

    // ── JOINs ─────────────────────────────────────────────────────────────────

    pub fn join(self, table: &str, local: &str, foreign: &str) -> Self {
        self.add_join(JoinType::Inner, table, local, "=", foreign)
    }
    pub fn left_join(self, table: &str, local: &str, foreign: &str) -> Self {
        self.add_join(JoinType::Left, table, local, "=", foreign)
    }
    pub fn right_join(self, table: &str, local: &str, foreign: &str) -> Self {
        self.add_join(JoinType::Right, table, local, "=", foreign)
    }
    pub fn cross_join(self, table: &str) -> Self {
        self.add_join(JoinType::Cross, table, "", "=", "")
    }
    fn add_join(mut self, jt: JoinType, table: &str, local: &str, op: &str, foreign: &str) -> Self {
        self.joins.push(Join::new(jt, table, local, op, foreign)); self
    }

    // ── ORDER / GROUP / HAVING ────────────────────────────────────────────────

    pub fn order_by(mut self, col: &str, order: Order) -> Self {
        self.order_by.push(OrderByClause::Column { column: col.to_string(), order }); self
    }
    pub fn order_by_asc(self, col: &str) -> Self   { self.order_by(col, Order::Asc) }
    pub fn order_by_desc(self, col: &str) -> Self  { self.order_by(col, Order::Desc) }
    pub fn latest(self, col: &str) -> Self         { self.order_by_desc(col) }
    pub fn oldest(self, col: &str) -> Self         { self.order_by_asc(col) }
    pub fn order_by_raw(mut self, expr: impl Into<String>) -> Self {
        self.order_by.push(OrderByClause::Raw(expr.into())); self
    }

    pub fn group_by<I: IntoIterator<Item = S>, S: Into<String>>(mut self, cols: I) -> Self {
        self.group_by.extend(cols.into_iter().map(|c| c.into())); self
    }
    pub fn having(mut self, col: &str, op: &str, val: impl Into<Value>) -> Self {
        self.having.push(HavingClause::Basic { column: col.to_string(), op: op.to_string(), value: val.into() }); self
    }
    pub fn having_raw(mut self, expr: impl Into<String>) -> Self {
        self.having.push(HavingClause::Raw(expr.into())); self
    }

    // ── LIMIT / OFFSET ────────────────────────────────────────────────────────

    pub fn limit(mut self, n: u64) -> Self  { self.limit = Some(n); self }
    pub fn offset(mut self, n: u64) -> Self { self.offset = Some(n); self }
    pub fn take(self, n: u64) -> Self  { self.limit(n) }
    pub fn skip(self, n: u64) -> Self  { self.offset(n) }

    // ── Eager loading ─────────────────────────────────────────────────────────

    pub fn with(mut self, rel: impl Into<String>) -> Self {
        self.eager_loads.push(rel.into()); self
    }
    pub fn with_many<I: IntoIterator<Item = S>, S: Into<String>>(mut self, rels: I) -> Self {
        self.eager_loads.extend(rels.into_iter().map(|r| r.into())); self
    }

    // ── SQL compilation ───────────────────────────────────────────────────────

    fn compile(&self, grammar: Grammar) -> CompiledQuery {
        let mut params: Vec<Value> = Vec::new();
        let mut idx: usize = 1;
        let mut sql = String::new();

        let sel = if self.select.is_empty() { "*".to_string() } else { self.select.join(", ") };
        if self.distinct {
            sql.push_str(&format!("SELECT DISTINCT {} FROM {}", sel, self.table));
        } else {
            sql.push_str(&format!("SELECT {} FROM {}", sel, self.table));
        }

        for j in &self.joins {
            if j.join_type == JoinType::Cross {
                sql.push_str(&format!(" {} {}", j.join_type.sql_keyword(), j.table));
            } else {
                sql.push_str(&format!(" {} {} ON {} {} {}", j.join_type.sql_keyword(), j.table, j.on_local, j.on_operator, j.on_foreign));
            }
        }

        let mut all_where = self.where_clauses.clone();
        if M::with_soft_deletes() {
            if self.only_trashed {
                all_where.push(WhereClause::Null { column: "deleted_at".into(), not_null: true, boolean: Boolean::And });
            } else if !self.with_trashed {
                all_where.push(WhereClause::Null { column: "deleted_at".into(), not_null: false, boolean: Boolean::And });
            }
        }
        if !all_where.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&compile_where(&all_where, grammar, &mut params, &mut idx));
        }

        if !self.group_by.is_empty() {
            sql.push_str(&format!(" GROUP BY {}", self.group_by.join(", ")));
        }

        if !self.having.is_empty() {
            sql.push_str(" HAVING ");
            let parts: Vec<String> = self.having.iter().map(|h| match h {
                HavingClause::Basic { column, op, value } => {
                    let ph = grammar.placeholder(idx); idx += 1;
                    params.push(value.clone());
                    format!("{} {} {}", column, op, ph)
                }
                HavingClause::Raw(s) => s.clone(),
            }).collect();
            sql.push_str(&parts.join(" AND "));
        }

        if !self.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            let parts: Vec<String> = self.order_by.iter().map(|o| match o {
                OrderByClause::Column { column, order } => format!("{} {}", column, order.as_str()),
                OrderByClause::Raw(s) => s.clone(),
            }).collect();
            sql.push_str(&parts.join(", "));
        }

        if let Some(l) = self.limit  { sql.push_str(&format!(" LIMIT {}", l)); }
        if let Some(o) = self.offset { sql.push_str(&format!(" OFFSET {}", o)); }

        CompiledQuery { sql, params }
    }

    // ── Terminal queries — all use Db::connection() internally ────────────────

    /// Build a MongoDB MongoQuery from the current builder state.
    pub fn to_mongo_query(&self) -> MongoQuery {
        let filter = {
            let mut clauses = self.where_clauses.clone();
            if M::with_soft_deletes() && !self.with_trashed {
                clauses.push(where_clause::WhereClause::Null {
                    column: "deleted_at".into(),
                    not_null: self.only_trashed,
                    boolean: where_clause::Boolean::And,
                });
            }
            clauses_to_filter(&clauses)
        };

        let sort = if self.order_by.is_empty() {
            None
        } else {
            let mut sort_doc = serde_json::Map::new();
            for o in &self.order_by {
                match o {
                    where_clause::OrderByClause::Column { column, order } => {
                        sort_doc.insert(column.clone(), serde_json::json!(match order {
                            where_clause::Order::Asc  => 1,
                            where_clause::Order::Desc => -1,
                        }));
                    }
                    where_clause::OrderByClause::Raw(_) => {}
                }
            }
            Some(JsonValue::Object(sort_doc))
        };

        MongoQuery {
            collection: self.table.clone(),
            filter,
            sort,
            limit: self.limit.map(|l| l as i64),
            skip: self.offset,
            projection: None,
        }
    }

    /// Fetch all matching rows as typed models.
    pub async fn get(self) -> Result<Vec<M>> {
        let db = Db::connection();
        if db.is_mongodb() {
            let mq = self.to_mongo_query();
            return db.mongo_find_all(mq).await?
                .into_iter().map(M::from_json_row).collect();
        }
        let grammar = db.grammar();
        let q = self.compile(grammar);
        db.fetch_all(q).await?.into_iter().map(M::from_json_row).collect()
    }

    /// Fetch first matching row.
    pub async fn first(mut self) -> Result<Option<M>> {
        let db = Db::connection();
        if db.is_mongodb() {
            let mut mq = self.to_mongo_query();
            mq.limit = Some(1);
            return db.mongo_find_one(mq).await?
                .map(M::from_json_row).transpose();
        }
        self.limit = Some(1);
        let grammar = db.grammar();
        let q = self.compile(grammar);
        db.fetch_one(q).await?.map(M::from_json_row).transpose()
    }

    /// Fetch first or `DbError::NotFound`.
    pub async fn first_or_fail(self) -> Result<M> {
        self.first().await?.ok_or(DbError::NotFound)
    }

    /// Paginate results — returns `Paginator<M>`.
    pub async fn paginate(mut self, per_page: u64, page: u64) -> Result<Paginator<M>> {
        let db = Db::connection();
        if db.is_mongodb() {
            let filter = clauses_to_filter(&self.where_clauses);
            let total = db.mongo_count(&self.table, filter).await?;
            let mut mq = self.to_mongo_query();
            mq.limit = Some(per_page as i64);
            mq.skip  = Some((page.saturating_sub(1)) * per_page);
            let rows: Vec<M> = db.mongo_find_all(mq).await?
                .into_iter().map(M::from_json_row).collect::<Result<Vec<_>>>()?;
            return Ok(Paginator::new(rows, total, per_page, page));
        }
        let total = self.count_inner(&db).await?;
        self.limit = Some(per_page);
        self.offset = Some((page.saturating_sub(1)) * per_page);
        let grammar = db.grammar();
        let q = self.compile(grammar);
        let rows: Vec<M> = db.fetch_all(q).await?
            .into_iter().map(M::from_json_row).collect::<Result<Vec<_>>>()?;
        Ok(Paginator::new(rows, total, per_page, page))
    }

    /// COUNT(*) or MongoDB count.
    pub async fn count(self) -> Result<u64> {
        let db = Db::connection();
        if db.is_mongodb() {
            let filter = clauses_to_filter(&self.where_clauses);
            return db.mongo_count(&self.table, filter).await;
        }
        self.count_inner(&db).await
    }

    async fn count_inner(&self, db: &crate::connection::DbConnection) -> Result<u64> {
        let mut qb: QueryBuilder<M> = QueryBuilder::new(&self.table, &self.primary_key);
        qb.where_clauses = self.where_clauses.clone();
        qb.joins = self.joins.clone();
        qb.select = vec!["COUNT(*) as __count".to_string()];
        qb.with_trashed = self.with_trashed;
        qb.only_trashed = self.only_trashed;
        let grammar = db.grammar();
        let q = qb.compile(grammar);
        let row = db.fetch_one(q).await?;
        Ok(row.and_then(|v| v.get("__count").and_then(|n| n.as_u64())).unwrap_or(0))
    }

    /// SUM of a column.
    pub async fn sum(self, col: &str) -> Result<f64> { self.aggregate("SUM", col).await }
    /// AVG of a column.
    pub async fn avg(self, col: &str) -> Result<f64> { self.aggregate("AVG", col).await }
    /// MIN of a column.
    pub async fn min(self, col: &str) -> Result<f64> { self.aggregate("MIN", col).await }
    /// MAX of a column.
    pub async fn max(self, col: &str) -> Result<f64> { self.aggregate("MAX", col).await }

    async fn aggregate(mut self, func: &str, col: &str) -> Result<f64> {
        self.select = vec![format!("{}({}) as __agg", func, col)];
        let db = Db::connection();
        let grammar = db.grammar();
        let q = self.compile(grammar);
        let row = db.fetch_one(q).await?;
        Ok(row.and_then(|v| v.get("__agg").and_then(|n| n.as_f64())).unwrap_or(0.0))
    }

    pub async fn exists(self) -> Result<bool>      { Ok(self.count().await? > 0) }
    pub async fn doesnt_exist(self) -> Result<bool> { Ok(self.count().await? == 0) }

    // ── Mutations — all use Db::connection() internally ───────────────────────

    /// UPDATE matching rows (SQL) or $set update (MongoDB).
    pub async fn update(self, data: HashMap<String, Value>) -> Result<u64> {
        let db = Db::connection();
        if db.is_mongodb() {
            let filter = clauses_to_filter(&self.where_clauses);
            let update_doc: serde_json::Value = data.iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::from(v.clone())))
                .collect::<serde_json::Map<_, _>>()
                .into();
            return db.mongo_update(&self.table, filter, update_doc).await;
        }
        let grammar = db.grammar();
        let mut params = Vec::new();
        let mut idx = 1usize;
        let set_parts: Vec<String> = data.iter().map(|(col, val)| {
            let ph = grammar.placeholder(idx); idx += 1;
            params.push(val.clone());
            format!("{} = {}", col, ph)
        }).collect();
        let mut sql = format!("UPDATE {} SET {}", self.table, set_parts.join(", "));
        if !self.where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&compile_where(&self.where_clauses, grammar, &mut params, &mut idx));
        }
        let result = db.execute(CompiledQuery { sql, params }).await?;
        Ok(result.rows_affected)
    }

    /// Hard DELETE matching rows.
    pub async fn delete(self) -> Result<u64> {
        let db = Db::connection();
        if db.is_mongodb() {
            let filter = clauses_to_filter(&self.where_clauses);
            return db.mongo_delete(&self.table, filter).await;
        }
        let grammar = db.grammar();
        let mut params = Vec::new();
        let mut idx = 1usize;
        let mut sql = format!("DELETE FROM {}", self.table);
        if !self.where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&compile_where(&self.where_clauses, grammar, &mut params, &mut idx));
        }
        let result = db.execute(CompiledQuery { sql, params }).await?;
        Ok(result.rows_affected)
    }

    /// Soft-delete: set `deleted_at` = now().
    pub async fn soft_delete(self) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let mut data = HashMap::new();
        data.insert("deleted_at".to_string(), Value::Text(now));
        self.update(data).await?;
        Ok(())
    }

    /// Restore: set `deleted_at` = NULL.
    pub async fn restore(self) -> Result<()> {
        let mut data = HashMap::new();
        data.insert("deleted_at".to_string(), Value::Null);
        self.update(data).await?;
        Ok(())
    }

    /// Chunk through records to avoid loading everything into memory.
    pub async fn chunk<F>(self, size: u64, mut callback: F) -> Result<()>
    where
        F: FnMut(Vec<M>) -> bool,
    {
        let mut page = 1u64;
        loop {
            let mut qb = QueryBuilder::new(&self.table, &self.primary_key);
            qb.where_clauses = self.where_clauses.clone();
            qb.joins = self.joins.clone();
            qb.order_by = self.order_by.clone();
            qb.limit = Some(size);
            qb.offset = Some((page - 1) * size);
            let rows = qb.get().await?;
            let done = rows.len() < size as usize;
            if !callback(rows) || done { break; }
            page += 1;
        }
        Ok(())
    }

    /// Upsert: find by conditions, update if found, insert if not.
    pub async fn update_or_insert(
        conditions: HashMap<String, Value>,
        data: HashMap<String, Value>,
    ) -> Result<M> {
        let mut finder: QueryBuilder<M> = QueryBuilder::new(M::table_name(), M::primary_key_column());
        for (col, val) in &conditions { finder = finder.where_eq(col, val.clone()); }

        if finder.exists().await? {
            let mut updater: QueryBuilder<M> = QueryBuilder::new(M::table_name(), M::primary_key_column());
            for (col, val) in &conditions { updater = updater.where_eq(col, val.clone()); }
            updater.update(data).await?;

            let mut getter: QueryBuilder<M> = QueryBuilder::new(M::table_name(), M::primary_key_column());
            for (col, val) in &conditions { getter = getter.where_eq(col, val.clone()); }
            getter.first_or_fail().await
        } else {
            let mut full = data;
            for (col, val) in conditions { full.insert(col, val); }
            QueryBuilder::new(M::table_name(), M::primary_key_column())
                .do_insert_and_get(full)
                .await
        }
    }

    /// Insert and return the new record (called internally by Model::create).
    pub(crate) async fn do_insert_and_get(&self, data: HashMap<String, Value>) -> Result<M> {
        let db = Db::connection();

        // ── MongoDB path ─────────────────────────────────────────────────────
        if db.is_mongodb() {
            let json_doc: serde_json::Value = data.iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::from(v.clone())))
                .collect::<serde_json::Map<_, _>>()
                .into();
            let inserted_id = db.mongo_insert(&self.table, json_doc.clone()).await?;
            // Fetch back the inserted document
            let filter = serde_json::json!({ "_id": inserted_id });
            return db.mongo_find_one(MongoQuery {
                collection: self.table.clone(),
                filter,
                ..Default::default()
            }).await?
                .map(M::from_json_row)
                .ok_or(DbError::NotFound)?;
        }

        // ── SQL path ──────────────────────────────────────────────────────────
        let grammar = db.grammar();
        match grammar {
            Grammar::Postgres => {
                let q = build_insert(&self.table, &data, grammar, true);
                let row = db.fetch_one(q).await?.ok_or(DbError::NotFound)?;
                M::from_json_row(row)
            }
            _ => {
                let q = build_insert(&self.table, &data, grammar, false);
                let exec = db.execute(q).await?;
                if let Some(id) = exec.last_insert_id {
                    QueryBuilder::new(&self.table, &self.primary_key)
                        .where_eq(&self.primary_key, id)
                        .first_or_fail()
                        .await
                } else {
                    Err(DbError::Other("No last_insert_id returned".into()))
                }
            }
        }
    }
}

// ── SQL builder helpers ────────────────────────────────────────────────────────

pub(crate) fn compile_where(
    clauses: &[WhereClause],
    grammar: Grammar,
    params: &mut Vec<Value>,
    idx: &mut usize,
) -> String {
    let mut parts: Vec<String> = Vec::new();
    for (i, clause) in clauses.iter().enumerate() {
        let connector = if i == 0 { String::new() } else {
            match clause.boolean() {
                Boolean::And => "AND ".to_string(),
                Boolean::Or  => "OR ".to_string(),
            }
        };
        let expr = match clause {
            WhereClause::Basic { column, op, value, .. } => {
                let ph = grammar.placeholder(*idx); *idx += 1;
                params.push(value.clone());
                format!("{} {} {}", column, op, ph)
            }
            WhereClause::In { column, values, negated, .. } => {
                let phs: Vec<String> = values.iter().map(|v| {
                    let ph = grammar.placeholder(*idx); *idx += 1;
                    params.push(v.clone()); ph
                }).collect();
                format!("{} {} ({})", column, if *negated { "NOT IN" } else { "IN" }, phs.join(", "))
            }
            WhereClause::Between { column, min, max, negated, .. } => {
                let ph1 = grammar.placeholder(*idx); *idx += 1; params.push(min.clone());
                let ph2 = grammar.placeholder(*idx); *idx += 1; params.push(max.clone());
                format!("{} {} {} AND {}", column, if *negated { "NOT BETWEEN" } else { "BETWEEN" }, ph1, ph2)
            }
            WhereClause::Null { column, not_null, .. } => {
                if *not_null { format!("{} IS NOT NULL", column) }
                else         { format!("{} IS NULL", column) }
            }
            WhereClause::Raw { sql, params: rp, .. } => {
                params.extend(rp.iter().cloned());
                *idx += rp.len();
                sql.clone()
            }
            WhereClause::Nested { clauses: inner, .. } => {
                format!("({})", compile_where(inner, grammar, params, idx))
            }
        };
        parts.push(format!("{}{}", connector, expr));
    }
    parts.join(" ")
}

pub(crate) fn build_insert(
    table: &str,
    data: &HashMap<String, Value>,
    grammar: Grammar,
    returning: bool,
) -> CompiledQuery {
    let mut params = Vec::new();
    let mut idx = 1usize;
    let cols: Vec<&String> = data.keys().collect();
    let phs: Vec<String> = cols.iter().map(|_| { let ph = grammar.placeholder(idx); idx += 1; ph }).collect();
    for c in &cols { params.push(data[*c].clone()); }
    let col_str = cols.iter().map(|c| c.as_str()).collect::<Vec<_>>().join(", ");
    let ph_str = phs.join(", ");
    let mut sql = format!("INSERT INTO {} ({}) VALUES ({})", table, col_str, ph_str);
    if returning { sql.push_str(" RETURNING *"); }
    CompiledQuery { sql, params }
}
