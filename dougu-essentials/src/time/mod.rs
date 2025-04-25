pub mod error;
pub mod instant;
pub mod duration;
pub mod period;

pub use duration::Duration as TimeDuration;
pub use error::TimeError;
pub use instant::Instant as TimeInstant;
pub use period::Period;

use chrono::{DateTime, Datelike, Duration as ChronoDuration, Local, NaiveDate, NaiveTime, TimeZone, Timelike, Utc};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents a time-based amount of time
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration {
    inner: ChronoDuration,
}

impl Duration {
    /// Creates a duration from a number of seconds
    pub fn of_seconds(seconds: i64) -> Self {
        Self {
            inner: ChronoDuration::seconds(seconds),
        }
    }

    /// Creates a duration from a number of seconds and an adjustment in nanoseconds
    pub fn of_seconds_nanos(seconds: i64, nanos: i64) -> Result<Self, TimeError> {
        if nanos < 0 || nanos >= 1_000_000_000 {
            return Err(TimeError::InvalidDuration(format!("Invalid nanoseconds: {}", nanos)));
        }
        Ok(Self {
            inner: ChronoDuration::seconds(seconds) + ChronoDuration::nanoseconds(nanos),
        })
    }

    /// Creates a duration from a number of milliseconds
    pub fn of_millis(millis: i64) -> Self {
        Self {
            inner: ChronoDuration::milliseconds(millis),
        }
    }

    /// Creates a duration from a number of minutes
    pub fn of_minutes(minutes: i64) -> Self {
        Self {
            inner: ChronoDuration::minutes(minutes),
        }
    }

    /// Creates a duration from a number of hours
    pub fn of_hours(hours: i64) -> Self {
        Self {
            inner: ChronoDuration::hours(hours),
        }
    }

    /// Creates a duration from a number of days
    pub fn of_days(days: i64) -> Self {
        Self {
            inner: ChronoDuration::days(days),
        }
    }

    /// Returns the number of seconds in this duration
    pub fn get_seconds(&self) -> i64 {
        self.inner.num_seconds()
    }

    /// Returns the number of milliseconds in this duration
    pub fn get_millis(&self) -> i64 {
        self.inner.num_milliseconds()
    }

    /// Returns the number of minutes in this duration
    pub fn get_minutes(&self) -> i64 {
        self.inner.num_minutes()
    }

    /// Returns the number of hours in this duration
    pub fn get_hours(&self) -> i64 {
        self.inner.num_hours()
    }

    /// Returns the number of days in this duration
    pub fn get_days(&self) -> i64 {
        self.inner.num_days()
    }

    /// Returns the nanoseconds part of this duration
    pub fn get_nanos(&self) -> i32 {
        (self.inner.num_nanoseconds().unwrap_or(0) % 1_000_000_000) as i32
    }

    /// Returns a copy of this duration with the specified duration added
    pub fn plus(&self, other: Duration) -> Self {
        Self {
            inner: self.inner + other.inner,
        }
    }

    /// Returns a copy of this duration with the specified duration subtracted
    pub fn minus(&self, other: Duration) -> Self {
        Self {
            inner: self.inner - other.inner,
        }
    }

    /// Returns a copy of this duration multiplied by the scalar
    pub fn multiplied_by(&self, scalar: i64) -> Self {
        Self {
            inner: ChronoDuration::seconds(self.inner.num_seconds() * scalar),
        }
    }

    /// Returns a copy of this duration divided by the divisor
    pub fn divided_by(&self, divisor: i64) -> Result<Self, TimeError> {
        if divisor == 0 {
            return Err(TimeError::InvalidDuration("Division by zero".to_string()));
        }
        Ok(Self {
            inner: ChronoDuration::seconds(self.inner.num_seconds() / divisor),
        })
    }

    /// Returns a copy of this duration with the length negated
    pub fn negated(&self) -> Self {
        Self {
            inner: -self.inner,
        }
    }

    /// Returns a copy of this duration with a positive length
    pub fn abs(&self) -> Self {
        Self {
            inner: self.inner.abs(),
        }
    }

