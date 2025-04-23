use std::fmt;
use std::str::FromStr;

/// Represents a BCP 47 language tag with components like language, script, region, etc.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Locale {
    /// The raw language tag string (e.g., "en-US", "zh-Hans-CN")
    raw: String,
    /// The language subtag (e.g., "en", "zh")
    language: String,
    /// The optional script subtag (e.g., "Hans", "Latn")
    script: Option<String>,
    /// The optional region subtag (e.g., "US", "CN")
    region: Option<String>,
    /// Any additional variant subtags
    variants: Vec<String>,
}

/// Error type for locale parsing failures
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleError {
    pub message: String,
}

impl fmt::Display for LocaleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Locale error: {}", self.message)
    }
}

impl std::error::Error for LocaleError {}

impl Locale {
    /// Create a new Locale with the given language
    pub fn new(language: &str) -> Self {
        Self {
            raw: language.to_string(),
            language: language.to_string(),
            script: None,
            region: None,
            variants: Vec::new(),
        }
    }

    /// Create a new Locale with the given language and region
    pub fn with_region(language: &str, region: &str) -> Self {
        let raw = format!("{}-{}", language, region);
        Self {
            raw,
            language: language.to_string(),
            script: None,
            region: Some(region.to_string()),
            variants: Vec::new(),
        }
    }

    /// Create a new Locale with the given language, script, and region
    pub fn with_script_region(language: &str, script: &str, region: &str) -> Self {
        let raw = format!("{}-{}-{}", language, script, region);
        Self {
            raw,
            language: language.to_string(),
            script: Some(script.to_string()),
            region: Some(region.to_string()),
            variants: Vec::new(),
        }
    }

    /// Get the raw language tag
    pub fn as_str(&self) -> &str {
        &self.raw
    }

    /// Get the language subtag
    pub fn language(&self) -> &str {
        &self.language
    }

    /// Get the script subtag if present
    pub fn script(&self) -> Option<&str> {
        self.script.as_deref()
    }

    /// Get the region subtag if present
    pub fn region(&self) -> Option<&str> {
        self.region.as_deref()
    }

    /// Get all variant subtags
    pub fn variants(&self) -> &[String] {
        &self.variants
    }

    /// Check if this locale has the same base language as another locale
    pub fn same_language(&self, other: &Locale) -> bool {
        self.language.eq_ignore_ascii_case(&other.language)
    }

    /// Get an iterator over fallback locales in order of specificity
    /// e.g., "zh-Hans-CN" -> ["zh-Hans-CN", "zh-Hans", "zh"]
    pub fn fallbacks(&self) -> Vec<Locale> {
        let mut fallbacks = Vec::new();
        
        // Add the full locale
        fallbacks.push(self.clone());
        
        // Add language + script if available
        if self.script.is_some() && self.region.is_some() {
            fallbacks.push(Locale::with_script_region(
                &self.language,
                self.script.as_ref().unwrap(),
                "",
            ));
        }
        
        // Add language + region if available
        if self.region.is_some() {
            fallbacks.push(Locale::with_region(
                &self.language,
                self.region.as_ref().unwrap(),
            ));
        }
        
        // Add language only
        if self.script.is_some() || self.region.is_some() || !self.variants.is_empty() {
            fallbacks.push(Locale::new(&self.language));
        }
        
        fallbacks
    }
}

impl FromStr for Locale {
    type Err = LocaleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(LocaleError {
                message: "Empty language tag".to_string(),
            });
        }

        let parts: Vec<&str> = s.split('-').collect();
        
        if parts.is_empty() {
            return Err(LocaleError {
                message: "Invalid language tag format".to_string(),
            });
        }

        let language = parts[0].to_string();
        
        // Validate language
        if !language.chars().all(|c| c.is_ascii_alphabetic()) {
            return Err(LocaleError {
                message: format!("Invalid language subtag: {}", language),
            });
        }

        let mut locale = Locale {
            raw: s.to_string(),
            language,
            script: None,
            region: None,
            variants: Vec::new(),
        };

        // Parse remaining parts
        for (i, part) in parts.iter().enumerate().skip(1) {
            let part = *part;
            
            if i == 1 && part.len() == 4 && part.chars().all(|c| c.is_ascii_alphabetic()) {
                // Script subtag (e.g., "Hans" in "zh-Hans-CN")
                locale.script = Some(part.to_string());
            } else if i == 1 || (i == 2 && locale.script.is_some()) {
                if part.len() == 2 && part.chars().all(|c| c.is_ascii_alphabetic()) {
                    // 2-letter region code (e.g., "US" in "en-US")
                    locale.region = Some(part.to_string());
                } else if part.len() == 3 && part.chars().all(|c| c.is_ascii_digit()) {
                    // 3-digit region code
                    locale.region = Some(part.to_string());
                } else {
                    // Must be a variant
                    locale.variants.push(part.to_string());
                }
            } else {
                // Additional subtags as variants
                locale.variants.push(part.to_string());
            }
        }

        Ok(locale)
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw)
    }
}

impl Default for Locale {
    fn default() -> Self {
        Self::new("en")
    }
}

/// Checks if a locale's language is supported by this application
/// This function determines if we have translations for this language
pub fn is_supported_language(locale: &Locale) -> bool {
    // Currently only English and Japanese are supported
    matches!(locale.language().to_lowercase().as_str(), "en" | "ja")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locale_parsing() {
        let locale = Locale::from_str("en-US").unwrap();
        assert_eq!(locale.language(), "en");
        assert_eq!(locale.region(), Some("US"));
        assert_eq!(locale.script(), None);

        let locale = Locale::from_str("zh-Hans-CN").unwrap();
        assert_eq!(locale.language(), "zh");
        assert_eq!(locale.script(), Some("Hans"));
        assert_eq!(locale.region(), Some("CN"));

        let locale = Locale::from_str("en").unwrap();
        assert_eq!(locale.language(), "en");
        assert_eq!(locale.region(), None);
        assert_eq!(locale.script(), None);
    }

    #[test]
    fn test_locale_fallbacks() {
        let locale = Locale::from_str("zh-Hans-CN").unwrap();
        let fallbacks = locale.fallbacks();
        
        assert_eq!(fallbacks.len(), 4);
        assert_eq!(fallbacks[0].as_str(), "zh-Hans-CN");
        assert_eq!(fallbacks[1].as_str(), "zh-Hans-");
        assert_eq!(fallbacks[2].as_str(), "zh-CN");
        assert_eq!(fallbacks[3].as_str(), "zh");
    }

    #[test]
    fn test_locale_same_language() {
        let en_us = Locale::from_str("en-US").unwrap();
        let en_gb = Locale::from_str("en-GB").unwrap();
        let ja = Locale::from_str("ja").unwrap();
        
        assert!(en_us.same_language(&en_gb));
        assert!(!en_us.same_language(&ja));
    }
} 