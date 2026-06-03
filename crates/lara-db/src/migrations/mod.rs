use async_trait::async_trait;
use crate::{db::Db, error::Result, schema::Schema};

#[async_trait]
pub trait Migration: Send + Sync {
    fn name(&self) -> &'static str;
    async fn up(&self) -> Result<()>;
    async fn down(&self) -> Result<()>;
}

pub struct MigrationRunner {
    migrations: Vec<Box<dyn Migration>>,
}

impl MigrationRunner {
    pub fn new() -> Self { Self { migrations: Vec::new() } }

    pub fn add(mut self, m: impl Migration + 'static) -> Self {
        self.migrations.push(Box::new(m)); self
    }

    pub async fn run(&self) -> Result<()> {
        self.ensure_table().await?;
        for m in &self.migrations {
            if !self.has_run(m.name()).await? {
                tracing::info!("Running migration: {}", m.name());
                m.up().await?;
                self.mark_run(m.name()).await?;
            }
        }
        Ok(())
    }

    pub async fn rollback(&self) -> Result<()> {
        self.ensure_table().await?;
        let last = self.last_batch().await?;
        for m in self.migrations.iter().rev() {
            if self.in_batch(m.name(), last).await? {
                m.down().await?;
                self.mark_rolled_back(m.name()).await?;
            }
        }
        Ok(())
    }

    pub async fn reset(&self) -> Result<()> {
        self.ensure_table().await?;
        for m in self.migrations.iter().rev() {
            if self.has_run(m.name()).await? {
                m.down().await?;
                self.mark_rolled_back(m.name()).await?;
            }
        }
        Ok(())
    }

    async fn ensure_table(&self) -> Result<()> {
        if !Schema::has_table("migrations").await? {
            Schema::create("migrations", |t| {
                t.id();
                t.string("migration", 255);
                t.integer("batch");
                t.timestamps();
            }).await?;
        }
        Ok(())
    }

    async fn has_run(&self, name: &str) -> Result<bool> {
        let db = Db::connection();
        let grammar = db.grammar();
        let ph = grammar.placeholder(1);
        let sql = format!("SELECT COUNT(*) as cnt FROM migrations WHERE migration = {}", ph);
        let row = db.fetch_one(crate::connection::CompiledQuery {
            sql, params: vec![crate::value::Value::Text(name.to_string())],
        }).await?;
        Ok(row.and_then(|v| v.get("cnt").and_then(|n| n.as_u64())).unwrap_or(0) > 0)
    }

    async fn last_batch(&self) -> Result<i64> {
        let db = Db::connection();
        let row = db.fetch_one(crate::connection::CompiledQuery {
            sql: "SELECT MAX(batch) as mb FROM migrations".into(),
            params: vec![],
        }).await?;
        Ok(row.and_then(|v| v.get("mb").and_then(|n| n.as_i64())).unwrap_or(0))
    }

    async fn in_batch(&self, name: &str, batch: i64) -> Result<bool> {
        let db = Db::connection();
        let grammar = db.grammar();
        let ph1 = grammar.placeholder(1);
        let ph2 = grammar.placeholder(2);
        let sql = format!("SELECT COUNT(*) as cnt FROM migrations WHERE migration = {} AND batch = {}", ph1, ph2);
        let row = db.fetch_one(crate::connection::CompiledQuery {
            sql,
            params: vec![
                crate::value::Value::Text(name.to_string()),
                crate::value::Value::Int(batch),
            ],
        }).await?;
        Ok(row.and_then(|v| v.get("cnt").and_then(|n| n.as_u64())).unwrap_or(0) > 0)
    }

    async fn mark_run(&self, name: &str) -> Result<()> {
        let db = Db::connection();
        let batch = self.last_batch().await? + 1;
        let grammar = db.grammar();
        let ph1 = grammar.placeholder(1);
        let ph2 = grammar.placeholder(2);
        let now = chrono::Utc::now().to_rfc3339();
        let ph3 = grammar.placeholder(3);
        let ph4 = grammar.placeholder(4);
        db.execute(crate::connection::CompiledQuery {
            sql: format!("INSERT INTO migrations (migration, batch, created_at, updated_at) VALUES ({}, {}, {}, {})", ph1, ph2, ph3, ph4),
            params: vec![
                crate::value::Value::Text(name.to_string()),
                crate::value::Value::Int(batch),
                crate::value::Value::Text(now.clone()),
                crate::value::Value::Text(now),
            ],
        }).await?;
        Ok(())
    }

    async fn mark_rolled_back(&self, name: &str) -> Result<()> {
        let db = Db::connection();
        let grammar = db.grammar();
        let ph = grammar.placeholder(1);
        db.execute(crate::connection::CompiledQuery {
            sql: format!("DELETE FROM migrations WHERE migration = {}", ph),
            params: vec![crate::value::Value::Text(name.to_string())],
        }).await?;
        Ok(())
    }
}

impl Default for MigrationRunner { fn default() -> Self { Self::new() } }
