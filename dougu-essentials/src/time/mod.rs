use chrono::{DateTime, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use thiserror::Error;

/// Errors that can occur when working with time operations
#[derive(Error, Debug)]
pub enum TimeError {
    #[error("Invalid date format: {0}")]
    InvalidDateFormat(String),
    #[error("Invalid time format: {0}")]
    InvalidTimeFormat(String),
    #[error("Invalid datetime format: {0}")]
    InvalidDateTimeFormat(String),
    #[error("Date/time operation failed: {0}")]
    OperationFailed(String),
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
            inner: self.inner + duration,
        }
    }

    /// Subtracts a duration from the time point
    pub fn minus(&self, duration: Duration) -> Self {
        Self {
            inner: self.inner - duration,
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
            inner: self.inner + Duration::days(days),
        }
    }

    /// Subtracts a duration from the date
    pub fn minus_days(&self, days: i64) -> Self {
        Self {
            inner: self.inner - Duration::days(days),
        }
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
            inner: self.inner + Duration::hours(hours),
        }
    }

    /// Subtracts a duration from the time
    pub fn minus_hours(&self, hours: i64) -> Self {
        Self {
            inner: self.inner - Duration::hours(hours),
        }
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

        let duration = Duration::hours(1);
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
} 