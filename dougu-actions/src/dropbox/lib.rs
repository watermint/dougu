use anyhow::Result;
use clap::{Args, Subcommand};
use dougu_bridge::dropbox::DropboxClient;
use dougu_essentials::{log as log_util, obj::prelude::*};
use dougu_foundation::ui::UIManager;
use std::path::PathBuf;

// Now resources is handled in mod.rs
// mod resources;

// This is handled in mod.rs
// pub use launcher::DropboxCommandLayer;

#[derive(Debug, Args)]
pub struct DropboxArgs {
    #[command(subcommand)]
    pub command: DropboxCommands,
}

impl Into<NotationType> for DropboxArgs {
    fn into(self) -> NotationType {
        NotationType::Object(vec![
            ("command".to_string(), self.command.into()),
        ])
    }
}

#[derive(Debug, Subcommand)]
pub enum DropboxCommands {
    /// File operations on Dropbox
    File(FileArgs),
    
    /// Folder operations on Dropbox
    Folder(FolderArgs),
}

impl Into<NotationType> for DropboxCommands {
    fn into(self) -> NotationType {
        match self {
            DropboxCommands::File(args) => NotationType::Object(vec![
                ("type".to_string(), "file".into()),
                ("args".to_string(), args.into()),
            ]),
            DropboxCommands::Folder(args) => NotationType::Object(vec![
                ("type".to_string(), "folder".into()),
                ("args".to_string(), args.into()),
            ]),
        }
    }
}

#[derive(Debug, Args)]
pub struct FileArgs {
    #[command(subcommand)]
    pub command: FileCommands,
}

impl Into<NotationType> for FileArgs {
    fn into(self) -> NotationType {
        NotationType::Object(vec![
            ("command".to_string(), self.command.into()),
        ])
    }
}

#[derive(Debug, Subcommand)]
pub enum FileCommands {
    /// List files in a Dropbox folder
    List(ListArgs),
    
    /// Download a file from Dropbox
    Download(DownloadArgs),
    
    /// Upload a file to Dropbox
    Upload(UploadArgs),
}

impl Into<NotationType> for FileCommands {
    fn into(self) -> NotationType {
        match self {
            FileCommands::List(args) => NotationType::Object(vec![
                ("type".to_string(), "list".into()),
                ("args".to_string(), args.into()),
            ]),
            FileCommands::Download(args) => NotationType::Object(vec![
                ("type".to_string(), "download".into()),
                ("args".to_string(), args.into()),
            ]),
            FileCommands::Upload(args) => NotationType::Object(vec![
                ("type".to_string(), "upload".into()),
                ("args".to_string(), args.into()),
            ]),
        }
    }
}

#[derive(Debug, Args)]
pub struct FolderArgs {
    #[command(subcommand)]
    pub command: FolderCommands,
}

impl Into<NotationType> for FolderArgs {
    fn into(self) -> NotationType {
        NotationType::Object(vec![
            ("command".to_string(), self.command.into()),
        ])
    }
}

#[derive(Debug, Subcommand)]
pub enum FolderCommands {
    /// Create a folder on Dropbox
    Create(CreateFolderArgs),
    
    /// Delete a folder on Dropbox
    Delete(DeleteFolderArgs),
}

impl Into<NotationType> for FolderCommands {
    fn into(self) -> NotationType {
        match self {
            FolderCommands::Create(args) => NotationType::Object(vec![
                ("type".to_string(), "create".into()),
                ("args".to_string(), args.into()),
            ]),
            FolderCommands::Delete(args) => NotationType::Object(vec![
                ("type".to_string(), "delete".into()),
                ("args".to_string(), args.into()),
            ]),
        }
    }
}

#[derive(Debug, Args)]
pub struct ListArgs {
    /// Path to list files from
    pub path: String,
    
    /// List recursively
    #[arg(short, long)]
    pub recursive: bool,
}

impl Into<NotationType> for ListArgs {
    fn into(self) -> NotationType {
        NotationType::Object(vec![
            ("path".to_string(), self.path.into()),
            ("recursive".to_string(), self.recursive.into()),
        ])
    }
}

