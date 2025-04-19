use anyhow::{Result, anyhow};
use async_trait::async_trait;
use log::{debug, error, info};
use serde::{Serialize, Deserialize};
use std::path::Path;
use tokio::io::AsyncRead;

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
    async fn list_directory<P: AsRef<Path> + Send + Sync>(&self, path: P) -> Result<Vec<FileSystemEntry>>;
    
    /// Read file content as bytes
    async fn read_file<P: AsRef<Path> + Send + Sync>(&self, path: P, options: ReadOptions) -> Result<Vec<u8>>;
    
    /// Write file content from bytes
    async fn write_file<P: AsRef<Path> + Send + Sync>(&self, path: P, content: Vec<u8>, options: WriteOptions) -> Result<()>;
    
    /// Delete a file or directory
    async fn delete<P: AsRef<Path> + Send + Sync>(&self, path: P, recursive: bool) -> Result<()>;
    
    /// Create a directory
    async fn create_directory<P: AsRef<Path> + Send + Sync>(&self, path: P, create_parents: bool) -> Result<()>;
    
    /// Get file or directory metadata
    async fn get_metadata<P: AsRef<Path> + Send + Sync>(&self, path: P) -> Result<FileMetadata>;
    
    /// Check if a file or directory exists
    async fn exists<P: AsRef<Path> + Send + Sync>(&self, path: P) -> Result<bool>;
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
            debug!(log_messages::PROVIDER_ALREADY_REGISTERED, provider_name);
            return;
        }
        
        info!(log_messages::PROVIDER_REGISTERED, provider_name);
        self.providers.push(provider);
    }
    
    /// Get a provider by name
    pub fn get_provider(&self, name: &str) -> Option<&dyn FileSystemProvider> {
        self.providers.iter()
            .find(|p| p.name() == name)
            .map(|p| p.as_ref())
    }
    
    /// List directory contents using the specified provider
    pub async fn list_directory<P: AsRef<Path> + Send + Sync>(
        &self, 
        provider_name: &str, 
        path: P
    ) -> Result<Vec<FileSystemEntry>> {
        let path_str = path.as_ref().to_string_lossy();
        debug!(log_messages::LISTING_DIRECTORY, path_str);
        
        let provider = self.get_provider(provider_name)
            .ok_or_else(|| anyhow!(error_messages::PROVIDER_NOT_FOUND))?;
            
        provider.list_directory(path).await
    }
    
    /// Read file content as bytes using the specified provider
    pub async fn read_file<P: AsRef<Path> + Send + Sync>(
        &self, 
        provider_name: &str, 
        path: P, 
        options: ReadOptions
    ) -> Result<Vec<u8>> {
        let path_str = path.as_ref().to_string_lossy();
        debug!(log_messages::READING_FILE, path_str);
        
        let provider = self.get_provider(provider_name)
            .ok_or_else(|| anyhow!(error_messages::PROVIDER_NOT_FOUND))?;
            
        provider.read_file(path, options).await
    }
    
    /// Write file content from bytes using the specified provider
    pub async fn write_file<P: AsRef<Path> + Send + Sync>(
        &self, 
        provider_name: &str, 
        path: P, 
        content: Vec<u8>, 
        options: WriteOptions
    ) -> Result<()> {
        let path_str = path.as_ref().to_string_lossy();
        debug!(log_messages::WRITING_FILE, path_str);
        
        let provider = self.get_provider(provider_name)
            .ok_or_else(|| anyhow!(error_messages::PROVIDER_NOT_FOUND))?;
            
        provider.write_file(path, content, options).await
    }
    
    /// Delete a file or directory using the specified provider
    pub async fn delete<P: AsRef<Path> + Send + Sync>(
        &self, 
        provider_name: &str, 
        path: P, 
        recursive: bool
    ) -> Result<()> {
        let path_str = path.as_ref().to_string_lossy();
        debug!(log_messages::DELETING_RESOURCE, path_str);
        
        let provider = self.get_provider(provider_name)
            .ok_or_else(|| anyhow!(error_messages::PROVIDER_NOT_FOUND))?;
            
        provider.delete(path, recursive).await
    }
    
    /// Create a directory using the specified provider
    pub async fn create_directory<P: AsRef<Path> + Send + Sync>(
        &self, 
        provider_name: &str, 
        path: P, 
        create_parents: bool
    ) -> Result<()> {
        let path_str = path.as_ref().to_string_lossy();
        debug!(log_messages::CREATING_DIRECTORY, path_str);
        
        let provider = self.get_provider(provider_name)
            .ok_or_else(|| anyhow!(error_messages::PROVIDER_NOT_FOUND))?;
            
        provider.create_directory(path, create_parents).await
    }
    
    /// Get file or directory metadata using the specified provider
    pub async fn get_metadata<P: AsRef<Path> + Send + Sync>(
        &self, 
        provider_name: &str, 
        path: P
    ) -> Result<FileMetadata> {
        let path_str = path.as_ref().to_string_lossy();
        debug!(log_messages::GETTING_METADATA, path_str);
        
        let provider = self.get_provider(provider_name)
            .ok_or_else(|| anyhow!(error_messages::PROVIDER_NOT_FOUND))?;
            
        provider.get_metadata(path).await
    }
    
    /// Check if a file or directory exists using the specified provider
    pub async fn exists<P: AsRef<Path> + Send + Sync>(
        &self, 
        provider_name: &str, 
        path: P
    ) -> Result<bool> {
        let path_str = path.as_ref().to_string_lossy();
        debug!(log_messages::CHECKING_EXISTS, path_str);
        
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