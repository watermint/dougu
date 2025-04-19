#[cfg(feature = "i18n")]
use dougu_foundation_i18n::{I18nInitializer, I18nContext, ErrorWithDetails, t, tf};
use crate::{CommandletError, LauncherLayer, LauncherContext};
use async_trait::async_trait;

#[cfg(feature = "i18n")]
// Implement I18nContext for LauncherContext to make it compatible with I18nInitializer
impl I18nContext for LauncherContext {
    fn get_context_data(&self, key: &str) -> Option<&String> {
        if key == "locale" || key == "active_locale" {
            // For locale keys, we'll return the language field directly
            Some(&self.language)
        } else {
            self.get_data(key)
        }
    }
    
    fn set_context_data(&mut self, key: &str, value: String) {
        if key == "locale" || key == "active_locale" {
            // For locale keys, update the language field
            self.set_language(&value);
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
}

#[cfg(feature = "i18n")]
impl I18nInitializerLayer {
    /// Create a new I18nInitializerLayer with the specified default locale
    pub fn new(default_locale: &str) -> Self {
        Self {
            initializer: I18nInitializer::new(default_locale),
        }
    }
    
    /// Create a new I18nInitializerLayer with the option to use filesystem resources
    pub fn with_filesystem(default_locale: &str, use_embedded: bool) -> Self {
        Self {
            initializer: I18nInitializer::with_filesystem(default_locale, use_embedded),
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
        
        // If initialization succeeded, ensure language field matches active_locale
        if result.is_ok() {
            if let Some(active_locale) = ctx.get_data("active_locale") {
                if ctx.get_language() != active_locale {
                    ctx.set_language(active_locale);
                }
            }
        }
        
        result
    }
}

#[cfg(not(feature = "i18n"))]
/// Placeholder implementation when i18n feature is not enabled
pub struct I18nInitializerLayer {
    default_locale: String,
}

#[cfg(not(feature = "i18n"))]
impl I18nInitializerLayer {
    /// Create a new placeholder I18nInitializerLayer
    pub fn new(default_locale: &str) -> Self {
        Self {
            default_locale: default_locale.to_string(),
        }
    }
    
    /// Create a new placeholder I18nInitializerLayer
    pub fn with_filesystem(default_locale: &str, _use_embedded: bool) -> Self {
        Self {
            default_locale: default_locale.to_string(),
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
        // Set both the language property and store in data map
        ctx.set_language(&self.default_locale);
        log::warn!("I18n feature is disabled, using placeholder implementation");
        Ok(())
    }
} 