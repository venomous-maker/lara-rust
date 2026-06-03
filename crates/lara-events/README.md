# lara-events

An async event dispatcher for [Lara Rust](https://github.com/venomous-maker/lara-rust).

## Example

```rust
use lara_events::{Event, make_dispatcher};
use std::sync::Arc;

#[derive(Clone)]
struct UserRegistered { user_id: i64 }
impl Event for UserRegistered {}

let dispatcher = make_dispatcher();

// Listen
dispatcher.listen::<UserRegistered, _, _>(|e: Arc<UserRegistered>| async move {
    println!("welcome user {}", e.user_id);
}).await;

// Dispatch
dispatcher.dispatch(UserRegistered { user_id: 1 }).await;
```

Multiple listeners per event are supported; each runs in registration order.
`SharedDispatcher` is a cheaply-cloneable `Arc` you can store in app state and
inject into services.

## License

MIT
