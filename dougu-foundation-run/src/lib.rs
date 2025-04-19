pub mod resources;
pub mod i18n_adapter;

use resources::error_messages;
use resources::log_messages;
use log::{debug, info, error};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use dougu_foundation_ui::{UIManager, format_commandlet_result};
use std::str::FromStr;

// Re-export i18n adapter for convenience
pub use i18n_adapter::I18nInitializerLayer;
// Re-export locale from essentials
pub use dougu_essentials_i18n::{Locale, LocaleError};

/// Commandlet represents a command implementation that takes serializable parameters and returns serializable results
#[async_trait]
pub trait Commandlet {
    /// The type of parameters this commandlet accepts
    type Params: Serialize + for<'a> Deserialize<'a> + Send + Sync;
    
    /// The type of results this commandlet returns 
    type Results: Serialize + for<'a> Deserialize<'a> + Send + Sync;
    
    /// Returns the name of this commandlet
    fn name(&self) -> &str;
    
    /// Executes the commandlet with the given parameters
    async fn execute(&self, params: Self::Params) -> Result<Self::Results, CommandletError>;
}

/// Error type for commandlet operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandletError {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
}

impl CommandletError {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            details: None,
        }
    }
    
    pub fn with_details(code: &str, message: &str, details: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            details: Some(details.to_string()),
        }
    }
}

impl std::fmt::Display for CommandletError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)?;
        if let Some(details) = &self.details {
            write!(f, " ({})", details)?;
        }
        Ok(())
    }
}

impl From<String> for CommandletError {
    fn from(message: String) -> Self {
        Self {
            code: "UNKNOWN_ERROR".to_string(),
            message,
            details: None,
        }
    }
}

impl From<&str> for CommandletError {
    fn from(message: &str) -> Self {
        Self {
            code: "UNKNOWN_ERROR".to_string(),
            message: message.to_string(),
            details: None,
        }
    }
}

#[async_trait]
pub trait LauncherLayer {
    fn name(&self) -> &str;
    async fn run(&self, ctx: &mut LauncherContext) -> Result<(), String>;
}

pub struct LauncherContext {
    // Store contextual information for command execution
    pub command_name: String,
    pub verbosity: u8,
    pub locale: Locale, // Use Locale struct instead of raw string
    pub data: std::collections::HashMap<String, String>,
}

impl LauncherContext {
    pub fn new(command_name: String, verbosity: u8) -> Self {
        let locale = Locale::default();
        let mut ctx = Self {
            command_name,
            verbosity,
            locale: locale.clone(),
            data: std::collections::HashMap::new(),
        };
        // Ensure active_locale is set in the data map for I18nContext implementation
        ctx.set_data("active_locale", locale.as_str().to_string());
        ctx
    }

    pub fn with_locale(command_name: String, verbosity: u8, locale: Locale) -> Self {
        let mut ctx = Self {
            command_name,
            verbosity,
            locale: locale.clone(),
            data: std::collections::HashMap::new(),
        };
        // Ensure active_locale is set in the data map for I18nContext implementation
        ctx.set_data("active_locale", locale.as_str().to_string());
        ctx
    }

    pub fn with_language(command_name: String, verbosity: u8, language: &str) -> Self {
        // For backward compatibility
        let locale = Locale::from_str(language).unwrap_or_else(|_| Locale::default());
        Self::with_locale(command_name, verbosity, locale)
    }

    pub fn set_locale(&mut self, locale: Locale) {
        // Clone locale before moving it to self.locale
        let locale_str = locale.as_str().to_string();
        self.locale = locale;
        // Also update in data map for backwards compatibility
        self.set_data("active_locale", locale_str);
    }

    pub fn set_language(&mut self, language: &str) {
        // For backward compatibility
        if let Ok(locale) = Locale::from_str(language) {
            self.set_locale(locale);
        }
    }

    pub fn get_locale(&self) -> &Locale {
        &self.locale
    }

    pub fn get_language(&self) -> &str {
        self.locale.language()
    }

    pub fn set_data(&mut self, key: &str, value: String) {
        self.data.insert(key.to_string(), value);
    }

    pub fn get_data(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }
}

pub struct CommandLauncher {
    layers: Vec<Box<dyn LauncherLayer>>,
}

