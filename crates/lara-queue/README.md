# lara-queue

Background job queues for [Lara Rust](https://github.com/venomous-maker/lara-rust).

## Drivers

- **Sync** — runs jobs inline (development / testing).
- **Database** — persists jobs to a table (in-process store by default).
- **Redis** — Redis-backed queue (feature `redis-driver`).

## Example

```rust
use lara_queue::{Job, JobMeta, QueueManager};
use lara_queue::drivers::sync::SyncDriver;
use lara_derive::Job;

#[derive(Serialize, Deserialize, Job)]
#[lara(queue = "emails", tries = 3, timeout = 30)]
struct SendEmail { to: String }

#[async_trait::async_trait]
impl Job for SendEmail {
    async fn handle(&self) -> anyhow::Result<()> { Ok(()) }
}

let manager = QueueManager::new(SyncDriver);
manager.dispatch(SendEmail { to: "a@b.com".into() }).await?;
```

A `Worker` processes jobs off a driver with retry/backoff handling.

## Feature flags

`database` (default), `redis-driver`.

## License

MIT
