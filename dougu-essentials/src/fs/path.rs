use crate::core::{Error, Result};
use std::fmt::{Debug, Display};
use std::ops::Deref;

/// Represents a namespace for a path, which could be a drive letter for Windows,
/// a server name, a cloud storage identifier, etc.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathNamespace {
    /// The identifier for this namespace
    id: String,
    /// The type of namespace this represents
    namespace_type: NamespaceType,
}

impl PathNamespace {
    /// Creates a new path namespace
    pub fn new<S: Into<String>>(id: S, namespace_type: NamespaceType) -> Self {
        Self {
            id: id.into(),
            namespace_type,
        }
    }

    /// Creates a local path namespace
    pub fn local<S: Into<String>>(id: S) -> Self {
        Self::new(id, NamespaceType::Local)
    }

    /// Creates a cloud path namespace
    pub fn cloud<S: Into<String>>(id: S) -> Self {
        Self::new(id, NamespaceType::Cloud)
    }

    /// Creates a service path namespace
    pub fn service<S: Into<String>>(id: S) -> Self {
        Self::new(id, NamespaceType::Service)
    }

    /// Returns the namespace ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the namespace type
    pub fn namespace_type(&self) -> NamespaceType {
        self.namespace_type
    }

    /// Checks if this namespace represents a local file system
    pub fn is_local(&self) -> bool {
        self.namespace_type == NamespaceType::Local
    }

    /// Checks if this namespace represents a cloud storage
    pub fn is_cloud(&self) -> bool {
        self.namespace_type == NamespaceType::Cloud
    }

    /// Checks if this namespace represents a service
    pub fn is_service(&self) -> bool {
        self.namespace_type == NamespaceType::Service
    }
}

/// Represents the type of namespace a path belongs to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NamespaceType {
    /// Local file system (e.g., C: drive, / root)
    Local,
    /// Cloud storage (e.g., S3, Azure Blob Storage)
    Cloud,
    /// External service (e.g., JIRA, GitHub)
    Service,
}

/// Represents components of a path
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathComponents {
    /// The individual parts of the path
    parts: Vec<String>,
}

impl PathComponents {
    /// Creates a new path components instance
    pub fn new<I, S>(parts: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            parts: parts.into_iter().map(Into::into).collect(),
        }
    }

    /// Normalizes the path components by removing redundant parts like "..", "."
    pub fn normalize(&self) -> Self {
        let mut result = Vec::new();

        for part in &self.parts {
            match part.as_str() {
                "." => {
                    // Skip current directory marker
                }
                ".." => {
                    // Go up one directory if possible
                    if !result.is_empty() && result.last().unwrap() != ".." {
                        result.pop();
                    } else {
                        // Can't go up any further, keep the .. (for relative paths)
                        result.push(part.clone());
                    }
                }
                "" => {
                    // Skip empty parts, except for the first one which might indicate root
                    if result.is_empty() {
                        result.push(part.clone());
                    }
                }
                _ => result.push(part.clone()),
            }
        }

        Self { parts: result }
    }

    /// Joins this component with another
    pub fn join<I, S>(&self, parts: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut new_parts = self.parts.clone();
        new_parts.extend(parts.into_iter().map(Into::into));
        Self { parts: new_parts }
    }

    /// Returns the parts of this path component
    pub fn parts(&self) -> &[String] {
        &self.parts
    }

    /// Returns true if this represents an absolute path (starts with empty string)
    pub fn is_absolute(&self) -> bool {
        !self.parts.is_empty() && self.parts[0].is_empty()
    }

    /// Returns true if this represents a relative path
    pub fn is_relative(&self) -> bool {
        !self.is_absolute()
    }

    /// Returns the parent component, if any
    pub fn parent(&self) -> Option<Self> {
        if self.parts.is_empty() {
            return None;
        }
        
        let mut parts = self.parts.clone();
        parts.pop();
        if parts.is_empty() {
            None
        } else {
            Some(Self { parts })
        }
    }

    /// Returns the last component as a string, if any
    pub fn file_name(&self) -> Option<&str> {
        self.parts.last().map(|s| s.as_str())
    }
}

impl From<&[&str]> for PathComponents {
    fn from(parts: &[&str]) -> Self {
        Self::new(parts.iter().map(|&s| s.to_string()))
    }
}

impl From<Vec<String>> for PathComponents {
    fn from(parts: Vec<String>) -> Self {
        Self { parts }
    }
}

/// Trait for path types that can be joined with additional components
pub trait Joinable {
    /// The resulting type after joining
    type Output;
    
    /// Join this path with additional components
    fn join<P: AsRef<str>>(&self, path: P) -> Self::Output;
}

/// Trait for path types that can be converted to a string representation
pub trait AsPathStr {
    /// Convert to a path string using the appropriate separators for the path type
    fn as_path_str(&self) -> String;
}

/// Abstract representation of a path, which could be a local file system path,
/// a cloud storage path, or a service path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path {
    /// The namespace this path belongs to
    namespace: PathNamespace,
    /// The components of this path
    components: PathComponents,
}

impl Path {
    /// Creates a new path
    pub fn new(namespace: PathNamespace, components: PathComponents) -> Self {
        Self { 
            namespace, 
            components,
        }
    }

