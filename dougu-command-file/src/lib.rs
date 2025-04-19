use anyhow::Result;
use clap::{Args, Subcommand};
use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use dougu_foundation_run::{Commandlet, CommandletError};
use dougu_essentials_i18n_foundation::{t, tf, vars, I18nCommandletError};

pub mod resources;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

#[derive(Debug, Args, Serialize, Deserialize)]
pub struct FileArgs {
    #[command(subcommand)]
    pub command: FileCommands,
}

#[derive(Debug, Subcommand, Serialize, Deserialize)]
pub enum FileCommands {
    /// Copy files from source to destination
    Copy(CopyArgs),
    
    /// Move files from source to destination
    Move(MoveArgs),
    
    /// List files in a directory
    List(ListArgs),
}

#[derive(Debug, Args, Serialize, Deserialize, Clone)]
pub struct CopyArgs {
    /// Source file path
    pub source: String,
    
    /// Destination file path
    pub destination: String,
    
    /// Overwrite destination if it exists
    #[arg(short, long)]
    pub force: bool,
}

#[derive(Debug, Args, Serialize, Deserialize, Clone)]
pub struct MoveArgs {
    /// Source file path
    pub source: String,
    
    /// Destination file path
    pub destination: String,
    
    /// Overwrite destination if it exists
    #[arg(short, long)]
    pub force: bool,
}

#[derive(Debug, Args, Serialize, Deserialize, Clone)]
pub struct ListArgs {
    /// Directory to list
    pub directory: Option<String>,
    
    /// Show hidden files
    #[arg(short, long)]
    pub all: bool,
    
    /// Use long listing format
    #[arg(short, long)]
    pub long: bool,
}

// Generic result structure for file commands
#[derive(Debug, Serialize, Deserialize)]
pub struct FileCommandResult {
    pub success: bool,
    pub message: String,
    pub details: Option<String>,
}

// File copy commandlet
pub struct FileCopyCommandlet;

#[async_trait]
impl Commandlet for FileCopyCommandlet {
    type Params = CopyArgs;
    type Results = FileCommandResult;
    
    fn name(&self) -> &str {
        "FileCopyCommandlet"
    }
    
    async fn execute(&self, params: Self::Params) -> Result<Self::Results, CommandletError> {
        // Use the i18n system for the log message
        dougu_essentials_logger::log_info(tf("FILE_COPY_START", vars!(
            "source" => &params.source,
            "destination" => &params.destination
        )));
        
        // Pseudo implementation
        // In a real app, this would perform the actual file copy
        
        Ok(FileCommandResult {
            success: true,
            message: tf("FILE_COPY_SUCCESS", vars!(
                "source" => &params.source,
                "destination" => &params.destination
            )),
            details: None,
        })
    }
}

// File move commandlet
pub struct FileMoveCommandlet;

#[async_trait]
impl Commandlet for FileMoveCommandlet {
    type Params = MoveArgs;
    type Results = FileCommandResult;
    
    fn name(&self) -> &str {
        "FileMoveCommandlet"
    }
    
    async fn execute(&self, params: Self::Params) -> Result<Self::Results, CommandletError> {
        // Use the i18n system for the log message
        dougu_essentials_logger::log_info(tf("FILE_MOVE_START", vars!(
            "source" => &params.source,
            "destination" => &params.destination
        )));
        
        // Pseudo implementation
        // In a real app, this would perform the actual file move
        
        Ok(FileCommandResult {
            success: true,
            message: tf("FILE_MOVE_SUCCESS", vars!(
                "source" => &params.source,
                "destination" => &params.destination
            )),
            details: None,
        })
    }
}

// File list commandlet
pub struct FileListCommandlet;

#[async_trait]
impl Commandlet for FileListCommandlet {
    type Params = ListArgs;
    type Results = FileCommandResult;
    
    fn name(&self) -> &str {
        "FileListCommandlet"
    }
    
    async fn execute(&self, params: Self::Params) -> Result<Self::Results, CommandletError> {
        let dir = params.directory.as_deref().unwrap_or(".");
        
        // Use the i18n system for the log message
        dougu_essentials_logger::log_info(tf("FILE_LIST_START", vars!(
            "directory" => dir
        )));
        
        // Pseudo implementation
        // In a real app, this would list the directory contents
        
        Ok(FileCommandResult {
            success: true,
            message: tf("FILE_LIST_SUCCESS", vars!(
                "directory" => dir
            )),
            details: None,
        })
    }
}

// Main file commandlet
pub struct FileCommandlet;

#[async_trait]
impl Commandlet for FileCommandlet {
    type Params = FileArgs;
    type Results = FileCommandResult;
    
    fn name(&self) -> &str {
        "FileCommandlet"
    }
    
    async fn execute(&self, params: Self::Params) -> Result<Self::Results, CommandletError> {
        match &params.command {
            FileCommands::Copy(copy_args) => {
                let commandlet = FileCopyCommandlet;
                commandlet.execute(copy_args.clone()).await
            }
            FileCommands::Move(move_args) => {
                let commandlet = FileMoveCommandlet;
                commandlet.execute(move_args.clone()).await
            }
            FileCommands::List(list_args) => {
                let commandlet = FileListCommandlet;
                commandlet.execute(list_args.clone()).await
            }
        }
    }
}

// Legacy execute functions for backward compatibility
pub fn execute_copy(args: &CopyArgs) -> Result<()> {
    let commandlet = FileCopyCommandlet;
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async {
            commandlet.execute(args.clone()).await.map_err(|e| anyhow::anyhow!(e.to_string()))?;
            Ok(())
        })
}

pub fn execute_move(args: &MoveArgs) -> Result<()> {
    let commandlet = FileMoveCommandlet;
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async {
            commandlet.execute(args.clone()).await.map_err(|e| anyhow::anyhow!(e.to_string()))?;
            Ok(())
        })
}

pub fn execute_list(args: &ListArgs) -> Result<()> {
    let commandlet = FileListCommandlet;
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async {
            commandlet.execute(args.clone()).await.map_err(|e| anyhow::anyhow!(e.to_string()))?;
            Ok(())
        })
}
