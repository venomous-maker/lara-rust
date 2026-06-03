# lara-router

HTTP routing for [Lara Rust](https://github.com/venomous-maker/lara-rust), built on
[Axum](https://github.com/tokio-rs/axum) with a Laravel-flavored API.

## Features

- **`LaraRouter`** — fluent route builder with `get/post/put/patch/delete`, route
  groups, prefixes, and a CORS helper.
- **`FormRequest` + `Validated<T>`** — a validating Axum extractor. Declare rules on
  a request struct; invalid bodies return `422` with field errors automatically.
- **`LaraRequest`** — an enriched request wrapper (input/query/params, bearer token, IP).
- Built-in middleware helpers (request logging, JSON enforcement).

## FormRequest example

```rust
use lara_router::{FormRequest, Validated};
use lara_validator::Rule;

#[derive(Deserialize)]
struct StoreUser { name: String, email: String, password: String }

impl FormRequest for StoreUser {
    fn rules() -> Vec<(&'static str, Vec<Rule>)> {
        vec![
            ("name",     vec![Rule::Required, Rule::MinLength(2)]),
            ("email",    vec![Rule::Required, Rule::Email]),
            ("password", vec![Rule::Required, Rule::MinLength(8)]),
        ]
    }
}

async fn store(Validated(body): Validated<StoreUser>) { /* body is validated */ }
```

## License

MIT
