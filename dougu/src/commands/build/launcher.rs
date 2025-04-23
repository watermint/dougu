use anyhow::Result;
use async_trait::async_trait;
use dougu_foundation_run::{LauncherContext, LauncherLayer};
use dougu_foundation_ui::{UIManager, OutputFormat};
use dougu_essentials_log::log_info;
use serde_json;

use crate::commands::build::{
    BuildArgs, BuildCommands, execute_package, execute_test, 
    execute_compile, execute_pack, execute_spec
};
use crate::commands::build::resources::log_messages;

/// Build command layer for the launcher
pub struct BuildCommandLayer;

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
                
            // Only show UI messages for non-JSON output
            if ctx.ui.format() != OutputFormat::JsonLines {
                ctx.ui.heading(1, "Build Operations");
            }
            
            match &args.command {
                BuildCommands::Package(package_args) => {
                    if ctx.ui.format() != OutputFormat::JsonLines {
                        ctx.ui.heading(2, "Packaging Application");
                        
                        let target = package_args.target.as_deref().unwrap_or("default");
                        let output_dir = package_args.output_dir.as_deref().unwrap_or("dist");
                        let build_mode = if package_args.release { "release" } else { "debug" };
                        
                        let msg = format!("Packaging for target: {} ({}) to {}", target, build_mode, output_dir);
                        ctx.ui.info(&msg);
                    }
                    
                    execute_package(package_args).await
                        .map_err(|e| format!("Build package failed: {}", e))?;
                    
                    if ctx.ui.format() != OutputFormat::JsonLines {
                        ctx.ui.success("Packaging completed successfully");
                    }
                }
                BuildCommands::Test(test_args) => {
                    if ctx.ui.format() != OutputFormat::JsonLines {
                        ctx.ui.heading(2, "Running Tests");
                        
                        let build_mode = if test_args.release { "release" } else { "debug" };
                        let test_type = if test_args.unit {
                            "unit tests"
                        } else if test_args.integration {
                            "integration tests"
                        } else {
                            "all tests"
                        };
                        
                        let filter = test_args.filter.as_deref().unwrap_or("all");
                        let msg = format!("Running {} ({}) with filter: {}", test_type, build_mode, filter);
                        ctx.ui.info(&msg);
                    }
                    
                    execute_test(test_args, &ctx.ui).await
                        .map_err(|e| format!("Build test failed: {}", e))?;
                    
                    if ctx.ui.format() != OutputFormat::JsonLines {
                        ctx.ui.success("Tests completed successfully");
                    }
                }
                BuildCommands::Compile(compile_args) => {
                    if ctx.ui.format() != OutputFormat::JsonLines {
                        ctx.ui.heading(2, "Compiling Application");
                        
                        let output_dir = compile_args.output_dir.as_deref().unwrap_or("target");
                        let build_mode = if compile_args.release { "release" } else { "debug" };
                        
                        let msg = format!("Compiling in {} mode to {}", build_mode, output_dir);
                        ctx.ui.info(&msg);
                    }
                    
                    execute_compile(compile_args, &ctx.ui).await
                        .map_err(|e| format!("Build compile failed: {}", e))?;
                    
                    if ctx.ui.format() != OutputFormat::JsonLines {
                        ctx.ui.success("Compilation completed successfully");
                    }
                }
                BuildCommands::Pack(pack_args) => {
                    if ctx.ui.format() != OutputFormat::JsonLines {
                        ctx.ui.heading(2, "Creating Archive");
                        
                        let name = pack_args.name.as_deref().unwrap_or("app");
                        let platform = pack_args.platform.as_deref().unwrap_or("any");
                        let input_dir = pack_args.input_dir.as_deref().unwrap_or("target/release");
                        
                        let msg = format!("Creating archive for {} on {} from {}", name, platform, input_dir);
                        ctx.ui.info(&msg);
                    }
                    
                    let output = execute_pack(pack_args, &ctx.ui).await
                        .map_err(|e| format!("Build pack failed: {}", e))?;
                    
                    // Output is now a formatted string, not a PackOutput struct
                    if ctx.ui.format() != OutputFormat::JsonLines {
                        ctx.ui.success(&format!("Archive created successfully: {}", output));
                    }
                }
                BuildCommands::Spec(spec_args) => {
                    if ctx.ui.format() != OutputFormat::JsonLines {
                        ctx.ui.heading(2, "Generating Commandlet Specification");
                        
                        let commandlet_name = spec_args.commandlet_name.as_deref().unwrap_or("all available");
                        let msg = format!("Generating spec for: {}", commandlet_name);
                        ctx.ui.info(&msg);
                    }
                    
                    // Execute the spec command
                    let result = execute_spec(spec_args, &ctx.ui).await
                        .map_err(|e| format!("Build spec failed: {}", e))?;
                    
                    ctx.ui.text(&result);
                }
            }
        }
        
        Ok(())
    }
} 