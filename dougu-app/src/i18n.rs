use dougu_foundation::i18n::{set_locale, Locale};
use dougu_foundation::run::{ActionLauncher, I18nInitializerLayer, LauncherContext};
use std::str::FromStr;

/// Initialize the i18n system with all available translations
pub fn initialize_i18n(default_locale: &str) -> Result<(), String> {
    // Create a launcher with the I18nInitializerLayer
    let mut launcher = ActionLauncher::new();
    
    // Parse locale or use default
    let locale = Locale::from_str(default_locale).unwrap_or_else(|_| Locale::new("en"));
    
    // Add the I18nInitializerLayer with the locale
    launcher.add_layer(I18nInitializerLayer::new(&locale.as_str()));
    
    // Create a context with command name and verbosity
    let mut context = LauncherContext::new("i18n".to_string(), 0);
    context.set_locale(locale.clone());
    
    // Set the locale
    set_locale(locale.as_str());
    
    Ok(())
}

/// Set the application locale
pub fn set_application_locale(locale_str: &str) -> Result<(), String> {
    // Try to parse the locale string
    let locale = Locale::new(locale_str);
    set_locale(locale.as_str())
} 