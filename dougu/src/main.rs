use anyhow::Result;
use clap::{Parser, Subcommand};
use log::LevelFilter;

use dougu_command_file::{FileArgs, FileCommands};
use dougu_command_dropbox::{DropboxArgs, DropboxCommands, FileCommands as DropboxFileCommands};
use dougu_command_obj::ObjCommand;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Set verbosity level (0-5)
    #[arg(short, long, default_value_t = 2)]
    verbose: u8,

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
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Set up logging based on verbosity
    let level = match cli.verbose {
        0 => LevelFilter::Off,
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        4 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    
    dougu_essentials_logger::init(level)?;
    
    // Process commands
    match cli.command {
        Commands::File(args) => {
            match &args.command {
                FileCommands::Copy(copy_args) => {
                    dougu_command_file::execute_copy(copy_args)?;
                }
                FileCommands::Move(move_args) => {
                    dougu_command_file::execute_move(move_args)?;
                }
                FileCommands::List(list_args) => {
                    dougu_command_file::execute_list(list_args)?;
                }
            }
        }
        Commands::Dropbox(args) => {
            // For demo purposes, use a dummy token
            let token = "dummy_dropbox_token";
            
            match &args.command {
                DropboxCommands::File(file_args) => {
                    match &file_args.command {
                        DropboxFileCommands::List(list_args) => {
                            dougu_command_dropbox::execute_file_list(list_args, token).await?;
                        }
                        DropboxFileCommands::Download(download_args) => {
                            dougu_command_dropbox::execute_file_download(download_args, token).await?;
                        }
                        DropboxFileCommands::Upload(upload_args) => {
                            dougu_command_dropbox::execute_file_upload(upload_args, token).await?;
                        }
                    }
                }
                DropboxCommands::Folder(folder_args) => {
                    match &folder_args.command {
                        dougu_command_dropbox::FolderCommands::Create(create_args) => {
                            dougu_command_dropbox::execute_folder_create(create_args, token).await?;
                        }
                        dougu_command_dropbox::FolderCommands::Delete(delete_args) => {
                            dougu_command_dropbox::execute_folder_delete(delete_args, token).await?;
                        }
                    }
                }
            }
        }
        Commands::Obj(cmd) => {
            cmd.execute().await?;
        }
    }
    
    Ok(())
}
