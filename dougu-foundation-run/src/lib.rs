pub mod resources;
pub mod i18n_adapter;
pub mod app_info;

use resources::error_messages;
use resources::log_messages;
use resources::spec_messages;
use log::{debug, info, error};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use dougu_foundation_ui::{UIManager, format_commandlet_result, OutputFormat};
use std::str::FromStr;

// Re-export i18n adapter for convenience
pub use i18n_adapter::I18nInitializerLayer;
// Re-export locale from essentials
pub use dougu_essentials_i18n::{Locale, LocaleError};

/// Represents a parameter or result type within a commandlet specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecField {
    pub name: String,
    pub description: Option<String>,
    pub field_type: String,
    pub required: bool,
    pub default_value: Option<String>,
}

/// Represents a possible error that can be returned by a commandlet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecError {
    pub code: String,
    pub description: String,
}

/// Represents the full specification document for a commandlet
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
    
    /// Generates a specification document for this commandlet
    fn generate_spec(&self) -> CommandletSpec {
        // Default implementation provides a basic spec structure
        // Commandlets should override this to provide detailed specifications
        CommandletSpec {
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
    pub fn format_results(&self, serialized_results: &str) -> Result<(), CommandletError> {
        // Parse the serialized JSON to any value
        let parsed_value: serde_json::Value = serde_json::from_str(serialized_results)
            .map_err(|e| CommandletError::with_details(
                "RESULT_PARSE_ERROR", 
                "Failed to parse results for formatting", 
                &e.to_string()
            ))?;
        
        let formatted = format_commandlet_result(&self.ui, &parsed_value);
        self.ui.text(&formatted);
        
        Ok(())
    }
    
    /// Format the serialized results to a string (without displaying)
    #[deprecated(since = "1.1.0", note = "Use format_results instead")]
    pub fn format_results_to_string(&self, serialized_results: &str) -> Result<String, CommandletError> {
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

    /// Generate specification document for this commandlet
    pub fn generate_spec(&self) -> CommandletSpec {
        self.commandlet.generate_spec()
    }

    /// Get the current locale from a context
    pub fn get_locale(ctx: &LauncherContext) -> &Locale {
        ctx.get_locale()
    }

    /// Get the current locale from a context parameter string
    pub fn get_context_locale(params_json: &str) -> Option<Locale> {
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
            }
        }
        None
    }
}

/// Parameters for spec generation commandlet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecParams {
    /// Name of the commandlet to generate spec for
    pub commandlet_name: Option<String>,
    /// Format of the spec (text, json, markdown)
    pub format: Option<String>,
}

/// Result of spec generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecResults {
    pub commandlet_name: String,
    pub spec: CommandletSpec,
    pub formatted_spec: String,
}

/// Formats a CommandletSpec as markdown
pub fn format_spec_as_markdown(spec: &CommandletSpec) -> String {
    let mut result = String::new();
    
    // Title
    result.push_str(&format!("# {} {}\n\n", spec_messages::SPEC_DOCUMENT_TITLE, spec.name));
    
    // Description if available
    if let Some(desc) = &spec.description {
        result.push_str(&format!("{}\n\n", desc));
    }
    
    // Behavior
    result.push_str(&format!("## {}\n\n{}\n\n", spec_messages::SPEC_BEHAVIOR, spec.behavior));
    
    // Parameters
    result.push_str(&format!("## {}\n\n", spec_messages::SPEC_PARAMETERS));
    if spec.parameters.is_empty() {
        result.push_str("No parameters defined.\n\n");
    } else {
        result.push_str(&format!("| {} | {} | {} | {} | {} |\n", 
            spec_messages::SPEC_NAME, 
            spec_messages::SPEC_DESCRIPTION, 
            spec_messages::SPEC_TYPE, 
            spec_messages::SPEC_REQUIRED, 
            spec_messages::SPEC_DEFAULT
        ));
        result.push_str("|-------|-------------|------|----------|-------|\n");
        
        for param in &spec.parameters {
            result.push_str(&format!("| {} | {} | {} | {} | {} |\n",
                param.name,
                param.description.as_ref().unwrap_or(&String::new()),
                param.field_type,
                if param.required { "Yes" } else { "No" },
                param.default_value.as_ref().unwrap_or(&String::new())
            ));
        }
        result.push_str("\n");
    }
    
    // Options
    if !spec.options.is_empty() {
        result.push_str(&format!("## {}\n\n", spec_messages::SPEC_OPTIONS));
        result.push_str(&format!("| {} | {} | {} | {} | {} |\n", 
            spec_messages::SPEC_NAME, 
            spec_messages::SPEC_DESCRIPTION, 
            spec_messages::SPEC_TYPE, 
            spec_messages::SPEC_REQUIRED, 
            spec_messages::SPEC_DEFAULT
        ));
        result.push_str("|-------|-------------|------|----------|-------|\n");
        
        for option in &spec.options {
            result.push_str(&format!("| {} | {} | {} | {} | {} |\n",
                option.name,
                option.description.as_ref().unwrap_or(&String::new()),
                option.field_type,
                if option.required { "Yes" } else { "No" },
                option.default_value.as_ref().unwrap_or(&String::new())
            ));
        }
        result.push_str("\n");
    }
    
    // Result Types
    result.push_str(&format!("## {}\n\n", spec_messages::SPEC_RESULTS));
    if spec.result_types.is_empty() {
        result.push_str("No result types defined.\n\n");
    } else {
        result.push_str(&format!("| {} | {} | {} |\n", 
            spec_messages::SPEC_NAME, 
            spec_messages::SPEC_DESCRIPTION, 
            spec_messages::SPEC_TYPE
        ));
        result.push_str("|-------|-------------|---------|\n");
        
        for result_type in &spec.result_types {
            result.push_str(&format!("| {} | {} | {} |\n",
                result_type.name,
                result_type.description.as_ref().unwrap_or(&String::new()),
                result_type.field_type
            ));
        }
        result.push_str("\n");
    }
    
    // Errors
    result.push_str(&format!("## {}\n\n", spec_messages::SPEC_ERRORS));
    if spec.errors.is_empty() {
        result.push_str("No errors defined.\n\n");
    } else {
        result.push_str(&format!("| {} | {} |\n", "Code", "Description"));
        result.push_str("|------|-------------|\n");
        
        for error in &spec.errors {
            result.push_str(&format!("| {} | {} |\n", error.code, error.description));
        }
    }
    
    result
}

