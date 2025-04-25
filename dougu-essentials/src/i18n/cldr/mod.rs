use crate::i18n::{CurrencyCode, LanguageId, LocaleId, RegionId, ScriptId};
// CLDR (Common Locale Data Repository) interface
use crate::time::{LocalDate, LocalTime, ZonedDateTime};

// Functional category modules
mod locale;
mod calendar;
mod number;
mod currency;
mod message;
mod collation;

pub use calendar::*;
pub use collation::*;
pub use currency::*;
// Re-export key types and functions
pub use locale::*;
pub use message::*;
pub use number::*;

/// Calendar type identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CalendarType {
    Gregorian,
    Japanese,
    Buddhist,
    Chinese,
    Hebrew,
    Islamic,
    Persian,
    Indian,
    Coptic,
    Ethiopic,
    Other(String),
}

/// Number system identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NumberSystem {
    Latin,
    Arab,
    Arabext,
    Deva,
    Other(String),
}

/// Formatted message for internationalization
pub trait Message {
    /// Get the formatted message for the given locale
    fn format(&self, locale: &LocaleId) -> String;

    /// Get the formatted message with arguments
    fn format_with_args(&self, locale: &LocaleId, args: &MessageArgs) -> String;
}

/// Arguments for message formatting
pub struct MessageArgs {
    args: std::collections::HashMap<String, MessageValue>,
}

/// Value types for message arguments
#[derive(Debug, Clone)]
pub enum MessageValue {
    String(String),
    Number(f64),
    Integer(i64),
    Date(LocalDate),
    Time(LocalTime),
    DateTime(ZonedDateTime),
    Currency(f64, CurrencyCode),
}

/// Date/Time formatter trait
pub trait DateTimeFormatter {
    /// Format a date according to the locale
    fn format_date(&self, date: &LocalDate, locale: &LocaleId) -> String;

    /// Format a time according to the locale
    fn format_time(&self, time: &LocalTime, locale: &LocaleId) -> String;

    /// Format a date and time according to the locale
    fn format_datetime(&self, datetime: &ZonedDateTime, locale: &LocaleId) -> String;
}

/// Number formatter trait
pub trait NumberFormatter {
    /// Format a number according to the locale
    fn format_number(&self, number: f64, locale: &LocaleId) -> String;

    /// Format a currency value according to the locale and currency code
    fn format_currency(&self, value: f64, currency: &CurrencyCode, locale: &LocaleId) -> String;

    /// Format a percentage according to the locale
    fn format_percent(&self, value: f64, locale: &LocaleId) -> String;
}

/// Pluralization rules according to CLDR
pub enum PluralCategory {
    Zero,
    One,
    Two,
    Few,
    Many,
    Other,
}

/// Plural rules trait for determining plural forms
pub trait PluralRules {
    /// Get the plural category for a number in the given locale
    fn get_category(&self, number: f64, locale: &LocaleId) -> PluralCategory;
}

/// Collation (string sorting) trait
pub trait Collator {
    /// Compare two strings according to the locale's collation rules
    fn compare(&self, a: &str, b: &str, locale: &LocaleId) -> std::cmp::Ordering;

    /// Get a sort key for efficient repeated comparisons
    fn get_sort_key(&self, s: &str, locale: &LocaleId) -> Vec<u8>;
}

/// Provider trait for locale data
pub trait LocaleDataProvider {
    /// Get a date/time formatter for the locale
    fn get_datetime_formatter(&self, locale: &LocaleId) -> Box<dyn DateTimeFormatter>;

    /// Get a number formatter for the locale
    fn get_number_formatter(&self, locale: &LocaleId) -> Box<dyn NumberFormatter>;

    /// Get plural rules for the locale
    fn get_plural_rules(&self, locale: &LocaleId) -> Box<dyn PluralRules>;

    /// Get a collator for the locale
    fn get_collator(&self, locale: &LocaleId) -> Box<dyn Collator>;

    /// Get a localized display name for a language
    fn get_language_display_name(&self, language: &LanguageId, locale: &LocaleId) -> String;

    /// Get a localized display name for a region
    fn get_region_display_name(&self, region: &RegionId, locale: &LocaleId) -> String;

    /// Get a localized display name for a script
    fn get_script_display_name(&self, script: &ScriptId, locale: &LocaleId) -> String;
}

/// Factory for creating CLDR data providers
pub struct CldrDataFactory;

impl CldrDataFactory {
    /// Create a new CLDR data provider
    pub fn new() -> Box<dyn LocaleDataProvider> {
        Box::new(DefaultLocaleDataProvider::new())
    }
}

/// Default implementation of LocaleDataProvider
struct DefaultLocaleDataProvider;

impl DefaultLocaleDataProvider {
    /// Create a new default locale data provider
    fn new() -> Self {
        Self
    }
}

impl LocaleDataProvider for DefaultLocaleDataProvider {
    fn get_datetime_formatter(&self, _locale: &LocaleId) -> Box<dyn DateTimeFormatter> {
        Box::new(calendar::CalendarFormatter::gregorian())
    }

    fn get_number_formatter(&self, _locale: &LocaleId) -> Box<dyn NumberFormatter> {
        Box::new(number::DefaultNumberFormatter::latin())
    }

    fn get_plural_rules(&self, _locale: &LocaleId) -> Box<dyn PluralRules> {
        Box::new(number::DefaultPluralRules::new())
    }

    fn get_collator(&self, _locale: &LocaleId) -> Box<dyn Collator> {
        Box::new(collation::BasicCollator::new())
    }

    fn get_language_display_name(&self, language: &LanguageId, _locale: &LocaleId) -> String {
        // This would use icu4x in a real implementation
        match language.as_str() {
            "en" => "English".to_string(),
            "fr" => "French".to_string(),
            "de" => "German".to_string(),
            "ja" => "Japanese".to_string(),
            "zh" => "Chinese".to_string(),
            _ => language.as_str().to_string(),
        }
    }

    fn get_region_display_name(&self, region: &RegionId, _locale: &LocaleId) -> String {
        // This would use icu4x in a real implementation
        match region.as_str() {
            "US" => "United States".to_string(),
            "GB" => "United Kingdom".to_string(),
            "FR" => "France".to_string(),
            "DE" => "Germany".to_string(),
            "JP" => "Japan".to_string(),
            "CN" => "China".to_string(),
            _ => region.as_str().to_string(),
        }
    }

    fn get_script_display_name(&self, script: &ScriptId, _locale: &LocaleId) -> String {
        // This would use icu4x in a real implementation
        match script.as_str() {
            "Latn" => "Latin".to_string(),
            "Cyrl" => "Cyrillic".to_string(),
            "Arab" => "Arabic".to_string(),
            "Hans" => "Simplified Chinese".to_string(),
            "Hant" => "Traditional Chinese".to_string(),
            "Jpan" => "Japanese".to_string(),
            _ => script.as_str().to_string(),
        }
    }
} 