    /// Returns true if this duration is zero
    pub fn is_zero(&self) -> bool {
        self.inner == ChronoDuration::zero()
    }

    /// Returns true if this duration is negative
    pub fn is_negative(&self) -> bool {
        self.inner < ChronoDuration::zero()
    }
}

impl std::ops::Add for Duration {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.plus(rhs)
    }
}

impl std::ops::Sub for Duration {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.minus(rhs)
    }
}

impl std::ops::Mul<i64> for Duration {
    type Output = Self;

    fn mul(self, rhs: i64) -> Self::Output {
        self.multiplied_by(rhs)
    }
}

impl std::ops::Div<i64> for Duration {
    type Output = Result<Self, TimeError>;

    fn div(self, rhs: i64) -> Self::Output {
        self.divided_by(rhs)
    }
}

impl std::ops::Neg for Duration {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.negated()
    }
}

/// Represents an instant in time
pub trait Instant {
    /// Returns the number of seconds from the epoch of 1970-01-01T00:00:00Z
    fn get_epoch_second(&self) -> i64;

    /// Returns the number of nanoseconds from the epoch of 1970-01-01T00:00:00Z
    fn get_epoch_nano(&self) -> (i64, u32);

    /// Returns the number of milliseconds from the epoch of 1970-01-01T00:00:00Z
    fn get_epoch_milli(&self) -> i64;

    /// Returns true if this instant is after the specified instant
    fn is_after(&self, other: &impl Instant) -> bool {
        self.get_epoch_second() > other.get_epoch_second() ||
            (self.get_epoch_second() == other.get_epoch_second() &&
                self.get_epoch_nano().1 > other.get_epoch_nano().1)
    }

    /// Returns true if this instant is before the specified instant
    fn is_before(&self, other: &impl Instant) -> bool {
        self.get_epoch_second() < other.get_epoch_second() ||
            (self.get_epoch_second() == other.get_epoch_second() &&
                self.get_epoch_nano().1 < other.get_epoch_nano().1)
    }

    /// Returns true if this instant is equal to the specified instant
    fn is_equal(&self, other: &impl Instant) -> bool {
        self.get_epoch_second() == other.get_epoch_second() &&
            self.get_epoch_nano().1 == other.get_epoch_nano().1
    }
}

/// Represents a point in time with timezone information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZonedDateTime {
    inner: DateTime<Utc>,
}

impl ZonedDateTime {
    /// Creates a new ZonedDateTime from the current time
    pub fn now() -> Self {
        Self {
            inner: Utc::now(),
        }
    }

    /// Creates a new ZonedDateTime from a UTC DateTime
    pub fn of_utc(dt: DateTime<Utc>) -> Self {
        Self { inner: dt }
    }

    /// Creates a new ZonedDateTime from a local DateTime
    pub fn of_local(dt: DateTime<Local>) -> Self {
        Self {
            inner: dt.with_timezone(&Utc),
        }
    }

    /// Creates a new ZonedDateTime from a string in RFC3339 format
    pub fn parse(s: &str) -> Result<Self, TimeError> {
        DateTime::parse_from_rfc3339(s)
            .map(|dt| Self {
                inner: dt.with_timezone(&Utc),
            })
            .map_err(|e| TimeError::InvalidDateTimeFormat(e.to_string()))
    }

    /// Returns the UTC DateTime
    pub fn to_utc(&self) -> DateTime<Utc> {
        self.inner
    }

    /// Returns the local DateTime
    pub fn to_local(&self) -> DateTime<Local> {
        self.inner.with_timezone(&Local)
    }

    /// Formats the time point as an RFC3339 string
    pub fn format(&self) -> String {
        self.inner.to_rfc3339()
    }

    /// Adds a duration to the time point
    pub fn plus(&self, duration: Duration) -> Self {
        Self {
            inner: self.inner + duration.inner,
        }
    }

