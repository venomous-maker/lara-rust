use async_trait::async_trait;
use lara_mail::{config::MailConfig, mailable::{Envelope, Mailable}, message::{Address, MailMessage, MessageBuilder}};

/// Sent to new users immediately after registration.
pub struct WelcomeEmail {
    pub name: String,
    pub email: String,
}

#[async_trait]
impl Mailable for WelcomeEmail {
    fn envelope(&self, cfg: &MailConfig) -> Envelope {
        Envelope::new(
            Address::new(&self.email, Some(self.name.clone())),
            format!("Welcome to {}!", cfg.from_name),
        )
    }

    async fn build(&self) -> anyhow::Result<MailMessage> {
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<body style="font-family:sans-serif;max-width:600px;margin:auto;padding:20px">
  <h1>Welcome, {}! 🎉</h1>
  <p>Thank you for joining us. Your account is all set and ready to go.</p>
  <a href="https://example.com/dashboard"
     style="background:#4F46E5;color:#fff;padding:12px 24px;border-radius:6px;text-decoration:none;display:inline-block;margin-top:12px">
    Go to Dashboard
  </a>
  <p style="margin-top:32px;color:#6B7280;font-size:14px">
    If you didn't create this account, you can safely ignore this email.
  </p>
</body>
</html>"#,
            self.name
        );

        let text = format!(
            "Welcome, {}!\n\nThank you for joining us.\nVisit https://example.com/dashboard to get started.",
            self.name
        );

        Ok(MessageBuilder::new()
            .html(html)
            .text(text)
            .build())
    }

    async fn sent(&self) {
        tracing::info!(email = %self.email, "Welcome email sent to {}", self.name);
    }
}
