use crate::core::error::{Error, Result};
use crate::fs::capability::{Capability, CapabilitySet};
use crate::fs::entry::{Entry, EntryMetadata, EntryType, FileEntry, FolderEntry, ReadWriteFile, ReadableFile, WritableFile};
use crate::fs::path::core::{Namespace, Path, PathComponents};
use crate::fs::path::default::{DefaultNamespace, DefaultPathComponents};
use crate::fs::path::local::{LocalPath, LocalPathType};
use crate::fs::provider::FileSystemProvider;
use crate::time::ZonedDateTime;
use std::fmt::Debug;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path as StdPath, PathBuf};

/// Metadata for a local file system entry
#[derive(Debug, Clone)]
pub struct LocalEntryMetadata {
    metadata: fs::Metadata,
    path: PathBuf,
}

impl LocalEntryMetadata {
    /// Creates a new metadata wrapper for a local file system entry
    pub fn new(path: PathBuf, metadata: fs::Metadata) -> Self {
        Self { metadata, path }
    }

    /// Gets the underlying std::fs::Metadata
    pub fn inner(&self) -> &fs::Metadata {
        &self.metadata
    }

    /// Gets the path this metadata is for
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl EntryMetadata for LocalEntryMetadata {
    fn entry_type(&self) -> EntryType {
        if self.metadata.is_file() {
            EntryType::File
        } else if self.metadata.is_dir() {
            EntryType::Folder
        } else if self.metadata.file_type().is_symlink() {
            EntryType::SymLink
        } else {
            EntryType::Other
        }
    }

    fn size(&self) -> u64 {
        self.metadata.len()
    }

    fn created(&self) -> Option<ZonedDateTime> {
        match self.metadata.created() {
            Ok(time) => Some(crate::time::system_time_to_zoned_date_time(time)),
            Err(_) => None,
        }
    }

    fn modified(&self) -> Option<ZonedDateTime> {
        match self.metadata.modified() {
            Ok(time) => Some(crate::time::system_time_to_zoned_date_time(time)),
            Err(_) => None,
        }
    }

    fn accessed(&self) -> Option<ZonedDateTime> {
        match self.metadata.accessed() {
            Ok(time) => Some(crate::time::system_time_to_zoned_date_time(time)),
            Err(_) => None,
        }
    }

    fn is_readonly(&self) -> bool {
        self.metadata.permissions().readonly()
    }

    fn is_hidden(&self) -> bool {
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
            match self.metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN {
                0 => false,
                _ => true,
            }
        }
        
        #[cfg(not(windows))]
        {
            // On Unix-like systems, files starting with a dot are considered hidden
            self.path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.starts_with('.'))
                .unwrap_or(false)
        }
    }
}

/// A wrapper for a local file that implements ReadableFile
#[derive(Debug)]
pub struct LocalReadableFile {
    file: File,
}

impl LocalReadableFile {
    /// Creates a new readable file wrapper
    pub fn new(file: File) -> Self {
        Self { file }
    }
}

impl Read for LocalReadableFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file.read(buf)
    }
}

impl Seek for LocalReadableFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.file.seek(pos)
    }
}

impl ReadableFile for LocalReadableFile {
    fn size(&self) -> Result<u64> {
        Ok(self.file.metadata()?.len())
    }
}

/// A wrapper for a local file that implements WritableFile
#[derive(Debug)]
pub struct LocalWritableFile {
    file: File,
}

impl LocalWritableFile {
    /// Creates a new writable file wrapper
    pub fn new(file: File) -> Self {
        Self { file }
    }
}

impl Write for LocalWritableFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl Seek for LocalWritableFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.file.seek(pos)
    }
}

impl WritableFile for LocalWritableFile {
    fn flush_all(&mut self) -> Result<()> {
        self.file.sync_all()?;
        Ok(())
    }

    fn size(&self) -> Result<u64> {
        Ok(self.file.metadata()?.len())
    }
}

/// A wrapper for a local file that implements ReadWriteFile
#[derive(Debug)]
pub struct LocalReadWriteFile {
    file: File,
}

impl LocalReadWriteFile {
    /// Creates a new read-write file wrapper
    pub fn new(file: File) -> Self {
        Self { file }
    }
}

