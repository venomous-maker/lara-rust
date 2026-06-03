use axum::{
    extract::{ConnectInfo, Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use serde_json::json;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use lara_cache::RateLimiter;

#[derive(Clone)]
pub struct ThrottleLayer {
    limiter: Arc<RateLimiter>,
    max_requests: u64,
    window: Duration,
}

impl ThrottleLayer {
    pub fn new(max_requests: u64, window: Duration) -> Self {
        Self {
            limiter: Arc::new(RateLimiter::new()),
            max_requests,
            window,
        }
    }

    /// `max_requests` per minute.
    pub fn per_minute(max_requests: u64) -> Self {
        Self::new(max_requests, Duration::from_secs(60))
    }

    /// `max_requests` per hour.
    pub fn per_hour(max_requests: u64) -> Self {
        Self::new(max_requests, Duration::from_secs(3600))
    }

    pub async fn check(
        State(layer): State<ThrottleLayer>,
        req: Request,
        next: Next,
    ) -> Response {
        // Key on IP address
        let ip = req
            .headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown")
            .to_string();

        let allowed = layer.limiter.attempt(&ip, layer.max_requests, layer.window).await;
        if allowed {
            next.run(req).await
        } else {
            (
                StatusCode::TOO_MANY_REQUESTS,
                Json(json!({"error": "Too many requests. Please slow down."})),
            ).into_response()
        }
    }
}
