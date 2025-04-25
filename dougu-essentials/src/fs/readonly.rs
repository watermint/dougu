// Implementation of read-only file systems
// This can be used for CD-ROMs, mounted disk images, or pseudo file systems for services

use crate::core::Result;
use crate::core::error::{Error, ErrorExt};
use crate::time::SystemTime;
use crate::fs::{
    FileAttributes, FileAttributesTrait, FilePermissions, FileReader, FileStream, FileSystem,
    FileSystemCapabilities, FileSystemFactory, FileSystemType, ReadableFileSystem,
    StreamableFileSystem, GenericPath, PosixPath,
};
use async_trait::async_trait;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};

/// Read-only file system implementation
pub struct ReadOnlyFileSystem {
    capabilities: FileSystemCapabilities,
    name: String,
    root_path: PathBuf,
    // Specific provider may need additional fields
}

impl ReadOnlyFileSystem {
    /// Create a new read-only file system
    pub fn new(name: &str, root_path: &Path) -> Self {
        Self {
            capabilities: FileSystemCapabilities {
                can_create: false,
                can_delete: false,
                can_move: false,
                can_rename: false,
                can_read: true,
                can_write: false,
                can_append: false,
                can_seek: true,
                can_stream: true,
                can_link: false,
                can_set_permissions: false,
                can_get_permissions: true,
            },
            name: name.to_string(),
            root_path: root_path.to_path_buf(),
        }
    }

    /// Get the name of this read-only file system
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the root path of this read-only file system
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }
}

#[async_trait]
impl FileSystem for ReadOnlyFileSystem {
    fn get_type(&self) -> FileSystemType {
        FileSystemType::ReadOnly
    }

    fn get_capabilities(&self) -> FileSystemCapabilities {
        self.capabilities
    }

    async fn is_available(&self) -> Result<bool> {
        // This should be overridden by specific read-only implementations
        // to check if the underlying storage is available
        Ok(true)
    }

    async fn list_folder(&self, _path: &Path) -> Result<Vec<Box<dyn FileAttributesTrait>>> {
        // Implementation should be provided by specific read-only filesystem provider
        unimplemented!("Method not implemented for base ReadOnlyFileSystem")
    }

    async fn get_attributes(&self, _path: &Path) -> Result<Box<dyn FileAttributesTrait>> {
        // Implementation should be provided by specific read-only filesystem provider
        unimplemented!("Method not implemented for base ReadOnlyFileSystem")
    }

    async fn exists(&self, _path: &Path) -> Result<bool> {
        // Implementation should be provided by specific read-only filesystem provider
        unimplemented!("Method not implemented for base ReadOnlyFileSystem")
    }
}

#[async_trait]
impl ReadableFileSystem for ReadOnlyFileSystem {
    async fn open_file(&self, _path: &Path) -> Result<Box<dyn FileReader>> {
        // Implementation should be provided by specific read-only filesystem provider
        unimplemented!("Method not implemented for base ReadOnlyFileSystem")
    }

    async fn read_to_string(&self, _path: &Path) -> Result<String> {
        // Implementation should be provided by specific read-only filesystem provider
        unimplemented!("Method not implemented for base ReadOnlyFileSystem")
    }

    async fn read_to_bytes(&self, _path: &Path) -> Result<Vec<u8>> {
        // Implementation should be provided by specific read-only filesystem provider
        unimplemented!("Method not implemented for base ReadOnlyFileSystem")
    }

    async fn get_permissions(&self, _path: &Path) -> Result<FilePermissions> {
        // Default implementation for read-only file systems
        Ok(FilePermissions {
            readable: true,
            writable: false,
            executable: false,
        })
    }
}

#[async_trait]
impl StreamableFileSystem for ReadOnlyFileSystem {
    async fn stream_file(&self, _path: &Path) -> Result<Box<dyn FileStream>> {
        // Implementation should be provided by specific read-only filesystem provider
        unimplemented!("Method not implemented for base ReadOnlyFileSystem")
    }
}

/// A read-only memory-based file system that stores files in memory
pub struct InMemoryReadOnlyFileSystem {
    inner: ReadOnlyFileSystem,
    files: std::collections::HashMap<PathBuf, Vec<u8>>,
    directories: std::collections::HashSet<PathBuf>,
}