impl Read for LocalReadWriteFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file.read(buf)
    }
}

impl Write for LocalReadWriteFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl Seek for LocalReadWriteFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.file.seek(pos)
    }
}

impl ReadWriteFile for LocalReadWriteFile {
    fn flush_all(&mut self) -> Result<()> {
        self.file.sync_all()?;
        Ok(())
    }

    fn size(&self) -> Result<u64> {
        Ok(self.file.metadata()?.len())
    }
}

/// Entry implementation for the local file system
#[derive(Debug)]
pub struct LocalEntry {
    path: Box<dyn LocalPath<ComponentsType=DefaultPathComponents, NamespaceType=DefaultNamespace> + 'static>,
}

impl LocalEntry {
    /// Creates a new local entry
    pub fn new(path: Box<dyn LocalPath<ComponentsType=DefaultPathComponents, NamespaceType=DefaultNamespace> + 'static>) -> Self {
        Self { path }
    }
    
    /// Gets the standard path for this entry
    pub fn std_path(&self) -> PathBuf {
        PathBuf::from(self.path.to_string())
    }
}

impl crate::fs::entry::Entry for LocalEntry {
    type PathType = Box<dyn LocalPath<ComponentsType=DefaultPathComponents, NamespaceType=DefaultNamespace> + 'static>;
    type MetadataType = LocalEntryMetadata;
    
    fn path(&self) -> &Self::PathType {
        &self.path
    }
    
    fn metadata(&self) -> Result<Self::MetadataType> {
        let metadata = std::fs::metadata(self.std_path())?;
        Ok(LocalEntryMetadata::new(self.std_path(), metadata))
    }
    
    fn exists(&self) -> Result<bool> {
        Ok(self.std_path().exists())
    }
}

/// File entry implementation for the local file system
#[derive(Debug)]
pub struct LocalFileEntry {
    entry: LocalEntry,
}

impl LocalFileEntry {
    /// Creates a new local file entry
    pub fn new(path: Box<dyn LocalPath<ComponentsType=DefaultPathComponents, NamespaceType=DefaultNamespace> + 'static>) -> Self {
        Self {
            entry: LocalEntry::new(path),
        }
    }
}

impl crate::fs::entry::Entry for LocalFileEntry {
    type PathType = Box<dyn LocalPath<ComponentsType=DefaultPathComponents, NamespaceType=DefaultNamespace> + 'static>;
    type MetadataType = LocalEntryMetadata;
    
    fn path(&self) -> &Self::PathType {
        self.entry.path()
    }
    
    fn metadata(&self) -> Result<Self::MetadataType> {
        self.entry.metadata()
    }
    
    fn exists(&self) -> Result<bool> {
        self.entry.exists()
    }
}

impl FileEntry for LocalFileEntry {
    fn open_read(&self) -> Result<Box<dyn ReadableFile>> {
        let file = std::fs::File::open(self.entry.std_path())?;
        Ok(Box::new(LocalReadableFile::new(file)))
    }

    fn open_write(&self) -> Result<Box<dyn WritableFile>> {
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(self.entry.std_path())?;
        Ok(Box::new(LocalWritableFile::new(file)))
    }

    fn open_append(&self) -> Result<Box<dyn WritableFile>> {
        let file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(self.entry.std_path())?;
        Ok(Box::new(LocalWritableFile::new(file)))
    }

