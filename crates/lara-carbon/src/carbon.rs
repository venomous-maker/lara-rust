use chrono::{
    DateTime, Datelike, Duration, NaiveDate, NaiveDateTime, TimeZone,
    Timelike, Utc, Local, FixedOffset,
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Fluent date/time utility — Rust equivalent of Laravel Carbon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Carbon(DateTime<Utc>);

impl Carbon {
    // ── Constructors ─────────────────────────────────────────────────────────

    pub fn now() -> Self { Self(Utc::now()) }

    pub fn today() -> Self {
        let now = Utc::now();
        Self(Utc.with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0).unwrap())
    }

    pub fn yesterday() -> Self { Self::today().sub_days(1) }
    pub fn tomorrow()  -> Self { Self::today().add_days(1) }

    pub fn from_timestamp(ts: i64) -> Self {
        Self(DateTime::from_timestamp(ts, 0).unwrap_or_else(Utc::now))
    }

    pub fn parse(s: &str) -> Option<Self> {
        DateTime::parse_from_rfc3339(s)
            .map(|dt| Self(dt.with_timezone(&Utc)))
            .ok()
            .or_else(|| {
                NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| Self(DateTime::from_naive_utc_and_offset(ndt, Utc)))
                    .ok()
            })
            .or_else(|| {
                NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .map(|nd| Self(DateTime::from_naive_utc_and_offset(nd.and_hms_opt(0, 0, 0).unwrap(), Utc)))
                    .ok()
            })
    }

    pub fn create(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> Option<Self> {
        Utc.with_ymd_and_hms(year, month, day, hour, minute, second)
            .single()
            .map(Self)
    }

    // ── Inner access ─────────────────────────────────────────────────────────

    pub fn inner(&self) -> &DateTime<Utc> { &self.0 }
    pub fn timestamp(&self) -> i64 { self.0.timestamp() }

    // ── Date parts ───────────────────────────────────────────────────────────

    pub fn year(&self)   -> i32  { self.0.year() }
    pub fn month(&self)  -> u32  { self.0.month() }
    pub fn day(&self)    -> u32  { self.0.day() }
    pub fn hour(&self)   -> u32  { self.0.hour() }
    pub fn minute(&self) -> u32  { self.0.minute() }
    pub fn second(&self) -> u32  { self.0.second() }
    pub fn weekday(&self) -> chrono::Weekday { self.0.weekday() }

    // ── Arithmetic ───────────────────────────────────────────────────────────

    pub fn add_seconds(self, n: i64) -> Self { Self(self.0 + Duration::seconds(n)) }
    pub fn sub_seconds(self, n: i64) -> Self { Self(self.0 - Duration::seconds(n)) }
    pub fn add_minutes(self, n: i64) -> Self { Self(self.0 + Duration::minutes(n)) }
    pub fn sub_minutes(self, n: i64) -> Self { Self(self.0 - Duration::minutes(n)) }
    pub fn add_hours(self, n: i64)   -> Self { Self(self.0 + Duration::hours(n)) }
    pub fn sub_hours(self, n: i64)   -> Self { Self(self.0 - Duration::hours(n)) }
    pub fn add_days(self, n: i64)    -> Self { Self(self.0 + Duration::days(n)) }
    pub fn sub_days(self, n: i64)    -> Self { Self(self.0 - Duration::days(n)) }
    pub fn add_weeks(self, n: i64)   -> Self { self.add_days(n * 7) }
    pub fn sub_weeks(self, n: i64)   -> Self { self.sub_days(n * 7) }
    pub fn add_months(self, n: u32)  -> Self {
        let m = self.0.month() + n;
        let y = self.0.year() + (m as i32 - 1) / 12;
        let m = ((m - 1) % 12) + 1;
        let d = self.0.day().min(days_in_month(y, m));
        Self(Utc.with_ymd_and_hms(y, m, d, self.hour(), self.minute(), self.second())
            .single().unwrap_or(self.0))
    }
    pub fn sub_months(self, n: u32) -> Self {
        let total = (self.0.month() as i32) - (n as i32);
        let (y, m) = if total <= 0 {
            (self.0.year() - 1 + total / 12, ((total - 1).rem_euclid(12) + 1) as u32)
        } else {
            (self.0.year(), total as u32)
        };
        let d = self.0.day().min(days_in_month(y, m));
        Self(Utc.with_ymd_and_hms(y, m, d, self.hour(), self.minute(), self.second())
            .single().unwrap_or(self.0))
    }
    pub fn add_years(self, n: i32) -> Self {
        Self(Utc.with_ymd_and_hms(
            self.0.year() + n, self.0.month(), self.0.day(),
            self.hour(), self.minute(), self.second(),
        ).single().unwrap_or(self.0))
    }
    pub fn sub_years(self, n: i32) -> Self { self.add_years(-n) }

    // ── Comparisons ──────────────────────────────────────────────────────────

    pub fn is_past(&self)    -> bool { self.0 < Utc::now() }
    pub fn is_future(&self)  -> bool { self.0 > Utc::now() }
    pub fn is_today(&self)   -> bool { self.0.date_naive() == Utc::now().date_naive() }
    pub fn is_weekend(&self) -> bool {
        use chrono::Weekday::*;
        matches!(self.weekday(), Sat | Sun)
    }
    pub fn is_weekday(&self) -> bool { !self.is_weekend() }

    pub fn diff_in_seconds(&self, other: &Carbon) -> i64 {
        (self.0 - other.0).num_seconds()
    }
    pub fn diff_in_minutes(&self, other: &Carbon) -> i64 {
        (self.0 - other.0).num_minutes()
    }
    pub fn diff_in_hours(&self, other: &Carbon) -> i64 {
        (self.0 - other.0).num_hours()
    }
    pub fn diff_in_days(&self, other: &Carbon) -> i64 {
        (self.0 - other.0).num_days()
    }
    pub fn diff_for_humans(&self) -> String {
        let diff = (Utc::now() - self.0).num_seconds().abs();
        match diff {
            0..=44     => "just now".to_string(),
            45..=89    => "a minute ago".to_string(),
            90..=2699  => format!("{} minutes ago", diff / 60),
            2700..=5399 => "an hour ago".to_string(),
            5400..=86399 => format!("{} hours ago", diff / 3600),
            _ => format!("{} days ago", diff / 86400),
        }
    }

    // ── Formatting ───────────────────────────────────────────────────────────

    pub fn format(&self, fmt: &str) -> String {
        self.0.format(fmt).to_string()
    }

    pub fn to_date_string(&self)      -> String { self.format("%Y-%m-%d") }
    pub fn to_time_string(&self)      -> String { self.format("%H:%M:%S") }
    pub fn to_datetime_string(&self)  -> String { self.format("%Y-%m-%d %H:%M:%S") }
    pub fn to_rfc3339(&self)          -> String { self.0.to_rfc3339() }
    pub fn to_rfc2822(&self)          -> String { self.0.to_rfc2822() }
}

impl fmt::Display for Carbon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_datetime_string())
    }
}

impl From<DateTime<Utc>> for Carbon {
    fn from(dt: DateTime<Utc>) -> Self { Self(dt) }
}

impl From<Carbon> for DateTime<Utc> {
    fn from(c: Carbon) -> Self { c.0 }
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let start = NaiveDate::from_ymd_opt(year, month, 1).unwrap_or_default();
    let next_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    };
    next_month
        .map(|d| (d - start).num_days() as u32)
        .unwrap_or(28)
}
