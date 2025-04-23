pub mod resources;
pub mod i18n_adapter;
pub mod app_info;

use resources::error_messages;
use resources::log_messages;
use resources::spec_messages;
use log::{debug, info, error};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use crate::ui::{UIManager, OutputFormat};
use std::str::FromStr;
use crate::i18n::{Locale, t};

// Re-export i18n adapter for convenience
pub use i18n_adapter::I18nInitializerLayer;
// Re-export locale from i18n module
pub use crate::i18n::{Locale, LocaleError};

/// Field specification for Action parameters and results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecField {
    pub name: String,
    pub description: Option<String>,
    pub field_type: String,
    pub required: bool,
    pub default_value: Option<String>,
}

/// Error specification for Action errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecError {
    pub code: String,
    pub description: String,
}

/// Specification for an Action's inputs and outputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionSpec {
    pub name: String,
    pub description: Option<String>,
    pub behavior: String,
    pub options: Vec<SpecField>,
    pub parameters: Vec<SpecField>,
    pub result_types: Vec<SpecField>,
    pub errors: Vec<SpecError>,
}

/// Action represents a command implementation that takes serializable parameters and returns serializable results
#[async_trait]
pub trait Action {
    /// The type of parameters this action accepts
    type Params: Serialize + for<'a> Deserialize<'a> + Send + Sync;
    
    /// The type of results this action returns 
    type Results: Serialize + for<'a> Deserialize<'a> + Send + Sync;
    
    /// Returns the name of this action
    fn name(&self) -> &str;
    
    /// Executes the action with the given parameters
    async fn execute(&self, params: Self::Params) -> Result<Self::Results, ActionError>;
    
    /// Generates a specification document for this action
    fn generate_spec(&self) -> ActionSpec {
        // Default implementation provides a basic spec structure
        // Actions should override this to provide detailed specifications
        ActionSpec {
            name: self.name().to_string(),
            description: None,
            behavior: "Not specified".to_string(),
            options: Vec::new(),
            parameters: Vec::new(),
            result_types: Vec::new(),
            errors: Vec::new(),
        }
    }
}

/// Error type for action operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionError {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
}

impl ActionError {
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

impl std::fmt::Display for ActionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)?;
        if let Some(details) = &self.details {
            write!(f, " ({})", details)?;
        }
        Ok(())
    }
}

impl From<String> for ActionError {
    fn from(message: String) -> Self {
        Self {
            code: "UNKNOWN_ERROR".to_string(),
            message,
            details: None,
        }
    }
}

impl From<&str> for ActionError {
    fn from(message: &str) -> Self {
        Self {
            code: "UNKNOWN_ERROR".to_string(),
            message: message.to_string(),
            details: None,
        }
    }
}

// Implement the I18nActionError trait from i18n module
impl crate::i18n::ErrorWithDetails for ActionError {
    fn new_with_i18n(code: &str, key: &str) -> Self {
        let message = crate::i18n::t(key);
        Self::new(code, &message)
    }
    
    fn with_i18n_vars(code: &str, key: &str, vars: &[(&str, &str)]) -> Self {
        let message = crate::i18n::tf(key, vars);
        Self::new(code, &message)
    }
    
    fn with_i18n_details(code: &str, key: &str, details: &str) -> Self {
        let message = crate::i18n::t(key);
        Self::with_details(code, &message, details)
    }
    
    fn with_i18n_vars_details(code: &str, key: &str, vars: &[(&str, &str)], details: &str) -> Self {
        let message = crate::i18n::tf(key, vars);
        Self::with_details(code, &message, details)
    }
}

#[async_trait]
pub trait LauncherLayer {
    fn name(&self) -> &str;
    async fn run(&self, ctx: &mut LauncherContext) -> Result<(), String>;
}

pub struct LauncherContext {
    // Store contextual information for action execution
    pub command_name: String,
    pub verbosity: u8,
    pub locale: Locale, // Use Locale struct instead of raw string
    pub data: std::collections::HashMap<String, String>,
    pub ui: UIManager,
}

