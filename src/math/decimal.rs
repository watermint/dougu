use crate::core::ErrorTrait;
use std::cmp::Ordering;
// Fixed-point decimal number implementation
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};
use std::str::FromStr;

/// Custom error type for fixed decimal operations
#[derive(ErrorTrait, Debug)]
pub enum FixedDecimalError {
    #[error("Failed to parse decimal: {0}")]
    ParseError(String),
    #[error("Arithmetic error: {0}")]
    ArithmeticError(String),
    #[error("Out of range: {0}")]
    OutOfRangeError(String),
}

/// A fixed-point decimal number implementation that provides precise decimal arithmetic.
/// This type stores a decimal value using an integer magnitude and a scale (number of
/// decimal places).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedDecimal {
    // Store as integer value and a scaling factor
    value: i64,
    scale: u8,  // Number of decimal places
    negative: bool,
}

/// The sign of a number: positive, negative, or zero
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    /// Positive number (greater than zero)
    Positive,
    /// Negative number (less than zero)
    Negative,
    /// Zero
    Zero,
}

impl FixedDecimal {
    /// Creates a new FixedDecimal from parts
    ///
    /// # Arguments
    /// * `value` - The absolute magnitude of the number
    /// * `scale` - The number of decimal places
    /// * `negative` - Whether the number is negative
    ///
    /// # Example
    /// ```
    /// use dougu_essentials::math::decimal::FixedDecimal;
    ///
    /// // Create a number representing 1.23
    /// let num = FixedDecimal::new(123, 2, false);
    /// assert_eq!(num.to_string(), "1.23");
    /// ```
    pub fn new(value: i64, scale: u8, negative: bool) -> Self {
        if value == 0 {
            Self { value: 0, scale, negative: false }
        } else {
            Self { value: value.abs() as i64, scale, negative }
        }
    }

    /// Creates a zero value with the specified scale
    ///
    /// # Arguments
    /// * `scale` - The number of decimal places
    pub fn zero(scale: u8) -> Self {
        Self { value: 0, scale, negative: false }
    }

    /// Creates a FixedDecimal from a float with default precision (6 decimal places)
    ///
    /// # Arguments
    /// * `value` - The float value to convert
    ///
    /// # Example
    /// ```
    /// use dougu_essentials::math::decimal::FixedDecimal;
    ///
    /// let num = FixedDecimal::from_f64(123.45);
    /// ```
    pub fn from_f64(value: f64) -> Self {
        Self::from_f64_with_scale(value, 6)
    }

    /// Creates a FixedDecimal from a float with specified precision
    ///
    /// # Arguments
    /// * `value` - The float value to convert
    /// * `scale` - The number of decimal places to use
    pub fn from_f64_with_scale(value: f64, scale: u8) -> Self {
        let negative = value < 0.0;
        let abs_value = value.abs();
        let scaling_factor = 10_f64.powi(scale as i32);
        let scaled_value = (abs_value * scaling_factor).round() as i64;

        if scaled_value == 0 {
            Self { value: 0, scale, negative: false }
        } else {
            Self { value: scaled_value, scale, negative }
        }
    }

    /// Try to create from a float, checking for validity
    ///
    /// # Arguments
    /// * `value` - The float value to convert
    ///
    /// # Returns
    /// * `Result<FixedDecimal, FixedDecimalError>` - Ok with a FixedDecimal or an error
    ///
    /// # Errors
    /// Returns an error if the float is NaN or infinite
    pub fn try_from(value: f64) -> Result<Self, FixedDecimalError> {
        if value.is_nan() || value.is_infinite() {
            return Err(FixedDecimalError::ParseError("Invalid numeric value".to_string()));
        }
        Ok(Self::from_f64(value))
    }

