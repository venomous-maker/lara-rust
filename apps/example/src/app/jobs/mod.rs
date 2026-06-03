use async_trait::async_trait;
use lara_derive::Job;
use lara_queue::Job;
use serde::{Deserialize, Serialize};

use crate::app::mail::WelcomeEmail;
use lara_mail::Mailer;

/// Send an email in the background (queue: `emails`, 3 tries).
#[derive(Debug, Serialize, Deserialize, Job)]
#[lara(queue = "emails", tries = 3, timeout = 30)]
pub struct SendMailJob {
    pub to: String,
    pub name: String,
}

#[async_trait]
impl Job for SendMailJob {
    async fn handle(&self) -> anyhow::Result<()> {
        tracing::info!(to = %self.to, "Job: SendMailJob running");
        Mailer::send(WelcomeEmail {
            name: self.name.clone(),
            email: self.to.clone(),
        })
        .await?;
        Ok(())
    }

    async fn failed(&self, error: &str) {
        tracing::error!(to = %self.to, "SendMailJob permanently failed: {}", error);
    }
}

/// Generate a (potentially expensive) report (queue: `reports`, 2 tries, 5-min timeout).
#[derive(Debug, Serialize, Deserialize, Job)]
#[lara(queue = "reports", tries = 2, timeout = 300)]
pub struct GenerateReportJob {
    pub requested_by: i64,
    pub format: String,
}

#[async_trait]
impl Job for GenerateReportJob {
    async fn handle(&self) -> anyhow::Result<()> {
        tracing::info!(
            user_id = self.requested_by,
            format = %self.format,
            "Job: GenerateReportJob — building {} report", self.format
        );
        // ... heavy report-generation work would happen here ...
        Ok(())
    }
}

/// Periodic cleanup of stale data (queue: `default`, 1 try).
#[derive(Debug, Serialize, Deserialize, Job)]
#[lara(queue = "default", tries = 1, timeout = 120)]
pub struct CleanupJob {
    pub older_than_days: u32,
}

#[async_trait]
impl Job for CleanupJob {
    async fn handle(&self) -> anyhow::Result<()> {
        tracing::info!(days = self.older_than_days, "Job: CleanupJob — pruning old records");
        Ok(())
    }
}
