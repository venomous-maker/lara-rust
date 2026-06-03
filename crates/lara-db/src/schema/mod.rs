pub mod blueprint;
pub mod column;

use crate::{
    connection::CompiledQuery,
    db::Db,
    error::Result,
};
use blueprint::Blueprint;

/// Schema builder façade — all DDL is run through the global connection.
pub struct Schema;

impl Schema {
    /// CREATE TABLE.
    pub async fn create<F>(table: &str, f: F) -> Result<()>
    where F: FnOnce(&mut Blueprint),
    {
        let mut bp = Blueprint::create(table);
        f(&mut bp);
        let db = Db::connection();
        let grammar = db.grammar();
        for sql in bp.to_sql(grammar) {
            db.execute(CompiledQuery { sql, params: vec![] }).await?;
        }
        Ok(())
    }

    /// ALTER TABLE.
    pub async fn table<F>(table: &str, f: F) -> Result<()>
    where F: FnOnce(&mut Blueprint),
    {
        let mut bp = Blueprint::table(table);
        f(&mut bp);
        let db = Db::connection();
        let grammar = db.grammar();
        for sql in bp.to_sql(grammar) {
            db.execute(CompiledQuery { sql, params: vec![] }).await?;
        }
        Ok(())
    }

    pub async fn drop(table: &str) -> Result<()> {
        let db = Db::connection();
        let sql = format!("DROP TABLE IF EXISTS {}", table);
        db.execute(CompiledQuery { sql, params: vec![] }).await?;
        Ok(())
    }

    pub async fn drop_if_exists(table: &str) -> Result<()> {
        Self::drop(table).await
    }

    pub async fn has_table(table: &str) -> Result<bool> {
        let db = Db::connection();
        let grammar = db.grammar();
        let sql = match grammar {
            crate::connection::Grammar::Postgres | crate::connection::Grammar::Mysql => {
                format!("SELECT 1 FROM information_schema.tables WHERE table_name = '{}'", table)
            }
            crate::connection::Grammar::Sqlite => {
                format!("SELECT 1 FROM sqlite_master WHERE type='table' AND name='{}'", table)
            }
            crate::connection::Grammar::Mongodb => return Ok(true),
        };
        let row = db.fetch_one(CompiledQuery { sql, params: vec![] }).await?;
        Ok(row.is_some())
    }

    pub async fn rename(from: &str, to: &str) -> Result<()> {
        let db = Db::connection();
        let sql = format!("ALTER TABLE {} RENAME TO {}", from, to);
        db.execute(CompiledQuery { sql, params: vec![] }).await?;
        Ok(())
    }
}
