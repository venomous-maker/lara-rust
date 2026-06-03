use lara_core::env::{env, env_or, env_or_parse};
use lara_mail::{MailConfig, MailDriverKind, SmtpConfig, SmtpEncryption};

pub fn mail_config() -> MailConfig {
    let driver = match env("MAIL_DRIVER").as_deref() {
        Some("smtp")     => MailDriverKind::Smtp,
        Some("mailgun")  => MailDriverKind::Mailgun,
        Some("sendgrid") => MailDriverKind::Sendgrid,
        Some("null")     => MailDriverKind::Null,
        _                => MailDriverKind::Log,
    };

    MailConfig {
        driver,
        from_address: env_or("MAIL_FROM_ADDRESS", "noreply@example.com"),
        from_name: env_or("MAIL_FROM_NAME", "Lara App"),
        smtp: SmtpConfig {
            host: env_or("MAIL_HOST", "127.0.0.1"),
            port: env_or_parse("MAIL_PORT", 1025),
            username: env("MAIL_USERNAME"),
            password: env("MAIL_PASSWORD"),
            encryption: match env("MAIL_ENCRYPTION").as_deref() {
                Some("tls")      => SmtpEncryption::Tls,
                Some("starttls") => SmtpEncryption::StartTls,
                _                => SmtpEncryption::None,
            },
        },
        ..MailConfig::default()
    }
}
