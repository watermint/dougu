use async_trait::async_trait;
use dougu_foundation_run::{CommandRunner, LauncherContext, LauncherLayer};
use serde_json;

use crate::{
    FileArgs,
    FileCommands,
    FileCommandResult,
    FileCopyCommandlet,
    FileMoveCommandlet,
    FileListCommandlet
};

/// File command layer for the launcher
pub struct FileCommandLayer;

#[async_trait]
impl LauncherLayer for FileCommandLayer {
    fn name(&self) -> &str {
        "FileCommandLayer"
    }

    async fn run(&self, ctx: &mut LauncherContext) -> Result<(), String> {
        if let Some(file_args_str) = ctx.get_data("file_args") {
            // Parse the serialized args into FileCommand
            let file_args: FileArgs = serde_json::from_str(file_args_str)
                .map_err(|e| format!("Failed to parse file args: {}", e))?;

            match file_args.command {
                FileCommands::Copy(copy_args) => {
                    dougu_essentials_logger::log_info(format!("Running file copy command with args: {:?}", copy_args));
                    
                    let commandlet = FileCopyCommandlet;
                    let runner = CommandRunner::with_ui(commandlet, ctx.ui.clone());
                    
                    // Serialize the CopyArgs to JSON string
                    let params = serde_json::to_string(&copy_args)
                        .map_err(|e| format!("Failed to serialize copy args: {}", e))?;
                    
                    // Execute the copy command
                    let result = runner.run(&params).await
                        .map_err(|e| format!("File copy execution failed: {}", e))?;
                    
                    // Parse the result to get details for display
                    let parsed_result: FileCommandResult = serde_json::from_str(&result)
                        .map_err(|e| format!("Failed to parse result: {}", e))?;
                    
                    // Display the result message and details - these are now void functions
                    ctx.ui.text(&parsed_result.message);
                    if let Some(details) = parsed_result.details {
                        ctx.ui.text(&details);
                    }
                },
                FileCommands::Move(move_args) => {
                    // Similar implementation for move command
                    dougu_essentials_logger::log_info(format!("Running file move command with args: {:?}", move_args));
                    
                    let commandlet = FileMoveCommandlet;
                    let runner = CommandRunner::with_ui(commandlet, ctx.ui.clone());
                    
                    // Serialize the MoveArgs to JSON string
                    let params = serde_json::to_string(&move_args)
                        .map_err(|e| format!("Failed to serialize move args: {}", e))?;
                    
                    // Execute the move command
                    let result = runner.run(&params).await
                        .map_err(|e| format!("File move execution failed: {}", e))?;
                    
                    // Parse the result to get details for display
                    let parsed_result: FileCommandResult = serde_json::from_str(&result)
                        .map_err(|e| format!("Failed to parse result: {}", e))?;
                    
                    // Display the result message and details - these are now void functions
                    ctx.ui.text(&parsed_result.message);
                    if let Some(details) = parsed_result.details {
                        ctx.ui.text(&details);
                    }
                },
                FileCommands::List(list_args) => {
                    dougu_essentials_logger::log_info(format!("Running file list command with args: {:?}", list_args));
                    
                    let commandlet = FileListCommandlet;
                    let runner = CommandRunner::with_ui(commandlet, ctx.ui.clone());
                    
                    // Serialize the ListArgs to JSON string
                    let params = serde_json::to_string(&list_args)
                        .map_err(|e| format!("Failed to serialize list args: {}", e))?;
                    
                    // Execute the list command
                    let result = runner.run(&params).await
                        .map_err(|e| format!("File list execution failed: {}", e))?;
                    
                    // Format and display results directly using the runner
                    runner.format_results(&result)
                        .map_err(|e| format!("Failed to format results: {}", e))?;
                }
            }
        }
        
        Ok(())
    }
} 