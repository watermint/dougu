use anyhow::Result;
use async_trait::async_trait;
use clap::{Parser, Subcommand};
use log::{info, LevelFilter};
use std::str::FromStr;
use env_logger;

// Remove the local actions module
// mod actions;

use dougu_actions::build::BuildArgs;
use dougu_actions::dropbox::{DropboxArgs};
// Use actions from the dougu-actions crate
use dougu_actions::file::{FileAction, FileArgs};
// Remove unused imports
// use dougu_actions::file::{FileCopyAction, FileListAction, FileMoveAction};
use dougu_actions::obj::{ObjCommand, CommandLayer};
use dougu_actions::root::LicenseActionLayer;
use dougu_foundation::i18n::Locale;
use dougu_foundation::resources::log_messages;
use dougu_foundation::run::{ActionLauncher, ActionRunner, I18nInitializerLayer, LauncherContext, LauncherLayer};
use dougu_foundation::ui::resources::ui_messages;
use dougu_foundation::ui::OutputFormat;
// Import app_info::display_app_info separately
use dougu_foundation::run::app_info::display_app_info;
use dougu_essentials::obj::{Notation, NotationType};

// Keep the i18n module for potential future use
mod i18n;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Set verbosity level (0-5)
    #[arg(long = "ui-verbose", default_value_t = 2, global = true)]
    verbose: u8,

    /// Set locale for internationalization (e.g., 'en', 'es')
    #[arg(long = "ui-locale", default_value = "en", global = true)]
    locale: String,

    /// Set output format (default, jsonl, markdown)
    #[arg(long = "ui-format", value_parser = ["default", "jsonl", "markdown"], default_value = "default", global = true)]
    format: String,

    /// Skip displaying application info at startup
    #[arg(long = "ui-skip-appinfo", help = ui_messages::SKIP_APPINFO_OPTION_DESCRIPTION, global = true)]
    skip_appinfo: bool,

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
    
    /// Display license information
    License,
}

// File action layer using new action architecture
struct FileActionLayer;

#[async_trait]
impl LauncherLayer for FileActionLayer {
    fn name(&self) -> &str {
        "FileActionLayer"
    }

    async fn run(&self, ctx: &mut LauncherContext) -> Result<(), String> {
        if let Some(args_str) = ctx.get_data("file_args") {
            info!("{}", log_messages::ACTION_START.replace("{}", "File"));
            
            let action = FileAction;
            let runner = ActionRunner::with_ui(action, ctx.ui.clone());
            
            let args_with_locale = if args_str.trim().starts_with('{') && args_str.trim().ends_with('}') {
                if let Ok(notation) = NotationType::Json.decode::<NotationType>(args_str.as_bytes()) {
                    if let NotationType::Object(mut obj) = notation {
                        if !obj.iter().any(|(k, _)| k == "context") {
                            let mut context = Vec::new();
                            context.push(("locale".to_string(), NotationType::String(ctx.get_locale().as_str().to_string())));
                            obj.push(("context".to_string(), NotationType::Object(context)));
                        } else if let Some((_, context)) = obj.iter_mut().find(|(k, _)| k == "context") {
                            if let NotationType::Object(context_obj) = context {
                                context_obj.push(("locale".to_string(), NotationType::String(ctx.get_locale().as_str().to_string())));
                            }
                        }
                        
                        if let Ok(new_json) = NotationType::Json.encode_to_string(&NotationType::Object(obj)) {
                            new_json
                        } else {
                            args_str.to_string()
                        }
                    } else {
                        args_str.to_string()
                    }
                } else {
                    args_str.to_string()
                }
            } else {
                args_str.to_string()
            };
            
            let result = runner.run(&args_with_locale).await
                .map_err(|e| format!("File action execution failed: {}", e))?;
            
            runner.format_results(&result)
                .map_err(|e| format!("Failed to format results: {}", e))?;
            
            info!("{}", log_messages::ACTION_COMPLETE.replace("{}", "File"));
        }
        
        Ok(())
    }
}

// Dropbox action layer
struct DropboxActionLayer;

#[async_trait]
impl LauncherLayer for DropboxActionLayer {
    fn name(&self) -> &str {
        "DropboxActionLayer"
    }