    /// Convert to a string representation
    ///
    /// # Returns
    /// * `String` - The string representation of the number
    pub fn to_string(&self) -> String {
        if self.value == 0 {
            if self.scale == 0 {
                return "0".to_string();
            } else {
                let mut result = "0.".to_string();
                for _ in 0..self.scale {
                    result.push('0');
                }
                return result;
            }
        }

        if self.scale == 0 {
            return format!("{}{}", if self.negative { "-" } else { "" }, self.value);
        }

        let mut result = self.value.to_string();
        let len = result.len();

        if len <= self.scale as usize {
            // Need to pad with leading zeros
            let mut padded = String::with_capacity(self.scale as usize + 2);
            padded.push_str("0.");
            for _ in 0..(self.scale as usize - len) {
                padded.push('0');
            }
            padded.push_str(&result);
            result = padded;
        } else {
            // Insert decimal point
            result.insert(len - self.scale as usize, '.');
        }

        if self.negative {
            result.insert(0, '-');
        }

        result
    }

    /// Get the magnitude (absolute value without sign)
    ///
    /// # Returns
    /// * `i64` - The absolute magnitude
    pub fn magnitude(&self) -> i64 {
        self.value
    }

    /// Get the magnitude with exponent
    /// This method is required for compatibility with ICU4X
    ///
    /// # Returns
    /// * `i64` - The absolute magnitude
    pub fn magnitude_with_exponent(&self) -> i64 {
        self.value
    }

    /// Get the exponent (negative of scale)
    /// This method is required for compatibility with ICU4X
    ///
    /// # Returns
    /// * `i32` - The exponent (negative of scale)
    pub fn exponent(&self) -> i32 {
        -(self.scale as i32)
    }

    /// Get the sign as an enum
    ///
    /// # Returns
    /// * `Sign` - The sign (Positive, Negative, or Zero)
    pub fn sign_enum(&self) -> Sign {
        if self.value == 0 {
            Sign::Zero
        } else if self.negative {
            Sign::Negative
        } else {
            Sign::Positive
        }
    }

    /// Get the sign (-1, 0, or 1)
    /// This method is required for compatibility with ICU4X
    ///
    /// # Returns
    /// * `i8` - The sign: -1 for negative, 0 for zero, 1 for positive
    pub fn sign(&self) -> i8 {
        match self.sign_enum() {
            Sign::Positive => 1,
            Sign::Zero => 0,
            Sign::Negative => -1,
        }
    }

    /// Check if this value is negative
    ///
    /// # Returns
    /// * `bool` - True if negative, false otherwise
    pub fn is_negative(&self) -> bool {
        self.negative && self.value != 0
    }

    /// Check if this value is zero
    ///
    /// # Returns
    /// * `bool` - True if zero, false otherwise
    pub fn is_zero(&self) -> bool {
        self.value == 0
    }

    /// Get the scale (number of decimal places)
    ///
    /// # Returns
    /// * `u8` - The scale (number of decimal places)
    pub fn scale(&self) -> u8 {
        self.scale
    }

    /// Convert to a different scale (decimal places)
    ///
    /// # Arguments
    /// * `new_scale` - The new scale to convert to
    ///
    /// # Returns
    /// * `FixedDecimal` - A new FixedDecimal with the specified scale
    ///
    /// # Example
    /// ```
    /// use dougu_essentials::math::decimal::FixedDecimal;
    ///
    /// let num = FixedDecimal::new(123, 2, false); // 1.23
    /// let scaled = num.with_scale(3); // 1.230
    /// assert_eq!(scaled.to_string(), "1.230");
    /// ```
    pub fn with_scale(&self, new_scale: u8) -> Self {
        if new_scale == self.scale {
            return self.clone();
        }

        if new_scale > self.scale {
            let scale_diff = new_scale - self.scale;
            let scale_factor = 10_i64.pow(scale_diff as u32);
            Self {
                value: self.value * scale_factor,
                scale: new_scale,
                negative: self.negative,
            }
        } else {
            let scale_diff = self.scale - new_scale;
            let scale_factor = 10_i64.pow(scale_diff as u32);
            Self {
                value: self.value / scale_factor,
                scale: new_scale,
                negative: self.negative,
            }
        }
    }

