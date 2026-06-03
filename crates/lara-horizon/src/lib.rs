use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Snapshot of queue metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueueMetrics {
    pub queue: String,
    pub size: u64,
    pub processed: u64,
    pub failed: u64,
    pub throughput_per_minute: f64,
}

/// Horizon monitors queue health and exposes metrics.
pub struct Horizon {
    metrics: Arc<RwLock<Vec<QueueMetrics>>>,
}

impl Horizon {
    pub fn new() -> Self {
        Self { metrics: Arc::new(RwLock::new(Vec::new())) }
    }

    pub async fn record(&self, metrics: QueueMetrics) {
        let mut m = self.metrics.write().await;
        if let Some(existing) = m.iter_mut().find(|q| q.queue == metrics.queue) {
            *existing = metrics;
        } else {
            m.push(metrics);
        }
    }

    pub async fn all_metrics(&self) -> Vec<QueueMetrics> {
        self.metrics.read().await.clone()
    }

    pub async fn metrics_for(&self, queue: &str) -> Option<QueueMetrics> {
        self.metrics.read().await.iter().find(|q| q.queue == queue).cloned()
    }

    /// Axum handler — returns JSON metrics.
    pub async fn handle_metrics(
        axum::extract::State(horizon): axum::extract::State<Arc<Horizon>>,
    ) -> axum::Json<Vec<QueueMetrics>> {
        axum::Json(horizon.all_metrics().await)
    }

    /// Build an Axum router for the Horizon dashboard.
    pub fn router(self: Arc<Self>) -> axum::Router {
        use axum::routing::get;
        axum::Router::new()
            .route("/horizon/metrics", get(Self::handle_metrics))
            .with_state(self)
    }
}

impl Default for Horizon {
    fn default() -> Self { Self::new() }
}
