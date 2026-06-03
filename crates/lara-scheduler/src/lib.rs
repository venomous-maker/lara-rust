use async_trait::async_trait;
use chrono::{Datelike, Timelike, Utc, Weekday};
use std::{pin::Pin, sync::Arc, time::Duration};
use tokio::time;

pub type TaskFuture = Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send>>;
pub type TaskFn = Arc<dyn Fn() -> TaskFuture + Send + Sync>;

/// A single scheduled event.
pub struct ScheduledEvent {
    name: String,
    task: TaskFn,
    schedule: Schedule,
}

/// Cron-like schedule definition.
#[derive(Debug, Clone)]
pub enum Schedule {
    EverySecond,
    EveryMinute,
    EveryNMinutes(u64),
    EveryHour,
    EveryNHours(u64),
    Daily,
    DailyAt { hour: u32, minute: u32 },
    Weekly,
    WeeklyOn { weekday: Weekday, hour: u32, minute: u32 },
    Monthly,
    MonthlyOn { day: u32, hour: u32, minute: u32 },
    Cron(String),
}

impl Schedule {
    pub fn interval_secs(&self) -> u64 {
        match self {
            Schedule::EverySecond      => 1,
            Schedule::EveryMinute      => 60,
            Schedule::EveryNMinutes(n) => n * 60,
            Schedule::EveryHour        => 3600,
            Schedule::EveryNHours(n)   => n * 3600,
            Schedule::Daily            => 86400,
            Schedule::DailyAt { .. }   => 60,   // checked every minute
            Schedule::Weekly           => 604800,
            Schedule::WeeklyOn { .. }  => 60,
            Schedule::Monthly          => 60,
            Schedule::MonthlyOn { .. } => 60,
            Schedule::Cron(_)          => 60,
        }
    }

    pub fn should_run_now(&self) -> bool {
        let now = Utc::now();
        match self {
            Schedule::EverySecond    => true,
            Schedule::EveryMinute    => now.second() == 0,
            Schedule::EveryNMinutes(n) => (now.minute() as u64 % n) == 0 && now.second() == 0,
            Schedule::EveryHour      => now.minute() == 0 && now.second() == 0,
            Schedule::EveryNHours(n) => (now.hour() as u64 % n) == 0 && now.minute() == 0,
            Schedule::Daily          => now.hour() == 0 && now.minute() == 0,
            Schedule::DailyAt { hour, minute } => {
                now.hour() == *hour && now.minute() == *minute && now.second() < 5
            }
            Schedule::WeeklyOn { weekday, hour, minute } => {
                now.weekday() == *weekday && now.hour() == *hour && now.minute() == *minute
            }
            Schedule::MonthlyOn { day, hour, minute } => {
                now.day() == *day && now.hour() == *hour && now.minute() == *minute
            }
            _ => false,
        }
    }
}

/// The task scheduler.
pub struct Scheduler {
    events: Vec<ScheduledEvent>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Register a task.
    pub fn call<F, Fut>(mut self, name: &str, schedule: Schedule, task: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        let task = Arc::new(move || -> TaskFuture { Box::pin(task()) });
        self.events.push(ScheduledEvent {
            name: name.to_string(),
            task,
            schedule,
        });
        self
    }

    /// Run the scheduler loop.  Checks every second and fires due tasks.
    pub async fn run(self) {
        let mut interval = time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            for event in &self.events {
                if event.schedule.should_run_now() {
                    let task = event.task.clone();
                    let name = event.name.clone();
                    tokio::spawn(async move {
                        tracing::info!("Running scheduled task: {}", name);
                        match task().await {
                            Ok(_) => tracing::info!("Task `{}` completed", name),
                            Err(e) => tracing::error!("Task `{}` failed: {}", name, e),
                        }
                    });
                }
            }
        }
    }
}

impl Default for Scheduler {
    fn default() -> Self { Self::new() }
}
