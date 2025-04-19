use crate::{ArchiveEntry, ArchiveMetadata, ArchiveProvider, EntryOptions, ExtractOptions};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use log::debug;
use std::fs::File;
use std::io::{Read, Write, Seek};
use std::path::{Path, PathBuf};
use tokio::task;
use zip::{ZipArchive, ZipWriter};
use zip::write::FileOptions;
use crate::resources::{error_messages, log_messages};

/// A provider that handles ZIP archives using the zip crate
pub struct ZipProvider;

impl ZipProvider {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl ArchiveProvider for ZipProvider {
    fn name(&self) -> &str {
        "zip"
    }
    
    async fn create_archive(&self, archive_path: &Path, sources: Vec<PathBuf>, _options: EntryOptions) -> Result<()> {
        debug!("{}: {}", log_messages::CREATING_ARCHIVE, archive_path.display());
        
        let archive_path = archive_path.to_path_buf(); // Clone path to avoid borrowed data escaping
        
        task::spawn_blocking(move || -> Result<()> {
            let file = File::create(&archive_path)
                .map_err(|e| anyhow!("{}: {}", error_messages::IO_ERROR, e))?;
                
            let mut zip = ZipWriter::new(file);
            let file_options = FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .unix_permissions(0o755);
                
            for source in sources {
                if !source.exists() {
                    return Err(anyhow!("{}: {}", error_messages::ENTRY_NOT_FOUND, source.display()));
                }
                
                if source.is_dir() {
                    add_directory_to_zip(&mut zip, source.as_path(), "", file_options)?;
                } else {
                    let file_name = source.file_name()
                        .ok_or_else(|| anyhow!("{}: {}", error_messages::INVALID_ENTRY_PATH, source.display()))?;
                    
                    let mut file = File::open(&source)
                        .map_err(|e| anyhow!("{}: {}", error_messages::IO_ERROR, e))?;
                    
                    zip.start_file(file_name.to_string_lossy(), file_options)
                        .map_err(|e| anyhow!("{}: {}", error_messages::COMPRESSION_ERROR, e))?;
                    
                    let mut buffer = Vec::new();
                    file.read_to_end(&mut buffer)
                        .map_err(|e| anyhow!("{}: {}", error_messages::IO_ERROR, e))?;
                    
                    zip.write_all(&buffer)
                        .map_err(|e| anyhow!("{}: {}", error_messages::COMPRESSION_ERROR, e))?;
                }
            }
            
            zip.finish()
                .map_err(|e| anyhow!("{}: {}", error_messages::COMPRESSION_ERROR, e))?;
                
            Ok(())
        }).await?
    }
    
    async fn extract_archive(&self, archive_path: &Path, target_dir: &Path, _options: ExtractOptions) -> Result<()> {
        debug!("{}: {}", log_messages::EXTRACTING_ARCHIVE, archive_path.display());
        
        let archive_path = PathBuf::from(archive_path);
        let target_dir = PathBuf::from(target_dir);
        
        task::spawn_blocking(move || -> Result<()> {
            let file = File::open(&archive_path)
                .map_err(|e| anyhow!("{}: {}", error_messages::ARCHIVE_NOT_FOUND, e))?;
                
            let mut archive = ZipArchive::new(file)
                .map_err(|e| anyhow!("{}: {}", error_messages::INVALID_ARCHIVE_FORMAT, e))?;
                
            for i in 0..archive.len() {
                let mut file = archive.by_index(i)
                    .map_err(|e| anyhow!("{}: {}", error_messages::DECOMPRESSION_ERROR, e))?;
                    
                let outpath = match file.enclosed_name() {
                    Some(path) => target_dir.join(path),
                    None => continue,
                };
                
                if (*file.name()).ends_with('/') {
                    std::fs::create_dir_all(&outpath)
                        .map_err(|e| anyhow!("{}: {}", error_messages::IO_ERROR, e))?;
                } else {
                    if let Some(p) = outpath.parent() {
                        if !p.exists() {
                            std::fs::create_dir_all(p)
                                .map_err(|e| anyhow!("{}: {}", error_messages::IO_ERROR, e))?;
                        }
                    }
                    
                    let mut outfile = File::create(&outpath)
                        .map_err(|e| anyhow!("{}: {}", error_messages::IO_ERROR, e))?;
                        
                    std::io::copy(&mut file, &mut outfile)
                        .map_err(|e| anyhow!("{}: {}", error_messages::IO_ERROR, e))?;
                }
            }
            
            Ok(())
        }).await?
    }
    
