use std::marker::PhantomData;
use crate::{error::Result, model::Model, query_builder::QueryBuilder, value::Value};

pub struct BelongsTo<Child: Model, Parent: Model> {
    fk_value: Value,
    parent_pk_col: String,
    _phantom: PhantomData<(Child, Parent)>,
}

impl<Child: Model, Parent: Model> BelongsTo<Child, Parent> {
    pub fn new(fk_value: Value, parent_pk_col: &str) -> Self {
        Self { fk_value, parent_pk_col: parent_pk_col.to_string(), _phantom: PhantomData }
    }

    pub async fn get(&self) -> Result<Option<Parent>> { self.query().first().await }

    pub fn query(&self) -> QueryBuilder<Parent> {
        QueryBuilder::<Parent>::new(Parent::table_name(), Parent::primary_key_column())
            .where_eq(&self.parent_pk_col, self.fk_value.clone())
    }
}
