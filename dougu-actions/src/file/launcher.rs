use anyhow::Result;
use async_trait::async_trait;
use dougu_foundation_run::{LauncherContext, LauncherLayer, ActionRunner, Action};
use dougu_foundation_ui::UIManager;
use log::info;
use dougu_foundation_run::resources::log_messages;
use serde_json;

use crate::file::{
    FileAction, FileActionResult
};

/// File action layer for the launcher
pub struct FileActionLayer;

#[async_trait]
impl LauncherLayer for FileActionLayer {
    fn name(&self) -> &str {
        "FileActionLayer"
    }

    async fn run(&self, ctx: &mut LauncherContext) -> Result<(), String> {
        if let Some(args_str) = ctx.get_data("file_args") {
            info!("{}", log_messages::ACTION_START.replace("{}", "File"));
            
            // Create the action and runner
            let action = FileAction;
            let runner = ActionRunner::with_ui(action, ctx.ui.clone());
            
            // Run with the serialized arguments
            let result = runner.run(args_str).await
                .map_err(|e| format!("File action execution failed: {}", e))?;
            
            // Parse the result to get details for display
            let parsed_result: FileActionResult = serde_json::from_str(&result)
                .map_err(|e| format!("Failed to parse result: {}", e))?;
            
            match parsed_result {
                FileActionResult::Copy(copy_result) => {
                    // Format and display copy results
                    ctx.ui.heading(2, "File Copy Results");
                    ctx.ui.info(&format!("Copied from: {}", copy_result.source));
                    ctx.ui.info(&format!("Copied to: {}", copy_result.destination));
                    
                    // If details are present, print them
                    if let Some(details) = copy_result.details.as_deref() {
                        ctx.ui.text(details);
                    }
                }
                FileActionResult::Move(move_result) => {
                    // Format and display move results
                    ctx.ui.heading(2, "File Move Results");
                    ctx.ui.info(&format!("Moved from: {}", move_result.source));
                    ctx.ui.info(&format!("Moved to: {}", move_result.destination));
                    
                    // If details are present, print them
                    if let Some(details) = move_result.details.as_deref() {
                        ctx.ui.text(details);
                    }
                }
                FileActionResult::List(list_result) => {
                    // Format and display list results
                    ctx.ui.heading(2, "File List Results");
                    ctx.ui.info(&format!("Directory: {}", list_result.directory));
                    
                    for file in &list_result.files {
                        ctx.ui.info(file);
                    }
                }
            }
            
            info!("{}", log_messages::ACTION_COMPLETE.replace("{}", "File"));
        }
        
        Ok(())
    }
} 