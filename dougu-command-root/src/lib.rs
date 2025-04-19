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

    async fn run(&self, ctx: &mut LauncherContext) -> Result<(), String> {
        // Create the commandlet and runner with UI formatting
        let commandlet = VersionCommandlet;
        // Use UI from context instead of creating a new one
        let runner = CommandRunner::with_ui(commandlet, ctx.ui.clone());
        
        // Create empty parameters
        let params = VersionParams {};
        let serialized_params = serde_json::to_string(&params)
            .map_err(|e| format!("{}: {}", t(VERSION_ERROR_SERIALIZE), e))?;
        
        // Run the commandlet
        let result = runner.run(&serialized_params).await
            .map_err(|e| format!("{}: {}", t(VERSION_ERROR_EXECUTION), e))?;
        
        // Format and print the result using the formatter (respects output format)
        let formatted_result = runner.format_results(&result)
            .map_err(|e| format!("{}: {}", t(VERSION_ERROR_FORMAT), e))?;
        println!("{}", formatted_result);
        
        Ok(())
    }
} 