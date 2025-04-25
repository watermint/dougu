use crate::core::error::Result;
use std::fmt::Debug;

/// PathComponents represents the parts of a path separated by delimiters.
/// This abstraction allows for path normalization across different systems.
pub trait PathComponents: Debug + Clone {
    /// Create a new empty set of components
    fn new() -> Self;

    /// Get the number of components in this path
    fn len(&self) -> usize;

    /// Check if path has no components
    fn is_empty(&self) -> bool;
    
    /// Get a specific component at the given index
    fn get(&self, index: usize) -> Option<&str>;
    
    /// Add a component to the end of the path
    fn push(&mut self, component: &str);
    
    /// Remove the last component and return it
    fn pop(&mut self) -> Option<String>;

    /// Join all components with the system-specific delimiter
    fn join(&self) -> String;
    
    /// Normalize the path by resolving `.`, `..`, and other redundancies
    fn normalize(&mut self);
    
    /// Create from string representation using system-specific delimiter
    fn from_string(path: &str) -> Self;
}

/// Namespace represents the authority part of a path.
/// Examples include:
/// - Drive letter for Windows (C:)
/// - Server name for network paths (\\server)
/// - Connection name or namespace ID for cloud storage
/// - Service identifiers for pseudo file systems
pub trait Namespace: Debug + Clone {
    /// Get the string representation of this namespace
    fn as_str(&self) -> &str;
    
    /// Check if this namespace is empty
    fn is_empty(&self) -> bool;
    
    /// Create a new namespace from a string
    fn from_string(s: &str) -> Self;
}

/// Path is an abstract representation of a path across different file systems.
/// It contains a namespace and components, allowing for consistent path manipulation.
pub trait Path: Debug + Clone {
    type ComponentsType: PathComponents;
    type NamespaceType: Namespace;
    
    /// Create a new empty path
    fn new() -> Self;
    
    /// Get a reference to the namespace part of this path
    fn namespace(&self) -> &Self::NamespaceType;
    
    /// Get a mutable reference to the namespace part of this path
    fn namespace_mut(&mut self) -> &mut Self::NamespaceType;
    
    /// Get a reference to the components part of this path
    fn components(&self) -> &Self::ComponentsType;
    
    /// Get a mutable reference to the components part of this path
    fn components_mut(&mut self) -> &mut Self::ComponentsType;
    
    /// Parse a string into a path
    fn parse(path_str: &str) -> Result<Self> where Self: Sized;
    
    /// Convert this path to a string representation
    fn to_string(&self) -> String;
    
    /// Join this path with a relative path
    fn join(&self, relative: &str) -> Result<Self> where Self: Sized;
    
    /// Get the parent path
    fn parent(&self) -> Option<Self> where Self: Sized;
    
    /// Get the file name (last component)
    fn file_name(&self) -> Option<String>;
    
    /// Normalize this path by removing redundancies
    fn normalize(&mut self);
    
    /// Check if this path is absolute
    fn is_absolute(&self) -> bool;
    
    /// Check if this path is relative
    fn is_relative(&self) -> bool {
        !self.is_absolute()
    }
    
    /// Convert this path to the local OS format if possible
    /// Returns None if the path cannot be represented as a local path
    fn to_local_path(&self) -> Option<Box<dyn LocalPath>>;
}

/// LocalPathType represents the specific implementation of a local path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalPathType {
    /// POSIX style paths (/usr/bin)
    Posix,
    
    /// Windows style paths (C:\Windows)
    Windows,
    
    /// Windows UNC paths (\\server\share)
    WindowsUNC,
}

/// LocalPath is a specialized path implementation for local file systems.
/// It supports different local path types (POSIX, Windows, Windows UNC).
pub trait LocalPath: Path + Debug {
    /// Get the path type of this local path
    fn path_type(&self) -> LocalPathType;
    
    /// Convert to a path string in the specified format
    fn to_path_string(&self, target_type: LocalPathType) -> String;
    
    /// Create a path in the current OS format
    fn create_os_path(path: &str) -> Result<Self> where Self: Sized;
    
    /// Get the current OS path type
    fn os_path_type() -> LocalPathType;
    
    /// Validate if this is a valid path according to the operating system rules
    fn validate(&self) -> Result<()>;
}

/// PathProvider is a factory trait for creating Path instances.
pub trait PathProvider {
    type PathType: Path;
    
    /// Create a new path from a string
    fn create_path(&self, path_str: &str) -> Result<Self::PathType>;
    
    /// Get a unique identifier for this path provider (e.g., "local", "dropbox", "jira")
    fn provider_id(&self) -> &str;
}

/// Factory function for creating a local path with the OS-specific implementation
pub fn create_local_path<P: LocalPath + 'static>(path: &str) -> Result<Box<dyn LocalPath>> {
    // This function will be implemented to return the appropriate
    // OS-specific LocalPath implementation
    #[cfg(target_family = "unix")]
    {
        // Return POSIX implementation when actually implemented
        unimplemented!("POSIX path implementation not provided yet")
    }
    
    #[cfg(target_family = "windows")]
    {
        // Return Windows implementation when actually implemented
        unimplemented!("Windows path implementation not provided yet")
    }
}

/// Get the default local path type for the current OS
pub fn default_path_type() -> LocalPathType {
    #[cfg(target_family = "unix")]
    {
        LocalPathType::Posix
    }
    
    #[cfg(target_family = "windows")]
    {
        LocalPathType::Windows
    }
} 