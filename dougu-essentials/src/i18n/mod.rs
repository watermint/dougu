use anyhow::{anyhow, Result};
use crate::obj::prelude::*;
use crate::obj::notation::{Notation, NotationType, JsonNotation};
use crate::obj::notation::json::{notation_type_to_json_value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str;
use serde_json;

pub mod integration;
pub mod locale;

pub use locale::{Locale, LocaleError};

/// Translation message container with metadata
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct TranslationMessage {
    /// The source text in default language
    pub source: String,
    /// Optional context to help translators understand the message usage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// Optional translator comments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// The translated text
    pub text: String,
}

impl TranslationMessage {
    /// Create a new translation message with just source and translated text
    pub fn new(source: &str, text: &str) -> Self {
        Self {
            source: source.to_string(),
            context: None,
            comments: None,
            text: text.to_string(),
        }
    }

    /// Add context to the translation message
    pub fn with_context(mut self, context: &str) -> Self {
        self.context = Some(context.to_string());
        self
    }

    /// Add translator comments
    pub fn with_comments(mut self, comments: &str) -> Self {
        self.comments = Some(comments.to_string());
        self
    }
    
    /// Replace variables in the translation text
    pub fn format(&self, variables: &HashMap<&str, &str>) -> String {
        let mut result = self.text.clone();
        for (key, value) in variables {
            result = result.replace(&format!("{{{}}}", key), value);
        }
        result
    }
}

/// Advanced locale map with full message containers
type AdvancedLocaleMap = HashMap<String, TranslationMessage>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct I18n {
    advanced_locales: HashMap<String, AdvancedLocaleMap>,
    current_locale: String,
}

impl I18n {
    /// Create a new I18n instance with default locale
    pub fn new(default_locale: &str) -> Self {
        Self {
            advanced_locales: HashMap::new(),
            current_locale: default_locale.to_string(),
        }
    }

    /// Load advanced translations with message containers from JSON file
    pub fn load_from_file<P: AsRef<Path>>(&mut self, locale: &str, file_path: P) -> Result<()> {
        let content = std::fs::read(file_path.as_ref())?; // read bytes directly
        let json_notation = JsonNotation;
        let decoded_notation: NotationType = json_notation.decode(&content)?;
        // Convert NotationType to AdvancedLocaleMap using the helper function
        let json_value = notation_type_to_json_value(&decoded_notation)?;
        let translations: AdvancedLocaleMap = serde_json::from_value(json_value)
            .map_err(|e| anyhow!("Decoded notation is not a valid AdvancedLocaleMap: {}", e))?;
        self.advanced_locales.insert(locale.to_string(), translations);
        Ok(())
    }
    
    /// Load advanced translations from string content
    /// This allows embedding translations in the binary
    pub fn load_content(&mut self, locale: &str, content: &str) -> Result<()> {
        let json_notation = JsonNotation;
        let decoded_notation: NotationType = json_notation.decode(content.as_bytes())?;
        // Convert NotationType to AdvancedLocaleMap using the helper function
        let json_value = notation_type_to_json_value(&decoded_notation)?;
        let translations: AdvancedLocaleMap = serde_json::from_value(json_value)
            .map_err(|e| anyhow!("Decoded notation is not a valid AdvancedLocaleMap: {}", e))?;
        self.advanced_locales.insert(locale.to_string(), translations);
        Ok(())
    }

    /// Set current locale
    pub fn set_locale(&mut self, locale: &str) -> Result<()> {
        let has_advanced = self.advanced_locales.contains_key(locale);
        
        if !has_advanced {
            return Err(anyhow!("Locale '{}' not loaded", locale));
        }
        
        self.current_locale = locale.to_string();
        Ok(())
    }

    /// Get translation for key from advanced locale map
    pub fn translate(&self, key: &str) -> Result<&str> {
        if let Some(locale_map) = self.advanced_locales.get(&self.current_locale) {
            if let Some(message) = locale_map.get(key) {
                return Ok(&message.text);
            }
        }
        Err(anyhow!("Translation key '{}' not found", key))
    }
    
    /// Get translation message container
    pub fn translate_message(&self, key: &str) -> Result<&TranslationMessage> {
        let locale_map = self.advanced_locales.get(&self.current_locale)
            .ok_or_else(|| anyhow!("Advanced locale '{}' not loaded", self.current_locale))?;
            
        locale_map.get(key)
            .ok_or_else(|| anyhow!("Translation message '{}' not found", key))
    }
    
    /// Shorthand for translate
    pub fn t(&self, key: &str) -> String {
        self.translate(key).unwrap_or(key).to_string()
    }
    
    /// Get a translation with variable substitution
    pub fn tf(&self, key: &str, variables: &HashMap<&str, &str>) -> String {
        match self.translate_message(key) {
            Ok(msg) => msg.format(variables),
            Err(_) => key.to_string(),
        }
    }
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub struct LocaleMap {
    translations: HashMap<String, String>,
}

impl LocaleMap {
    pub fn new() -> Self {
        Self {
            translations: HashMap::new(),
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let json_notation = JsonNotation;
        let notation: NotationType = json_notation.decode(content.as_bytes())?;
        
        let mut translations = HashMap::new();
        if let NotationType::Object(obj) = notation {
            for (key, value) in obj {
                if let NotationType::String(s) = value {
                    translations.insert(key, s);
                }
            }
        }
        
        Ok(Self { translations })
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.translations.get(key)
    }

    pub fn insert(&mut self, key: String, value: String) {
        self.translations.insert(key, value);
    }

    pub fn to_string(&self) -> Result<String> {
        let json_notation = JsonNotation;
        json_notation.encode_to_string(&self.translations) // Encode the map directly
    }
}

impl Into<NotationType> for LocaleMap {
    fn into(self) -> NotationType {
        NotationType::Object(
            self.translations
                .into_iter()
                .map(|(k, v)| (k, NotationType::String(v)))
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
    
    #[test]
    fn test_translation_message() {
        let msg = TranslationMessage::new("Hello", "Bonjour")
            .with_context("Greeting")
            .with_comments("Formal greeting");
            
        assert_eq!(msg.source, "Hello");
        assert_eq!(msg.text, "Bonjour");
        assert_eq!(msg.context, Some("Greeting".to_string()));
        assert_eq!(msg.comments, Some("Formal greeting".to_string()));
    }
    
    #[test]
    fn test_format_variables() {
        let msg = TranslationMessage::new(
            "Hello, {name}!", 
            "Bonjour, {name}!"
        );
        
        let mut vars = HashMap::new();
        vars.insert("name", "Alice");
        
        assert_eq!(msg.format(&vars), "Bonjour, Alice!");
    }

    #[test]
    fn test_locale_map() -> Result<()> {
        let mut map = LocaleMap::new();
        
        // Test inserting and getting translations
        map.insert("hello".to_string(), "こんにちは".to_string());
        assert_eq!(map.get("hello"), Some(&"こんにちは".to_string()));
        
        // Test serialization and deserialization
        let temp_file = NamedTempFile::new()?;
        let notation: NotationType = map.into();
        let json_notation = JsonNotation;
        let content = json_notation.encode_to_string(&notation)?;
        std::fs::write(temp_file.path(), content)?;
        
        let loaded_map = LocaleMap::from_file(temp_file.path())?;
        assert_eq!(loaded_map.get("hello"), Some(&"こんにちは".to_string()));
        
        Ok(())
    }
} 