    async fn run(&self, ctx: &mut LauncherContext) -> Result<(), String> {
        if let Some(args_str) = ctx.get_data("dropbox_args") {
            // Decode using NotationType::Json
            let args: DropboxArgs = NotationType::Json.decode::<DropboxArgs>(args_str.as_bytes())
                .map_err(|e| format!("Failed to parse dropbox args: {}", e))?;
            
            let token = "dummy_dropbox_token"; // Replace with actual token management
            
            info!("{}", log_messages::ACTION_START.replace("{}", "Dropbox"));
            
            ctx.ui.heading(1, "Dropbox Operations");
            
            match &args.command {
                dougu_actions::dropbox::DropboxCommands::File(file_args) => {
                    ctx.ui.heading(2, "File Operations");
                    
                    match &file_args.command {
                        dougu_actions::dropbox::FileCommands::List(list_args) => {
                            info!("{}", log_messages::ACTION_START.replace("{}", "File List"));
                            ctx.ui.info("Listing files from Dropbox...");
                            
                            dougu_actions::dropbox::execute_file_list(list_args, token, &ctx.ui).await
                                .map_err(|e| format!("Dropbox file list failed: {}", e))?;
                            
                            info!("{}", log_messages::ACTION_COMPLETE.replace("{}", "File List"));
                        }
                        dougu_actions::dropbox::FileCommands::Download(download_args) => {
                            info!("{}", log_messages::ACTION_START.replace("{}", "File Download"));
                            // Create a local variable for the formatted message
                            let msg = format!("Downloading file: {}", download_args.path);
                            ctx.ui.info(&msg);
                            
                            dougu_actions::dropbox::execute_file_download(download_args, token).await
                                .map_err(|e| format!("Dropbox file download failed: {}", e))?;
                            
                            ctx.ui.success("Download completed successfully");
                            info!("{}", log_messages::ACTION_COMPLETE.replace("{}", "File Download"));
                        }
                        dougu_actions::dropbox::FileCommands::Upload(upload_args) => {
                            info!("{}", log_messages::ACTION_START.replace("{}", "File Upload"));
                            // Create a local variable for the formatted message
                            let msg = format!("Uploading file to: {}", upload_args.dropbox_path);
                            ctx.ui.info(&msg);
                            
                            dougu_actions::dropbox::execute_file_upload(upload_args, token).await
                                .map_err(|e| format!("Dropbox file upload failed: {}", e))?;
                            
                            ctx.ui.success("Upload completed successfully");
                            info!("{}", log_messages::ACTION_COMPLETE.replace("{}", "File Upload"));
                        }
                    }
                }
                dougu_actions::dropbox::DropboxCommands::Folder(folder_args) => {
                    ctx.ui.heading(2, "Folder Operations");
                    
                    match &folder_args.command {
                        dougu_actions::dropbox::FolderCommands::Create(create_args) => {
                            info!("{}", log_messages::ACTION_START.replace("{}", "Folder Create"));
                            // Create a local variable for the formatted message
                            let msg = format!("Creating folder: {}", create_args.path);
                            ctx.ui.info(&msg);
                            
                            dougu_actions::dropbox::execute_folder_create(create_args, token).await
                                .map_err(|e| format!("Dropbox folder create failed: {}", e))?;
                            
                            ctx.ui.success("Folder created successfully");
                            info!("{}", log_messages::ACTION_COMPLETE.replace("{}", "Folder Create"));
                        }
                        dougu_actions::dropbox::FolderCommands::Delete(delete_args) => {
                            info!("{}", log_messages::ACTION_START.replace("{}", "Folder Delete"));
                            // Create a local variable for the formatted message
                            let msg = format!("Deleting folder: {}", delete_args.path);
                            ctx.ui.info(&msg);
                            
                            dougu_actions::dropbox::execute_folder_delete(delete_args, token).await
                                .map_err(|e| format!("Dropbox folder delete failed: {}", e))?;
                            
                            ctx.ui.success("Folder deleted successfully");
                            info!("{}", log_messages::ACTION_COMPLETE.replace("{}", "Folder Delete"));
                        }
                    }
                }
            }
            
            info!("{}", log_messages::ACTION_COMPLETE.replace("{}", "Dropbox"));
        }
        
        Ok(())
    }
}

