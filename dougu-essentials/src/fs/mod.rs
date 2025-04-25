// File system module that abstracts various file system operations
// including local file systems, cloud file systems, and read-only file systems.

use crate::core::Result;
use crate::time::SystemTime;
use async_trait::async_trait;
use std::io::{Read, Seek, Write};
use std::path::{Path as StdPath, PathBuf as StdPathBuf};
use std::any::Any;
use std::fmt::Debug;
use std::collections::HashMap;

pub mod local;
pub mod cloud;
pub mod readonly;
pub mod path;

// Re-export path types and functions
pub use path::{
    Component, GenericPath, PathFormat, PathTrait, PosixPath, UrlPath,
    from_str as path_from_str, posix_path, url_path,
};

/// Base trait for file attributes that all file systems must implement
pub trait FileAttributesTrait: Debug + Send + Sync {
    /// Get the path of the file
    fn path(&self) -> &GenericPath;
    
    /// Get the size of the file in bytes
    fn size(&self) -> u64;
    
    /// Get the last modified time if available
    fn modified(&self) -> Option<SystemTime>;
    
    /// Get the creation time if available
    fn created(&self) -> Option<SystemTime>;
    
    /// Check if this entry is a folder
    fn is_folder(&self) -> bool;
    
    /// Check if this entry is a file
    fn is_file(&self) -> bool;
    
    /// Check if this entry is a symlink
    fn is_symlink(&self) -> bool;
    
    /// Check if this entry is hidden
    fn is_hidden(&self) -> bool;
    
    /// Cast to a specific attribute extension trait
    fn as_any(&self) -> &dyn Any;
    
    /// Get custom attribute as a string if it exists
    fn get_attribute(&self, name: &str) -> Option<String> {
        None
    }
    
    /// Get all available custom attribute names
    fn attribute_names(&self) -> Vec<String> {
        Vec::new()
    }
}

/// Basic implementation of file attributes that all file systems can use or extend
#[derive(Debug, Clone)]
pub struct FileAttributes {
    /// Path of the file
    pub path: GenericPath,
    /// Size of the file in bytes
    pub size: u64,
    /// Last modified time
    pub modified: Option<SystemTime>,
    /// Created time
    pub created: Option<SystemTime>,
    /// Is this a folder
    pub is_folder: bool,
    /// Is this a file
    pub is_file: bool,
    /// Is this a symlink
    pub is_symlink: bool,
    /// Is this a hidden file
    pub is_hidden: bool,
    /// Custom attributes map
    pub custom_attributes: HashMap<String, String>,
}

impl FileAttributesTrait for FileAttributes {
    fn path(&self) -> &GenericPath {
        &self.path
    }
    
    fn size(&self) -> u64 {
        self.size
    }
    
    fn modified(&self) -> Option<SystemTime> {
        self.modified
    }
    
    fn created(&self) -> Option<SystemTime> {
        self.created
    }
    
    fn is_folder(&self) -> bool {
        self.is_folder
    }
    
    fn is_file(&self) -> bool {
        self.is_file
    }
    
    fn is_symlink(&self) -> bool {
        self.is_symlink
    }
    
    fn is_hidden(&self) -> bool {
        self.is_hidden
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn get_attribute(&self, name: &str) -> Option<String> {
        self.custom_attributes.get(name).cloned()
    }
    
    fn attribute_names(&self) -> Vec<String> {
        self.custom_attributes.keys().cloned().collect()
    }
}

impl FileAttributes {
    /// Create a new FileAttributes instance
    pub fn new(
        path: GenericPath,
        size: u64,
        modified: Option<SystemTime>,
        created: Option<SystemTime>,
        is_folder: bool,
        is_file: bool,
        is_symlink: bool,
        is_hidden: bool,
    ) -> Self {
        Self {
            path,
            size,
            modified,
            created,
            is_folder,
            is_file,
            is_symlink,
            is_hidden,
            custom_attributes: HashMap::new(),
        }
    }
    
    /// Add a custom attribute
    pub fn with_attribute(mut self, name: &str, value: &str) -> Self {
        self.custom_attributes.insert(name.to_string(), value.to_string());
        self
    }
    
    /// Add multiple custom attributes
    pub fn with_attributes(mut self, attributes: HashMap<String, String>) -> Self {
        self.custom_attributes.extend(attributes);
        self
    }
}

/// Represents file permissions
#[derive(Debug, Clone, Default)]
pub struct FilePermissions {
    /// Is this file readable
    pub readable: bool,
    /// Is this file writable
    pub writable: bool,
    /// Is this file executable
    pub executable: bool,
}

/// Link to a file or folder in cloud file systems
#[derive(Debug, Clone)]
pub struct FileLink {
    /// URL to access the file
    pub url: String,
    /// Expiry time of the link if applicable
    pub expires_at: Option<SystemTime>,
    /// Whether this link is for direct download
    pub is_download_link: bool,
}

/// Represents different types of file systems
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileSystemType {
    /// Local file system
    Local,
    /// Cloud file system
    Cloud,
    /// Read-only file system
    ReadOnly,
    /// Custom file system
    Custom(u8),
}

/// Base trait for all file systems
#[async_trait]
pub trait FileSystem: Send + Sync {
    /// Get the type of file system
    fn get_type(&self) -> FileSystemType;
    
    /// Get capabilities of this file system
    fn get_capabilities(&self) -> FileSystemCapabilities;
    
    /// Check if the file system is available
    async fn is_available(&self) -> Result<bool>;
    
    /// List files in a folder
    async fn list_folder(&self, path: &GenericPath) -> Result<Vec<Box<dyn FileAttributesTrait>>>;
    
