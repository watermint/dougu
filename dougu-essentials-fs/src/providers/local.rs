use crate::{FileSystemProvider, FileSystemEntry, FileMetadata, ReadOptions, WriteOptions};
use crate::resources::error_messages;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub struct LocalFileSystemProvider {
    root_dir: Option<PathBuf>,
}

impl LocalFileSystemProvider {
    pub fn new() -> Self {
        Self {
            root_dir: None,
        }
    }

    pub fn with_root<P: AsRef<Path>>(root: P) -> Self {
        Self {
            root_dir: Some(PathBuf::from(root.as_ref())),
        }
    }

    fn resolve_path(&self, path: &Path) -> PathBuf {
        match &self.root_dir {
            Some(root) => root.join(path),
            None => PathBuf::from(path),
        }
    }
}

#[async_trait]
impl FileSystemProvider for LocalFileSystemProvider {
    fn name(&self) -> &str {
        "local"
    }

    async fn list_directory(&self, path: &Path) -> Result<Vec<FileSystemEntry>> {
        let full_path = self.resolve_path(path);
        
        if !full_path.is_dir() {
            return Err(anyhow!(error_messages::NOT_A_DIRECTORY));
        }
        
        let mut entries = Vec::new();
        let mut readdir = fs::read_dir(&full_path).await?;
        
        while let Some(entry) = readdir.next_entry().await? {
            let path = entry.path();
            let metadata = entry.metadata().await?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            
            entries.push(FileSystemEntry {
                metadata: FileMetadata {
                    name: file_name,
                    path: path.to_string_lossy().to_string(),
                    size: metadata.len(),
                    is_directory: metadata.is_dir(),
                    last_modified: metadata.modified().ok().map(|time| {
                        time.duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0)
                    }),
                    content_hash: None,
                },
                provider_info: None,
            });
        }
        
        Ok(entries)
    }

    async fn read_file(&self, path: &Path, options: ReadOptions) -> Result<Vec<u8>> {
        let full_path = self.resolve_path(path);
        
        if !full_path.is_file() {
            return Err(anyhow!(error_messages::NOT_A_FILE));
        }
        
        let content = fs::read(&full_path).await?;
        
        // Apply read options if specified
        if options.offset.is_some() || options.length.is_some() {
            let offset = options.offset.unwrap_or(0) as usize;
            let end = match options.length {
                Some(len) => (offset as usize + len as usize).min(content.len()),
                None => content.len(),
            };
            
            if offset >= content.len() {
                return Ok(Vec::new());
            }
            
            return Ok(content[offset..end].to_vec());
        }
        
        Ok(content)
    }

    async fn write_file(&self, path: &Path, content: Vec<u8>, options: WriteOptions) -> Result<()> {
        let full_path = self.resolve_path(path);
        
        // Check if file exists and whether we should overwrite
        if full_path.exists() && !options.overwrite {
            return Err(anyhow!(error_messages::FILE_ALREADY_EXISTS));
        }
        
        // Create parent directories if specified
        if options.create_parents {
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).await?;
            }
        }
        
        let mut file = fs::File::create(&full_path).await?;
        file.write_all(&content).await?;
        
        Ok(())
    }

    async fn delete(&self, path: &Path, recursive: bool) -> Result<()> {
        let full_path = self.resolve_path(path);
        
        if !full_path.exists() {
            return Err(anyhow!(error_messages::RESOURCE_NOT_FOUND));
        }
        
        if full_path.is_dir() {
            if recursive {
                fs::remove_dir_all(&full_path).await?;
            } else {
                fs::remove_dir(&full_path).await?;
            }
        } else {
            fs::remove_file(&full_path).await?;
        }
        
        Ok(())
    }

    async fn create_directory(&self, path: &Path, create_parents: bool) -> Result<()> {
        let full_path = self.resolve_path(path);
        
        if full_path.exists() {
            if full_path.is_dir() {
                return Ok(());
            } else {
                return Err(anyhow!(error_messages::NOT_A_DIRECTORY));
            }
        }
        
        if create_parents {
            fs::create_dir_all(&full_path).await?;
        } else {
            fs::create_dir(&full_path).await?;
        }
        
        Ok(())
    }

    async fn get_metadata(&self, path: &Path) -> Result<FileMetadata> {
        let full_path = self.resolve_path(path);
        
        if !full_path.exists() {
            return Err(anyhow!(error_messages::RESOURCE_NOT_FOUND));
        }
        
        let metadata = fs::metadata(&full_path).await?;
        let name = full_path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| String::from(""));
        
        Ok(FileMetadata {
            name,
            path: full_path.to_string_lossy().to_string(),
            size: metadata.len(),
            is_directory: metadata.is_dir(),
            last_modified: metadata.modified().ok().map(|time| {
                time.duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0)
            }),
            content_hash: None,
        })
    }

    async fn exists(&self, path: &Path) -> Result<bool> {
        let full_path = self.resolve_path(path);
        Ok(full_path.exists())
    }
} 