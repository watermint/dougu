use anyhow::Result;
use clap::{Parser, Subcommand};
use essentials::init_logging;

#[derive(Parser)]
#[command(name = "db", about = "Command line tool with nested commands", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// File operations
    #[command(subcommand)]
    File(db_file::FileCommands),
    
    /// Dropbox operations
    #[command(subcommand)]
    Dropbox(db_dropbox::DropboxCommands),
}

fn main() -> Result<()> {
    init_logging();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::File(cmd) => db_file::run(cmd),
        Commands::Dropbox(cmd) => db_dropbox::run(cmd),
    }
} 