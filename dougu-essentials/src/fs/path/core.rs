use crate::core::error::Result;
use std::any::Any;
use std::fmt::Debug;

use super::local::LocalPath;

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
pub trait Path: Debug {
    type ComponentsType: PathComponents;
    type NamespaceType: Namespace;
    
    /// Create a new empty path
    fn new() -> Self where Self: Sized;
    
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
    fn to_local_path(&self) -> Option<Box<dyn LocalPath<ComponentsType = Self::ComponentsType, NamespaceType = Self::NamespaceType>>>;
    
    /// Convert this object to Any for downcasting
    fn as_any(&self) -> &dyn Any where Self: 'static;
    
    /// Check if this path starts with the specified path
    /// Returns true if this path starts with the given path (considering all components)
    fn starts_with(&self, other: &Self) -> bool where Self: Sized {
        if self.namespace().as_str() != other.namespace().as_str() {
            return false;
        }

        let self_components = self.components();
        let other_components = other.components();
        
        if other_components.len() > self_components.len() {
            return false;
        }
        
        for i in 0..other_components.len() {
            if self_components.get(i) != other_components.get(i) {
                return false;
            }
        }
        
        true
    }
    
    /// Check if this path ends with the specified path
    /// Returns true if this path ends with the given path (considering components)
    fn ends_with(&self, other: &Self) -> bool where Self: Sized {
        let self_components = self.components();
        let other_components = other.components();
        
        if other_components.len() > self_components.len() {
            return false;
        }
        
        let offset = self_components.len() - other_components.len();
        
        for i in 0..other_components.len() {
            if self_components.get(i + offset) != other_components.get(i) {
                return false;
            }
        }
        
        true
    }
    
    /// Creates a relative path between this path and the given path
    /// Returns a path that, when resolved against this path, yields the other path
    /// Returns None if the two paths don't have the same namespace or cannot be relativized
    ///
    /// # Examples
    ///
    /// ```
    /// # use dougu_essentials::fs::Path; // Conceptual example only
    /// # fn example() {
    /// // Example 1: Basic path traversal
    /// // If a = "/home/user/docs"
    /// // and b = "/home/user/docs/project/file.txt"
    /// // Then a.relativize(b) = "project/file.txt"
    ///
    /// // Example 2: Moving up directories
    /// // If a = "/home/user/docs/project1"
    /// // and b = "/home/user/docs/project2/file.txt"
    /// // Then a.relativize(b) = "../project2/file.txt"
    ///
    /// // Example 3: No common prefix
    /// // If a = "/home/user/docs"
    /// // and b = "/var/log/messages"
    /// // Then a.relativize(b) = "../../../var/log/messages"
    ///
    /// // Example 4: Same paths
    /// // If a = "/home/user/docs"
    /// // and b = "/home/user/docs"
    /// // Then a.relativize(b) = "" (empty path)
    ///
    /// // Example 5: Different namespaces
    /// // If a = "drive1:/path"
    /// // and b = "drive2:/path"
    /// // Then a.relativize(b) = None (cannot relativize)
    /// # }
    /// ```
    fn relativize(&self, other: &Self) -> Option<Self> where Self: Sized {
        // Paths must have the same namespace to be relativized
        if self.namespace().as_str() != other.namespace().as_str() {
            return None;
        }
        
        // Both paths should be absolute or both should be relative
        if self.is_absolute() != other.is_absolute() {
            return None;
        }
        
        let self_components = self.components();
        let other_components = other.components();
        
        // Find common prefix length
        let mut common_prefix_len = 0;
        let min_len = std::cmp::min(self_components.len(), other_components.len());
        
        for i in 0..min_len {
            if self_components.get(i) != other_components.get(i) {
                break;
            }
            common_prefix_len += 1;
        }
        
        // Create new empty path
        let mut result = Self::new();
        
        // Add parent directory references for each component in this path beyond the common prefix
        for _ in common_prefix_len..self_components.len() {
            result.components_mut().push("..");
        }
        
        // Add remaining components from other path
        for i in common_prefix_len..other_components.len() {
            if let Some(component) = other_components.get(i) {
                result.components_mut().push(component);
            }
        }
        
        Some(result)
    }
}

/// PathProvider is a factory trait for creating Path instances.
pub trait PathProvider {
    type PathType: Path;
    
    /// Create a new path from a string
    fn create_path(&self, path_str: &str) -> Result<Self::PathType>;
    
    /// Get a unique identifier for this path provider (e.g., "local", "dropbox", "jira")
    fn provider_id(&self) -> &str;
} 