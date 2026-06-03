use serde::{Deserialize, Serialize};

/// Which mail driver to use.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MailDriver {
    Smtp,
    Sendmail,
    Mailgun,
    Sendgrid,
    Log,
    /// Silently discard all mail (useful for tests).
    Null,
}

impl Default for MailDriver {
    fn default() -> Self { Self::Log }
}

/// Root mail configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailConfig {
    #[serde(default)]
    pub driver: MailDriver,

    /// SMTP settings — used when `driver = "smtp"`.
    #[serde(default)]
    pub smtp: SmtpConfig,

    /// Mailgun settings — used when `driver = "mailgun"`.
    #[serde(default)]
    pub mailgun: MailgunConfig,

    /// SendGrid settings — used when `driver = "sendgrid"`.
    #[serde(default)]
    pub sendgrid: SendgridConfig,

    /// Default "from" address for all outgoing mail.
    pub from_address: String,
    /// Default "from" display name.
    #[serde(default = "default_from_name")]
    pub from_name: String,

    /// Optional global "reply-to" address.
    pub reply_to: Option<String>,
}

impl Default for MailConfig {
    fn default() -> Self {
        Self {
            driver: MailDriver::Log,
            smtp: SmtpConfig::default(),
            mailgun: MailgunConfig::default(),
            sendgrid: SendgridConfig::default(),
            from_address: "noreply@example.com".to_string(),
            from_name: default_from_name(),
            reply_to: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub encryption: SmtpEncryption,
}

impl Default for SmtpConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 1025,
            username: None,
            password: None,
            encryption: SmtpEncryption::None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SmtpEncryption {
    #[default]
    None,
    Tls,
    StartTls,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MailgunConfig {
    pub api_key: String,
    pub domain: String,
    #[serde(default = "default_mailgun_endpoint")]
    pub endpoint: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SendgridConfig {
    pub api_key: String,
}

fn default_from_name()         -> String { "Lara App".to_string() }
fn default_mailgun_endpoint()  -> String { "https://api.mailgun.net/v3".to_string() }
