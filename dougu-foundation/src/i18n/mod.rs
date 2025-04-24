// Import only what we need directly
use std::path::Path;

// New module for embedded resources
pub mod embedded;

// Re-export the integration functions from dougu-essentials
pub use dougu_essentials::i18n::integration::{init, init_with_locale, load_translations, set_locale, set_locale_object, t, tf};
pub use dougu_essentials::i18n::locale::is_supported_language;
pub use dougu_essentials::i18n::{Locale, LocaleError};
pub use dougu_essentials::vars;

// Generic error trait for compatibility with CommandletError without direct dependency
pub trait ErrorWithDetails {
    fn new_with_i18n(code: &str, key: &str) -> Self;
    fn with_i18n_vars(code: &str, key: &str, vars: &[(&str, &str)]) -> Self;
    fn with_i18n_details(code: &str, key: &str, details: &str) -> Self;
    fn with_i18n_vars_details(code: &str, key: &str, vars: &[(&str, &str)], details: &str) -> Self;
}

// Re-export the trait for convenience
pub use dougu_essentials::i18n::integration::I18nCommandletError;

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
        let foundation_path = Path::new("foundation").join("src").join("resources");
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
    fn load_embedded_translations(&self, locale_str: &str) -> Result<(), String> {
        // Parse the requested locale using Locale's constructors
        let locale_parts: Vec<&str> = locale_str.split(|c| c == '-' || c == '_').collect();
        
        let locale_obj = match locale_parts.len() {
            1 => Locale::new(locale_parts[0]),
            2 => Locale::with_region(locale_parts[0], locale_parts[1]),
            3 => Locale::with_script_region(locale_parts[0], locale_parts[1], locale_parts[2]),
            _ => {
                log::warn!("Invalid locale format: '{}'", locale_str);
                return Err(format!("Invalid locale format: {}", locale_str));
            }
        };
        
        // Get the base language for fallback
        let base_language = locale_obj.language();
        
        // Foundation resources
        self.load_module_resource("foundation", locale_str, base_language)?;
        
        // File command resources (non-critical, so we don't propagate errors)
        if let Err(e) = self.load_module_resource("file", locale_str, base_language) {
            log::warn!("Failed to load file resources: {}", e);
        }
        
        // Root command resources (non-critical, so we don't propagate errors)
        if let Err(e) = self.load_module_resource("root", locale_str, base_language) {
            log::warn!("Failed to load root resources: {}", e);
        }
        
        Ok(())
    }
    
    /// Helper to load a specific module resource with proper fallback
    fn load_module_resource(&self, module: &str, locale_str: &str, base_language: &str) -> Result<(), String> {
        // Try with exact locale first
        if let Some(content) = embedded::get_resource(module, locale_str) {
            if let Err(e) = dougu_essentials::i18n::integration::load_translations_content(locale_str, content) {
                log::warn!("Failed to load {} translations for {}: {}", module, locale_str, e);
                // Continue with fallbacks even if this fails
            } else {
                return Ok(());
            }
        }
        
        // If exact locale failed, try with base language
        if locale_str != base_language {
            if let Some(content) = embedded::get_resource(module, base_language) {
                if let Err(e) = dougu_essentials::i18n::integration::load_translations_content(locale_str, content) {
                    log::warn!("Failed to load {} translations for base language {}: {}", 
                               module, base_language, e);
                    // Continue with English fallback
                } else {
                    return Ok(());
                }
            }
        }
        
        // If all else fails, try English as final fallback
        if base_language != "en" {
            if let Some(content) = embedded::get_resource(module, "en") {
                if let Err(e) = dougu_essentials::i18n::integration::load_translations_content(locale_str, content) {
                    log::warn!("Failed to load {} translations for fallback locale en: {}", 
                               module, e);
                    return Err(format!("Failed to load {} translations for any locale", module));
                } else {
                    return Ok(());
                }
            }
        }
        
        Err(format!("No translations found for module: {}", module))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct LauncherContext {
        data: std::collections::HashMap<String, String>,
    }
    
    impl I18nContext for LauncherContext {
        fn get_context_data(&self, key: &str) -> Option<&String> {
            self.data.get(key)
        }
        
        fn set_context_data(&mut self, key: &str, value: String) {
            self.data.insert(key.to_string(), value);
        }
    }
} 