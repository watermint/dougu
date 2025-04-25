use icu_locid::Locale;
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

    pub fn as_str(&self) -> &str {
        self.language.as_str()
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
        // Try to detect the system locale, with fallback to en-US
        self.detect_system_locale().unwrap_or_else(|_| {
            // Fallback to en-US if locale detection fails
            LocaleId::new(
                LanguageId::new("en"),
                Some(RegionId::new("US")),
            )
        })
    }

    /// Attempt to detect the system locale
    fn detect_system_locale(&self) -> Result<LocaleId, &'static str> {
        // Get the locale string from the OS
        let locale_str = self.get_locale_string();

        // Convert the locale string to a proper LocaleId
        match locale_str.parse::<LocaleId>() {
            Ok(locale_id) => Ok(locale_id),
            Err(_) => {
                // Try to parse it using ICU
                match Locale::try_from_bytes(locale_str.as_bytes()) {
                    Ok(locale) => {
                        let language = LanguageId::new(locale.id.language.as_str());
                        let region = locale.id.region.map(|r| RegionId::new(r.as_str()));
                        Ok(LocaleId::new(language, region))
                    }
                    Err(_) => Err("Failed to parse system locale"),
                }
            }
        }
    }

    /// Get the locale string from the system
    fn get_locale_string(&self) -> String {
        #[cfg(any(
            target_os = "linux",
            target_os = "android",
            target_os = "freebsd",
            target_os = "openbsd",
            target_os = "netbsd",
            target_os = "dragonfly",
            target_os = "solaris",
            target_os = "illumos",
            target_os = "haiku"
        ))]
        {
            // Unix-like systems: check LC_ALL, LC_MESSAGES, LANG environment variables
            for env_var in &["LC_ALL", "LC_MESSAGES", "LANG"] {
                if let Ok(locale) = std::env::var(env_var) {
                    if !locale.is_empty() {
                        return self.sanitize_locale_string(&locale);
                    }
                }
            }

            // Fallback to en-US
            String::from("en-US")
        }

        #[cfg(target_os = "macos")]
        {
            // On macOS, we can use environment variables first
            for env_var in &["LC_ALL", "LC_MESSAGES", "LANG"] {
                if let Ok(locale) = std::env::var(env_var) {
                    if !locale.is_empty() {
                        return self.sanitize_locale_string(&locale);
                    }
                }
            }

            // Alternatively, use the defaults command (more reliable for user settings)
            use std::process::Command;

            let output = Command::new("defaults")
                .args(["read", "-g", "AppleLocale"])
                .output();

            match output {
                Ok(output) if output.status.success() => {
                    let locale = String::from_utf8_lossy(&output.stdout)
                        .trim()
                        .to_string();
                    if !locale.is_empty() {
                        return self.sanitize_locale_string(&locale);
                    }
                }
                _ => {}
            }

            // Fallback to en-US
            String::from("en-US")
        }

        #[cfg(target_os = "windows")]
        {
            // On Windows, try environment variables first
            for env_var in &["LC_ALL", "LC_MESSAGES", "LANG"] {
                if let Ok(locale) = std::env::var(env_var) {
                    if !locale.is_empty() {
                        return self.sanitize_locale_string(&locale);
                    }
                }
            }

            // Then try PowerShell
            use std::process::Command;

            let output = Command::new("powershell")
                .args(["-Command", "(Get-Culture).Name"])
                .output();

            match output {
                Ok(output) if output.status.success() => {
                    let locale = String::from_utf8_lossy(&output.stdout)
                        .trim()
                        .to_string();
                    if !locale.is_empty() {
                        return self.sanitize_locale_string(&locale);
                    }
                }
                _ => {}
            }

            // Fallback to en-US
            String::from("en-US")
        }

        #[cfg(not(any(
            target_os = "linux",
            target_os = "android",
            target_os = "macos",
            target_os = "ios",
            target_os = "freebsd",
            target_os = "openbsd",
            target_os = "netbsd",
            target_os = "dragonfly",
            target_os = "solaris",
            target_os = "illumos",
            target_os = "haiku",
            target_os = "windows"
        )))]
        {
            // Fallback to en-US for unsupported platforms
            String::from("en-US")
        }
    }

    /// Sanitize locale string to ensure it's in a valid BCP 47 format
    fn sanitize_locale_string(&self, locale: &str) -> String {
        // Remove encoding information if present (e.g., en_US.UTF-8 -> en_US)
        let locale = locale.split('.').next().unwrap_or(locale);
        // Remove any variants or extensions (e.g., en_US@posix -> en_US)
        let locale = locale.split('@').next().unwrap_or(locale);
        // Convert underscore to hyphen for BCP 47 format (e.g., en_US -> en-US)
        let locale = locale.replace('_', "-");

        locale
    }

    /// Parse a locale string
    pub fn parse_locale(&self, locale_str: &str) -> Result<LocaleId, &'static str> {
        locale_str.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_parse_language_id() {
        // Valid language codes
        assert!(LanguageId::from_str("en").is_ok());
        assert!(LanguageId::from_str("ja").is_ok());
        assert!(LanguageId::from_str("eng").is_ok());

        // Invalid language codes
        assert!(LanguageId::from_str("").is_err());
        assert!(LanguageId::from_str("e").is_err());
        assert!(LanguageId::from_str("english").is_err());
        assert!(LanguageId::from_str("en-US").is_err());
        assert!(LanguageId::from_str("123").is_err());
    }

    #[test]
    fn test_parse_region_id() {
        // Valid region codes
        assert!(RegionId::from_str("US").is_ok());
        assert!(RegionId::from_str("JP").is_ok());
        assert!(RegionId::from_str("uk").is_ok()); // Should be ok, casing doesn't matter

        // Invalid region codes
        assert!(RegionId::from_str("").is_err());
        assert!(RegionId::from_str("U").is_err());
        assert!(RegionId::from_str("USA").is_err());
        assert!(RegionId::from_str("12").is_err());
    }

    #[test]
    fn test_parse_locale_id() {
        // Valid locale IDs
        let en_us = LocaleId::from_str("en-US").unwrap();
        assert_eq!(en_us.language().as_str(), "en");
        assert_eq!(en_us.region().unwrap().as_str(), "US");

        let ja = LocaleId::from_str("ja").unwrap();
        assert_eq!(ja.language().as_str(), "ja");
        assert_eq!(ja.region(), None);

        // Invalid locale IDs
        assert!(LocaleId::from_str("").is_err());
        assert!(LocaleId::from_str("en-US-variant").is_err());
        assert!(LocaleId::from_str("english-USA").is_err());
    }

    #[test]
    fn test_locale_id_to_string() {
        let en_us = LocaleId::new(
            LanguageId::new("en"),
            Some(RegionId::new("US")),
        );
        assert_eq!(en_us.to_string(), "en-US");

        let ja = LocaleId::new(
            LanguageId::new("ja"),
            None,
        );
        assert_eq!(ja.to_string(), "ja");
    }

    #[test]
    fn test_sanitize_locale_string() {
        let service = LocaleService::new();

        assert_eq!(service.sanitize_locale_string("en_US"), "en-US");
        assert_eq!(service.sanitize_locale_string("en_US.UTF-8"), "en-US");
        assert_eq!(service.sanitize_locale_string("en_US@posix"), "en-US");
        assert_eq!(service.sanitize_locale_string("en-US"), "en-US");
        assert_eq!(service.sanitize_locale_string("ja_JP.UTF-8@japaneseCalendar"), "ja-JP");
    }

    #[test]
    fn test_get_locale_with_env_vars() {
        // Test with environment variables
        // Save original env vars to restore after test
        let original_lc_all = env::var("LC_ALL").ok();
        let original_lc_messages = env::var("LC_MESSAGES").ok();
        let original_lang = env::var("LANG").ok();

        // Test with LC_ALL set
        unsafe { env::set_var("LC_ALL", "fr-FR"); }
        let service = LocaleService::new();
        let locale_str = service.get_locale_string();
        assert_eq!(locale_str, "fr-FR");

        // Test with LC_MESSAGES set (LC_ALL takes precedence)
        unsafe { env::set_var("LC_ALL", ""); }
        unsafe { env::set_var("LC_MESSAGES", "de-DE"); }
        let locale_str = service.get_locale_string();
        assert_eq!(locale_str, "de-DE");

        // Test with LANG set (lowest precedence)
        unsafe { env::set_var("LC_MESSAGES", ""); }
        unsafe { env::set_var("LANG", "es-ES"); }
        let locale_str = service.get_locale_string();
        assert_eq!(locale_str, "es-ES");

        // Restore original environment variables
        match original_lc_all {
            Some(val) => unsafe { env::set_var("LC_ALL", val) },
            None => unsafe { env::remove_var("LC_ALL") },
        }
        match original_lc_messages {
            Some(val) => unsafe { env::set_var("LC_MESSAGES", val) },
            None => unsafe { env::remove_var("LC_MESSAGES") },
        }
        match original_lang {
            Some(val) => unsafe { env::set_var("LANG", val) },
            None => unsafe { env::remove_var("LANG") },
        }
    }

    #[test]
    fn test_system_locale() {
        // This is more of a functional test that should return something valid
        let service = LocaleService::new();
        let locale = service.system_locale();

        // Just check that we got something reasonable back
        assert!(!locale.language().as_str().is_empty());

        // Test the system locale representation
        println!("Detected system locale: {}", locale.to_string());
    }
} 