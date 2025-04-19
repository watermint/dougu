use dougu_essentials_i18n_foundation::{set_locale, I18nInitializerLayer};
use dougu_foundation_run::{CommandLauncher, LauncherContext};

/// Initialize the i18n system with all available translations
/// Using legacy direct approach - prefer using I18nInitializerLayer
pub fn initialize_i18n(default_locale: &str) -> Result<(), String> {
    // Create a launcher with the I18nInitializerLayer
    let mut launcher = CommandLauncher::new();
    launcher.add_layer(I18nInitializerLayer::new(default_locale));
    
    // Create a simple context
    let mut context = LauncherContext::new("I18nInitialization".to_string(), 0);
    
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