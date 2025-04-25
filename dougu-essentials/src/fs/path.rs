use std::fmt;
use crate::core::Error;

/// Type of path, used to distinguish between different file systems
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathType {
    /// Local file system path (posix style)
    PosixLocal,
    /// Windows style local path (with drive letter)
    WindowsLocal,
    /// Windows UNC path
    WindowsUNC,
    /// Cloud storage path
    Cloud,
    /// Service specific path (e.g., task tracking systems)
    Service,
    /// Other specialized path types can be added as needed
    Other(String),
}

/// Represents a namespace for a path
/// This could be a drive letter for Windows, server name for UNC paths,
/// cloud storage ID, or service identifier
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathNamespace {
    /// Type of the namespace (e.g., "drive", "server", "cloud_id")
    pub namespace_type: String,
    /// Value of the namespace (e.g., "C:", "server01", "aws-s3-bucket1")
    pub value: String,
}

impl PathNamespace {
    /// Creates a new path namespace
    pub fn new(namespace_type: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            namespace_type: namespace_type.into(),
            value: value.into(),
        }
    }
    
    /// Creates a Windows drive namespace
    pub fn windows_drive(drive: char) -> Self {
        Self {
            namespace_type: "drive".to_string(),
            value: format!("{}:", drive),
        }
    }
    
    /// Creates a UNC server namespace
    pub fn unc_server(server: impl Into<String>) -> Self {
        Self {
            namespace_type: "server".to_string(),
            value: server.into(),
        }
    }
    
    /// Creates a cloud storage namespace
    pub fn cloud(provider: impl Into<String>, bucket: impl Into<String>) -> Self {
        Self {
            namespace_type: "cloud".to_string(),
            value: format!("{}:{}", provider.into(), bucket.into()),
        }
    }
    
    /// Creates a service namespace
    pub fn service(service_id: impl Into<String>) -> Self {
        Self {
            namespace_type: "service".to_string(),
            value: service_id.into(),
        }
    }
}

/// Represents path components (the parts of a path delimited by separators)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathComponents {
    /// The individual components of the path
    components: Vec<String>,
    /// Whether this path is absolute
    is_absolute: bool,
}

impl PathComponents {
    /// Creates a new set of path components
    pub fn new(components: Vec<String>, is_absolute: bool) -> Self {
        Self {
            components,
            is_absolute,
        }
    }
    
    /// Creates path components from a string, parsing it according to the given path type
    pub fn from_str(path: &str, path_type: &PathType) -> Result<Self, Error> {
        let (is_absolute, separator) = match path_type {
            PathType::PosixLocal => (path.starts_with('/'), '/'),
            PathType::WindowsLocal | PathType::WindowsUNC => (
                path.starts_with('\\') || (path.len() >= 2 && path.chars().nth(1) == Some(':')),
                '\\'
            ),
            PathType::Cloud | PathType::Service | PathType::Other(_) => (path.starts_with('/'), '/'),
        };
        
        // Split the path into components
        let components = if path.is_empty() {
            Vec::new()
        } else {
            path.split(separator)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect()
        };
        
        Ok(Self {
            components,
            is_absolute,
        })
    }
    
    /// Returns the components as a slice
    pub fn as_slice(&self) -> &[String] {
        &self.components
    }
    
    /// Returns whether this path is absolute
    pub fn is_absolute(&self) -> bool {
        self.is_absolute
    }
    
    /// Normalizes the path components by removing "." and resolving ".."
    pub fn normalize(&self) -> Self {
        let mut result = Vec::new();
        
        for component in &self.components {
            match component.as_str() {
                "." => {}, // Skip this component
                ".." => {
                    if !result.is_empty() && result.last().unwrap() != ".." {
                        result.pop();
                    } else if !self.is_absolute {
                        // For relative paths, keep the ".." components
                        result.push("..".to_string());
                    }
                    // For absolute paths, ".." at the root is ignored
                },
                _ => result.push(component.clone()),
            }
        }
        
        Self {
            components: result,
            is_absolute: self.is_absolute,
        }
    }
    
    /// Joins this path with another path component
    pub fn join(&self, other: &PathComponents) -> Self {
        if other.is_absolute {
            // If the other path is absolute, just return it
            return other.clone();
        }
        
        let mut result = self.components.clone();
        result.extend(other.components.clone());
        
        Self {
            components: result,
            is_absolute: self.is_absolute,
        }
    }
    
