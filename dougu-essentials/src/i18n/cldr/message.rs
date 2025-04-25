// Message formatting functionality

use crate::i18n::{CurrencyCode, LocaleId};
use crate::time::{LocalDate, LocalTime, ZonedDateTime};
use std::collections::HashMap;

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

/// Arguments for message formatting
pub struct MessageArgs {
    args: HashMap<String, MessageValue>,
}

impl MessageArgs {
    /// Create a new empty message args
    pub fn new() -> Self {
        Self { args: HashMap::new() }
    }

    /// Add a string argument
    pub fn with_string(mut self, key: &str, value: String) -> Self {
        self.args.insert(key.to_string(), MessageValue::String(value));
        self
    }

    /// Add a number argument
    pub fn with_number(mut self, key: &str, value: f64) -> Self {
        self.args.insert(key.to_string(), MessageValue::Number(value));
        self
    }

    /// Add an integer argument
    pub fn with_integer(mut self, key: &str, value: i64) -> Self {
        self.args.insert(key.to_string(), MessageValue::Integer(value));
        self
    }

    /// Add a date argument
    pub fn with_date(mut self, key: &str, value: LocalDate) -> Self {
        self.args.insert(key.to_string(), MessageValue::Date(value));
        self
    }

    /// Add a time argument
    pub fn with_time(mut self, key: &str, value: LocalTime) -> Self {
        self.args.insert(key.to_string(), MessageValue::Time(value));
        self
    }

    /// Add a date time argument
    pub fn with_datetime(mut self, key: &str, value: ZonedDateTime) -> Self {
        self.args.insert(key.to_string(), MessageValue::DateTime(value));
        self
    }

    /// Add a currency argument
    pub fn with_currency(mut self, key: &str, value: f64, currency: CurrencyCode) -> Self {
        self.args.insert(key.to_string(), MessageValue::Currency(value, currency));
        self
    }

    /// Get an argument by key
    pub fn get(&self, key: &str) -> Option<&MessageValue> {
        self.args.get(key)
    }
}

/// Formatted message for internationalization
pub trait Message {
    /// Get the formatted message for the given locale
    fn format(&self, locale: &LocaleId) -> String;

    /// Get the formatted message with arguments
    fn format_with_args(&self, locale: &LocaleId, args: &MessageArgs) -> String;
}

/// A simple message implementation
pub struct SimpleMessage {
    message_id: String,
    translations: HashMap<String, String>,
}

impl SimpleMessage {
    /// Create a new simple message
    pub fn new(message_id: &str) -> Self {
        Self {
            message_id: message_id.to_string(),
            translations: HashMap::new(),
        }
    }

    /// Add a translation for a locale
    pub fn with_translation(mut self, locale: &LocaleId, message: &str) -> Self {
        self.translations.insert(locale.to_string(), message.to_string());
        self
    }

    /// Get the message ID
    pub fn message_id(&self) -> &str {
        &self.message_id
    }
}

impl Message for SimpleMessage {
    fn format(&self, locale: &LocaleId) -> String {
        // Try to find an exact match for the locale
        if let Some(message) = self.translations.get(&locale.to_string()) {
            return message.clone();
        }

        // Try to find a match for just the language
        let language_code = locale.language().as_str();
        for (locale_key, message) in &self.translations {
            if locale_key.starts_with(language_code) {
                return message.clone();
            }
        }

        // Fallback to English
        if let Some(message) = self.translations.get("en") {
            return message.clone();
        }

        // Last resort: return first available translation
        self.translations.values().next().cloned().unwrap_or_else(|| format!("?{}?", self.message_id))
    }

    fn format_with_args(&self, locale: &LocaleId, args: &MessageArgs) -> String {
        // Get the base message
        let mut message = self.format(locale);

        // Replace placeholders with argument values
        for (key, value) in &args.args {
            let placeholder = format!("{{{}}}", key);
            let replacement = match value {
                MessageValue::String(s) => s.clone(),
                MessageValue::Number(n) => n.to_string(),
                MessageValue::Integer(i) => i.to_string(),
                MessageValue::Date(d) => format!("{:04}-{:02}-{:02}", d.year(), d.month(), d.day()),
                MessageValue::Time(t) => format!("{:02}:{:02}:{:02}", t.hour(), t.minute(), t.second()),
                MessageValue::DateTime(dt) => {
                    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                            dt.year(), dt.month(), dt.day(),
                            dt.hour(), dt.minute(), dt.second())
                }
                MessageValue::Currency(amount, code) => format!("{} {}", code.as_str(), amount),
            };
            message = message.replace(&placeholder, &replacement);
        }

        message
    }
}

/// Repository of messages
pub struct MessageRepository {
    messages: HashMap<String, Box<dyn Message>>,
}

impl MessageRepository {
    /// Create a new message repository
    pub fn new() -> Self {
        Self { messages: HashMap::new() }
    }

    /// Add a message to the repository
    pub fn add_message(&mut self, message_id: &str, message: Box<dyn Message>) {
        self.messages.insert(message_id.to_string(), message);
    }

    /// Get a message by ID
    pub fn get_message(&self, message_id: &str) -> Option<&Box<dyn Message>> {
        self.messages.get(message_id)
    }
} 