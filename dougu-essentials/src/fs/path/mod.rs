// dougu-essentials::fs::path module
//
// Abstracts various types of paths including local file systems, cloud storage,
// and service-specific paths (like task tracking systems).

use std::fmt;
use std::ops::Deref;
use crate::core::error::{Error, Result};

/// Represents a namespace for a path
/// 
/// Examples:
/// - Windows drive letter (C:)
/// - Server name for UNC paths (\\server)
/// - Cloud storage namespace ID
/// - Service identifier for abstract file systems
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Namespace {
    /// Local file system namespace
    /// On Windows, this would be a drive letter or UNC server name
    /// On Unix, this is typically empty
    Local(String),
    
    /// Cloud storage namespace with provider and ID
    Cloud {
        provider: String,
        id: String,
    },
    
    /// Service-specific namespace for abstract file systems
    /// For example, JIRA projects, Git repositories, etc.
    Service {
        provider: String,
        id: String,
    },
}

impl Namespace {
    /// Creates a new local namespace
    pub fn local<S: Into<String>>(name: S) -> Self {
        Namespace::Local(name.into())
    }
    
    /// Creates a new cloud namespace
    pub fn cloud<S1: Into<String>, S2: Into<String>>(provider: S1, id: S2) -> Self {
        Namespace::Cloud {
            provider: provider.into(),
            id: id.into(),
        }
    }
    
    /// Creates a new service namespace
    pub fn service<S1: Into<String>, S2: Into<String>>(provider: S1, id: S2) -> Self {
        Namespace::Service {
            provider: provider.into(),
            id: id.into(),
        }
    }
    
    /// Returns true if this is a local namespace
    pub fn is_local(&self) -> bool {
        matches!(self, Namespace::Local(_))
    }
    
    /// Returns true if this is a cloud namespace
    pub fn is_cloud(&self) -> bool {
        matches!(self, Namespace::Cloud { .. })
    }
    
    /// Returns true if this is a service namespace
    pub fn is_service(&self) -> bool {
        matches!(self, Namespace::Service { .. })
    }
}

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Namespace::Local(name) => {
                if name.is_empty() {
                    write!(f, "")
                } else {
                    write!(f, "{}", name)
                }
            }
            Namespace::Cloud { provider, id } => {
                write!(f, "{}://{}", provider, id)
            }
            Namespace::Service { provider, id } => {
                write!(f, "{}://{}", provider, id)
            }
        }
    }
}

/// Represents components of a path
/// 
/// This is an abstraction over path segments that can be manipulated
/// and normalized consistently across different path types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathComponents {
    components: Vec<String>,
}

impl PathComponents {
    /// Creates a new empty PathComponents
    pub fn new() -> Self {
        PathComponents {
            components: Vec::new(),
        }
    }
    
    /// Creates PathComponents from a string representation
    /// 
    /// The string will be split on '/' or '\' depending on the platform
    pub fn from_string<S: AsRef<str>>(s: S) -> Self {
        let s = s.as_ref();
        let mut components = Vec::new();
        
        // Split by both / and \ to handle different path formats
        for part in s.split(|c| c == '/' || c == '\\') {
            if !part.is_empty() {
                components.push(part.to_string());
            }
        }
        
        PathComponents { components }
    }
    
    /// Adds a component to the end of the path
    pub fn push<S: Into<String>>(&mut self, component: S) {
        self.components.push(component.into());
    }
    
    /// Removes the last component and returns it, or None if the path is empty
    pub fn pop(&mut self) -> Option<String> {
        self.components.pop()
    }
    
    /// Returns the number of components
    pub fn len(&self) -> usize {
        self.components.len()
    }
    
    /// Returns true if there are no components
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }
    
    /// Returns an iterator over the components
    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.components.iter()
    }
    
    /// Normalizes the path components by resolving ".." and "." segments
    pub fn normalize(&mut self) {
        let mut result = Vec::new();
        
        for component in &self.components {
            match component.as_str() {
                "." => {
                    // Current directory, skip
                },
                ".." => {
                    // Parent directory, remove last component if possible
                    if !result.is_empty() && result.last().unwrap() != ".." {
                        result.pop();
                    } else {
                        // If we're already at root or have only ".." components, keep the ".."
                        result.push("..".to_string());
                    }
                },
                _ => {
                    // Regular component, add it
                    result.push(component.clone());
                }
            }
        }
        
        self.components = result;
    }
    
    /// Returns a normalized copy of the path components
    pub fn normalized(&self) -> Self {
        let mut result = self.clone();
        result.normalize();
        result
    }
    
    /// Joins the components with the given separator
    pub fn join(&self, separator: &str) -> String {
        self.components.join(separator)
    }
}

