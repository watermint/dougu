use crate::time::error::TimeError;
use chrono::Duration as ChronoDuration;

/// Represents a duration of time
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration {
    inner: i64, // Store as i64 to avoid type mismatches
    nanos: i64, // Store nanoseconds separately for precision
}

impl Duration {
    /// Creates a new Duration from a chrono::Duration
    pub fn of(duration: ChronoDuration) -> Self {
        Self {
            inner: duration.num_seconds(),
            nanos: duration.num_nanoseconds().unwrap_or(0) % 1_000_000_000,
        }
    }

    /// Creates a new Duration from seconds
    pub fn of_seconds(seconds: i64) -> Self {
        Self { inner: seconds, nanos: 0 }
    }

    /// Creates a new Duration from milliseconds
    pub fn of_millis(millis: i64) -> Self {
        Self {
            inner: millis / 1000,
            nanos: (millis % 1000) * 1_000_000,
        }
    }

    /// Creates a new Duration from nanoseconds
    pub fn of_nanos(nanos: i64) -> Self {
        Self {
            inner: nanos / 1_000_000_000,
            nanos: nanos % 1_000_000_000,
        }
    }

    /// Creates a new Duration from minutes
    pub fn of_minutes(minutes: i64) -> Self {
        Self { inner: minutes * 60, nanos: 0 }
    }

    /// Creates a new Duration from hours
    pub fn of_hours(hours: i64) -> Self {
        Self { inner: hours * 3600, nanos: 0 }
    }

    /// Creates a new Duration from days
    pub fn of_days(days: i64) -> Self {
        Self { inner: days * 86400, nanos: 0 }
    }

    /// Returns the number of seconds in this duration
    pub fn get_seconds(&self) -> i64 {
        self.inner
    }

    /// Returns the number of milliseconds in this duration
    pub fn get_millis(&self) -> i64 {
        self.inner * 1000 + self.nanos / 1_000_000
    }

    /// Returns the number of nanoseconds in this duration
    pub fn get_nanos(&self) -> i64 {
        self.inner * 1_000_000_000 + self.nanos
    }

    /// Returns the number of minutes in this duration
    pub fn get_minutes(&self) -> i64 {
        self.inner / 60
    }

    /// Returns the number of hours in this duration
    pub fn get_hours(&self) -> i64 {
        self.inner / 3600
    }

    /// Returns the number of days in this duration
    pub fn get_days(&self) -> i64 {
        self.inner / 86400
    }

    /// Returns true if this duration is negative
    pub fn is_negative(&self) -> bool {
        self.inner < 0 || self.nanos < 0
    }

    /// Returns true if this duration is zero
    pub fn is_zero(&self) -> bool {
        self.inner == 0 && self.nanos == 0
    }

    /// Returns the absolute value of this duration
    pub fn abs(&self) -> Self {
        Self {
            inner: self.inner.abs(),
            nanos: self.nanos.abs(),
        }
    }

    /// Returns the negated value of this duration
    pub fn negated(&self) -> Self {
        Self {
            inner: -self.inner,
            nanos: -self.nanos,
        }
    }

    /// Adds another duration to this duration
    pub fn plus(&self, other: Duration) -> Self {
        let mut nanos = self.nanos + other.nanos;
        let mut seconds = self.inner + other.inner;

        if nanos >= 1_000_000_000 {
            seconds += 1;
            nanos -= 1_000_000_000;
        } else if nanos <= -1_000_000_000 {
            seconds -= 1;
            nanos += 1_000_000_000;
        }

        Self { inner: seconds, nanos }
    }

    /// Subtracts another duration from this duration
    pub fn minus(&self, other: Duration) -> Self {
        self.plus(other.negated())
    }

    /// Multiplies this duration by a scalar
    pub fn multiplied_by(&self, scalar: i64) -> Self {
        Self {
            inner: self.inner * scalar,
            nanos: self.nanos * scalar,
        }
    }

    /// Divides this duration by a scalar
    pub fn divided_by(&self, scalar: i64) -> Result<Self, TimeError> {
        if scalar == 0 {
            return Err(TimeError::InvalidDuration("Division by zero".to_string()));
        }
        Ok(Self {
            inner: self.inner / scalar,
            nanos: self.nanos / scalar,
        })
    }

    /// Returns the minimum of this duration and another duration
    pub fn min(&self, other: Duration) -> Self {
        if self.get_nanos() < other.get_nanos() {
            *self
        } else {
            other
        }
    }

    /// Returns the maximum of this duration and another duration
    pub fn max(&self, other: Duration) -> Self {
        if self.get_nanos() > other.get_nanos() {
            *self
        } else {
            other
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_creation() {
        let d1 = Duration::of_seconds(60);
        assert_eq!(d1.get_seconds(), 60);
        assert_eq!(d1.get_minutes(), 1);

        let d2 = Duration::of_millis(1500);
        assert_eq!(d2.get_millis(), 1500);
        assert_eq!(d2.get_seconds(), 1);

        let d3 = Duration::of_nanos(1_000_000_000);
        assert_eq!(d3.get_nanos(), 1_000_000_000);
        assert_eq!(d3.get_seconds(), 1);

        let d4 = Duration::of_minutes(2);
        assert_eq!(d4.get_minutes(), 2);
        assert_eq!(d4.get_seconds(), 120);

        let d5 = Duration::of_hours(1);
        assert_eq!(d5.get_hours(), 1);
        assert_eq!(d5.get_minutes(), 60);

        let d6 = Duration::of_days(1);
        assert_eq!(d6.get_days(), 1);
        assert_eq!(d6.get_hours(), 24);
    }

    #[test]
    fn test_duration_operations() {
        let d1 = Duration::of_seconds(60);
        let d2 = Duration::of_seconds(30);

        // Addition
        let sum = d1.plus(d2);
        assert_eq!(sum.get_seconds(), 90);

        // Subtraction
        let diff = d1.minus(d2);
        assert_eq!(diff.get_seconds(), 30);

        // Multiplication
        let product = d1.multiplied_by(2);
        assert_eq!(product.get_seconds(), 120);

        // Division
        let quotient = d1.divided_by(2).unwrap();
        assert_eq!(quotient.get_seconds(), 30);

        // Division by zero
        assert!(d1.divided_by(0).is_err());

        // Min/Max
        assert_eq!(d1.min(d2).get_seconds(), 30);
        assert_eq!(d1.max(d2).get_seconds(), 60);
    }

    #[test]
    fn test_duration_negation() {
        let d1 = Duration::of_seconds(60);
        let d2 = d1.negated();

        assert!(d2.is_negative());
        assert_eq!(d2.get_seconds(), -60);

        let d3 = d2.abs();
        assert!(!d3.is_negative());
        assert_eq!(d3.get_seconds(), 60);
    }

    #[test]
    fn test_duration_zero() {
        let zero = Duration::of_seconds(0);
        assert!(zero.is_zero());
        assert!(!zero.is_negative());
    }
} 