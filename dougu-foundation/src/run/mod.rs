pub mod resources;
pub mod i18n;
pub mod app_info;

use crate::i18n::Locale;
use crate::ui::{OutputFormat, UIManager, format_commandlet_result};
use dougu_essentials::obj::{Notation, NotationType};
use dougu_essentials::obj::notation::JsonNotation;
use log::{debug, error};
use resources::error_messages;
use resources::spec_messages;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use async_trait::async_trait;

pub use crate::i18n::{I18nRunner, t, tf};

/// Field specification for Action parameters and results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecField {
    pub name: String,
    pub description: String,
    pub field_type: String,
    pub required: bool,
    pub default_value: Option<String>,
}

/// Error specification for Action errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecError {
    pub code: String,
    pub message: String,
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
    type Params: Into<NotationType> + From<NotationType> + Send + Sync;
    
    /// The type of results this action returns 
    type Results: Into<NotationType> + From<NotationType> + Send + Sync;
    
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
#[derive(Debug, Clone)]
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

/// A runner for actions that can format results
pub struct ActionRunner<A: Action + 'static> {
    action: Box<A>,
    ui: UIManager,
}

impl<A: Action + 'static> ActionRunner<A> {
    pub fn with_ui(action: A, ui: UIManager) -> Self {
        Self {
            action: Box::new(action),
            ui,
        }
    }
    
    pub fn format_results(&self, serialized_results: &str) -> Result<(), ActionError> {
        let json_notation = JsonNotation::new();
        let result_value = json_notation.decode(serialized_results.as_bytes())
            .map_err(|e| ActionError::with_details(
                "RESULT_PARSE_ERROR",
                &error_messages::RESULT_PARSE_ERROR,
                &e.to_string()
            ))?;
        
        // Format and display the results based on UI manager
        let formatted = format_commandlet_result(&self.ui, &result_value);
        println!("{}", formatted);
        
        Ok(())
    }
    
    pub fn format_results_to_string(&self, serialized_results: &str) -> Result<String, ActionError> {
        let json_notation = JsonNotation::new();
        let result_value = json_notation.decode(serialized_results.as_bytes())
            .map_err(|e| ActionError::with_details(
                "RESULT_PARSE_ERROR",
                &error_messages::RESULT_PARSE_ERROR,
                &e.to_string()
            ))?;
        
        // Format the results based on UI manager
        let formatted = format_commandlet_result(&self.ui, &result_value);
        
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
}

#[derive(Debug, Clone)]
pub struct SpecParams {
    /// Name of the action to generate spec for
    pub action_name: Option<String>,
    /// Format of the spec (text, json, markdown)
    pub format: Option<String>,
}

impl From<NotationType> for SpecParams {
    fn from(value: NotationType) -> Self {
        let mut action_name = None;
        let mut format = None;
        
        if let NotationType::Object(obj) = value {
            if let Some(NotationType::String(name)) = obj.get("action_name") {
                action_name = Some(name.clone());
            }
            if let Some(NotationType::String(fmt)) = obj.get("format") {
                format = Some(fmt.clone());
            }
        }
        
        SpecParams {
            action_name,
            format,
        }
    }
}

impl Into<NotationType> for SpecParams {
    fn into(self) -> NotationType {
        let mut map = HashMap::new();
        if let Some(name) = self.action_name {
            map.insert("action_name".to_string(), NotationType::String(name));
        }
        if let Some(fmt) = self.format {
            map.insert("format".to_string(), NotationType::String(fmt));
        }
        NotationType::Object(map)
    }
}

impl Into<NotationType> for &SpecParams {
    fn into(self) -> NotationType {
        let mut map = HashMap::new();
        if let Some(name) = &self.action_name {
            map.insert("action_name".to_string(), NotationType::String(name.clone()));
        }
        if let Some(fmt) = &self.format {
            map.insert("format".to_string(), NotationType::String(fmt.clone()));
        }
        NotationType::Object(map)
    }
}

#[derive(Debug, Clone)]
pub struct SpecResults {
    pub action_name: String,
    pub spec: ActionSpec,
    pub formatted_spec: String,
}

