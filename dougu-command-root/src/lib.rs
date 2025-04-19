use async_trait::async_trait;
use dougu_foundation_run::{Commandlet, CommandletError, CommandRunner, LauncherContext, LauncherLayer};
use dougu_foundation_ui::UIManager;
use dougu_foundation_i18n::t;
use serde::{Deserialize, Serialize};

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
    pub version: String,
    pub rust_version: String,
    pub target: String,
    pub profile: String,
    pub timestamp: String,
}

#[async_trait]
impl Commandlet for VersionCommandlet {
    type Params = VersionParams;
    type Results = VersionResults;
    
    fn name(&self) -> &str {
        "VersionCommandlet"
    }
    
    async fn execute(&self, _params: Self::Params) -> Result<Self::Results, CommandletError> {
        Ok(VersionResults {
            version: env!("CARGO_PKG_VERSION").to_string(),
            rust_version: std::env::var("RUSTC_VERSION").unwrap_or_else(|_| "unknown".to_string()),
            target: std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string()),
            profile: std::env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string()),
            timestamp: chrono::Local::now().to_rfc3339(),
        })
    }
}

// Version command layer for the launcher
pub struct VersionCommandLayer;

#[async_trait]
impl LauncherLayer for VersionCommandLayer {
    fn name(&self) -> &str {
        "VersionCommandLayer"
    }

    async fn run(&self, _ctx: &mut LauncherContext) -> Result<(), String> {
        // Create the commandlet and runner with UI formatting
        let commandlet = VersionCommandlet;
        let ui = UIManager::default();
        let runner = CommandRunner::with_ui(commandlet, ui);
        
        // Create empty parameters
        let params = VersionParams {};
        let serialized_params = serde_json::to_string(&params)
            .map_err(|e| format!("{}: {}", t(VERSION_ERROR_SERIALIZE), e))?;
        
        // Run the commandlet
        let result = runner.run(&serialized_params).await
            .map_err(|e| format!("{}: {}", t(VERSION_ERROR_EXECUTION), e))?;
            
        // Parse the result
        let parsed_result: VersionResults = serde_json::from_str(&result)
            .map_err(|e| format!("{}: {}", t(VERSION_ERROR_PARSE), e))?;
        
        // Format output using the UI manager
        let ui = runner.ui();
        let heading = ui.heading(1, &t(VERSION_HEADING));
        ui.print(&heading);
        
        // Convert i18n strings to static strings for table headers
        let property_text = t(VERSION_PROPERTY);
        let value_text = t(VERSION_VALUE);
        let headers = &[property_text.as_str(), value_text.as_str()];
        
        // Create rows with property labels
        let rows = vec![
            vec![t(VERSION_PROPERTY_VERSION), parsed_result.version],
            vec![t(VERSION_PROPERTY_RUST_VERSION), parsed_result.rust_version],
            vec![t(VERSION_PROPERTY_BUILD_TARGET), parsed_result.target],
            vec![t(VERSION_PROPERTY_BUILD_PROFILE), parsed_result.profile],
            vec![t(VERSION_PROPERTY_BUILD_TIMESTAMP), parsed_result.timestamp],
        ];
        
        let table = ui.table(headers, &rows);
        ui.print(&table);
        
        Ok(())
    }
} 