/// Formats a CommandletSpec as plain text
pub fn format_spec_as_text(spec: &CommandletSpec) -> String {
    let mut result = String::new();
    
    // Title
    result.push_str(&format!("{} {}\n", spec_messages::SPEC_DOCUMENT_TITLE, spec.name));
    result.push_str(&format!("{}\n\n", "=".repeat(spec.name.len() + spec_messages::SPEC_DOCUMENT_TITLE.len() + 1)));
    
    // Description if available
    if let Some(desc) = &spec.description {
        result.push_str(&format!("{}\n\n", desc));
    }
    
    // Behavior
    result.push_str(&format!("{}:\n{}\n\n", spec_messages::SPEC_BEHAVIOR, spec.behavior));
    
    // Parameters
    result.push_str(&format!("{}:\n", spec_messages::SPEC_PARAMETERS));
    if spec.parameters.is_empty() {
        result.push_str("No parameters defined.\n\n");
    } else {
        for param in &spec.parameters {
            result.push_str(&format!("- {} ({}): ", param.name, param.field_type));
            if let Some(desc) = &param.description {
                result.push_str(desc);
            }
            if param.required {
                result.push_str(" [Required]");
            }
            if let Some(default) = &param.default_value {
                result.push_str(&format!(" [Default: {}]", default));
            }
            result.push_str("\n");
        }
        result.push_str("\n");
    }
    
    // Options
    if !spec.options.is_empty() {
        result.push_str(&format!("{}:\n", spec_messages::SPEC_OPTIONS));
        for option in &spec.options {
            result.push_str(&format!("- {} ({}): ", option.name, option.field_type));
            if let Some(desc) = &option.description {
                result.push_str(desc);
            }
            if option.required {
                result.push_str(" [Required]");
            }
            if let Some(default) = &option.default_value {
                result.push_str(&format!(" [Default: {}]", default));
            }
            result.push_str("\n");
        }
        result.push_str("\n");
    }
    
    // Result Types
    result.push_str(&format!("{}:\n", spec_messages::SPEC_RESULTS));
    if spec.result_types.is_empty() {
        result.push_str("No result types defined.\n\n");
    } else {
        for result_type in &spec.result_types {
            result.push_str(&format!("- {} ({})", result_type.name, result_type.field_type));
            if let Some(desc) = &result_type.description {
                result.push_str(&format!(": {}", desc));
            }
            result.push_str("\n");
        }
        result.push_str("\n");
    }
    
    // Errors
    result.push_str(&format!("{}:\n", spec_messages::SPEC_ERRORS));
    if spec.errors.is_empty() {
        result.push_str("No errors defined.\n");
    } else {
        for error in &spec.errors {
            result.push_str(&format!("- {} - {}\n", error.code, error.description));
        }
    }
    
    result
}

/// Spec generation commandlet
pub struct SpecCommandlet {
    available_commandlets: Vec<Box<dyn AnyCommandlet>>,
}

