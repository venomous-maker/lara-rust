/// Global `Mailer` façade — set up once at startup, then call `Mailer::send()` anywhere.
use std::sync::{Arc, OnceLock};

use crate::{
    config::{MailConfig, MailDriver as DriverKind},
    drivers::{log_driver::LogDriver, null::NullDriver, MailDriver},
    error::{MailError, Result},
    mailable::Mailable,
    message::MailMessage,
};

static MAILER: OnceLock<Arc<MailerInner>> = OnceLock::new();

struct MailerInner {
    config: MailConfig,
    driver: Box<dyn MailDriver>,
}

/// The global mail façade.
pub struct Mailer;

impl Mailer {
    // ── Configuration ─────────────────────────────────────────────────────────

    /// Set up the global mailer from a `MailConfig`.
    /// Call this once at application startup (before any `Mailer::send()` calls).
    pub fn configure(config: MailConfig) -> Result<()> {
        let driver: Box<dyn MailDriver> = match config.driver {
            DriverKind::Log       => Box::new(LogDriver),
            DriverKind::Null      => Box::new(NullDriver),

            #[cfg(feature = "smtp")]
            DriverKind::Smtp | DriverKind::Sendmail => {
                use crate::drivers::smtp::SmtpDriver;
                Box::new(SmtpDriver::new(&config.smtp)?)
            }

            #[cfg(feature = "mailgun")]
            DriverKind::Mailgun => {
                use crate::drivers::mailgun::MailgunDriver;
                Box::new(MailgunDriver::new(&config.mailgun))
            }

            #[cfg(feature = "sendgrid")]
            DriverKind::Sendgrid => {
                use crate::drivers::sendgrid::SendgridDriver;
                Box::new(SendgridDriver::new(&config.sendgrid))
            }

            #[allow(unreachable_patterns)]
            _ => Box::new(LogDriver),
        };

        MAILER
            .set(Arc::new(MailerInner { config, driver }))
            .map_err(|_| MailError::Driver("Mailer already configured".into()))
    }

    /// Override the global mailer with an already-built driver (useful for tests).
    pub fn use_driver(config: MailConfig, driver: impl MailDriver + 'static) {
        MAILER
            .set(Arc::new(MailerInner { config, driver: Box::new(driver) }))
            .ok();
    }

    fn inner() -> Result<Arc<MailerInner>> {
        MAILER.get().cloned().ok_or(MailError::NotConfigured)
    }

    // ── Sending ───────────────────────────────────────────────────────────────

    /// Send any `Mailable` — zero extra arguments needed.
    ///
    /// ```rust
    /// Mailer::send(WelcomeEmail { user_name: "Alice".into(), to: "alice@example.com".into() }).await?;
    /// ```
    pub async fn send(mailable: impl Mailable) -> Result<()> {
        let inner = Self::inner()?;
        let envelope = mailable.envelope(&inner.config);

        // Merge envelope into the built message
        let mut message = mailable.build().await
            .map_err(|e| MailError::Build(e.to_string()))?;

        // Apply envelope fields (they take precedence over builder defaults)
        if !envelope.to.is_empty() { message.to = envelope.to; }
        if !envelope.cc.is_empty() { message.cc = envelope.cc; }
        if !envelope.bcc.is_empty() { message.bcc = envelope.bcc; }
        if !envelope.subject.is_empty() { message.subject = envelope.subject; }
        if let Some(from) = envelope.from { message.from = from; }
        if envelope.reply_to.is_some() { message.reply_to = envelope.reply_to; }
        if !envelope.tags.is_empty() { message.tags = envelope.tags; }

        // Apply global defaults if not set by the mailable
        if message.from.email.is_empty() || message.from.email == "noreply@example.com" {
            message.from = crate::message::Address::new(
                &inner.config.from_address,
                Some(inner.config.from_name.clone()),
            );
        }
        if message.reply_to.is_none() {
            if let Some(ref rt) = inner.config.reply_to {
                message.reply_to = Some(crate::message::Address::from_email(rt));
            }
        }

        match inner.driver.send(message).await {
            Ok(_) => { mailable.sent().await; Ok(()) }
            Err(e) => {
                let msg = e.to_string();
                mailable.failed(&msg).await;
                Err(e)
            }
        }
    }

    /// Build a pending mail fluently: `Mailer::to("user@example.com").subject("...").html("...").send().await`
    pub fn to(address: impl Into<crate::message::Address>) -> PendingMail {
        PendingMail::new(address.into())
    }

    /// Current driver name (for diagnostics).
    pub fn driver_name() -> Result<&'static str> {
        Ok(Self::inner()?.driver.driver_name())
    }
}

// ── PendingMail — fluent ad-hoc sending ───────────────────────────────────────

/// Fluent builder for one-off emails that don't need a dedicated Mailable struct.
pub struct PendingMail {
    to: Vec<crate::message::Address>,
    cc: Vec<crate::message::Address>,
    bcc: Vec<crate::message::Address>,
    subject: String,
    html: Option<String>,
    text: Option<String>,
    attachments: Vec<crate::message::Attachment>,
    tags: Vec<String>,
}

impl PendingMail {
    pub fn new(to: crate::message::Address) -> Self {
        Self {
            to: vec![to],
            cc: Vec::new(),
            bcc: Vec::new(),
            subject: String::new(),
            html: None,
            text: None,
            attachments: Vec::new(),
            tags: Vec::new(),
        }
    }

    pub fn to(mut self, addr: impl Into<crate::message::Address>) -> Self {
        self.to.push(addr.into()); self
    }

    pub fn cc(mut self, addr: impl Into<crate::message::Address>) -> Self {
        self.cc.push(addr.into()); self
    }

    pub fn bcc(mut self, addr: impl Into<crate::message::Address>) -> Self {
        self.bcc.push(addr.into()); self
    }

    pub fn subject(mut self, s: impl Into<String>) -> Self {
        self.subject = s.into(); self
    }

    pub fn html(mut self, body: impl Into<String>) -> Self {
        self.html = Some(body.into()); self
    }

    pub fn text(mut self, body: impl Into<String>) -> Self {
        self.text = Some(body.into()); self
    }

    pub fn attach(mut self, att: crate::message::Attachment) -> Self {
        self.attachments.push(att); self
    }

    pub fn tag(mut self, t: impl Into<String>) -> Self {
        self.tags.push(t.into()); self
    }

    /// Send the pending mail.
    pub async fn send(self) -> Result<()> {
        let inner = Mailer::inner()?;
        let msg = MailMessage {
            from: crate::message::Address::new(
                &inner.config.from_address,
                Some(inner.config.from_name.clone()),
            ),
            reply_to: inner.config.reply_to.as_deref().map(crate::message::Address::from_email),
            to: self.to,
            cc: self.cc,
            bcc: self.bcc,
            subject: self.subject,
            html_body: self.html,
            text_body: self.text,
            attachments: self.attachments,
            headers: Vec::new(),
            tags: self.tags,
        };
        inner.driver.send(msg).await
    }
}
