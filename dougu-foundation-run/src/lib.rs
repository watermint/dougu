pub mod resources;
pub mod i18n_adapter;

use resources::error_messages;
use resources::log_messages;
use log::{debug, info, error};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use dougu_foundation_ui::{UIManager, format_commandlet_result};

// Re-export i18n adapter for convenience
pub use i18n_adapter::I18nInitializerLayer;

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
    pub language: String, // Store current language/locale
    pub data: std::collections::HashMap<String, String>,
}

impl LauncherContext {
    pub fn new(command_name: String, verbosity: u8) -> Self {
        Self {
            command_name,
            verbosity,
            language: "en".to_string(), // Default to English
            data: std::collections::HashMap::new(),
        }
    }

    pub fn with_language(command_name: String, verbosity: u8, language: &str) -> Self {
        Self {
            command_name,
            verbosity,
            language: language.to_string(),
            data: std::collections::HashMap::new(),
        }
    }

    pub fn set_language(&mut self, language: &str) {
        self.language = language.to_string();
        // Also update in data map for backwards compatibility
        self.set_data("active_locale", language.to_string());
    }

    pub fn get_language(&self) -> &str {
        &self.language
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

    /// Get the current language from a context
    pub fn get_language(ctx: &LauncherContext) -> &str {
        ctx.get_language()
    }

    /// Get the current language from a context parameter string
    pub fn get_context_language(params_json: &str) -> Option<String> {
        // Attempt to parse the context portion of the parameters
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(params_json) {
            if let Some(obj) = value.as_object() {
                // Check for "context" object with "language" field
                if let Some(context) = obj.get("context") {
                    if let Some(context_obj) = context.as_object() {
                        if let Some(language) = context_obj.get("language") {
                            if let Some(language_str) = language.as_str() {
                                return Some(language_str.to_string());
                            }
                        }
                    }
                }
                
                // Also check for "language" field directly
                if let Some(language) = obj.get("language") {
                    if let Some(language_str) = language.as_str() {
                        return Some(language_str.to_string());
                    }
                }
            }
        }
        
        None
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