pub mod resources;

pub use resources::ErrorMessages;

use crate::core::error::Result;
use std::fs;
use std::path::Path;

/// Ensures a directory exists, creating it and any parent directories if necessary
pub fn ensure_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}