    /// Round to a specific number of decimal places
    ///
    /// # Arguments
    /// * `places` - The number of decimal places to round to
    ///
    /// # Returns
    /// * `FixedDecimal` - A new FixedDecimal rounded to the specified places
    pub fn round(&self, places: u8) -> Self {
        if places >= self.scale {
            return self.clone();
        }

        let scale_diff = self.scale - places;
        let div = 10_i64.pow(scale_diff as u32);
        let rem = self.value % div;
        let mut value = self.value / div;

        // Round up if remainder is >= half of divisor
        if rem >= div / 2 {
            value += 1;
        }

        Self {
            value: value * 10_i64.pow(places as u32),
            scale: places,
            negative: self.negative,
        }
    }

    /// Get the absolute value
    ///
    /// # Returns
    /// * `FixedDecimal` - The absolute value
    pub fn abs(&self) -> Self {
        if !self.negative {
            return self.clone();
        }

        Self {
            value: self.value,
            scale: self.scale,
            negative: false,
        }
    }

    /// Get the integer part as a string
    ///
    /// # Returns
    /// * `String` - The integer part as a string
    pub fn integer_part(&self) -> String {
        if self.value == 0 {
            return "0".to_string();
        }

        let value_str = self.value.to_string();
        if value_str.len() <= self.scale as usize {
            "0".to_string()
        } else {
            let int_part = &value_str[0..(value_str.len() - self.scale as usize)];
            int_part.to_string()
        }
    }

    /// Get the fractional part as a string
    ///
    /// # Returns
    /// * `String` - The fractional part as a string
    pub fn fraction_part(&self) -> String {
        if self.value == 0 || self.scale == 0 {
            return "0".to_string();
        }

        let value_str = self.value.to_string();
        if value_str.len() <= self.scale as usize {
            let mut frac = String::with_capacity(self.scale as usize);
            for _ in 0..(self.scale as usize - value_str.len()) {
                frac.push('0');
            }
            frac.push_str(&value_str);
            frac
        } else {
            let frac_part = &value_str[(value_str.len() - self.scale as usize)..];
            frac_part.to_string()
        }
    }

    /// Convert to a f64 value
    ///
    /// # Returns
    /// * `f64` - The value as a floating point number
    pub fn to_f64(&self) -> f64 {
        let float_value = self.value as f64 / 10_f64.powi(self.scale as i32);
        if self.negative {
            -float_value
        } else {
            float_value
        }
    }

    /// Get the number of digits in the integer part
    ///
    /// # Returns
    /// * `usize` - The number of digits in the integer part
    pub fn integer_digits(&self) -> usize {
        if self.value == 0 {
            return 1;
        }

        let int_part = self.integer_part();
        int_part.len()
    }

    /// Get the number of visible fraction digits (ignoring trailing zeros)
    ///
    /// # Returns
    /// * `usize` - The number of significant fraction digits
    pub fn visible_fraction_digits(&self) -> usize {
        if self.value == 0 || self.scale == 0 {
            return 0;
        }

        let fraction = self.fraction_part();
        // Trim trailing zeros
        let mut visible_count = fraction.len();
        for c in fraction.chars().rev() {
            if c == '0' {
                visible_count -= 1;
            } else {
                break;
            }
        }
        visible_count
    }

    /// Add two FixedDecimal values
    ///
    /// # Arguments
    /// * `other` - The other FixedDecimal to add
    ///
    /// # Returns
    /// * `FixedDecimal` - The sum
    pub fn add(&self, other: &Self) -> Self {
        self.clone() + other.clone()
    }

