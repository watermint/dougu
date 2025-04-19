use anyhow::Result;
use clap::{Parser, Subcommand};
use log::{info, LevelFilter};
use async_trait::async_trait;
use std::str::FromStr;
use serde::{Serialize, Deserialize};

use dougu_command_file::{FileArgs, FileCommandlet};
use dougu_command_file::{FileCopyCommandlet, FileMoveCommandlet, FileListCommandlet};
use dougu_command_dropbox::{DropboxArgs, DropboxCommands, FileCommands as DropboxFileCommands};
use dougu_command_obj::ObjCommand;
use dougu_command_build::BuildArgs;
use dougu_foundation_run::{CommandLauncher, LauncherContext, LauncherLayer, CommandRunner, I18nInitializerLayer};
use dougu_foundation_i18n::Locale;
use dougu_foundation_run::resources::log_messages;
use dougu_foundation_ui::OutputFormat;
use dougu_command_root::{VersionCommandlet, HelpCommandlet, HelpCommandLayer};

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

    /// Set output format (default, json, markdown)
    #[arg(long = "ui-format", value_parser = ["default", "json", "markdown"], default_value = "default", global = true)]
    format: String,

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
            
            // Handle output based on format
            let formatted_result = runner.format_results(&result)
                .map_err(|e| format!("Failed to format results: {}", e))?;
            
            // Print the formatted result
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
            
            // Use UI manager from context directly
            ctx.ui.print(&ctx.ui.heading(1, "Dropbox Operations"));
            
            match &args.command {
                DropboxCommands::File(file_args) => {
                    ctx.ui.print(&ctx.ui.heading(2, "File Operations"));
                    
                    match &file_args.command {
                        DropboxFileCommands::List(list_args) => {
                            info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "File List"));
                            ctx.ui.print(&ctx.ui.info("Listing files from Dropbox..."));
                            
                            dougu_command_dropbox::execute_file_list(list_args, token, &ctx.ui).await
                                .map_err(|e| format!("Dropbox file list failed: {}", e))?;
                            
                            info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "File List"));
                        }
                        DropboxFileCommands::Download(download_args) => {
                            info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "File Download"));
                            // Create a local variable for the formatted message
                            let msg = format!("Downloading file: {}", download_args.path);
                            ctx.ui.print(&ctx.ui.info(&msg));
                            
                            dougu_command_dropbox::execute_file_download(download_args, token).await
                                .map_err(|e| format!("Dropbox file download failed: {}", e))?;
                            
                            ctx.ui.print(&ctx.ui.success("Download completed successfully"));
                            info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "File Download"));
                        }
                        DropboxFileCommands::Upload(upload_args) => {
                            info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "File Upload"));
                            // Create a local variable for the formatted message
                            let msg = format!("Uploading file to: {}", upload_args.dropbox_path);
                            ctx.ui.print(&ctx.ui.info(&msg));
                            
                            dougu_command_dropbox::execute_file_upload(upload_args, token).await
                                .map_err(|e| format!("Dropbox file upload failed: {}", e))?;
                            
                            ctx.ui.print(&ctx.ui.success("Upload completed successfully"));
                            info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "File Upload"));
                        }
                    }
                }
                DropboxCommands::Folder(folder_args) => {
                    ctx.ui.print(&ctx.ui.heading(2, "Folder Operations"));
                    
                    match &folder_args.command {
                        dougu_command_dropbox::FolderCommands::Create(create_args) => {
                            info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "Folder Create"));
                            // Create a local variable for the formatted message
                            let msg = format!("Creating folder: {}", create_args.path);
                            ctx.ui.print(&ctx.ui.info(&msg));
                            
                            dougu_command_dropbox::execute_folder_create(create_args, token).await
                                .map_err(|e| format!("Dropbox folder create failed: {}", e))?;
                            
                            ctx.ui.print(&ctx.ui.success("Folder created successfully"));
                            info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "Folder Create"));
                        }
                        dougu_command_dropbox::FolderCommands::Delete(delete_args) => {
                            info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "Folder Delete"));
                            // Create a local variable for the formatted message
                            let msg = format!("Deleting folder: {}", delete_args.path);
                            ctx.ui.print(&ctx.ui.info(&msg));
                            
                            dougu_command_dropbox::execute_folder_delete(delete_args, token).await
                                .map_err(|e| format!("Dropbox folder delete failed: {}", e))?;
                            
                            ctx.ui.print(&ctx.ui.success("Folder deleted successfully"));
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
            
            // Use UI manager from context directly
            // Only show UI messages for non-JSON output
            if ctx.ui.format() != OutputFormat::Json {
                ctx.ui.print(&ctx.ui.heading(1, "Build Operations"));
            }
            
            match &args.command {
                dougu_command_build::BuildCommands::Package(package_args) => {
                    info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "Package"));
                    if ctx.ui.format() != OutputFormat::Json {
                        ctx.ui.print(&ctx.ui.heading(2, "Packaging Application"));
                        
                        // Get the output directory as a placeholder for package name
                        let package_name = package_args.output_dir.clone().unwrap_or_else(|| "default".to_string());
                        let msg = format!("Creating package in: {}", package_name);
                        ctx.ui.print(&ctx.ui.info(&msg));
                    }
                    
                    dougu_command_build::execute_package(package_args).await
                        .map_err(|e| format!("Build package failed: {}", e))?;
                    
                    if ctx.ui.format() != OutputFormat::Json {
                        ctx.ui.print(&ctx.ui.success("Package created successfully"));
                    }
                    info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "Package"));
                }
                dougu_command_build::BuildCommands::Test(test_args) => {
                    info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "Test"));
                    if ctx.ui.format() != OutputFormat::Json {
                        ctx.ui.print(&ctx.ui.heading(2, "Running Tests"));
                        
                        let test_filter = test_args.filter.clone().unwrap_or_else(|| "all tests".to_string());
                        let msg = format!("Running test suite with filter: {}", test_filter);
                        ctx.ui.print(&ctx.ui.info(&msg));
                    }
                    
                    dougu_command_build::execute_test(test_args, &ctx.ui).await
                        .map_err(|e| format!("Build test failed: {}", e))?;
                    
                    if ctx.ui.format() != OutputFormat::Json {
                        ctx.ui.print(&ctx.ui.success("Tests completed successfully"));
                    }
                    info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "Test"));
                }
                dougu_command_build::BuildCommands::Compile(compile_args) => {
                    info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "Compile"));
                    if ctx.ui.format() != OutputFormat::Json {
                        ctx.ui.print(&ctx.ui.heading(2, "Compiling Project"));
                        
                        let build_type = if compile_args.release { "release" } else { "debug" };
                        let msg = format!("Compiling with build type: {}", build_type);
                        ctx.ui.print(&ctx.ui.info(&msg));
                    }
                    
                    dougu_command_build::execute_compile(compile_args, &ctx.ui).await
                        .map_err(|e| format!("Build compile failed: {}", e))?;
                    
                    if ctx.ui.format() != OutputFormat::Json {
                        ctx.ui.print(&ctx.ui.success("Compilation completed successfully"));
                    }
                    info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "Compile"));
                }
                dougu_command_build::BuildCommands::Pack(pack_args) => {
                    info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "Pack"));
                    if ctx.ui.format() != OutputFormat::Json {
                        ctx.ui.print(&ctx.ui.heading(2, "Creating Archive"));
                        
                        let output_dir = pack_args.output_dir.clone().unwrap_or_else(|| "./target/dist".to_string());
                        let msg = format!("Creating archive in: {}", output_dir);
                        ctx.ui.print(&ctx.ui.info(&msg));
                    }
                    
                    // Pass UI context to execute_pack
                    let result = dougu_command_build::execute_pack(pack_args, &ctx.ui).await
                        .map_err(|e| format!("Build pack failed: {}", e))?;
                    
                    // Print the result directly - it's already formatted by the execute_pack function
                    match ctx.ui.format() {
                        OutputFormat::Json => {
                            // For JSON, just print the raw result without any additional formatting
                            println!("{}", result);
                        },
                        _ => {
                            // For other formats, show success message and the result
                            ctx.ui.print(&ctx.ui.success("Archive created successfully"));
                            ctx.ui.print(&result);
                        }
                    }
                    
                    info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "Pack"));
                }
                dougu_command_build::BuildCommands::Spec(spec_args) => {
                    info!("{}", log_messages::SUBCOMMAND_START.replace("{}", "Spec"));
                    
                    if ctx.ui.format() != OutputFormat::Json {
                        ctx.ui.print(&ctx.ui.heading(2, "Generating Commandlet Specification"));
                        
                        let commandlet_name = spec_args.commandlet_name.as_deref().unwrap_or("all available");
                        let msg = format!("Generating spec for: {}", commandlet_name);
                        ctx.ui.print(&ctx.ui.info(&msg));
                    }
                    
                    // Execute the spec command
                    let result = dougu_command_build::execute_spec(spec_args, &ctx.ui).await
                        .map_err(|e| format!("Build spec failed: {}", e))?;
                    
                    println!("{}", result);
                    
                    info!("{}", log_messages::SUBCOMMAND_COMPLETE.replace("{}", "Spec"));
                }
            }
            
            info!("{}", log_messages::COMMAND_COMPLETE.replace("{}", "Build"));
        }
        
        Ok(())
    }
}

