use async_trait::async_trait;
use dougu_foundation_i18n::t;
use dougu_foundation_run::{Commandlet, CommandletError, CommandRunner, LauncherContext, LauncherLayer};
use serde::{Deserialize, Serialize};

use crate::resources::messages::*;

// Help command as a Commandlet
pub struct HelpCommandlet;

#[derive(Serialize, Deserialize)]
pub struct HelpParams {
    // Optional command to get help for
    pub command: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct HelpResults {
    pub content: String,
}

#[async_trait]
impl Commandlet for HelpCommandlet {
    type Params = HelpParams;
    type Results = HelpResults;
    
    fn name(&self) -> &str {
        "HelpCommandlet"
    }
    
    async fn execute(&self, params: Self::Params) -> Result<Self::Results, CommandletError> {
        // Generate help content based on the command
        let content = match params.command.as_deref() {
            Some("file") => t(HELP_COMMAND_FILE),
            Some("dropbox") => t(HELP_COMMAND_DROPBOX),
            Some("obj") => t(HELP_COMMAND_OBJ),
            Some("build") => t(HELP_COMMAND_BUILD),
            Some("version") => t(HELP_COMMAND_VERSION),
            Some("help") => t(HELP_COMMAND_HELP),
            Some(cmd) => return Err(CommandletError::new(
                "UNKNOWN_COMMAND", 
                &format!("Unknown command: {}", cmd)
            )),
            None => t(HELP_GENERAL),
        };
        
        Ok(HelpResults { content })
    }
}

// Help command layer for the launcher
pub struct HelpCommandLayer;

#[async_trait]
impl LauncherLayer for HelpCommandLayer {
    fn name(&self) -> &str {
        "HelpCommandLayer"
    }

    async fn run(&self, ctx: &mut LauncherContext) -> Result<(), String> {
        // Create the commandlet and runner with UI formatting
        let commandlet = HelpCommandlet;
        // Use UI from context instead of creating a new one
        let runner = CommandRunner::with_ui(commandlet, ctx.ui.clone());
        
        // Get command argument if any
        let command = ctx.get_data("help_command").cloned();
        
        // Create parameters
        let params = HelpParams { command };
        let serialized_params = serde_json::to_string(&params)
            .map_err(|e| format!("Failed to serialize help params: {}", e))?;
        
        // Run the commandlet and get the serialized result
        let command_result = runner.run(&serialized_params).await
            .map_err(|e| format!("Help command execution failed: {}", e))?;
        
        // Parse and print the result
        if let Ok(results) = serde_json::from_str::<HelpResults>(&command_result) {
            ctx.ui.print(&results.content);
        } else {
            // It's probably an error message
            ctx.ui.print(&command_result);
        }
        
        Ok(())
    }
} 