    /// Subtract one FixedDecimal from another
    ///
    /// # Arguments
    /// * `other` - The FixedDecimal to subtract
    ///
    /// # Returns
    /// * `FixedDecimal` - The difference
    pub fn subtract(&self, other: &Self) -> Self {
        self.clone() - other.clone()
    }

    /// Multiply two FixedDecimal values
    ///
    /// # Arguments
    /// * `other` - The other FixedDecimal to multiply by
    ///
    /// # Returns
    /// * `FixedDecimal` - The product
    pub fn multiply(&self, other: &Self) -> Self {
        self.clone() * other.clone()
    }

    /// Divide one FixedDecimal by another
    ///
    /// # Arguments
    /// * `other` - The FixedDecimal to divide by
    ///
    /// # Returns
    /// * `Result<FixedDecimal, FixedDecimalError>` - The quotient or an error
    ///
    /// # Errors
    /// Returns an error if `other` is zero
    pub fn divide(&self, other: &Self) -> Result<Self, FixedDecimalError> {
        if other.is_zero() {
            return Err(FixedDecimalError::ArithmeticError("Division by zero".to_string()));
        }
        Ok(self.clone() / other.clone())
    }

    /// Parse a string into a FixedDecimal
    ///
    /// # Arguments
    /// * `s` - The string to parse
    ///
    /// # Returns
    /// * `Result<FixedDecimal, FixedDecimalError>` - The parsed value or an error
    pub fn parse(s: &str) -> Result<Self, FixedDecimalError> {
        Self::from_str(s)
    }
}

impl From<f64> for FixedDecimal {
    fn from(value: f64) -> Self {
        Self::from_f64(value)
    }
}

impl From<i32> for FixedDecimal {
    fn from(value: i32) -> Self {
        Self::new(value as i64, 0, value < 0)
    }
}

impl From<i64> for FixedDecimal {
    fn from(value: i64) -> Self {
        Self::new(value, 0, value < 0)
    }
}

impl FromStr for FixedDecimal {
    type Err = FixedDecimalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(FixedDecimalError::ParseError("Empty string".to_string()));
        }

        let negative = s.starts_with('-');
        let s = if negative { &s[1..] } else { s };

        let parts: Vec<&str> = s.split('.').collect();
        match parts.len() {
            1 => {
                // No decimal point
                let value = parts[0].parse::<i64>()
                    .map_err(|_| FixedDecimalError::ParseError(format!("Failed to parse integer part: {}", s)))?;
                Ok(Self::new(value, 0, negative))
            }
            2 => {
                // Has decimal point
                let int_part = if parts[0].is_empty() { "0" } else { parts[0] };
                let frac_part = parts[1];
                let scale = frac_part.len() as u8;

                let int_value = int_part.parse::<i64>()
                    .map_err(|_| FixedDecimalError::ParseError(format!("Failed to parse integer part: {}", int_part)))?;

                if scale == 0 {
                    return Ok(Self::new(int_value, 0, negative));
                }

                let frac_value = frac_part.parse::<i64>()
                    .map_err(|_| FixedDecimalError::ParseError(format!("Failed to parse fraction part: {}", frac_part)))?;

                let scaling_factor = 10_i64.pow(scale as u32);
                let value = int_value * scaling_factor + frac_value;
                Ok(Self::new(value, scale, negative))
            }
            _ => Err(FixedDecimalError::ParseError(format!("Invalid decimal format: {}", s))),
        }
    }
}

