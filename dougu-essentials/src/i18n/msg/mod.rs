// Message management module
// This module provides a generic interface for message bundles and implementations
// for specific message formats like Fluent and JSON.

use std::collections::HashMap;
use std::sync::Arc;

use crate::core::error::{error, Result};
use crate::i18n::locale::LocaleId;

pub mod format;

// Re-export specific implementations
pub use format::MessageFormat;

/// MessageArgs represents arguments to a message that can be passed to formatting
#[derive(Debug, Clone, Default)]
pub struct MessageArgs {
    args: HashMap<String, String>,
}

impl MessageArgs {
    /// Create a new empty MessageArgs
    pub fn new() -> Self {
        Self {
            args: HashMap::new(),
        }
    }

    /// Add a string argument
    pub fn add<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) -> &mut Self {
        self.args.insert(key.into(), value.into());
        self
    }

    /// Create MessageArgs with a single argument
    pub fn with<K: Into<String>, V: Into<String>>(key: K, value: V) -> Self {
        let mut args = Self::new();
        args.add(key, value);
        args
    }

    /// Get all arguments as a reference to the internal HashMap
    pub fn args(&self) -> &HashMap<String, String> {
        &self.args
    }
}

/// Unified message bundle implementation that works with any format
#[derive(Debug, Clone)]
pub struct MessageBundle {
    locale: LocaleId,
    messages: HashMap<String, String>,
}

impl MessageBundle {
    /// Create a new empty message bundle for the specified locale
    pub fn new(locale: LocaleId) -> Self {
        Self {
            locale,
            messages: HashMap::new(),
        }
    }

    /// Get the locale for this bundle
    pub fn locale(&self) -> &LocaleId {
        &self.locale
    }

    /// Add a single message to the bundle
    pub fn add_message<K: Into<String>, V: Into<String>>(&mut self, key: K, message: V) -> &mut Self {
        self.messages.insert(key.into(), message.into());
        self
    }

    /// Add multiple messages from a HashMap
    pub fn add_messages(&mut self, messages: HashMap<String, String>) -> &mut Self {
        self.messages.extend(messages);
        self
    }

    /// Get all messages in this bundle
    pub fn messages(&self) -> &HashMap<String, String> {
        &self.messages
    }

    /// Format a message with the given ID and arguments
    pub fn format_message(&self, message_id: &str, args: Option<&MessageArgs>) -> Result<String> {
        let template = self.messages.get(message_id)
            .ok_or_else(|| error(format!("Message not found: {}", message_id)))?;

        // If no args, return the template directly
        if args.is_none() || args.unwrap().args().is_empty() {
            return Ok(template.clone());
        }

        // Simple placeholder replacement using {name} format
        let mut result = template.clone();
        if let Some(message_args) = args {
            for (key, value) in message_args.args().iter() {
                let placeholder = format!("{{{}}}", key);
                result = result.replace(&placeholder, value);
            }
        }

        Ok(result)
    }

    /// Merge another bundle into this one
    /// Messages from the other bundle will overwrite existing messages with the same key
    pub fn merge(&mut self, other: &Self) -> Result<()> {
        if self.locale != other.locale {
            return Err(error(format!(
                "Cannot merge bundles with different locales: {:?} and {:?}",
                self.locale, other.locale
            )));
        }

        self.messages.extend(other.messages.clone());
        Ok(())
    }
}

/// ResourceManager manages multiple bundles and handles locale fallback
#[derive(Debug, Clone)]
pub struct ResourceManager {
    bundles: HashMap<LocaleId, Arc<MessageBundle>>,
    fallback_locale: LocaleId,
}

impl ResourceManager {
    /// Create a new ResourceManager with the specified fallback locale
    pub fn new(fallback_locale: LocaleId) -> Self {
        Self {
            bundles: HashMap::new(),
            fallback_locale,
        }
    }

    /// Add a message bundle
    pub fn add_bundle(&mut self, bundle: MessageBundle) -> Result<()> {
        let locale = bundle.locale().clone();

        // If we already have a bundle for this locale, merge them
        if let Some(existing_bundle) = self.bundles.get(&locale) {
            let mut new_bundle = (**existing_bundle).clone();
            new_bundle.merge(&bundle)?;
            self.bundles.insert(locale, Arc::new(new_bundle));
        } else {
            // Otherwise just insert the new bundle
            self.bundles.insert(locale, Arc::new(bundle));
        }

        Ok(())
    }

    /// Format a message with the given ID and arguments for the specified locale
    pub fn format_message(
        &self,
        locale: &LocaleId,
        message_id: &str,
        args: Option<&MessageArgs>,
    ) -> Result<String> {
        // Try the requested locale first
        if let Some(bundle) = self.bundles.get(locale) {
            match bundle.format_message(message_id, args) {
                Ok(message) => return Ok(message),
                Err(_) => {} // Continue to fallback
            }
        }

        // Fall back to fallback locale if requested locale failed
        if locale != &self.fallback_locale {
            if let Some(bundle) = self.bundles.get(&self.fallback_locale) {
                match bundle.format_message(message_id, args) {
                    Ok(message) => return Ok(message),
                    Err(_) => {} // Continue to last resort
                }
            }
        }

        // If all attempts fail, return the message ID as a last resort
        Ok(message_id.to_string())
    }

    /// Set the fallback locale and returns the previous one
    pub fn set_fallback_locale(&mut self, locale: LocaleId) -> LocaleId {
        let previous = self.fallback_locale.clone();
        self.fallback_locale = locale;
        previous
    }

    /// Get the current fallback locale
    pub fn fallback_locale(&self) -> &LocaleId {
        &self.fallback_locale
    }
}

/// MessageFormatter provides a simplified interface for message formatting
#[derive(Debug, Clone)]
pub struct MessageFormatter {
    manager: Arc<ResourceManager>,
    locale: LocaleId,
}

impl MessageFormatter {
    /// Create a new MessageFormatter with the given ResourceManager and locale
    pub fn new(manager: Arc<ResourceManager>, locale: LocaleId) -> Self {
        Self { manager, locale }
    }

    /// Format a message with the given ID and arguments
    pub fn format(
        &self,
        message_id: &str,
        args: Option<&MessageArgs>,
    ) -> Result<String> {
        self.manager.format_message(&self.locale, message_id, args)
    }

    /// Set the current locale and returns the previous one
    pub fn set_locale(&mut self, locale: LocaleId) -> LocaleId {
        let previous = self.locale.clone();
        self.locale = locale;
        previous
    }

    /// Get the current locale
    pub fn locale(&self) -> &LocaleId {
        &self.locale
    }
} 