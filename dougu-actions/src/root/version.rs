use anyhow::Result;
use async_trait::async_trait;
use dougu_essentials::build::get_build_info;
use dougu_foundation::{
    i18n::t,
    run::{Action, ActionError, ActionRunner, LauncherContext, LauncherLayer},
    ui::UIManager
};

// Add the import for message resources
use crate::root::resources::messages::*;

// Version action
pub struct VersionAction;

pub struct VersionParams {
    // No parameters needed for version action
}

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
impl Action for VersionAction {
    type Params = VersionParams;
    type Results = VersionResults;
    
    fn name(&self) -> &str {
        "VersionAction"
    }
    
    async fn execute(&self, _params: &Self::Params, _ctx: &LauncherContext) -> Result<Self::Results, ActionError> {
        let build_info = get_build_info();
        
        Ok(VersionResults {
            build_release: build_info.build_release,
            build_timestamp: build_info.build_timestamp,
            build_type: build_info.build_type,
            repository_name: build_info.repository_name,
            repository_owner: build_info.repository_owner,
            rust_version: build_info.rust_version,
            version: build_info.version,
            executable_name: build_info.executable_name,
        })
    }
}

// Display version results directly with UI methods
pub fn display_version_results(ui: &UIManager, results: &VersionResults) -> Result<(), ActionError> {
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

/// Version action layer for the launcher
pub struct VersionActionLayer;

#[async_trait]
impl LauncherLayer for VersionActionLayer {
    fn name(&self) -> &str {
        "VersionActionLayer"
    }

    async fn run(&self, ctx: &mut LauncherContext) -> Result<(), String> {
        // Create the version action
        let action = VersionAction;
        
        // Create a runner with UI manager
        let runner = ActionRunner::with_ui(action, ctx.ui.clone());
        
        // Execute the action with empty params
        let params = VersionParams {};
        let params_str = NotationType::Json.encode_to_string(&params)
            .map_err(|e| format!("Failed to serialize params: {}", e))?;
        
        let result = runner.run(&params_str).await
            .map_err(|e| format!("Version action execution failed: {}", e))?;
        
        // Format results
        runner.format_results(&result)
            .map_err(|e| format!("Failed to format version results: {}", e))?;
        
        Ok(())
    }
} 