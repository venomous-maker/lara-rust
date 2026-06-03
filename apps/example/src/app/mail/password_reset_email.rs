use async_trait::async_trait;
use lara_mail::{config::MailConfig, mailable::{Envelope, Mailable}, message::{Address, MailMessage, MessageBuilder}};

pub struct PasswordResetEmail {
    pub name: String,
    pub email: String,
    pub reset_url: String,
    /// Expiry in minutes.
    pub expires_in: u32,
}

#[async_trait]
impl Mailable for PasswordResetEmail {
    fn envelope(&self, _cfg: &MailConfig) -> Envelope {
        Envelope::new(
            Address::new(&self.email, Some(self.name.clone())),
            "Reset your password".to_string(),
        )
    }

    async fn build(&self) -> anyhow::Result<MailMessage> {
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<body style="font-family:sans-serif;max-width:600px;margin:auto;padding:20px">
  <h2>Password Reset Request</h2>
  <p>Hi {name},</p>
  <p>We received a request to reset your password. Click the link below to continue.
     This link expires in <strong>{expires} minutes</strong>.</p>
  <a href="{url}"
     style="background:#DC2626;color:#fff;padding:12px 24px;border-radius:6px;text-decoration:none;display:inline-block;margin:16px 0">
    Reset Password
  </a>
  <p style="color:#6B7280;font-size:14px">
    If you didn't request a password reset, ignore this email. Your password will remain unchanged.
  </p>
</body>
</html>"#,
            name = self.name,
            expires = self.expires_in,
            url = self.reset_url,
        );

        let text = format!(
            "Hi {},\n\nReset your password: {}\n\nThis link expires in {} minutes.\n\nIf you didn't request this, ignore this email.",
            self.name, self.reset_url, self.expires_in
        );

        Ok(MessageBuilder::new().html(html).text(text).build())
    }
}