impl fmt::Display for FixedDecimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl PartialOrd for FixedDecimal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FixedDecimal {
    fn cmp(&self, other: &Self) -> Ordering {
        // First compare signs
        match (self.sign_enum(), other.sign_enum()) {
            (Sign::Zero, Sign::Zero) => return Ordering::Equal,
            (Sign::Negative, Sign::Zero | Sign::Positive) => return Ordering::Less,
            (Sign::Zero | Sign::Positive, Sign::Negative) => return Ordering::Greater,
            _ => {}
        }

        // Same sign, compare values
        let self_scale = self.scale;
        let other_scale = other.scale;

        if self_scale == other_scale {
            // Direct comparison possible
            match (self.negative, other.negative) {
                (false, false) => self.value.cmp(&other.value),
                (true, true) => other.value.cmp(&self.value),
                _ => unreachable!("Signs should be handled by first match"),
            }
        } else {
            // Need to adjust scales
            let max_scale = self_scale.max(other_scale);
            let self_adjusted = self.with_scale(max_scale);
            let other_adjusted = other.with_scale(max_scale);

            match (self.negative, other.negative) {
                (false, false) => self_adjusted.value.cmp(&other_adjusted.value),
                (true, true) => other_adjusted.value.cmp(&self_adjusted.value),
                _ => unreachable!("Signs should be handled by first match"),
            }
        }
    }
}

impl Add for FixedDecimal {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let max_scale = self.scale.max(other.scale);
        let self_adjusted = self.with_scale(max_scale);
        let other_adjusted = other.with_scale(max_scale);

        if self.sign() == 0 {
            return other;
        }
        if other.sign() == 0 {
            return self;
        }

        if self.negative == other.negative {
            // Same sign, just add
            Self {
                value: self_adjusted.value + other_adjusted.value,
                scale: max_scale,
                negative: self.negative,
            }
        } else {
            // Different signs, subtract
            if self_adjusted.value > other_adjusted.value {
                Self {
                    value: self_adjusted.value - other_adjusted.value,
                    scale: max_scale,
                    negative: self.negative,
                }
            } else if self_adjusted.value < other_adjusted.value {
                Self {
                    value: other_adjusted.value - self_adjusted.value,
                    scale: max_scale,
                    negative: other.negative,
                }
            } else {
                // Equal magnitudes, result is zero
                Self {
                    value: 0,
                    scale: max_scale,
                    negative: false,
                }
            }
        }
    }
}

impl Sub for FixedDecimal {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        // Subtraction is just addition with negated second operand
        self + other.neg()
    }
}

impl Mul for FixedDecimal {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        if self.is_zero() || other.is_zero() {
            return Self::new(0, self.scale + other.scale, false);
        }

        // Result has sign = product of signs
        let negative = self.negative != other.negative;

        // Result scale is sum of scales
        let scale = self.scale + other.scale;

        // Compute the product
        let value = self.value * other.value;

        Self::new(value, scale, negative)
    }
}

impl Div for FixedDecimal {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        if other.is_zero() {
            panic!("Division by zero");
        }

        if self.is_zero() {
            return Self::new(0, self.scale, false);
        }

        // Result has sign = quotient of signs
        let negative = self.negative != other.negative;

        // Scale the dividend to get proper precision
        let scaled_self = self.with_scale(self.scale + 6); // Add 6 extra digits for precision

        // Result scale is the difference
        let scale = scaled_self.scale;

        // Compute the quotient
        let value = scaled_self.value / other.value;

        Self::new(value, scale, negative)
    }
}

impl Neg for FixedDecimal {
    type Output = Self;

    fn neg(self) -> Self {
        if self.is_zero() {
            return self;
        }
        Self {
            value: self.value,
            scale: self.scale,
            negative: !self.negative,
        }
    }
}

// Methods needed for ICU compatibility
impl AsRef<FixedDecimal> for FixedDecimal {
    fn as_ref(&self) -> &FixedDecimal {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        assert_eq!(FixedDecimal::new(123, 0, false).to_string(), "123");
        assert_eq!(FixedDecimal::new(123, 0, true).to_string(), "-123");
        assert_eq!(FixedDecimal::new(123, 2, false).to_string(), "1.23");
        assert_eq!(FixedDecimal::new(123, 2, true).to_string(), "-1.23");
        assert_eq!(FixedDecimal::new(123, 3, false).to_string(), "0.123");
        assert_eq!(FixedDecimal::new(0, 0, false).to_string(), "0");
    }

