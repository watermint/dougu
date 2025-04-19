use anyhow::Result;
use clap::{Parser, Subcommand};
use log::{info, LevelFilter};
use async_trait::async_trait;

use dougu_command_file::{FileArgs, FileCommandlet};
use dougu_command_dropbox::{DropboxArgs, DropboxCommands, FileCommands as DropboxFileCommands};
use dougu_command_obj::ObjCommand;
use dougu_command_build::BuildArgs;
use dougu_command_root::VersionCommandLayer;
use dougu_foundation_run::{CommandLauncher, LauncherContext, LauncherLayer, CommandRunner};
use dougu_foundation_run::resources::log_messages;
use dougu_foundation_ui::UIManager;
use dougu_essentials_i18n_foundation::I18nInitializerLayer;

// Keep the i18n module for potential future use
mod i18n;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Set verbosity level (0-5)
    #[arg(short, long, default_value_t = 2)]
    verbose: u8,

    /// Set locale for internationalization (e.g., 'en', 'es')
    #[arg(short, long, default_value = "en")]
    locale: String,

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
            
            // Create the commandlet with UI manager
            let commandlet = FileCommandlet;
            let ui = UIManager::default();
            let runner = CommandRunner::with_ui(commandlet, ui);
            
            // Run the commandlet with the serialized arguments
            let result = runner.run(args_str).await
                .map_err(|e| format!("File command execution failed: {}", e))?;
            
            // Format and display the result using UI manager
            let formatted_result = runner.format_results(&result)
                .map_err(|e| format!("Failed to format results: {}", e))?;
            
            // Display with appropriate UI formatting
            let heading = runner.ui().heading(1, "File Operation Result");
            runner.ui().print(&heading);
            runner.ui().print(&formatted_result);
            
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
            
            // Create UI manager for formatted output
            let ui = UIManager::default();
            ui.print(&ui.heading(1, "Dropbox Operations"));
            
            match &args.command {
                DropboxCommands::File(file_args) => {
                    ui.print(&ui.heading(2, "File Operations"));
                    
                    match &file_args.command {
                        DropboxFileCommands::List(list_args) => {
                            info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "File List"));
                            ui.print(&ui.info("Listing files from Dropbox..."));
                            
                            dougu_command_dropbox::execute_file_list(list_args, token).await
                                .map_err(|e| format!("Dropbox file list failed: {}", e))?;
                            
                            info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "File List"));
                        }
                        DropboxFileCommands::Download(download_args) => {
                            info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "File Download"));
                            // Create a local variable for the formatted message
                            let msg = format!("Downloading file: {}", download_args.path);
                            ui.print(&ui.info(&msg));
                            
                            dougu_command_dropbox::execute_file_download(download_args, token).await
                                .map_err(|e| format!("Dropbox file download failed: {}", e))?;
                            
                            ui.print(&ui.success("Download completed successfully"));
                            info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "File Download"));
                        }
                        DropboxFileCommands::Upload(upload_args) => {
                            info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "File Upload"));
                            // Create a local variable for the formatted message
                            let msg = format!("Uploading file to: {}", upload_args.dropbox_path);
                            ui.print(&ui.info(&msg));
                            
                            dougu_command_dropbox::execute_file_upload(upload_args, token).await
                                .map_err(|e| format!("Dropbox file upload failed: {}", e))?;
                            
                            ui.print(&ui.success("Upload completed successfully"));
                            info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "File Upload"));
                        }
                    }
                }
                DropboxCommands::Folder(folder_args) => {
                    ui.print(&ui.heading(2, "Folder Operations"));
                    
                    match &folder_args.command {
                        dougu_command_dropbox::FolderCommands::Create(create_args) => {
                            info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "Folder Create"));
                            // Create a local variable for the formatted message
                            let msg = format!("Creating folder: {}", create_args.path);
                            ui.print(&ui.info(&msg));
                            
                            dougu_command_dropbox::execute_folder_create(create_args, token).await
                                .map_err(|e| format!("Dropbox folder create failed: {}", e))?;
                            
                            ui.print(&ui.success("Folder created successfully"));
                            info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "Folder Create"));
                        }
                        dougu_command_dropbox::FolderCommands::Delete(delete_args) => {
                            info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "Folder Delete"));
                            // Create a local variable for the formatted message
                            let msg = format!("Deleting folder: {}", delete_args.path);
                            ui.print(&ui.info(&msg));
                            
                            dougu_command_dropbox::execute_folder_delete(delete_args, token).await
                                .map_err(|e| format!("Dropbox folder delete failed: {}", e))?;
                            
                            ui.print(&ui.success("Folder deleted successfully"));
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
            
            // Create UI manager for formatted output
            let ui = UIManager::default();
            ui.print(&ui.heading(1, "Build Operations"));
            
            match &args.command {
                dougu_command_build::BuildCommands::Package(package_args) => {
                    info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "Package"));
                    ui.print(&ui.heading(2, "Packaging Application"));
                    
                    // Get the output directory as a placeholder for package name
                    let package_name = package_args.output_dir.clone().unwrap_or_else(|| "default".to_string());
                    let msg = format!("Creating package in: {}", package_name);
                    ui.print(&ui.info(&msg));
                    
                    dougu_command_build::execute_package(package_args).await
                        .map_err(|e| format!("Build package failed: {}", e))?;
                    
                    ui.print(&ui.success("Package created successfully"));
                    info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "Package"));
                }
                dougu_command_build::BuildCommands::Test(test_args) => {
                    info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "Test"));
                    ui.print(&ui.heading(2, "Running Tests"));
                    
                    let test_filter = test_args.filter.clone().unwrap_or_else(|| "all tests".to_string());
                    let msg = format!("Running test suite with filter: {}", test_filter);
                    ui.print(&ui.info(&msg));
                    
                    dougu_command_build::execute_test(test_args).await
                        .map_err(|e| format!("Build test failed: {}", e))?;
                    
                    ui.print(&ui.success("Tests completed successfully"));
                    info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "Test"));
                }
                dougu_command_build::BuildCommands::Compile(compile_args) => {
                    info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "Compile"));
                    ui.print(&ui.heading(2, "Compiling Project"));
                    
                    let build_type = if compile_args.release { "release" } else { "debug" };
                    let msg = format!("Compiling with build type: {}", build_type);
                    ui.print(&ui.info(&msg));
                    
                    dougu_command_build::execute_compile(compile_args).await
                        .map_err(|e| format!("Build compile failed: {}", e))?;
                    
                    ui.print(&ui.success("Compilation completed successfully"));
                    info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "Compile"));
                }
            }
            
            info!("{}", log_messages::COMMAND_COMPLETE.replace("{}", "Build"));
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
        Commands::Build(_) => "Build",
        Commands::Version => "Version",
    };
    
    let mut context = LauncherContext::new(command_name.to_string(), cli.verbose);
    
    // Store the locale in the context for I18nInitializerLayer to use
    context.set_data("locale", cli.locale.clone());
    
    // Add the I18nInitializerLayer as the first layer
    launcher.add_layer(I18nInitializerLayer::new(&cli.locale));
    
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
