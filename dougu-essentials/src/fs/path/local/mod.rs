use std::any::Any;
use std::fmt::Debug;

use crate::core::error;
use crate::fs::path::core::{Path, PathComponents, Namespace};
use crate::fs::path::default::{DefaultNamespace, DefaultPathComponents};

// Export child modules
pub mod posix;
pub mod windows;
pub mod unc;
pub mod nfs;
pub mod smb;

// Re-export types
pub use posix::PosixLocalPath;
pub use windows::WindowsLocalPath;
pub use unc::UNCLocalPath;
pub use nfs::NFSLocalPath;
pub use smb::{SMBLocalPath, SMBServerInfo};

/// ServerInfo provides access to server-related information for network paths
pub trait ServerInfo: Debug + Send + Sync {
    /// Get the server name or host
    fn server(&self) -> &str;
    
    /// Get the share name if available
    fn share(&self) -> Option<&str>;
    
    /// Get authentication credentials if available
    fn credentials(&self) -> Option<&PathCredentials>;
    
    /// Get additional server properties
    fn properties(&self) -> &[(&str, String)];
    
    /// Check if this server info has a specific property
    fn has_property(&self, key: &str) -> bool {
        self.properties().iter().any(|(k, _)| *k == key)
    }
    
    /// Get a specific property value
    fn get_property(&self, key: &str) -> Option<&str> {
        self.properties().iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v.as_str())
    }
}

/// StandardServerInfo provides a default implementation of the ServerInfo trait
#[derive(Debug, Clone)]
pub struct StandardServerInfo {
    server_name: String,
    share_name: Option<String>,
    credentials: Option<PathCredentials>,
    properties: Vec<(String, String)>,
}

impl StandardServerInfo {
    /// Create a new StandardServerInfo
    pub fn new(
        server_name: &str,
        share_name: Option<&str>,
        credentials: Option<PathCredentials>,
    ) -> Self {
        StandardServerInfo {
            server_name: server_name.to_string(),
            share_name: share_name.map(|s| s.to_string()),
            credentials,
            properties: Vec::new(),
        }
    }
    
    /// Add a property to this server info
    pub fn add_property(&mut self, key: &str, value: &str) {
        self.properties.push((key.to_string(), value.to_string()));
    }
}

impl ServerInfo for StandardServerInfo {
    fn server(&self) -> &str {
        &self.server_name
    }
    
    fn share(&self) -> Option<&str> {
        self.share_name.as_deref()
    }
    
    fn credentials(&self) -> Option<&PathCredentials> {
        self.credentials.as_ref()
    }
    
    fn properties(&self) -> &[(&str, String)] {
        // This is a bit of a hack due to Rust's lifetimes.
        // In a real implementation, we might want to use a different approach.
        &[]
    }
}

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
pub trait LocalPath: Path {
    /// Get the path type of this local path
    fn path_type(&self) -> LocalPathType;
    
    /// Convert to a path string in the specified format
    fn to_path_string(&self, target_type: LocalPathType) -> String;
    
    /// Create a path in the current OS format
    fn create_os_path(path: &str) -> error::Result<Self> where Self: Sized;
    
    /// Get the current OS path type
    fn os_path_type() -> LocalPathType where Self: Sized;
    
    /// Validate if this is a valid path according to the operating system rules
    fn validate(&self) -> error::Result<()>;
    
    /// Get server information if this is a server path (PosixServer, WindowsUNC, or SMB)
    /// Returns server information if available
    /// Returns None if this is not a server path
    fn server_info(&self) -> Option<Box<dyn ServerInfo>> {
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
                    components.get(1)
                } else {
                    None
                };
                
                let server_info = StandardServerInfo::new(
                    &server_name,
                    share_name,
                    None
                );
                
                Some(Box::new(server_info))
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
                    
                    (server_part, creds)
                } else {
                    (without_protocol, None)
                };
                
                // Get share name from first component if available
                let share_name = if !self.components().is_empty() {
                    self.components().get(0)
                } else {
                    None
                };
                
                let server_info = StandardServerInfo::new(
                    server,
                    share_name,
                    credentials
                );
                
                Some(Box::new(server_info))
            },
            _ => None
        }
    }
} 