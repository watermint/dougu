// Implementation of local file system operations

use crate::core::Result;
use crate::time::SystemTime;
use crate::fs::{
    FileAttributes, FileAttributesTrait, FilePermissions, FileReader, FileStream, FileSystem,
    FileSystemCapabilities, FileSystemFactory, FileSystemType, FileWriter, GenericPath,
    ReadableFileSystem, SeekableFileReader, SeekableFileWriter, StreamableFileSystem, PosixPath,
    path_from_str,
};
use async_trait::async_trait;
use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// Local file system implementation
pub struct LocalFileSystem {
    capabilities: FileSystemCapabilities,
}

impl LocalFileSystem {
    /// Create a new instance of local file system
    pub fn new() -> Self {
        Self {
            capabilities: FileSystemCapabilities {
                can_create: true,
                can_delete: true,
                can_move: true,
                can_rename: true,
                can_read: true,
                can_write: true,
                can_append: true,
                can_seek: true,
                can_stream: true,
                can_link: false,
                can_set_permissions: true,
                can_get_permissions: true,
            },
        }
    }

    /// Convert std::fs::Metadata to FileAttributes
    fn convert_metadata(&self, path: &GenericPath, metadata: &Metadata) -> FileAttributes {
        FileAttributes::new(
            path.clone(),
            metadata.len(),
            metadata.modified().ok().map(SystemTime::from_std),
            metadata.created().ok().map(SystemTime::from_std),
            metadata.is_dir(),
            metadata.is_file(),
            metadata.file_type().is_symlink(),
            path.file_name()
                .map(|name| name.starts_with("."))
                .unwrap_or(false),
        )
    }

    /// Convert std::fs::Permissions to FilePermissions
    fn convert_permissions(&self, permissions: &fs::Permissions) -> FilePermissions {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = permissions.mode();
            FilePermissions {
                readable: (mode & 0o444) != 0,
                writable: (mode & 0o222) != 0 && !permissions.readonly(),
                executable: (mode & 0o111) != 0,
            }
        }

        #[cfg(not(unix))]
        {
            FilePermissions {
                readable: true,
                writable: !permissions.readonly(),
                executable: false,
            }
        }
    }
    
    /// Get standard path from generic path
    fn get_std_path(&self, path: &GenericPath) -> Result<PathBuf> {
        path.to_std_path().ok_or_else(|| {
            crate::core::error::Error::from_string(
                format!("Cannot convert path to standard path: {}", path)
            )
        })
    }
}

impl Default for LocalFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FileSystem for LocalFileSystem {
    fn get_type(&self) -> FileSystemType {
        FileSystemType::Local
    }

    fn get_capabilities(&self) -> FileSystemCapabilities {
        self.capabilities
    }

    async fn is_available(&self) -> Result<bool> {
        Ok(true)
    }

    async fn list_folder(&self, path: &GenericPath) -> Result<Vec<Box<dyn FileAttributesTrait>>> {
        let std_path = self.get_std_path(path)?;
        let entries = fs::read_dir(std_path)?;
        
        let mut results = Vec::new();
        for entry in entries {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let path = GenericPath::from(PosixPath::new(entry.path()));
            results.push(Box::new(self.convert_metadata(&path, &metadata)) as Box<dyn FileAttributesTrait>);
        }
        
        Ok(results)
    }

    async fn get_attributes(&self, path: &GenericPath) -> Result<Box<dyn FileAttributesTrait>> {
        let std_path = self.get_std_path(path)?;
        let metadata = fs::metadata(std_path)?;
        Ok(Box::new(self.convert_metadata(path, &metadata)))
    }

    async fn exists(&self, path: &GenericPath) -> Result<bool> {
        let std_path = self.get_std_path(path)?;
        Ok(std_path.exists())
    }
}

#[async_trait]
impl WriteableFileSystem for LocalFileSystem {
    async fn create_file(&self, path: &GenericPath) -> Result<Box<dyn FileWriter>> {
        let std_path = self.get_std_path(path)?;
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(std_path)?;
        
        Ok(Box::new(LocalFileWriter {
            writer: BufWriter::new(file),
            position: 0,
        }))
    }

    async fn write_file(&self, path: &GenericPath) -> Result<Box<dyn FileWriter>> {
        let std_path = self.get_std_path(path)?;
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(std_path)?;
        
        Ok(Box::new(LocalFileWriter {
            writer: BufWriter::new(file),
            position: 0,
        }))
    }

    async fn append_file(&self, path: &GenericPath) -> Result<Box<dyn FileWriter>> {
        let std_path = self.get_std_path(path)?;
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(std_path)?;
        
        let position = file.metadata()?.len();
        
        Ok(Box::new(LocalFileWriter {
            writer: BufWriter::new(file),
            position,
        }))
    }

    async fn create_folder(&self, path: &GenericPath) -> Result<()> {
        let std_path = self.get_std_path(path)?;
        fs::create_dir(std_path)?;
        Ok(())
    }

    async fn create_folder_all(&self, path: &GenericPath) -> Result<()> {
        let std_path = self.get_std_path(path)?;
        fs::create_dir_all(std_path)?;
        Ok(())
    }

    async fn delete_file(&self, path: &GenericPath) -> Result<()> {
        let std_path = self.get_std_path(path)?;
        fs::remove_file(std_path)?;
        Ok(())
    }

    async fn delete_folder(&self, path: &GenericPath) -> Result<()> {
        let std_path = self.get_std_path(path)?;
        fs::remove_dir(std_path)?;
        Ok(())
    }