    fn open_read_write(&self) -> Result<Box<dyn ReadWriteFile>> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(self.entry.std_path())?;
        Ok(Box::new(LocalReadWriteFile::new(file)))
    }

    fn create_new(&self) -> Result<Box<dyn WritableFile>> {
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(self.entry.std_path())?;
        Ok(Box::new(LocalWritableFile::new(file)))
    }

    fn delete(&self) -> Result<()> {
        fs::remove_file(self.entry.std_path())?;
        Ok(())
    }

    fn parent_folder(&self) -> Option<Box<dyn FolderEntry<PathType=Self::PathType, MetadataType=Self::MetadataType>>> {
        let std_path = self.entry.std_path();
        std_path.parent().map(|parent| {
            let path_enum = crate::fs::create_local_path(parent.to_string_lossy().as_ref()).unwrap();
            let boxed_path = crate::fs::path::path_enum_to_boxed_local_path(path_enum);
            Box::new(LocalFolderEntry::new(boxed_path)) as Box<dyn FolderEntry<PathType = Self::PathType, MetadataType = Self::MetadataType>>
        })
    }

    fn copy_to(&self, destination: &dyn FileEntry<PathType = Self::PathType, MetadataType = Self::MetadataType>) -> Result<()> {
        let dest_path = destination.path().to_string();
        std::fs::copy(self.entry.std_path(), dest_path)?;
        Ok(())
    }
    
    fn move_to(&self, destination: &dyn FileEntry<PathType = Self::PathType, MetadataType = Self::MetadataType>) -> Result<()> {
        let dest_path = destination.path().to_string();
        std::fs::rename(self.entry.std_path(), dest_path)?;
        Ok(())
    }
}

/// Folder entry implementation for the local file system
#[derive(Debug)]
pub struct LocalFolderEntry {
    entry: LocalEntry,
}

impl LocalFolderEntry {
    /// Creates a new local folder entry
    pub fn new(path: Box<dyn LocalPath<ComponentsType=DefaultPathComponents, NamespaceType=DefaultNamespace> + 'static>) -> Self {
        Self {
            entry: LocalEntry::new(path),
        }
    }
}

impl crate::fs::entry::Entry for LocalFolderEntry {
    type PathType = Box<dyn LocalPath<ComponentsType=DefaultPathComponents, NamespaceType=DefaultNamespace> + 'static>;
    type MetadataType = LocalEntryMetadata;
    
    fn path(&self) -> &Self::PathType {
        self.entry.path()
    }
    
    fn metadata(&self) -> Result<Self::MetadataType> {
        self.entry.metadata()
    }
    
    fn exists(&self) -> Result<bool> {
        self.entry.exists()
    }
}

impl FolderEntry for LocalFolderEntry {
    fn list_entries(&self) -> Result<Vec<Box<dyn crate::fs::entry::Entry<PathType = Self::PathType, MetadataType = Self::MetadataType>>>> {
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(self.entry.std_path())? {
            let entry = entry?;
            let path = entry.path();
            let path_enum = crate::fs::create_local_path(path.to_string_lossy().as_ref())?;
            let boxed_path = crate::fs::path::path_enum_to_boxed_local_path(path_enum);
            entries.push(Box::new(LocalEntry::new(boxed_path)) as Box<dyn crate::fs::entry::Entry<PathType = Self::PathType, MetadataType = Self::MetadataType>>);
        }
        Ok(entries)
    }
    
