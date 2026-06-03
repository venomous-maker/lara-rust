use std::env;
use lara_mail::{MailConfig, MailDriverKind, SmtpConfig, SmtpEncryption};

pub fn mail_config() -> MailConfig {
    let driver = match env::var("MAIL_DRIVER").as_deref() {
        Ok("smtp")     => MailDriverKind::Smtp,
        Ok("mailgun")  => MailDriverKind::Mailgun,
        Ok("sendgrid") => MailDriverKind::Sendgrid,
        Ok("null")     => MailDriverKind::Null,
        _              => MailDriverKind::Log,
    };

    MailConfig {
        driver,
        from_address: env::var("MAIL_FROM_ADDRESS")
            .unwrap_or_else(|_| "noreply@example.com".into()),
        from_name: env::var("MAIL_FROM_NAME")
            .unwrap_or_else(|_| "Lara App".into()),
        smtp: SmtpConfig {
            host: env::var("MAIL_HOST").unwrap_or_else(|_| "127.0.0.1".into()),
            port: env::var("MAIL_PORT").ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(1025),
            username: env::var("MAIL_USERNAME").ok(),
            password: env::var("MAIL_PASSWORD").ok(),
            encryption: match env::var("MAIL_ENCRYPTION").as_deref() {
                Ok("tls")      => SmtpEncryption::Tls,
                Ok("starttls") => SmtpEncryption::StartTls,
                _              => SmtpEncryption::None,
            },
        },
        ..MailConfig::default()
    }
}
