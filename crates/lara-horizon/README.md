# lara-horizon

A queue-monitoring dashboard for [Lara Rust](https://github.com/venomous-maker/lara-rust),
inspired by Laravel Horizon.

## Example

```rust
use std::sync::Arc;
use lara_horizon::{Horizon, QueueMetrics};

let horizon = Arc::new(Horizon::new());

horizon.record(QueueMetrics {
    queue: "emails".into(),
    size: 12,
    processed: 340,
    failed: 2,
    throughput_per_minute: 18.5,
}).await;

// Mount the JSON metrics endpoint at GET /horizon/metrics
let app = horizon.router();
```

Tracks per-queue size, processed/failed counts, and throughput, and exposes them
as JSON over an Axum router.

## License

MIT