impl LauncherContext {
    pub fn new(command_name: String, verbosity: u8) -> Self {
        let locale = Locale::default();
        let mut ctx = Self {
            command_name,
            verbosity,
            locale: locale.clone(),
            data: std::collections::HashMap::new(),
            ui: UIManager::default(),
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
            ui: UIManager::default(),
        };
        // Ensure active_locale is set in the data map for I18nContext implementation
        ctx.set_data("active_locale", locale.as_str().to_string());
        ctx
    }
    
    pub fn with_ui_format(command_name: String, verbosity: u8, locale: Locale, format: OutputFormat) -> Self {
        let mut ctx = Self {
            command_name,
            verbosity,
            locale: locale.clone(),
            data: std::collections::HashMap::new(),
            ui: UIManager::with_format(format),
        };
        // Ensure active_locale is set in the data map for I18nContext implementation
        ctx.set_data("active_locale", locale.as_str().to_string());
        ctx.set_data("output_format", match format {
            OutputFormat::Default => "default",
            OutputFormat::JsonLines => "jsonl",
            OutputFormat::Markdown => "markdown",
        }.to_string());
        ctx
    }

    pub fn set_locale(&mut self, locale: Locale) {
        let locale_str = locale.as_str().to_string();
        self.locale = locale;
        self.set_data("active_locale", locale_str);
    }
    
    pub fn set_output_format(&mut self, format: OutputFormat) {
        self.ui = UIManager::with_format(format);
        self.set_data("output_format", match format {
            OutputFormat::Default => "default",
            OutputFormat::JsonLines => "jsonl",
            OutputFormat::Markdown => "markdown",
        }.to_string());
    }
    
    pub fn set_data(&mut self, key: &str, value: String) {
        self.data.insert(key.to_string(), value);
    }
    
    pub fn get_locale(&self) -> &Locale {
        &self.locale
    }
    
    pub fn get_data(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }
}

impl crate::i18n::I18nContext for LauncherContext {
    fn get_context_data(&self, key: &str) -> Option<&String> {
        self.get_data(key)
    }
    
    fn set_context_data(&mut self, key: &str, value: String) {
        self.set_data(key, value);
    }
}

pub struct ActionLauncher {
    layers: Vec<Box<dyn LauncherLayer>>,
}

impl ActionLauncher {
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
        }
    }
    
    pub fn add_layer<L: LauncherLayer + 'static>(&mut self, layer: L) {
        self.layers.push(Box::new(layer));
    }
    
    pub async fn launch(&self, ctx: &mut LauncherContext) -> Result<(), String> {
        for layer in &self.layers {
            debug!("Running layer: {}", layer.name());
            match layer.run(ctx).await {
                Ok(_) => {
                    debug!("Layer {} completed successfully", layer.name());
                },
                Err(e) => {
                    error!("Layer {} failed: {}", layer.name(), e);
                    return Err(e);
                }
            }
        }
        
        Ok(())
    }
}

pub struct ActionRunner<A: Action> {
    action: A,
    ui: UIManager,
}

impl<A: Action> ActionRunner<A> {
    pub fn new(action: A) -> Self {
        Self {
            action,
            ui: UIManager::default(),
        }
    }
    
    pub fn with_ui(action: A, ui: UIManager) -> Self {
        Self {
            action,
            ui,
        }
    }
    
    pub async fn run(&self, serialized_params: &str) -> Result<String, ActionError> {
        let params = serde_json::from_str::<A::Params>(serialized_params)
            .map_err(|e| ActionError::with_details(
                "PARAM_PARSE_ERROR",
                &error_messages::PARAM_PARSE_ERROR,
                &e.to_string()
            ))?;
        
        let results = self.action.execute(params).await?;
        
        let serialized_results = serde_json::to_string(&results)
            .map_err(|e| ActionError::with_details(
                "RESULT_SERIALIZE_ERROR",
                &error_messages::RESULT_SERIALIZE_ERROR,
                &e.to_string()
            ))?;
        
        Ok(serialized_results)
    }
    
    pub fn format_results(&self, serialized_results: &str) -> Result<(), ActionError> {
        let result_value: serde_json::Value = serde_json::from_str(serialized_results)
            .map_err(|e| ActionError::with_details(
                "RESULT_PARSE_ERROR",
                &error_messages::RESULT_PARSE_ERROR,
                &e.to_string()
            ))?;
        
        // Format and display the results based on UI manager
        let formatted = crate::ui::format_commandlet_result(&self.ui, &result_value);
        println!("{}", formatted);
        
        Ok(())
    }
    
