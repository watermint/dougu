use crate::archive::{ArchiveEntry, ArchiveMetadata, ArchiveProvider, EntryOptions, ExtractOptions};
use crate::archive::resources::error_messages;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tempfile::tempdir;
use zip::{ZipArchive, ZipWriter};
use zip::write::FileOptions;

pub struct ZipArchiveProvider;

impl ZipArchiveProvider {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl ArchiveProvider for ZipArchiveProvider {
    fn name(&self) -> &str {
        "zip"
    }
    
    async fn create_archive(&self, archive_path: &Path, sources: Vec<PathBuf>, options: EntryOptions) -> Result<()> {
        let file = std::fs::File::create(archive_path)?;
        let mut zip = ZipWriter::new(file);
        
        let compression_method = match options.compression_level {
            Some(level) => {
                if level == 0 {
                    zip::CompressionMethod::Stored
                } else {
                    zip::CompressionMethod::Deflated
                }
            },
            None => zip::CompressionMethod::Deflated,
        };
        
        let zip_options = FileOptions::default()
            .compression_method(compression_method);
        
        for source in sources {
            if !source.exists() {
                return Err(anyhow!(error_messages::RESOURCE_NOT_FOUND));
            }
            
            if source.is_dir() {
                // Scan the directory recursively
                for entry in walkdir::WalkDir::new(&source) {
                    let entry = entry?;
                    let path = entry.path();
                    
                    // Skip directories in the archive, they'll be created automatically
                    if path.is_dir() {
                        continue;
                    }
                    
                    // Calculate the entry name relative to the source path
                    let entry_name = path.strip_prefix(&source)?;
                    let entry_name = entry_name.to_string_lossy();
                    
                    // Read and add the file
                    let mut file = std::fs::File::open(path)?;
                    let mut buffer = Vec::new();
                    file.read_to_end(&mut buffer)?;
                    
                    zip.start_file(entry_name.to_string(), zip_options)?;
                    zip.write_all(&buffer)?;
                }
            } else {
                // Add a single file
                let file_name = source.file_name()
                    .ok_or_else(|| anyhow!(error_messages::INVALID_PATH))?
                    .to_string_lossy()
                    .to_string();
                
                let mut file = std::fs::File::open(&source)?;
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                
                zip.start_file(file_name, zip_options)?;
                zip.write_all(&buffer)?;
            }
        }
        
        zip.finish()?;
        Ok(())
    }
    
    async fn extract_archive(&self, archive_path: &Path, target_dir: &Path, options: ExtractOptions) -> Result<()> {
        // Open the ZIP archive
        let file = std::fs::File::open(archive_path)?;
        let mut archive = ZipArchive::new(file)?;
        
        // Create the target directory if it doesn't exist
        if !target_dir.exists() {
            fs::create_dir_all(target_dir)?;
        }
        
        // Extract each file
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => {
                    // Apply filtering if needed
                    if let Some(ref prefix) = options.filter_prefix {
                        let path_str = path.to_string_lossy();
                        if !path_str.starts_with(prefix) {
                            continue;
                        }
                    }
                    
                    target_dir.join(path)
                },
                None => continue,
            };
            
            // Skip if the file already exists and overwrite is false
            if outpath.exists() && !options.overwrite {
                continue;
            }
            
