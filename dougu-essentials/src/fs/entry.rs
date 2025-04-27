use crate::core::error::Result;
use crate::fs::path::Path;
use crate::time::ZonedDateTime;
use std::fmt::Debug;
use std::io::{Read, Seek, Write};
use std::time::SystemTime;

/// Represents the type of a file system entry
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    /// Regular file
    File,
    /// Folder/directory
    Folder,
    /// Symbolic link
    SymLink,
    /// Special device file
    Device,
    /// Named pipe
    Pipe,
    /// Socket
    Socket,
    /// Other or unknown type
    Other,
}

/// Represents metadata for a file system entry (file or folder)
pub trait EntryMetadata: Debug {
    /// Returns the type of this entry
    fn entry_type(&self) -> EntryType;
    
    /// Returns the size of this entry in bytes
    fn size(&self) -> u64;
    
    /// Returns true if this entry is a file
    fn is_file(&self) -> bool {
        self.entry_type() == EntryType::File
    }
    
    /// Returns true if this entry is a folder
    fn is_folder(&self) -> bool {
        self.entry_type() == EntryType::Folder
    }
    
    /// Returns true if this entry is a symbolic link
    fn is_symlink(&self) -> bool {
        self.entry_type() == EntryType::SymLink
    }
    
    /// Returns the creation time of this entry, if available
    fn created(&self) -> Option<ZonedDateTime>;
    
    /// Returns the last modified time of this entry, if available
    fn modified(&self) -> Option<ZonedDateTime>;
    
    /// Returns the last accessed time of this entry, if available
    fn accessed(&self) -> Option<ZonedDateTime>;
    
    /// Returns true if this entry is readonly
    fn is_readonly(&self) -> bool;
    
    /// Returns true if this entry is hidden
    fn is_hidden(&self) -> bool;
}

/// Represents a file system entry, which could be a file or a folder
/// 
/// This trait defines the common operations that can be performed on both files and folders.
/// Different file system providers will implement these operations according to their
/// specific APIs and capabilities.
pub trait Entry: Debug + Send + Sync {
    type PathType: Path;
    type MetadataType: EntryMetadata;
    
    /// Returns the path of this entry
    fn path(&self) -> &Self::PathType;
    
    /// Returns the metadata of this entry
    fn metadata(&self) -> Result<Self::MetadataType>;
    
    /// Returns the name of this entry (last path component)
    fn name(&self) -> Option<String> {
        self.path().file_name()
    }
    
    /// Returns true if this entry exists
    fn exists(&self) -> Result<bool>;
    
    /// Returns true if this entry is a file
    fn is_file(&self) -> Result<bool> {
        Ok(self.metadata()?.is_file())
    }
    
    /// Returns true if this entry is a folder
    fn is_folder(&self) -> Result<bool> {
        Ok(self.metadata()?.is_folder())
    }
}

/// Represents a file in a file system
pub trait FileEntry: Entry {
    /// Opens this file for reading
    fn open_read(&self) -> Result<Box<dyn ReadableFile>>;
    
    /// Opens this file for writing, truncating it if it already exists
    fn open_write(&self) -> Result<Box<dyn WritableFile>>;
    
    /// Opens this file for appending
    fn open_append(&self) -> Result<Box<dyn WritableFile>>;
    
    /// Opens this file for both reading and writing
    fn open_read_write(&self) -> Result<Box<dyn ReadWriteFile>>;
    
    /// Creates a new file, failing if it already exists
    fn create_new(&self) -> Result<Box<dyn WritableFile>>;
    
    /// Deletes this file
    fn delete(&self) -> Result<()>;
    
    /// Returns the parent folder of this file
    fn parent_folder(&self) -> Option<Box<dyn FolderEntry<PathType=Self::PathType, MetadataType=Self::MetadataType>>>;
    
    /// Copies this file to the specified destination
    /// 
    /// The destination is specified as a FileEntry that must have the same PathType and MetadataType
    fn copy_to(&self, destination: &dyn FileEntry<PathType=Self::PathType, MetadataType=Self::MetadataType>) -> Result<()>;
    
    /// Moves this file to the specified destination
    /// 
    /// The destination is specified as a FileEntry that must have the same PathType and MetadataType
    fn move_to(&self, destination: &dyn FileEntry<PathType=Self::PathType, MetadataType=Self::MetadataType>) -> Result<()>;
}

/// Represents a folder in a file system
pub trait FolderEntry: Entry {
    /// Lists all entries in this folder
    fn list_entries(&self) -> Result<Vec<Box<dyn Entry<PathType=Self::PathType, MetadataType=Self::MetadataType>>>>;
    
    /// Lists only files in this folder
    fn list_files(&self) -> Result<Vec<Box<dyn FileEntry<PathType=Self::PathType, MetadataType=Self::MetadataType>>>>;
    
    /// Lists only subfolders in this folder
    fn list_folders(&self) -> Result<Vec<Box<dyn FolderEntry<PathType=Self::PathType, MetadataType=Self::MetadataType>>>>;
    
    /// Creates this folder if it doesn't exist
    fn create(&self) -> Result<()>;
    
    /// Creates this folder and all parent folders if they don't exist
    fn create_recursive(&self) -> Result<()>;
    
    /// Deletes this folder
    fn delete(&self) -> Result<()>;
    
    /// Deletes this folder and all its contents recursively
    fn delete_recursive(&self) -> Result<()>;
    
    /// Returns the parent folder, if any
    fn parent(&self) -> Option<Box<dyn FolderEntry<PathType=Self::PathType, MetadataType=Self::MetadataType>>>;
    
    /// Gets a child file with the given name
    fn get_file(&self, name: &str) -> Result<Box<dyn FileEntry<PathType=Self::PathType, MetadataType=Self::MetadataType>>>;
    
    /// Gets a child folder with the given name
    fn get_folder(&self, name: &str) -> Result<Box<dyn FolderEntry<PathType=Self::PathType, MetadataType=Self::MetadataType>>>;
}

/// Represents a readable file stream
pub trait ReadableFile: Read + Seek + Debug {
    /// Returns the file size in bytes
    fn size(&self) -> Result<u64>;
}

/// Represents a writable file stream
pub trait WritableFile: Write + Seek + Debug {
    /// Flushes any buffered data to the underlying storage
    fn flush_all(&mut self) -> Result<()>;
    
    /// Returns the current file size in bytes
    fn size(&self) -> Result<u64>;
}

/// Represents a file that can be both read from and written to
pub trait ReadWriteFile: Read + Write + Seek + Debug {
    /// Flushes any buffered data to the underlying storage
    fn flush_all(&mut self) -> Result<()>;
    
    /// Returns the current file size in bytes
    fn size(&self) -> Result<u64>;
} 