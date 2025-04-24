use anyhow::Result;
use async_trait::async_trait;
use dougu_foundation::{
    run::{LauncherContext, LauncherLayer},
    resources::log_messages,
    ui::UIManager
};
use log::info;
use serde_json;

use crate::dropbox::{
    execute_file_download, execute_file_list, execute_file_upload, execute_folder_create,
    execute_folder_delete, DropboxArgs, DropboxCommands,
    FileCommands, FolderCommands
};

/// Dropbox command layer for the launcher
pub struct DropboxCommandLayer;

#[async_trait]
impl LauncherLayer for DropboxCommandLayer {
    fn name(&self) -> &str {
        "DropboxCommandLayer"
    }

    async fn run(&self, ctx: &mut LauncherContext) -> Result<(), String> {
        if let Some(args_str) = ctx.get_data("dropbox_args") {
            // Parse the serialized args
            let args: DropboxArgs = serde_json::from_str(args_str)
                .map_err(|e| format!("Failed to parse dropbox args: {}", e))?;
            
            // For demo purposes, use a dummy token
            let token = "dummy_dropbox_token";
            
            info!("{}", log_messages::COMMAND_START.replace("{}", "Dropbox"));
            
            // Use UI manager from context directly
            ctx.ui.heading(1, "Dropbox Operations");
            
            match &args.command {
                DropboxCommands::File(file_args) => {
                    ctx.ui.heading(2, "File Operations");
                    
                    match &file_args.command {
                        FileCommands::List(list_args) => {
                            info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "File List"));
                            ctx.ui.info("Listing files from Dropbox...");
                            
                            execute_file_list(list_args, token, &ctx.ui).await
                                .map_err(|e| format!("Dropbox file list failed: {}", e))?;
                            
                            info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "File List"));
                        }
                        FileCommands::Download(download_args) => {
                            info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "File Download"));
                            // Create a local variable for the formatted message
                            let msg = format!("Downloading file: {}", download_args.path);
                            ctx.ui.info(&msg);
                            
                            execute_file_download(download_args, token).await
                                .map_err(|e| format!("Dropbox file download failed: {}", e))?;
                            
                            ctx.ui.success("Download completed successfully");
                            info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "File Download"));
                        }
                        FileCommands::Upload(upload_args) => {
                            info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "File Upload"));
                            // Create a local variable for the formatted message
                            let msg = format!("Uploading file to: {}", upload_args.dropbox_path);
                            ctx.ui.info(&msg);
                            
                            execute_file_upload(upload_args, token).await
                                .map_err(|e| format!("Dropbox file upload failed: {}", e))?;
                            
                            ctx.ui.success("Upload completed successfully");
                            info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "File Upload"));
                        }
                    }
                }
                DropboxCommands::Folder(folder_args) => {
                    ctx.ui.heading(2, "Folder Operations");
                    
                    match &folder_args.command {
                        FolderCommands::Create(create_args) => {
                            info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "Folder Create"));
                            // Create a local variable for the formatted message
                            let msg = format!("Creating folder: {}", create_args.path);
                            ctx.ui.info(&msg);
                            
                            execute_folder_create(create_args, token).await
                                .map_err(|e| format!("Dropbox folder create failed: {}", e))?;
                            
                            ctx.ui.success("Folder created successfully");
                            info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "Folder Create"));
                        }
                        FolderCommands::Delete(delete_args) => {
                            info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "Folder Delete"));
                            // Create a local variable for the formatted message
                            let msg = format!("Deleting folder: {}", delete_args.path);
                            ctx.ui.info(&msg);
                            
                            execute_folder_delete(delete_args, token).await
                                .map_err(|e| format!("Dropbox folder delete failed: {}", e))?;
                            
                            ctx.ui.success("Folder deleted successfully");
                            info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "Folder Delete"));
                        }
                    }
                }
            }
            
            info!("{}", log_messages::COMMAND_COMPLETE.replace("{}", "Dropbox"));
        }
        
        Ok(())
    }
} 