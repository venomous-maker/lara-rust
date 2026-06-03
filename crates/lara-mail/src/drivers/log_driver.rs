use async_trait::async_trait;
use crate::{drivers::MailDriver, error::Result, message::MailMessage};

/// Logs mail to the tracing output instead of sending — perfect for development.
pub struct LogDriver;

#[async_trait]
impl MailDriver for LogDriver {
    async fn send(&self, msg: MailMessage) -> Result<()> {
        let to: Vec<String> = msg.to.iter().map(|a| a.display()).collect();
        tracing::info!(
            driver = "log",
            from   = %msg.from.display(),
            to     = %to.join(", "),
            subject = %msg.subject,
            "[Mail] would send → subject: {}", msg.subject
        );
        if let Some(ref body) = msg.text_body {
            tracing::debug!("[Mail] Text body:\n{}", body);
        }
        if let Some(ref body) = msg.html_body {
            tracing::debug!("[Mail] HTML body ({}B)", body.len());
        }
        Ok(())
    }

    fn driver_name(&self) -> &'static str { "log" }
}
