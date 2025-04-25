// Generic path wrapper implementation

use super::component::Component;
use super::traits::{PathFormat, PathTrait};
use super::posix::PosixPath;
use super::url::UrlPath;
use super::windows::WindowsPath;
use std::fmt::{Debug, Display};
use std::path::PathBuf as StdPathBuf;
use std::sync::Arc;

/// Universal path for all file systems
#[derive(Clone, Debug)]
pub struct GenericPath {
    /// Inner implementation
    inner: Arc<dyn PathTrait>,
}

impl GenericPath {
    /// Create a new path from a PathTrait implementation
    pub fn new<P: PathTrait + 'static>(path: P) -> Self {
        Self {
            inner: Arc::new(path),
        }
    }
    
    /// Get the path format
    pub fn format(&self) -> PathFormat {
        self.inner.format()
    }
    
    /// Check if this path is absolute
    pub fn is_absolute(&self) -> bool {
        self.inner.is_absolute()
    }
    
    /// Check if this path is relative
    pub fn is_relative(&self) -> bool {
        self.inner.is_relative()
    }
    
    /// Get the file name (last component)
    pub fn file_name(&self) -> Option<String> {
        self.inner.file_name()
    }
    
    /// Get the parent path
    pub fn parent(&self) -> Option<Self> {
        self.inner.parent().map(|p| Self::new(p))
    }
    
    /// Check if the path has a root
    pub fn has_root(&self) -> bool {
        self.inner.has_root()
    }
    
    /// Check if the path is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    
    /// Join with another path
    pub fn join(&self, path: &Self) -> Self {
        Self::new(self.inner.join(path))
    }
    
    /// Convert to standard Path if possible
    pub fn to_std_path(&self) -> Option<StdPathBuf> {
        self.inner.to_std_path()
    }
    
    /// Get path components
    pub fn components(&self) -> Vec<Component> {
        self.inner.components()
    }
    
    /// Check if this path is a prefix of another path
    pub fn starts_with(&self, base: &Self) -> bool {
        self.inner.starts_with(base)
    }
    
    /// Check if this path ends with the specified path
    pub fn ends_with(&self, child: &Self) -> bool {
        self.inner.ends_with(child)
    }
    
    /// Normalize the path (resolve . and .. components)
    pub fn normalize(&self) -> Self {
        Self::new(self.inner.normalize())
    }
    
    /// Get the extension
    pub fn extension(&self) -> Option<String> {
        self.inner.extension()
    }
    
    /// Get the file stem (file name without extension)
    pub fn file_stem(&self) -> Option<String> {
        self.inner.file_stem()
    }
    
    /// Get file name with extension replaced
    pub fn with_extension(&self, extension: &str) -> Self {
        Self::new(self.inner.with_extension(extension))
    }
    
    /// Get file name replaced
    pub fn with_file_name(&self, file_name: &str) -> Self {
        Self::new(self.inner.with_file_name(file_name))
    }
    
    /// Try to convert this path to a PosixPath
    pub fn as_posix_path(&self) -> Option<PosixPath> {
        if self.format() == PathFormat::Posix {
            // Try to downcast to PosixPath
            if let Some(std_path) = self.to_std_path() {
                return Some(PosixPath::new(std_path));
            }
        }
        None
    }
    
    /// Try to convert this path to a UrlPath
    pub fn as_url_path(&self) -> Option<UrlPath> {
        if self.format() == PathFormat::UrlLike {
            // Try to parse the string representation
            UrlPath::parse(&self.to_string()).ok()
        } else {
            None
        }
    }
    
    /// Try to convert this path to a WindowsPath
    pub fn as_windows_path(&self) -> Option<WindowsPath> {
        if self.format() == PathFormat::Windows {
            // Try to downcast to WindowsPath
            if let Some(std_path) = self.to_std_path() {
                return Some(WindowsPath::new(std_path));
            }
        }
        None
    }
    
    /// Convert to a WindowsPath (for Windows-specific operations)
    pub fn to_windows_path(&self) -> WindowsPath {
        if let Some(win_path) = self.as_windows_path() {
            win_path
        } else if let Some(std_path) = self.to_std_path() {
            WindowsPath::new(std_path)
        } else {
            // Fall back to string conversion
            WindowsPath::new(self.to_string())
        }
    }
    
    /// Convert to a UNC path on Windows
    pub fn to_unc_path(&self) -> Option<GenericPath> {
        if self.format() == PathFormat::Windows {
            let win_path = self.to_windows_path();
            Some(GenericPath::new(win_path.to_unc()))
        } else {
            None
        }
    }
}

impl Display for GenericPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl From<PosixPath> for GenericPath {
    fn from(path: PosixPath) -> Self {
        Self::new(path)
    }
}

impl From<UrlPath> for GenericPath {
    fn from(path: UrlPath) -> Self {
        Self::new(path)
    }
}

impl From<WindowsPath> for GenericPath {
    fn from(path: WindowsPath) -> Self {
        Self::new(path)
    }
} 