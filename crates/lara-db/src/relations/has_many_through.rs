use std::marker::PhantomData;
use crate::{error::Result, model::Model, query_builder::QueryBuilder, value::Value};

pub struct HasManyThrough<Owner: Model, Final: Model, Intermediate: Model> {
    owner_key: Value,
    intermediate_fk: String,
    final_fk: String,
    _phantom: PhantomData<(Owner, Final, Intermediate)>,
}

impl<Owner: Model, Final: Model, Intermediate: Model> HasManyThrough<Owner, Final, Intermediate> {
    pub fn new(owner_key: Value, intermediate_fk: &str, final_fk: &str) -> Self {
        Self { owner_key, intermediate_fk: intermediate_fk.to_string(), final_fk: final_fk.to_string(), _phantom: PhantomData }
    }

    pub async fn get(&self) -> Result<Vec<Final>> { self.query().get().await }

    pub fn query(&self) -> QueryBuilder<Final> {
        let it = Intermediate::table_name();
        let ipk = Intermediate::primary_key_column();
        let ft = Final::table_name();
        let fpk = Final::primary_key_column();
        QueryBuilder::<Final>::new(ft, fpk)
            .select([format!("{}.*", ft)])
            .join(it, &format!("{}.{}", ft, &self.final_fk), &format!("{}.{}", it, ipk))
            .where_eq(&format!("{}.{}", it, &self.intermediate_fk), self.owner_key.clone())
    }
}
