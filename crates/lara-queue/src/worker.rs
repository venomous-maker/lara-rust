use std::{collections::HashMap, sync::Arc, time::Duration};
use anyhow::Result;
use crate::{
    drivers::QueueDriver,
    job::{Job, JobPayload},
    queue::Queue,
};

type HandlerFn = Arc<
    dyn Fn(JobPayload) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>
        + Send + Sync,
>;

/// Processes jobs off a queue.
pub struct Worker {
    queue: Arc<Queue>,
    queue_name: String,
    handlers: HashMap<String, HandlerFn>,
    sleep_ms: u64,
}

impl Worker {
    pub fn new(driver: impl QueueDriver + 'static, queue_name: &str) -> Self {
        Self {
            queue: Arc::new(Queue::new(driver)),
            queue_name: queue_name.to_string(),
            handlers: HashMap::new(),
            sleep_ms: 1000,
        }
    }

    /// Register a handler for a specific job class name.
    pub fn register<J, F, Fut>(mut self, class_name: &str, f: F) -> Self
    where
        F: Fn(J) -> Fut + Send + Sync + 'static,
        J: Job,
        Fut: std::future::Future<Output = Result<()>> + Send + 'static,
    {
        let f = Arc::new(f);
        let handler: HandlerFn = Arc::new(move |payload: JobPayload| {
            let job: J = serde_json::from_str(&payload.payload).expect("Deserialize job");
            let fut = f(job);
            Box::pin(fut)
        });
        self.handlers.insert(class_name.to_string(), handler);
        self
    }

    pub fn sleep_ms(mut self, ms: u64) -> Self { self.sleep_ms = ms; self }

    /// Run the worker loop until `stop_signal` is resolved.
    pub async fn run(self, mut stop: tokio::sync::oneshot::Receiver<()>) {
        let q = self.queue.clone();
        let queue_name = self.queue_name.clone();
        let handlers = self.handlers;
        let sleep_ms = self.sleep_ms;

        loop {
            tokio::select! {
                _ = &mut stop => {
                    tracing::info!("Worker shutting down");
                    break;
                }
                _ = tokio::time::sleep(Duration::from_millis(sleep_ms)) => {
                    match q.pop(&queue_name).await {
                        Ok(Some(payload)) => {
                            if let Some(handler) = handlers.get(&payload.job_class) {
                                match handler(payload.clone()).await {
                                    Ok(_) => { let _ = q.ack(&payload).await; }
                                    Err(e) => {
                                        if payload.attempts >= payload.max_tries {
                                            let _ = q.fail(&payload, &e.to_string()).await;
                                        } else {
                                            let _ = q.release(&payload, 5).await;
                                        }
                                    }
                                }
                            } else {
                                tracing::warn!("No handler for job class: {}", payload.job_class);
                            }
                        }
                        Ok(None) => {}
                        Err(e) => tracing::error!("Queue pop error: {}", e),
                    }
                }
            }
        }
    }
}
