use anyhow::Result;
use async_trait::async_trait;
use dougu_foundation::{
    run::{Action, ActionError, ActionRunner, LauncherContext, LauncherLayer},
    ui::UIManager
};
use serde::{Deserialize, Serialize};
use serde_json;

// License action
pub struct LicenseAction;

#[derive(Serialize, Deserialize)]
pub struct LicenseParams {
    // No parameters needed for license action
}

#[derive(Serialize, Deserialize)]
pub struct LicenseResults {
    pub license_text: String,
    pub license_type: String,
}

#[async_trait]
impl Action for LicenseAction {
    type Params = LicenseParams;
    type Results = LicenseResults;
    
    fn name(&self) -> &str {
        "LicenseAction"
    }
    
    async fn execute(&self, _params: Self::Params) -> Result<Self::Results, ActionError> {
        // In a real app, this would return actual license text
        Ok(LicenseResults {
            license_text: "MIT and Apache 2.0 License Text would go here".to_string(),
            license_type: "Dual License: MIT/Apache 2.0".to_string(),
        })
    }
}

// Display license results directly with UI methods
pub fn display_license_results(ui: &UIManager, results: &LicenseResults) -> Result<(), ActionError> {
    // Display license heading
    ui.heading(1, "License Information");
    ui.line_break();
    
    // Display license type
    ui.heading(2, &results.license_type);
    ui.line_break();
    
    // Display license text
    ui.text(&results.license_text);
    
    Ok(())
}

/// License action layer for the launcher
pub struct LicenseActionLayer;

#[async_trait]
impl LauncherLayer for LicenseActionLayer {
    fn name(&self) -> &str {
        "LicenseActionLayer"
    }

    async fn run(&self, ctx: &mut LauncherContext) -> Result<(), String> {
        // Create the LicenseAction
        let action = LicenseAction;
        
        // Create a runner with UI manager
        let runner = ActionRunner::with_ui(action, ctx.ui.clone());
        
        // Execute the action with empty params
        let params = LicenseParams {};
        let params_str = serde_json::to_string(&params)
            .map_err(|e| format!("Failed to serialize license params: {}", e))?;
        
        let result = runner.run(&params_str).await
            .map_err(|e| format!("License action execution failed: {}", e))?;
        
        // Format results
        runner.format_results(&result)
            .map_err(|e| format!("Failed to format license results: {}", e))?;
        
        Ok(())
    }
} 