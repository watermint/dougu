use async_trait::async_trait;
use dougu_foundation_run::{CommandRunner, LauncherContext, LauncherLayer};
use serde_json;

use crate::{FileCommandlet, FileCommandResult};

/// File command layer for the launcher
pub struct FileCommandLayer;

#[async_trait]
impl LauncherLayer for FileCommandLayer {
    fn name(&self) -> &str {
        "FileCommandLayer"
    }

    async fn run(&self, ctx: &mut LauncherContext) -> Result<(), String> {
        // Get the file command arguments from context
        let args_str = match ctx.get_data("file_args") {
            Some(args) => args,
            None => return Err("Missing file command arguments".to_string()),
        };
        
        // Create the commandlet and runner with UI from context
        let commandlet = FileCommandlet;
        let runner = CommandRunner::with_ui(commandlet, ctx.ui.clone());
        
        // Run the commandlet and get the result
        let command_result = runner.run(args_str).await
            .map_err(|e| format!("File command execution failed: {}", e))?;
        
        // Parse and display the result
        if let Ok(result) = serde_json::from_str::<FileCommandResult>(&command_result) {
            ctx.ui.print(&result.message);
            if let Some(details) = result.details {
                ctx.ui.print(&format!("\n{}", details));
            }
        } else {
            // It's probably an error message
            ctx.ui.print(&command_result);
        }
        
        Ok(())
    }
} 