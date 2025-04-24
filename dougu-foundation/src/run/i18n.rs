use std::collections::HashMap;
use crate::i18n::Locale;

pub struct I18nRunner {
    locale: Locale,
    messages: HashMap<String, String>,
}

impl I18nRunner {
    pub fn new(locale: Locale) -> Self {
        Self {
            locale,
            messages: HashMap::new(),
        }
    }

    pub fn t(&self, key: &str) -> String {
        self.messages.get(key)
            .map(|s| s.to_string())
            .unwrap_or_else(|| key.to_string())
    }

    pub fn tf(&self, key: &str, vars: &[(&str, &str)]) -> String {
        let mut result = self.t(key);
        for (k, v) in vars {
            result = result.replace(&format!("{{{}}}", k), v);
        }
        result
    }
}

pub fn t(key: &str) -> String {
    key.to_string()
}

pub fn tf(key: &str, vars: &[(&str, &str)]) -> String {
    let mut result = key.to_string();
    for (k, v) in vars {
        result = result.replace(&format!("{{{}}}", k), v);
    }
    result
} 