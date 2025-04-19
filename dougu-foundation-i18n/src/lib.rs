// Import only what we need directly
use std::path::Path;

// New module for embedded resources
pub mod embedded;

// Re-export the integration functions from dougu-essentials-i18n
pub use dougu_essentials_i18n::integration::{init, load_translations, set_locale, t, tf};
pub use dougu_essentials_i18n::vars;

// Generic error trait for compatibility with CommandletError without direct dependency
pub trait ErrorWithDetails {
    fn new_with_i18n(code: &str, key: &str) -> Self;
    fn with_i18n_vars(code: &str, key: &str, vars: &[(&str, &str)]) -> Self;
    fn with_i18n_details(code: &str, key: &str, details: &str) -> Self;
    fn with_i18n_vars_details(code: &str, key: &str, vars: &[(&str, &str)], details: &str) -> Self;
}

// Re-export the trait for convenience
pub use dougu_essentials_i18n::integration::I18nCommandletError;

// Generic context trait to avoid direct dependency
pub trait I18nContext {
    fn get_context_data(&self, key: &str) -> Option<&String>;
    fn set_context_data(&mut self, key: &str, value: String);
}

/// I18nInitializer provides i18n initialization functions
/// This implementation doesn't depend on foundation-run
pub struct I18nInitializer {
    default_locale: String,
    use_embedded: bool,
}

impl I18nInitializer {
    /// Create a new I18nInitializer with the specified default locale
    pub fn new(default_locale: &str) -> Self {
        Self {
            default_locale: default_locale.to_string(),
            use_embedded: true, // Default to using embedded resources
        }
    }
    
    /// Create a new I18nInitializer with the option to use filesystem resources
    pub fn with_filesystem(default_locale: &str, use_embedded: bool) -> Self {
        Self {
            default_locale: default_locale.to_string(),
            use_embedded,
        }
    }
    
    /// Initialize i18n with the given context
    pub fn initialize<C: I18nContext>(&self, ctx: &mut C) -> Result<(), String> {
        // Extract locale from context if available, otherwise use default
        let locale = ctx.get_context_data("locale")
                        .map(|s| s.as_str())
                        .unwrap_or(&self.default_locale);
        
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
        ctx.set_context_data("active_locale", locale.to_string());
        
        Ok(())
    }
    
    /// Load translations from filesystem
    fn load_filesystem_translations(&self, locale: &str) -> Result<(), String> {
        // Load foundation translations
        let foundation_path = Path::new("dougu-foundation-run").join("src").join("resources");
        let foundation_file = foundation_path.join(format!("i18n-{}.json", locale));
        
        if let Some(file_path) = foundation_file.to_str() {
            if let Err(e) = load_translations(locale, file_path) {
                return Err(format!("Failed to load foundation translations: {}", e));
            }
        } else {
            return Err("Invalid foundation path".to_string());
        }
        
        // Load file command translations
        let file_path = Path::new("dougu-command-file").join("src").join("resources");
        let file_file = file_path.join(format!("i18n-{}.json", locale));
        
        if let Some(file_path) = file_file.to_str() {
            if let Err(e) = load_translations(locale, file_path) {
                return Err(format!("Failed to load file command translations: {}", e));
            }
        } else {
            return Err("Invalid file command path".to_string());
        }
        
        // Load root command translations
        let root_path = Path::new("dougu-command-root").join("src").join("resources");
        let root_file = root_path.join(format!("i18n-{}.json", locale));
        
        if let Some(file_path) = root_file.to_str() {
            if let Err(e) = load_translations(locale, file_path) {
                return Err(format!("Failed to load root command translations: {}", e));
            }
        } else {
            return Err("Invalid root command path".to_string());
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
        
        // Load root command translations from embedded resources
        let root_content = embedded::get_resource("root", locale)
            .ok_or_else(|| format!("Root command resource not found for locale: {}", locale))?;
        
        if let Err(e) = dougu_essentials_i18n::integration::load_translations_content(locale, root_content) {
            return Err(format!("Failed to load embedded root command translations: {}", e));
        }
        
        Ok(())
    }
}

// Launcher context adapter for dougu-foundation-run (in integration code)
#[cfg(test)]
mod tests {
    use super::*;
    use dougu_foundation_run::LauncherContext;
    
    // Implement I18nContext for LauncherContext (only in tests)
    impl I18nContext for LauncherContext {
        fn get_context_data(&self, key: &str) -> Option<&String> {
            self.get_data(key)
        }
        
        fn set_context_data(&mut self, key: &str, value: String) {
            self.set_data(key, value);
        }
    }
}

// For backward compatibility with code already using this module
#[cfg(test)]
pub use dougu_foundation_run::CommandletError; 