use anyhow::Result;
use async_trait::async_trait;
use dougu_essentials::{
    obj::notation::{Notation, json::JsonNotation},
    obj::prelude::*
};
use dougu_foundation::{
    run::{Action, ActionError, ActionRunner, LauncherContext, LauncherLayer},
    ui::UIManager
};

// License action
pub struct LicenseAction;

pub struct LicenseParams {
    // No parameters needed for license action
}

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
    ui.line_break();
    
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
        if let Some(args_str) = ctx.get_data("license_args") {
            let json_notation = JsonNotation::new();
            let params: LicenseParams = json_notation.decode(args_str.as_bytes())
                .map_err(|e| format!("Failed to parse license args: {}", e))?;
                
            let action = LicenseAction;
            let results = action.execute(params)
                .await
                .map_err(|e| format!("License action failed: {}", e))?;
                
            display_license_results(ctx.ui, &results)
                .map_err(|e| format!("Failed to display license results: {}", e))?;
        }
        Ok(())
    }
} 