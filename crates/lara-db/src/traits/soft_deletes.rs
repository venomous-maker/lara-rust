use crate::{error::Result, model::Model};

pub trait SoftDeletes: Model {
    fn is_trashed(&self) -> bool;
    async fn delete_soft(&self) -> Result<()> { self.delete().await }
    async fn restore_record(&self) -> Result<()> { self.restore().await }
}