    /// Creates a new local path
    pub fn local<S: Into<String>, I, P>(namespace_id: S, parts: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<String>,
    {
        Self {
            namespace: PathNamespace::local(namespace_id),
            components: PathComponents::new(parts),
        }
    }

    /// Creates a new cloud path
    pub fn cloud<S: Into<String>, I, P>(namespace_id: S, parts: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<String>,
    {
        Self {
            namespace: PathNamespace::cloud(namespace_id),
            components: PathComponents::new(parts),
        }
    }

    /// Creates a new service path
    pub fn service<S: Into<String>, I, P>(namespace_id: S, parts: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<String>,
    {
        Self {
            namespace: PathNamespace::service(namespace_id),
            components: PathComponents::new(parts),
        }
    }

    /// Returns the namespace of this path
    pub fn namespace(&self) -> &PathNamespace {
        &self.namespace
    }

    /// Returns the components of this path
    pub fn components(&self) -> &PathComponents {
        &self.components
    }

    /// Returns a normalized version of this path
    pub fn normalize(&self) -> Self {
        Self {
            namespace: self.namespace.clone(),
            components: self.components.normalize(),
        }
    }

    /// Returns true if this path is absolute
    pub fn is_absolute(&self) -> bool {
        self.components.is_absolute()
    }

    /// Returns true if this path is relative
    pub fn is_relative(&self) -> bool {
        self.components.is_relative()
    }

    /// Returns true if this path is on a local file system
    pub fn is_local(&self) -> bool {
        self.namespace.is_local()
    }

    /// Returns true if this path is on a cloud storage
    pub fn is_cloud(&self) -> bool {
        self.namespace.is_cloud()
    }

    /// Returns true if this path is on a service
    pub fn is_service(&self) -> bool {
        self.namespace.is_service()
    }

    /// Returns the parent of this path, if any
    pub fn parent(&self) -> Option<Self> {
        self.components.parent().map(|components| Self {
            namespace: self.namespace.clone(),
            components,
        })
    }

    /// Returns the file name of this path, if any
    pub fn file_name(&self) -> Option<&str> {
        self.components.file_name()
    }
}

impl Joinable for Path {
    type Output = Result<Self>;

    fn join<P: AsRef<str>>(&self, path: P) -> Self::Output {
        // Simple string splitting implementation
        // In a real implementation, we'd need more sophisticated parsing
        let parts: Vec<String> = path
            .as_ref()
            .split('/')
            .map(|s| s.to_string())
            .collect();

        Ok(Self {
            namespace: self.namespace.clone(),
            components: self.components.join(parts),
        })
    }
}

impl AsPathStr for Path {
    fn as_path_str(&self) -> String {
        // Format depends on the namespace type
        match self.namespace.namespace_type() {
            NamespaceType::Local => {
                // For Windows-style paths with drive letters
                if !self.namespace.id().is_empty() {
                    format!(
                        "{}:{}",
                        self.namespace.id(),
                        self.components.parts().join("\\")
                    )
                } else {
                    // For Unix-style paths
                    format!("/{}", self.components.parts().join("/"))
                }
            }
            NamespaceType::Cloud => {
                // Cloud paths typically use forward slashes
                format!("{}://{}", 
                    self.namespace.namespace_type().prefix(),
                    self.components.parts().join("/")
                )
            }
            NamespaceType::Service => {
                // Service paths use domain-specific separators
                format!("{}://{}", 
                    self.namespace.namespace_type().prefix(), 
                    self.components.parts().join("/")
                )
            }
        }
    }
}

impl NamespaceType {
    /// Returns a string prefix for this namespace type, used in URIs
    fn prefix(&self) -> &'static str {
        match self {
            NamespaceType::Local => "file",
            NamespaceType::Cloud => "cloud",
            NamespaceType::Service => "service",
        }
    }
}

/// Trait implemented by types that can be converted to a Path
pub trait AsPath {
    /// Convert the type to a Path
    fn as_path(&self) -> Result<Path>;
}

impl AsPath for std::path::Path {
    fn as_path(&self) -> Result<Path> {
        // Convert a std::path::Path to our Path abstraction
        // This is simplified and would need platform-specific logic
        
        let path_str = self.to_string_lossy();
        
        // Check if it has a drive letter (Windows)
        let (namespace, path_without_prefix) = if let Some(drive_letter) = path_str.chars().next() {
            if path_str.len() > 1 && path_str.chars().nth(1) == Some(':') {
                (PathNamespace::local(drive_letter.to_string()), &path_str[2..])
            } else {
                (PathNamespace::local(""), &*path_str)
            }
        } else {
            (PathNamespace::local(""), &*path_str)
        };
        
        // Split the path on separators
        let components: Vec<String> = path_without_prefix
            .split(|c| c == '/' || c == '\\')
            .map(|s| s.to_string())
            .collect();
            
        Ok(Path::new(namespace, components.into()))
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_path_str())
    }
}

// Extension trait for convenient interaction with std::path::Path
pub trait PathExt {
    /// Convert to a std::path::Path if this is a local path
    fn to_std_path(&self) -> Result<std::path::PathBuf>;
}

impl PathExt for Path {
    fn to_std_path(&self) -> Result<std::path::PathBuf> {
        if !self.is_local() {
            return Err(Error::msg("Cannot convert non-local path to std::path::Path"));
        }
        
        let path_str = self.as_path_str();
        Ok(std::path::PathBuf::from(path_str))
    }
} 