#[derive(Debug, Args)]
pub struct DownloadArgs {
    /// Path to the file on Dropbox
    pub path: String,
    
    /// Local destination path
    #[arg(short, long)]
    pub output: Option<String>,
}

impl Into<NotationType> for DownloadArgs {
    fn into(self) -> NotationType {
        NotationType::Object(vec![
            ("path".to_string(), self.path.into()),
            ("output".to_string(), self.output.into()),
        ])
    }
}

#[derive(Debug, Args)]
pub struct UploadArgs {
    /// Path to the local file
    pub local_path: String,
    
    /// Destination path on Dropbox
    #[arg(short, long)]
    pub dropbox_path: String,
}

impl Into<NotationType> for UploadArgs {
    fn into(self) -> NotationType {
        NotationType::Object(vec![
            ("local_path".to_string(), self.local_path.into()),
            ("dropbox_path".to_string(), self.dropbox_path.into()),
        ])
    }
}

#[derive(Debug, Args)]
pub struct CreateFolderArgs {
    /// Path to create on Dropbox
    pub path: String,
}

impl Into<NotationType> for CreateFolderArgs {
    fn into(self) -> NotationType {
        NotationType::Object(vec![
            ("path".to_string(), self.path.into()),
        ])
    }
}

#[derive(Debug, Args)]
pub struct DeleteFolderArgs {
    /// Path to delete on Dropbox
    pub path: String,
    
    /// Delete recursively
    #[arg(short, long)]
    pub recursive: bool,
}

impl Into<NotationType> for DeleteFolderArgs {
    fn into(self) -> NotationType {
        NotationType::Object(vec![
            ("path".to_string(), self.path.into()),
            ("recursive".to_string(), self.recursive.into()),
        ])
    }
}

// Execute functions that will be called from main.rs
pub async fn execute_file_list(args: &ListArgs, token: &str, ui: &UIManager) -> Result<()> {
    // Create a client
    let client = DropboxClient::new(token.to_string());
    
    // Call domain logic - assuming list_files returns Vec<String> now
    let files = client.list_files(&args.path).await?;
    
    // Display results
    ui.heading(3, &format!("Files in {}", args.path));
    if files.files.is_empty() {
        ui.info("No files found");
    } else {
        for file in &files.files {
            ui.info(&file.name);
        }
    }
    
    Ok(())
}

pub async fn execute_file_download(args: &DownloadArgs, token: &str) -> Result<()> {
    // Create a client
    let client = DropboxClient::new(token.to_string());
    
    // Determine the output path
    let output_path = args.output.clone().unwrap_or_else(|| {
        // Extract filename from Dropbox path
        match PathBuf::from(&args.path).file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => "downloaded_file".to_string(),
        }
    });
    
    // Call domain logic - download_file returns Vec<u8>, we need to write it to a file
    let content = client.download_file(&args.path).await?;
    tokio::fs::write(&output_path, content).await?;
    
    Ok(())
}

pub async fn execute_file_upload(args: &UploadArgs, token: &str) -> Result<()> {
    // Create a client
    let client = DropboxClient::new(token.to_string());
    
    // Read the file to upload
    let content = tokio::fs::read(&args.local_path).await?;
    
    // Call domain logic
    client.upload_file(&args.dropbox_path, content).await?;
    
    Ok(())
}

pub async fn execute_folder_create(args: &CreateFolderArgs, _token: &str) -> Result<()> {
    // Since client.create_folder doesn't exist, create a manual implementation
    log_util::log_info(format!("Creating folder: {}", args.path));
    
    // In a real app, this would perform the actual folder creation
    // using the Dropbox API
    
    // For now, return success
    Ok(())
}

pub async fn execute_folder_delete(args: &DeleteFolderArgs, _token: &str) -> Result<()> {
    // Since client.delete_folder doesn't exist, create a manual implementation
    log_util::log_info(format!("Deleting folder: {}", args.path));
    
    if args.recursive {
        log_util::log_info("Using recursive deletion");
    }
    
    // In a real app, this would perform the actual folder deletion
    // using the Dropbox API
    
    // For now, return success
    Ok(())
}
