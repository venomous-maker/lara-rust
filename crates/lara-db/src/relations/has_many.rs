use std::{collections::HashMap, marker::PhantomData};
use crate::{db::Db, error::Result, model::Model, query_builder::{pagination::Paginator, QueryBuilder}, value::Value};

pub struct HasMany<Parent: Model, Related: Model> {
    parent_key: Value,
    foreign_key: String,
    _phantom: PhantomData<(Parent, Related)>,
}

impl<Parent: Model, Related: Model> HasMany<Parent, Related> {
    pub fn new(parent_key: Value, foreign_key: &str) -> Self {
        Self { parent_key, foreign_key: foreign_key.to_string(), _phantom: PhantomData }
    }

    pub async fn get(&self) -> Result<Vec<Related>> { self.query().get().await }
    pub async fn count(&self) -> Result<u64> { self.query().count().await }
    pub async fn paginate(&self, per_page: u64, page: u64) -> Result<Paginator<Related>> {
        self.query().paginate(per_page, page).await
    }

    pub async fn create(&self, mut data: HashMap<String, Value>) -> Result<Related> {
        data.insert(self.foreign_key.clone(), self.parent_key.clone());
        QueryBuilder::<Related>::new(Related::table_name(), Related::primary_key_column())
            .do_insert_and_get(data)
            .await
    }

    pub async fn delete_all(&self) -> Result<u64> { self.query().delete().await }

    pub fn query(&self) -> QueryBuilder<Related> {
        QueryBuilder::<Related>::new(Related::table_name(), Related::primary_key_column())
            .where_eq(&self.foreign_key, self.parent_key.clone())
    }
}
