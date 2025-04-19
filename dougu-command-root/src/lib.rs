use async_trait::async_trait;
use dougu_foundation_run::{Commandlet, CommandletError, CommandRunner, LauncherContext, LauncherLayer};
use dougu_foundation_ui::UIManager;
use dougu_foundation_i18n::t;
use serde::{Deserialize, Serialize};
use dougu_essentials_build::{BuildInfo, get_build_info};

mod resources;
use resources::messages::*;

// Version command as a Commandlet
pub struct VersionCommandlet;

#[derive(Serialize, Deserialize)]
pub struct VersionParams {
    // Empty parameters for version command
}

#[derive(Serialize, Deserialize)]
pub struct VersionResults {
    pub build_release: u32,
    pub build_timestamp: String,
    pub build_type: String,
    pub repository_name: String,
    pub repository_owner: String,
    pub rust_version: String,
    pub version: String,
    pub executable_name: String,
}

#[async_trait]
impl Commandlet for VersionCommandlet {
    type Params = VersionParams;
    type Results = VersionResults;
    
    fn name(&self) -> &str {
        "VersionCommandlet"
    }
    
    async fn execute(&self, _params: Self::Params) -> Result<Self::Results, CommandletError> {
        // Get build information
        let build_info = get_build_info();
        
        // Generate semantic version from build info instead of using CARGO_PKG_VERSION
        let version = build_info.semantic_version();
        
        // Extract fields from build_info
        let build_timestamp = build_info.build_timestamp.clone();
        let build_type = build_info.build_type.clone();
        let build_release = build_info.build_release;
        let repository_owner = build_info.repository_owner.clone();
        let repository_name = build_info.repository_name.clone();
        let executable_name = build_info.executable_name.clone();
        
        Ok(VersionResults {
            version,
            build_type,
            build_release,
            rust_version: std::env::var("RUSTC_VERSION").unwrap_or_else(|_| "unknown".to_string()),
            build_timestamp,
            repository_owner,
            repository_name,
            executable_name,
        })
    }
}

// Custom formatter for version results
pub fn format_version_results(ui: &UIManager, results: &VersionResults) -> Result<String, CommandletError> {
    // Create a formatted version output
    let mut output = ui.heading(1, &t(VERSION_HEADING));
    output.push_str("\n\n");
    
    // Create a table with properties and values
    let mut table_data = Vec::<Vec<String>>::new();
    
    // Create all the strings we need to avoid temporary value issues
    let prop_build_release = t(VERSION_PROPERTY_BUILD_RELEASE);
    let prop_build_timestamp = t(VERSION_PROPERTY_BUILD_TIMESTAMP);
    let prop_build_type = t(VERSION_PROPERTY_BUILD_TYPE);
    let prop_repository_name = "Repository Name".to_string();
    let prop_repository_owner = "Repository Owner".to_string();
    let prop_rust_version = t(VERSION_PROPERTY_RUST_VERSION);
    let prop_version = t(VERSION_PROPERTY_VERSION);
    let prop_executable_name = t(VERSION_PROPERTY_EXECUTABLE_NAME);
    let prop_header = t(VERSION_PROPERTY);
    let value_header = t(VERSION_VALUE);
    
    // Add table data in alphabetical order by property
    table_data.push(vec![
        prop_build_release.clone(),
        results.build_release.to_string()
    ]);
    
    table_data.push(vec![
        prop_build_timestamp.clone(),
        results.build_timestamp.clone()
    ]);
    
    table_data.push(vec![
        prop_build_type.clone(),
        results.build_type.clone()
    ]);
    
    table_data.push(vec![
        prop_executable_name.clone(),
        results.executable_name.clone()
    ]);
    
    table_data.push(vec![
        prop_repository_name.clone(),
        results.repository_name.clone()
    ]);
    
    table_data.push(vec![
        prop_repository_owner.clone(),
        results.repository_owner.clone()
    ]);
    
    table_data.push(vec![
        prop_rust_version.clone(),
        results.rust_version.clone()
    ]);
    
    table_data.push(vec![
        prop_version.clone(),
        results.version.clone()
    ]);
    
    // Convert table data to the format expected by ui.table
    let str_table_data: Vec<Vec<&str>> = table_data.iter()
        .map(|row| row.iter().map(|s| s.as_str()).collect())
        .collect();
    
    // Add the table to the output
    let headers = &[prop_header.as_str(), value_header.as_str()];
    output.push_str(&ui.table(headers, &str_table_data));
    
    Ok(output)
}

// Version command layer for the launcher
pub struct VersionCommandLayer;

#[async_trait]
impl LauncherLayer for VersionCommandLayer {
    fn name(&self) -> &str {
        "VersionCommandLayer"
    }

    async fn run(&self, ctx: &mut LauncherContext) -> Result<(), String> {
        // Create the commandlet and runner with UI formatting
        let commandlet = VersionCommandlet;
        // Use UI from context instead of creating a new one
        let runner = CommandRunner::with_ui(commandlet, ctx.ui.clone());
        
        // Create empty parameters
        let params = VersionParams {};
        let serialized_params = serde_json::to_string(&params)
            .map_err(|e| format!("{}: {}", t(VERSION_ERROR_SERIALIZE), e))?;
        
        // Run the commandlet and get the serialized result
        let command_result = runner.run(&serialized_params).await
            .map_err(|e| format!("{}: {}", t(VERSION_ERROR_EXECUTION), e))?;
        
        // Format the result based on its content
        let formatted_result = if command_result.starts_with('{') {
            // Try to parse as VersionResults
            match serde_json::from_str::<VersionResults>(&command_result) {
                Ok(results) => format_version_results(&ctx.ui, &results)
                    .map_err(|e| format!("{}: {}", t(VERSION_ERROR_FORMAT), e))?,
                Err(e) => format!("Error parsing result: {}", e),
            }
        } else {
            // It's already an error message
            command_result
        };
        
        println!("{}", formatted_result);
        
        Ok(())
    }
} 