    #[test]
    fn test_from_f64() {
        assert_eq!(FixedDecimal::from_f64(123.45), FixedDecimal::new(123450000, 6, false));
        assert_eq!(FixedDecimal::from_f64(-123.45), FixedDecimal::new(123450000, 6, true));
        assert_eq!(FixedDecimal::from_f64(0.0), FixedDecimal::new(0, 6, false));
    }

    #[test]
    fn test_from_str() {
        assert_eq!("123".parse::<FixedDecimal>().unwrap(), FixedDecimal::new(123, 0, false));
        assert_eq!("-123".parse::<FixedDecimal>().unwrap(), FixedDecimal::new(123, 0, true));
        assert_eq!("123.45".parse::<FixedDecimal>().unwrap(), FixedDecimal::new(12345, 2, false));
        assert_eq!("-123.45".parse::<FixedDecimal>().unwrap(), FixedDecimal::new(12345, 2, true));
        assert_eq!("0.123".parse::<FixedDecimal>().unwrap(), FixedDecimal::new(123, 3, false));
        assert_eq!("0".parse::<FixedDecimal>().unwrap(), FixedDecimal::new(0, 0, false));
    }

    #[test]
    fn test_comparison() {
        assert!(FixedDecimal::new(123, 0, false) > FixedDecimal::new(122, 0, false));
        assert!(FixedDecimal::new(123, 0, false) < FixedDecimal::new(124, 0, false));
        assert!(FixedDecimal::new(123, 0, false) == FixedDecimal::new(123, 0, false));
        assert!(FixedDecimal::new(123, 0, false) > FixedDecimal::new(123, 0, true));
        assert!(FixedDecimal::new(123, 0, true) < FixedDecimal::new(123, 0, false));
        assert!(FixedDecimal::new(123, 0, false) == FixedDecimal::new(1230, 1, false));
        assert!(FixedDecimal::new(123, 0, false) < FixedDecimal::new(1231, 1, false));
    }

    #[test]
    fn test_arithmetic() {
        // Addition
        assert_eq!(
            FixedDecimal::new(123, 0, false) + FixedDecimal::new(456, 0, false),
            FixedDecimal::new(579, 0, false)
        );
        assert_eq!(
            FixedDecimal::new(123, 0, true) + FixedDecimal::new(456, 0, true),
            FixedDecimal::new(579, 0, true)
        );
        assert_eq!(
            FixedDecimal::new(123, 0, false) + FixedDecimal::new(123, 0, true),
            FixedDecimal::new(0, 0, false)
        );

        // Subtraction
        assert_eq!(
            FixedDecimal::new(456, 0, false) - FixedDecimal::new(123, 0, false),
            FixedDecimal::new(333, 0, false)
        );
        assert_eq!(
            FixedDecimal::new(123, 0, false) - FixedDecimal::new(456, 0, false),
            FixedDecimal::new(333, 0, true)
        );

        // Multiplication
        assert_eq!(
            FixedDecimal::new(123, 0, false) * FixedDecimal::new(2, 0, false),
            FixedDecimal::new(246, 0, false)
        );
        assert_eq!(
            FixedDecimal::new(123, 1, false) * FixedDecimal::new(2, 1, false),
            FixedDecimal::new(246, 2, false)
        );

        // Division
        assert_eq!(
            FixedDecimal::new(246, 0, false) / FixedDecimal::new(2, 0, false),
            FixedDecimal::new(123000000, 6, false)
        );
    }

    #[test]
    fn test_rounding() {
        let num = FixedDecimal::from_f64_with_scale(123.456789, 6);
        assert_eq!(num.round(2).to_string(), "123.46");
        assert_eq!(num.round(4).to_string(), "123.4568");
        assert_eq!(num.round(0).to_string(), "123");
    }
} 