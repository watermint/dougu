// Archive operations module

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use log::{debug, info};
use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};

pub mod resources;
use resources::{error_messages, log_messages};
pub mod providers;

/// Represents archive entry metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveMetadata {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_directory: bool,
    pub compressed_size: Option<u64>,
    pub last_modified: Option<u64>,
}

/// Represents an archive entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveEntry {
    pub metadata: ArchiveMetadata,
    pub provider_info: Option<serde_json::Value>,
}

/// Options for entry operations
#[derive(Debug, Clone, Default)]
pub struct EntryOptions {
    pub compression_level: Option<i32>,
    pub preserve_permissions: bool,
}

/// Options for extraction operations
#[derive(Debug, Clone, Default)]
pub struct ExtractOptions {
    pub overwrite: bool,
    pub preserve_permissions: bool,
    pub filter_prefix: Option<String>,
}

/// Abstraction for different archive providers
#[async_trait]
pub trait ArchiveProvider: Send + Sync {
    /// Get the provider name
    fn name(&self) -> &str;
    
    /// Create a new archive from source files/directories
    async fn create_archive(&self, archive_path: &Path, sources: Vec<PathBuf>, options: EntryOptions) -> Result<()>;
    
    /// Extract an archive to a target directory
    async fn extract_archive(&self, archive_path: &Path, target_dir: &Path, options: ExtractOptions) -> Result<()>;
    
    /// List entries in an archive
    async fn list_entries(&self, archive_path: &Path) -> Result<Vec<ArchiveEntry>>;
    
    /// Extract a single entry from an archive
    async fn extract_entry(&self, archive_path: &Path, entry_path: &str, target_path: &Path) -> Result<()>;
    
    /// Add or update a file/directory in an archive
    async fn add_entry(&self, archive_path: &Path, source_path: &Path, entry_name: &str, options: EntryOptions) -> Result<()>;
    
    /// Check if an entry exists in an archive
    async fn entry_exists(&self, archive_path: &Path, entry_path: &str) -> Result<bool>;
    
    /// Get metadata for an entry
    async fn get_entry_metadata(&self, archive_path: &Path, entry_path: &str) -> Result<ArchiveMetadata>;
}

/// Archive is the main entry point for interacting with archives
pub struct Archive {
    providers: Vec<Box<dyn ArchiveProvider>>,
}

