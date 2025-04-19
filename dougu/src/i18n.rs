use dougu_foundation_i18n::set_locale;
use dougu_foundation_run::{CommandLauncher, LauncherContext, I18nInitializerLayer};

/// Initialize the i18n system with all available translations
/// Using legacy direct approach - prefer using I18nInitializerLayer
pub fn initialize_i18n(default_locale: &str) -> Result<(), String> {
    // Create a launcher with the I18nInitializerLayer
    let mut launcher = CommandLauncher::new();
    launcher.add_layer(I18nInitializerLayer::new(default_locale));
    
    // Create a simple context with the specified language
    let mut context = LauncherContext::with_language("I18nInitialization".to_string(), 0, default_locale);
    
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
pub fn set_application_locale(locale: &str) -> Result<(), String> {
    set_locale(locale)
} 