    async fn list_entries(&self, archive_path: &Path) -> Result<Vec<ArchiveEntry>> {
        debug!("{}: {}", log_messages::LISTING_ENTRIES, archive_path.display());
        
        let archive_path = PathBuf::from(archive_path);
        
        task::spawn_blocking(move || -> Result<Vec<ArchiveEntry>> {
            let file = File::open(&archive_path)
                .map_err(|e| anyhow!("{}: {}", error_messages::ARCHIVE_NOT_FOUND, e))?;
                
            let mut archive = ZipArchive::new(file)
                .map_err(|e| anyhow!("{}: {}", error_messages::INVALID_ARCHIVE_FORMAT, e))?;
                
            let mut entries = Vec::new();
            
            for i in 0..archive.len() {
                let file = archive.by_index(i)
                    .map_err(|e| anyhow!("{}: {}", error_messages::DECOMPRESSION_ERROR, e))?;
                    
                let name = file.name().to_string();
                let last_modified = file.last_modified().to_time()
                    .map(|dt| dt.unix_timestamp() as u64)
                    .unwrap_or(0);
                
                entries.push(ArchiveEntry {
                    metadata: ArchiveMetadata {
                        name: name.clone(),
                        path: name,
                        size: file.size(),
                        is_directory: file.is_dir(),
                        compressed_size: Some(file.compressed_size()),
                        last_modified: Some(last_modified),
                    },
                    provider_info: None,
                });
            }
            
            Ok(entries)
        }).await?
    }
    
    async fn extract_entry(&self, archive_path: &Path, entry_path: &str, target_path: &Path) -> Result<()> {
        debug!("{}: {}", log_messages::EXTRACTING_ENTRY, entry_path);
        
        let archive_path = PathBuf::from(archive_path);
        let target_path = PathBuf::from(target_path);
        let entry_path = entry_path.to_string();
        
        task::spawn_blocking(move || -> Result<()> {
            let file = File::open(&archive_path)
                .map_err(|e| anyhow!("{}: {}", error_messages::ARCHIVE_NOT_FOUND, e))?;
                
            let mut archive = ZipArchive::new(file)
                .map_err(|e| anyhow!("{}: {}", error_messages::INVALID_ARCHIVE_FORMAT, e))?;
                
            let mut file = match archive.by_name(&entry_path) {
                Ok(file) => file,
                Err(_) => return Err(anyhow!("{}: {}", error_messages::ENTRY_NOT_FOUND, entry_path)),
            };
            
            if file.is_dir() {
                std::fs::create_dir_all(&target_path)
                    .map_err(|e| anyhow!("{}: {}", error_messages::IO_ERROR, e))?;
                return Ok(());
            }
            
            if let Some(p) = target_path.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p)
                        .map_err(|e| anyhow!("{}: {}", error_messages::IO_ERROR, e))?;
                }
            }
            
            let mut outfile = File::create(&target_path)
                .map_err(|e| anyhow!("{}: {}", error_messages::IO_ERROR, e))?;
                
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| anyhow!("{}: {}", error_messages::IO_ERROR, e))?;
                