// Obj action layer
struct ObjActionLayer;

// Implement CommandLayer for ObjActionLayer
impl CommandLayer for ObjActionLayer {
    fn handle_command(&self, command: &str, args: Vec<String>) -> Result<(), String> {
        // Implementation of command handling
        match command {
            "query" => {
                println!("Executing query: {:?}", args);
                // Implement query command
                Ok(())
            }
            "convert" => {
                println!("Converting: {:?}", args);
                // Implement convert command
                Ok(())
            }
            "extract" => {
                println!("Extracting: {:?}", args);
                // Implement extract command
                Ok(())
            }
            _ => Err(format!("Unknown command: {}", command))
        }
    }
    
    fn is_empty(&self, result: &str) -> bool {
        result.is_empty() || result == "{}" || result == "[]" || result == "null"
    }
}

// Implement LauncherLayer for ObjActionLayer to use with launcher
#[async_trait]
impl LauncherLayer for ObjActionLayer {
    fn name(&self) -> &str {
        "ObjActionLayer"
    }
    
    async fn run(&self, ctx: &mut LauncherContext) -> Result<(), String> {
        if let Some(args_str) = ctx.get_data("obj_args") {
            info!("{}", log_messages::ACTION_START.replace("{}", "Object Operation"));
            
            // For debugging
            println!("ObjCommand arguments: {}", args_str);
            
            // Basic handling based on the string content
            if args_str.contains("Query") {
                ctx.ui.heading(1, "Object Query");
                self.handle_command("query", vec!["placeholder".to_string()])?;
            } else if args_str.contains("Convert") {
                ctx.ui.heading(1, "Object Conversion");
                self.handle_command("convert", vec!["placeholder".to_string()])?;
            } else if args_str.contains("Extract") {
                ctx.ui.heading(1, "Object Extraction");
                self.handle_command("extract", vec![])?;
            } else {
                return Err("Unknown ObjCommand variant".to_string());
            }
            
            info!("{}", log_messages::ACTION_COMPLETE.replace("{}", "Object Operation"));
        }
        
        Ok(())
    }
}

// Build action layer
struct BuildActionLayer;

#[async_trait]
impl LauncherLayer for BuildActionLayer {
    fn name(&self) -> &str {
        "BuildActionLayer"  
    }
    
    async fn run(&self, ctx: &mut LauncherContext) -> Result<(), String> {
        if let Some(args_str) = ctx.get_data("build_args") {
            info!("{}", log_messages::ACTION_START.replace("{}", "Build"));
            
            // Parse the serialized args using NotationType::Json
            let args: BuildArgs = NotationType::Json.decode::<BuildArgs>(args_str.as_bytes())
                .map_err(|e| format!("Failed to parse build args: {}", e))?;
            
            // Execute using appropriate action type
            match &args.command {
                dougu_actions::build::BuildCommands::Package(package_args) => {
                    dougu_actions::build::execute_package(package_args).await
                        .map_err(|e| e.to_string())?;
                },
                dougu_actions::build::BuildCommands::Test(test_args) => {
                    dougu_actions::build::execute_test(test_args, &ctx.ui).await
                        .map_err(|e| e.to_string())?;
                },
                dougu_actions::build::BuildCommands::Compile(compile_args) => {
                    dougu_actions::build::execute_compile(compile_args, &ctx.ui).await
                        .map_err(|e| e.to_string())?;
                },
                dougu_actions::build::BuildCommands::Pack(pack_args) => {
                    dougu_actions::build::execute_pack(pack_args, &ctx.ui).await
                        .map_err(|e| e.to_string())?;
                },
                dougu_actions::build::BuildCommands::Spec(spec_args) => {
                    dougu_actions::build::execute_spec(spec_args, &ctx.ui).await
                        .map_err(|e| e.to_string())?;
                },
            }
            
            info!("{}", log_messages::ACTION_COMPLETE.replace("{}", "Build"));
        }
        
        Ok(())
    }
}

