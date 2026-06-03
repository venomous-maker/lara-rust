use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

use crate::{
    config::MailgunConfig,
    drivers::MailDriver,
    error::{MailError, Result},
    message::MailMessage,
};

pub struct MailgunDriver {
    client: Client,
    api_key: String,
    domain: String,
    endpoint: String,
}

impl MailgunDriver {
    pub fn new(cfg: &MailgunConfig) -> Self {
        Self {
            client: Client::new(),
            api_key: cfg.api_key.clone(),
            domain: cfg.domain.clone(),
            endpoint: cfg.endpoint.clone(),
        }
    }
}

#[async_trait]
impl MailDriver for MailgunDriver {
    async fn send(&self, msg: MailMessage) -> Result<()> {
        let url = format!("{}/{}/messages", self.endpoint, self.domain);

        let to: Vec<String> = msg.to.iter().map(|a| a.display()).collect();
        let cc: Vec<String> = msg.cc.iter().map(|a| a.display()).collect();
        let bcc: Vec<String> = msg.bcc.iter().map(|a| a.display()).collect();

        let mut form = reqwest::multipart::Form::new()
            .text("from", msg.from.display())
            .text("to", to.join(","))
            .text("subject", msg.subject.clone());

        if !cc.is_empty()  { form = form.text("cc", cc.join(",")); }
        if !bcc.is_empty() { form = form.text("bcc", bcc.join(",")); }

        if let Some(html) = &msg.html_body { form = form.text("html", html.clone()); }
        if let Some(text) = &msg.text_body { form = form.text("text", text.clone()); }

        for tag in &msg.tags { form = form.text("o:tag", tag.clone()); }

        for att in &msg.attachments {
            let part = reqwest::multipart::Part::bytes(att.content.clone())
                .file_name(att.filename.clone())
                .mime_str(&att.mime_type)
                .map_err(|e| MailError::Driver(e.to_string()))?;
            let key = if att.inline { "inline" } else { "attachment" };
            form = form.part(key, part);
        }

        let response = self.client
            .post(&url)
            .basic_auth("api", Some(&self.api_key))
            .multipart(form)
            .send()
            .await
            .map_err(|e| MailError::Driver(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(MailError::Driver(format!("Mailgun {} → {}", status, body)));
        }

        Ok(())
    }

    fn driver_name(&self) -> &'static str { "mailgun" }
}