    fn list_files(&self) -> Result<Vec<Box<dyn FileEntry<PathType = Self::PathType, MetadataType = Self::MetadataType>>>> {
        let mut files = Vec::new();
        for entry in std::fs::read_dir(self.entry.std_path())? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let path_enum = crate::fs::create_local_path(path.to_string_lossy().as_ref())?;
                let boxed_path = crate::fs::path::path_enum_to_boxed_local_path(path_enum);
                files.push(Box::new(LocalFileEntry::new(boxed_path)) as Box<dyn FileEntry<PathType = Self::PathType, MetadataType = Self::MetadataType>>);
            }
        }
        Ok(files)
    }
    
    fn list_folders(&self) -> Result<Vec<Box<dyn FolderEntry<PathType = Self::PathType, MetadataType = Self::MetadataType>>>> {
        let mut folders = Vec::new();
        for entry in std::fs::read_dir(self.entry.std_path())? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let path_enum = crate::fs::create_local_path(path.to_string_lossy().as_ref())?;
                let boxed_path = crate::fs::path::path_enum_to_boxed_local_path(path_enum);
                folders.push(Box::new(LocalFolderEntry::new(boxed_path)) as Box<dyn FolderEntry<PathType = Self::PathType, MetadataType = Self::MetadataType>>);
            }
        }
        Ok(folders)
    }
    
    fn create(&self) -> Result<()> {
        std::fs::create_dir(self.entry.std_path())?;
        Ok(())
    }
    
    fn create_recursive(&self) -> Result<()> {
        std::fs::create_dir_all(self.entry.std_path())?;
        Ok(())
    }
    
    fn delete(&self) -> Result<()> {
        std::fs::remove_dir(self.entry.std_path())?;
        Ok(())
    }
    
    fn delete_recursive(&self) -> Result<()> {
        std::fs::remove_dir_all(self.entry.std_path())?;
        Ok(())
    }
    
    fn parent(&self) -> Option<Box<dyn FolderEntry<PathType = Self::PathType, MetadataType = Self::MetadataType>>> {
        let std_path = self.entry.std_path();
        std_path.parent().map(|parent| {
            let path_enum = crate::fs::create_local_path(parent.to_string_lossy().as_ref()).unwrap();
            let boxed_path = crate::fs::path::path_enum_to_boxed_local_path(path_enum);
            Box::new(LocalFolderEntry::new(boxed_path)) as Box<dyn FolderEntry<PathType = Self::PathType, MetadataType = Self::MetadataType>>
        })
    }
    
    fn get_file(&self, name: &str) -> Result<Box<dyn FileEntry<PathType = Self::PathType, MetadataType = Self::MetadataType>>> {
        let mut path = self.entry.std_path();
        path.push(name);
        
        if !path.exists() {
            return Err(Error::msg(format!("File {} does not exist", path.display())));
        }
        if !path.is_file() {
            return Err(Error::msg(format!("{} is not a file", path.display())));
        }
        
        let path_enum = crate::fs::create_local_path(path.to_string_lossy().as_ref())?;
        let boxed_path = crate::fs::path::path_enum_to_boxed_local_path(path_enum);
        Ok(Box::new(LocalFileEntry::new(boxed_path)) as Box<dyn FileEntry<PathType = Self::PathType, MetadataType = Self::MetadataType>>)
    }
    
    fn get_folder(&self, name: &str) -> Result<Box<dyn FolderEntry<PathType = Self::PathType, MetadataType = Self::MetadataType>>> {
        let mut path = self.entry.std_path();
        path.push(name);
        
        if !path.exists() {
            return Err(Error::msg(format!("Folder {} does not exist", path.display())));
        }
        if !path.is_dir() {
            return Err(Error::msg(format!("{} is not a folder", path.display())));
        }
        
        let path_enum = crate::fs::create_local_path(path.to_string_lossy().as_ref())?;
        let boxed_path = crate::fs::path::path_enum_to_boxed_local_path(path_enum);
        Ok(Box::new(LocalFolderEntry::new(boxed_path)) as Box<dyn FolderEntry<PathType = Self::PathType, MetadataType = Self::MetadataType>>)
    }
}

/// Local file system provider implementation
#[derive(Debug)]
pub struct LocalFileSystemProvider {
    capabilities: CapabilitySet,
}

impl LocalFileSystemProvider {
    /// Creates a new local file system provider
    pub fn new() -> Self {
        let mut capabilities = CapabilitySet::new();
        capabilities.add(Capability::Read);
        capabilities.add(Capability::Write);
        capabilities.add(Capability::Create);
        capabilities.add(Capability::Delete);
        capabilities.add(Capability::Move);
        capabilities.add(Capability::Copy);
        capabilities.add(Capability::Seek);
        capabilities.add(Capability::List);
        capabilities.add(Capability::Metadata);
        capabilities.add(Capability::Stream);
        
        Self { capabilities }
    }
}

impl Default for LocalFileSystemProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystemProvider for LocalFileSystemProvider {
    type PathType = Box<dyn LocalPath<ComponentsType=DefaultPathComponents, NamespaceType=DefaultNamespace> + 'static>;
    type EntryType = LocalEntry;
    type FileEntryType = LocalFileEntry;
    type FolderEntryType = LocalFolderEntry;
    type MetadataType = LocalEntryMetadata;

    fn provider_id(&self) -> &str {
        "local"
    }

    fn display_name(&self) -> &str {
        "Local File System"
    }

    fn capabilities(&self) -> &CapabilitySet {
        &self.capabilities
    }

