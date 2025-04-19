use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::fs::File;
use std::io::Read;

type LocaleMap = HashMap<String, String>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct I18n {
    locales: HashMap<String, LocaleMap>,
    current_locale: String,
}

impl I18n {
    /// Create a new I18n instance with default locale
    pub fn new(default_locale: &str) -> Self {
        Self {
            locales: HashMap::new(),
            current_locale: default_locale.to_string(),
        }
    }

    /// Load translations from JSON file
    pub fn load_file<P: AsRef<Path>>(&mut self, locale: &str, path: P) -> Result<()> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        
        let translations: LocaleMap = serde_json::from_str(&content)?;
        self.locales.insert(locale.to_string(), translations);
        
        Ok(())
    }

    /// Set current locale
    pub fn set_locale(&mut self, locale: &str) -> Result<()> {
        if !self.locales.contains_key(locale) {
            return Err(anyhow!("Locale '{}' not loaded", locale));
        }
        self.current_locale = locale.to_string();
        Ok(())
    }

    /// Get translation for key
    pub fn translate(&self, key: &str) -> Result<&str> {
        let locale_map = self.locales.get(&self.current_locale)
            .ok_or_else(|| anyhow!("Current locale '{}' not loaded", self.current_locale))?;
            
        locale_map.get(key)
            .map(|s| s.as_str())
            .ok_or_else(|| anyhow!("Translation key '{}' not found", key))
    }
    
    /// Shorthand for translate
    pub fn t(&self, key: &str) -> String {
        self.translate(key).unwrap_or(key).to_string()
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
}