    pub fn format_results_to_string(&self, serialized_results: &str) -> Result<String, ActionError> {
        let result_value: serde_json::Value = serde_json::from_str(serialized_results)
            .map_err(|e| ActionError::with_details(
                "RESULT_PARSE_ERROR",
                &error_messages::RESULT_PARSE_ERROR,
                &e.to_string()
            ))?;
        
        // Format the results based on UI manager
        let formatted = crate::ui::format_commandlet_result(&self.ui, &result_value);
        
        Ok(formatted)
    }
    
    pub fn ui(&self) -> &UIManager {
        &self.ui
    }
    
    pub fn generate_spec(&self) -> ActionSpec {
        self.action.generate_spec()
    }
    
    pub fn get_locale(ctx: &LauncherContext) -> &Locale {
        &ctx.locale
    }
    
    pub fn get_context_locale(params_json: &str) -> Option<Locale> {
        // Parse the parameters as an untyped JSON value 
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(params_json) {
            if let serde_json::Value::Object(map) = value {
                // Check if there's a locale parameter
                if let Some(locale_value) = map.get("locale") {
                    if let serde_json::Value::String(locale_str) = locale_value {
                        // Try to parse the locale
                        if let Ok(locale) = Locale::from_str(locale_str) {
                            return Some(locale);
                        }
                    }
                }
                
                // Check if there's a context parameter with locale
                if let Some(serde_json::Value::Object(ctx_map)) = map.get("context") {
                    if let Some(serde_json::Value::String(locale_str)) = ctx_map.get("locale") {
                        // Try to parse the locale
                        if let Ok(locale) = Locale::from_str(locale_str) {
                            return Some(locale);
                        }
                    }
                }
            }
        }
        
        None
    }
}

pub struct SpecParams {
    /// Name of the action to generate spec for
    pub action_name: Option<String>,
    /// Format of the spec (text, json, markdown)
    pub format: Option<String>,
}

pub struct SpecResults {
    pub action_name: String,
    pub spec: ActionSpec,
    pub formatted_spec: String,
}

pub fn format_spec_as_markdown(spec: &ActionSpec) -> String {
    let mut result = String::new();
    
    // Title
    result.push_str(&format!("# {} Specification\n\n", spec.name));
    
    // Description
    if let Some(desc) = &spec.description {
        result.push_str(&format!("**Description**: {}\n\n", desc));
    }
    
    // Behavior
    result.push_str(&format!("**Behavior**: {}\n\n", spec.behavior));
    
    // Options
    if !spec.options.is_empty() {
        result.push_str("## Options\n\n");
        result.push_str("| Name | Type | Required | Default | Description |\n");
        result.push_str("|------|------|----------|---------|-------------|\n");
        
        for option in &spec.options {
            let desc = option.description.as_deref().unwrap_or("-");
            let default = option.default_value.as_deref().unwrap_or("-");
            let required = if option.required { "Yes" } else { "No" };
            
            result.push_str(&format!("| {} | {} | {} | {} | {} |\n", 
                option.name, option.field_type, required, default, desc));
        }
        
        result.push_str("\n");
    }
    
    // Parameters
    if !spec.parameters.is_empty() {
        result.push_str("## Parameters\n\n");
        result.push_str("| Name | Type | Required | Default | Description |\n");
        result.push_str("|------|------|----------|---------|-------------|\n");
        
        for param in &spec.parameters {
            let desc = param.description.as_deref().unwrap_or("-");
            let default = param.default_value.as_deref().unwrap_or("-");
            let required = if param.required { "Yes" } else { "No" };
            
            result.push_str(&format!("| {} | {} | {} | {} | {} |\n", 
                param.name, param.field_type, required, default, desc));
        }
        
        result.push_str("\n");
    }
    
    // Result Types
    if !spec.result_types.is_empty() {
        result.push_str("## Result Types\n\n");
        result.push_str("| Name | Type | Description |\n");
        result.push_str("|------|------|-------------|\n");
        
        for res in &spec.result_types {
            let desc = res.description.as_deref().unwrap_or("-");
            
            result.push_str(&format!("| {} | {} | {} |\n", 
                res.name, res.field_type, desc));
        }
        
        result.push_str("\n");
    }
    
    // Errors
    if !spec.errors.is_empty() {
        result.push_str("## Errors\n\n");
        result.push_str("| Code | Description |\n");
        result.push_str("|------|-------------|\n");
        
        for err in &spec.errors {
            result.push_str(&format!("| {} | {} |\n", err.code, err.description));
        }
    }
    
    result
}

