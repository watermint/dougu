// Collation (string sorting) functionality

use super::Collator as CldrCollator;
use crate::i18n::LocaleId;
use std::cmp::Ordering;

/// Collation (string sorting) trait
pub trait Collator {
    /// Compare two strings according to the locale's collation rules
    fn compare(&self, a: &str, b: &str, locale: &LocaleId) -> Ordering;

    /// Get a sort key for efficient repeated comparisons
    fn get_sort_key(&self, s: &str, locale: &LocaleId) -> Vec<u8>;
}

/// Collation strength (level of comparison)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollationStrength {
    /// Primary level: base characters only
    Primary,
    /// Secondary level: base characters and accents
    Secondary,
    /// Tertiary level: base characters, accents, and case/variant
    Tertiary,
    /// Quaternary level: For specific usages (punctuation, etc.)
    Quaternary,
    /// Identical level: Compare code points
    Identical,
}

/// Collation options
pub struct CollationOptions {
    /// The strength of the collation
    strength: CollationStrength,
    /// Whether to compare case
    case_level: bool,
    /// Whether to use numeric collation for sequences of digits
    numeric: bool,
}

impl Default for CollationOptions {
    fn default() -> Self {
        Self {
            strength: CollationStrength::Tertiary,
            case_level: false,
            numeric: false,
        }
    }
}

impl CollationOptions {
    /// Create a new collation options
    pub fn new(strength: CollationStrength, case_level: bool, numeric: bool) -> Self {
        Self { strength, case_level, numeric }
    }

    /// Set the collation strength
    pub fn with_strength(mut self, strength: CollationStrength) -> Self {
        self.strength = strength;
        self
    }

    /// Set whether to compare case
    pub fn with_case_level(mut self, case_level: bool) -> Self {
        self.case_level = case_level;
        self
    }

    /// Set whether to use numeric collation
    pub fn with_numeric(mut self, numeric: bool) -> Self {
        self.numeric = numeric;
        self
    }
}

/// Basic collator implementation
pub struct BasicCollator {
    options: CollationOptions,
}

impl BasicCollator {
    /// Create a new collator with default options
    pub fn new() -> Self {
        Self { options: CollationOptions::default() }
    }

    /// Create a new collator with the given options
    pub fn with_options(options: CollationOptions) -> Self {
        Self { options }
    }
}

impl CldrCollator for BasicCollator {
    fn compare(&self, a: &str, b: &str, locale: &LocaleId) -> Ordering {
        // This would use icu4x in a real implementation

        if self.options.numeric {
            // Simple numeric collation for demonstration
            let a_parts: Vec<&str> = a.split(|c: char| !c.is_ascii_digit()).collect();
            let b_parts: Vec<&str> = b.split(|c: char| !c.is_ascii_digit()).collect();

            for (a_part, b_part) in a_parts.iter().zip(b_parts.iter()) {
                // If both parts are numeric, compare as numbers
                if a_part.chars().all(|c| c.is_ascii_digit()) && b_part.chars().all(|c| c.is_ascii_digit()) {
                    let a_num = a_part.parse::<i64>().unwrap_or(0);
                    let b_num = b_part.parse::<i64>().unwrap_or(0);
                    let cmp = a_num.cmp(&b_num);
                    if cmp != Ordering::Equal {
                        return cmp;
                    }
                } else {
                    // Otherwise compare as strings
                    let cmp = self.simple_string_compare(a_part, b_part, locale);
                    if cmp != Ordering::Equal {
                        return cmp;
                    }
                }
            }

            // If we get here, compare the lengths
            a_parts.len().cmp(&b_parts.len())
        } else {
            // Simple string comparison
            self.simple_string_compare(a, b, locale)
        }
    }

    fn get_sort_key(&self, s: &str, _locale: &LocaleId) -> Vec<u8> {
        // This would use icu4x in a real implementation
        // For now, just return the bytes of the string
        s.as_bytes().to_vec()
    }
}

impl BasicCollator {
    /// Simple string comparison based on collation options
    fn simple_string_compare(&self, a: &str, b: &str, _locale: &LocaleId) -> Ordering {
        match self.options.strength {
            CollationStrength::Primary => {
                // For primary, we just compare the lowercase base characters
                let a_folded = a.to_lowercase();
                let b_folded = b.to_lowercase();
                a_folded.cmp(&b_folded)
            }
            CollationStrength::Secondary | CollationStrength::Tertiary => {
                // For tertiary, we compare with case sensitivity
                if self.options.strength == CollationStrength::Tertiary {
                    a.cmp(b)
                } else {
                    // For secondary, we ignore case
                    a.to_lowercase().cmp(&b.to_lowercase())
                }
            }
            CollationStrength::Quaternary | CollationStrength::Identical => {
                // For quaternary and identical, we compare byte by byte
                a.as_bytes().cmp(b.as_bytes())
            }
        }
    }
} 