    /// Subtracts a duration from the time point
    pub fn minus(&self, duration: Duration) -> Self {
        Self {
            inner: self.inner - duration.inner,
        }
    }

    /// Returns the year
    pub fn year(&self) -> i32 {
        self.inner.year()
    }

    /// Returns the month (1-12)
    pub fn month(&self) -> u32 {
        self.inner.month()
    }

    /// Returns the day of the month (1-31)
    pub fn day(&self) -> u32 {
        self.inner.day()
    }

    /// Returns the hour (0-23)
    pub fn hour(&self) -> u32 {
        self.inner.hour()
    }

    /// Returns the minute (0-59)
    pub fn minute(&self) -> u32 {
        self.inner.minute()
    }

    /// Returns the second (0-59)
    pub fn second(&self) -> u32 {
        self.inner.second()
    }

    /// Returns the nanosecond (0-999,999,999)
    pub fn nanosecond(&self) -> u32 {
        self.inner.nanosecond()
    }

    /// Creates a new ZonedDateTime from a Unix timestamp (seconds since epoch)
    pub fn of_unix(seconds: i64) -> Result<Self, TimeError> {
        Utc.timestamp_opt(seconds, 0)
            .single()
            .map(|dt| Self { inner: dt })
            .ok_or_else(|| TimeError::InvalidUnixTimestamp(format!("Invalid Unix timestamp: {}", seconds)))
    }

    /// Creates a new ZonedDateTime from a Unix timestamp with nanoseconds
    pub fn of_unix_nanos(seconds: i64, nanos: u32) -> Result<Self, TimeError> {
        Utc.timestamp_opt(seconds, nanos)
            .single()
            .map(|dt| Self { inner: dt })
            .ok_or_else(|| TimeError::InvalidUnixTimestamp(format!("Invalid Unix timestamp: {}.{}", seconds, nanos)))
    }

    /// Returns the Unix timestamp (seconds since epoch)
    pub fn to_unix(&self) -> i64 {
        self.inner.timestamp()
    }

    /// Returns the Unix timestamp with nanoseconds
    pub fn to_unix_nanos(&self) -> (i64, u32) {
        (self.inner.timestamp(), self.inner.timestamp_subsec_nanos())
    }

    /// Returns the Unix timestamp in milliseconds
    pub fn to_unix_millis(&self) -> i64 {
        self.inner.timestamp_millis()
    }

    /// Returns the milliseconds since Unix epoch (1970-01-01T00:00:00Z)
    pub fn milliseconds_since_epoch(&self) -> u64 {
        self.to_unix_millis() as u64
    }
}

impl Instant for ZonedDateTime {
    fn get_epoch_second(&self) -> i64 {
        self.to_unix()
    }

    fn get_epoch_nano(&self) -> (i64, u32) {
        (self.to_unix(), self.nanosecond())
    }

    fn get_epoch_milli(&self) -> i64 {
        self.to_unix_millis()
    }
}

/// Represents a date without time information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalDate {
    inner: NaiveDate,
}

impl LocalDate {
    /// Creates a new LocalDate from year, month, and day
    pub fn of(year: i32, month: u32, day: u32) -> Result<Self, TimeError> {
        NaiveDate::from_ymd_opt(year, month, day)
            .map(|d| Self { inner: d })
            .ok_or_else(|| TimeError::InvalidDateFormat(format!("Invalid date: {}-{}-{}", year, month, day)))
    }

    /// Creates a new LocalDate from the current date
    pub fn now() -> Self {
        Self {
            inner: Local::now().date_naive(),
        }
    }

    /// Creates a new LocalDate from an ISO 8601 string (YYYY-MM-DD)
    pub fn parse(s: &str) -> Result<Self, TimeError> {
        NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map(|d| Self { inner: d })
            .map_err(|e| TimeError::InvalidDateFormat(e.to_string()))
    }

    /// Returns the year
    pub fn year(&self) -> i32 {
        self.inner.year()
    }

    /// Returns the month (1-12)
    pub fn month(&self) -> u32 {
        self.inner.month()
    }