    /// Appends a single component to the path
    pub fn push(&mut self, component: impl Into<String>) {
        self.components.push(component.into());
    }
    
    /// Returns the parent path components
    pub fn parent(&self) -> Option<Self> {
        if self.components.is_empty() {
            return None;
        }
        
        let mut components = self.components.clone();
        components.pop();
        
        Some(Self {
            components,
            is_absolute: self.is_absolute,
        })
    }
    
    /// Returns the last component of the path, if any
    pub fn file_name(&self) -> Option<&str> {
        self.components.last().map(|s| s.as_str())
    }
}

/// The main Path trait defining operations common to all path types
pub trait Path: Clone + fmt::Debug {
    /// Returns the type of this path
    fn path_type(&self) -> PathType;
    
    /// Returns the namespace of this path
    fn namespace(&self) -> &PathNamespace;
    
    /// Returns the components of this path
    fn components(&self) -> &PathComponents;
    
    /// Returns a normalized version of this path
    fn normalize(&self) -> Self;
    
    /// Joins this path with another path
    fn join(&self, path: &impl AsRef<str>) -> Result<Self, Error>;
    
    /// Converts this path to a string representation
    fn to_string(&self) -> String;
    
    /// Returns whether this path is absolute
    fn is_absolute(&self) -> bool {
        self.components().is_absolute()
    }
    
    /// Returns the parent path, if any
    fn parent(&self) -> Option<Self>;
    
    /// Returns the last component of the path, if any
    fn file_name(&self) -> Option<String> {
        self.components().file_name().map(|s| s.to_string())
    }
    
    /// Validates that this path is valid according to its path type's rules
    fn validate(&self) -> Result<(), Error>;
}

/// Specific implementation for local POSIX paths
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PosixPath {
    namespace: PathNamespace,
    components: PathComponents,
}

impl PosixPath {
    /// Creates a new POSIX path from a string
    pub fn new(path: impl AsRef<str>) -> Result<Self, Error> {
        let path = path.as_ref();
        let namespace = PathNamespace::new("posix", "");
        let components = PathComponents::from_str(path, &PathType::PosixLocal)?;
        
        let result = Self {
            namespace,
            components,
        };
        
        result.validate()?;
        Ok(result)
    }
}

impl Path for PosixPath {
    fn path_type(&self) -> PathType {
        PathType::PosixLocal
    }
    
    fn namespace(&self) -> &PathNamespace {
        &self.namespace
    }
    
    fn components(&self) -> &PathComponents {
        &self.components
    }
    
    fn normalize(&self) -> Self {
        Self {
            namespace: self.namespace.clone(),
            components: self.components.normalize(),
        }
    }
    
    fn join(&self, path: &impl AsRef<str>) -> Result<Self, Error> {
        let other_components = PathComponents::from_str(path.as_ref(), &PathType::PosixLocal)?;
        
        Ok(Self {
            namespace: self.namespace.clone(),
            components: self.components.join(&other_components),
        })
    }
    
    fn to_string(&self) -> String {
        let prefix = if self.is_absolute() { "/" } else { "" };
        format!("{}{}", prefix, self.components().as_slice().join("/"))
    }
    
    fn parent(&self) -> Option<Self> {
        self.components.parent().map(|components| Self {
            namespace: self.namespace.clone(),
            components,
        })
    }
    
    fn validate(&self) -> Result<(), Error> {
        // POSIX paths have few restrictions, but we should check for any invalid characters
        for component in self.components.as_slice() {
            if component.contains('\0') {
                return Err(crate::core::error("Path contains null characters"));
            }
        }
        Ok(())
    }
}

/// Specific implementation for Windows paths
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowsPath {
    namespace: PathNamespace,
    components: PathComponents,
}

impl WindowsPath {
    /// Creates a new Windows path from a string
    pub fn new(path: impl AsRef<str>) -> Result<Self, Error> {
        let path = path.as_ref();
        
        // Check if this is a UNC path
        if path.starts_with("\\\\") {
            return Self::new_unc(path);
        }
        
        // Check for drive letter
        let (namespace, remaining_path) = if path.len() >= 2 && path.chars().nth(1) == Some(':') {
            let drive = path.chars().next().unwrap();
            (
                PathNamespace::windows_drive(drive),
                &path[2..]
            )
        } else {
            // No drive letter, use current drive
            (
                PathNamespace::new("drive", ""),
                path
            )
        };
        
        let components = PathComponents::from_str(remaining_path, &PathType::WindowsLocal)?;
        
        let result = Self {
            namespace,
            components,
        };
        
        result.validate()?;
        Ok(result)
    }
    