// Replace VersionCommandLayerWithFormat with VersionCommandLayer
struct VersionCommandLayer;

#[async_trait]
impl LauncherLayer for VersionCommandLayer {
    fn name(&self) -> &str {
        "VersionCommandLayer"
    }

    async fn run(&self, ctx: &mut LauncherContext) -> Result<(), String> {
        let commandlet = dougu_command_root::VersionCommandlet;
        let runner = CommandRunner::with_ui(commandlet, ctx.ui.clone());
        let params = dougu_command_root::VersionParams {};
        let serialized_params = serde_json::to_string(&params)
            .map_err(|e| format!("Failed to serialize version params: {}", e))?;
        let result = runner.run(&serialized_params).await
            .map_err(|e| format!("Version command execution failed: {}", e))?;
        
        // Format and print the result
        let formatted_result = runner.format_results(&result)
            .map_err(|e| format!("Failed to format version results: {}", e))?;
        println!("{}", formatted_result);
        
        Ok(())
    }
}

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
    dougu_essentials_logger::init(level)?;
    
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
    
    // Create CommandLauncher
    let mut launcher = CommandLauncher::new();
    
    // Create context with command name and verbosity
    let command_name = match &cli.command {
        Commands::File(_) => "File",
        Commands::Dropbox(_) => "Dropbox",
        Commands::Obj(_) => "Obj",
        Commands::Build(_) => "Build",
        Commands::Version => "Version",
        Commands::Help(_) => "Help",
    };
    
    // Create context with command name, verbosity, locale and output format
    let mut context = LauncherContext::with_ui_format(
        command_name.to_string(), 
        cli.verbose, 
        locale.clone(), 
        output_format
    );
    
    // Add the I18nInitializerLayer as the first layer
    launcher.add_layer(I18nInitializerLayer::with_locale(locale));
    
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
    }
    
    // Execute the command through the launcher
    launcher.launch(&mut context).await
        .map_err(|e| anyhow::anyhow!("Command execution failed: {}", e))?;
    
    Ok(())
}
