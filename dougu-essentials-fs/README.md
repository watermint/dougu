# dougu-essentials-fs

A filesystem abstraction layer for the `dougu` project, designed to hide differences between local file systems, cloud file systems, and other storage services.

## Features

- Abstract interface for different file system providers
- Support for local file system operations
- Extensible architecture for adding cloud providers
- Typed error handling using anyhow
- Async I/O operations with Tokio

## Usage

```rust
use dougu_essentials_fs::{FileSystem, ReadOptions, WriteOptions};
use dougu_essentials_fs::providers::local::LocalFileSystemProvider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create a new FileSystem instance
    let mut fs = FileSystem::new();
    
    // Register a local file system provider
    fs.register_provider(Box::new(LocalFileSystemProvider::new()));
    
    // List directory contents
    let entries = fs.list_directory("local", "./").await?;
    for entry in entries {
        println!("{}: {}, is_dir: {}", 
            entry.metadata.name, 
            entry.metadata.size, 
            entry.metadata.is_directory
        );
    }
    
    // Read a file
    let content = fs.read_file("local", "example.txt", ReadOptions::default()).await?;
    println!("File content: {}", String::from_utf8_lossy(&content));
    
    // Write a file
    let write_options = WriteOptions {
        overwrite: true,
        create_parents: true,
    };
    fs.write_file("local", "new/example.txt", b"Hello, World!".to_vec(), write_options).await?;
    
    Ok(())
}
```

## Provider Implementation

To implement a new file system provider, simply implement the `FileSystemProvider` trait:

```rust
use async_trait::async_trait;
use dougu_essentials_fs::{FileSystemProvider, FileSystemEntry, FileMetadata, ReadOptions, WriteOptions};

struct MyCloudProvider;

#[async_trait]
impl FileSystemProvider for MyCloudProvider {
    fn name(&self) -> &str {
        "my-cloud"
    }
    
    // Implement the required methods here...
} 