#[cfg(feature = "i18n")]
use dougu_foundation_i18n::{I18nInitializer, I18nContext, ErrorWithDetails, t, tf};
use crate::{CommandletError, LauncherLayer, LauncherContext};
use async_trait::async_trait;

#[cfg(feature = "i18n")]
// Implement I18nContext for LauncherContext to make it compatible with I18nInitializer
impl I18nContext for LauncherContext {
    fn get_context_data(&self, key: &str) -> Option<&String> {
        self.get_data(key)
    }
    
    fn set_context_data(&mut self, key: &str, value: String) {
        self.set_data(key, value);
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
        // Use the initializer to set up i18n
        self.initializer.initialize(ctx)
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
        // Just store the locale in context
        ctx.set_data("active_locale", self.default_locale.clone());
        log::warn!("I18n feature is disabled, using placeholder implementation");
        Ok(())
    }
} 