# lara-scheduler

A cron-style task scheduler for [Lara Rust](https://github.com/venomous-maker/lara-rust).

## Example

```rust
use lara_scheduler::{Scheduler, Schedule};

let scheduler = Scheduler::new()
    .call("nightly-cleanup", Schedule::DailyAt { hour: 2, minute: 0 }, || async {
        // ... do work ...
        Ok(())
    })
    .call("heartbeat", Schedule::EveryMinute, || async {
        tracing::info!("alive");
        Ok(())
    });

tokio::spawn(scheduler.run()); // checks every second, fires due tasks
```

## Schedules

`EverySecond`, `EveryMinute`, `EveryNMinutes(n)`, `EveryHour`, `EveryNHours(n)`,
`Daily`, `DailyAt { hour, minute }`, `Weekly`, `WeeklyOn { .. }`, `Monthly`,
`MonthlyOn { .. }`, and `Cron(expr)`.

## License

MIT