    /// Returns the day of the month (1-31)
    pub fn day(&self) -> u32 {
        self.inner.day()
    }

    /// Formats the date as an ISO 8601 string (YYYY-MM-DD)
    pub fn format(&self) -> String {
        self.inner.format("%Y-%m-%d").to_string()
    }

    /// Adds a duration to the date
    pub fn plus_days(&self, days: i64) -> Self {
        Self {
            inner: self.inner + ChronoDuration::days(days),
        }
    }

    /// Subtracts a duration from the date
    pub fn minus_days(&self, days: i64) -> Self {
        Self {
            inner: self.inner - ChronoDuration::days(days),
        }
    }

    /// Creates a new LocalDate from a Unix timestamp (seconds since epoch)
    pub fn of_unix(seconds: i64) -> Result<Self, TimeError> {
        Utc.timestamp_opt(seconds, 0)
            .single()
            .map(|dt| Self { inner: dt.naive_utc().date() })
            .ok_or_else(|| TimeError::InvalidUnixTimestamp(format!("Invalid Unix timestamp: {}", seconds)))
    }

    /// Returns the Unix timestamp at midnight of this date
    pub fn to_unix(&self) -> i64 {
        let datetime = self.inner.and_hms_opt(0, 0, 0).unwrap();
        datetime.and_utc().timestamp()
    }
}

impl Instant for LocalDate {
    fn get_epoch_second(&self) -> i64 {
        self.to_unix()
    }

    fn get_epoch_nano(&self) -> (i64, u32) {
        (self.to_unix(), 0)
    }

    fn get_epoch_milli(&self) -> i64 {
        self.to_unix() * 1000
    }
}

/// Represents a time without date information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalTime {
    inner: NaiveTime,
}

impl LocalTime {
    /// Creates a new LocalTime from hour, minute, second, and nanosecond
    pub fn of(hour: u32, min: u32, sec: u32, nano: u32) -> Result<Self, TimeError> {
        NaiveTime::from_hms_nano_opt(hour, min, sec, nano)
            .map(|t| Self { inner: t })
            .ok_or_else(|| TimeError::InvalidTimeFormat(format!("Invalid time: {}:{}:{}.{}", hour, min, sec, nano)))
    }

    /// Creates a new LocalTime from the current time
    pub fn now() -> Self {
        Self {
            inner: Local::now().time(),
        }
    }

    /// Creates a new LocalTime from an ISO 8601 string (HH:MM:SS)
    pub fn parse(s: &str) -> Result<Self, TimeError> {
        NaiveTime::parse_from_str(s, "%H:%M:%S")
            .map(|t| Self { inner: t })
            .map_err(|e| TimeError::InvalidTimeFormat(e.to_string()))
    }

    /// Returns the hour (0-23)
    pub fn hour(&self) -> u32 {
        self.inner.hour()
    }

    /// Returns the minute (0-59)
    pub fn minute(&self) -> u32 {
        self.inner.minute()
    }

    /// Returns the second (0-59)
    pub fn second(&self) -> u32 {
        self.inner.second()
    }

    /// Returns the nanosecond (0-999,999,999)
    pub fn nanosecond(&self) -> u32 {
        self.inner.nanosecond()
    }

    /// Formats the time as an ISO 8601 string (HH:MM:SS)
    pub fn format(&self) -> String {
        self.inner.format("%H:%M:%S").to_string()
    }

    /// Adds a duration to the time
    pub fn plus_hours(&self, hours: i64) -> Self {
        Self {
            inner: self.inner + ChronoDuration::hours(hours),
        }
    }

    /// Subtracts a duration from the time
    pub fn minus_hours(&self, hours: i64) -> Self {
        Self {
            inner: self.inner - ChronoDuration::hours(hours),
        }
    }

    /// Returns the Unix timestamp of this time on the Unix epoch (1970-01-01)
    pub fn to_unix(&self) -> i64 {
        // Fixed date of 1970-01-01 in UTC timezone with the time component
        let date_with_time = NaiveDate::from_ymd_opt(1970, 1, 1)
            .unwrap()
            .and_hms_opt(self.hour(), self.minute(), self.second())
            .unwrap();

        // Convert to seconds since epoch
        date_with_time.and_utc().timestamp()
    }
}

