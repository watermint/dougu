use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::fs::File;
use std::io::Read;

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
    pub fn load_advanced_file<P: AsRef<Path>>(&mut self, locale: &str, path: P) -> Result<()> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        
        let translations: AdvancedLocaleMap = serde_json::from_str(&content)?;
        self.advanced_locales.insert(locale.to_string(), translations);
        
        Ok(())
    }
    
    /// Load advanced translations from string content
    /// This allows embedding translations in the binary
    pub fn load_content(&mut self, locale: &str, content: &str) -> Result<()> {
        let translations: AdvancedLocaleMap = serde_json::from_str(content)?;
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

#[cfg(test)]
mod tests {
    use super::*;

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
} 