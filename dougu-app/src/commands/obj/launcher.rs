use async_trait::async_trait;
use dougu_foundation_run::{LauncherContext, LauncherLayer};
use serde_json;

use crate::ObjCommand;

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
            let cmd: ObjCommand = serde_json::from_str(args_str)
                .map_err(|e| format!("Failed to parse obj args: {}", e))?;
            
            // Execute the command directly
            cmd.execute().await
                .map_err(|e| format!("Obj command execution failed: {}", e))?;
        }
        
        Ok(())
    }
} 