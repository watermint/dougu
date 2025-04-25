use std::fmt;
// Currency related types and implementations
use std::str::FromStr;

/// Currency code as defined by ISO 4217
///
/// Three-letter codes (e.g., "USD" for US Dollar)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CurrencyCode(String);

impl CurrencyCode {
    /// Create a new currency code following ISO 4217
    pub fn new(code: &str) -> Self {
        // ISO 4217 specifies uppercase
        Self(code.to_uppercase())
    }

    /// Get the currency code
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for CurrencyCode {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // ISO 4217 (three-letter)
        if s.len() != 3 || !s.chars().all(|c| c.is_ascii_alphabetic()) {
            return Err("Invalid currency code: must be 3 ASCII letters (ISO 4217)");
        }
        Ok(Self::new(s))
    }
}

impl fmt::Display for CurrencyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
} 