/// Path abstraction for both local and remote file systems
/// 
/// This type represents a path to a file or folder in any supported
/// file system, whether it's local, cloud-based, or service-based.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path {
    /// The namespace of the path
    namespace: Namespace,
    
    /// The components of the path
    components: PathComponents,
}

impl Path {
    /// Creates a new Path with the given namespace and components
    pub fn new(namespace: Namespace, components: PathComponents) -> Self {
        Path { namespace, components }
    }
    
    /// Creates a new empty Path with a local namespace
    pub fn empty_local() -> Self {
        Path {
            namespace: Namespace::local(""),
            components: PathComponents::new(),
        }
    }
    
    /// Attempts to parse a string as a Path
    /// 
    /// For local paths, this will use platform-specific parsing rules.
    /// For remote paths, this expects a URI-like format.
    pub fn parse<S: AsRef<str>>(s: S) -> Result<Self> {
        let s = s.as_ref();
        
        // Check if this is a remote path (contains "://")
        if let Some(scheme_end) = s.find("://") {
            let scheme = &s[0..scheme_end];
            let rest = &s[scheme_end + 3..];
            
            // Split the rest into namespace ID and path
            let (id, path) = if let Some(path_start) = rest.find('/') {
                (&rest[0..path_start], &rest[path_start + 1..])
            } else {
                (rest, "")
            };
            
            // Determine if this is a cloud or service namespace
            let namespace = if scheme.starts_with("cloud.") {
                let provider = scheme.strip_prefix("cloud.").unwrap_or(scheme);
                Namespace::cloud(provider, id)
            } else {
                Namespace::service(scheme, id)
            };
            
            let components = PathComponents::from_string(path);
            Ok(Path { namespace, components })
        } else {
            // This is a local path
            #[cfg(windows)]
            {
                // Windows path handling
                if s.starts_with("\\\\") {
                    // UNC path
                    if let Some(server_end) = s[2..].find('\\') {
                        let server = &s[2..2 + server_end];
                        let path = &s[2 + server_end + 1..];
                        let namespace = Namespace::local(format!("\\\\{}", server));
                        let components = PathComponents::from_string(path);
                        return Ok(Path { namespace, components });
                    }
                } else if s.len() >= 2 && s.chars().nth(1) == Some(':') {
                    // Drive letter
                    let drive = &s[0..2];
                    let path = if s.len() > 2 { &s[2..] } else { "" };
                    let namespace = Namespace::local(drive);
                    let components = PathComponents::from_string(path);
                    return Ok(Path { namespace, components });
                }
            }
            
            // Unix or simple Windows path with no drive letter
            let namespace = Namespace::local("");
            let components = PathComponents::from_string(s);
            Ok(Path { namespace, components })
        }
    }
    
    /// Returns the namespace of the path
    pub fn namespace(&self) -> &Namespace {
        &self.namespace
    }
    
    /// Returns the components of the path
    pub fn components(&self) -> &PathComponents {
        &self.components
    }
    
    /// Returns a mutable reference to the components
    pub fn components_mut(&mut self) -> &mut PathComponents {
        &mut self.components
    }
    
    /// Returns true if this is a local path
    pub fn is_local(&self) -> bool {
        self.namespace.is_local()
    }
    
    /// Returns true if this is a cloud path
    pub fn is_cloud(&self) -> bool {
        self.namespace.is_cloud()
    }
    
    /// Returns true if this is a service path
    pub fn is_service(&self) -> bool {
        self.namespace.is_service()
    }
    
    /// Normalizes this path in place by resolving ".." and "." components
    pub fn normalize(&mut self) {
        self.components.normalize();
    }
    
    /// Returns a normalized copy of this path
    pub fn normalized(&self) -> Self {
        let mut result = self.clone();
        result.normalize();
        result
    }
    
    /// Joins this path with a relative path and returns a new Path
    pub fn join<S: AsRef<str>>(&self, path: S) -> Self {
        let mut result = self.clone();
        let components = PathComponents::from_string(path);
        
        for component in components.components {
            result.components.push(component);
        }
        
        result.normalize();
        result
    }
    
    /// Returns the parent path, or None if this is a root path
    pub fn parent(&self) -> Option<Self> {
        if self.components.is_empty() {
            return None;
        }
        
        let mut parent = self.clone();
        parent.components.pop();
        Some(parent)
    }
    
    /// Returns the file name component of the path, or None if empty
    pub fn file_name(&self) -> Option<&str> {
        self.components.components.last().map(|s| s.as_str())
    }
    