impl Instant for LocalTime {
    fn get_epoch_second(&self) -> i64 {
        self.to_unix()
    }

    fn get_epoch_nano(&self) -> (i64, u32) {
        (self.to_unix(), self.nanosecond())
    }

    fn get_epoch_milli(&self) -> i64 {
        self.to_unix() * 1000 + (self.nanosecond() as i64 / 1_000_000)
    }
}

/// A clock providing access to the current instant, date and time using a time-zone.
pub trait Clock: Send + Sync {
    /// Gets the current instant of the clock
    fn instant(&self) -> ZonedDateTime;

    /// Gets the current millisecond instant of the clock
    fn millis(&self) -> i64;

    /// Gets the time-zone being used to create dates and times
    fn zone(&self) -> String;

    /// Returns a copy of this clock with a different time-zone
    fn with_zone(&self, zone_id: &str) -> Result<Arc<dyn Clock>, TimeError>;

    /// Returns a copy of this clock with the specified duration added
    fn plus(&self, duration: Duration) -> Arc<dyn Clock>;

    /// Returns a copy of this clock with the specified duration subtracted
    fn minus(&self, duration: Duration) -> Arc<dyn Clock>;
}

/// A clock that provides access to the current instant using the system clock and UTC time-zone
#[derive(Clone)]
pub struct SystemClock {
    offset: Duration,
}

impl SystemClock {
    /// Creates a new system clock
    pub fn new() -> Self {
        Self {
            offset: Duration::of_seconds(0),
        }
    }

    /// Creates a new system clock with the specified offset
    pub fn with_offset(offset: Duration) -> Self {
        Self { offset }
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for SystemClock {
    fn instant(&self) -> ZonedDateTime {
        let now = Utc::now();
        if self.offset.is_zero() {
            ZonedDateTime::of_utc(now)
        } else {
            ZonedDateTime::of_utc(now + self.offset.inner)
        }
    }

    fn millis(&self) -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
            + self.offset.get_millis()
    }

    fn zone(&self) -> String {
        "UTC".to_string()
    }

    fn with_zone(&self, zone_id: &str) -> Result<Arc<dyn Clock>, TimeError> {
        if zone_id != "UTC" {
            return Err(TimeError::ClockError(format!("Unsupported time zone: {}", zone_id)));
        }
        Ok(Arc::new(self.clone()))
    }

    fn plus(&self, duration: Duration) -> Arc<dyn Clock> {
        Arc::new(SystemClock::with_offset(self.offset + duration))
    }

    fn minus(&self, duration: Duration) -> Arc<dyn Clock> {
        Arc::new(SystemClock::with_offset(self.offset - duration))
    }
}

/// A clock that always returns the same instant
#[derive(Clone)]
pub struct FixedClock {
    instant: ZonedDateTime,
}

impl FixedClock {
    /// Creates a new fixed clock with the specified instant
    pub fn new(instant: ZonedDateTime) -> Self {
        Self { instant }
    }

    /// Creates a new fixed clock with the current instant
    pub fn now() -> Self {
        Self {
            instant: ZonedDateTime::now(),
        }
    }
}

impl Clock for FixedClock {
    fn instant(&self) -> ZonedDateTime {
        self.instant.clone()
    }

    fn millis(&self) -> i64 {
        self.instant.get_epoch_milli()
    }

    fn zone(&self) -> String {
        "UTC".to_string()
    }

    fn with_zone(&self, zone_id: &str) -> Result<Arc<dyn Clock>, TimeError> {
        if zone_id != "UTC" {
            return Err(TimeError::ClockError(format!("Unsupported time zone: {}", zone_id)));
        }
        Ok(Arc::new(self.clone()))
    }

    fn plus(&self, duration: Duration) -> Arc<dyn Clock> {
        Arc::new(FixedClock::new(self.instant.plus(duration)))
    }

