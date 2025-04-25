// Language and locale related types and implementations
use std::str::FromStr;

/// Language identifier as defined by ISO 639
///
/// ISO 639-1: Two-letter codes (e.g., "en" for English)
/// ISO 639-2/T: Three-letter codes (e.g., "eng" for English)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LanguageId(String);

/// Region identifier as defined by ISO 3166-1 alpha-2
///
/// Two-letter country codes (e.g., "US" for United States)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RegionId(String);

/// Locale identifier following BCP 47 (RFC 5646) format
///
/// Combines language and optional region (e.g., "en-US")
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LocaleId {
    language: LanguageId,
    region: Option<RegionId>,
}

impl LanguageId {
    /// Create a new language identifier following ISO 639
    pub fn new(code: &str) -> Self {
        // Lowercase according to BCP 47
        Self(code.to_lowercase())
    }

    /// Get the language code
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for LanguageId {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // ISO 639-1 (two-letter) or ISO 639-2 (three-letter)
        if s.is_empty() || (s.len() != 2 && s.len() != 3) || !s.chars().all(|c| c.is_ascii_alphabetic()) {
            return Err("Invalid language identifier: must be 2 or 3 ASCII letters (ISO 639)");
        }
        Ok(Self::new(s))
    }
}

impl RegionId {
    /// Create a new region identifier following ISO 3166-1 alpha-2
    pub fn new(code: &str) -> Self {
        // Uppercase according to BCP 47
        Self(code.to_uppercase())
    }

    /// Get the region code
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for RegionId {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // ISO 3166-1 alpha-2 (two-letter)
        if s.len() != 2 || !s.chars().all(|c| c.is_ascii_alphabetic()) {
            return Err("Invalid region identifier: must be 2 ASCII letters (ISO 3166-1)");
        }
        Ok(Self::new(s))
    }
}

impl LocaleId {
    /// Create a new locale identifier following BCP 47 (RFC 5646)
    pub fn new(language: LanguageId, region: Option<RegionId>) -> Self {
        Self { language, region }
    }

    /// Get the language identifier
    pub fn language(&self) -> &LanguageId {
        &self.language
    }

    /// Get the region identifier if present
    pub fn region(&self) -> Option<&RegionId> {
        self.region.as_ref()
    }

    /// Format the locale identifier as a string following BCP 47
    pub fn to_string(&self) -> String {
        match &self.region {
            Some(region) => format!("{}-{}", self.language.as_str(), region.as_str()),
            None => self.language.as_str().to_string(),
        }
    }
}

impl FromStr for LocaleId {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Simple BCP 47 format: language-REGION
        let parts: Vec<&str> = s.split('-').collect();
        if parts.is_empty() || parts.len() > 2 {
            return Err("Invalid locale format: must follow BCP 47 (language-REGION)");
        }

        let language = LanguageId::from_str(parts[0])?;
        let region = if parts.len() > 1 {
            Some(RegionId::from_str(parts[1])?)
        } else {
            None
        };

        Ok(Self::new(language, region))
    }
}

/// Wraps the functionality of locale detection and parsing
pub struct LocaleService;

impl LocaleService {
    /// Creates a new locale service
    pub fn new() -> Self {
        Self
    }

    /// Get the system locale
    pub fn system_locale(&self) -> LocaleId {
        // This would use platform-specific APIs in a real implementation
        // For now, default to en-US
        LocaleId::new(
            LanguageId::new("en"),
            Some(RegionId::new("US")),
        )
    }

    /// Parse a locale string
    pub fn parse_locale(&self, locale_str: &str) -> Result<LocaleId, &'static str> {
        locale_str.parse()
    }
} 