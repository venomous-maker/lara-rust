pub mod log_driver;
#[cfg(feature = "smtp")]
pub mod smtp;
#[cfg(feature = "mailgun")]
pub mod mailgun;
#[cfg(feature = "sendgrid")]
pub mod sendgrid;
pub mod null;

use async_trait::async_trait;
use crate::{error::Result, message::MailMessage};

/// A mail-sending backend.
#[async_trait]
pub trait MailDriver: Send + Sync {
    async fn send(&self, message: MailMessage) -> Result<()>;
    fn driver_name(&self) -> &'static str;
}