    /// Creates a new Windows UNC path
    fn new_unc(path: &str) -> Result<Self, Error> {
        if !path.starts_with("\\\\") {
            return Err(crate::core::error("UNC path must start with '\\\\'"));
        }
        
        // Extract server name and share name
        let parts: Vec<&str> = path[2..].splitn(2, '\\').collect();
        
        if parts.is_empty() {
            return Err(crate::core::error("Invalid UNC path: missing server name"));
        }
        
        let server = parts[0].to_string();
        let namespace = PathNamespace::unc_server(server);
        
        let remaining_path = if parts.len() > 1 { parts[1] } else { "" };
        let components = PathComponents::from_str(remaining_path, &PathType::WindowsUNC)?;
        
        let result = Self {
            namespace,
            components,
        };
        
        result.validate()?;
        Ok(result)
    }
}

impl Path for WindowsPath {
    fn path_type(&self) -> PathType {
        if self.namespace.namespace_type == "server" {
            PathType::WindowsUNC
        } else {
            PathType::WindowsLocal
        }
    }
    
    fn namespace(&self) -> &PathNamespace {
        &self.namespace
    }
    
    fn components(&self) -> &PathComponents {
        &self.components
    }
    
    fn normalize(&self) -> Self {
        Self {
            namespace: self.namespace.clone(),
            components: self.components.normalize(),
        }
    }
    
    fn join(&self, path: &impl AsRef<str>) -> Result<Self, Error> {
        let path_str = path.as_ref();
        
        // Check if the other path has a drive letter
        if path_str.len() >= 2 && path_str.chars().nth(1) == Some(':') {
            // If it has a drive letter, create a new path
            return Self::new(path_str);
        }
        
        // Check if it's a UNC path
        if path_str.starts_with("\\\\") {
            return Self::new(path_str);
        }
        
        // Otherwise, join the paths
        let other_components = PathComponents::from_str(path_str, &self.path_type())?;
        
        Ok(Self {
            namespace: self.namespace.clone(),
            components: self.components.join(&other_components),
        })
    }
    
    fn to_string(&self) -> String {
        let path_type = self.path_type();
        
        match path_type {
            PathType::WindowsLocal => {
                let prefix = if !self.namespace.value.is_empty() {
                    format!("{}", self.namespace.value)
                } else {
                    "".to_string()
                };
                
                let separator = if self.is_absolute() || !prefix.is_empty() { "\\" } else { "" };
                format!("{}{}{}", prefix, separator, self.components().as_slice().join("\\"))
            },
            PathType::WindowsUNC => {
                format!("\\\\{}\\{}", self.namespace.value, self.components().as_slice().join("\\"))
            },
            _ => unreachable!("WindowsPath should only have WindowsLocal or WindowsUNC path types"),
        }
    }
    
    fn parent(&self) -> Option<Self> {
        self.components.parent().map(|components| Self {
            namespace: self.namespace.clone(),
            components,
        })
    }
    
    fn validate(&self) -> Result<(), Error> {
        // Check for invalid characters in Windows paths
        let invalid_chars = ['<', '>', ':', '"', '|', '?', '*'];
        
        for component in self.components.as_slice() {
            if component.contains(|c| invalid_chars.contains(&c) || c < ' ') {
                return Err(crate::core::error("Path contains invalid characters"));
            }
        }
        
        Ok(())
    }
}

/// Implementation for cloud storage paths
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloudPath {
    namespace: PathNamespace,
    components: PathComponents,
}

impl CloudPath {
    /// Creates a new cloud path
    pub fn new(provider: impl Into<String>, bucket: impl Into<String>, path: impl AsRef<str>) -> Result<Self, Error> {
        let namespace = PathNamespace::cloud(provider, bucket);
        let components = PathComponents::from_str(path.as_ref(), &PathType::Cloud)?;
        
        Ok(Self {
            namespace,
            components,
        })
    }
}

impl Path for CloudPath {
    fn path_type(&self) -> PathType {
        PathType::Cloud
    }
    