impl From<NotationType> for SpecResults {
    fn from(value: NotationType) -> Self {
        let mut action_name = String::new();
        let mut formatted_spec = String::new();
        let spec = ActionSpec {
            name: String::new(),
            description: None,
            behavior: "Not specified".to_string(),
            options: Vec::new(),
            parameters: Vec::new(),
            result_types: Vec::new(),
            errors: Vec::new(),
        };
        
        if let NotationType::Object(obj) = value {
            if let Some(NotationType::String(name)) = obj.get("action_name") {
                action_name = name.clone();
            }
            if let Some(NotationType::String(fmt_spec)) = obj.get("formatted_spec") {
                formatted_spec = fmt_spec.clone();
            }
        }
        
        SpecResults {
            action_name,
            spec,
            formatted_spec,
        }
    }
}

impl Into<NotationType> for SpecResults {
    fn into(self) -> NotationType {
        let mut map = HashMap::new();
        map.insert("action_name".to_string(), NotationType::String(self.action_name));
        map.insert("formatted_spec".to_string(), NotationType::String(self.formatted_spec));
        
        // Create a simplified spec object
        let mut spec_map = HashMap::new();
        spec_map.insert("name".to_string(), NotationType::String(self.spec.name));
        if let Some(desc) = self.spec.description {
            spec_map.insert("description".to_string(), NotationType::String(desc));
        }
        spec_map.insert("behavior".to_string(), NotationType::String(self.spec.behavior));
        
        map.insert("spec".to_string(), NotationType::Object(spec_map));
        NotationType::Object(map)
    }
}

impl Into<NotationType> for &SpecResults {
    fn into(self) -> NotationType {
        let mut map = HashMap::new();
        map.insert("action_name".to_string(), NotationType::String(self.action_name.clone()));
        map.insert("formatted_spec".to_string(), NotationType::String(self.formatted_spec.clone()));
        
        // Create a simplified spec object
        let mut spec_map = HashMap::new();
        spec_map.insert("name".to_string(), NotationType::String(self.spec.name.clone()));
        if let Some(desc) = &self.spec.description {
            spec_map.insert("description".to_string(), NotationType::String(desc.clone()));
        }
        spec_map.insert("behavior".to_string(), NotationType::String(self.spec.behavior.clone()));
        
        map.insert("spec".to_string(), NotationType::Object(spec_map));
        NotationType::Object(map)
    }
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
        
