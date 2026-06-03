pub mod config;
pub mod drivers;
pub mod error;
pub mod mailable;
pub mod mailer;
pub mod message;

// Re-export the key public API
pub use config::{MailConfig, MailDriver as MailDriverKind, SmtpConfig, SmtpEncryption, MailgunConfig, SendgridConfig};
pub use error::{MailError, Result};
pub use mailable::{Envelope, Mailable};
pub use mailer::{Mailer, PendingMail};
pub use message::{Address, Attachment, MailMessage, MessageBuilder};
