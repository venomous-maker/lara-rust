use std::marker::PhantomData;
use crate::{error::Result, model::Model, query_builder::QueryBuilder, value::Value};

pub struct MorphOne<Owner: Model, Related: Model> {
    owner_key: Value,
    owner_type: String,
    morph_name: String,
    _phantom: PhantomData<(Owner, Related)>,
}

impl<Owner: Model, Related: Model> MorphOne<Owner, Related> {
    pub fn new(owner_key: Value, owner_type: &str, morph_name: &str) -> Self {
        Self { owner_key, owner_type: owner_type.to_string(), morph_name: morph_name.to_string(), _phantom: PhantomData }
    }

    pub async fn get(&self) -> Result<Option<Related>> { self.query().first().await }

    pub fn query(&self) -> QueryBuilder<Related> {
        QueryBuilder::<Related>::new(Related::table_name(), Related::primary_key_column())
            .where_eq(&format!("{}_id", self.morph_name), self.owner_key.clone())
            .where_eq(&format!("{}_type", self.morph_name), self.owner_type.as_str())
    }
}
