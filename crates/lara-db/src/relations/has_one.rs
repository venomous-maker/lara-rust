use std::{collections::HashMap, marker::PhantomData};
use crate::{db::Db, error::Result, model::Model, query_builder::QueryBuilder, value::Value};

pub struct HasOne<Parent: Model, Related: Model> {
    parent_key: Value,
    foreign_key: String,
    _phantom: PhantomData<(Parent, Related)>,
}

impl<Parent: Model, Related: Model> HasOne<Parent, Related> {
    pub fn new(parent_key: Value, foreign_key: &str) -> Self {
        Self { parent_key, foreign_key: foreign_key.to_string(), _phantom: PhantomData }
    }

    pub async fn get(&self) -> Result<Option<Related>> {
        self.query().first().await
    }

    pub async fn create(&self, mut data: HashMap<String, Value>) -> Result<Related> {
        data.insert(self.foreign_key.clone(), self.parent_key.clone());
        QueryBuilder::<Related>::new(Related::table_name(), Related::primary_key_column())
            .do_insert_and_get(data)
            .await
    }

    pub fn query(&self) -> QueryBuilder<Related> {
        QueryBuilder::<Related>::new(Related::table_name(), Related::primary_key_column())
            .where_eq(&self.foreign_key, self.parent_key.clone())
    }
}
