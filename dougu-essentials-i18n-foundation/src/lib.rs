use dougu_essentials_i18n::integration::{t, tf, I18nCommandletError};
use dougu_foundation_run::{CommandletError, LauncherLayer, LauncherContext};
use async_trait::async_trait;
use std::path::Path;

// New module for embedded resources
pub mod embedded;

// Re-export the integration functions from dougu-essentials-i18n
pub use dougu_essentials_i18n::integration::{init, load_translations, set_locale};
pub use dougu_essentials_i18n::vars;

// Implement the I18nCommandletError trait for CommandletError
impl I18nCommandletError for CommandletError {
    fn with_i18n(code: &str, key: &str) -> Self {
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

/// I18nInitializerLayer implements LauncherLayer to initialize i18n during command execution
pub struct I18nInitializerLayer {
    default_locale: String,
    use_embedded: bool,
}

impl I18nInitializerLayer {
    /// Create a new I18nInitializerLayer with the specified default locale
    pub fn new(default_locale: &str) -> Self {
        Self {
            default_locale: default_locale.to_string(),
            use_embedded: true, // Default to using embedded resources
        }
    }
    
    /// Create a new I18nInitializerLayer with the option to use filesystem resources
    pub fn with_filesystem(default_locale: &str, use_embedded: bool) -> Self {
        Self {
            default_locale: default_locale.to_string(),
            use_embedded,
        }
    }
    
    /// Load translations from filesystem
    fn load_filesystem_translations(&self, locale: &str) -> Result<(), String> {
        // Load foundation translations
        let foundation_path = Path::new("dougu-foundation-run").join("src").join("resources");
        let foundation_file = foundation_path.join(format!("{}.json", locale));
        
        if let Some(file_path) = foundation_file.to_str() {
            if let Err(e) = load_translations(locale, file_path) {
                return Err(format!("Failed to load foundation translations: {}", e));
            }
        } else {
            return Err("Invalid foundation path".to_string());
        }
        
        // Load file command translations
        let file_path = Path::new("dougu-command-file").join("src").join("resources");
        let file_file = file_path.join(format!("{}.json", locale));
        
        if let Some(file_path) = file_file.to_str() {
            if let Err(e) = load_translations(locale, file_path) {
                return Err(format!("Failed to load file command translations: {}", e));
            }
        } else {
            return Err("Invalid file command path".to_string());
        }
        
        Ok(())
    }
    
    /// Load translations from embedded resources
    fn load_embedded_translations(&self, locale: &str) -> Result<(), String> {
        // Load foundation translations from embedded resources
        let foundation_content = embedded::get_resource("foundation", locale)
            .ok_or_else(|| format!("Foundation resource not found for locale: {}", locale))?;
        
        if let Err(e) = dougu_essentials_i18n::integration::load_translations_content(locale, foundation_content) {
            return Err(format!("Failed to load embedded foundation translations: {}", e));
        }
        
        // Load file command translations from embedded resources
        let file_content = embedded::get_resource("file", locale)
            .ok_or_else(|| format!("File command resource not found for locale: {}", locale))?;
        
        if let Err(e) = dougu_essentials_i18n::integration::load_translations_content(locale, file_content) {
            return Err(format!("Failed to load embedded file command translations: {}", e));
        }
        
        Ok(())
    }
}

#[async_trait]
impl LauncherLayer for I18nInitializerLayer {
    fn name(&self) -> &str {
        "I18nInitializerLayer"
    }
    
    async fn run(&self, ctx: &mut LauncherContext) -> Result<(), String> {
        // Extract locale from context if available, otherwise use default
        let locale = ctx.get_data("locale").map(|s| s.as_str()).unwrap_or(&self.default_locale);
        
        // Initialize with locale
        init(locale)?;
        
        // Load translations based on the configuration
        if self.use_embedded {
            // Load from embedded resources
            self.load_embedded_translations(locale)?;
            
            // Try to load other locales as well to support language switching
            for other_locale in embedded::available_locales() {
                if other_locale != locale {
                    if let Err(e) = self.load_embedded_translations(other_locale) {
                        log::warn!("Failed to load embedded translations for {}: {}", other_locale, e);
                    }
                }
            }
        } else {
            // Load from filesystem
            self.load_filesystem_translations(locale)?;
            
            // Define available locales when using filesystem
            let available_locales = ["en", "ja"];
            
            // Load other locales as well to support language switching
            for other_locale in available_locales {
                if other_locale != locale {
                    if let Err(e) = self.load_filesystem_translations(other_locale) {
                        log::warn!("Failed to load filesystem translations for {}: {}", other_locale, e);
                    }
                }
            }
        }
        
        // Set the locale (to ensure it's applied after all translations are loaded)
        set_locale(locale)?;
        
        // Store the active locale in the context for other layers to use
        ctx.set_data("active_locale", locale.to_string());
        
        Ok(())
    }
}

// Re-export the trait for convenience
pub use dougu_essentials_i18n::integration::I18nCommandletError; 