    fn minus(&self, duration: Duration) -> Arc<dyn Clock> {
        Arc::new(FixedClock::new(self.instant.minus(duration)))
    }
}

/// A clock that adds an offset to an underlying clock
#[derive(Clone)]
pub struct OffsetClock {
    base: Arc<dyn Clock>,
    offset: Duration,
}

impl OffsetClock {
    /// Creates a new offset clock
    pub fn new(base: Arc<dyn Clock>, offset: Duration) -> Self {
        Self { base, offset }
    }
}

impl Clock for OffsetClock {
    fn instant(&self) -> ZonedDateTime {
        self.base.instant().plus(self.offset)
    }

    fn millis(&self) -> i64 {
        self.base.millis() + self.offset.get_millis()
    }

    fn zone(&self) -> String {
        self.base.zone()
    }

    fn with_zone(&self, zone_id: &str) -> Result<Arc<dyn Clock>, TimeError> {
        Ok(Arc::new(OffsetClock::new(
            self.base.with_zone(zone_id)?,
            self.offset,
        )))
    }

    fn plus(&self, duration: Duration) -> Arc<dyn Clock> {
        Arc::new(OffsetClock::new(self.base.clone(), self.offset + duration))
    }

    fn minus(&self, duration: Duration) -> Arc<dyn Clock> {
        Arc::new(OffsetClock::new(self.base.clone(), self.offset - duration))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zoned_date_time() {
        let now = ZonedDateTime::now();
        let utc = now.to_utc();
        let local = now.to_local();
        assert_eq!(now.format().len() > 0, true);

        let duration = Duration::of_hours(1);
        let future = now.plus(duration);
        assert!(future.inner > now.inner);

        let past = now.minus(duration);
        assert!(past.inner < now.inner);
    }

    #[test]
    fn test_local_date() {
        let date = LocalDate::of(2024, 4, 25).unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 4);
        assert_eq!(date.day(), 25);
        assert_eq!(date.format(), "2024-04-25");

        let parsed = LocalDate::parse("2024-04-25").unwrap();
        assert_eq!(parsed, date);

        let tomorrow = date.plus_days(1);
        assert_eq!(tomorrow.day(), 26);
    }

    #[test]
    fn test_local_time() {
        let time = LocalTime::of(14, 30, 0, 0).unwrap();
        assert_eq!(time.hour(), 14);
        assert_eq!(time.minute(), 30);
        assert_eq!(time.second(), 0);
        assert_eq!(time.nanosecond(), 0);
        assert_eq!(time.format(), "14:30:00");

        let parsed = LocalTime::parse("14:30:00").unwrap();
        assert_eq!(parsed, time);

        let next_hour = time.plus_hours(1);
        assert_eq!(next_hour.hour(), 15);
    }

    #[test]
    fn test_unix_timestamp() {
        // Test ZonedDateTime Unix timestamp conversion
        let dt = ZonedDateTime::of_unix(1714003200).unwrap(); // 2024-04-25T00:00:00Z
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 4);
        assert_eq!(dt.day(), 25);
        assert_eq!(dt.to_unix(), 1714003200);

        // Test with nanoseconds
        let dt_nano = ZonedDateTime::of_unix_nanos(1714003200, 500_000_000).unwrap();
        assert_eq!(dt_nano.to_unix_nanos(), (1714003200, 500_000_000));

        // Test LocalDate Unix timestamp conversion
        let date = LocalDate::of_unix(1714003200).unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 4);
        assert_eq!(date.day(), 25);
        assert_eq!(date.to_unix(), 1714003200);

