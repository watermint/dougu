use anyhow::Result;
use clap::{Args, Subcommand};
use essentials::Error;
use tracing::info;

#[derive(Subcommand)]
pub enum FileCommands {
    /// Copy files
    Copy(CopyArgs),
    
    /// List files
    List(ListArgs),
}

#[derive(Args)]
pub struct CopyArgs {
    /// Source file path
    #[arg(short, long)]
    source: String,
    
    /// Destination file path
    #[arg(short, long)]
    destination: String,
    
    /// Overwrite if destination exists
    #[arg(short, long)]
    force: bool,
}

#[derive(Args)]
pub struct ListArgs {
    /// Directory path to list
    #[arg(default_value = ".")]
    path: String,
    
    /// Show hidden files
    #[arg(short, long)]
    all: bool,
}

pub fn run(command: FileCommands) -> Result<()> {
    match command {
        FileCommands::Copy(args) => copy_file(args),
        FileCommands::List(args) => list_files(args),
    }
}

fn copy_file(args: CopyArgs) -> Result<()> {
    info!("Copying {} to {}", args.source, args.destination);
    
    if !essentials::utils::file_exists(&args.source) {
        return Err(Error::ResourceNotFound(args.source).into());
    }
    
    if essentials::utils::file_exists(&args.destination) && !args.force {
        return Err(Error::CommandFailed(format!("Destination file already exists. Use --force to overwrite")).into());
    }
    
    std::fs::copy(&args.source, &args.destination)?;
    info!("File copied successfully");
    
    Ok(())
}

fn list_files(args: ListArgs) -> Result<()> {
    info!("Listing files in {}", args.path);
    
    if !essentials::utils::dir_exists(&args.path) {
        return Err(Error::ResourceNotFound(args.path).into());
    }
    
    let entries = std::fs::read_dir(&args.path)?;
    
    for entry in entries {
        let entry = entry?;
        let filename = entry.file_name();
        let name = filename.to_string_lossy();
        
        if !args.all && name.starts_with('.') {
            continue;
        }
        
        println!("{}", name);
    }
    
    Ok(())
} 