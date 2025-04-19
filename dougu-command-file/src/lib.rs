use anyhow::Result;
use clap::{Args, Subcommand};
use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use dougu_foundation_run::{Commandlet, CommandletError, CommandletSpec, SpecField, SpecError};
use dougu_foundation_i18n::{tf, vars};

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
    
    fn generate_spec(&self) -> CommandletSpec {
        CommandletSpec {
            name: self.name().to_string(),
            description: Some("Copies a file from source to destination".to_string()),
            behavior: "Copies a file from the specified source path to the destination path".to_string(),
            options: vec![
                SpecField {
                    name: "force".to_string(),
                    description: Some("Whether to overwrite destination if it exists".to_string()),
                    field_type: "boolean".to_string(),
                    required: false,
                    default_value: Some("false".to_string()),
                },
            ],
            parameters: vec![
                SpecField {
                    name: "source".to_string(),
                    description: Some("Path to the source file".to_string()),
                    field_type: "string".to_string(),
                    required: true,
                    default_value: None,
                },
                SpecField {
                    name: "destination".to_string(),
                    description: Some("Path to the destination file".to_string()),
                    field_type: "string".to_string(),
                    required: true,
                    default_value: None,
                },
            ],
            result_types: vec![
                SpecField {
                    name: "success".to_string(),
                    description: Some("Whether the copy operation was successful".to_string()),
                    field_type: "boolean".to_string(),
                    required: true,
                    default_value: None,
                },
                SpecField {
                    name: "message".to_string(),
                    description: Some("A human-readable message about the copy operation result".to_string()),
                    field_type: "string".to_string(),
                    required: true,
                    default_value: None,
                },
                SpecField {
                    name: "details".to_string(),
                    description: Some("Additional details about the copy operation".to_string()),
                    field_type: "string".to_string(),
                    required: false,
                    default_value: None,
                },
            ],
            errors: vec![
                SpecError {
                    code: "FILE_NOT_FOUND".to_string(),
                    description: "The source file was not found".to_string(),
                },
                SpecError {
                    code: "ACCESS_DENIED".to_string(),
                    description: "Access to the source or destination file was denied".to_string(),
                },
                SpecError {
                    code: "ALREADY_EXISTS".to_string(),
                    description: "The destination file already exists and force option not specified".to_string(),
                },
                SpecError {
                    code: "INVALID_PATH".to_string(),
                    description: "The source or destination path is invalid".to_string(),
                },
            ],
        }
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
    
    fn generate_spec(&self) -> CommandletSpec {
        CommandletSpec {
            name: self.name().to_string(),
            description: Some("Moves a file from source to destination".to_string()),
            behavior: "Moves a file from the specified source path to the destination path".to_string(),
            options: vec![
                SpecField {
                    name: "force".to_string(),
                    description: Some("Whether to overwrite destination if it exists".to_string()),
                    field_type: "boolean".to_string(),
                    required: false,
                    default_value: Some("false".to_string()),
                },
            ],
            parameters: vec![
                SpecField {
                    name: "source".to_string(),
                    description: Some("Path to the source file".to_string()),
                    field_type: "string".to_string(),
                    required: true,
                    default_value: None,
                },
                SpecField {
                    name: "destination".to_string(),
                    description: Some("Path to the destination file".to_string()),
                    field_type: "string".to_string(),
                    required: true,
                    default_value: None,
                },
            ],
            result_types: vec![
                SpecField {
                    name: "success".to_string(),
                    description: Some("Whether the move operation was successful".to_string()),
                    field_type: "boolean".to_string(),
                    required: true,
                    default_value: None,
                },
                SpecField {
                    name: "message".to_string(),
                    description: Some("A human-readable message about the move operation result".to_string()),
                    field_type: "string".to_string(),
                    required: true,
                    default_value: None,
                },
                SpecField {
                    name: "details".to_string(),
                    description: Some("Additional details about the move operation".to_string()),
                    field_type: "string".to_string(),
                    required: false,
                    default_value: None,
                },
            ],
            errors: vec![
                SpecError {
                    code: "FILE_NOT_FOUND".to_string(),
                    description: "The source file was not found".to_string(),
                },
                SpecError {
                    code: "ACCESS_DENIED".to_string(),
                    description: "Access to the source or destination file was denied".to_string(),
                },
                SpecError {
                    code: "ALREADY_EXISTS".to_string(),
                    description: "The destination file already exists and force option not specified".to_string(),
                },
                SpecError {
                    code: "INVALID_PATH".to_string(),
                    description: "The source or destination path is invalid".to_string(),
                },
            ],
        }
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
    
    fn generate_spec(&self) -> CommandletSpec {
        CommandletSpec {
            name: self.name().to_string(),
            description: Some("Lists files in a directory".to_string()),
            behavior: "Lists files in the specified directory with optional formatting".to_string(),
            options: vec![
                SpecField {
                    name: "all".to_string(),
                    description: Some("Whether to show hidden files".to_string()),
                    field_type: "boolean".to_string(),
                    required: false,
                    default_value: Some("false".to_string()),
                },
                SpecField {
                    name: "long".to_string(),
                    description: Some("Whether to use long listing format".to_string()),
                    field_type: "boolean".to_string(),
                    required: false,
                    default_value: Some("false".to_string()),
                },
            ],
            parameters: vec![
                SpecField {
                    name: "directory".to_string(),
                    description: Some("Path to the directory to list".to_string()),
                    field_type: "string".to_string(),
                    required: false,
                    default_value: Some(".".to_string()),
                },
            ],
            result_types: vec![
                SpecField {
                    name: "success".to_string(),
                    description: Some("Whether the list operation was successful".to_string()),
                    field_type: "boolean".to_string(),
                    required: true,
                    default_value: None,
                },
                SpecField {
                    name: "message".to_string(),
                    description: Some("A human-readable message about the list operation result".to_string()),
                    field_type: "string".to_string(),
                    required: true,
                    default_value: None,
                },
                SpecField {
                    name: "details".to_string(),
                    description: Some("Additional details about the list operation, including file list".to_string()),
                    field_type: "string".to_string(),
                    required: false,
                    default_value: None,
                },
            ],
            errors: vec![
                SpecError {
                    code: "DIRECTORY_NOT_FOUND".to_string(),
                    description: "The specified directory was not found".to_string(),
                },
                SpecError {
                    code: "ACCESS_DENIED".to_string(),
                    description: "Access to the directory was denied".to_string(),
                },
                SpecError {
                    code: "INVALID_PATH".to_string(),
                    description: "The specified path is invalid or not a directory".to_string(),
                },
            ],
        }
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
    
    fn generate_spec(&self) -> CommandletSpec {
        CommandletSpec {
            name: self.name().to_string(),
            description: Some("Performs file operations like copy, move, and list".to_string()),
            behavior: "Delegates to sub-commandlets based on the operation requested".to_string(),
            options: Vec::new(),
            parameters: vec![
                SpecField {
                    name: "command".to_string(),
                    description: Some("The file operation to perform".to_string()),
                    field_type: "FileCommands enum".to_string(),
                    required: true,
                    default_value: None,
                },
            ],
            result_types: vec![
                SpecField {
                    name: "success".to_string(),
                    description: Some("Whether the operation was successful".to_string()),
                    field_type: "boolean".to_string(),
                    required: true,
                    default_value: None,
                },
                SpecField {
                    name: "message".to_string(),
                    description: Some("A human-readable message about the operation result".to_string()),
                    field_type: "string".to_string(),
                    required: true,
                    default_value: None,
                },
                SpecField {
                    name: "details".to_string(),
                    description: Some("Additional details about the operation result".to_string()),
                    field_type: "string".to_string(),
                    required: false,
                    default_value: None,
                },
            ],
            errors: vec![
                SpecError {
                    code: "FILE_NOT_FOUND".to_string(),
                    description: "The specified file was not found".to_string(),
                },
                SpecError {
                    code: "ACCESS_DENIED".to_string(),
                    description: "Access to the file was denied".to_string(),
                },
                SpecError {
                    code: "ALREADY_EXISTS".to_string(),
                    description: "The destination file already exists and force option not specified".to_string(),
                },
                SpecError {
                    code: "INVALID_PATH".to_string(),
                    description: "The specified path is invalid".to_string(),
                },
            ],
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
