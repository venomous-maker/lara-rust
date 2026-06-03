# Lara Rust 🦀

A Laravel-inspired web framework for Rust — a port of the [vest](https://github.com/) Node.js
framework. Built as a Cargo workspace (monorepo) of focused crates plus a full example app.

## Workspace layout

```
lara-rust/
├── crates/
│   ├── lara-core         # IoC container, Application lifecycle, ServiceProvider
│   ├── lara-derive       # #[derive(Model, Job, Command)] proc-macros
│   ├── lara-db           # Eloquent-style ORM (Postgres / MySQL / SQLite / MongoDB)
│   ├── lara-router       # Axum-based router, FormRequest + Validated extractor
│   ├── lara-validator    # 50+ validation rules
│   ├── lara-events       # Event dispatcher + listeners
│   ├── lara-queue        # Job queue (sync / database / redis drivers)
│   ├── lara-cache        # Cache (memory / file / redis) + rate limiter
│   ├── lara-carbon       # Fluent date/time (Carbon equivalent)
│   ├── lara-scheduler    # Cron-style task scheduler
│   ├── lara-horizon      # Queue monitoring dashboard
│   ├── lara-middleware   # CORS, throttle, JWT auth middleware
│   └── lara-mail         # Mailable classes + SMTP/Mailgun/SendGrid/Log drivers
└── apps/
    └── example           # Full reference application
```

## The ORM

```rust
use lara_db::{Db, ModelTrait};
use lara_derive::Model;

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[lara(table = "users", primary_key = "id", soft_deletes)]
pub struct User {
    pub id: Option<i64>,
    pub name: String,
    pub email: String,
    #[lara(hidden)]
    pub password: String,
}

// Configure once at startup — no `db` parameter needed anywhere after.
Db::configure(config).await?;

// Query builder returns typed models
let active = User::query().where_eq("status", "active").get().await?;

// Relationships (optional local key — None = use primary key)
let roles = user.belongs_to_many::<Role>("role_user", "user_id", "role_id", None).get().await?;
```

Works across **PostgreSQL, MySQL, SQLite, and MongoDB** — the query builder compiles to
SQL or BSON automatically based on the active connection.

## The example app

The `apps/example` crate demonstrates the full stack:

- **Service providers** with `register` → `boot` phases (`app/providers/`)
- **Dependency injection** — services composed from singletons in the DI container
- **Listeners / events**, **jobs / queues**, **mailables**, **observers**
- **FormRequest** validation via the `Validated<T>` Axum extractor
- **Middleware** — JWT auth, rate-limit throttle, account-status gate
- **Migrations + seeders**, and an `artisan` CLI binary

```bash
# Run the HTTP server
cargo run -p example --bin server

# Run console commands
cargo run -p example --bin artisan -- migrate
cargo run -p example --bin artisan -- db:seed
cargo run -p example --bin artisan -- permissions:list
```

## Building

```bash
cargo build --workspace
cargo check --workspace
```

## License

MIT