        for opt in &spec.options {
            result.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                opt.name,
                opt.field_type,
                opt.required,
                opt.default_value.as_ref().unwrap_or(""),
                opt.description.as_ref().unwrap_or("")
            ));
        }
        result.push_str("\n");
    }
    
    // Parameters
    if !spec.parameters.is_empty() {
        result.push_str("## Parameters\n\n");
        result.push_str("| Name | Type | Required | Default | Description |\n");
        result.push_str("|------|------|----------|---------|-------------|\n");
        
        for param in &spec.parameters {
            result.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                param.name,
                param.field_type,
                param.required,
                param.default_value.as_ref().unwrap_or(""),
                param.description.as_ref().unwrap_or("")
            ));
        }
        result.push_str("\n");
    }
    
    // Result Types
    if !spec.result_types.is_empty() {
        result.push_str("## Result Types\n\n");
        result.push_str("| Name | Type | Required | Default | Description |\n");
        result.push_str("|------|------|----------|---------|-------------|\n");
        
        for result_type in &spec.result_types {
            result.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                result_type.name,
                result_type.field_type,
                result_type.required,
                result_type.default_value.as_ref().unwrap_or(""),
                result_type.description.as_ref().unwrap_or("")
            ));
        }
        result.push_str("\n");
    }
    
    // Errors
    if !spec.errors.is_empty() {
        result.push_str("## Errors\n\n");
        result.push_str("| Code | Description |\n");
        result.push_str("|------|-------------|\n");
        
        for error in &spec.errors {
            result.push_str(&format!(
                "| {} | {} |\n",
                error.code,
                error.message
            ));
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
            result.push_str(&format!("* {} - {}\n", err.code, err.message));
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

#[async_trait]
impl Action for SpecAction {
    type Params = SpecParams;
    type Results = SpecResults;
    
    fn name(&self) -> &str {
        "spec"
    }
    
    async fn execute(&self, params: Self::Params) -> Result<Self::Results, ActionError> {
        let spec = if let Some(action_name) = params.action_name {
            if let Some(action) = self.find_action(&action_name) {
                action.generate_spec()
            } else {
                return Err(ActionError::new(
                    "ACTION_NOT_FOUND",
                    &format!("Action '{}' not found", action_name),
                ));
            }
        } else {
            // Generate a spec for all available actions
            let available = self.list_available_actions();
            let formatted_spec = match params.format.as_deref() {
                Some("text") => format_spec_as_text(&ActionSpec {
                    name: "available_actions".to_string(),
                    description: "List of available actions".to_string(),
                    behavior: "Lists all available actions".to_string(),
                    options: Vec::new(),
                    parameters: Vec::new(),
                    result_types: Vec::new(),
                    errors: Vec::new(),
                }),
                Some("json") => {
                    let mut obj = Vec::new();
                    obj.push(("actions".to_string(), NotationType::Array(available.iter().map(|a| NotationType::String(a.clone())).collect())));
                    let json_notation = JsonNotation::new();
                    json_notation.encode_to_string(&NotationType::Object(obj.into_iter().collect()))
                        .map_err(|e| ActionError::new("SERIALIZATION_ERROR", &format!("Failed to serialize spec: {}", e)))?
                }
                Some("markdown") => format_spec_as_markdown(&ActionSpec {
                    name: "available_actions".to_string(),
                    description: "List of available actions".to_string(),
                    behavior: "Lists all available actions".to_string(),
                    options: Vec::new(),
                    parameters: Vec::new(),
                    result_types: Vec::new(),
                    errors: Vec::new(),
                }),
                _ => format_spec_as_text(&ActionSpec {
                    name: "available_actions".to_string(),
                    description: "List of available actions".to_string(),
                    behavior: "Lists all available actions".to_string(),
                    options: Vec::new(),
                    parameters: Vec::new(),
                    result_types: Vec::new(),
                    errors: Vec::new(),
                }),
            };

            return Ok(SpecResults {
                action_name: "available_actions".to_string(),
                spec: ActionSpec {
                    name: "available_actions".to_string(),
                    description: "List of available actions".to_string(),
                    behavior: "Lists all available actions".to_string(),
                    options: Vec::new(),
                    parameters: Vec::new(),
                    result_types: Vec::new(),
                    errors: Vec::new(),
                },
                formatted_spec,
            });
        };

        let formatted_spec = match params.format.as_deref() {
            Some("text") => format_spec_as_text(&spec),
            Some("json") => {
                let mut obj = Vec::new();
                obj.push(("name".to_string(), NotationType::String(spec.name.clone())));
                if let Some(desc) = &spec.description {
                    obj.push(("description".to_string(), NotationType::String(desc.clone())));
                }
                obj.push(("behavior".to_string(), NotationType::String(spec.behavior.clone())));
                obj.push(("options".to_string(), NotationType::Array(spec.options.iter().map(|o| {
                    let mut opt = Vec::new();
                    opt.push(("name".to_string(), NotationType::String(o.name.clone())));
                    if let Some(desc) = &o.description {
                        opt.push(("description".to_string(), NotationType::String(desc.clone())));
                    }
                    opt.push(("type".to_string(), NotationType::String(o.field_type.clone())));
                    opt.push(("required".to_string(), NotationType::Boolean(o.required)));
                    if let Some(default) = &o.default_value {
                        opt.push(("default".to_string(), NotationType::String(default.clone())));
                    }
                    NotationType::Object(opt.into_iter().collect())
                }).collect())));
                obj.push(("parameters".to_string(), NotationType::Array(spec.parameters.iter().map(|p| {
                    let mut param = Vec::new();
                    param.push(("name".to_string(), NotationType::String(p.name.clone())));
                    if let Some(desc) = &p.description {
                        param.push(("description".to_string(), NotationType::String(desc.clone())));
                    }
                    param.push(("type".to_string(), NotationType::String(p.field_type.clone())));
                    param.push(("required".to_string(), NotationType::Boolean(p.required)));
                    if let Some(default) = &p.default_value {
                        param.push(("default".to_string(), NotationType::String(default.clone())));
                    }
                    NotationType::Object(param.into_iter().collect())
                }).collect())));
                obj.push(("result_types".to_string(), NotationType::Array(spec.result_types.iter().map(|r| {
                    let mut result = Vec::new();
                    result.push(("name".to_string(), NotationType::String(r.name.clone())));
                    if let Some(desc) = &r.description {
                        result.push(("description".to_string(), NotationType::String(desc.clone())));
                    }
                    result.push(("type".to_string(), NotationType::String(r.field_type.clone())));
                    result.push(("required".to_string(), NotationType::Boolean(r.required)));
                    if let Some(default) = &r.default_value {
                        result.push(("default".to_string(), NotationType::String(default.clone())));
                    }
                    NotationType::Object(result.into_iter().collect())
                }).collect())));
                obj.push(("errors".to_string(), NotationType::Array(spec.errors.iter().map(|e| {
                    let mut error = Vec::new();
                    error.push(("code".to_string(), NotationType::String(e.code.clone())));
                    error.push(("description".to_string(), NotationType::String(e.message.clone())));
                    NotationType::Object(error.into_iter().collect())
                }).collect())));
                let json_notation = JsonNotation::new();
                json_notation.encode_to_string(&NotationType::Object(obj.into_iter().collect()))
                    .map_err(|e| ActionError::new("SERIALIZATION_ERROR", &format!("Failed to serialize spec: {}", e)))?
            }
            Some("markdown") => format_spec_as_markdown(&spec),
            _ => format_spec_as_text(&spec),
        };

        Ok(SpecResults {
            action_name: spec.name.clone(),
            spec,
            formatted_spec,
        })
    }
    
    fn generate_spec(&self) -> ActionSpec {
        ActionSpec {
            name: Action::name(self).to_string(),
            description: "Specification for available actions".to_string(),
            behavior: "Lists all available actions".to_string(),
            options: Vec::new(),
            parameters: vec![
                SpecField {
                    name: "action_name".to_string(),
                    description: "Name of the action to generate spec for".to_string(),
                    field_type: "string".to_string(),
                    required: false,
                    default_value: None,
                },
                SpecField {
                    name: "format".to_string(),
                    description: "Format of the spec (text, json, markdown)".to_string(),
                    field_type: "string".to_string(),
                    required: false,
                    default_value: Some("text".to_string()),
                },
            ],
            result_types: vec![
                SpecField {
                    name: "action_name".to_string(),
                    description: "Name of the action".to_string(),
                    field_type: "string".to_string(),
                    required: true,
                    default_value: None,
                },
                SpecField {
                    name: "spec".to_string(),
                    description: "Full specification structure".to_string(),
                    field_type: "object".to_string(),
                    required: true,
                    default_value: None,
                },
                SpecField {
                    name: "formatted_spec".to_string(),
                    description: "Formatted specification in the requested format".to_string(),
                    field_type: "string".to_string(),
                    required: true,
                    default_value: None,
                },
            ],
            errors: vec![
                SpecError {
                    code: "COMMANDLET_NOT_FOUND".to_string(),
                    message: "The requested action was not found".to_string(),
                },
                SpecError {
                    code: "SPEC_FORMAT_ERROR".to_string(),
                    message: "Error formatting the specification".to_string(),
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

/// Specification for a commandlet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandletSpec {
    pub name: String,
    pub description: Option<String>,
    pub behavior: String,
    pub options: Vec<SpecField>,
    pub parameters: Vec<SpecField>,
    pub result_types: Vec<SpecField>,
    pub errors: Vec<SpecError>,
}

impl ActionSpec {
    pub fn format_as_text(&self) -> String {
        let mut result = String::new();
        result.push_str(&format!("Action: {}\n", self.name));
        result.push_str(&format!("Description: {}\n", self.description));
        result.push_str(&format!("Behavior: {}\n\n", self.behavior));

        if !self.options.is_empty() {
            result.push_str("Options:\n");
            for opt in &self.options {
                result.push_str(&format!("* {} ({})\n", opt.name, opt.field_type));
                result.push_str(&format!("  Description: {}\n", opt.description));
                if opt.required {
                    result.push_str("  Required: Yes\n");
                }
                if let Some(default) = &opt.default_value {
                    result.push_str(&format!("  Default: {}\n", default));
                }
            }
            result.push('\n');
        }

        if !self.parameters.is_empty() {
            result.push_str("Parameters:\n");
            for param in &self.parameters {
                result.push_str(&format!("* {} ({})\n", param.name, param.field_type));
                result.push_str(&format!("  Description: {}\n", param.description));
                if param.required {
                    result.push_str("  Required: Yes\n");
                }
                if let Some(default) = &param.default_value {
                    result.push_str(&format!("  Default: {}\n", default));
                }
            }
            result.push('\n');
        }

        if !self.result_types.is_empty() {
            result.push_str("Result Types:\n");
            for res in &self.result_types {
                result.push_str(&format!("* {} ({})\n", res.name, res.field_type));
                result.push_str(&format!("  Description: {}\n", res.description));
                if res.required {
                    result.push_str("  Required: Yes\n");
                }
            }
            result.push('\n');
        }

        if !self.errors.is_empty() {
            result.push_str("Errors:\n");
            for err in &self.errors {
                result.push_str(&format!("* {} - {}\n", err.code, err.message));
            }
        }

        result
    }
} 