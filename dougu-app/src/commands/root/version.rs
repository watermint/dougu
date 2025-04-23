use async_trait::async_trait;
use dougu_essentials_build::get_build_info;
use dougu_foundation_i18n::t;
use dougu_foundation_run::{
    Commandlet, 
    CommandletError, 
    CommandRunner,
    LauncherContext, 
    LauncherLayer
};
use dougu_foundation_ui::UIManager;
use serde::{Serialize, Deserialize};
use anyhow::{anyhow, Result};
use serde_json;

// Add the import for message resources
use crate::commands::root::resources::messages::*;

// Version command as a Commandlet
pub struct VersionCommandlet;

#[derive(Serialize, Deserialize)]
pub struct VersionParams {
    // No parameters needed for version command
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

// Display version results directly with UI methods
pub fn display_version_results(ui: &UIManager, results: &VersionResults) -> Result<(), CommandletError> {
    // Display version heading
    ui.heading(1, &t(VERSION_HEADING));
    ui.line_break();
    
    // Create a table with properties and values
    let mut table_data = Vec::<Vec<String>>::new();
    
    // Create all the strings we need
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
    
    // Display the table
    let headers = &[prop_header.as_str(), value_header.as_str()];
    ui.table(headers, &table_data);
    
    Ok(())
}

/// Version command layer for the launcher
pub struct VersionCommandLayer;

#[async_trait]
impl LauncherLayer for VersionCommandLayer {
    fn name(&self) -> &str {
        "VersionCommandLayer"
    }

    async fn run(&self, ctx: &mut LauncherContext) -> Result<(), String> {
        // Create the version commandlet
        let commandlet = VersionCommandlet;
        
        // Create a runner with UI manager
        let runner = CommandRunner::with_ui(commandlet, ctx.ui.clone());
        
        // Execute the commandlet with empty params
        let params = VersionParams {};
        let params_str = serde_json::to_string(&params)
            .map_err(|e| format!("Failed to serialize version params: {}", e))?;
        
        let result = runner.run(&params_str).await
            .map_err(|e| format!("Version command execution failed: {}", e))?;
        
        // Format results
        runner.format_results(&result)
            .map_err(|e| format!("Failed to format version results: {}", e))?;
        
        Ok(())
    }
} 