/// Trait to allow storing different commandlet types in a collection
pub trait AnyCommandlet: Send + Sync {
    fn name(&self) -> &str;
    fn generate_spec(&self) -> CommandletSpec;
}

impl<T: Commandlet + Send + Sync> AnyCommandlet for T {
    fn name(&self) -> &str {
        self.name()
    }
    
    fn generate_spec(&self) -> CommandletSpec {
        <T as Commandlet>::generate_spec(self)
    }
}

impl SpecCommandlet {
    pub fn new() -> Self {
        Self {
            available_commandlets: Vec::new(),
        }
    }
    
    pub fn register_commandlet<C: Commandlet + 'static + Send + Sync>(&mut self, commandlet: C) {
        self.available_commandlets.push(Box::new(commandlet));
    }
    
    fn find_commandlet(&self, name: &str) -> Option<&Box<dyn AnyCommandlet>> {
        self.available_commandlets.iter().find(|c| c.name() == name)
    }
    
    fn list_available_commandlets(&self) -> Vec<String> {
        self.available_commandlets.iter().map(|c| c.name().to_string()).collect()
    }
}

#[async_trait]
impl Commandlet for SpecCommandlet {
    type Params = SpecParams;
    type Results = SpecResults;
    
    fn name(&self) -> &str {
        "SpecCommandlet"
    }
    
    async fn execute(&self, params: Self::Params) -> Result<Self::Results, CommandletError> {
        let commandlet_name = params.commandlet_name.as_deref();
        let format = params.format.as_deref().unwrap_or("text");
        
        // If no commandlet specified, generate spec for this commandlet
        if commandlet_name.is_none() {
            let spec = Commandlet::generate_spec(self);
            let formatted_spec = match format {
                "json" => serde_json::to_string_pretty(&spec)
                    .map_err(|e| CommandletError::with_details(
                        "SPEC_FORMAT_ERROR", 
                        "Failed to format spec as JSON", 
                        &e.to_string()
                    ))?,
                "markdown" => format_spec_as_markdown(&spec),
                _ => format_spec_as_text(&spec),
            };
            
            return Ok(SpecResults {
                commandlet_name: Commandlet::name(self).to_string(),
                spec,
                formatted_spec,
            });
        }
        
        let commandlet_name = commandlet_name.unwrap();
        
        // Find the requested commandlet
        let commandlet = self.find_commandlet(commandlet_name)
            .ok_or_else(|| {
                let available = self.list_available_commandlets().join(", ");
                CommandletError::with_details(
                    "UNKNOWN_COMMANDLET", 
                    &format!("Unknown commandlet '{}'", commandlet_name),
                    &format!("Available commandlets: {}", available)
                )
            })?;
        
        // Generate and format the spec
        let spec = commandlet.generate_spec();
        let formatted_spec = match format {
            "json" => serde_json::to_string_pretty(&spec)
                .map_err(|e| CommandletError::with_details(
                    "SPEC_FORMAT_ERROR", 
                    "Failed to format spec as JSON", 
                    &e.to_string()
                ))?,
            "markdown" => format_spec_as_markdown(&spec),
            _ => format_spec_as_text(&spec),
        };
        
        Ok(SpecResults {
            commandlet_name: commandlet_name.to_string(),
            spec,
            formatted_spec,
        })
    }
    
    fn generate_spec(&self) -> CommandletSpec {
        CommandletSpec {
            name: Commandlet::name(self).to_string(),
            description: Some("Generates specification documents for available commandlets".to_string()),
            behavior: "Retrieves and formats the specification of a commandlet".to_string(),
            options: Vec::new(),
            parameters: vec![
                SpecField {
                    name: "commandlet_name".to_string(),
                    description: Some("Name of the commandlet to generate specification for".to_string()),
                    field_type: "string".to_string(),
                    required: false,
                    default_value: None,
                },
                SpecField {
                    name: "format".to_string(),
                    description: Some("Format of the generated specification (text, json, markdown)".to_string()),
                    field_type: "string".to_string(),
                    required: false,
                    default_value: Some("text".to_string()),
                },
            ],
            result_types: vec![
                SpecField {
                    name: "commandlet_name".to_string(),
                    description: Some("Name of the commandlet the specification is for".to_string()),
                    field_type: "string".to_string(),
                    required: true,
                    default_value: None,
                },
                SpecField {
                    name: "spec".to_string(),
                    description: Some("Full specification document structure".to_string()),
                    field_type: "CommandletSpec".to_string(),
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
                    code: "UNKNOWN_COMMANDLET".to_string(),
                    description: "The requested commandlet was not found".to_string(),
                },
                SpecError {
                    code: "SPEC_FORMAT_ERROR".to_string(),
                    description: "Failed to format the specification document".to_string(),
                },
            ],
        }
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

// Public re-exports
pub use app_info::display_app_info; 