    async fn delete_folder_all(&self, path: &GenericPath) -> Result<()> {
        let std_path = self.get_std_path(path)?;
        fs::remove_dir_all(std_path)?;
        Ok(())
    }

    async fn rename(&self, from: &GenericPath, to: &GenericPath) -> Result<()> {
        let std_from = self.get_std_path(from)?;
        let std_to = self.get_std_path(to)?;
        fs::rename(std_from, std_to)?;
        Ok(())
    }

    async fn copy_file(&self, from: &GenericPath, to: &GenericPath) -> Result<()> {
        let std_from = self.get_std_path(from)?;
        let std_to = self.get_std_path(to)?;
        fs::copy(std_from, std_to)?;
        Ok(())
    }

    async fn move_file(&self, from: &GenericPath, to: &GenericPath) -> Result<()> {
        let std_from = self.get_std_path(from)?;
        let std_to = self.get_std_path(to)?;
        fs::rename(std_from, std_to)?;
        Ok(())
    }

    async fn set_permissions(&self, path: &GenericPath, permissions: FilePermissions) -> Result<()> {
        let std_path = self.get_std_path(path)?;
        let mut fs_permissions = fs::metadata(std_path.clone())?.permissions();
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut mode = fs_permissions.mode() & !0o777; // Clear rwx bits
            
            if permissions.readable {
                mode |= 0o444;
            }
            if permissions.writable {
                mode |= 0o222;
            }
            if permissions.executable {
                mode |= 0o111;
            }
            
            fs_permissions.set_mode(mode);
        }
        
        #[cfg(not(unix))]
        {
            fs_permissions.set_readonly(!permissions.writable);
        }
        
        fs::set_permissions(std_path, fs_permissions)?;
        Ok(())
    }
}

#[async_trait]
impl ReadableFileSystem for LocalFileSystem {
    async fn open_file(&self, path: &GenericPath) -> Result<Box<dyn FileReader>> {
        let std_path = self.get_std_path(path)?;
        let file = File::open(std_path)?;
        Ok(Box::new(LocalFileReader {
            reader: BufReader::new(file),
            position: 0,
        }))
    }

    async fn read_to_string(&self, path: &GenericPath) -> Result<String> {
        let std_path = self.get_std_path(path)?;
        let content = fs::read_to_string(std_path)?;
        Ok(content)
    }

    async fn read_to_bytes(&self, path: &GenericPath) -> Result<Vec<u8>> {
        let std_path = self.get_std_path(path)?;
        let content = fs::read(std_path)?;
        Ok(content)
    }

    async fn get_permissions(&self, path: &GenericPath) -> Result<FilePermissions> {
        let std_path = self.get_std_path(path)?;
        let metadata = fs::metadata(std_path)?;
        Ok(self.convert_permissions(&metadata.permissions()))
    }
}

#[async_trait]
impl StreamableFileSystem for LocalFileSystem {
    async fn stream_file(&self, path: &GenericPath) -> Result<Box<dyn FileStream>> {
        let std_path = self.get_std_path(path)?;
        let file = File::open(std_path)?;
        Ok(Box::new(LocalFileStream {
            reader: BufReader::new(file),
        }))
    }
}

/// Local file writer implementation
pub struct LocalFileWriter {
    writer: BufWriter<File>,
    position: u64,
}

impl Write for LocalFileWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let bytes_written = self.writer.write(buf)?;
        self.position += bytes_written as u64;
        Ok(bytes_written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl FileWriter for LocalFileWriter {
    fn sync_all(&mut self) -> io::Result<()> {
        self.writer.flush()?;
        self.writer.get_ref().sync_all()
    }

    fn sync_data(&mut self) -> io::Result<()> {
        self.writer.flush()?;
        self.writer.get_ref().sync_data()
    }

    fn position(&self) -> io::Result<u64> {
        Ok(self.position)
    }

    fn close(&mut self) -> io::Result<()> {
        self.flush()
    }
}

impl Seek for LocalFileWriter {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.writer.flush()?;
        let new_pos = self.writer.get_mut().seek(pos)?;
        self.position = new_pos;
        Ok(new_pos)
    }
}

impl SeekableFileWriter for LocalFileWriter {}

/// Local file reader implementation
pub struct LocalFileReader {
    reader: BufReader<File>,
    position: u64,
}

impl Read for LocalFileReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let bytes_read = self.reader.read(buf)?;
        self.position += bytes_read as u64;
        Ok(bytes_read)
    }
}

impl FileReader for LocalFileReader {
    fn position(&self) -> io::Result<u64> {
        Ok(self.position)
    }

    fn close(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Seek for LocalFileReader {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_pos = self.reader.seek(pos)?;
        self.position = new_pos;
        Ok(new_pos)
    }
}

impl SeekableFileReader for LocalFileReader {}

/// Local file stream implementation
pub struct LocalFileStream {
    reader: BufReader<File>,
}

impl FileStream for LocalFileStream {
    fn read_next(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf)
    }

    fn skip(&mut self, n: u64) -> io::Result<u64> {
        let current = self.reader.stream_position()?;
        let new_pos = self.reader.seek(SeekFrom::Current(n as i64))?;
        Ok(new_pos - current)
    }

    fn close(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Factory for local file system
pub struct LocalFileSystemFactory;

impl FileSystemFactory for LocalFileSystemFactory {
    fn create(&self) -> Result<Box<dyn FileSystem>> {
        Ok(Box::new(LocalFileSystem::new()))
    }
} 