    fn namespace(&self) -> &PathNamespace {
        &self.namespace
    }
    
    fn components(&self) -> &PathComponents {
        &self.components
    }
    
    fn normalize(&self) -> Self {
        Self {
            namespace: self.namespace.clone(),
            components: self.components.normalize(),
        }
    }
    
    fn join(&self, path: &impl AsRef<str>) -> Result<Self, Error> {
        let other_components = PathComponents::from_str(path.as_ref(), &PathType::Cloud)?;
        
        Ok(Self {
            namespace: self.namespace.clone(),
            components: self.components.join(&other_components),
        })
    }
    
    fn to_string(&self) -> String {
        let prefix = if self.is_absolute() { "/" } else { "" };
        format!("{}/{}{}", self.namespace.value, prefix, self.components().as_slice().join("/"))
    }
    
    fn parent(&self) -> Option<Self> {
        self.components.parent().map(|components| Self {
            namespace: self.namespace.clone(),
            components,
        })
    }
    
    fn validate(&self) -> Result<(), Error> {
        // Cloud storage path validation - typically more permissive than local paths
        // Each provider might have specific rules
        Ok(())
    }
}

/// Implementation for service-specific paths (e.g., task tracking systems)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServicePath {
    namespace: PathNamespace,
    components: PathComponents,
}

impl ServicePath {
    /// Creates a new service path
    pub fn new(service_id: impl Into<String>, path: impl AsRef<str>) -> Result<Self, Error> {
        let namespace = PathNamespace::service(service_id);
        let components = PathComponents::from_str(path.as_ref(), &PathType::Service)?;
        
        Ok(Self {
            namespace,
            components,
        })
    }
}

impl Path for ServicePath {
    fn path_type(&self) -> PathType {
        PathType::Service
    }
    
    fn namespace(&self) -> &PathNamespace {
        &self.namespace
    }
    
    fn components(&self) -> &PathComponents {
        &self.components
    }
    
    fn normalize(&self) -> Self {
        Self {
            namespace: self.namespace.clone(),
            components: self.components.normalize(),
        }
    }
    
    fn join(&self, path: &impl AsRef<str>) -> Result<Self, Error> {
        let other_components = PathComponents::from_str(path.as_ref(), &PathType::Service)?;
        
        Ok(Self {
            namespace: self.namespace.clone(),
            components: self.components.join(&other_components),
        })
    }
    
    fn to_string(&self) -> String {
        let prefix = if self.is_absolute() { "/" } else { "" };
        format!("{}/{}{}", self.namespace.value, prefix, self.components().as_slice().join("/"))
    }
    
    fn parent(&self) -> Option<Self> {
        self.components.parent().map(|components| Self {
            namespace: self.namespace.clone(),
            components,
        })
    }
    
    fn validate(&self) -> Result<(), Error> {
        // Service path validation - rules would depend on the specific service
        Ok(())
    }
}

/// Factory function to create a local path appropriate for the current OS
pub fn create_local_path(path: impl AsRef<str>) -> Result<Box<dyn Path>, Error> {
    // Detect OS and create appropriate path
    #[cfg(target_os = "windows")]
    {
        Ok(Box::new(WindowsPath::new(path)?))
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Ok(Box::new(PosixPath::new(path)?))
    }
}

