pub mod attributes;
pub mod events;

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

use crate::{
    error::{DbError, Result},
    query_builder::QueryBuilder,
    relations::{
        BelongsTo, BelongsToMany, HasMany, HasManyThrough,
        HasOne, HasOneThrough, MorphMany, MorphOne,
    },
    value::Value,
};

// ── ModelMeta ─────────────────────────────────────────────────────────────────

pub trait ModelMeta {
    fn table_name() -> &'static str;
    fn primary_key_column() -> &'static str { "id" }
    fn fillable_columns() -> &'static [&'static str] { &[] }
    fn hidden_columns() -> &'static [&'static str] { &[] }
    fn with_timestamps() -> bool { true }
    fn with_soft_deletes() -> bool { false }
}

// ── Model ─────────────────────────────────────────────────────────────────────

/// Central ORM trait — uses the global `Db::connection()` for all operations.
/// No database parameter is needed anywhere in the public API.
#[async_trait]
pub trait Model: ModelMeta + Serialize + DeserializeOwned + Clone + Send + Sync + 'static {

    fn query() -> QueryBuilder<Self> {
        QueryBuilder::new(Self::table_name(), Self::primary_key_column())
    }

    async fn all() -> Result<Vec<Self>> {
        Self::query().get().await
    }

    async fn find(id: impl Into<Value> + Send) -> Result<Option<Self>> {
        Self::query().where_eq(Self::primary_key_column(), id.into()).first().await
    }

    async fn find_or_fail(id: impl Into<Value> + Send) -> Result<Self> {
        Self::find(id).await?.ok_or(DbError::NotFound)
    }

    /// INSERT using the model struct directly as the data source.
    async fn create(data: Self) -> Result<Self> {
        let json = serde_json::to_value(&data)?;
        let obj = json.as_object().ok_or_else(|| DbError::Other("Not a JSON object".into()))?;
        let fillable = Self::fillable_columns();
        let pk = Self::primary_key_column();

        let mut row: HashMap<String, Value> = obj
            .iter()
            .filter(|(k, v)| {
                let is_pk = *k == pk;
                let allowed = fillable.is_empty() || fillable.contains(&k.as_str());
                let non_null = !v.is_null();
                allowed && non_null && !(is_pk && v.as_i64().map(|n| n == 0).unwrap_or(false))
            })
            .map(|(k, v)| (k.clone(), Value::from(v.clone())))
            .collect();

        if Self::with_timestamps() {
            let now = chrono::Utc::now().to_rfc3339();
            row.entry("created_at".into()).or_insert(Value::Text(now.clone()));
            row.entry("updated_at".into()).or_insert(Value::Text(now));
        }

        QueryBuilder::<Self>::new(Self::table_name(), pk)
            .do_insert_and_get(row)
            .await
    }

    async fn create_many(items: Vec<Self>) -> Result<Vec<Self>> {
        let mut out = Vec::with_capacity(items.len());
        for item in items { out.push(Self::create(item).await?); }
        Ok(out)
    }

    async fn save(&self) -> Result<()> {
        let json = serde_json::to_value(self)?;
        let obj = json.as_object().ok_or_else(|| DbError::Other("Not a JSON object".into()))?;
        let pk_col = Self::primary_key_column();
        let pk_val = obj.get(pk_col).cloned()
            .ok_or_else(|| DbError::Other(format!("Missing `{}`", pk_col)))?;

        let mut data: HashMap<String, Value> = obj
            .iter()
            .filter(|(k, _)| *k != pk_col)
            .map(|(k, v)| (k.clone(), Value::from(v.clone())))
            .collect();

        if Self::with_timestamps() {
            data.insert("updated_at".into(), Value::Text(chrono::Utc::now().to_rfc3339()));
        }

        QueryBuilder::<Self>::new(Self::table_name(), pk_col)
            .where_eq(pk_col, Value::from(pk_val))
            .update(data)
            .await?;
        Ok(())
    }

    async fn delete(&self) -> Result<()> {
        let pk = self.primary_key_value()?;
        let pk_col = Self::primary_key_column();
        if Self::with_soft_deletes() {
            QueryBuilder::<Self>::new(Self::table_name(), pk_col)
                .where_eq(pk_col, pk)
                .soft_delete()
                .await
        } else {
            QueryBuilder::<Self>::new(Self::table_name(), pk_col)
                .where_eq(pk_col, pk)
                .delete()
                .await
                .map(|_| ())
        }
    }

    async fn force_delete(&self) -> Result<()> {
        let pk = self.primary_key_value()?;
        let pk_col = Self::primary_key_column();
        QueryBuilder::<Self>::new(Self::table_name(), pk_col)
            .where_eq(pk_col, pk)
            .delete()
            .await
            .map(|_| ())
    }

    async fn restore(&self) -> Result<()> {
        let pk = self.primary_key_value()?;
        let pk_col = Self::primary_key_column();
        QueryBuilder::<Self>::new(Self::table_name(), pk_col)
            .where_eq(pk_col, pk)
            .restore()
            .await
    }

    // ── Serialization ─────────────────────────────────────────────────────────

    fn to_json(&self) -> Result<JsonValue> { Ok(serde_json::to_value(self)?) }

    fn to_json_public(&self) -> Result<JsonValue> {
        let mut v = self.to_json()?;
        if let Some(obj) = v.as_object_mut() {
            for col in Self::hidden_columns() { obj.remove(*col); }
        }
        Ok(v)
    }

    fn from_json_row(row: JsonValue) -> Result<Self> {
        Ok(serde_json::from_value(row)?)
    }

    fn primary_key_value(&self) -> Result<Value> {
        let json = serde_json::to_value(self)?;
        json.get(Self::primary_key_column())
            .cloned()
            .map(Value::from)
            .ok_or_else(|| DbError::Other(format!("Missing `{}`", Self::primary_key_column())))
    }

    /// Read any column's value from this model instance (as a [`Value`]).
    /// Returns [`Value::Null`] if the field is absent.
    fn attribute_value(&self, column: &str) -> Value {
        let json = serde_json::to_value(self).unwrap_or_default();
        json.get(column).cloned().map(Value::from).unwrap_or(Value::Null)
    }

    /// Resolve the "local key" value for a relationship:
    /// the named column when `local_key` is `Some`, otherwise the primary key.
    fn local_key_value(&self, local_key: Option<&str>) -> Value {
        match local_key {
            Some(col) => self.attribute_value(col),
            None => self.primary_key_value().unwrap_or(Value::Null),
        }
    }

    // ── Relationship factories ─────────────────────────────────────────────────
    //
    // Every factory accepts an optional `local_key`/`owner_key` column. Pass
    // `None` to use the primary key (the common case); pass `Some("col")` to
    // match against a different column instead.

    fn has_one<R: Model>(&self, fk: &str, local_key: Option<&str>) -> HasOne<Self, R>
    where Self: Sized {
        HasOne::new(self.local_key_value(local_key), fk)
    }

    fn has_many<R: Model>(&self, fk: &str, local_key: Option<&str>) -> HasMany<Self, R>
    where Self: Sized {
        HasMany::new(self.local_key_value(local_key), fk)
    }

    /// `owner_key` is the column on the *related* model to match the FK against
    /// (defaults to the related model's primary key).
    fn belongs_to<R: Model>(&self, fk: &str, owner_key: Option<&str>) -> BelongsTo<Self, R>
    where Self: Sized {
        let fk_val = self.attribute_value(fk);
        let owner_col = owner_key.unwrap_or(R::primary_key_column());
        BelongsTo::new(fk_val, owner_col)
    }

    fn belongs_to_many<R: Model>(
        &self,
        pivot: &str,
        lfk: &str,
        ffk: &str,
        local_key: Option<&str>,
    ) -> BelongsToMany<Self, R>
    where Self: Sized {
        BelongsToMany::new(self.local_key_value(local_key), pivot, lfk, ffk)
    }

    fn has_one_through<R: Model, I: Model>(
        &self,
        int_fk: &str,
        final_fk: &str,
        local_key: Option<&str>,
    ) -> HasOneThrough<Self, R, I>
    where Self: Sized {
        HasOneThrough::new(self.local_key_value(local_key), int_fk, final_fk)
    }

    fn has_many_through<R: Model, I: Model>(
        &self,
        int_fk: &str,
        final_fk: &str,
        local_key: Option<&str>,
    ) -> HasManyThrough<Self, R, I>
    where Self: Sized {
        HasManyThrough::new(self.local_key_value(local_key), int_fk, final_fk)
    }

    fn morph_one<R: Model>(&self, morph_name: &str, local_key: Option<&str>) -> MorphOne<Self, R>
    where Self: Sized {
        MorphOne::new(self.local_key_value(local_key), Self::table_name(), morph_name)
    }

    fn morph_many<R: Model>(&self, morph_name: &str, local_key: Option<&str>) -> MorphMany<Self, R>
    where Self: Sized {
        MorphMany::new(self.local_key_value(local_key), Self::table_name(), morph_name)
    }
}

impl<T> Model for T where
    T: ModelMeta + Serialize + DeserializeOwned + Clone + Send + Sync + 'static
{}
