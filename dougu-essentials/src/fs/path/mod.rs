// File path abstraction module

// Internal modules
mod component;
mod traits;
mod generic;
mod posix;
mod url;
mod windows;

// Export core components
pub use component::Component;
pub use traits::{PathFormat, PathTrait};
pub use generic::GenericPath;
pub use posix::PosixPath;
pub use url::UrlPath;
pub use windows::WindowsPath;

// Helper functions for creating paths from strings
pub fn from_str(s: &str, format: PathFormat) -> GenericPath {
    match format {
        PathFormat::Posix => posix_path(s),
        PathFormat::Windows => windows_path(s),
        PathFormat::UrlLike => url_path(s),
    }
}

/// Create a POSIX path from a string
pub fn posix_path(s: &str) -> GenericPath {
    GenericPath::from(PosixPath::new(s))
}

/// Create a Windows path from a string
pub fn windows_path(s: &str) -> GenericPath {
    GenericPath::from(WindowsPath::new(s))
}

/// Create a URL path from a string
pub fn url_path(s: &str) -> GenericPath {
    match UrlPath::parse(s) {
        Ok(path) => GenericPath::from(path),
        Err(_) => {
            // Fall back to a generic string path if URL parsing fails
            // This should only happen for invalid URLs
            posix_path(s)
        }
    }
}

/// Create a Windows UNC path
pub fn windows_unc_path(server: &str, share: &str, path: Option<&str>) -> GenericPath {
    let mut unc_path = format!("\\\\{}\\{}", server, share);
    if let Some(p) = path {
        if !p.starts_with('\\') {
            unc_path.push('\\');
        }
        unc_path.push_str(p);
    }
    windows_path(&unc_path)
}

// Type aliases for convenience
pub type Path = GenericPath;
pub type PathBuf = GenericPath;