impl Archive {
    /// Create a new Archive instance
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }
    
    /// Register a provider
    pub fn register_provider(&mut self, provider: Box<dyn ArchiveProvider>) {
        let provider_name = provider.name();
        if self.get_provider(provider_name).is_some() {
            debug!("{}: {}", log_messages::PROVIDER_ALREADY_REGISTERED, provider_name);
            return;
        }
        
        info!("{}: {}", log_messages::PROVIDER_REGISTERED, provider_name);
        self.providers.push(provider);
    }
    
    /// Get a provider by name
    pub fn get_provider(&self, name: &str) -> Option<&dyn ArchiveProvider> {
        self.providers.iter()
            .find(|p| p.name() == name)
            .map(|p| p.as_ref())
    }
    
    /// Get a provider based on file extension
    pub fn get_provider_for_extension(&self, extension: &str) -> Option<&dyn ArchiveProvider> {
        match extension.to_lowercase().as_str() {
            "zip" => self.get_provider("zip"),
            // Add support for other formats as needed
            _ => None,
        }
    }
    
    /// Create a new archive from source files/directories
    pub async fn create_archive(
        &self, 
        provider_name: &str, 
        archive_path: &Path,
        sources: Vec<PathBuf>,
        options: EntryOptions
    ) -> Result<()> {
        debug!("{}: {}", log_messages::CREATING_ARCHIVE, archive_path.display());
        
        let provider = self.get_provider(provider_name)
            .ok_or_else(|| anyhow!(error_messages::PROVIDER_NOT_FOUND))?;
            
        provider.create_archive(archive_path, sources, options).await
    }
    
    /// Extract an archive to a target directory
    pub async fn extract_archive(
        &self, 
        provider_name: &str, 
        archive_path: &Path,
        target_dir: &Path,
        options: ExtractOptions
    ) -> Result<()> {
        debug!("{}: {}", log_messages::EXTRACTING_ARCHIVE, archive_path.display());
        
        let provider = self.get_provider(provider_name)
            .ok_or_else(|| anyhow!(error_messages::PROVIDER_NOT_FOUND))?;
            
        provider.extract_archive(archive_path, target_dir, options).await
    }
    
    /// List entries in an archive
    pub async fn list_entries(
        &self, 
        provider_name: &str, 
        archive_path: &Path
    ) -> Result<Vec<ArchiveEntry>> {
        debug!("{}: {}", log_messages::LISTING_ENTRIES, archive_path.display());
        
        let provider = self.get_provider(provider_name)
            .ok_or_else(|| anyhow!(error_messages::PROVIDER_NOT_FOUND))?;
            
        provider.list_entries(archive_path).await
    }
    
    /// Extract a single entry from an archive
    pub async fn extract_entry(
        &self, 
        provider_name: &str, 
        archive_path: &Path,
        entry_path: &str,
        target_path: &Path
    ) -> Result<()> {
        debug!("{}: {}", log_messages::EXTRACTING_ENTRY, entry_path);
        
        let provider = self.get_provider(provider_name)
            .ok_or_else(|| anyhow!(error_messages::PROVIDER_NOT_FOUND))?;
            
        provider.extract_entry(archive_path, entry_path, target_path).await
    }
    
    /// Add or update a file/directory in an archive
    pub async fn add_entry(
        &self, 
        provider_name: &str, 
        archive_path: &Path,
        source_path: &Path,
        entry_name: &str,
        options: EntryOptions
    ) -> Result<()> {
        debug!("{}: {}", log_messages::ADDING_ENTRY, entry_name);
        
        let provider = self.get_provider(provider_name)
            .ok_or_else(|| anyhow!(error_messages::PROVIDER_NOT_FOUND))?;
            
        provider.add_entry(archive_path, source_path, entry_name, options).await
    }
    
    /// Check if an entry exists in an archive
    pub async fn entry_exists(
        &self, 
        provider_name: &str, 
        archive_path: &Path,
        entry_path: &str
    ) -> Result<bool> {
        debug!("{}: {}", log_messages::CHECKING_ENTRY_EXISTS, entry_path);
        
        let provider = self.get_provider(provider_name)
            .ok_or_else(|| anyhow!(error_messages::PROVIDER_NOT_FOUND))?;
            
        provider.entry_exists(archive_path, entry_path).await
    }
    
    /// Get metadata for an entry
    pub async fn get_entry_metadata(
        &self, 
        provider_name: &str, 
        archive_path: &Path,
        entry_path: &str
    ) -> Result<ArchiveMetadata> {
        debug!("{}: {}", log_messages::GETTING_ENTRY_METADATA, entry_path);
        
        let provider = self.get_provider(provider_name)
            .ok_or_else(|| anyhow!(error_messages::PROVIDER_NOT_FOUND))?;
            
        provider.get_entry_metadata(archive_path, entry_path).await
    }
    
    /// Automatically select a provider based on file extension and create an archive
    pub async fn create_archive_auto(
        &self,
        archive_path: &Path,
        sources: Vec<PathBuf>,
        options: EntryOptions
    ) -> Result<()> {
        let extension = archive_path.extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| anyhow!(error_messages::INVALID_ARCHIVE_EXTENSION))?;
            
        let provider = self.get_provider_for_extension(extension)
            .ok_or_else(|| anyhow!(error_messages::UNSUPPORTED_ARCHIVE_TYPE))?;
            
        provider.create_archive(archive_path, sources, options).await
    }
    
    /// Automatically select a provider based on file extension and extract an archive
    pub async fn extract_archive_auto(
        &self,
        archive_path: &Path,
        target_dir: &Path,
        options: ExtractOptions
    ) -> Result<()> {
        let extension = archive_path.extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| anyhow!(error_messages::INVALID_ARCHIVE_EXTENSION))?;
            
        let provider = self.get_provider_for_extension(extension)
            .ok_or_else(|| anyhow!(error_messages::UNSUPPORTED_ARCHIVE_TYPE))?;
            
        provider.extract_archive(archive_path, target_dir, options).await
    }
}

impl Default for Archive {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn it_initializes() {
        let archive = Archive::new();
        assert_eq!(archive.providers.len(), 0);
    }
    
    #[test]
    fn it_registers_provider() {
        // This is just a skeleton test that would be expanded with a mock provider
        let archive = Archive::new();
        assert_eq!(archive.providers.len(), 0);
    }
} 