/// Factory function to create a path from a string, detecting the type
pub fn create_path(path: impl AsRef<str>) -> Result<Box<dyn Path>, Error> {
    let path = path.as_ref();
    
    // Check for special URI schemes
    if path.contains("://") {
        let parts: Vec<&str> = path.splitn(2, "://").collect();
        let scheme = parts[0];
        let rest = parts[1];
        
        match scheme {
            "file" => create_local_path(rest),
            s if s.starts_with("cloud-") => {
                let provider = s.strip_prefix("cloud-").unwrap();
                
                // Parse bucket and path
                let bucket_parts: Vec<&str> = rest.splitn(2, "/").collect();
                let bucket = bucket_parts[0];
                let path = if bucket_parts.len() > 1 { bucket_parts[1] } else { "" };
                
                Ok(Box::new(CloudPath::new(provider, bucket, path)?))
            },
            s if s.starts_with("service-") => {
                let service = s.strip_prefix("service-").unwrap();
                
                // Parse service specific path
                let parts: Vec<&str> = rest.splitn(2, "/").collect();
                let service_id = parts[0];
                let path = if parts.len() > 1 { parts[1] } else { "" };
                
                Ok(Box::new(ServicePath::new(format!("{}:{}", service, service_id), path)?))
            },
            _ => Err(crate::core::error(format!("Unsupported URI scheme: {}", scheme))),
        }
    } else {
        // Try to detect based on path format
        if cfg!(windows) && (path.contains('\\') || (path.len() >= 2 && path.chars().nth(1) == Some(':'))) {
            Ok(Box::new(WindowsPath::new(path)?))
        } else {
            Ok(Box::new(PosixPath::new(path)?))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_posix_path() {
        let path = PosixPath::new("/usr/local/bin").unwrap();
        
        assert_eq!(path.path_type(), PathType::PosixLocal);
        assert_eq!(path.is_absolute(), true);
        assert_eq!(path.to_string(), "/usr/local/bin");
        assert_eq!(path.components().as_slice(), &["usr", "local", "bin"]);
        
        let parent = path.parent().unwrap();
        assert_eq!(parent.to_string(), "/usr/local");
        
        let joined = path.join(&"../share").unwrap();
        let normalized = joined.normalize();
        assert_eq!(normalized.to_string(), "/usr/share");
    }
    
    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_create_local_path() {
        let path = create_local_path("/home/user/documents").unwrap();
        assert_eq!(path.path_type(), PathType::PosixLocal);
    }
    
    #[test]
    #[cfg(target_os = "windows")]
    fn test_create_local_path() {
        let path = create_local_path("C:\\Users\\user\\Documents").unwrap();
        assert_eq!(path.path_type(), PathType::WindowsLocal);
    }
    
    #[test]
    fn test_windows_path() {
        let path = WindowsPath::new("C:\\Users\\user\\Documents").unwrap();
        
        assert_eq!(path.path_type(), PathType::WindowsLocal);
        assert_eq!(path.is_absolute(), true);
        assert_eq!(path.to_string(), "C:\\Users\\user\\Documents");
        
        let parent = path.parent().unwrap();
        assert_eq!(parent.to_string(), "C:\\Users\\user");
        
        let joined = path.join(&"..\\Downloads").unwrap();
        let normalized = joined.normalize();
        assert_eq!(normalized.to_string(), "C:\\Users\\Downloads");
    }
    
    #[test]
    fn test_windows_unc_path() {
        let path = WindowsPath::new("\\\\server\\share\\folder").unwrap();
        
        assert_eq!(path.path_type(), PathType::WindowsUNC);
        assert_eq!(path.is_absolute(), true);
        assert_eq!(path.to_string(), "\\\\server\\share\\folder");
        
        let parent = path.parent().unwrap();
        assert_eq!(parent.to_string(), "\\\\server\\share");
    }
    
    #[test]
    fn test_cloud_path() {
        let path = CloudPath::new("aws", "my-bucket", "/folder/file.txt").unwrap();
        
        assert_eq!(path.path_type(), PathType::Cloud);
        assert_eq!(path.namespace().value, "aws:my-bucket");
        assert_eq!(path.to_string(), "aws:my-bucket//folder/file.txt");
        
        let parent = path.parent().unwrap();
        assert_eq!(parent.to_string(), "aws:my-bucket//folder");
    }
    
    #[test]
    fn test_service_path() {
        let path = ServicePath::new("jira", "/project/ticket-123").unwrap();
        
        assert_eq!(path.path_type(), PathType::Service);
        assert_eq!(path.namespace().value, "jira");
        assert_eq!(path.to_string(), "jira//project/ticket-123");
        
        let file_name = path.file_name().unwrap();
        assert_eq!(file_name, "ticket-123");
    }
    
    #[test]
    fn test_create_path_from_uri() {
        let path = create_path("file:///home/user/documents").unwrap();
        assert_eq!(path.to_string(), "/home/user/documents");
        
        let path = create_path("cloud-aws://my-bucket/folder/file.txt").unwrap();
        assert_eq!(path.path_type(), PathType::Cloud);
        assert_eq!(path.to_string(), "aws:my-bucket//folder/file.txt");
        
        let path = create_path("service-jira://project-x/ticket-123").unwrap();
        assert_eq!(path.path_type(), PathType::Service);
    }
} 