    /// Get attributes for a file
    async fn get_attributes(&self, path: &GenericPath) -> Result<Box<dyn FileAttributesTrait>>;
    
    /// Check if a file exists
    async fn exists(&self, path: &GenericPath) -> Result<bool>;
}

/// Capabilities of a file system
#[derive(Debug, Clone, Copy)]
pub struct FileSystemCapabilities {
    /// Can create files
    pub can_create: bool,
    /// Can delete files
    pub can_delete: bool,
    /// Can move files
    pub can_move: bool,
    /// Can rename files
    pub can_rename: bool,
    /// Can read files
    pub can_read: bool,
    /// Can write files
    pub can_write: bool,
    /// Can append to files
    pub can_append: bool,
    /// Can seek within files
    pub can_seek: bool,
    /// Can stream files
    pub can_stream: bool,
    /// Can create links to files
    pub can_link: bool,
    /// Can set file permissions
    pub can_set_permissions: bool,
    /// Can read file permissions
    pub can_get_permissions: bool,
}

impl Default for FileSystemCapabilities {
    fn default() -> Self {
        Self {
            can_create: false,
            can_delete: false,
            can_move: false,
            can_rename: false,
            can_read: false,
            can_write: false,
            can_append: false,
            can_seek: false,
            can_stream: false,
            can_link: false,
            can_set_permissions: false,
            can_get_permissions: false,
        }
    }
}

/// Trait for writeable file systems
#[async_trait]
pub trait WriteableFileSystem: FileSystem {
    /// Create a new file and return a handle to write to it
    async fn create_file(&self, path: &GenericPath) -> Result<Box<dyn FileWriter>>;
    
    /// Open a file for writing (overwrite)
    async fn write_file(&self, path: &GenericPath) -> Result<Box<dyn FileWriter>>;
    
    /// Open a file for appending
    async fn append_file(&self, path: &GenericPath) -> Result<Box<dyn FileWriter>>;
    
    /// Create a folder
    async fn create_folder(&self, path: &GenericPath) -> Result<()>;
    
    /// Create folders recursively
    async fn create_folder_all(&self, path: &GenericPath) -> Result<()>;
    
    /// Delete a file
    async fn delete_file(&self, path: &GenericPath) -> Result<()>;
    
    /// Delete a folder
    async fn delete_folder(&self, path: &GenericPath) -> Result<()>;
    
    /// Delete a folder and all its contents
    async fn delete_folder_all(&self, path: &GenericPath) -> Result<()>;
    
    /// Rename a file or folder
    async fn rename(&self, from: &GenericPath, to: &GenericPath) -> Result<()>;
    
    /// Copy a file
    async fn copy_file(&self, from: &GenericPath, to: &GenericPath) -> Result<()>;
    
    /// Move a file
    async fn move_file(&self, from: &GenericPath, to: &GenericPath) -> Result<()>;
    
    /// Set file permissions
    async fn set_permissions(&self, path: &GenericPath, permissions: FilePermissions) -> Result<()>;
}

/// Trait for readable file systems
#[async_trait]
pub trait ReadableFileSystem: FileSystem {
    /// Open a file for reading
    async fn open_file(&self, path: &GenericPath) -> Result<Box<dyn FileReader>>;
    
    /// Read the entire contents of a file into a string
    async fn read_to_string(&self, path: &GenericPath) -> Result<String>;
    
    /// Read the entire contents of a file into a byte vector
    async fn read_to_bytes(&self, path: &GenericPath) -> Result<Vec<u8>>;
    
    /// Get file permissions
    async fn get_permissions(&self, path: &GenericPath) -> Result<FilePermissions>;
}

/// Trait for cloud file systems that support links
#[async_trait]
pub trait LinkableFileSystem: FileSystem {
    /// Create a link to a file or folder
    async fn create_link(&self, path: &GenericPath, expires_in_secs: Option<u64>) -> Result<FileLink>;
    
    /// Create a download link for a file
    async fn create_download_link(&self, path: &GenericPath, expires_in_secs: Option<u64>) -> Result<FileLink>;
}

/// Trait for file systems that support streaming
#[async_trait]
pub trait StreamableFileSystem: FileSystem {
    /// Open a file for streaming
    async fn stream_file(&self, path: &GenericPath) -> Result<Box<dyn FileStream>>;
}

/// Trait for file writer
pub trait FileWriter: Write + Send + Sync {
    /// Sync all buffered data to storage
    fn sync_all(&mut self) -> std::io::Result<()>;
    
    /// Sync only data to storage, not metadata
    fn sync_data(&mut self) -> std::io::Result<()>;
    
    /// Get the current position in the file
    fn position(&self) -> std::io::Result<u64>;
    
    /// Close the file
    fn close(&mut self) -> std::io::Result<()>;
}

/// Trait for file reader
pub trait FileReader: Read + Send + Sync {
    /// Get the current position in the file
    fn position(&self) -> std::io::Result<u64>;
    
    /// Close the file
    fn close(&mut self) -> std::io::Result<()>;
}

/// Trait for seekable file readers
pub trait SeekableFileReader: FileReader + Seek {}

/// Trait for seekable file writers
pub trait SeekableFileWriter: FileWriter + Seek {}

/// Trait for file stream
pub trait FileStream: Send + Sync {
    /// Read the next chunk of data
    fn read_next(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;
    
    /// Skip the next N bytes
    fn skip(&mut self, n: u64) -> std::io::Result<u64>;
    
    /// Close the stream
    fn close(&mut self) -> std::io::Result<()>;
}

/// Factory for creating file systems
pub trait FileSystemFactory: Send + Sync {
    /// Create a file system instance
    fn create(&self) -> Result<Box<dyn FileSystem>>;
}
