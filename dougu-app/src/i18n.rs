use dougu_foundation_i18n::{set_locale, Locale};
use dougu_foundation_run::{CommandLauncher, LauncherContext, I18nInitializerLayer};
use std::str::FromStr;

/// Initialize the i18n system with all available translations
pub fn initialize_i18n(default_locale: &str) -> Result<(), String> {
    // Create a launcher with the I18nInitializerLayer
    let mut launcher = CommandLauncher::new();
    
    // Parse locale or use default
    let locale = Locale::from_str(default_locale).unwrap_or_else(|_| Locale::default());
    
    // Add the I18nInitializerLayer with the locale
    launcher.add_layer(I18nInitializerLayer::with_locale(locale.clone()));
    
    // Create a simple context with the specified locale
    let mut context = LauncherContext::with_locale("I18nInitialization".to_string(), 0, locale);
    
    // Launch the i18n layer
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Failed to build runtime: {}", e))?
        .block_on(async {
            launcher.launch(&mut context).await
        })
}

/// Set the application locale
pub fn set_application_locale(locale_str: &str) -> Result<(), String> {
    // Try to parse the locale string
    match Locale::from_str(locale_str) {
        Ok(locale) => set_locale(locale.language()),
        Err(_) => Err(format!("Invalid locale string: {}", locale_str)),
    }
} 