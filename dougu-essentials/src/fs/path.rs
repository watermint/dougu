use crate::core::error::Result;
use std::fmt::Debug;

//
// Core Traits
//

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
    
    /// Convert this object to Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any where Self: 'static;
    
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

//
// Local File System Path Types
//

/// LocalPathType represents the specific implementation of a local path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalPathType {
    /// POSIX path format (/usr/bin)
    /// 
    /// Defined in IEEE Std 1003.1 (POSIX.1-2017) under "Pathname Resolution".
    /// Hierarchical naming system with forward slash (/) as directory separator.
    /// Absolute paths begin with a slash, relative paths do not.
    /// Special entries include "." (current directory) and ".." (parent directory).
    PosixPath,
    
    /// Network File System (NFS) path format (//server/share)
    /// 
    /// Defined in RFC 7530 (NFSv4) and RFC 1813 (NFSv3).
    /// Uses the format //server/path to identify remote resources.
    /// Typically mounted locally and then accessed through the local file system.
    NFSPath,
    
    /// Windows local path format (C:\Windows)
    /// 
    /// Defined in Microsoft Windows API documentation.
    /// Uses backslash (\) as directory separator and drive letters (e.g., C:) for storage volumes.
    /// Supports both absolute paths (C:\path) and relative paths (path\to\file).
    /// Maximum path length is typically 260 characters (MAX_PATH) but can be extended.
    WindowsPath,
    
    /// Universal Naming Convention (UNC) path format (\\server\share)
    /// 
    /// Defined in Microsoft SMB Protocol and CIFS Documentation.
    /// Format: \\server\share\path\to\resource
    /// Used for accessing network resources in Windows environments.
    /// Can include administrative shares (e.g., \\server\C$) and named shares.
    UNCPath,
    
    /// Server Message Block (SMB) URL format (smb://user:pass@server/path)
    /// 
    /// Defined in SNIA Technical Position: Common Internet File System (CIFS).
    /// URI scheme format for SMB/CIFS protocol access (smb://server/share/path).
    /// May include authentication credentials (smb://user:pass@server/share).
    /// Common in cross-platform environments, especially on Unix-like systems accessing Windows shares.
    SMBUrl,
}