/// Initialize logging system with the specified log level
pub fn initialize_logging(log_level: LevelFilter) {
    env_logger::builder()
        .filter_level(log_level)
        .format_timestamp(None)
        .format_module_path(false)
        .init();
}

fn main() -> Result<()> {
    // Create tokio runtime
    let runtime = tokio::runtime::Runtime::new()?;
    
    // Run async main function in the runtime
    runtime.block_on(async_main())
}

async fn async_main() -> Result<()> {
    // Parse the command line arguments
    let cli = Cli::parse();
    
    // Set verbosity level based on command line arguments
    let log_level = match cli.verbose {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    
    // Initialize logging
    initialize_logging(log_level);
    
    // Parse locale from command line arguments
    let locale = Locale::from_str(&cli.locale).unwrap_or_else(|_| Locale::default());
    
    // Parse output format
    let output_format = OutputFormat::from_str(&cli.format).unwrap_or(OutputFormat::Default);
    
    info!("Starting application with locale: {}", locale);
    
    // Create ActionLauncher and add layers
    let mut launcher = ActionLauncher::new();
    launcher.add_layer(I18nInitializerLayer::new(&locale.as_str()));
    launcher.add_layer(FileActionLayer);
    launcher.add_layer(DropboxActionLayer);
    launcher.add_layer(BuildActionLayer);
    launcher.add_layer(LicenseActionLayer);
    launcher.add_layer(ObjActionLayer);
    
    // Create launcher context
    let mut ctx = LauncherContext::new("dougu".to_string(), cli.verbose);
    ctx.set_locale(locale.clone());
    ctx.set_output_format(output_format);
    
    // Show application banner unless the user requested to skip it
    if !cli.skip_appinfo {
        display_app_info(&ctx.ui, cli.verbose > 2);
    }
    
    // Process command
    match cli.command {
        Commands::File(file_args) => {
            // Convert args using NotationType::Json
            let serialized = NotationType::Json.encode_to_string(&file_args)
                .map_err(|e| anyhow::anyhow!("Failed to serialize file args: {}", e))?;
            ctx.set_data("file_args", serialized);
            
            // Launch the action
            launcher.launch(&mut ctx).await
                .map_err(|e| anyhow::anyhow!(e))?;
        },
        Commands::Dropbox(dropbox_args) => {
            // Convert args using NotationType::Json
            let serialized = NotationType::Json.encode_to_string(&dropbox_args)
                .map_err(|e| anyhow::anyhow!("Failed to serialize dropbox args: {}", e))?;
            ctx.set_data("dropbox_args", serialized);
            
            // Launch the action
            launcher.launch(&mut ctx).await
                .map_err(|e| anyhow::anyhow!(e))?;
        },
        Commands::Obj(obj_args) => {
            // Convert args using NotationType::Json
            let serialized = NotationType::Json.encode_to_string(&obj_args)
                .unwrap_or_else(|_| "{}".to_string());
                
            ctx.set_data("obj_args", serialized);
            
            // Launch the action
            launcher.launch(&mut ctx).await
                .map_err(|e| anyhow::anyhow!(e))?;
        },
        Commands::Build(build_args) => {
            // Convert args using NotationType::Json
            let serialized = NotationType::Json.encode_to_string(&build_args)
                .map_err(|e| anyhow::anyhow!("Failed to serialize build args: {}", e))?;
            ctx.set_data("build_args", serialized);
            
            // Launch the action
            launcher.launch(&mut ctx).await
                .map_err(|e| anyhow::anyhow!(e))?;
        },
        Commands::Version => {
            ctx.ui.heading(1, "Version Information");
            
            // Use the BuildActionLayer directly
            let build_layer = BuildActionLayer;
            build_layer.run(&mut ctx).await
                .map_err(|e| anyhow::anyhow!(e))?;
        },
        Commands::License => {
            ctx.ui.heading(1, "License Information");
            
            // Use the LicenseActionLayer directly
            let license_layer = LicenseActionLayer;
            ctx.set_data("license_requested", "true".to_string());
            
            // Run the license action
            license_layer.run(&mut ctx).await
                .map_err(|e| anyhow::anyhow!(e))?;
        },
    }
    
    Ok(())
}

