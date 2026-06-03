# lara-db

The Eloquent-style ORM for [Lara Rust](https://github.com/venomous-maker/lara-rust).
One model API across **PostgreSQL, MySQL, SQLite, and MongoDB** — the query builder
compiles to SQL or BSON automatically based on the active connection.

## Features

- **Global connection** — `Db::configure(cfg).await?` once at startup; no `db`
  parameter anywhere in the public API afterward.
- **Typed models** via `#[derive(Model)]` — `Model::create()` takes the struct itself.
- **Fluent query builder** — `where_*`, `join`, `order_by`, `group_by`, `having`,
  `limit/offset`, `paginate`, aggregates (`count/sum/avg/min/max`), `exists`, `chunk`.
- **Relationships** — `has_one`, `has_many`, `belongs_to`, `belongs_to_many`,
  `has_one_through`, `has_many_through`, `morph_one`, `morph_many` — each with an
  optional local/owner key (`None` = primary key).
- **Soft deletes**, **timestamps**, **migrations**, and a **schema builder**.

## Example

```rust
use lara_db::{Db, ModelTrait};

Db::configure(config).await?;

// Query → typed models
let users = User::query()
    .where_eq("status", "active")
    .order_by_desc("created_at")
    .paginate(15, 1)
    .await?;

// Relationships (optional local key)
let roles = user.belongs_to_many::<Role>("role_user", "user_id", "role_id", None)
    .get().await?;

// Create / update / soft-delete
let user = User::create(User { name: "Ada".into(), ..Default::default() }).await?;
user.delete().await?; // soft delete when #[lara(soft_deletes)]
```

## Feature flags

`postgres`, `mysql`, `sqlite`, `mongodb` (all enabled by default).

## License

MIT
