use async_trait::async_trait;
use dougu_foundation_run::{LauncherContext, LauncherLayer};
use serde_json;

use crate::{
    DropboxArgs,
    DropboxCommands,
    FileCommands as DropboxFileCommands,
    FolderCommands as DropboxFolderCommands
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
            
            // Use UI manager from context directly
            ctx.ui.text(&ctx.ui.heading(1, "Dropbox Operations"));
            
            match &args.command {
                DropboxCommands::File(file_args) => {
                    ctx.ui.text(&ctx.ui.heading(2, "File Operations"));
                    
                    match &file_args.command {
                        DropboxFileCommands::List(list_args) => {
                            ctx.ui.text(&ctx.ui.info("Listing files from Dropbox..."));
                            
                            crate::execute_file_list(list_args, token, &ctx.ui).await
                                .map_err(|e| format!("Dropbox file list failed: {}", e))?;
                        }
                        DropboxFileCommands::Download(download_args) => {
                            let msg = format!("Downloading file: {}", download_args.path);
                            ctx.ui.text(&ctx.ui.info(&msg));
                            
                            crate::execute_file_download(download_args, token).await
                                .map_err(|e| format!("Dropbox file download failed: {}", e))?;
                            
                            ctx.ui.text(&ctx.ui.success("Download completed successfully"));
                        }
                        DropboxFileCommands::Upload(upload_args) => {
                            let msg = format!("Uploading file to: {}", upload_args.dropbox_path);
                            ctx.ui.text(&ctx.ui.info(&msg));
                            
                            crate::execute_file_upload(upload_args, token).await
                                .map_err(|e| format!("Dropbox file upload failed: {}", e))?;
                            
                            ctx.ui.text(&ctx.ui.success("Upload completed successfully"));
                        }
                    }
                }
                DropboxCommands::Folder(folder_args) => {
                    ctx.ui.text(&ctx.ui.heading(2, "Folder Operations"));
                    
                    match &folder_args.command {
                        DropboxFolderCommands::Create(create_args) => {
                            let msg = format!("Creating folder: {}", create_args.path);
                            ctx.ui.text(&ctx.ui.info(&msg));
                            
                            crate::execute_folder_create(create_args, token).await
                                .map_err(|e| format!("Dropbox folder create failed: {}", e))?;
                            
                            ctx.ui.text(&ctx.ui.success("Folder created successfully"));
                        }
                        DropboxFolderCommands::Delete(delete_args) => {
                            let msg = format!("Deleting folder: {}", delete_args.path);
                            ctx.ui.text(&ctx.ui.info(&msg));
                            
                            crate::execute_folder_delete(delete_args, token).await
                                .map_err(|e| format!("Dropbox folder delete failed: {}", e))?;
                            
                            ctx.ui.text(&ctx.ui.success("Folder deleted successfully"));
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
} 