            if file.is_dir() {
                fs::create_dir_all(&outpath)?;
            } else {
                // Create parent directory if it doesn't exist
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p)?;
                    }
                }
                
                let mut outfile = fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
            
            // Get and Set permissions
            #[cfg(unix)]
            {
                if options.preserve_permissions {
                    if let Some(mode) = file.unix_mode() {
                        use std::os::unix::fs::PermissionsExt;
                        fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    async fn list_entries(&self, archive_path: &Path) -> Result<Vec<ArchiveEntry>> {
        let file = std::fs::File::open(archive_path)?;
        let mut archive = ZipArchive::new(file)?;
        let mut entries = Vec::new();
        
        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            let name = file.name().to_string();
            let path = name.clone();
            
            let entry = ArchiveEntry {
                metadata: ArchiveMetadata {
                    name,
                    path,
                    size: file.size(),
                    is_directory: file.is_dir(),
                    compressed_size: Some(file.compressed_size()),
                    last_modified: file.last_modified().and_then(|dt| {
                        dt.to_time().map(|t| t.unix_timestamp() as u64)
                    }),
                },
                provider_info: None,
            };
            
            entries.push(entry);
        }
        
        Ok(entries)
    }
    
    async fn extract_entry(&self, archive_path: &Path, entry_path: &str, target_path: &Path) -> Result<()> {
        let file = std::fs::File::open(archive_path)?;
        let mut archive = ZipArchive::new(file)?;
        
        // Try to find the entry
        let mut entry = archive.by_name(entry_path)
            .map_err(|_| anyhow!(error_messages::ENTRY_NOT_FOUND))?;
        
        if entry.is_dir() {
            // Create the directory if it doesn't exist
            fs::create_dir_all(target_path)?;
        } else {
            // Create parent directory if needed
            if let Some(parent) = target_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }
            
            // Extract the file
            let mut outfile = fs::File::create(target_path)?;
            std::io::copy(&mut entry, &mut outfile)?;
        }
        
        // Set permissions if requested
        #[cfg(unix)]
        {
            if let Some(mode) = entry.unix_mode() {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(target_path, fs::Permissions::from_mode(mode))?;
            }
        }
        
        Ok(())
    }
    
    async fn add_entry(&self, archive_path: &Path, source_path: &Path, entry_name: &str, options: EntryOptions) -> Result<()> {
        // We need to:
        // 1. Create a temporary copy of the archive
        // 2. Extract all files
        // 3. Add/update the new entry
        // 4. Create a new archive
        // 5. Replace the original archive
        
        // Create a temporary directory
        let temp_dir = tempdir()?;
        let temp_extract_dir = temp_dir.path().join("extract");
        let temp_archive_path = temp_dir.path().join("temp_archive.zip");
        
        // Create extract directory
        fs::create_dir_all(&temp_extract_dir)?;
        
        // Extract the original archive if it exists
        if archive_path.exists() {
            let extract_options = ExtractOptions {
                overwrite: true,
                preserve_permissions: options.preserve_permissions,
                filter_prefix: None,
            };
            self.extract_archive(archive_path, &temp_extract_dir, extract_options).await?;
        }
        
        // Prepare the target path in the extraction directory
        let target_entry_path = temp_extract_dir.join(entry_name);
        
        // Create parent directories if needed
        if let Some(parent) = target_entry_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        
        // Copy the source file/directory to the temporary location
        if source_path.is_dir() {
            // Copy directory recursively
            for entry in walkdir::WalkDir::new(source_path) {
                let entry = entry?;
                let rel_path = entry.path().strip_prefix(source_path)?;
                let target_path = target_entry_path.join(rel_path);
                
                if entry.path().is_dir() {
                    fs::create_dir_all(&target_path)?;
                } else {
                    if let Some(parent) = target_path.parent() {
                        if !parent.exists() {
                            fs::create_dir_all(parent)?;
                        }
                    }
                    fs::copy(entry.path(), &target_path)?;
                }
            }
        } else {
            // Copy a single file
            fs::copy(source_path, &target_entry_path)?;
        }
        
        // Create a new archive with all files including the new/updated entry
        let create_options = EntryOptions {
            compression_level: options.compression_level,
            preserve_permissions: options.preserve_permissions,
        };
        
        // Get all paths in the temporary directory
        let mut sources = Vec::new();
        for entry in fs::read_dir(&temp_extract_dir)? {
            let entry = entry?;
            sources.push(entry.path());
        }
        
        // Create the new archive
        self.create_archive(&temp_archive_path, sources, create_options).await?;
        
        // Replace the original archive
        fs::rename(&temp_archive_path, archive_path)?;
        
        Ok(())
    }
    
    async fn entry_exists(&self, archive_path: &Path, entry_path: &str) -> Result<bool> {
        let file = std::fs::File::open(archive_path)?;
        let mut archive = ZipArchive::new(file)?;
        
        Ok(archive.by_name(entry_path).is_ok())
    }
    
    async fn get_entry_metadata(&self, archive_path: &Path, entry_path: &str) -> Result<ArchiveMetadata> {
        let file = std::fs::File::open(archive_path)?;
        let mut archive = ZipArchive::new(file)?;
        
        let entry = archive.by_name(entry_path)
            .map_err(|_| anyhow!(error_messages::ENTRY_NOT_FOUND))?;
        
        Ok(ArchiveMetadata {
            name: entry.name().to_string(),
            path: entry_path.to_string(),
            size: entry.size(),
            is_directory: entry.is_dir(),
            compressed_size: Some(entry.compressed_size()),
            last_modified: entry.last_modified().and_then(|dt| {
                dt.to_time().map(|t| t.unix_timestamp() as u64)
            }),
        })
    }
} 