use async_trait::async_trait;
use dougu_foundation_i18n::t;
use dougu_foundation_run::{Commandlet, CommandletError, CommandRunner, LauncherContext, LauncherLayer};
use dougu_foundation_ui::UIManager;
use dougu_essentials_build::get_build_info;
use serde::{Serialize, Deserialize};
use serde_json;

// Import messages
use crate::commands::root::resources::messages::*;

// License command as a Commandlet
pub struct LicenseCommandlet;

#[derive(Serialize, Deserialize)]
pub struct LicenseParams {
    // Empty parameters for license command
}

#[derive(Serialize, Deserialize)]
pub struct LicenseResults {
    pub license_name: String,
    pub license_text: String,
}

#[async_trait]
impl Commandlet for LicenseCommandlet {
    type Params = LicenseParams;
    type Results = LicenseResults;
    
    fn name(&self) -> &str {
        "LicenseCommandlet"
    }
    
    async fn execute(&self, _params: Self::Params) -> Result<Self::Results, CommandletError> {
        // Get build information for copyright year formatting
        let build_info = get_build_info();
        
        // Format the license text with dynamic copyright information
        let license_text = t(LICENSE_TEXT)
            .replace("{start_year}", &build_info.copyright_start_year.to_string())
            .replace("{current_year}", &build_info.copyright_year.to_string())
            .replace("{copyright_owner}", &build_info.copyright_owner);
        
        // Return the license information
        Ok(LicenseResults {
            license_name: "Apache License 2.0".to_string(),
            license_text,
        })
    }
}

// Display license results directly with UI methods
pub fn display_license_results(ui: &UIManager, results: &LicenseResults) -> Result<(), CommandletError> {
    // Display license heading
    ui.heading(1, &t(LICENSE_HEADING));
    ui.line_break();
    
    // Display license name
    ui.heading(2, &results.license_name);
    ui.line_break();
    
    // Display license text
    ui.text(&results.license_text);
    
    Ok(())
}

// License command layer for the launcher
pub struct LicenseCommandLayer;

#[async_trait]
impl LauncherLayer for LicenseCommandLayer {
    fn name(&self) -> &str {
        "LicenseCommandLayer"
    }

    async fn run(&self, ctx: &mut LauncherContext) -> Result<(), String> {
        // Create the LicenseCommandlet 
        let commandlet = LicenseCommandlet;
        
        // Create a CommandRunner with UI manager from context
        let runner = CommandRunner::with_ui(commandlet, ctx.ui.clone());
        
        // Empty params for license command
        let params = LicenseParams {};
        
        // Serialize the params
        let serialized_params = serde_json::to_string(&params)
            .map_err(|e| format!("{}: {}", t(LICENSE_ERROR_SERIALIZE), e))?;
        
        // Run the command with serialized parameters
        let result = runner.run(&serialized_params).await
            .map_err(|e| format!("{}: {}", t(LICENSE_ERROR_EXECUTION), e))?;
        
        // Parse the results
        let license_results: LicenseResults = serde_json::from_str(&result)
            .map_err(|e| format!("Failed to parse license results: {}", e))?;
        
        // Display the results directly
        display_license_results(&ctx.ui, &license_results)
            .map_err(|e| format!("Failed to display license results: {}", e))?;
        
        Ok(())
    }
} 