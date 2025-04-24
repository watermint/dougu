use async_trait::async_trait;
use dougu_foundation::run::{LauncherContext, LauncherLayer};
use dougu_essentials::obj::NotationType;

use crate::obj::ObjCommand;

/// Object command layer for the launcher
pub struct ObjCommandLayer;

#[async_trait]
impl LauncherLayer for ObjCommandLayer {
    fn name(&self) -> &str {
        "ObjCommandLayer"
    }

    async fn run(&self, ctx: &mut LauncherContext) -> Result<(), String> {
        if let Some(args_str) = ctx.get_data("obj_args") {
            // Parse the serialized args
            let args_value = NotationType::Json.decode::<NotationType>(args_str.as_bytes())
                .map_err(|e| format!("Failed to parse obj args: {}", e))?;
            
            // Convert to ObjCommand
            let cmd = ObjCommand::from(args_value)
                .map_err(|e| format!("Failed to convert args to command: {}", e))?;
            
            // Execute the command directly
            cmd.execute().await
                .map_err(|e| format!("Obj command execution failed: {}", e))?;
        }
        
        Ok(())
    }
} 