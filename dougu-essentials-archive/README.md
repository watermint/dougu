# dougu-essentials-archive

A Rust library for working with archive files in the Dougu ecosystem.

## Features

- Abstract interface for working with various archive formats
- Provider-based architecture for extensibility
- Built-in support for ZIP archives
- Full async API
- Comprehensive error handling

## Usage

### Basic Usage

```rust
use dougu_essentials_archive::{Archive, EntryOptions, ExtractOptions};
use dougu_essentials_archive::providers::zip::ZipProvider;
use std::path::{Path, PathBuf};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create and configure the archive manager
    let mut archive = Archive::new();
    
    // Register the ZIP provider
    archive.register_provider(Box::new(ZipProvider::new()));
    
    // Create a new ZIP archive
    let archive_path = Path::new("my_archive.zip");
    let sources = vec![
        PathBuf::from("file1.txt"),
        PathBuf::from("directory_to_include"),
    ];
    let options = EntryOptions::default();
    
    archive.create_archive("zip", archive_path, sources, options).await?;
    
    // List entries in the archive
    let entries = archive.list_entries("zip", archive_path).await?;
    for entry in entries {
        println!("Entry: {}, Size: {} bytes", entry.metadata.path, entry.metadata.size);
    }
    
    // Extract the entire archive
    let extract_options = ExtractOptions {
        overwrite: true,
        ..Default::default()
    };
    
    archive.extract_archive(
        "zip", 
        archive_path, 
        Path::new("extraction_directory"), 
        extract_options
    ).await?;
    
    // Extract a single file
    archive.extract_entry(
        "zip",
        archive_path,
        "file1.txt",
        Path::new("extracted_file.txt")
    ).await?;
    
    // Add a new file to the archive
    archive.add_entry(
        "zip",
        archive_path,
        Path::new("new_file.txt"),
        "new_file.txt",
        EntryOptions::default()
    ).await?;
    
    // Check if an entry exists
    let exists = archive.entry_exists("zip", archive_path, "file1.txt").await?;
    println!("Entry exists: {}", exists);
    
    // Convenience methods with automatic provider detection
    archive.create_archive_auto(
        Path::new("another_archive.zip"),
        vec![PathBuf::from("data_directory")],
        EntryOptions::default()
    ).await?;
    
    Ok(())
}
```

### Using Different Formats

The library uses a provider-based architecture, making it easy to add support for additional archive formats:

```rust
// Register multiple providers
archive.register_provider(Box::new(ZipProvider::new()));
// When more providers are implemented, they can be added here
// archive.register_provider(Box::new(TarProvider::new()));
// archive.register_provider(Box::new(RarProvider::new()));
```

## Providers

Current implementations:

- `ZipProvider`: Support for ZIP archives using the `zip` crate

## License

See the main Dougu license. 