pub fn format_spec_as_text(spec: &ActionSpec) -> String {
    let mut result = String::new();
    
    // Title
    result.push_str(&format!("{} Specification\n", spec.name));
    result.push_str(&format!("{}\n\n", "=".repeat(spec.name.len() + 14)));
    
    // Description
    if let Some(desc) = &spec.description {
        result.push_str(&format!("Description: {}\n\n", desc));
    }
    
    // Behavior
    result.push_str(&format!("Behavior: {}\n\n", spec.behavior));
    
    // Options
    if !spec.options.is_empty() {
        result.push_str("Options:\n");
        result.push_str("-------\n\n");
        
        for option in &spec.options {
            result.push_str(&format!("* {} ({})\n", option.name, option.field_type));
            
            if let Some(desc) = &option.description {
                result.push_str(&format!("  Description: {}\n", desc));
            }
            
            result.push_str(&format!("  Required: {}\n", if option.required { "Yes" } else { "No" }));
            
            if let Some(default) = &option.default_value {
                result.push_str(&format!("  Default: {}\n", default));
            }
            
            result.push_str("\n");
        }
    }
    
    // Parameters
    if !spec.parameters.is_empty() {
        result.push_str("Parameters:\n");
        result.push_str("-----------\n\n");
        
        for param in &spec.parameters {
            result.push_str(&format!("* {} ({})\n", param.name, param.field_type));
            
            if let Some(desc) = &param.description {
                result.push_str(&format!("  Description: {}\n", desc));
            }
            
            result.push_str(&format!("  Required: {}\n", if param.required { "Yes" } else { "No" }));
            
            if let Some(default) = &param.default_value {
                result.push_str(&format!("  Default: {}\n", default));
            }
            
            result.push_str("\n");
        }
    }
    
    // Result Types
    if !spec.result_types.is_empty() {
        result.push_str("Result Types:\n");
        result.push_str("------------\n\n");
        
        for res in &spec.result_types {
            result.push_str(&format!("* {} ({})\n", res.name, res.field_type));
            
            if let Some(desc) = &res.description {
                result.push_str(&format!("  Description: {}\n", desc));
            }
            
            result.push_str("\n");
        }
    }
    
    // Errors
    if !spec.errors.is_empty() {
        result.push_str("Errors:\n");
        result.push_str("-------\n\n");
        
        for err in &spec.errors {
            result.push_str(&format!("* {} - {}\n", err.code, err.description));
        }
    }
    
    result
}

pub struct SpecAction {
    available_actions: Vec<Box<dyn AnyAction>>,
}

pub trait AnyAction: Send + Sync {
    fn name(&self) -> &str;
    fn generate_spec(&self) -> ActionSpec;
}

impl<T: Action + Send + Sync> AnyAction for T {
    fn name(&self) -> &str {
        self.name()
    }
    
    fn generate_spec(&self) -> ActionSpec {
        self.generate_spec()
    }
}

impl SpecAction {
    pub fn new() -> Self {
        Self {
            available_actions: Vec::new(),
        }
    }
    
    pub fn register_action<A: Action + 'static + Send + Sync>(&mut self, action: A) {
        self.available_actions.push(Box::new(action));
    }
    
    fn find_action(&self, name: &str) -> Option<&Box<dyn AnyAction>> {
        self.available_actions.iter().find(|a| a.name() == name)
    }
    
    fn list_available_actions(&self) -> Vec<String> {
        self.available_actions.iter().map(|a| a.name().to_string()).collect()
    }
}

impl Action for SpecAction {
    type Params = SpecParams;
    type Results = SpecResults;
    
    fn name(&self) -> &str {
        "SpecAction"
    }
    
