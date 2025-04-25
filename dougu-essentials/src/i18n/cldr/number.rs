// Number system functionality

use super::{NumberFormatter as CldrNumberFormatter, PluralCategory, PluralRules as CldrPluralRules};
use crate::i18n::LocaleId;
use std::fmt;

/// Number system identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NumberSystem {
    Latin,
    Arab,
    Arabext,
    Deva,
    Other(String),
}

impl fmt::Display for NumberSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NumberSystem::Latin => write!(f, "latn"),
            NumberSystem::Arab => write!(f, "arab"),
            NumberSystem::Arabext => write!(f, "arabext"),
            NumberSystem::Deva => write!(f, "deva"),
            NumberSystem::Other(s) => write!(f, "{}", s),
        }
    }
}

/// Number formatter implementation
pub struct DefaultNumberFormatter {
    number_system: NumberSystem,
}

impl DefaultNumberFormatter {
    /// Create a new number formatter with the specified number system
    pub fn new(number_system: NumberSystem) -> Self {
        Self { number_system }
    }

    /// Create a new number formatter using the Latin number system
    pub fn latin() -> Self {
        Self::new(NumberSystem::Latin)
    }

    /// Get the number system
    pub fn number_system(&self) -> &NumberSystem {
        &self.number_system
    }
}

impl CldrNumberFormatter for DefaultNumberFormatter {
    fn format_number(&self, number: f64, locale: &LocaleId) -> String {
        // This would use icu4x in a real implementation
        // For now, use a simple format based on locale
        match locale.region() {
            Some(region) if region.as_str() == "US" => {
                // 1,234.56
                if number.fract() == 0.0 {
                    // Use manual formatting since Rust doesn't support comma separators in format strings
                    let num_str = format!("{:.0}", number);
                    self.add_thousands_separators(&num_str, ',', '.')
                } else {
                    let num_str = format!("{:.2}", number);
                    self.add_thousands_separators(&num_str, ',', '.')
                }
            }
            _ => {
                // 1.234,56 (European format)
                let formatted = if number.fract() == 0.0 {
                    format!("{:.0}", number)
                } else {
                    format!("{:.2}", number)
                };

                // Replace . with , for decimal separator and add . for thousands
                let parts: Vec<&str> = formatted.split('.').collect();
                if parts.len() == 1 {
                    formatted
                } else {
                    let int_part = parts[0];
                    let frac_part = parts[1];

                    // Add thousand separators
                    let mut with_separators = String::new();
                    let digits: Vec<char> = int_part.chars().collect();
                    for (i, &digit) in digits.iter().enumerate() {
                        if i > 0 && (digits.len() - i) % 3 == 0 {
                            with_separators.push('.');
                        }
                        with_separators.push(digit);
                    }

                    format!("{},{}", with_separators, frac_part)
                }
            }
        }
    }

    fn format_percent(&self, value: f64, locale: &LocaleId) -> String {
        // This would use icu4x in a real implementation
        let formatted_value = self.format_number(value * 100.0, locale);
        format!("{}%", formatted_value)
    }

    fn format_currency(&self, value: f64, currency: &crate::i18n::CurrencyCode, locale: &LocaleId) -> String {
        // Simple implementation
        match locale.region() {
            Some(region) if region.as_str() == "US" => {
                format!("{} {}", currency, self.format_number(value, locale))
            }
            _ => {
                format!("{} {}", self.format_number(value, locale), currency)
            }
        }
    }
}

impl DefaultNumberFormatter {
    // Helper method to add thousands separators to a number string
    fn add_thousands_separators(&self, num_str: &str, thousand_sep: char, decimal_sep: char) -> String {
        let parts: Vec<&str> = num_str.split('.').collect();
        let int_part = parts[0];

        // Add thousand separators
        let mut with_separators = String::new();
        let digits: Vec<char> = int_part.chars().collect();
        for (i, &digit) in digits.iter().enumerate() {
            if i > 0 && (digits.len() - i) % 3 == 0 {
                with_separators.push(thousand_sep);
            }
            with_separators.push(digit);
        }

        // Add decimal part if it exists
        if parts.len() > 1 {
            with_separators.push(decimal_sep);
            with_separators.push_str(parts[1]);
        }

        with_separators
    }
}

/// Plural rules implementation
pub struct DefaultPluralRules;

impl DefaultPluralRules {
    /// Create a new plural rules implementation
    pub fn new() -> Self {
        Self
    }
}

impl CldrPluralRules for DefaultPluralRules {
    fn get_category(&self, number: f64, locale: &LocaleId) -> PluralCategory {
        // This would use icu4x in a real implementation
        // For now, use a simple implementation for English
        if locale.language().as_str() == "en" {
            if number == 1.0 {
                PluralCategory::One
            } else {
                PluralCategory::Other
            }
        } else {
            // For other languages, default to Other
            PluralCategory::Other
        }
    }
} 