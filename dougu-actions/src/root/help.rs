use async_trait::async_trait;
use dougu_foundation_i18n::t;
use dougu_foundation_run::{Action, ActionError, ActionRunner, LauncherContext, LauncherLayer};
use dougu_foundation_ui::UIManager;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use serde_json;

// Import messages
use crate::root::resources::messages::*;

// Help action
pub struct HelpAction;

#[derive(Serialize, Deserialize)]
pub struct HelpParams {
    pub command: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct HelpResults {
    pub content: String,
}

#[async_trait]
impl Action for HelpAction {
    type Params = HelpParams;
    type Results = HelpResults;
    
    fn name(&self) -> &str {
        "HelpAction"
    }
    
    async fn execute(&self, params: Self::Params) -> Result<Self::Results, ActionError> {
        let content = match params.command.as_deref() {
            Some("file") => t(HELP_COMMAND_FILE),
            Some("dropbox") => t(HELP_COMMAND_DROPBOX),
            Some("obj") => t(HELP_COMMAND_OBJ),
            Some("build") => t(HELP_COMMAND_BUILD),
            Some("version") => t(HELP_COMMAND_VERSION),
            Some("help") => t(HELP_COMMAND_HELP),
            Some("license") => t(HELP_COMMAND_LICENSE),
            Some(cmd) => return Err(ActionError::new(
                "UNKNOWN_COMMAND", 
                &format!("Unknown command: {}", cmd)
            )),
            None => t(HELP_GENERAL)
        };
        
        Ok(HelpResults { content })
    }
}

/// Help action layer for the launcher
pub struct HelpActionLayer;

#[async_trait]
impl LauncherLayer for HelpActionLayer {
    fn name(&self) -> &str {
        "HelpActionLayer"
    }

    async fn run(&self, ctx: &mut LauncherContext) -> Result<(), String> {
        // Create the action
        let action = HelpAction;
        
        // Create a ActionRunner with UI manager from context
        let runner = ActionRunner::with_ui(action, ctx.ui.clone());
        
        // Create help parameters from context data
        let params = HelpParams {
            command: ctx.get_data("help_command").cloned(),
        };
        
        // Serialize parameters
        let serialized_params = serde_json::to_string(&params)
            .map_err(|e| format!("Failed to serialize help params: {}", e))?;
        
        // Run the action and get results
        let result = runner.run(&serialized_params).await
            .map_err(|e| format!("Help action execution failed: {}", e))?;
        
        // Parse the result
        let results: HelpResults = serde_json::from_str(&result)
            .map_err(|e| format!("Failed to parse help results: {}", e))?;
        
        // Display the help content
        ctx.ui.text(&results.content);
        
        Ok(())
    }
} 