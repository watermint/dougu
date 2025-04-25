// Posix path implementation for Unix-like paths

use super::component::Component;
use super::traits::{PathFormat, PathTrait};
use std::fmt::Display;
use std::path::{Path as StdPath, PathBuf as StdPathBuf};
use crate::core::error::Error;
use crate::core::Result;

/// Posix path implementation
#[derive(Clone, Debug)]
pub struct PosixPath {
    /// Inner path buffer
    path: StdPathBuf,
}

impl PosixPath {
    /// Create a new Posix path
    pub fn new<P: AsRef<StdPath>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }
    
    /// Create a new Posix path, validating that it exists
    pub fn new_existing<P: AsRef<StdPath>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(Error::from_string(format!("Path does not exist: {}", path.display())));
        }
        Ok(Self::new(path))
    }
    
    /// Get the inner path buffer
    pub fn as_path_buf(&self) -> &StdPathBuf {
        &self.path
    }
    
    /// Convert to inner path buffer
    pub fn into_path_buf(self) -> StdPathBuf {
        self.path
    }
    
    /// Join with another path and verify the result exists
    pub fn join_existing<P: AsRef<StdPath>>(&self, path: P) -> Result<Self> {
        let joined = self.path.join(path.as_ref());
        if !joined.exists() {
            return Err(Error::from_string(format!("Joined path does not exist: {}", joined.display())));
        }
        Ok(Self { path: joined })
    }
}

impl PathTrait for PosixPath {
    fn format(&self) -> PathFormat {
        PathFormat::Posix
    }
    
    fn is_absolute(&self) -> bool {
        self.path.is_absolute()
    }
    
    fn to_string(&self) -> String {
        self.path.to_string_lossy().to_string()
    }
    
    fn file_name(&self) -> Option<String> {
        self.path.file_name().map(|s| s.to_string_lossy().to_string())
    }
    
    fn parent(&self) -> Option<Self> {
        self.path.parent().map(Self::new)
    }
    
    fn has_root(&self) -> bool {
        self.path.has_root()
    }
    
    fn is_empty(&self) -> bool {
        self.path == StdPathBuf::new()
    }
    
    fn join<P: AsRef<Self>>(&self, path: P) -> Self {
        let p = path.as_ref();
        
        // If the path to join is absolute, return it
        if p.is_absolute() {
            return p.clone();
        }
        
        Self::new(self.path.join(&p.path))
    }
    
    fn to_std_path(&self) -> Option<StdPathBuf> {
        Some(self.path.clone())
    }
    
    fn components(&self) -> Vec<Component> {
        let mut result = Vec::new();
        
        // Convert standard path components to our components
        for component in self.path.components() {
            match component {
                std::path::Component::Prefix(_) => {
                    // Skip prefix components (Windows-specific)
                },
                std::path::Component::RootDir => {
                    result.push(Component::root());
                },
                std::path::Component::CurDir => {
                    result.push(Component::current());
                },
                std::path::Component::ParentDir => {
                    result.push(Component::parent());
                },
                std::path::Component::Normal(s) => {
                    result.push(Component::new(s.to_string_lossy().as_ref()));
                },
            }
        }
        
        result
    }
    
    fn starts_with<P: AsRef<Self>>(&self, base: P) -> bool {
        if let Some(p) = base.as_ref().to_std_path() {
            self.path.starts_with(p)
        } else {
            false
        }
    }
    
    fn ends_with<P: AsRef<Self>>(&self, child: P) -> bool {
        if let Some(p) = child.as_ref().to_std_path() {
            self.path.ends_with(p)
        } else {
            false
        }
    }
    
    fn normalize(&self) -> Self {
        // Process path components to resolve relative references
        let mut components = Vec::new();
        
        for component in self.components() {
            if component.is_current {
                // Skip current directory references (.) as they don't affect the path
                continue;
            } else if component.is_parent {
                // For parent directory references (..), remove the last path component
                // unless we're already at the root
                if !components.is_empty() && !components.last().unwrap().is_root {
                    components.pop();
                }
            } else {
                components.push(component);
            }
        }
        
        // Reconstruct the normalized path
        let mut result = String::new();
        let mut first = true;
        
        for component in components {
            if component.is_root {
                result.push('/');
                first = true; // Don't add another separator after root
            } else {
                if !first {
                    result.push('/');
                }
                result.push_str(&component.name);
                first = false;
            }
        }
        
        // Ensure root paths have at least "/"
        if result.is_empty() && self.is_absolute() {
            result.push('/');
        }
        
        Self::new(result)
    }
    
    fn extension(&self) -> Option<String> {
        self.path.extension().map(|s| s.to_string_lossy().to_string())
    }
    
    fn file_stem(&self) -> Option<String> {
        self.path.file_stem().map(|s| s.to_string_lossy().to_string())
    }
    
    fn pop(&mut self) -> bool {
        self.path.pop()
    }
    
    fn push<P: AsRef<Self>>(&mut self, path: P) {
        if let Some(p) = path.as_ref().to_std_path() {
            self.path.push(p);
        }
    }
    
    fn with_extension(&self, extension: &str) -> Self {
        Self::new(self.path.with_extension(extension))
    }
    
    fn with_file_name(&self, file_name: &str) -> Self {
        Self::new(self.path.with_file_name(file_name))
    }
}

impl AsRef<PosixPath> for PosixPath {
    fn as_ref(&self) -> &PosixPath {
        self
    }
}

impl Display for PosixPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.display())
    }
} 