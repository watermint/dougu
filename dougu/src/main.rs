use anyhow::Result;
use clap::{Parser, Subcommand};
use log::{info, LevelFilter};
use async_trait::async_trait;
use std::str::FromStr;
use serde::{Serialize, Deserialize};

// Add the commands module
mod commands;

// Use commands from our local modules instead of external crates
use crate::commands::file::{FileArgs, FileCommandlet};
use crate::commands::file::{FileCopyCommandlet, FileMoveCommandlet, FileListCommandlet};
use crate::commands::dropbox::{DropboxArgs, DropboxCommands, FileCommands as DropboxFileCommands};
use crate::commands::obj::ObjCommand;
use crate::commands::build::BuildArgs;
use dougu_foundation_run::{CommandLauncher, LauncherContext, LauncherLayer, CommandRunner, I18nInitializerLayer, display_app_info};
use dougu_foundation_i18n::Locale;
use dougu_foundation_run::resources::log_messages;
use dougu_foundation_ui::OutputFormat;
use dougu_foundation_ui::resources::ui_messages;
use crate::commands::root::{VersionCommandlet, HelpCommandlet, HelpCommandLayer, LicenseCommandLayer};

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
    Obj(ObjCommand),
    
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
            
            // Create the commandlet with UI manager from context directly
            let commandlet = FileCommandlet;
            let runner = CommandRunner::with_ui(commandlet, ctx.ui.clone());
            
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
            
            // Run the commandlet with the serialized arguments
            let result = runner.run(&args_with_locale).await
                .map_err(|e| format!("File command execution failed: {}", e))?;
            
            // Format results using CommandRunner
            runner.format_results(&result)
                .map_err(|e| format!("Failed to format results: {}", e))?;
            
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
            
            // Use UI manager from context directly
            ctx.ui.heading(1, "Dropbox Operations");
            
            match &args.command {
                DropboxCommands::File(file_args) => {
                    ctx.ui.heading(2, "File Operations");
                    
                    match &file_args.command {
                        DropboxFileCommands::List(list_args) => {
                            info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "File List"));
                            ctx.ui.info("Listing files from Dropbox...");
                            
                            crate::commands::dropbox::execute_file_list(list_args, token, &ctx.ui).await
                                .map_err(|e| format!("Dropbox file list failed: {}", e))?;
                            
                            info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "File List"));
                        }
                        DropboxFileCommands::Download(download_args) => {
                            info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "File Download"));
                            // Create a local variable for the formatted message
                            let msg = format!("Downloading file: {}", download_args.path);
                            ctx.ui.info(&msg);
                            
                            crate::commands::dropbox::execute_file_download(download_args, token).await
                                .map_err(|e| format!("Dropbox file download failed: {}", e))?;
                            
                            ctx.ui.success("Download completed successfully");
                            info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "File Download"));
                        }
                        DropboxFileCommands::Upload(upload_args) => {
                            info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "File Upload"));
                            // Create a local variable for the formatted message
                            let msg = format!("Uploading file to: {}", upload_args.dropbox_path);
                            ctx.ui.info(&msg);
                            
                            crate::commands::dropbox::execute_file_upload(upload_args, token).await
                                .map_err(|e| format!("Dropbox file upload failed: {}", e))?;
                            
                            ctx.ui.success("Upload completed successfully");
                            info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "File Upload"));
                        }
                    }
                }
                DropboxCommands::Folder(folder_args) => {
                    ctx.ui.heading(2, "Folder Operations");
                    
                    match &folder_args.command {
                        crate::commands::dropbox::FolderCommands::Create(create_args) => {
                            info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "Folder Create"));
                            // Create a local variable for the formatted message
                            let msg = format!("Creating folder: {}", create_args.path);
                            ctx.ui.info(&msg);
                            
                            crate::commands::dropbox::execute_folder_create(create_args, token).await
                                .map_err(|e| format!("Dropbox folder create failed: {}", e))?;
                            
                            ctx.ui.success("Folder created successfully");
                            info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "Folder Create"));
                        }
                        crate::commands::dropbox::FolderCommands::Delete(delete_args) => {
                            info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "Folder Delete"));
                            // Create a local variable for the formatted message
                            let msg = format!("Deleting folder: {}", delete_args.path);
                            ctx.ui.info(&msg);
                            
                            crate::commands::dropbox::execute_folder_delete(delete_args, token).await
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

// Object command layer
struct ObjCommandLayer;

impl crate::commands::obj::layer::CommandLayer for ObjCommandLayer {
    fn handle_command(&self, command: &str, args: Vec<String>) -> Result<(), String> {
        let cmd = commands::obj::lib::ObjCommand::from_args(command, args);
        let result = commands::obj::lib::execute_command(&cmd)?;
        
        if let Some(output) = result {
            if !output.is_empty() {
                println!("{}", output);
            }
        }
        
        Ok(())
    }

    fn is_empty(&self, result: &str) -> bool {
        result.is_empty() || result == "null" || result == "{}"
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
        let args_str = match ctx.get_data("build_args") {
            Some(args) => args,
            None => return Ok(()),
        };
        
        // Use the build command layer from the build module
        let layer = crate::commands::build::BuildCommandLayer;
        layer.run(ctx).await
    }
}

