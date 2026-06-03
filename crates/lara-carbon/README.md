# lara-carbon

A fluent date/time API for [Lara Rust](https://github.com/venomous-maker/lara-rust) —
the equivalent of Laravel's Carbon, built on [`chrono`](https://github.com/chronotope/chrono).

## Example

```rust
use lara_carbon::Carbon;

let now = Carbon::now();
let next_week = now.clone().add_days(7);

now.is_weekend();
now.diff_in_hours(&next_week);
now.format("%Y-%m-%d %H:%M:%S");
now.diff_for_humans();           // "just now", "3 hours ago", ...

Carbon::parse("2026-06-03T12:00:00Z");
Carbon::today().add_months(1).sub_years(1);
```

Chainable arithmetic (`add/sub` seconds → years), comparisons
(`is_past`, `is_future`, `is_today`, `is_weekday`), diffing, and formatting.

## License

MIT
