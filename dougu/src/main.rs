use anyhow::Result;
use clap::{Parser, Subcommand};
use log::{info, LevelFilter};
use async_trait::async_trait;
use serde_json::json;

use dougu_command_file::{FileArgs, FileCommands, FileCommandlet};
use dougu_command_dropbox::{DropboxArgs, DropboxCommands, FileCommands as DropboxFileCommands};
use dougu_command_obj::ObjCommand;
use dougu_command_build::BuildArgs;
use dougu_foundation_run::{CommandLauncher, LauncherContext, LauncherLayer, CommandRunner, Commandlet};
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
    
    /// Build operations
    Build(BuildArgs),
    
    /// Show version information
    Version,
}

// File command layer using new commandlet architecture
struct FileCommandletLayer;

#[async_trait]
impl LauncherLayer for FileCommandletLayer {
    fn name(&self) -> &str {
        "FileCommandletLayer"
    }

    async fn run(&self, ctx: &mut LauncherContext) -> Result<(), String> {
        if let Some(args_str) = ctx.get_data("file_args") {
            info!("{}", log_messages::COMMAND_START.replace("{}", "File"));
            
            // Create the commandlet
            let commandlet = FileCommandlet;
            let runner = CommandRunner::new(commandlet);
            
            // Run the commandlet with the serialized arguments
            let result = runner.run(args_str).await
                .map_err(|e| format!("File command execution failed: {}", e))?;
            
            // Format and display the result
            let formatted_result = runner.format_results(&result)
                .map_err(|e| format!("Failed to format results: {}", e))?;
            
            // Print the result
            println!("{}", formatted_result);
            
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

// Build command layer
struct BuildCommandLayer;

#[async_trait]
impl LauncherLayer for BuildCommandLayer {
    fn name(&self) -> &str {
        "BuildCommandLayer"
    }

    async fn run(&self, ctx: &mut LauncherContext) -> Result<(), String> {
        if let Some(args_str) = ctx.get_data("build_args") {
            // Parse the serialized args
            let args: BuildArgs = serde_json::from_str(args_str)
                .map_err(|e| format!("Failed to parse build args: {}", e))?;
                
            info!("{}", log_messages::COMMAND_START.replace("{}", "Build"));
            
            match &args.command {
                dougu_command_build::BuildCommands::Package(package_args) => {
                    info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "Package"));
                    dougu_command_build::execute_package(package_args).await
                        .map_err(|e| format!("Build package failed: {}", e))?;
                    info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "Package"));
                }
                dougu_command_build::BuildCommands::Test(test_args) => {
                    info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "Test"));
                    dougu_command_build::execute_test(test_args).await
                        .map_err(|e| format!("Build test failed: {}", e))?;
                    info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "Test"));
                }
                dougu_command_build::BuildCommands::Compile(compile_args) => {
                    info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "Compile"));
                    dougu_command_build::execute_compile(compile_args).await
                        .map_err(|e| format!("Build compile failed: {}", e))?;
                    info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "Compile"));
                }
            }
            
            info!("{}", log_messages::COMMAND_COMPLETE.replace("{}", "Build"));
        }
        
        Ok(())
    }
}

// Version command layer as a Commandlet
struct VersionCommandlet;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct VersionParams {
    // Empty parameters for version command
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct VersionResults {
    pub version: String,
    pub rust_version: String,
    pub target: String,
    pub profile: String,
    pub timestamp: String,
}

#[async_trait]
impl Commandlet for VersionCommandlet {
    type Params = VersionParams;
    type Results = VersionResults;
    
    fn name(&self) -> &str {
        "VersionCommandlet"
    }
    
    async fn execute(&self, _params: Self::Params) -> Result<Self::Results, dougu_foundation_run::CommandletError> {
        Ok(VersionResults {
            version: env!("CARGO_PKG_VERSION").to_string(),
            rust_version: std::env::var("RUSTC_VERSION").unwrap_or_else(|_| "unknown".to_string()),
            target: std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string()),
            profile: std::env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string()),
            timestamp: chrono::Local::now().to_rfc3339(),
        })
    }
}

struct VersionCommandLayer;

#[async_trait]
impl LauncherLayer for VersionCommandLayer {
    fn name(&self) -> &str {
        "VersionCommandLayer"
    }

    async fn run(&self, _ctx: &mut LauncherContext) -> Result<(), String> {
        // Create the commandlet and runner
        let commandlet = VersionCommandlet;
        let runner = CommandRunner::new(commandlet);
        
        // Create empty parameters
        let params = VersionParams {};
        let serialized_params = serde_json::to_string(&params)
            .map_err(|e| format!("Failed to serialize version params: {}", e))?;
        
        // Run the commandlet
        let result = runner.run(&serialized_params).await
            .map_err(|e| format!("Version command execution failed: {}", e))?;
            
        // Parse the result
        let parsed_result: VersionResults = serde_json::from_str(&result)
            .map_err(|e| format!("Failed to parse version results: {}", e))?;
            
        // Display the result
        println!("dougu version: {}", parsed_result.version);
        println!("Rust version: {}", parsed_result.rust_version);
        println!("Build target: {}", parsed_result.target);
        println!("Build profile: {}", parsed_result.profile);
        println!("Build timestamp: {}", parsed_result.timestamp);
        
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
        Commands::Build(_) => "Build",
        Commands::Version => "Version",
    };
    
    let mut context = LauncherContext::new(command_name.to_string(), cli.verbose);
    
    // Add appropriate command layers based on the command
    match &cli.command {
        Commands::File(args) => {
            // Serialize args to string to pass through context
            let args_json = serde_json::to_string(args)
                .map_err(|e| anyhow::anyhow!("Failed to serialize file args: {}", e))?;
            context.set_data("file_args", args_json);
            launcher.add_layer(FileCommandletLayer);
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
        Commands::Build(args) => {
            // Serialize args to string to pass through context
            let args_json = serde_json::to_string(args)
                .map_err(|e| anyhow::anyhow!("Failed to serialize build args: {}", e))?;
            context.set_data("build_args", args_json);
            launcher.add_layer(BuildCommandLayer);
        }
        Commands::Version => {
            launcher.add_layer(VersionCommandLayer);
        }
    }
    
    // Execute the command through the launcher
    launcher.launch(&mut context).await
        .map_err(|e| anyhow::anyhow!("Command execution failed: {}", e))?;
    
    Ok(())
}