        // Test LocalTime Unix timestamp conversion
        let time = LocalTime::of(12, 0, 0, 0).unwrap();
        let epoch_time = NaiveDate::from_ymd_opt(1970, 1, 1)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp();
        assert_eq!(time.to_unix(), epoch_time);
    }

    #[test]
    fn test_instant_trait() {
        let dt1 = ZonedDateTime::of_unix(1714003200).unwrap(); // 2024-04-25T00:00:00Z
        let dt2 = ZonedDateTime::of_unix_nanos(1714003200, 500_000_000).unwrap();
        let date = LocalDate::of_unix(1714003200).unwrap();
        let time = LocalTime::of(12, 0, 0, 0).unwrap();

        // Test epoch seconds
        assert_eq!(dt1.get_epoch_second(), 1714003200);
        assert_eq!(dt2.get_epoch_second(), 1714003200);
        assert_eq!(date.get_epoch_second(), 1714003200);

        // Test epoch nanoseconds
        assert_eq!(dt1.get_epoch_nano(), (1714003200, 0));
        assert_eq!(dt2.get_epoch_nano(), (1714003200, 500_000_000));
        assert_eq!(date.get_epoch_nano(), (1714003200, 0));

        // Test epoch milliseconds
        assert_eq!(dt1.get_epoch_milli(), 1714003200000);
        assert_eq!(dt2.get_epoch_milli(), 1714003200500);
        assert_eq!(date.get_epoch_milli(), 1714003200000);

        // Test comparison methods
        assert!(dt2.is_after(&dt1));
        assert!(dt1.is_before(&dt2));
        assert!(dt1.is_equal(&date));
        assert!(!time.is_equal(&dt1));
    }

    #[test]
    fn test_duration() {
        // Test creation
        let d1 = Duration::of_seconds(60);
        assert_eq!(d1.get_seconds(), 60);
        assert_eq!(d1.get_minutes(), 1);

        let d2 = Duration::of_seconds_nanos(60, 500_000_000).unwrap();
        assert_eq!(d2.get_seconds(), 60);
        assert_eq!(d2.get_nanos(), 500_000_000);

        // Test arithmetic
        let sum = d1.plus(d2);
        assert_eq!(sum.get_seconds(), 120);
        assert_eq!(sum.get_nanos(), 500_000_000);

        let diff = d2.minus(d1);
        assert_eq!(diff.get_seconds(), 0);
        assert_eq!(diff.get_nanos(), 500_000_000);

        // Test multiplication and division
        let multiplied = d1.multiplied_by(2);
        assert_eq!(multiplied.get_seconds(), 120);

        let divided = d1.divided_by(2).unwrap();
        assert_eq!(divided.get_seconds(), 30);

        // Test negation and absolute value
        let negated = d1.negated();
        assert!(negated.is_negative());
        assert_eq!(negated.get_seconds(), -60);

        let abs = negated.abs();
        assert!(!abs.is_negative());
        assert_eq!(abs.get_seconds(), 60);

        // Test zero duration
        let zero = Duration::of_seconds(0);
        assert!(zero.is_zero());
        assert!(!zero.is_negative());
    }

    #[test]
    fn test_system_clock() {
        let clock = SystemClock::new();
        let instant1 = clock.instant();
        let instant2 = clock.instant();
        assert!(instant2.get_epoch_second() >= instant1.get_epoch_second());

        let offset_clock = SystemClock::with_offset(Duration::of_hours(1));
        let offset_instant = offset_clock.instant();
        assert!(offset_instant.get_epoch_second() >= instant1.get_epoch_second() + 3600);
    }

    #[test]
    fn test_fixed_clock() {
        let fixed_time = ZonedDateTime::of_unix(1714003200).unwrap();
        let clock = FixedClock::new(fixed_time.clone());

        assert_eq!(clock.instant(), fixed_time);
    }

    #[test]
    fn test_offset_clock() {
        let base_clock = Arc::new(SystemClock::new());
        let offset = Duration::of_hours(1);
        let clock = OffsetClock::new(base_clock, offset);

        let instant = clock.instant();
        let base_instant = clock.base.instant();
        assert_eq!(instant.get_epoch_second() - base_instant.get_epoch_second(), 3600);

        let minus_clock = clock.minus(Duration::of_minutes(30));
        assert_eq!(minus_clock.instant().get_epoch_second() - base_instant.get_epoch_second(), 1800);
    }
} 