    fn create_file_entry(&self, path: &Self::PathType) -> Result<Self::FileEntryType> {
        // Since we can't clone Box<dyn LocalPath>, we'll create a new path from the string representation
        let path_str = path.to_string();
        let path_enum = crate::fs::create_local_path(&path_str)?;
        let boxed_path = crate::fs::path::path_enum_to_boxed_local_path(path_enum);
        Ok(LocalFileEntry::new(boxed_path))
    }

    fn create_folder_entry(&self, path: &Self::PathType) -> Result<Self::FolderEntryType> {
        // Since we can't clone Box<dyn LocalPath>, we'll create a new path from the string representation
        let path_str = path.to_string();
        let path_enum = crate::fs::create_local_path(&path_str)?;
        let boxed_path = crate::fs::path::path_enum_to_boxed_local_path(path_enum);
        Ok(LocalFolderEntry::new(boxed_path))
    }

    fn get_entry(&self, path: &Self::PathType) -> Result<Self::EntryType> {
        // Since we can't clone Box<dyn LocalPath>, we'll create a new path from the string representation
        let path_str = path.to_string();
        let path_enum = crate::fs::create_local_path(&path_str)?;
        let boxed_path = crate::fs::path::path_enum_to_boxed_local_path(path_enum);
        Ok(LocalEntry::new(boxed_path))
    }

    fn get_root_folder(&self) -> Result<Self::FolderEntryType> {
        // For local file system, we use the current directory as the root
        let current_dir = std::env::current_dir()?;
        let path_enum = crate::fs::create_local_path(current_dir.to_string_lossy().as_ref())?;
        let boxed_path = crate::fs::path::path_enum_to_boxed_local_path(path_enum);
        Ok(LocalFolderEntry::new(boxed_path))
    }

    fn from_local_path(&self, local_path: &str) -> Result<Option<Self::PathType>> {
        let path_enum = crate::fs::create_local_path(local_path)?;
        let boxed_path = crate::fs::path::path_enum_to_boxed_local_path(path_enum);
        Ok(Some(boxed_path))
    }

    fn copy(
        &self,
        source: &Self::PathType,
        destination: &Self::PathType,
        _overwrite: bool,
    ) -> Result<()> {
        let src_path = source.to_std_path();
        let dst_path = destination.to_std_path();
        
        if src_path.is_file() {
            fs::copy(&src_path, &dst_path)?;
        } else if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        }
        
        Ok(())
    }

    fn move_entry(
        &self,
        source: &Self::PathType,
        destination: &Self::PathType,
        _overwrite: bool,
    ) -> Result<()> {
        let src_path = source.to_std_path();
        let dst_path = destination.to_std_path();
        
        fs::rename(src_path, dst_path)?;
        Ok(())
    }

    fn delete(&self, path: &Self::PathType, recursive: bool) -> Result<()> {
        let std_path = path.to_string();
        if std::path::Path::new(&std_path).is_dir() {
            if recursive {
                std::fs::remove_dir_all(std_path)?;
            } else {
                std::fs::remove_dir(std_path)?;
            }
        } else {
            std::fs::remove_file(std_path)?;
        }
        Ok(())
    }

    fn get_metadata(&self, path: &Self::PathType) -> Result<Self::MetadataType> {
        let std_path = std::path::PathBuf::from(path.to_string());
        let metadata = std::fs::metadata(&std_path)?;
        Ok(LocalEntryMetadata::new(std_path, metadata))
    }
}

/// Recursively copies a directory from source to destination
fn copy_dir_recursive(source: &StdPath, destination: &StdPath) -> Result<()> {
    if !destination.exists() {
        fs::create_dir_all(destination)?;
    }
    
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = destination.join(entry.file_name());
        
        if src_path.is_file() {
            fs::copy(&src_path, &dst_path)?;
        } else if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        }
    }
    
    Ok(())
}

// Extension trait for Box<dyn LocalPath> to convert to standard path
trait LocalPathExt {
    fn to_std_path(&self) -> PathBuf;
}

// Implement for boxed local paths
impl<C: PathComponents + 'static, N: Namespace + 'static> LocalPathExt for &Box<dyn LocalPath<ComponentsType=C, NamespaceType=N> + 'static> {
    fn to_std_path(&self) -> PathBuf {
        PathBuf::from((**self).to_string())
    }
} 