            Ok(())
        }).await?
    }
    
    async fn add_entry(&self, archive_path: &Path, source_path: &Path, entry_name: &str, _options: EntryOptions) -> Result<()> {
        debug!("{}: {}", log_messages::ADDING_ENTRY, entry_name);
        
        let archive_path = PathBuf::from(archive_path);
        let source_path = PathBuf::from(source_path);
        let entry_name = entry_name.to_string();
        
        task::spawn_blocking(move || -> Result<()> {
            let mut archive_file = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(&archive_path)
                .map_err(|e| anyhow!("{}: {}", error_messages::ARCHIVE_NOT_FOUND, e))?;
                
            let mut temp_file = tempfile::tempfile()
                .map_err(|e| anyhow!("{}: {}", error_messages::IO_ERROR, e))?;
                
            // Copy the original archive to a temp file
            std::io::copy(&mut archive_file, &mut temp_file)
                .map_err(|e| anyhow!("{}: {}", error_messages::IO_ERROR, e))?;
                
            temp_file.rewind()
                .map_err(|e| anyhow!("{}: {}", error_messages::IO_ERROR, e))?;
                
            // Read the original archive
            let mut original_archive = ZipArchive::new(temp_file)
                .map_err(|e| anyhow!("{}: {}", error_messages::INVALID_ARCHIVE_FORMAT, e))?;
                
            // Create a new archive
            archive_file.rewind()
                .map_err(|e| anyhow!("{}: {}", error_messages::IO_ERROR, e))?;
            archive_file.set_len(0)
                .map_err(|e| anyhow!("{}: {}", error_messages::IO_ERROR, e))?;
                
            let mut zip = ZipWriter::new(archive_file);
            let file_options = FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .unix_permissions(0o755);
                
            // Copy all entries except the one we're updating (if it exists)
            for i in 0..original_archive.len() {
                let mut file = original_archive.by_index(i)
                    .map_err(|e| anyhow!("{}: {}", error_messages::DECOMPRESSION_ERROR, e))?;
                    
                if file.name() == entry_name {
                    continue; // Skip this entry, we'll add the new one later
                }
                
                zip.start_file(file.name(), file_options)
                    .map_err(|e| anyhow!("{}: {}", error_messages::COMPRESSION_ERROR, e))?;
                    
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)
                    .map_err(|e| anyhow!("{}: {}", error_messages::IO_ERROR, e))?;
                    
                zip.write_all(&buffer)
                    .map_err(|e| anyhow!("{}: {}", error_messages::COMPRESSION_ERROR, e))?;
            }
            
            // Add the new entry
            if source_path.is_dir() {
                add_directory_to_zip(&mut zip, source_path.as_path(), &entry_name, file_options)?;
            } else {
                zip.start_file(entry_name, file_options)
                    .map_err(|e| anyhow!("{}: {}", error_messages::COMPRESSION_ERROR, e))?;
                    
                let mut file = File::open(&source_path)
                    .map_err(|e| anyhow!("{}: {}", error_messages::IO_ERROR, e))?;
                    
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)
                    .map_err(|e| anyhow!("{}: {}", error_messages::IO_ERROR, e))?;
                    
                zip.write_all(&buffer)
                    .map_err(|e| anyhow!("{}: {}", error_messages::COMPRESSION_ERROR, e))?;
            }
            
            zip.finish()
                .map_err(|e| anyhow!("{}: {}", error_messages::COMPRESSION_ERROR, e))?;
                
            Ok(())
        }).await?
    }
    
    async fn entry_exists(&self, archive_path: &Path, entry_path: &str) -> Result<bool> {
        debug!("{}: {}", log_messages::CHECKING_ENTRY_EXISTS, entry_path);
        
        let archive_path = PathBuf::from(archive_path);
        let entry_path = entry_path.to_string();
        
        task::spawn_blocking(move || -> Result<bool> {
            let file = File::open(&archive_path)
                .map_err(|e| anyhow!("{}: {}", error_messages::ARCHIVE_NOT_FOUND, e))?;
                
            let mut archive = ZipArchive::new(file)
                .map_err(|e| anyhow!("{}: {}", error_messages::INVALID_ARCHIVE_FORMAT, e))?;
                
            Ok(archive.by_name(&entry_path).is_ok())
        }).await?
    }
    
    async fn get_entry_metadata(&self, archive_path: &Path, entry_path: &str) -> Result<ArchiveMetadata> {
        debug!("{}: {}", log_messages::GETTING_ENTRY_METADATA, entry_path);
        
        let archive_path = PathBuf::from(archive_path);
        let entry_path = entry_path.to_string();
        
        task::spawn_blocking(move || -> Result<ArchiveMetadata> {
            let file = File::open(&archive_path)
                .map_err(|e| anyhow!("{}: {}", error_messages::ARCHIVE_NOT_FOUND, e))?;
                
            let mut archive = ZipArchive::new(file)
                .map_err(|e| anyhow!("{}: {}", error_messages::INVALID_ARCHIVE_FORMAT, e))?;
                
            let file = match archive.by_name(&entry_path) {
                Ok(file) => file,
                Err(_) => return Err(anyhow!("{}: {}", error_messages::ENTRY_NOT_FOUND, entry_path)),
            };
            
            let last_modified = file.last_modified().to_time()
                .map(|dt| dt.unix_timestamp() as u64)
                .unwrap_or(0);
            
            Ok(ArchiveMetadata {
                name: file.name().to_string(),
                path: file.name().to_string(),
                size: file.size(),
                is_directory: file.is_dir(),
                compressed_size: Some(file.compressed_size()),
                last_modified: Some(last_modified),
            })
        }).await?
    }
}

// Helper function to add a directory to a zip archive
fn add_directory_to_zip(
    zip: &mut ZipWriter<File>,
    dir_path: &Path,
    parent_path: &str,
    options: FileOptions,
) -> Result<()> {
    for entry in std::fs::read_dir(dir_path)
        .map_err(|e| anyhow!("{}: {}", error_messages::IO_ERROR, e))? 
    {
        let entry = entry.map_err(|e| anyhow!("{}: {}", error_messages::IO_ERROR, e))?;
        let path = entry.path();
        
        let name = path.file_name()
            .ok_or_else(|| anyhow!("{}: {}", error_messages::INVALID_ENTRY_PATH, path.display()))?
            .to_string_lossy();
            
        let entry_path = if parent_path.is_empty() {
            name.to_string()
        } else {
            format!("{}/{}", parent_path, name)
        };
        
        if path.is_dir() {
            // Add directory entry with trailing slash
            let dir_path = format!("{}/", entry_path);
            zip.add_directory(dir_path, options)
                .map_err(|e| anyhow!("{}: {}", error_messages::COMPRESSION_ERROR, e))?;
                
            // Recursively add directory contents
            add_directory_to_zip(zip, &path, &entry_path, options)?;
        } else {
            zip.start_file(entry_path, options)
                .map_err(|e| anyhow!("{}: {}", error_messages::COMPRESSION_ERROR, e))?;
                
            let mut file = File::open(&path)
                .map_err(|e| anyhow!("{}: {}", error_messages::IO_ERROR, e))?;
                
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)
                .map_err(|e| anyhow!("{}: {}", error_messages::IO_ERROR, e))?;
                
            zip.write_all(&buffer)
                .map_err(|e| anyhow!("{}: {}", error_messages::COMPRESSION_ERROR, e))?;
        }
    }
    
    Ok(())
} 