# lara-cache

Caching and rate limiting for [Lara Rust](https://github.com/venomous-maker/lara-rust).

## Drivers

- **Memory** — in-process `HashMap` with TTL expiry.
- **File** — JSON files on disk with TTL.
- **Redis** — Redis-backed (feature `redis-driver`).

## Example

```rust
use lara_cache::{Cache, MemoryCache};
use std::time::Duration;

let cache = MemoryCache::new();
cache.set("key", serde_json::json!({"hits": 1}), Some(Duration::from_secs(60))).await?;
let value = cache.get("key").await?;
```

## Rate limiting

```rust
use lara_cache::RateLimiter;
use std::time::Duration;

let limiter = RateLimiter::new();
if limiter.attempt("ip:1.2.3.4", 60, Duration::from_secs(60)).await {
    // within the limit
}
```

## Feature flags

`memory` (default), `file` (default), `redis-driver`.

## License

MIT