    async fn execute(&self, params: Self::Params) -> Result<Self::Results, ActionError> {
        if let Some(action_name) = params.action_name {
            // Generate spec for a specific action
            if let Some(action) = self.find_action(&action_name) {
                let spec = action.generate_spec();
                
                // Format the spec based on the requested format
                let formatted_spec = match params.format.as_deref() {
                    Some("json") => serde_json::to_string_pretty(&spec)
                        .map_err(|e| ActionError::with_details(
                            "SPEC_FORMAT_ERROR",
                            &spec_messages::SPEC_FORMAT_ERROR,
                            &e.to_string()
                        ))?,
                    Some("markdown") => format_spec_as_markdown(&spec),
                    _ => format_spec_as_text(&spec), // Default to text format
                };
                
                Ok(SpecResults {
                    action_name: action_name.clone(),
                    spec,
                    formatted_spec,
                })
            } else {
                // Commandlet not found
                Err(ActionError::with_i18n_vars(
                    "COMMANDLET_NOT_FOUND",
                    "errors.commandlet_not_found",
                    &[("name", &action_name)]
                ))
            }
        } else {
            // No specific commandlet requested, return list of available commandlets
            let available = self.list_available_actions();
            let formatted = match params.format.as_deref() {
                Some("json") => serde_json::to_string_pretty(&available)
                    .map_err(|e| ActionError::with_details(
                        "SPEC_FORMAT_ERROR",
                        &spec_messages::SPEC_FORMAT_ERROR,
                        &e.to_string()
                    ))?,
                Some("markdown") => {
                    let mut result = String::from("# Available Actions\n\n");
                    for cmd in &available {
                        result.push_str(&format!("- {}\n", cmd));
                    }
                    result
                },
                _ => {
                    let mut result = String::from("Available Actions:\n");
                    result.push_str(&format!("{}\n\n", "=".repeat(22)));
                    for cmd in &available {
                        result.push_str(&format!("* {}\n", cmd));
                    }
                    result
                }
            };
            
            // Create a placeholder spec for the list
            let spec = ActionSpec {
                name: "Available Actions".to_string(),
                description: Some("List of all available actions".to_string()),
                behavior: "Lists all registered actions".to_string(),
                options: Vec::new(),
                parameters: Vec::new(),
                result_types: Vec::new(),
                errors: Vec::new(),
            };
            
            Ok(SpecResults {
                action_name: "available_actions".to_string(),
                spec,
                formatted_spec: formatted,
            })
        }
    }
    
    fn generate_spec(&self) -> ActionSpec {
        ActionSpec {
            name: self.name().to_string(),
            description: Some(spec_messages::SPEC_DESCRIPTION.to_string()),
            behavior: spec_messages::SPEC_BEHAVIOR.to_string(),
            options: Vec::new(),
            parameters: vec![
                SpecField {
                    name: "action_name".to_string(),
                    description: Some(spec_messages::SPEC_PARAM_NAME_DESC.to_string()),
                    field_type: "string".to_string(),
                    required: false,
                    default_value: None,
                },
                SpecField {
                    name: "format".to_string(),
                    description: Some(spec_messages::SPEC_PARAM_FORMAT_DESC.to_string()),
                    field_type: "string".to_string(),
                    required: false,
                    default_value: Some("text".to_string()),
                },
            ],
            result_types: vec![
                SpecField {
                    name: "action_name".to_string(),
                    description: Some("Name of the action".to_string()),
                    field_type: "string".to_string(),
                    required: true,
                    default_value: None,
                },
                SpecField {
                    name: "spec".to_string(),
                    description: Some("Full specification structure".to_string()),
                    field_type: "ActionSpec".to_string(),
                    required: true,
                    default_value: None,
                },
                SpecField {
                    name: "formatted_spec".to_string(),
                    description: Some("Formatted specification in the requested format".to_string()),
                    field_type: "string".to_string(),
                    required: true,
                    default_value: None,
                },
            ],
            errors: vec![
                SpecError {
                    code: "COMMANDLET_NOT_FOUND".to_string(),
                    description: "The requested action was not found".to_string(),
                },
                SpecError {
                    code: "SPEC_FORMAT_ERROR".to_string(),
                    description: "Error formatting the specification".to_string(),
                },
            ],
        }
    }
}

/// Utility function to throw a new error if a resource is missing
pub fn abort_if_resource_missing(resource: Option<&str>) -> Result<(), String> {
    if resource.is_none() {
        Err(crate::i18n::t("errors.missing_resource"))
    } else {
        Ok(())
    }
} 