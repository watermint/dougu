use anyhow::Result;
use clap::{Parser, Subcommand};
use log::{info, LevelFilter};
use async_trait::async_trait;

use dougu_command_file::{FileArgs, FileCommands};
use dougu_command_dropbox::{DropboxArgs, DropboxCommands, FileCommands as DropboxFileCommands};
use dougu_command_obj::ObjCommand;
use dougu_foundation_run::{CommandLauncher, LauncherContext, LauncherLayer};
use dougu_foundation_run::resources::log_messages;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Set verbosity level (0-5)
    #[arg(short, long, default_value_t = 2)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// File operations
    File(FileArgs),
    
    /// Dropbox operations
    Dropbox(DropboxArgs),
    
    /// Object notation operations (JSON, BSON, XML, CBOR)
    Obj(ObjCommand),
}

// File command layer
struct FileCommandLayer;

#[async_trait]
impl LauncherLayer for FileCommandLayer {
    fn name(&self) -> &str {
        "FileCommandLayer"
    }

    async fn run(&self, ctx: &mut LauncherContext) -> Result<(), String> {
        if let Some(args_str) = ctx.get_data("file_args") {
            // Parse the serialized args
            let args: FileArgs = serde_json::from_str(args_str)
                .map_err(|e| format!("Failed to parse file args: {}", e))?;
                
            info!("{}", log_messages::COMMAND_START.replace("{}", "File"));
            
            match &args.command {
                FileCommands::Copy(copy_args) => {
                    info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "Copy"));
                    dougu_command_file::execute_copy(copy_args)
                        .map_err(|e| format!("File copy failed: {}", e))?;
                    info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "Copy"));
                }
                FileCommands::Move(move_args) => {
                    info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "Move"));
                    dougu_command_file::execute_move(move_args)
                        .map_err(|e| format!("File move failed: {}", e))?;
                    info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "Move"));
                }
                FileCommands::List(list_args) => {
                    info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "List"));
                    dougu_command_file::execute_list(list_args)
                        .map_err(|e| format!("File list failed: {}", e))?;
                    info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "List"));
                }
            }
            
            info!("{}", log_messages::COMMAND_COMPLETE.replace("{}", "File"));
        }
        
        Ok(())
    }
}

// Dropbox command layer
struct DropboxCommandLayer;

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
            
            match &args.command {
                DropboxCommands::File(file_args) => {
                    match &file_args.command {
                        DropboxFileCommands::List(list_args) => {
                            info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "File List"));
                            dougu_command_dropbox::execute_file_list(list_args, token).await
                                .map_err(|e| format!("Dropbox file list failed: {}", e))?;
                            info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "File List"));
                        }
                        DropboxFileCommands::Download(download_args) => {
                            info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "File Download"));
                            dougu_command_dropbox::execute_file_download(download_args, token).await
                                .map_err(|e| format!("Dropbox file download failed: {}", e))?;
                            info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "File Download"));
                        }
                        DropboxFileCommands::Upload(upload_args) => {
                            info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "File Upload"));
                            dougu_command_dropbox::execute_file_upload(upload_args, token).await
                                .map_err(|e| format!("Dropbox file upload failed: {}", e))?;
                            info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "File Upload"));
                        }
                    }
                }
                DropboxCommands::Folder(folder_args) => {
                    match &folder_args.command {
                        dougu_command_dropbox::FolderCommands::Create(create_args) => {
                            info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "Folder Create"));
                            dougu_command_dropbox::execute_folder_create(create_args, token).await
                                .map_err(|e| format!("Dropbox folder create failed: {}", e))?;
                            info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "Folder Create"));
                        }
                        dougu_command_dropbox::FolderCommands::Delete(delete_args) => {
                            info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "Folder Delete"));
                            dougu_command_dropbox::execute_folder_delete(delete_args, token).await
                                .map_err(|e| format!("Dropbox folder delete failed: {}", e))?;
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

// Object command layer
struct ObjCommandLayer;

#[async_trait]
impl LauncherLayer for ObjCommandLayer {
    fn name(&self) -> &str {
        "ObjCommandLayer"
    }

    async fn run(&self, ctx: &mut LauncherContext) -> Result<(), String> {
        if let Some(args_str) = ctx.get_data("obj_args") {
            // Parse the serialized args
            let cmd: ObjCommand = serde_json::from_str(args_str)
                .map_err(|e| format!("Failed to parse obj args: {}", e))?;
            
            info!("{}", log_messages::COMMAND_START.replace("{}", "Obj"));
            
            cmd.execute().await
                .map_err(|e| format!("Obj command execution failed: {}", e))?;
            
            info!("{}", log_messages::COMMAND_COMPLETE.replace("{}", "Obj"));
        }
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Set up logging based on verbosity
    let level = match cli.verbose {
        0 => LevelFilter::Off,
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        4 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    
    info!("{}", log_messages::SETTING_VERBOSITY.replace("{}", &level.to_string()));
    dougu_essentials_logger::init(level)?;
    
    // Create CommandLauncher
    let mut launcher = CommandLauncher::new();
    
    // Create context with command name and verbosity
    let command_name = match &cli.command {
        Commands::File(_) => "File",
        Commands::Dropbox(_) => "Dropbox",
        Commands::Obj(_) => "Obj",
    };
    
    let mut context = LauncherContext::new(command_name.to_string(), cli.verbose);
    
    // Add appropriate command layers based on the command
    match &cli.command {
        Commands::File(args) => {
            // Serialize args to string to pass through context
            let args_json = serde_json::to_string(args)
                .map_err(|e| anyhow::anyhow!("Failed to serialize file args: {}", e))?;
            context.set_data("file_args", args_json);
            launcher.add_layer(FileCommandLayer);
        }
        Commands::Dropbox(args) => {
            // Serialize args to string to pass through context
            let args_json = serde_json::to_string(args)
                .map_err(|e| anyhow::anyhow!("Failed to serialize dropbox args: {}", e))?;
            context.set_data("dropbox_args", args_json);
            launcher.add_layer(DropboxCommandLayer);
        }
        Commands::Obj(cmd) => {
            // Serialize args to string to pass through context
            let args_json = serde_json::to_string(cmd)
                .map_err(|e| anyhow::anyhow!("Failed to serialize obj args: {}", e))?;
            context.set_data("obj_args", args_json);
            launcher.add_layer(ObjCommandLayer);
        }
    }
    
    // Execute the command through the launcher
    launcher.launch(&mut context).await
        .map_err(|e| anyhow::anyhow!("Command execution failed: {}", e))?;
    
    Ok(())
}
