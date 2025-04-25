use chrono::{NaiveDate, Datelike};
use crate::time::error::TimeError;

/// Represents a period of time between two dates
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Period {
    years: i32,
    months: i32,
    days: i32,
}

impl Period {
    /// Creates a new Period with the given years, months, and days
    pub fn of(years: i32, months: i32, days: i32) -> Self {
        Self {
            years,
            months,
            days,
        }
    }

    /// Creates a new Period of years
    pub fn of_years(years: i32) -> Self {
        Self {
            years,
            months: 0,
            days: 0,
        }
    }

    /// Creates a new Period of months
    pub fn of_months(months: i32) -> Self {
        Self {
            years: 0,
            months,
            days: 0,
        }
    }

    /// Creates a new Period of days
    pub fn of_days(days: i32) -> Self {
        Self {
            years: 0,
            months: 0,
            days,
        }
    }

    /// Creates a new Period between two dates
    pub fn between(start: NaiveDate, end: NaiveDate) -> Self {
        let mut years = end.year() - start.year();
        let mut months = end.month() as i32 - start.month() as i32;
        let mut days = end.day() as i32 - start.day() as i32;

        // Adjust for negative months
        if months < 0 {
            years -= 1;
            months += 12;
        }

        // Adjust for negative days
        if days < 0 {
            months -= 1;
            // Get the last day of the previous month
            let last_day = if start.month() == 1 {
                NaiveDate::from_ymd_opt(start.year() - 1, 12, 1)
                    .unwrap()
                    .with_day(31)
                    .unwrap()
            } else {
                NaiveDate::from_ymd_opt(start.year(), start.month() - 1, 1)
                    .unwrap()
                    .with_day(31)
                    .unwrap()
            };
            days += last_day.day() as i32;
        }

        Self {
            years,
            months,
            days,
        }
    }

    /// Returns the number of years in this period
    pub fn get_years(&self) -> i32 {
        self.years
    }

    /// Returns the number of months in this period
    pub fn get_months(&self) -> i32 {
        self.months
    }

    /// Returns the number of days in this period
    pub fn get_days(&self) -> i32 {
        self.days
    }

    /// Returns true if this period is negative
    pub fn is_negative(&self) -> bool {
        self.years < 0 || self.months < 0 || self.days < 0
    }

    /// Returns true if this period is zero
    pub fn is_zero(&self) -> bool {
        self.years == 0 && self.months == 0 && self.days == 0
    }

    /// Returns the absolute value of this period
    pub fn abs(&self) -> Self {
        Self {
            years: self.years.abs(),
            months: self.months.abs(),
            days: self.days.abs(),
        }
    }

    /// Returns the negated value of this period
    pub fn negated(&self) -> Self {
        Self {
            years: -self.years,
            months: -self.months,
            days: -self.days,
        }
    }

    /// Adds another period to this period
    pub fn plus(&self, other: Period) -> Self {
        Self {
            years: self.years + other.years,
            months: self.months + other.months,
            days: self.days + other.days,
        }
    }

    /// Subtracts another period from this period
    pub fn minus(&self, other: Period) -> Self {
        Self {
            years: self.years - other.years,
            months: self.months - other.months,
            days: self.days - other.days,
        }
    }

    /// Multiplies this period by a scalar
    pub fn multiplied_by(&self, scalar: i32) -> Self {
        Self {
            years: self.years * scalar,
            months: self.months * scalar,
            days: self.days * scalar,
        }
    }

    /// Normalizes this period by converting excess months into years
    pub fn normalized(&self) -> Self {
        let mut years = self.years;
        let mut months = self.months;
        let days = self.days;

        // Convert excess months into years
        years += months / 12;
        months %= 12;

        Self {
            years,
            months,
            days,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_period_creation() {
        let p1 = Period::of(1, 2, 3);
        assert_eq!(p1.get_years(), 1);
        assert_eq!(p1.get_months(), 2);
        assert_eq!(p1.get_days(), 3);

        let p2 = Period::of_years(5);
        assert_eq!(p2.get_years(), 5);
        assert_eq!(p2.get_months(), 0);
        assert_eq!(p2.get_days(), 0);

        let p3 = Period::of_months(6);
        assert_eq!(p3.get_years(), 0);
        assert_eq!(p3.get_months(), 6);
        assert_eq!(p3.get_days(), 0);

        let p4 = Period::of_days(10);
        assert_eq!(p4.get_years(), 0);
        assert_eq!(p4.get_months(), 0);
        assert_eq!(p4.get_days(), 10);
    }

    #[test]
    fn test_period_between() {
        let start = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2021, 3, 15).unwrap();
        let period = Period::between(start, end);

        assert_eq!(period.get_years(), 1);
        assert_eq!(period.get_months(), 2);
        assert_eq!(period.get_days(), 14);
    }

    #[test]
    fn test_period_operations() {
        let p1 = Period::of(1, 2, 3);
        let p2 = Period::of(4, 5, 6);

        // Addition
        let sum = p1.plus(p2);
        assert_eq!(sum.get_years(), 5);
        assert_eq!(sum.get_months(), 7);
        assert_eq!(sum.get_days(), 9);

        // Subtraction
        let diff = p2.minus(p1);
        assert_eq!(diff.get_years(), 3);
        assert_eq!(diff.get_months(), 3);
        assert_eq!(diff.get_days(), 3);

        // Multiplication
        let product = p1.multiplied_by(2);
        assert_eq!(product.get_years(), 2);
        assert_eq!(product.get_months(), 4);
        assert_eq!(product.get_days(), 6);
    }

    #[test]
    fn test_period_negation() {
        let p1 = Period::of(1, 2, 3);
        let p2 = p1.negated();

        assert!(p2.is_negative());
        assert_eq!(p2.get_years(), -1);
        assert_eq!(p2.get_months(), -2);
        assert_eq!(p2.get_days(), -3);

        let p3 = p2.abs();
        assert!(!p3.is_negative());
        assert_eq!(p3.get_years(), 1);
        assert_eq!(p3.get_months(), 2);
        assert_eq!(p3.get_days(), 3);
    }

    #[test]
    fn test_period_normalization() {
        let p1 = Period::of(1, 14, 3);
        let normalized = p1.normalized();

        assert_eq!(normalized.get_years(), 2);
        assert_eq!(normalized.get_months(), 2);
        assert_eq!(normalized.get_days(), 3);
    }

    #[test]
    fn test_period_zero() {
        let zero = Period::of(0, 0, 0);
        assert!(zero.is_zero());
        assert!(!zero.is_negative());
    }
} 