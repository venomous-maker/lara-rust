use async_trait::async_trait;
use mongodb::{
    bson::{doc, Bson, Document},
    options::{ClientOptions, FindOptions},
    Client,
};
use serde_json::Value as JsonValue;
use futures_util::TryStreamExt;

use super::{CompiledQuery, Driver, ExecResult, Grammar, MongoQuery};
use crate::{
    connection::config::MongoConfig,
    error::{DbError, Result},
};

pub struct MongoDriver {
    client: Client,
    db_name: String,
}

impl MongoDriver {
    pub async fn connect(cfg: &MongoConfig) -> Result<Self> {
        let mut opts = ClientOptions::parse(&cfg.uri).await?;
        opts.max_pool_size = Some(cfg.max_pool_size);
        let client = Client::with_options(opts)?;
        Ok(Self { client, db_name: cfg.database.clone() })
    }

    fn db(&self) -> mongodb::Database {
        self.client.database(&self.db_name)
    }

    fn collection(&self, name: &str) -> mongodb::Collection<Document> {
        self.db().collection(name)
    }
}

// ── JSON ↔ BSON helpers ──────────────────────────────────────────────────────

fn json_to_bson(v: JsonValue) -> Bson {
    bson::to_bson(&v).unwrap_or(Bson::Null)
}

fn json_to_doc(v: JsonValue) -> Document {
    match bson::to_document(&v) {
        Ok(d) => d,
        Err(_) => Document::new(),
    }
}

fn bson_to_json(bson: Bson) -> JsonValue {
    match bson {
        Bson::Null | Bson::Undefined => JsonValue::Null,
        Bson::Boolean(b) => JsonValue::Bool(b),
        Bson::Int32(n) => n.into(),
        Bson::Int64(n) => n.into(),
        Bson::Double(f) => serde_json::Number::from_f64(f)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null),
        Bson::String(s) => JsonValue::String(s),
        Bson::ObjectId(oid) => JsonValue::String(oid.to_hex()),
        Bson::DateTime(dt) => JsonValue::String(dt.to_string()),
        Bson::Document(doc) => doc_to_json(doc),
        Bson::Array(arr) => JsonValue::Array(arr.into_iter().map(bson_to_json).collect()),
        other => JsonValue::String(other.to_string()),
    }
}

fn doc_to_json(doc: Document) -> JsonValue {
    let mut map = serde_json::Map::new();
    for (k, v) in doc {
        let key = if k == "_id" { "id".to_string() } else { k };
        map.insert(key, bson_to_json(v));
    }
    JsonValue::Object(map)
}

// ── Driver implementation ─────────────────────────────────────────────────────

#[async_trait]
impl Driver for MongoDriver {
    // SQL methods are not supported for MongoDB
    async fn execute(&self, _q: CompiledQuery) -> Result<ExecResult> {
        Err(DbError::UnsupportedOperation("Use mongo_insert/update/delete for MongoDB".into()))
    }
    async fn fetch_all(&self, _q: CompiledQuery) -> Result<Vec<JsonValue>> {
        Err(DbError::UnsupportedOperation("Use mongo_find_all for MongoDB".into()))
    }
    async fn fetch_one(&self, _q: CompiledQuery) -> Result<Option<JsonValue>> {
        Err(DbError::UnsupportedOperation("Use mongo_find_one for MongoDB".into()))
    }

    // ── MongoDB operations ────────────────────────────────────────────────────

    async fn mongo_find_all(&self, q: MongoQuery) -> Result<Vec<JsonValue>> {
        let coll = self.collection(&q.collection);
        let filter = if q.filter.is_null() || q.filter == serde_json::json!({}) {
            Document::new()
        } else {
            json_to_doc(q.filter)
        };

        let mut find_opts = FindOptions::default();
        if let Some(sort) = q.sort {
            find_opts.sort = Some(json_to_doc(sort));
        }
        if let Some(limit) = q.limit {
            find_opts.limit = Some(limit);
        }
        if let Some(skip) = q.skip {
            find_opts.skip = Some(skip);
        }
        if let Some(proj) = q.projection {
            find_opts.projection = Some(json_to_doc(proj));
        }

        let mut cursor = coll.find(filter).with_options(find_opts).await?;
        let mut results = Vec::new();
        while let Some(doc) = cursor.try_next().await? {
            results.push(doc_to_json(doc));
        }
        Ok(results)
    }

    async fn mongo_find_one(&self, q: MongoQuery) -> Result<Option<JsonValue>> {
        let coll = self.collection(&q.collection);
        let filter = json_to_doc(q.filter);
        let doc = coll.find_one(filter).await?;
        Ok(doc.map(doc_to_json))
    }

    async fn mongo_insert(&self, collection: &str, doc: JsonValue) -> Result<String> {
        let coll = self.collection(collection);
        let bson_doc = json_to_doc(doc);
        let result = coll.insert_one(bson_doc).await?;
        Ok(result.inserted_id.to_string())
    }

    async fn mongo_update(
        &self,
        collection: &str,
        filter: JsonValue,
        update: JsonValue,
    ) -> Result<u64> {
        let coll = self.collection(collection);
        let filter_doc = json_to_doc(filter);
        // Wrap update in $set if not already an operator document
        let update_doc = if update.get("$set").is_some()
            || update.get("$unset").is_some()
            || update.get("$push").is_some()
        {
            json_to_doc(update)
        } else {
            doc! { "$set": json_to_doc(update) }
        };
        let result = coll.update_many(filter_doc, update_doc).await?;
        Ok(result.modified_count)
    }

    async fn mongo_delete(&self, collection: &str, filter: JsonValue) -> Result<u64> {
        let coll = self.collection(collection);
        let filter_doc = json_to_doc(filter);
        let result = coll.delete_many(filter_doc).await?;
        Ok(result.deleted_count)
    }

    async fn mongo_count(&self, collection: &str, filter: JsonValue) -> Result<u64> {
        let coll = self.collection(collection);
        let filter_doc = json_to_doc(filter);
        Ok(coll.count_documents(filter_doc).await?)
    }

    fn grammar(&self) -> Grammar { Grammar::Mongodb }
    fn driver_name(&self) -> &'static str { "mongodb" }
    fn is_mongodb(&self) -> bool { true }
}
