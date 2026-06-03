# lara-mail

Mailables and mail delivery for [Lara Rust](https://github.com/venomous-maker/lara-rust).

## Drivers

- **Log** — logs messages via `tracing` (development default).
- **SMTP** — via [`lettre`](https://github.com/lettre/lettre) (feature `smtp`).
- **Mailgun** / **SendGrid** — HTTP API drivers (features `mailgun`, `sendgrid`).
- **Null** — discards mail (tests).

## Mailable classes

```rust
use lara_mail::{Mailer, Mailable, Envelope, MailMessage, MessageBuilder, Address, MailConfig};
use async_trait::async_trait;

struct WelcomeEmail { name: String, email: String }

#[async_trait]
impl Mailable for WelcomeEmail {
    fn envelope(&self, _cfg: &MailConfig) -> Envelope {
        Envelope::new(Address::new(&self.email, Some(self.name.clone())), "Welcome!")
    }
    async fn build(&self) -> anyhow::Result<MailMessage> {
        Ok(MessageBuilder::new().html("<h1>Hi!</h1>").text("Hi!").build())
    }
}

Mailer::configure(MailConfig::default())?;   // once at startup
Mailer::send(WelcomeEmail { name: "Ada".into(), email: "ada@example.com".into() }).await?;
```

## Fluent ad-hoc mail

```rust
Mailer::to("user@example.com")
    .subject("Hello")
    .html("<p>Hi</p>")
    .send()
    .await?;
```

## Feature flags

`smtp` (default), `log-driver` (default), `mailgun`, `sendgrid`, `templating`.

## License

MIT
