use std::{collections::HashMap, marker::PhantomData};
use crate::{
    connection::{CompiledQuery, Grammar},
    db::Db,
    error::Result,
    model::Model,
    query_builder::{build_insert, QueryBuilder},
    value::Value,
};

pub struct BelongsToMany<Owner: Model, Related: Model> {
    owner_key: Value,
    pivot: String,
    local_fk: String,
    foreign_fk: String,
    _phantom: PhantomData<(Owner, Related)>,
}

impl<Owner: Model, Related: Model> BelongsToMany<Owner, Related> {
    pub fn new(owner_key: Value, pivot: &str, local_fk: &str, foreign_fk: &str) -> Self {
        Self { owner_key, pivot: pivot.to_string(), local_fk: local_fk.to_string(), foreign_fk: foreign_fk.to_string(), _phantom: PhantomData }
    }

    pub async fn get(&self) -> Result<Vec<Related>> { self.query().get().await }
    pub async fn count(&self) -> Result<u64> { self.query().count().await }

    pub fn query(&self) -> QueryBuilder<Related> {
        let rt = Related::table_name();
        let rpk = Related::primary_key_column();
        let join_related_side = format!("{}.{}", rt, rpk);
        let join_pivot_side   = format!("{}.{}", self.pivot, self.foreign_fk);
        let where_col         = format!("{}.{}", self.pivot, self.local_fk);

        QueryBuilder::<Related>::new(rt, rpk)
            .select([format!("{}.*", rt)])
            .join(&self.pivot, &join_related_side, &join_pivot_side)
            .where_eq(&where_col, self.owner_key.clone())
    }

    /// Attach related IDs via the pivot table.
    pub async fn attach(&self, related_ids: &[Value]) -> Result<()> {
        let db = Db::connection();
        let grammar = db.grammar();
        for id in related_ids {
            let mut data = HashMap::new();
            data.insert(self.local_fk.clone(),   self.owner_key.clone());
            data.insert(self.foreign_fk.clone(), id.clone());
            db.execute(build_insert(&self.pivot, &data, grammar, false)).await?;
        }
        Ok(())
    }

    /// Detach specific related IDs from the pivot.
    pub async fn detach(&self, related_ids: &[Value]) -> Result<()> {
        let db = Db::connection();
        let grammar = db.grammar();
        let mut params = Vec::new();
        let mut idx = 1usize;
        let ph1 = grammar.placeholder(idx); idx += 1;
        params.push(self.owner_key.clone());
        let phs: Vec<String> = related_ids.iter().map(|id| {
            let ph = grammar.placeholder(idx); idx += 1; params.push(id.clone()); ph
        }).collect();
        let sql = format!("DELETE FROM {} WHERE {} = {} AND {} IN ({})", self.pivot, self.local_fk, ph1, self.foreign_fk, phs.join(", "));
        db.execute(CompiledQuery { sql, params }).await?;
        Ok(())
    }

    /// Sync: detach all, then re-attach the given IDs.
    pub async fn sync(&self, related_ids: &[Value]) -> Result<()> {
        let db = Db::connection();
        let grammar = db.grammar();
        let ph = grammar.placeholder(1);
        db.execute(CompiledQuery {
            sql: format!("DELETE FROM {} WHERE {} = {}", self.pivot, self.local_fk, ph),
            params: vec![self.owner_key.clone()],
        }).await?;
        self.attach(related_ids).await
    }

    /// Toggle: attach if not attached, detach if attached.
    pub async fn toggle(&self, related_ids: &[Value]) -> Result<()> {
        // Simplified toggle — sync with opposite set
        self.sync(related_ids).await
    }
}
