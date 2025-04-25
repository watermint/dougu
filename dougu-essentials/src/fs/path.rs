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
    
    /// Check if this path starts with the specified path
    /// Returns true if this path starts with the given path (considering all components)
    fn starts_with(&self, other: &Self) -> bool {
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
    fn ends_with(&self, other: &Self) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec::Vec;

    #[derive(Debug, Clone)]
    struct MockNamespace {
        value: String,
    }

    impl Namespace for MockNamespace {
        fn as_str(&self) -> &str {
            &self.value
        }

        fn is_empty(&self) -> bool {
            self.value.is_empty()
        }

        fn from_string(s: &str) -> Self {
            MockNamespace { value: s.to_string() }
        }
    }

    #[derive(Debug, Clone)]
    struct MockPathComponents {
        components: Vec<String>,
    }

    impl PathComponents for MockPathComponents {
        fn new() -> Self {
            MockPathComponents { components: Vec::new() }
        }

        fn len(&self) -> usize {
            self.components.len()
        }

        fn is_empty(&self) -> bool {
            self.components.is_empty()
        }

        fn get(&self, index: usize) -> Option<&str> {
            self.components.get(index).map(|s| s.as_str())
        }

        fn push(&mut self, component: &str) {
            self.components.push(component.to_string());
        }

        fn pop(&mut self) -> Option<String> {
            self.components.pop()
        }

        fn join(&self) -> String {
            self.components.join("/")
        }

        fn normalize(&mut self) {
            // Basic normalization for tests
            let mut normalized = Vec::new();
            for comp in &self.components {
                if comp == "." {
                    continue;
                } else if comp == ".." {
                    if !normalized.is_empty() && normalized.last().unwrap() != ".." {
                        normalized.pop();
                    } else {
                        normalized.push(comp.clone());
                    }
                } else {
                    normalized.push(comp.clone());
                }
            }
            self.components = normalized;
        }

        fn from_string(path: &str) -> Self {
            let components = path.split('/')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
            MockPathComponents { components }
        }
    }

    #[derive(Debug, Clone)]
    struct MockPath {
        namespace: MockNamespace,
        components: MockPathComponents,
        is_absolute: bool,
    }

    impl Path for MockPath {
        type ComponentsType = MockPathComponents;
        type NamespaceType = MockNamespace;

        fn new() -> Self {
            MockPath {
                namespace: MockNamespace { value: String::new() },
                components: MockPathComponents::new(),
                is_absolute: false,
            }
        }

        fn namespace(&self) -> &Self::NamespaceType {
            &self.namespace
        }

        fn namespace_mut(&mut self) -> &mut Self::NamespaceType {
            &mut self.namespace
        }

        fn components(&self) -> &Self::ComponentsType {
            &self.components
        }

        fn components_mut(&mut self) -> &mut Self::ComponentsType {
            &mut self.components
        }

        fn parse(path_str: &str) -> Result<Self> {
            let is_absolute = path_str.starts_with('/');
            
            // Split namespace and path
            let mut parts = path_str.splitn(2, ':');
            let namespace_str = parts.next().unwrap_or("");
            let path_part = parts.next().unwrap_or("");
            
            let namespace = if path_part.is_empty() {
                // No namespace specified
                MockNamespace::from_string("")
            } else {
                MockNamespace::from_string(namespace_str)
            };
            
            let path_to_parse = if path_part.is_empty() { namespace_str } else { path_part };
            
            Ok(MockPath {
                namespace,
                components: MockPathComponents::from_string(path_to_parse),
                is_absolute,
            })
        }

        fn to_string(&self) -> String {
            let ns = if self.namespace.is_empty() {
                String::new()
            } else {
                format!("{}:", self.namespace.as_str())
            };
            
            let path = self.components.join();
            
            if self.is_absolute {
                format!("{}/{}", ns, path)
            } else {
                format!("{}{}", ns, path)
            }
        }

        fn join(&self, relative: &str) -> Result<Self> {
            let mut result = self.clone();
            let rel_components = MockPathComponents::from_string(relative);
            
            for i in 0..rel_components.len() {
                if let Some(component) = rel_components.get(i) {
                    result.components_mut().push(component);
                }
            }
            
            Ok(result)
        }

        fn parent(&self) -> Option<Self> {
            if self.components().is_empty() {
                return None;
            }
            
            let mut parent = self.clone();
            parent.components_mut().pop();
            Some(parent)
        }

        fn file_name(&self) -> Option<String> {
            if self.components().is_empty() {
                None
            } else {
                self.components().get(self.components().len() - 1).map(|s| s.to_string())
            }
        }

        fn normalize(&mut self) {
            self.components_mut().normalize();
        }

        fn is_absolute(&self) -> bool {
            self.is_absolute
        }

        fn to_local_path(&self) -> Option<Box<dyn LocalPath>> {
            None // Mock doesn't support conversion to local path
        }
    }

    // Helper function to create paths for testing
    fn create_path(path_str: &str, is_absolute: bool, namespace: &str) -> MockPath {
        let mut path = MockPath {
            namespace: MockNamespace::from_string(namespace),
            components: MockPathComponents::from_string(path_str),
            is_absolute,
        };
        path.normalize();
        path
    }

    #[test]
    fn test_starts_with() {
        // Test case 1: Same namespace, path starts with other
        let path1 = create_path("a/b/c/d", true, "test");
        let path2 = create_path("a/b", true, "test");
        assert!(path1.starts_with(&path2));
        
        // Test case 2: Same path but different namespace
        let path3 = create_path("a/b/c/d", true, "other");
        assert!(!path1.starts_with(&path3));
        
        // Test case 3: Other path is longer
        let path4 = create_path("a/b/c/d/e", true, "test");
        assert!(!path1.starts_with(&path4));
        
        // Test case 4: Path doesn't start with other
        let path5 = create_path("x/y/z", true, "test");
        assert!(!path1.starts_with(&path5));
        
        // Test case 5: Empty path
        let path6 = create_path("", true, "test");
        assert!(path1.starts_with(&path6));
    }

    #[test]
    fn test_ends_with() {
        // Test case 1: Path ends with other
        let path1 = create_path("a/b/c/d", true, "test");
        let path2 = create_path("c/d", false, "other"); // Namespace shouldn't matter for ends_with
        assert!(path1.ends_with(&path2));
        
        // Test case 2: Other path is longer
        let path3 = create_path("x/y/a/b/c/d", true, "test");
        assert!(!path1.ends_with(&path3));
        
        // Test case 3: Path doesn't end with other
        let path4 = create_path("x/y", false, "test");
        assert!(!path1.ends_with(&path4));
        
        // Test case 4: Empty path
        let path5 = create_path("", false, "test");
        assert!(path1.ends_with(&path5));
    }

    #[test]
    fn test_relativize() {
        // Test case 1: Basic relativization
        let path1 = create_path("a/b/c", true, "test");
        let path2 = create_path("a/b/c/d/e", true, "test");
        let relative = path1.relativize(&path2).unwrap();
        assert_eq!(relative.to_string(), "d/e");
        
        // Test case 2: Going up directories
        let path3 = create_path("a/b/c", true, "test");
        let path4 = create_path("a/b/x/y", true, "test");
        let relative = path3.relativize(&path4).unwrap();
        assert_eq!(relative.to_string(), "../x/y");
        
        // Test case 3: Common root only
        let path5 = create_path("a/b/c", true, "test");
        let path6 = create_path("a/x/y/z", true, "test");
        let relative = path5.relativize(&path6).unwrap();
        assert_eq!(relative.to_string(), "../../x/y/z");
        
        // Test case 4: No common elements
        let path7 = create_path("a/b/c", true, "test");
        let path8 = create_path("x/y/z", true, "test");
        let relative = path7.relativize(&path8).unwrap();
        assert_eq!(relative.to_string(), "../../../x/y/z");
        
        // Test case 5: Same path
        let path9 = create_path("a/b/c", true, "test");
        let path10 = create_path("a/b/c", true, "test");
        let relative = path9.relativize(&path10).unwrap();
        assert_eq!(relative.to_string(), "");
        
        // Test case 6: Different namespace
        let path11 = create_path("a/b/c", true, "test1");
        let path12 = create_path("a/b/c", true, "test2");
        assert!(path11.relativize(&path12).is_none());
        
        // Test case 7: One absolute, one relative
        let path13 = create_path("a/b/c", true, "test");
        let path14 = create_path("a/b/c", false, "test");
        assert!(path13.relativize(&path14).is_none());
    }
} 