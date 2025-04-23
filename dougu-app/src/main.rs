use anyhow::Result;
use clap::{Parser, Subcommand};
use log::{info, LevelFilter};
use async_trait::async_trait;
use std::str::FromStr;
use serde::{Serialize, Deserialize};

// Remove the local actions module
// mod actions;

// Use actions from the dougu-actions crate
use dougu_actions::file::{FileArgs, FileAction};
use dougu_actions::file::{FileCopyAction, FileMoveAction, FileListAction};
use dougu_actions::dropbox::{DropboxArgs, DropboxActions, file::FileActions as DropboxFileActions};
use dougu_actions::obj::ObjAction;
use dougu_actions::build::BuildArgs;
use dougu_foundation_run::{ActionLauncher, LauncherContext, LauncherLayer, ActionRunner, I18nInitializerLayer, display_app_info};
use dougu_foundation_i18n::Locale;
use dougu_foundation_run::resources::log_messages;
use dougu_foundation_ui::OutputFormat;
use dougu_foundation_ui::resources::ui_messages;
use dougu_actions::root::{VersionAction, HelpAction, HelpActionLayer, LicenseActionLayer};

// Keep the i18n module for potential future use
mod i18n;

#[derive(Parser)]
#[command(author, version, about, long_about = None, disable_help_flag=true, disable_version_flag=true, disable_help_subcommand=true)]
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
    Obj(ObjAction),
    
    /// Build operations
    Build(BuildArgs),
    
    /// Show version information
    Version,
    
    /// Display help information
    Help(HelpArgs),
    
    /// Display license information
    License,
}

#[derive(Parser, Serialize, Deserialize)]
pub struct HelpArgs {
    /// Command to get help for
    command: Option<String>,
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
            
            // Create the action with UI manager from context directly
            let action = FileAction;
            let runner = ActionRunner::with_ui(action, ctx.ui.clone());
            
