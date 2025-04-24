use crate::i18n::{I18nInitializer, Locale};
use crate::run::{LauncherContext, LauncherLayer};
use async_trait::async_trait;
use std::str::FromStr;

/// I18nInitializerLayer implements LauncherLayer to initialize i18n during command execution
pub struct I18nInitializerLayer {
    initializer: I18nInitializer,
    default_locale: Locale,
}

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