use anyhow::Result;
use async_trait::async_trait;
use dougu_essentials::{
    obj::notation::{Notation, json::JsonNotation},
    obj::prelude::*
};
use dougu_foundation::{
    i18n::t,
    run::{Action, ActionError, ActionRunner, LauncherContext, LauncherLayer},
    ui::UIManager
};

// Import messages
use crate::root::resources::messages::*;

// Help action
pub struct HelpAction;

pub struct HelpParams {
    pub command: Option<String>,
}

pub struct HelpResults {
    pub content: String,
}

#[async_trait]
impl Action for HelpAction {
    type Params = HelpParams;
    type Results = HelpResults;
    
    fn name(&self) -> &str {
        "HelpAction"
    }
    
    async fn execute(&self, params: Self::Params) -> Result<Self::Results, ActionError> {
        let content = match params.command.as_deref() {
            Some("file") => t(HELP_COMMAND_FILE),
            Some("dropbox") => t(HELP_COMMAND_DROPBOX),
            Some("obj") => t(HELP_COMMAND_OBJ),
            Some("build") => t(HELP_COMMAND_BUILD),
            Some("version") => t(HELP_COMMAND_VERSION),
            Some("help") => t(HELP_COMMAND_HELP),
            Some("license") => t(HELP_COMMAND_LICENSE),
            Some(cmd) => return Err(ActionError::new(
                "UNKNOWN_COMMAND", 
                &format!("Unknown command: {}", cmd)
            )),
            None => t(HELP_GENERAL)
        };
        
        Ok(HelpResults { content })
    }
}

/// Help action layer for the launcher
pub struct HelpActionLayer;

#[async_trait]
impl LauncherLayer for HelpActionLayer {
    fn name(&self) -> &str {
        "HelpActionLayer"
    }

    async fn run(&self, ctx: &mut LauncherContext) -> Result<(), String> {
        if let Some(args_str) = ctx.get_data("help_args") {
            let json_notation = JsonNotation::new();
            let params: HelpParams = json_notation.decode(args_str.as_bytes())
                .map_err(|e| format!("Failed to parse help args: {}", e))?;
                
            let action = HelpAction;
            let results = action.execute(params)
                .await
                .map_err(|e| format!("Help action failed: {}", e))?;
                
            ctx.ui.info(&results.content);
        }
        Ok(())
    }
} 