impl CommandLauncher {
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    pub fn add_layer<L: LauncherLayer + 'static>(&mut self, layer: L) {
        self.layers.push(Box::new(layer));
    }

    pub async fn launch(&self, ctx: &mut LauncherContext) -> Result<(), String> {
        info!("{}", log_messages::LAUNCHER_START);
        
        for layer in &self.layers {
            debug!("{}", format!("{}", log_messages::LAYER_EXECUTION).replace("{}", layer.name()));
            layer.run(ctx).await?;
        }
        
        info!("{}", log_messages::LAUNCHER_COMPLETE);
        Ok(())
    }
}

/// CommandRunner handles parsing command line arguments into params and formatting results
pub struct CommandRunner<C: Commandlet> {
    commandlet: C,
    ui: UIManager,
}

impl<C: Commandlet> CommandRunner<C> {
    pub fn new(commandlet: C) -> Self {
        Self { 
            commandlet,
            ui: UIManager::default(),
        }
    }
    
    pub fn with_ui(commandlet: C, ui: UIManager) -> Self {
        Self { 
            commandlet,
            ui,
        }
    }
    
    /// Run the commandlet with the given serialized parameters
    pub async fn run(&self, serialized_params: &str) -> Result<String, CommandletError> {
        // Deserialize the parameters
        let params: C::Params = serde_json::from_str(serialized_params)
            .map_err(|e| CommandletError::with_details(
                "PARAM_PARSE_ERROR", 
                "Failed to parse parameters", 
                &e.to_string()
            ))?;
        
        // Execute the commandlet
        let results = self.commandlet.execute(params).await?;
        
        // Serialize the results
        let serialized_results = serde_json::to_string(&results)
            .map_err(|e| CommandletError::with_details(
                "RESULT_SERIALIZE_ERROR", 
                "Failed to serialize results", 
                &e.to_string()
            ))?;
        
        Ok(serialized_results)
    }
    
    /// Format the serialized results for display
    pub fn format_results(&self, serialized_results: &str) -> Result<String, CommandletError> {
        // Parse the serialized JSON to any value
        let parsed_value: serde_json::Value = serde_json::from_str(serialized_results)
            .map_err(|e| CommandletError::with_details(
                "RESULT_PARSE_ERROR", 
                "Failed to parse results for formatting", 
                &e.to_string()
            ))?;
        
        Ok(format_commandlet_result(&self.ui, &parsed_value))
    }
    
    /// Get UI manager reference
    pub fn ui(&self) -> &UIManager {
        &self.ui
    }

    /// Get the current locale from a context
    pub fn get_locale(ctx: &LauncherContext) -> &Locale {
        ctx.get_locale()
    }

    /// Get the current language from a context
    pub fn get_language(ctx: &LauncherContext) -> &str {
        ctx.get_language()
    }

    /// Get the current locale from a context parameter string
    pub fn get_context_locale(params_json: &str) -> Option<Locale> {
        // Attempt to parse the context portion of the parameters
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(params_json) {
            if let Some(obj) = value.as_object() {
                // Check for "context" object with "locale" field
                if let Some(context) = obj.get("context") {
                    if let Some(context_obj) = context.as_object() {
                        if let Some(locale) = context_obj.get("locale") {
                            if let Some(locale_str) = locale.as_str() {
                                if let Ok(locale) = Locale::from_str(locale_str) {
                                    return Some(locale);
                                }
                            }
                        }
                        // Also check for "language" field for backward compatibility
                        if let Some(language) = context_obj.get("language") {
                            if let Some(language_str) = language.as_str() {
                                if let Ok(locale) = Locale::from_str(language_str) {
                                    return Some(locale);
                                }
                            }
                        }
                    }
                }
                
                // Also check for "locale" field directly
                if let Some(locale) = obj.get("locale") {
                    if let Some(locale_str) = locale.as_str() {
                        if let Ok(locale) = Locale::from_str(locale_str) {
                            return Some(locale);
                        }
                    }
                }
                
                // Also check for "language" field directly for backward compatibility
                if let Some(language) = obj.get("language") {
                    if let Some(language_str) = language.as_str() {
                        if let Ok(locale) = Locale::from_str(language_str) {
                            return Some(locale);
                        }
                    }
                }
            }
        }
        
        None
    }

    /// Get the current language from a context parameter string (for backward compatibility)
    pub fn get_context_language(params_json: &str) -> Option<String> {
        Self::get_context_locale(params_json).map(|locale| locale.language().to_string())
    }
}

// Example error abort if resource not found
pub fn abort_if_resource_missing(resource: Option<&str>) -> Result<(), String> {
    if resource.is_none() {
        error!("{}", error_messages::RESOURCE_NOT_FOUND);
        return Err(error_messages::RESOURCE_NOT_FOUND.to_string());
    }
    Ok(())
} 