/// Credentials for authenticated network paths
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathCredentials {
    /// Username for authentication
    pub username: String,
    
    /// Password for authentication
    pub password: Option<String>,
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
    
    /// Get server information if this is a server path (PosixServer, WindowsUNC, or SMB)
    /// Returns a tuple of (server_name, share_name, credentials) if available
    /// Returns None if this is not a server path
    fn server_info(&self) -> Option<(String, Option<String>, Option<PathCredentials>)> {
        match self.path_type() {
            LocalPathType::NFSPath | LocalPathType::UNCPath => {
                let components = self.components();
                
                // Server paths should have at least one component (the server name)
                if components.is_empty() {
                    return None;
                }
                
                let server_name = components.get(0)
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                
                let share_name = if components.len() > 1 {
                    components.get(1).map(|s| s.to_string())
                } else {
                    None
                };
                
                Some((server_name, share_name, None))
            },
            LocalPathType::SMBUrl => {
                // For SMB URLs, the server info is stored in the namespace
                // in the format "smb://user:pass@server"
                let namespace = self.namespace().as_str();
                
                // Extract the server from the namespace
                if !namespace.starts_with("smb://") {
                    return None;
                }
                
                let without_protocol = &namespace[6..];
                
                // Parse credentials if they exist
                let (server, credentials) = if without_protocol.contains('@') {
                    let parts: Vec<&str> = without_protocol.splitn(2, '@').collect();
                    let cred_part = parts[0];
                    let server_part = parts[1];
                    
                    // Parse username and password
                    let creds = if cred_part.contains(':') {
                        let cred_parts: Vec<&str> = cred_part.splitn(2, ':').collect();
                        Some(PathCredentials {
                            username: cred_parts[0].to_string(),
                            password: Some(cred_parts[1].to_string()),
                        })
                    } else {
                        Some(PathCredentials {
                            username: cred_part.to_string(),
                            password: None,
                        })
                    };
                    
                    (server_part.to_string(), creds)
                } else {
                    (without_protocol.to_string(), None)
                };
                
                // Get share name from first component if available
                let share_name = if !self.components().is_empty() {
                    self.components().get(0).map(|s| s.to_string())
                } else {
                    None
                };
                
                Some((server, share_name, credentials))
            },
            _ => None
        }
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

//
// Default Implementations
//

/// Default implementation of PathComponents that uses a vector of strings
#[derive(Debug, Clone)]
pub struct DefaultPathComponents {
    components: Vec<String>,
    delimiter: char,
}

impl DefaultPathComponents {
    /// Create a new DefaultPathComponents with the specified delimiter
    pub fn with_delimiter(delimiter: char) -> Self {
        DefaultPathComponents {
            components: Vec::new(),
            delimiter,
        }
    }
}

impl PathComponents for DefaultPathComponents {
    fn new() -> Self {
        // Default to '/' as the delimiter
        DefaultPathComponents {
            components: Vec::new(),
            delimiter: '/',
        }
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
        self.components.join(&self.delimiter.to_string())
    }

    fn normalize(&mut self) {
        let mut normalized = Vec::new();
        
        for component in &self.components {
            match component.as_str() {
                "." => continue, // Skip "." components
                ".." => {
                    if !normalized.is_empty() && normalized.last().unwrap() != ".." {
                        normalized.pop(); // Go up one level
                    } else {
                        normalized.push(component.clone()); // Keep ".." if we're at the top
                    }
                }
                _ => normalized.push(component.clone()),
            }
        }
        
        self.components = normalized;
    }

    fn from_string(path: &str) -> Self {
        let delimiter = if path.contains('\\') { '\\' } else { '/' };
        
        let components = path.split(delimiter)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
            
        DefaultPathComponents {
            components,
            delimiter,
        }
    }
}

/// Default implementation of Namespace
#[derive(Debug, Clone)]
pub struct DefaultNamespace {
    value: String,
}

impl Namespace for DefaultNamespace {
    fn as_str(&self) -> &str {
        &self.value
    }

    fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    fn from_string(s: &str) -> Self {
        DefaultNamespace {
            value: s.to_string(),
        }
    }
}

//
// Utility Functions
//

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
    if cfg!(target_family = "unix") {
        LocalPathType::PosixPath
    } else if cfg!(target_family = "windows") {
        LocalPathType::WindowsPath
    } else {
        // Default to POSIX paths as a fallback
        LocalPathType::PosixPath
    }
}

//
// Tests
//

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

    #[test]
    fn test_default_path_components() {
        let mut components = DefaultPathComponents::new();
        
        // Test basic operations
        assert!(components.is_empty());
        components.push("a");
        components.push("b");
        components.push("c");
        
        assert_eq!(components.len(), 3);
        assert_eq!(components.get(0), Some("a"));
        assert_eq!(components.get(1), Some("b"));
        assert_eq!(components.get(2), Some("c"));
        
        // Test join
        assert_eq!(components.join(), "a/b/c");
        
        // Test with different delimiter
        let mut win_components = DefaultPathComponents::with_delimiter('\\');
        win_components.push("C:");
        win_components.push("Windows");
        win_components.push("System32");
        
        assert_eq!(win_components.join(), "C:\\Windows\\System32");
        
        // Test normalization
        let mut path = DefaultPathComponents::from_string("a/./b/../c/./d");
        path.normalize();
        assert_eq!(path.join(), "a/c/d");
        
        // Test with trailing/leading dots
        let mut path2 = DefaultPathComponents::from_string("../../a/b/c/../../d");
        path2.normalize();
        assert_eq!(path2.join(), "../../a/d");
    }
}

//
// EssentialPath Implementation
//

/// EssentialPath is the central path abstraction that can be converted to and from other path types.
/// It serves as the common format for path representation across different backends.
#[derive(Debug, Clone)]
pub struct EssentialPath {
    namespace: DefaultNamespace,
    components: DefaultPathComponents,
    is_absolute: bool,
}

impl EssentialPath {
    /// Create a new empty EssentialPath
    pub fn new() -> Self {
        EssentialPath {
            namespace: DefaultNamespace::from_string(""),
            components: DefaultPathComponents::new(),
            is_absolute: false,
        }
    }
    
    /// Create an EssentialPath from a string representation
    pub fn from_string(path_str: &str) -> Result<Self> {
        Self::parse(path_str)
    }
    
