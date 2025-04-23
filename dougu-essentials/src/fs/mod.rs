// File system operations module

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use log::{debug, info};
use serde::{Serialize, Deserialize};
use std::path::Path;

pub mod resources;
use resources::error_messages;
use resources::log_messages;
pub mod providers;

/// Represents file metadata across different file systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_directory: bool,
    pub last_modified: Option<u64>,
    pub content_hash: Option<String>,
}

/// Represents a file system entry (file or directory)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemEntry {
    pub metadata: FileMetadata,
    pub provider_info: Option<serde_json::Value>,
}

/// Read options for controlling file operations
#[derive(Debug, Clone, Default)]
pub struct ReadOptions {
    pub offset: Option<u64>,
    pub length: Option<u64>,
}

/// Write options for controlling file operations
#[derive(Debug, Clone, Default)]
pub struct WriteOptions {
    pub overwrite: bool,
    pub create_parents: bool,
}

/// Abstraction for different file system providers
#[async_trait]
pub trait FileSystemProvider: Send + Sync {
    /// Get the provider name
    fn name(&self) -> &str;
    
    /// List directory contents
    async fn list_directory(&self, path: &Path) -> Result<Vec<FileSystemEntry>>;
    
    /// Read file content as bytes
    async fn read_file(&self, path: &Path, options: ReadOptions) -> Result<Vec<u8>>;
    
    /// Write file content from bytes
    async fn write_file(&self, path: &Path, content: Vec<u8>, options: WriteOptions) -> Result<()>;
    
    /// Delete a file or directory
    async fn delete(&self, path: &Path, recursive: bool) -> Result<()>;
    
    /// Create a directory
    async fn create_directory(&self, path: &Path, create_parents: bool) -> Result<()>;
    
    /// Get file or directory metadata
    async fn get_metadata(&self, path: &Path) -> Result<FileMetadata>;
    
    /// Check if a file or directory exists
    async fn exists(&self, path: &Path) -> Result<bool>;
}

/// FileSystem is the main entry point for interacting with file systems
pub struct FileSystem {
    providers: Vec<Box<dyn FileSystemProvider>>,
}

impl FileSystem {
    /// Create a new FileSystem instance
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }
    
    /// Register a provider
    pub fn register_provider(&mut self, provider: Box<dyn FileSystemProvider>) {
        let provider_name = provider.name();
        if self.get_provider(provider_name).is_some() {
            debug!("{}: {}", log_messages::PROVIDER_ALREADY_REGISTERED, provider_name);
            return;
        }
        
        info!("{}: {}", log_messages::PROVIDER_REGISTERED, provider_name);
        self.providers.push(provider);
    }
    
    /// Get a provider by name
    pub fn get_provider(&self, name: &str) -> Option<&dyn FileSystemProvider> {
        self.providers.iter()
            .find(|p| p.name() == name)
            .map(|p| p.as_ref())
    }
    
    /// List directory contents using the specified provider
    pub async fn list_directory(
        &self, 
        provider_name: &str, 
        path: &Path
    ) -> Result<Vec<FileSystemEntry>> {
        let path_str = path.to_string_lossy();
        debug!("{}: {}", log_messages::LISTING_DIRECTORY, path_str);
        
        let provider = self.get_provider(provider_name)
            .ok_or_else(|| anyhow!(error_messages::PROVIDER_NOT_FOUND))?;
            
        provider.list_directory(path).await
    }
    
    /// Read file content as bytes using the specified provider
    pub async fn read_file(
        &self, 
        provider_name: &str, 
        path: &Path, 
        options: ReadOptions
    ) -> Result<Vec<u8>> {
        let path_str = path.to_string_lossy();
        debug!("{}: {}", log_messages::READING_FILE, path_str);
        
        let provider = self.get_provider(provider_name)
            .ok_or_else(|| anyhow!(error_messages::PROVIDER_NOT_FOUND))?;
            
        provider.read_file(path, options).await
    }
    
    /// Write file content from bytes using the specified provider
    pub async fn write_file(
        &self, 
        provider_name: &str, 
        path: &Path, 
        content: Vec<u8>, 
        options: WriteOptions
    ) -> Result<()> {
        let path_str = path.to_string_lossy();
        debug!("{}: {}", log_messages::WRITING_FILE, path_str);
        
        let provider = self.get_provider(provider_name)
            .ok_or_else(|| anyhow!(error_messages::PROVIDER_NOT_FOUND))?;
            
        provider.write_file(path, content, options).await
    }
    
    /// Delete a file or directory using the specified provider
    pub async fn delete(
        &self, 
        provider_name: &str, 
        path: &Path, 
        recursive: bool
    ) -> Result<()> {
        let path_str = path.to_string_lossy();
        debug!("{}: {}", log_messages::DELETING_RESOURCE, path_str);
        
        let provider = self.get_provider(provider_name)
            .ok_or_else(|| anyhow!(error_messages::PROVIDER_NOT_FOUND))?;
            
        provider.delete(path, recursive).await
    }
    
    /// Create a directory using the specified provider
    pub async fn create_directory(
        &self, 
        provider_name: &str, 
        path: &Path, 
        create_parents: bool
    ) -> Result<()> {
        let path_str = path.to_string_lossy();
        debug!("{}: {}", log_messages::CREATING_DIRECTORY, path_str);
        
        let provider = self.get_provider(provider_name)
            .ok_or_else(|| anyhow!(error_messages::PROVIDER_NOT_FOUND))?;
            
        provider.create_directory(path, create_parents).await
    }
    
    /// Get file or directory metadata using the specified provider
    pub async fn get_metadata(
        &self, 
        provider_name: &str, 
        path: &Path
    ) -> Result<FileMetadata> {
        let path_str = path.to_string_lossy();
        debug!("{}: {}", log_messages::GETTING_METADATA, path_str);
        
        let provider = self.get_provider(provider_name)
            .ok_or_else(|| anyhow!(error_messages::PROVIDER_NOT_FOUND))?;
            
        provider.get_metadata(path).await
    }
    
    /// Check if a file or directory exists using the specified provider
    pub async fn exists(
        &self, 
        provider_name: &str, 
        path: &Path
    ) -> Result<bool> {
        let path_str = path.to_string_lossy();
        debug!("{}: {}", log_messages::CHECKING_EXISTS, path_str);
        
        let provider = self.get_provider(provider_name)
            .ok_or_else(|| anyhow!(error_messages::PROVIDER_NOT_FOUND))?;
            
        provider.exists(path).await
    }
}

impl Default for FileSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn it_initializes() {
        let fs = FileSystem::new();
        assert_eq!(fs.providers.len(), 0);
    }
} 