# lara-derive

Procedural macros for [Lara Rust](https://github.com/venomous-maker/lara-rust).

## Derives

### `#[derive(Model)]`
Generates the `ModelMeta` impl for an ORM model. Configure with `#[lara(...)]`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[lara(table = "users", primary_key = "id", soft_deletes)]
pub struct User {
    pub id: Option<i64>,
    pub name: String,
    #[lara(hidden)]    // excluded from to_json_public()
    pub password: String,
    #[lara(fillable)]  // allowed for mass-assignment
    pub bio: Option<String>,
}
```

Struct attributes: `table`, `primary_key`, `timestamps` / `no_timestamps`, `soft_deletes`.
Field attributes: `hidden`, `fillable`.

### `#[derive(Job)]`
Generates `JobMeta` for a queueable job:

```rust
#[derive(Serialize, Deserialize, Job)]
#[lara(queue = "emails", tries = 3, timeout = 30)]
pub struct SendWelcomeEmail { pub user_id: i64 }
```

### `#[derive(Command)]`
Generates `CommandMeta` for an Artisan command (`name`, `description`).

## License

MIT