impl InMemoryReadOnlyFileSystem {
    /// Create a new in-memory read-only file system
    pub fn new(name: &str) -> Self {
        let mut directories = std::collections::HashSet::new();
        directories.insert(PathBuf::from("/")); // Root folder always exists
        
        Self {
            inner: ReadOnlyFileSystem::new(name, Path::new("/")),
            files: std::collections::HashMap::new(),
            directories,
        }
    }
    
    /// Add a file to the in-memory file system
    pub fn add_file(&mut self, path: &Path, content: Vec<u8>) -> Result<()> {
        // Ensure parent folders exist
        let mut current = PathBuf::new();
        for component in path.parent().unwrap_or(Path::new("")).components() {
            current.push(component);
            self.directories.insert(current.clone());
        }
        
        self.files.insert(path.to_path_buf(), content);
        Ok(())
    }
    
    /// Add a folder to the in-memory file system
    pub fn add_folder(&mut self, path: &Path) -> Result<()> {
        let path_buf = path.to_path_buf();
        self.directories.insert(path_buf.clone());
        
        // Ensure parent folders exist
        let mut current = path_buf.clone();
        while current.pop() {
            if !current.as_os_str().is_empty() {
                self.directories.insert(current.clone());
            } else {
                break;
            }
        }
        
        Ok(())
    }
}

#[async_trait]
impl FileSystem for InMemoryReadOnlyFileSystem {
    fn get_type(&self) -> FileSystemType {
        self.inner.get_type()
    }

    fn get_capabilities(&self) -> FileSystemCapabilities {
        self.inner.get_capabilities()
    }

    async fn is_available(&self) -> Result<bool> {
        Ok(true)
    }

    async fn list_folder(&self, path: &Path) -> Result<Vec<Box<dyn FileAttributesTrait>>> {
        let path_buf = path.to_path_buf();
        
        // Check if path exists
        if !self.directories.contains(&path_buf) && !self.files.contains_key(&path_buf) {
            return Err(Error::from_string(format!("Path not found: {}", path.display())));
        }
        
        // List all files and folders whose parent is the given path
        let mut results = Vec::new();
        
        // Add subfolders
        for dir in &self.directories {
            if let Some(parent) = dir.parent() {
                if parent == path {
                    let attributes = FileAttributes::new(
                        GenericPath::from(PosixPath::new(dir)),
                        0,
                        None,
                        None,
                        true,
                        false,
                        false,
                        false,
                    );
                    results.push(Box::new(attributes) as Box<dyn FileAttributesTrait>);
                }
            }
        }
        
        // Add files
        for (file_path, content) in &self.files {
            if let Some(parent) = file_path.parent() {
                if parent == path {
                    let attributes = FileAttributes::new(
                        GenericPath::from(PosixPath::new(file_path)),
                        content.len() as u64,
                        None,
                        None,
                        false,
                        true,
                        false,
                        file_path.file_name()
                            .and_then(|s| s.to_str())
                            .map(|s| s.starts_with("."))
                            .unwrap_or(false),
                    );
                    results.push(Box::new(attributes) as Box<dyn FileAttributesTrait>);
                }
            }
        }
        
        Ok(results)
    }

    async fn get_attributes(&self, path: &Path) -> Result<Box<dyn FileAttributesTrait>> {
        let path_buf = path.to_path_buf();
        
        if self.directories.contains(&path_buf) {
            return Ok(Box::new(FileAttributes::new(
                GenericPath::from(PosixPath::new(path)),
                0,
                None,
                None,
                true,
                false,
                false,
                false,
            )));
        } else if let Some(content) = self.files.get(&path_buf) {
            return Ok(Box::new(FileAttributes::new(
                GenericPath::from(PosixPath::new(path)),
                content.len() as u64,
                None,
                None,
                false,
                true,
                false,
                path.file_name()
                    .and_then(|s| s.to_str())
                    .map(|s| s.starts_with("."))
                    .unwrap_or(false),
            )));
        }
        
        Err(Error::from_string("File or folder not found"))
    }

    async fn exists(&self, path: &Path) -> Result<bool> {
        Ok(self.directories.contains(&path.to_path_buf()) || 
           self.files.contains_key(&path.to_path_buf()))
    }
}

#[async_trait]
impl ReadableFileSystem for InMemoryReadOnlyFileSystem {
    async fn open_file(&self, path: &Path) -> Result<Box<dyn FileReader>> {
        if let Some(content) = self.files.get(&path.to_path_buf()) {
            return Ok(Box::new(InMemoryFileReader {
                data: content.clone(),
                position: 0,
            }));
        }
        
        Err(Error::from_string("File not found"))
    }

