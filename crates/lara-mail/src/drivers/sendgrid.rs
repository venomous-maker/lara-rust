use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

use crate::{
    config::SendgridConfig,
    drivers::MailDriver,
    error::{MailError, Result},
    message::{Address, MailMessage},
};

pub struct SendgridDriver {
    client: Client,
    api_key: String,
}

impl SendgridDriver {
    pub fn new(cfg: &SendgridConfig) -> Self {
        Self { client: Client::new(), api_key: cfg.api_key.clone() }
    }
}

fn addr_to_sg(a: &Address) -> Value {
    match &a.name {
        Some(n) => json!({ "email": a.email, "name": n }),
        None    => json!({ "email": a.email }),
    }
}

#[async_trait]
impl MailDriver for SendgridDriver {
    async fn send(&self, msg: MailMessage) -> Result<()> {
        let mut content: Vec<Value> = Vec::new();
        if let Some(t) = &msg.text_body {
            content.push(json!({ "type": "text/plain", "value": t }));
        }
        if let Some(h) = &msg.html_body {
            content.push(json!({ "type": "text/html", "value": h }));
        }

        let to: Vec<Value>  = msg.to.iter().map(addr_to_sg).collect();
        let cc: Vec<Value>  = msg.cc.iter().map(addr_to_sg).collect();
        let bcc: Vec<Value> = msg.bcc.iter().map(addr_to_sg).collect();

        let mut personalization = json!({ "to": to });
        if !cc.is_empty()  { personalization["cc"]  = json!(cc); }
        if !bcc.is_empty() { personalization["bcc"] = json!(bcc); }

        let mut body = json!({
            "personalizations": [personalization],
            "from": addr_to_sg(&msg.from),
            "subject": msg.subject,
            "content": content,
        });

        if let Some(rt) = &msg.reply_to {
            body["reply_to"] = addr_to_sg(rt);
        }

        if !msg.tags.is_empty() {
            body["categories"] = json!(msg.tags);
        }

        // Attachments
        if !msg.attachments.is_empty() {
            use base64::Engine;
            let engine = base64::engine::general_purpose::STANDARD;
            let atts: Vec<Value> = msg.attachments.iter().map(|a| {
                let mut att = json!({
                    "content": engine.encode(&a.content),
                    "filename": a.filename,
                    "type": a.mime_type,
                    "disposition": if a.inline { "inline" } else { "attachment" },
                });
                if let Some(cid) = &a.content_id { att["content_id"] = json!(cid); }
                att
            }).collect();
            body["attachments"] = json!(atts);
        }

        let response = self.client
            .post("https://api.sendgrid.com/v3/mail/send")
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| MailError::Driver(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(MailError::Driver(format!("SendGrid {} → {}", status, body)));
        }

        Ok(())
    }

    fn driver_name(&self) -> &'static str { "sendgrid" }
}