    /// Convert this EssentialPath to a specific path type using a resolver
    pub fn to_specific_path<T: Path>(&self, converter: &dyn PathConverter<T>) -> Result<T> {
        converter.from_essential_path(self)
    }
}

impl Path for EssentialPath {
    type ComponentsType = DefaultPathComponents;
    type NamespaceType = DefaultNamespace;
    
    fn new() -> Self {
        EssentialPath::new()
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
        // Handle empty paths
        if path_str.is_empty() {
            return Ok(Self::new());
        }
        
        let is_absolute = path_str.starts_with('/') || path_str.starts_with('\\');
        
        // Parse namespace and path components
        let (namespace_str, path_part) = if path_str.contains(':') {
            let parts: Vec<&str> = path_str.splitn(2, ':').collect();
            (parts[0], parts.get(1).unwrap_or(&""))
        } else {
            ("", path_str)
        };
        
        // Create the path components
        let mut components = if is_absolute {
            // Remove leading slash for absolute paths when parsing components
            let path_without_slash = if path_part.starts_with('/') || path_part.starts_with('\\') {
                &path_part[1..]
            } else {
                path_part
            };
            DefaultPathComponents::from_string(path_without_slash)
        } else {
            DefaultPathComponents::from_string(path_part)
        };
        
        // Normalize the components
        components.normalize();
        
        Ok(EssentialPath {
            namespace: DefaultNamespace::from_string(namespace_str),
            components,
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
            if path.is_empty() {
                format!("{}/", ns)
            } else {
                format!("{}/{}", ns, path)
            }
        } else {
            format!("{}{}", ns, path)
        }
    }
    
    fn join(&self, relative: &str) -> Result<Self> {
        // Cannot join an absolute path to another path
        if relative.starts_with('/') || relative.starts_with('\\') {
            return Err(crate::core::error::Error::InvalidArgument(
                "Cannot join an absolute path".to_string()
            ));
        }
        
        let rel_path = Self::parse(relative)?;
        
        // If rel_path has a namespace, it's a different path type
        if !rel_path.namespace().is_empty() {
            return Err(crate::core::error::Error::InvalidArgument(
                "Cannot join paths with different namespaces".to_string()
            ));
        }
        
        let mut result = self.clone();
        
        // Add components from the relative path
        for i in 0..rel_path.components().len() {
            if let Some(component) = rel_path.components().get(i) {
                result.components_mut().push(component);
            }
        }
        
        // Normalize the result
        result.normalize();
        
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
        // This would be implemented using the resolver repository
        // For now, return None as we haven't implemented local paths yet
        None
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

//
// Path Converter Trait
//

/// PathConverter provides conversion between EssentialPath and specific path types
pub trait PathConverter<T: Path> {
    /// Convert an EssentialPath to a specific path type
    fn from_essential_path(&self, path: &EssentialPath) -> Result<T>;
    
    /// Convert a specific path type to an EssentialPath
    fn to_essential_path(&self, path: &T) -> Result<EssentialPath>;
}

//
// Path Resolver Repository
//

/// PathResolver is responsible for resolving and converting paths for a specific service or system
pub trait PathResolver: Send + Sync {
    /// Get the unique ID for this resolver
    fn id(&self) -> &str;
    
    /// Check if this resolver can handle the given namespace
    fn can_resolve(&self, namespace: &str) -> bool;
    
    /// Convert an EssentialPath to a specific implementation
    fn resolve(&self, path: &EssentialPath) -> Result<Box<dyn Path>>;
    
    /// Convert a specific path implementation back to an EssentialPath
    fn to_essential_path(&self, path: &dyn Path) -> Result<EssentialPath>;
}

/// PathResolverRepository manages a collection of path resolvers and provides path resolution services
pub struct PathResolverRepository {
    resolvers: Vec<Box<dyn PathResolver>>,
    local_resolver: Option<Box<dyn PathResolver>>,
}

impl PathResolverRepository {
    /// Create a new empty repository
    pub fn new() -> Self {
        PathResolverRepository {
            resolvers: Vec::new(),
            local_resolver: None,
        }
    }
    
    /// Register a new path resolver
    pub fn register_resolver(&mut self, resolver: Box<dyn PathResolver>) {
        self.resolvers.push(resolver);
    }
    
    /// Set the local path resolver
    pub fn set_local_resolver(&mut self, resolver: Box<dyn PathResolver>) {
        self.local_resolver = Some(resolver);
    }
    
    /// Resolve an EssentialPath to a specific path implementation
    pub fn resolve(&self, path: &EssentialPath) -> Result<Box<dyn Path>> {
        let namespace = path.namespace().as_str();
        
        // If namespace is empty or resolvers list is empty, use local resolver if available
        if namespace.is_empty() || self.resolvers.is_empty() {
            if let Some(local_resolver) = &self.local_resolver {
                return local_resolver.resolve(path);
            } else {
                return Err(crate::core::error::Error::NotFound(
                    "No local resolver registered".to_string()
                ));
            }
        }
        
        // Find a resolver that can handle this namespace
        for resolver in &self.resolvers {
            if resolver.can_resolve(namespace) {
                return resolver.resolve(path);
            }
        }
        
        // If no resolver found, try the local resolver as fallback
        if let Some(local_resolver) = &self.local_resolver {
            return local_resolver.resolve(path);
        }
        
        Err(crate::core::error::Error::NotFound(
            format!("No resolver found for namespace: {}", namespace)
        ))
    }
    
    /// Convert a specific path implementation back to an EssentialPath
    pub fn to_essential_path(&self, path: &dyn Path) -> Result<EssentialPath> {
        // Try to find a resolver that can handle this path type
        for resolver in &self.resolvers {
            if let Ok(essential_path) = resolver.to_essential_path(path) {
                return Ok(essential_path);
            }
        }
        
        // Try local resolver as fallback
        if let Some(local_resolver) = &self.local_resolver {
            return local_resolver.to_essential_path(path);
        }
        
        Err(crate::core::error::Error::InvalidArgument(
            "No resolver can convert this path".to_string()
        ))
    }
    
    /// Get a resolver by ID
    pub fn get_resolver(&self, id: &str) -> Option<&dyn PathResolver> {
        for resolver in &self.resolvers {
            if resolver.id() == id {
                return Some(resolver.as_ref());
            }
        }
        None
    }
}

//
// Tests for EssentialPath and PathResolverRepository
//

#[cfg(test)]
mod essential_path_tests {
    use super::*;
    use std::any::Any;
    
    // Mock path type for testing
    #[derive(Debug, Clone)]
    struct MockServicePath {
        account: String,
        path: String,
    }
    
    impl MockServicePath {
        fn new(account: &str, path: &str) -> Self {
            MockServicePath {
                account: account.to_string(),
                path: path.to_string(),
            }
        }
    }
    
    impl Path for MockServicePath {
        type ComponentsType = DefaultPathComponents;
        type NamespaceType = DefaultNamespace;
        
        fn new() -> Self {
            MockServicePath {
                account: String::new(),
                path: String::new(),
            }
        }
        
        fn namespace(&self) -> &Self::NamespaceType {
            // This is a dummy implementation since we don't use these methods
            unimplemented!("Not needed for test")
        }
        
        fn namespace_mut(&mut self) -> &mut Self::NamespaceType {
            unimplemented!("Not needed for test")
        }
        
        fn components(&self) -> &Self::ComponentsType {
            unimplemented!("Not needed for test")
        }
        
        fn components_mut(&mut self) -> &mut Self::ComponentsType {
            unimplemented!("Not needed for test")
        }
        
        fn parse(path_str: &str) -> Result<Self> {
            unimplemented!("Not needed for test")
        }
        
        fn to_string(&self) -> String {
            format!("{}:{}", self.account, self.path)
        }
        
        fn join(&self, _relative: &str) -> Result<Self> {
            unimplemented!("Not needed for test")
        }
        
        fn parent(&self) -> Option<Self> {
            unimplemented!("Not needed for test")
        }
        
        fn file_name(&self) -> Option<String> {
            unimplemented!("Not needed for test")
        }
        
        fn normalize(&mut self) {
            // No-op for test
        }
        
        fn is_absolute(&self) -> bool {
            self.path.starts_with('/')
        }
        
        fn to_local_path(&self) -> Option<Box<dyn LocalPath>> {
            None
        }
        
        fn as_any(&self) -> &dyn Any {
            self
        }
    }
    
    // Mock resolver for testing
    struct MockServiceResolver {
        service_id: String,
        accounts: Vec<String>,
    }
    
    impl MockServiceResolver {
        fn new(service_id: &str, accounts: Vec<String>) -> Self {
            MockServiceResolver {
                service_id: service_id.to_string(),
                accounts,
            }
        }
    }
    
    impl PathResolver for MockServiceResolver {
        fn id(&self) -> &str {
            &self.service_id
        }
        
        fn can_resolve(&self, namespace: &str) -> bool {
            self.accounts.contains(&namespace.to_string())
        }
        
        fn resolve(&self, path: &EssentialPath) -> Result<Box<dyn Path>> {
            let namespace = path.namespace().as_str();
            
            if !self.can_resolve(namespace) {
                return Err(crate::core::error::Error::InvalidArgument(
                    format!("Account {} not supported by service {}", namespace, self.service_id)
                ));
            }
            
            let components_str = path.components().join();
            let path_str = if path.is_absolute() {
                format!("/{}", components_str)
            } else {
                components_str
            };
            
            let service_path = MockServicePath::new(namespace, &path_str);
            
            // Box the service path as a Box<dyn Path>
            Ok(Box::new(service_path))
        }
        
        fn to_essential_path(&self, path: &dyn Path) -> Result<EssentialPath> {
            // Try to downcast to MockServicePath
            if let Some(service_path) = path.as_any().downcast_ref::<MockServicePath>() {
                let path_str = if service_path.path.starts_with('/') {
                    format!("{}:{}", service_path.account, service_path.path)
                } else {
                    format!("{}:{}", service_path.account, service_path.path)
                };
                
                EssentialPath::parse(&path_str)
            } else {
                Err(crate::core::error::Error::InvalidArgument(
                    "Path is not a MockServicePath".to_string()
                ))
            }
        }
    }
    
    #[test]
    fn test_essential_path_parsing() {
        // Test absolute path with namespace
        let path1 = EssentialPath::parse("account1:/sales/report").unwrap();
        assert_eq!(path1.namespace().as_str(), "account1");
        assert_eq!(path1.components().get(0), Some("sales"));
        assert_eq!(path1.components().get(1), Some("report"));
        assert!(path1.is_absolute());
        
        // Test relative path with namespace
        let path2 = EssentialPath::parse("account2:sales/forecast").unwrap();
        assert_eq!(path2.namespace().as_str(), "account2");
        assert_eq!(path2.components().get(0), Some("sales"));
        assert_eq!(path2.components().get(1), Some("forecast"));
        assert!(!path2.is_absolute());
        
        // Test absolute path without namespace
        let path3 = EssentialPath::parse("/sales/invoice").unwrap();
        assert_eq!(path3.namespace().as_str(), "");
        assert_eq!(path3.components().get(0), Some("sales"));
        assert_eq!(path3.components().get(1), Some("invoice"));
        assert!(path3.is_absolute());
        
        // Test to_string
        assert_eq!(path1.to_string(), "account1:/sales/report");
        assert_eq!(path2.to_string(), "account2:sales/forecast");
        assert_eq!(path3.to_string(), "/sales/invoice");
    }
    
    #[test]
    fn test_resolver_repository() {
        // Create mock resolvers
        let dropbox_resolver = MockServiceResolver::new(
            "dropbox", 
            vec!["account1".to_string(), "dropbox-user".to_string()]
        );
        
        let onedrive_resolver = MockServiceResolver::new(
            "onedrive",
            vec!["account2".to_string(), "onedrive-user".to_string()]
        );
        
        // Create repository and register resolvers
        let mut repo = PathResolverRepository::new();
        repo.register_resolver(Box::new(dropbox_resolver));
        repo.register_resolver(Box::new(onedrive_resolver));
        
        // Test resolution
        let path1 = EssentialPath::parse("account1:/sales/report").unwrap();
        let resolved1 = repo.resolve(&path1).unwrap();
        assert_eq!(resolved1.to_string(), "account1:/sales/report");
        
        let path2 = EssentialPath::parse("account2:/sales/forecast").unwrap();
        let resolved2 = repo.resolve(&path2).unwrap();
        assert_eq!(resolved2.to_string(), "account2:/sales/forecast");
        
        // Test unresolvable path
        let path3 = EssentialPath::parse("unknown:/sales/document").unwrap();
        assert!(repo.resolve(&path3).is_err());
    }
} 