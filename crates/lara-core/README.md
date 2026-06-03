# lara-core

The foundation of [Lara Rust](https://github.com/venomous-maker/lara-rust): the IoC
container, application lifecycle, and service-provider system.

## What's inside

- **`Container`** — a dependency-injection container with `bind`, `singleton`,
  `instance`, and `alias`; resolve typed values with `make::<T>()`.
- **`Application`** — wires configuration, providers, and an Axum router, then
  drives the `register` → `boot` → `serve` lifecycle.
- **`ServiceProvider`** — the two-phase (`register`/`boot`) extension point every
  package and app feature plugs into.
- **`Config`** — a dot-notation configuration store (`database.connections.pg.host`).

## Example

```rust
use lara_core::{Application, ServiceProvider, Container, Config, Result};
use async_trait::async_trait;

struct AppProvider;

#[async_trait]
impl ServiceProvider for AppProvider {
    fn name(&self) -> &'static str { "AppProvider" }
    async fn register(&self, c: &mut Container, _cfg: &Config) -> Result<()> {
        c.singleton("clock", |_| std::time::Instant::now());
        Ok(())
    }
}

let mut app = Application::new().register(AppProvider);
app.boot().await?;
```

## License

MIT
