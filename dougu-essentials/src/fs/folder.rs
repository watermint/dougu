use std::fs;
use std::path::Path;
use crate::core::error::Result;

/// Ensures that the specified folder exists, creating it and any parent folders if necessary.
/// 
/// # Arguments
///
/// * `folder` - The folder path to ensure exists
///
/// # Returns
///
/// * `Ok(())` if the folder exists or was successfully created
/// * `Err` if the folder could not be created
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use dougu_essentials::fs::ensure_folder;
///
/// // Ensure the logs folder exists
/// ensure_folder(Path::new("logs")).unwrap();
/// ```
pub fn ensure_folder<P: AsRef<Path>>(folder: P) -> Result<()> {
    let path = folder.as_ref();
    if !path.exists() {
        fs::create_dir_all(path)?;
    } else if !path.is_dir() {
        let error_msg = format!("Path exists but is not a folder: {}", path.display());
        return Err(crate::core::error::Error::msg(error_msg));
    }
    Ok(())
} 