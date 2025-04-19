use crate::I18n;
use crate::locale::{Locale, is_supported_language};
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use std::str::FromStr;

// Global i18n instance
static I18N: Lazy<Arc<Mutex<Option<I18n>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

/// Initialize the global i18n instance with a default locale
pub fn init(default_locale: &str) -> Result<(), String> {
    let mut lock = I18N.lock().map_err(|e| format!("Failed to lock i18n instance: {}", e))?;
    *lock = Some(I18n::new(default_locale));
    Ok(())
}

/// Initialize the global i18n instance with a Locale object
pub fn init_with_locale(locale: &Locale) -> Result<(), String> {
    let mut lock = I18N.lock().map_err(|e| format!("Failed to lock i18n instance: {}", e))?;
    *lock = Some(I18n::new(locale.language()));
    Ok(())
}

/// Load a translation file for a locale
pub fn load_translations(locale: &str, path: &str) -> Result<(), String> {
    let mut lock = I18N.lock().map_err(|e| format!("Failed to lock i18n instance: {}", e))?;
    
    if let Some(i18n) = lock.as_mut() {
        i18n.load_advanced_file(locale, path)
            .map_err(|e| format!("Failed to load translations: {}", e))?;
    } else {
        return Err("i18n not initialized".to_string());
    }
    
    Ok(())
}

/// Load a translation file for a locale object
pub fn load_translations_for_locale(locale: &Locale, path: &str) -> Result<(), String> {
    load_translations(locale.language(), path)
}

/// Load translations from string content instead of a file
/// This allows embedding translations in the binary
pub fn load_translations_content(locale: &str, content: &str) -> Result<(), String> {
    let mut lock = I18N.lock().map_err(|e| format!("Failed to lock i18n instance: {}", e))?;
    
    if let Some(i18n) = lock.as_mut() {
        i18n.load_content(locale, content)
            .map_err(|e| format!("Failed to load translations from content: {}", e))?;
    } else {
        return Err("i18n not initialized".to_string());
    }
    
    Ok(())
}

/// Load translations from string content for a locale object
pub fn load_translations_content_for_locale(locale: &Locale, content: &str) -> Result<(), String> {
    load_translations_content(locale.language(), content)
}

/// Set the current locale
pub fn set_locale(locale: &str) -> Result<(), String> {
    let mut lock = I18N.lock().map_err(|e| format!("Failed to lock i18n instance: {}", e))?;
    
    if let Some(i18n) = lock.as_mut() {
        i18n.set_locale(locale)
            .map_err(|e| format!("Failed to set locale: {}", e))?;
    } else {
        return Err("i18n not initialized".to_string());
    }
    
    Ok(())
}

/// Set the current locale using a Locale object
pub fn set_locale_object(locale: &Locale) -> Result<(), String> {
    set_locale(locale.language())
}

/// Check if a locale is supported by the application
pub fn is_locale_supported(locale_str: &str) -> bool {
    if let Ok(locale) = Locale::from_str(locale_str) {
        is_supported_language(&locale)
    } else {
        false
    }
}

/// Get translation for key
pub fn t(key: &str) -> String {
    if let Ok(lock) = I18N.lock() {
        if let Some(i18n) = lock.as_ref() {
            return i18n.t(key);
        }
    }
    
    // Return the key itself as fallback
    key.to_string()
}

/// Get message with variable interpolation
pub fn tf(key: &str, vars: &[(&str, &str)]) -> String {
    if let Ok(lock) = I18N.lock() {
        if let Some(i18n) = lock.as_ref() {
            let var_map: std::collections::HashMap<&str, &str> = vars.iter().copied().collect();
            return i18n.tf(key, &var_map);
        }
    }
    
    // Simple fallback with variable replacement
    let mut result = key.to_string();
    for (k, v) in vars {
        result = result.replace(&format!("{{{}}}", k), v);
    }
    result
}

/// Convenience macro to create key-value pairs for variable interpolation
#[macro_export]
macro_rules! vars {
    ($($key:expr => $value:expr),* $(,)?) => {
        &[$(($key, $value),)*]
    };
}

// Extension trait for CommandletError
pub trait I18nCommandletError {
    fn with_i18n(code: &str, key: &str) -> Self;
    fn with_i18n_vars(code: &str, key: &str, vars: &[(&str, &str)]) -> Self;
    fn with_i18n_details(code: &str, key: &str, details: &str) -> Self;
    fn with_i18n_vars_details(code: &str, key: &str, vars: &[(&str, &str)], details: &str) -> Self;
} 