            // Add locale to the args if it's a JSON object
            let args_with_locale = if args_str.trim().starts_with('{') && args_str.trim().ends_with('}') {
                // Parse the JSON, add locale, and reserialize
                if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(args_str) {
                    if let Some(obj) = json.as_object_mut() {
                        // Add context object with locale if it doesn't exist
                        if !obj.contains_key("context") {
                            let mut context = serde_json::Map::new();
                            context.insert("locale".to_string(), serde_json::Value::String(ctx.get_locale().as_str().to_string()));
                            obj.insert("context".to_string(), serde_json::Value::Object(context));
                        } else if let Some(context) = obj.get_mut("context") {
                            // Add locale to existing context
                            if let Some(context_obj) = context.as_object_mut() {
                                context_obj.insert("locale".to_string(), serde_json::Value::String(ctx.get_locale().as_str().to_string()));
                            }
                        }
                        
                        if let Ok(new_json) = serde_json::to_string(obj) {
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
            
            // Run the action with the serialized arguments
            let result = runner.run(&args_with_locale).await
                .map_err(|e| format!("File action execution failed: {}", e))?;
            
            // Format results using ActionRunner
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
            // Parse the serialized args
            let args: DropboxArgs = serde_json::from_str(args_str)
                .map_err(|e| format!("Failed to parse dropbox args: {}", e))?;
            
            // For demo purposes, use a dummy token
            let token = "dummy_dropbox_token";
            
            info!("{}", log_messages::ACTION_START.replace("{}", "Dropbox"));
            
            // Use UI manager from context directly
            ctx.ui.heading(1, "Dropbox Operations");
            
            match &args.command {
                DropboxActions::File(file_args) => {
                    ctx.ui.heading(2, "File Operations");
                    
                    match &file_args.command {
                        DropboxFileActions::List(list_args) => {
                            info!("{}", log_messages::SUBACTION_START.replace("{}", "File List"));
                            ctx.ui.info("Listing files from Dropbox...");
                            
                            dougu_actions::dropbox::execute_file_list(list_args, token, &ctx.ui).await
                                .map_err(|e| format!("Dropbox file list failed: {}", e))?;
                            
                            info!("{}", log_messages::SUBACTION_COMPLETE.replace("{}", "File List"));
                        }
                        DropboxFileActions::Download(download_args) => {
                            info!("{}", log_messages::SUBACTION_START.replace("{}", "File Download"));
                            // Create a local variable for the formatted message
                            let msg = format!("Downloading file: {}", download_args.path);
                            ctx.ui.info(&msg);
                            
                            dougu_actions::dropbox::execute_file_download(download_args, token).await
                                .map_err(|e| format!("Dropbox file download failed: {}", e))?;
                            
                            ctx.ui.success("Download completed successfully");
                            info!("{}", log_messages::SUBACTION_COMPLETE.replace("{}", "File Download"));
                        }
                        DropboxFileActions::Upload(upload_args) => {
                            info!("{}", log_messages::SUBACTION_START.replace("{}", "File Upload"));
                            // Create a local variable for the formatted message
                            let msg = format!("Uploading file to: {}", upload_args.dropbox_path);
                            ctx.ui.info(&msg);
                            
                            dougu_actions::dropbox::execute_file_upload(upload_args, token).await
                                .map_err(|e| format!("Dropbox file upload failed: {}", e))?;
                            
                            ctx.ui.success("Upload completed successfully");
                            info!("{}", log_messages::SUBACTION_COMPLETE.replace("{}", "File Upload"));
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

#[async_trait]
impl dougu_actions::obj::layer::ActionLayer for ObjActionLayer {
    fn name(&self) -> &str {
        "ObjActionLayer"
    }
    
    fn handle_action(&self, action: &str, args: Vec<String>) -> Result<(), String> {
        match action {
            "convert" => {
                // Handle convert action with provided arguments
                if args.len() < 3 {
                    return Err("Not enough arguments for convert action".to_string());
                }
                
                dougu_actions::obj::convert(&args[0], &args[1], &args[2])
                    .map_err(|e| format!("Convert failed: {}", e))
            }
            _ => Err(format!("Unknown action: {}", action))
        }
    }
    
    fn is_empty(&self, result: &str) -> bool {
        result.is_empty() || result == "{}" || result == "[]"
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
            // Parse the serialized args
            let args: BuildArgs = serde_json::from_str(args_str)
                .map_err(|e| format!("Failed to parse build args: {}", e))?;
            
            info!("{}", log_messages::ACTION_START.replace("{}", "Build"));
            
            // Use UI manager from context directly
            ctx.ui.heading(1, "Build Operations");
            
            // Execute build operations
            dougu_actions::build::execute_build(&args, &ctx.ui)
                .map_err(|e| format!("Build failed: {}", e))?;
            
            info!("{}", log_messages::ACTION_COMPLETE.replace("{}", "Build"));
        }
        
        Ok(())
    }
}

async fn main() -> Result<()> {
    // Initialize logger with environment variables
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .parse_env("DOUGU_LOG")
        .init();
    
    info!("{}", log_messages::LAUNCHER_START);
    
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Parse locale from command line
    let locale = Locale::from_str(&cli.locale).unwrap_or_else(|_| {
        info!("Invalid locale specified, using default (en)");
        Locale::default()
    });
    
    // Parse output format from command line
    let format = match cli.format.as_str() {
        "jsonl" => OutputFormat::JsonLines,
        "markdown" => OutputFormat::Markdown,
        _ => OutputFormat::Default,
    };
    
    // Create launcher context
    let mut ctx = LauncherContext::with_ui_format("dougu".to_string(), cli.verbose, locale, format);
    
    // Display application info if not skipped
    if !cli.skip_appinfo {
        display_app_info(&ctx.ui);
    }
    
    // Process command and arguments
    match &cli.command {
        Commands::File(args) => {
            // Store serialized arguments in context
            let serialized = serde_json::to_string(args).map_err(|e| {
                format!("Failed to serialize file args: {}", e)
            })?;
            ctx.set_data("file_args", serialized);
            
            // Create and run the file action layer
            let file_layer = FileActionLayer;
            file_layer.run(&mut ctx).await?;
        }
        Commands::Dropbox(args) => {
            // Store serialized arguments in context
            let serialized = serde_json::to_string(args).map_err(|e| {
                format!("Failed to serialize dropbox args: {}", e)
            })?;
            ctx.set_data("dropbox_args", serialized);
            
            // Create and run the dropbox action layer
            let dropbox_layer = DropboxActionLayer;
            dropbox_layer.run(&mut ctx).await?;
        }
        Commands::Obj(args) => {
            // Create obj action layer
            let obj_layer = ObjActionLayer;
            
            // Handle obj actions
            match &args.command {
                dougu_actions::obj::ObjCommands::Convert(convert_args) => {
                    obj_layer.handle_action("convert", vec![
                        convert_args.input.clone(),
                        convert_args.output.clone(),
                        convert_args.format.clone(),
                    ])?;
                }
            }
        }
        Commands::Build(args) => {
            // Store serialized arguments in context
            let serialized = serde_json::to_string(args).map_err(|e| {
                format!("Failed to serialize build args: {}", e)
            })?;
            ctx.set_data("build_args", serialized);
            
            // Create and run the build action layer
            let build_layer = BuildActionLayer;
            build_layer.run(&mut ctx).await?;
        }
        Commands::Version => {
            // Create version action layer
            let version_layer = I18nInitializerLayer::wrap(dougu_actions::root::VersionActionLayer);
            
            // Create action launcher and add layers
            let mut launcher = ActionLauncher::new();
            launcher.add_layer(version_layer);
            
            // Launch the action
            launcher.launch(&mut ctx).await?;
        }
        Commands::Help(args) => {
            // Store command to get help for
            if let Some(command) = &args.command {
                ctx.set_data("help_command", command.to_string());
            }
            
            // Create help action layer
            let help_layer = I18nInitializerLayer::wrap(HelpActionLayer);
            
            // Create action launcher and add layers
            let mut launcher = ActionLauncher::new();
            launcher.add_layer(help_layer);
            
            // Launch the action
            launcher.launch(&mut ctx).await?;
        }
        Commands::License => {
            // Create license action layer
            let license_layer = I18nInitializerLayer::wrap(LicenseActionLayer);
            
            // Create action launcher and add layers
            let mut launcher = ActionLauncher::new();
            launcher.add_layer(license_layer);
            
            // Launch the action
            launcher.launch(&mut ctx).await?;
        }
    }
    
    info!("{}", log_messages::LAUNCHER_COMPLETE);
    
    Ok(())
}

