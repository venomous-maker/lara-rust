use async_trait::async_trait;
use crate::{drivers::MailDriver, error::Result, message::MailMessage};

/// Discards all mail — useful for tests.
pub struct NullDriver;

#[async_trait]
impl MailDriver for NullDriver {
    async fn send(&self, _msg: MailMessage) -> Result<()> { Ok(()) }
    fn driver_name(&self) -> &'static str { "null" }
}
