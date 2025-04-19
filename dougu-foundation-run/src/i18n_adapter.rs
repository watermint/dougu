#[cfg(feature = "i18n")]
use dougu_foundation_i18n::{I18nInitializer, I18nContext, ErrorWithDetails, t, tf};
use dougu_essentials_i18n::Locale;
use crate::{CommandletError, LauncherLayer, LauncherContext};
use async_trait::async_trait;
use std::str::FromStr;

#[cfg(feature = "i18n")]
// Implement I18nContext for LauncherContext to make it compatible with I18nInitializer
impl I18nContext for LauncherContext {
    fn get_context_data(&self, key: &str) -> Option<&String> {
        if key == "locale" || key == "active_locale" {
            // Ensure we have the active_locale stored in the data map
            if self.get_data("active_locale").is_none() {
                // This is a temporary workaround - we need to update the data map
                // but this method is &self, not &mut self
                // In a real implementation, ensure active_locale is set before calling this method
                log::warn!("active_locale not found in context data map");
            }
            self.get_data("active_locale")
        } else {
            self.get_data(key)
        }
    }
    
    fn set_context_data(&mut self, key: &str, value: String) {
        if key == "locale" || key == "active_locale" {
            // For locale keys, update the locale field
            if let Ok(locale) = Locale::from_str(&value) {
                self.set_locale(locale);
            } else {
                // If parsing fails, at least store the value in data map
                self.set_data(key, value);
            }
        } else {
            self.set_data(key, value);
        }
    }
}

#[cfg(feature = "i18n")]
// Implement ErrorWithDetails for CommandletError to make it compatible with i18n-foundation
impl ErrorWithDetails for CommandletError {
    fn new_with_i18n(code: &str, key: &str) -> Self {
        Self {
            code: code.to_string(),
            message: t(key),
            details: None,
        }
    }
    
    fn with_i18n_vars(code: &str, key: &str, vars: &[(&str, &str)]) -> Self {
        Self {
            code: code.to_string(),
            message: tf(key, vars),
            details: None,
        }
    }
    
    fn with_i18n_details(code: &str, key: &str, details: &str) -> Self {
        Self {
            code: code.to_string(),
            message: t(key),
            details: Some(details.to_string()),
        }
    }
    
    fn with_i18n_vars_details(code: &str, key: &str, vars: &[(&str, &str)], details: &str) -> Self {
        Self {
            code: code.to_string(),
            message: tf(key, vars),
            details: Some(details.to_string()),
        }
    }
}

#[cfg(feature = "i18n")]
/// I18nInitializerLayer implements LauncherLayer to initialize i18n during command execution
pub struct I18nInitializerLayer {
    initializer: I18nInitializer,
    default_locale: Locale,
}

#[cfg(feature = "i18n")]
impl I18nInitializerLayer {
    /// Create a new I18nInitializerLayer with the specified default locale
    pub fn new(default_locale: &str) -> Self {
        let locale = Locale::from_str(default_locale).unwrap_or_else(|_| Locale::default());
        Self {
            initializer: I18nInitializer::new(locale.language()),
            default_locale: locale,
        }
    }
    
    /// Create a new I18nInitializerLayer with a Locale object
    pub fn with_locale(locale: Locale) -> Self {
        Self {
            initializer: I18nInitializer::new(locale.language()),
            default_locale: locale,
        }
    }
    
    /// Create a new I18nInitializerLayer with the option to use filesystem resources
    pub fn with_filesystem(default_locale: &str, use_embedded: bool) -> Self {
        let locale = Locale::from_str(default_locale).unwrap_or_else(|_| Locale::default());
        Self {
            initializer: I18nInitializer::with_filesystem(locale.language(), use_embedded),
            default_locale: locale,
        }
    }
}

#[cfg(feature = "i18n")]
#[async_trait]
impl LauncherLayer for I18nInitializerLayer {
    fn name(&self) -> &str {
        "I18nInitializerLayer"
    }
    
    async fn run(&self, ctx: &mut LauncherContext) -> Result<(), String> {
        // Store default locale as a fallback (will be used if no locale key exists in context)
        let result = self.initializer.initialize(ctx);
        
        // If initialization succeeded, ensure locale field matches active_locale
        if result.is_ok() {
            if let Some(active_locale) = ctx.get_data("active_locale") {
                let current_locale = ctx.get_locale().as_str();
                if current_locale != active_locale {
                    if let Ok(locale) = Locale::from_str(active_locale) {
                        ctx.set_locale(locale);
                    }
                }
            }
        }
        
        result
    }
}

#[cfg(not(feature = "i18n"))]
/// Placeholder implementation when i18n feature is not enabled
pub struct I18nInitializerLayer {
    default_locale: Locale,
}

#[cfg(not(feature = "i18n"))]
impl I18nInitializerLayer {
    /// Create a new placeholder I18nInitializerLayer
    pub fn new(default_locale: &str) -> Self {
        let locale = Locale::from_str(default_locale).unwrap_or_else(|_| Locale::default());
        Self {
            default_locale: locale,
        }
    }
    
    /// Create a new I18nInitializerLayer with a Locale object
    pub fn with_locale(locale: Locale) -> Self {
        Self {
            default_locale: locale,
        }
    }
    
    /// Create a new placeholder I18nInitializerLayer
    pub fn with_filesystem(default_locale: &str, _use_embedded: bool) -> Self {
        let locale = Locale::from_str(default_locale).unwrap_or_else(|_| Locale::default());
        Self {
            default_locale: locale,
        }
    }
}

#[cfg(not(feature = "i18n"))]
#[async_trait]
impl LauncherLayer for I18nInitializerLayer {
    fn name(&self) -> &str {
        "I18nInitializerLayer"
    }
    
    async fn run(&self, ctx: &mut LauncherContext) -> Result<(), String> {
        // Set the locale property and store in data map
        ctx.set_locale(self.default_locale.clone());
        log::warn!("I18n feature is disabled, using placeholder implementation");
        Ok(())
    }
} 