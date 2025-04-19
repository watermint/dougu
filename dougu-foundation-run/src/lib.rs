pub mod resources;

use resources::error_messages;
use resources::log_messages;
use log::{debug, info, error};
use async_trait::async_trait;

#[async_trait]
pub trait LauncherLayer {
    fn name(&self) -> &str;
    async fn run(&self, ctx: &mut LauncherContext) -> Result<(), String>;
}

pub struct LauncherContext {
    // Store contextual information for command execution
    pub command_name: String,
    pub verbosity: u8,
    pub data: std::collections::HashMap<String, String>,
}

impl LauncherContext {
    pub fn new(command_name: String, verbosity: u8) -> Self {
        Self {
            command_name,
            verbosity,
            data: std::collections::HashMap::new(),
        }
    }

    pub fn set_data(&mut self, key: &str, value: String) {
        self.data.insert(key.to_string(), value);
    }

    pub fn get_data(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }
}

pub struct CommandLauncher {
    layers: Vec<Box<dyn LauncherLayer>>,
}

impl CommandLauncher {
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    pub fn add_layer<L: LauncherLayer + 'static>(&mut self, layer: L) {
        self.layers.push(Box::new(layer));
    }

    pub async fn launch(&self, ctx: &mut LauncherContext) -> Result<(), String> {
        info!("{}", log_messages::LAUNCHER_START);
        
        for layer in &self.layers {
            debug!("{}", format!("{}", log_messages::LAYER_EXECUTION).replace("{}", layer.name()));
            layer.run(ctx).await?;
        }
        
        info!("{}", log_messages::LAUNCHER_COMPLETE);
        Ok(())
    }
}

// Example error abort if resource not found
pub fn abort_if_resource_missing(resource: Option<&str>) -> Result<(), String> {
    if resource.is_none() {
        error!("{}", error_messages::RESOURCE_NOT_FOUND);
        return Err(error_messages::RESOURCE_NOT_FOUND.to_string());
    }
    Ok(())
} 