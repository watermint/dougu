// Path traits and interfaces

use super::component::Component;
use std::fmt::{Debug, Display};
use std::path::PathBuf as StdPathBuf;

/// Path format types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathFormat {
    /// Unix-style path format (Posix)
    Posix,
    /// Windows path format
    Windows,
    /// URL path format
    Url,
    /// Custom path format
    Custom(&'static str),
}

/// Trait for path abstraction
pub trait PathTrait: Debug + Display + Send + Sync + Clone {
    /// Get the path format
    fn format(&self) -> PathFormat;
    
    /// Check if this path is absolute
    fn is_absolute(&self) -> bool;
    
    /// Check if this path is relative
    fn is_relative(&self) -> bool {
        !self.is_absolute()
    }
    
    /// Convert to a string representation
    fn to_string(&self) -> String;
    
    /// Get the file name (last component)
    fn file_name(&self) -> Option<String>;
    
    /// Get the parent path
    fn parent(&self) -> Option<Self> where Self: Sized;
    
    /// Check if the path has a root
    fn has_root(&self) -> bool;
    
    /// Check if the path is empty
    fn is_empty(&self) -> bool;
    
    /// Join with another path
    fn join<P: AsRef<Self>>(&self, path: P) -> Self where Self: Sized;
    
    /// Convert to standard Path if possible
    fn to_std_path(&self) -> Option<StdPathBuf>;
    
    /// Get path components
    fn components(&self) -> Vec<Component>;
    
    /// Check if this path is a prefix of another path
    fn starts_with<P: AsRef<Self>>(&self, base: P) -> bool;
    
    /// Check if this path ends with the specified path
    fn ends_with<P: AsRef<Self>>(&self, child: P) -> bool;
    
    /// Normalize the path (resolve . and .. components)
    fn normalize(&self) -> Self;
    
    /// Get the extension
    fn extension(&self) -> Option<String>;
    
    /// Get the file stem (file name without extension)
    fn file_stem(&self) -> Option<String>;
    
    /// Pop the last component from the path
    fn pop(&mut self) -> bool;
    
    /// Push a component to the path
    fn push<P: AsRef<Self>>(&mut self, path: P);
    
    /// Get file name with extension replaced
    fn with_extension(&self, extension: &str) -> Self;
    
    /// Get file name replaced
    fn with_file_name(&self, file_name: &str) -> Self;
} 