    async fn read_to_string(&self, path: &Path) -> Result<String> {
        if let Some(content) = self.files.get(&path.to_path_buf()) {
            return String::from_utf8(content.clone())
                .map_err(|e| Error::from_string(format!("Invalid UTF-8: {}", e)));
        }
        
        Err(Error::from_string("File not found"))
    }

    async fn read_to_bytes(&self, path: &Path) -> Result<Vec<u8>> {
        if let Some(content) = self.files.get(&path.to_path_buf()) {
            return Ok(content.clone());
        }
        
        Err(Error::from_string("File not found"))
    }

    async fn get_permissions(&self, _path: &Path) -> Result<FilePermissions> {
        // All files in memory read-only file system have the same permissions
        Ok(FilePermissions {
            readable: true,
            writable: false,
            executable: false,
        })
    }
}

#[async_trait]
impl StreamableFileSystem for InMemoryReadOnlyFileSystem {
    async fn stream_file(&self, path: &Path) -> Result<Box<dyn FileStream>> {
        if let Some(content) = self.files.get(&path.to_path_buf()) {
            return Ok(Box::new(InMemoryFileStream {
                reader: InMemoryFileReader {
                    data: content.clone(),
                    position: 0,
                },
            }));
        }
        
        Err(Error::from_string("File not found"))
    }
}

/// In-memory file reader
pub struct InMemoryFileReader {
    data: Vec<u8>,
    position: usize,
}

impl Read for InMemoryFileReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.position >= self.data.len() {
            return Ok(0);
        }
        
        let bytes_available = self.data.len() - self.position;
        let bytes_to_read = std::cmp::min(bytes_available, buf.len());
        
        buf[..bytes_to_read].copy_from_slice(&self.data[self.position..self.position + bytes_to_read]);
        self.position += bytes_to_read;
        
        Ok(bytes_to_read)
    }
}

impl FileReader for InMemoryFileReader {
    fn position(&self) -> io::Result<u64> {
        Ok(self.position as u64)
    }
    
    fn close(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// In-memory file stream
pub struct InMemoryFileStream {
    reader: InMemoryFileReader,
}

impl FileStream for InMemoryFileStream {
    fn read_next(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf)
    }
    
    fn skip(&mut self, n: u64) -> io::Result<u64> {
        let original_pos = self.reader.position;
        let new_pos_u64 = (original_pos as u64) + n;
        
        if new_pos_u64 > self.reader.data.len() as u64 {
            self.reader.position = self.reader.data.len();
            Ok((self.reader.data.len() - original_pos) as u64)
        } else {
            self.reader.position = new_pos_u64 as usize;
            Ok(n)
        }
    }
    
    fn close(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Factory for in-memory read-only file system
pub struct InMemoryReadOnlyFileSystemFactory {
    name: String,
    files: std::collections::HashMap<PathBuf, Vec<u8>>,
    directories: std::collections::HashSet<PathBuf>,
}

impl InMemoryReadOnlyFileSystemFactory {
    /// Create a new factory for in-memory read-only file system
    pub fn new(name: &str) -> Self {
        let mut directories = std::collections::HashSet::new();
        directories.insert(PathBuf::from("/")); // Root folder always exists
        
        Self {
            name: name.to_string(),
            files: std::collections::HashMap::new(),
            directories,
        }
    }
    
    /// Add a file to the factory
    pub fn add_file(&mut self, path: &Path, content: Vec<u8>) -> &mut Self {
        // Ensure parent folders exist
        let mut current = PathBuf::new();
        for component in path.parent().unwrap_or(Path::new("")).components() {
            current.push(component);
            self.directories.insert(current.clone());
        }
        
        self.files.insert(path.to_path_buf(), content);
        self
    }
    
    /// Add a folder to the factory
    pub fn add_folder(&mut self, path: &Path) -> &mut Self {
        self.directories.insert(path.to_path_buf());
        self
    }
}

impl FileSystemFactory for InMemoryReadOnlyFileSystemFactory {
    fn create(&self) -> Result<Box<dyn FileSystem>> {
        let mut fs = InMemoryReadOnlyFileSystem::new(&self.name);
        
        // Copy all files and folders
        for (path, content) in &self.files {
            fs.add_file(path, content.clone())?;
        }
        
        for path in &self.directories {
            fs.add_folder(path)?;
        }
        
        Ok(Box::new(fs))
    }
} 