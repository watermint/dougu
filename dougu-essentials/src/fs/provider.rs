use crate::core::error::Result;
use crate::fs::entry::{Entry, EntryMetadata, FileEntry, FolderEntry};
use crate::fs::capability::CapabilitySet;
use crate::fs::path::{Path, PathComponents, Namespace};
use std::fmt::Debug;

/// Represents a provider of file system operations
pub trait FileSystemProvider: Debug {
    type PathType: Path;
    type EntryType: Entry<PathType=Self::PathType>;
    type FileEntryType: FileEntry<PathType=Self::PathType>;
    type FolderEntryType: FolderEntry<PathType=Self::PathType>;
    type MetadataType: EntryMetadata;
    
    /// Returns a unique identifier for this provider
    fn provider_id(&self) -> &str;
    
    /// Returns a human-readable name for this provider
    fn display_name(&self) -> &str;
    
    /// Returns the capabilities of this file system
    fn capabilities(&self) -> &CapabilitySet;
    
    /// Checks if a specific capability is supported
    fn supports(&self, capability: crate::fs::capability::Capability) -> bool {
        self.capabilities().has(capability)
    }
    
    /// Creates a new file entry at the specified path
    fn create_file_entry(&self, path: &Self::PathType) -> Result<Self::FileEntryType>;
    
    /// Creates a new folder entry at the specified path
    fn create_folder_entry(&self, path: &Self::PathType) -> Result<Self::FolderEntryType>;
    
    /// Gets the entry at the specified path
    fn get_entry(&self, path: &Self::PathType) -> Result<Self::EntryType>;
    
    /// Gets the root folder of this file system
    fn get_root_folder(&self) -> Result<Self::FolderEntryType>;
    
    /// Converts a standard OS path string to a path for this file system, if possible
    fn from_local_path(&self, local_path: &str) -> Result<Option<Self::PathType>>;
    
    /// Copies an entry from one path to another
    fn copy(
        &self, 
        source: &Self::PathType, 
        destination: &Self::PathType, 
        overwrite: bool
    ) -> Result<()>;
    
    /// Moves an entry from one path to another
    fn move_entry(
        &self, 
        source: &Self::PathType, 
        destination: &Self::PathType, 
        overwrite: bool
    ) -> Result<()>;
    
    /// Deletes an entry at the specified path
    fn delete(&self, path: &Self::PathType, recursive: bool) -> Result<()>;
    
    /// Gets the entry metadata at the specified path
    fn get_metadata(&self, path: &Self::PathType) -> Result<Self::MetadataType>;
}

/// Repository of file system providers
pub trait FileSystemProviderRepository: Debug + Send + Sync {
    type PathType: Path;
    type EntryType: Entry<PathType=Self::PathType>;
    type FileEntryType: FileEntry<PathType=Self::PathType>;
    type FolderEntryType: FolderEntry<PathType=Self::PathType>;
    type MetadataType: EntryMetadata;
    
    /// Gets a provider by ID
    fn get_provider(&self, provider_id: &str) -> Result<Box<dyn FileSystemProvider<
        PathType = Self::PathType,
        EntryType = Self::EntryType,
        FileEntryType = Self::FileEntryType,
        FolderEntryType = Self::FolderEntryType,
        MetadataType = Self::MetadataType
    >>>;
    
    /// Registers a provider
    fn register_provider(&mut self, provider: Box<dyn FileSystemProvider<
        PathType = Self::PathType,
        EntryType = Self::EntryType,
        FileEntryType = Self::FileEntryType,
        FolderEntryType = Self::FolderEntryType,
        MetadataType = Self::MetadataType
    >>) -> Result<()>;
    
    /// Gets all registered providers
    fn get_all_providers(&self) -> Vec<Box<dyn FileSystemProvider<
        PathType = Self::PathType,
        EntryType = Self::EntryType,
        FileEntryType = Self::FileEntryType,
        FolderEntryType = Self::FolderEntryType,
        MetadataType = Self::MetadataType
    >>>;
    
    /// Checks if a provider with the given ID is registered
    fn has_provider(&self, provider_id: &str) -> bool;
} 