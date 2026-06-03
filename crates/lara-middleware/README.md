# lara-middleware

Reusable HTTP middleware for [Lara Rust](https://github.com/venomous-maker/lara-rust),
built for [Axum](https://github.com/tokio-rs/axum) / [Tower](https://github.com/tower-rs/tower).

## What's inside

- **CORS** — `cors_layer()` (permissive) and `cors_for_origins(&[..])` (restrictive).
- **Throttle** — IP-based rate limiting backed by `lara-cache`'s `RateLimiter`.
- **Auth** — `AuthLayer` for `Bearer` JWT verification, injecting claims into
  request extensions.

## Example

```rust
use lara_middleware::{cors_layer, ThrottleLayer, AuthLayer};
use std::time::Duration;

let throttle = ThrottleLayer::per_minute(60);
let auth = AuthLayer::new(jwt_secret);

let app = router
    .layer(cors_layer());
```

## License

MIT
