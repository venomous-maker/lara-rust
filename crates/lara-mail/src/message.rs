use serde::{Deserialize, Serialize};

/// A fully-built email message ready to hand to a driver.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailMessage {
    pub from: Address,
    pub reply_to: Option<Address>,
    pub to: Vec<Address>,
    pub cc: Vec<Address>,
    pub bcc: Vec<Address>,
    pub subject: String,
    pub html_body: Option<String>,
    pub text_body: Option<String>,
    pub attachments: Vec<Attachment>,
    pub headers: Vec<(String, String)>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub email: String,
    pub name: Option<String>,
}

impl Address {
    pub fn new(email: impl Into<String>, name: impl Into<Option<String>>) -> Self {
        Self { email: email.into(), name: name.into() }
    }

    pub fn from_email(email: impl Into<String>) -> Self {
        Self { email: email.into(), name: None }
    }

    pub fn display(&self) -> String {
        match &self.name {
            Some(n) => format!("{} <{}>", n, self.email),
            None    => self.email.clone(),
        }
    }
}

impl<S: Into<String>> From<S> for Address {
    fn from(s: S) -> Self {
        Self::from_email(s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub filename: String,
    pub content: Vec<u8>,
    pub mime_type: String,
    pub inline: bool,
    pub content_id: Option<String>,
}

impl Attachment {
    pub fn from_bytes(filename: &str, content: Vec<u8>, mime_type: &str) -> Self {
        Self {
            filename: filename.to_string(),
            content,
            mime_type: mime_type.to_string(),
            inline: false,
            content_id: None,
        }
    }

    pub fn inline(mut self, content_id: &str) -> Self {
        self.inline = true;
        self.content_id = Some(content_id.to_string());
        self
    }
}

/// Builder for constructing a `MailMessage` fluently.
pub struct MessageBuilder {
    inner: MailMessage,
}

impl MessageBuilder {
    pub fn new() -> Self {
        Self {
            inner: MailMessage {
                from: Address::from_email("noreply@example.com"),
                reply_to: None,
                to: Vec::new(),
                cc: Vec::new(),
                bcc: Vec::new(),
                subject: String::new(),
                html_body: None,
                text_body: None,
                attachments: Vec::new(),
                headers: Vec::new(),
                tags: Vec::new(),
            },
        }
    }

    pub fn from(mut self, email: impl Into<String>, name: impl Into<Option<String>>) -> Self {
        self.inner.from = Address::new(email, name.into()); self
    }

    pub fn reply_to(mut self, email: impl Into<String>) -> Self {
        self.inner.reply_to = Some(Address::from_email(email)); self
    }

    pub fn to(mut self, addr: impl Into<Address>) -> Self {
        self.inner.to.push(addr.into()); self
    }

    pub fn cc(mut self, addr: impl Into<Address>) -> Self {
        self.inner.cc.push(addr.into()); self
    }

    pub fn bcc(mut self, addr: impl Into<Address>) -> Self {
        self.inner.bcc.push(addr.into()); self
    }

    pub fn subject(mut self, s: impl Into<String>) -> Self {
        self.inner.subject = s.into(); self
    }

    pub fn html(mut self, body: impl Into<String>) -> Self {
        self.inner.html_body = Some(body.into()); self
    }

    pub fn text(mut self, body: impl Into<String>) -> Self {
        self.inner.text_body = Some(body.into()); self
    }

    pub fn attach(mut self, attachment: Attachment) -> Self {
        self.inner.attachments.push(attachment); self
    }

    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.inner.headers.push((key.into(), value.into())); self
    }

    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.inner.tags.push(tag.into()); self
    }

    pub fn build(self) -> MailMessage { self.inner }
}

impl Default for MessageBuilder {
    fn default() -> Self { Self::new() }
}
