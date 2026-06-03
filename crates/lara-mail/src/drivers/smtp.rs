use async_trait::async_trait;
use lettre::{
    message::{header::ContentType, Mailbox, MultiPart, SinglePart},
    transport::smtp::{
        authentication::Credentials,
        client::{Tls, TlsParameters},
    },
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

use crate::{
    config::{SmtpConfig, SmtpEncryption},
    drivers::MailDriver,
    error::{MailError, Result},
    message::MailMessage,
};

pub struct SmtpDriver {
    transport: AsyncSmtpTransport<Tokio1Executor>,
}

impl SmtpDriver {
    pub fn new(cfg: &SmtpConfig) -> Result<Self> {
        let mut builder = match cfg.encryption {
            SmtpEncryption::Tls => AsyncSmtpTransport::<Tokio1Executor>::relay(&cfg.host)
                .map_err(|e| MailError::Driver(e.to_string()))?,
            SmtpEncryption::StartTls | SmtpEncryption::None => {
                AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&cfg.host)
            }
        };

        builder = builder.port(cfg.port);

        if let (Some(user), Some(pass)) = (&cfg.username, &cfg.password) {
            builder = builder.credentials(Credentials::new(user.clone(), pass.clone()));
        }

        Ok(Self { transport: builder.build() })
    }
}

#[async_trait]
impl MailDriver for SmtpDriver {
    async fn send(&self, msg: MailMessage) -> Result<()> {
        let from: Mailbox = msg.from.display().parse()
            .map_err(|e: lettre::address::AddressError| MailError::InvalidAddress(e.to_string()))?;

        let mut builder = Message::builder()
            .from(from)
            .subject(&msg.subject);

        for to in &msg.to {
            let m: Mailbox = to.display().parse()
                .map_err(|e: lettre::address::AddressError| MailError::InvalidAddress(e.to_string()))?;
            builder = builder.to(m);
        }
        for cc in &msg.cc {
            let m: Mailbox = cc.display().parse()
                .map_err(|e: lettre::address::AddressError| MailError::InvalidAddress(e.to_string()))?;
            builder = builder.cc(m);
        }
        for bcc in &msg.bcc {
            let m: Mailbox = bcc.display().parse()
                .map_err(|e: lettre::address::AddressError| MailError::InvalidAddress(e.to_string()))?;
            builder = builder.bcc(m);
        }
        if let Some(ref rt) = msg.reply_to {
            let m: Mailbox = rt.display().parse()
                .map_err(|e: lettre::address::AddressError| MailError::InvalidAddress(e.to_string()))?;
            builder = builder.reply_to(m);
        }

        let body = match (&msg.html_body, &msg.text_body) {
            (Some(html), Some(text)) => {
                builder.multipart(
                    MultiPart::alternative()
                        .singlepart(SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(text.clone()))
                        .singlepart(SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(html.clone()))
                )
            }
            (Some(html), None) => {
                builder.singlepart(SinglePart::builder()
                    .header(ContentType::TEXT_HTML)
                    .body(html.clone()))
            }
            (None, Some(text)) => {
                builder.singlepart(SinglePart::builder()
                    .header(ContentType::TEXT_PLAIN)
                    .body(text.clone()))
            }
            (None, None) => {
                builder.singlepart(SinglePart::builder()
                    .header(ContentType::TEXT_PLAIN)
                    .body(String::new()))
            }
        }.map_err(|e| MailError::Driver(e.to_string()))?;

        self.transport
            .send(body)
            .await
            .map_err(|e| MailError::Driver(e.to_string()))?;

        Ok(())
    }

    fn driver_name(&self) -> &'static str { "smtp" }
}