    /// Returns a string representation of this path
    /// 
    /// For local paths, this will use platform-specific formatting.
    /// For remote paths, this will use a URI-like format.
    pub fn to_string(&self) -> String {
        match &self.namespace {
            Namespace::Local(name) => {
                let prefix = if name.is_empty() { String::new() } else { name.clone() };
                
                if self.components.is_empty() {
                    prefix
                } else {
                    #[cfg(windows)]
                    {
                        if prefix.is_empty() || prefix.ends_with('\\') {
                            format!("{}{}", prefix, self.components.join("\\"))
                        } else {
                            format!("{}\\{}", prefix, self.components.join("\\"))
                        }
                    }
                    
                    #[cfg(not(windows))]
                    {
                        if prefix.is_empty() || prefix.ends_with('/') {
                            format!("{}{}", prefix, self.components.join("/"))
                        } else {
                            format!("{}/{}", prefix, self.components.join("/"))
                        }
                    }
                }
            },
            Namespace::Cloud { provider, id } => {
                if self.components.is_empty() {
                    format!("cloud://{}:{}", provider, id)
                } else {
                    format!("cloud://{}:{}/{}", provider, id, self.components.join("/"))
                }
            },
            Namespace::Service { provider, id } => {
                if self.components.is_empty() {
                    format!("{}://{}", provider, id)
                } else {
                    format!("{}://{}/{}", provider, id, self.components.join("/"))
                }
            }
        }
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// An owned, mutable path with the same semantics as Path
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathBuf {
    inner: Path,
}

impl PathBuf {
    /// Creates a new empty PathBuf with a local namespace
    pub fn new() -> Self {
        PathBuf {
            inner: Path::empty_local(),
        }
    }
    
    /// Creates a PathBuf from a Path
    pub fn from_path(path: Path) -> Self {
        PathBuf { inner: path }
    }
    
    /// Attempts to parse a string as a PathBuf
    pub fn parse<S: AsRef<str>>(s: S) -> Result<Self> {
        Ok(PathBuf { inner: Path::parse(s)? })
    }
    
    /// Converts this PathBuf into a Path
    pub fn into_path(self) -> Path {
        self.inner
    }
    
    /// Returns a reference to the underlying Path
    pub fn as_path(&self) -> &Path {
        &self.inner
    }
}

impl Deref for PathBuf {
    type Target = Path;
    
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<Path> for PathBuf {
    fn from(path: Path) -> Self {
        PathBuf::from_path(path)
    }
}

impl<'a> From<&'a Path> for PathBuf {
    fn from(path: &'a Path) -> Self {
        PathBuf::from_path(path.clone())
    }
}

impl AsRef<Path> for PathBuf {
    fn as_ref(&self) -> &Path {
        &self.inner
    }
}

impl Default for PathBuf {
    fn default() -> Self {
        PathBuf::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_path_components_normalize() {
        let mut components = PathComponents::from_string("a/b/../c/./d/../../e");
        components.normalize();
        assert_eq!(components.join("/"), "a/e");
        
        let mut components = PathComponents::from_string("../a/b/../../c");
        components.normalize();
        assert_eq!(components.join("/"), "../c");
    }
    
    #[test]
    fn test_path_parse_local() {
        #[cfg(windows)]
        {
            let path = Path::parse(r"C:\Users\test\Documents").unwrap();
            assert_eq!(path.namespace(), &Namespace::local("C:"));
            assert_eq!(path.components().join("\\"), r"Users\test\Documents");
            
            let path = Path::parse(r"\\server\share\folder").unwrap();
            assert_eq!(path.namespace(), &Namespace::local(r"\\server"));
            assert_eq!(path.components().join("\\"), r"share\folder");
        }
        
        #[cfg(not(windows))]
        {
            let path = Path::parse("/home/user/documents").unwrap();
            assert_eq!(path.namespace(), &Namespace::local(""));
            assert_eq!(path.components().join("/"), "home/user/documents");
        }
    }
    
    #[test]
    fn test_path_parse_remote() {
        let path = Path::parse("cloud.dropbox://account123/folder/file.txt").unwrap();
        assert_eq!(path.namespace(), &Namespace::cloud("dropbox", "account123"));
        assert_eq!(path.components().join("/"), "folder/file.txt");
        
        let path = Path::parse("jira://PROJECT/ticket-123").unwrap();
        assert_eq!(path.namespace(), &Namespace::service("jira", "PROJECT"));
        assert_eq!(path.components().join("/"), "ticket-123");
    }
    
    #[test]
    fn test_path_join() {
        let path = Path::parse("/home/user").unwrap();
        let joined = path.join("documents/file.txt");
        assert_eq!(joined.to_string(), "/home/user/documents/file.txt");
        
        let path = Path::parse("cloud.dropbox://account123/folder").unwrap();
        let joined = path.join("subfolder/file.txt");
        assert_eq!(joined.to_string(), "cloud://dropbox:account123/folder/subfolder/file.txt");
    }
    
    #[test]
    fn test_path_normalize() {
        let mut path = Path::parse("/home/user/../documents/./file.txt").unwrap();
        path.normalize();
        assert_eq!(path.to_string(), "/home/documents/file.txt");
    }
} 