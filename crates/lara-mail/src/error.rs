use thiserror::Error;

#[derive(Debug, Error)]
pub enum MailError {
    #[error("Mail driver error: {0}")]
    Driver(String),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Build error: {0}")]
    Build(String),

    #[error("Mail not configured — call Mailer::configure() at startup")]
    NotConfigured,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, MailError>;