// Use the VersionCommandLayer from the root module
use crate::commands::root::VersionCommandLayer;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Parse the output format
    let output_format = OutputFormat::from_str(&cli.format)
        .unwrap_or(OutputFormat::Default);
    
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
    dougu_essentials_log::init(level)?;
    
    // Determine locale from command-line or environment variable
    let locale = if cli.locale != "en" {
        // If explicitly set on command-line, use that
        info!("{}", log_messages::SETTING_LANGUAGE.replace("{}", &cli.locale));
        Locale::from_str(&cli.locale).unwrap_or_else(|_| Locale::default())
    } else {
        // Otherwise check LANG environment variable
        match std::env::var("LANG") {
            Ok(lang) => {
                info!("LANG environment variable: {}", lang);
                // Extract language tag from LANG (e.g., "en" from "en_US.UTF-8")
                let lang_tag = lang.split('.').next().unwrap_or("en").replace('_', "-");
                info!("Extracted language tag: {}", lang_tag);
                
                // Parse the language tag as a Locale
                let extracted_locale = Locale::from_str(&lang_tag).unwrap_or_else(|_| Locale::default());
                
                // Only use if we have translations for this language
                if dougu_foundation_i18n::is_supported_language(&extracted_locale) {
                    info!("{}", log_messages::SETTING_LANGUAGE.replace("{}", extracted_locale.as_str()));
                    extracted_locale
                } else {
                    let msg = log_messages::LANGUAGE_UNSUPPORTED
                        .replace("{}", extracted_locale.language())
                        .replace("{}", "en");
                    info!("{}", msg);
                    Locale::default() // Default to English for unsupported languages
                }
            },
            Err(_) => {
                info!("{}", log_messages::USING_DEFAULT_LANGUAGE.replace("{}", "en"));
                Locale::default() // Default to English if LANG is not set
            }
        }
    };
    
    // Create context with command name, verbosity, locale and output format
    let command_name = match &cli.command {
        Commands::File(_) => "File",
        Commands::Dropbox(_) => "Dropbox",
        Commands::Obj(_) => "Obj",
        Commands::Build(_) => "Build",
        Commands::Version => "Version",
        Commands::Help(_) => "Help",
        Commands::License => "License",
    };
    
    let mut context = LauncherContext::with_ui_format(
        command_name.to_string(), 
        cli.verbose, 
        locale.clone(), 
        output_format
    );
    
    // Create CommandLauncher
    let mut launcher = CommandLauncher::new();
    
    // Add the I18nInitializerLayer as the first layer and initialize it first
    // to ensure translations are loaded before we display app info
    let i18n_layer = I18nInitializerLayer::with_locale(locale.clone());
    let mut i18n_context = LauncherContext::with_ui_format(
        "i18n_init".to_string(),
        cli.verbose,
        locale.clone(),
        output_format
    );
    i18n_layer.run(&mut i18n_context).await
        .map_err(|e| anyhow::anyhow!("Failed to initialize i18n: {}", e))?;
    launcher.add_layer(i18n_layer);
    
    // Now display application information banner with i18n initialized
    if !cli.skip_appinfo {
        display_app_info(&context.ui, cli.verbose >= 2);
    }
    
    // Add appropriate command layers based on the command
    match &cli.command {
        Commands::File(args) => {
            let args_json = serde_json::to_string(args)
                .map_err(|e| anyhow::anyhow!("Failed to serialize file args: {}", e))?;
            context.set_data("file_args", args_json);
            launcher.add_layer(FileCommandletLayer);
        }
        Commands::Dropbox(args) => {
            let args_json = serde_json::to_string(args)
                .map_err(|e| anyhow::anyhow!("Failed to serialize dropbox args: {}", e))?;
            context.set_data("dropbox_args", args_json);
            launcher.add_layer(DropboxCommandLayer);
        }
        Commands::Obj(cmd) => {
            let args_json = serde_json::to_string(cmd)
                .map_err(|e| anyhow::anyhow!("Failed to serialize obj args: {}", e))?;
            context.set_data("obj_args", args_json);
            launcher.add_layer(ObjCommandLayer);
        }
        Commands::Build(args) => {
            let args_json = serde_json::to_string(args)
                .map_err(|e| anyhow::anyhow!("Failed to serialize build args: {}", e))?;
            context.set_data("build_args", args_json);
            launcher.add_layer(BuildCommandLayer);
        }
        Commands::Version => {
            launcher.add_layer(VersionCommandLayer);
        }
        Commands::Help(help_args) => {
            if let Some(cmd) = &help_args.command {
                context.set_data("help_command", cmd.clone());
            }
            launcher.add_layer(HelpCommandLayer);
        }
        Commands::License => {
            launcher.add_layer(LicenseCommandLayer);
        }
    }
    
    // Execute the command through the launcher
    launcher.launch(&mut context).await
        .map_err(|e| anyhow::anyhow!("Command execution failed: {}", e))?;
    
    Ok(())
}
