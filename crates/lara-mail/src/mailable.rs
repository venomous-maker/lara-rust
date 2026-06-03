use async_trait::async_trait;
use crate::{
    config::MailConfig,
    message::{Address, MailMessage, MessageBuilder},
};

/// Implement `Mailable` on any struct to make it sendable via `Mailer::send()`.
///
/// # Example
/// ```rust
/// pub struct WelcomeEmail {
///     pub user_name: String,
///     pub to_address: String,
/// }
///
/// #[async_trait]
/// impl Mailable for WelcomeEmail {
///     fn envelope(&self, cfg: &MailConfig) -> Envelope {
///         Envelope {
///             to: vec![Address::new(&self.to_address, Some(self.user_name.clone()))],
///             subject: format!("Welcome, {}!", self.user_name),
///             from: Some(Address::new(&cfg.from_address, Some(cfg.from_name.clone()))),
///         }
///     }
///
///     async fn build(&self) -> anyhow::Result<MailMessage> {
///         Ok(MessageBuilder::new()
///             .html(format!("<h1>Welcome, {}!</h1>", self.user_name))
///             .text(format!("Welcome, {}!", self.user_name))
///             .build())
///     }
/// }
/// ```
#[async_trait]
pub trait Mailable: Send + Sync {
    /// Provides recipient addresses, subject, and optional from override.
    fn envelope(&self, cfg: &MailConfig) -> Envelope;

    /// Build the actual email message (HTML body, text body, attachments).
    async fn build(&self) -> anyhow::Result<MailMessage>;

    /// Called after the message is sent successfully.
    async fn sent(&self) {}

    /// Called when sending fails.
    async fn failed(&self, error: &str) {
        tracing::error!("Mail send failed: {}", error);
    }

    /// If `true`, this mail will be queued instead of sent inline.
    fn should_queue(&self) -> bool { false }

    /// Queue name to use when `should_queue` is true.
    fn queue(&self) -> &'static str { "default" }
}

/// Routing information extracted from a Mailable before building the message body.
#[derive(Debug, Clone)]
pub struct Envelope {
    /// Recipient(s).
    pub to: Vec<Address>,
    pub subject: String,
    /// Override the global from address (optional).
    pub from: Option<Address>,
    pub reply_to: Option<Address>,
    pub cc: Vec<Address>,
    pub bcc: Vec<Address>,
    pub tags: Vec<String>,
}

impl Envelope {
    pub fn new(to: impl Into<Address>, subject: impl Into<String>) -> Self {
        Self {
            to: vec![to.into()],
            subject: subject.into(),
            from: None,
            reply_to: None,
            cc: Vec::new(),
            bcc: Vec::new(),
            tags: Vec::new(),
        }
    }

    pub fn to_many(recipients: Vec<Address>, subject: impl Into<String>) -> Self {
        Self {
            to: recipients,
            subject: subject.into(),
            from: None,
            reply_to: None,
            cc: Vec::new(),
            bcc: Vec::new(),
            tags: Vec::new(),
        }
    }

    pub fn reply_to(mut self, addr: impl Into<Address>) -> Self {
        self.reply_to = Some(addr.into()); self
    }

    pub fn cc(mut self, addr: impl Into<Address>) -> Self {
        self.cc.push(addr.into()); self
    }

    pub fn bcc(mut self, addr: impl Into<Address>) -> Self {
        self.bcc.push(addr.into()); self
    }

    pub fn tag(mut self, t: impl Into<String>) -> Self {
        self.tags.push(t.into()); self
    }
}
