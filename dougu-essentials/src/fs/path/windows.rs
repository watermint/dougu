// Windows path implementation that handles regular, network, and UNC paths

use super::component::Component;
use super::traits::{PathFormat, PathTrait};
use std::fmt::Display;
use std::path::{Path as StdPath, PathBuf as StdPathBuf};
use crate::core::error::Error;
use crate::core::Result;

/// Windows path formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowsPathType {
    /// Regular path with drive letter (e.g., C:\Users\username)
    DriveRelative,
    /// Network path (e.g., \\server\share\folder)
    NetworkPath,
    /// UNC path (e.g., \\?\C:\very\long\path or \\?\UNC\server\share)
    UncPath,
    /// Current directory relative path (e.g., folder\file.txt)
    Relative,
}

/// Windows path implementation that handles regular, network, and UNC paths
#[derive(Clone, Debug)]
pub struct WindowsPath {
    /// Inner path buffer
    path: StdPathBuf,
    /// Type of Windows path
    path_type: WindowsPathType,
}

impl WindowsPath {
    /// Create a new Windows path
    pub fn new<P: AsRef<StdPath>>(path: P) -> Self {
        let path_buf = path.as_ref().to_path_buf();
        let path_str = path_buf.to_string_lossy();
        
        let path_type = if path_str.starts_with("\\\\?\\") {
            WindowsPathType::UncPath
        } else if path_str.starts_with("\\\\") && !path_str.starts_with("\\\\?\\") {
            WindowsPathType::NetworkPath
        } else if path_buf.has_root() && path_str.chars().nth(1) == Some(':') {
            WindowsPathType::DriveRelative
        } else {
            WindowsPathType::Relative
        };
        
        Self {
            path: path_buf,
            path_type,
        }
    }
    
    /// Create a new Windows path, validating that it exists
    pub fn new_existing<P: AsRef<StdPath>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(Error::from_string(format!("Path does not exist: {}", path.display())));
        }
        Ok(Self::new(path))
    }
    
    /// Get the path type
    pub fn path_type(&self) -> WindowsPathType {
        self.path_type
    }
    
    /// Get the inner path buffer
    pub fn as_path_buf(&self) -> &StdPathBuf {
        &self.path
    }
    
    /// Convert to inner path buffer
    pub fn into_path_buf(self) -> StdPathBuf {
        self.path
    }
    
    /// Get the drive letter if available
    pub fn drive_letter(&self) -> Option<char> {
        if self.path_type == WindowsPathType::DriveRelative {
            let path_str = self.path.to_string_lossy();
            path_str.chars().next()
        } else if self.path_type == WindowsPathType::UncPath {
            // Handle UNC paths with drive letters (\\?\C:\...)
            let path_str = self.path.to_string_lossy();
            if path_str.len() > 4 && path_str.chars().nth(4) != Some('U') {
                path_str.chars().nth(4)
            } else {
                None
            }
        } else {
            None
        }
    }
    
    /// Get the server name for network paths
    pub fn server_name(&self) -> Option<String> {
        if self.path_type == WindowsPathType::NetworkPath {
            let path_str = self.path.to_string_lossy();
            let parts: Vec<&str> = path_str.split('\\').filter(|s| !s.is_empty()).collect();
            if parts.len() >= 1 {
                Some(parts[0].to_string())
            } else {
                None
            }
        } else if self.path_type == WindowsPathType::UncPath && self.path.to_string_lossy().starts_with("\\\\?\\UNC\\") {
            let path_str = self.path.to_string_lossy();
            let parts: Vec<&str> = path_str.split('\\').filter(|s| !s.is_empty()).collect();
            if parts.len() >= 2 && parts[0] == "?" && parts[1] == "UNC" {
                if parts.len() >= 3 {
                    Some(parts[2].to_string())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }
    
    /// Get the share name for network paths
    pub fn share_name(&self) -> Option<String> {
        if self.path_type == WindowsPathType::NetworkPath {
            let path_str = self.path.to_string_lossy();
            let parts: Vec<&str> = path_str.split('\\').filter(|s| !s.is_empty()).collect();
            if parts.len() >= 2 {
                Some(parts[1].to_string())
            } else {
                None
            }
        } else if self.path_type == WindowsPathType::UncPath && self.path.to_string_lossy().starts_with("\\\\?\\UNC\\") {
            let path_str = self.path.to_string_lossy();
            let parts: Vec<&str> = path_str.split('\\').filter(|s| !s.is_empty()).collect();
            if parts.len() >= 2 && parts[0] == "?" && parts[1] == "UNC" {
                if parts.len() >= 4 {
                    Some(parts[3].to_string())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }
    
    /// Convert a relative path to an absolute path using the current directory
    pub fn to_absolute(&self, current_dir: &WindowsPath) -> Self {
        if self.is_absolute() {
            return self.clone();
        }
        
        match current_dir.path_type {
            WindowsPathType::DriveRelative | WindowsPathType::UncPath => {
                // Combine the current directory with the relative path
                Self::new(current_dir.path.join(&self.path))
            },
            WindowsPathType::NetworkPath => {
                // Combine the current network path with the relative path
                Self::new(current_dir.path.join(&self.path))
            },
            WindowsPathType::Relative => {
                // Both paths are relative, which doesn't make sense for conversion
                // Return the original path
                self.clone()
            }
        }
    }
    
    /// Convert a regular path to a UNC path
    pub fn to_unc(&self) -> Self {
        let path_str = self.path.to_string_lossy();
        
        match self.path_type {
            WindowsPathType::DriveRelative => {
                // Convert C:\path to \\?\C:\path
                Self::new(format!("\\\\?\\{}", path_str))
            },
            WindowsPathType::NetworkPath => {
                // Convert \\server\share to \\?\UNC\server\share
                let without_prefix = path_str.trim_start_matches('\\');
                Self::new(format!("\\\\?\\UNC\\{}", without_prefix))
            },
            WindowsPathType::UncPath | WindowsPathType::Relative => {
                // Already UNC or can't convert
                self.clone()
            }
        }
    }
}

impl PathTrait for WindowsPath {
    fn format(&self) -> PathFormat {
        PathFormat::Windows
    }
    
    fn is_absolute(&self) -> bool {
        match self.path_type {
            WindowsPathType::DriveRelative | WindowsPathType::NetworkPath | WindowsPathType::UncPath => true,
            WindowsPathType::Relative => false,
        }
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
        
        // Add a special handling for Windows paths
        match self.path_type {
            WindowsPathType::DriveRelative => {
                // Add drive component (e.g., "C:")
                if let Some(drive) = self.drive_letter() {
                    result.push(Component::new(&format!("{}:", drive)));
                }
                result.push(Component::root());
            },
            WindowsPathType::NetworkPath => {
                // Add server and share components
                result.push(Component::root());
                if let Some(server) = self.server_name() {
                    result.push(Component::new(&server));
                }
                if let Some(share) = self.share_name() {
                    result.push(Component::new(&share));
                }
            },
            WindowsPathType::UncPath => {
                // Handle UNC paths specially
                let path_str = self.path.to_string_lossy();
                if path_str.starts_with("\\\\?\\UNC\\") {
                    // UNC network path
                    result.push(Component::root());
                    // Add server and share components
                    if let Some(server) = self.server_name() {
                        result.push(Component::new(&server));
                    }
                    if let Some(share) = self.share_name() {
                        result.push(Component::new(&share));
                    }
                } else {
                    // UNC local path
                    // Add drive component
                    if let Some(drive) = self.drive_letter() {
                        result.push(Component::new(&format!("{}:", drive)));
                    }
                    result.push(Component::root());
                }
            },
            WindowsPathType::Relative => {
                // No special components for relative paths
            }
        }
        
        // Add the regular components (excluding the special handling above)
        let skip_count = match self.path_type {
            WindowsPathType::DriveRelative => 1, // Skip the drive component
            WindowsPathType::NetworkPath => 2,   // Skip server and share
            WindowsPathType::UncPath => {
                if self.path.to_string_lossy().starts_with("\\\\?\\UNC\\") {
                    3 // Skip prefix, server, and share for UNC network paths
                } else {
                    2 // Skip prefix and drive for UNC local paths
                }
            },
            WindowsPathType::Relative => 0,      // No skipping for relative paths
        };
        
        // Convert standard path components to our components
        let std_components: Vec<_> = self.path.components().collect();
        for component in std_components.iter().skip(skip_count) {
            match component {
                std::path::Component::Prefix(_) => {
                    // Skip prefix components as we've already handled them
                },
                std::path::Component::RootDir => {
                    // Skip root dir as we've already added it
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
                if !components.is_empty() && 
                   !components.last().unwrap().is_root && 
                   // Don't pop drive or server/share components
                   components.len() > 1 && 
                   (self.path_type != WindowsPathType::DriveRelative || components.len() > 2) &&
                   (self.path_type != WindowsPathType::NetworkPath || components.len() > 3) &&
                   (self.path_type != WindowsPathType::UncPath || components.len() > 3) {
                    components.pop();
                }
            } else {
                components.push(component);
            }
        }
        
        // Reconstruct the normalized path based on path type
        let mut result = String::new();
        
        match self.path_type {
            WindowsPathType::DriveRelative => {
                // Format: C:\path\to\file
                // First component should be the drive letter
                if components.len() >= 1 {
                    result.push_str(&components[0].name);
                    result.push('\\');
                    
                    // Add remaining components
                    for component in components.iter().skip(2) { // Skip drive and root
                        if !component.is_root {
                            result.push_str(&component.name);
                            result.push('\\');
                        }
                    }
                }
            },
            WindowsPathType::NetworkPath => {
                // Format: \\server\share\path\to\file
                result.push_str("\\\\");
                
                // Add server and share
                if components.len() >= 2 {
                    result.push_str(&components[1].name); // Server
                    result.push('\\');
                    
                    if components.len() >= 3 {
                        result.push_str(&components[2].name); // Share
                        result.push('\\');
                        
                        // Add remaining components
                        for component in components.iter().skip(3) {
                            if !component.is_root {
                                result.push_str(&component.name);
                                result.push('\\');
                            }
                        }
                    }
                }
            },
            WindowsPathType::UncPath => {
                // Format depends on UNC type
                let path_str = self.path.to_string_lossy();
                if path_str.starts_with("\\\\?\\UNC\\") {
                    // UNC network path: \\?\UNC\server\share\path
                    result.push_str("\\\\?\\UNC\\");
                    
                    // Add server and share
                    if components.len() >= 2 {
                        result.push_str(&components[1].name); // Server
                        result.push('\\');
                        
                        if components.len() >= 3 {
                            result.push_str(&components[2].name); // Share
                            result.push('\\');
                            
                            // Add remaining components
                            for component in components.iter().skip(3) {
                                if !component.is_root {
                                    result.push_str(&component.name);
                                    result.push('\\');
                                }
                            }
                        }
                    }
                } else {
                    // UNC local path: \\?\C:\path
                    result.push_str("\\\\?\\");
                    
                    // Add drive letter
                    if components.len() >= 1 {
                        result.push_str(&components[0].name);
                        result.push('\\');
                        
                        // Add remaining components
                        for component in components.iter().skip(2) { // Skip drive and root
                            if !component.is_root {
                                result.push_str(&component.name);
                                result.push('\\');
                            }
                        }
                    }
                }
            },
            WindowsPathType::Relative => {
                // Just join the components with backslashes
                let mut first = true;
                for component in components {
                    if !first {
                        result.push('\\');
                    }
                    result.push_str(&component.name);
                    first = false;
                }
            }
        }
        
        // Remove trailing backslash if it exists (except for root paths)
        if result.ends_with('\\') && 
           !((self.path_type == WindowsPathType::DriveRelative && result.len() == 3) || // C:\
             (self.path_type == WindowsPathType::NetworkPath && result.len() <= 4) ||   // \\a\
             (self.path_type == WindowsPathType::UncPath && result.len() <= 8)) {       // \\?\C:\
            result.pop();
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
            
            // Update path type if the new path has a root
            if path.as_ref().has_root() {
                let path_str = self.path.to_string_lossy();
                self.path_type = if path_str.starts_with("\\\\?\\") {
                    WindowsPathType::UncPath
                } else if path_str.starts_with("\\\\") && !path_str.starts_with("\\\\?\\") {
                    WindowsPathType::NetworkPath
                } else if self.path.has_root() && path_str.chars().nth(1) == Some(':') {
                    WindowsPathType::DriveRelative
                } else {
                    WindowsPathType::Relative
                };
            }
        }
    }
    
    fn with_extension(&self, extension: &str) -> Self {
        Self::new(self.path.with_extension(extension))
    }
    
    fn with_file_name(&self, file_name: &str) -> Self {
        Self::new(self.path.with_file_name(file_name))
    }
}

impl AsRef<WindowsPath> for WindowsPath {
    fn as_ref(&self) -> &WindowsPath {
        self
    }
}

impl Display for WindowsPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_windows_path_types() {
        // Drive relative paths
        let path1 = WindowsPath::new("C:\\Users\\username\\Documents");
        assert_eq!(path1.path_type(), WindowsPathType::DriveRelative);
        assert_eq!(path1.drive_letter(), Some('C'));
        assert!(path1.is_absolute());
        
        // Network paths
        let path2 = WindowsPath::new("\\\\server\\share\\folder");
        assert_eq!(path2.path_type(), WindowsPathType::NetworkPath);
        assert_eq!(path2.server_name(), Some("server".to_string()));
        assert_eq!(path2.share_name(), Some("share".to_string()));
        assert!(path2.is_absolute());
        
        // UNC paths
        let path3 = WindowsPath::new("\\\\?\\C:\\very\\long\\path");
        assert_eq!(path3.path_type(), WindowsPathType::UncPath);
        assert_eq!(path3.drive_letter(), Some('C'));
        assert!(path3.is_absolute());
        
        let path4 = WindowsPath::new("\\\\?\\UNC\\server\\share\\folder");
        assert_eq!(path4.path_type(), WindowsPathType::UncPath);
        assert_eq!(path4.server_name(), Some("server".to_string()));
        assert_eq!(path4.share_name(), Some("share".to_string()));
        assert!(path4.is_absolute());
        
        // Relative paths
        let path5 = WindowsPath::new("folder\\file.txt");
        assert_eq!(path5.path_type(), WindowsPathType::Relative);
        assert!(!path5.is_absolute());
    }
    
    #[test]
    fn test_normalize_paths() {
        // Test drive relative path normalization
        let path1 = WindowsPath::new("C:\\Users\\..\\Public\\.\\Downloads");
        let norm1 = path1.normalize();
        assert_eq!(norm1.to_string(), "C:\\Public\\Downloads");
        
        // Test network path normalization
        let path2 = WindowsPath::new("\\\\server\\share\\folder\\.\\..\\otherFolder");
        let norm2 = path2.normalize();
        assert_eq!(norm2.to_string(), "\\\\server\\share\\otherFolder");
        
        // Test UNC path normalization
        let path3 = WindowsPath::new("\\\\?\\C:\\temp\\..\\Program Files\\.\\folder");
        let norm3 = path3.normalize();
        assert_eq!(norm3.to_string(), "\\\\?\\C:\\Program Files\\folder");
        
        // Test relative path normalization
        let path4 = WindowsPath::new("folder\\..\\otherFolder\\.\\file.txt");
        let norm4 = path4.normalize();
        assert_eq!(norm4.to_string(), "otherFolder\\file.txt");
    }
    
    #[test]
    fn test_windows_path_components() {
        // Test drive relative path components
        let path1 = WindowsPath::new("C:\\Users\\username");
        let components1 = path1.components();
        assert_eq!(components1.len(), 4);
        assert_eq!(components1[0].name, "C:");
        assert!(components1[1].is_root);
        assert_eq!(components1[2].name, "Users");
        assert_eq!(components1[3].name, "username");
        
        // Test network path components
        let path2 = WindowsPath::new("\\\\server\\share\\folder");
        let components2 = path2.components();
        assert_eq!(components2.len(), 4);
        assert!(components2[0].is_root);
        assert_eq!(components2[1].name, "server");
        assert_eq!(components2[2].name, "share");
        assert_eq!(components2[3].name, "folder");
        
        // Test UNC path components
        let path3 = WindowsPath::new("\\\\?\\UNC\\server\\share\\folder");
        let components3 = path3.components();
        assert_eq!(components3.len(), 4);
        assert!(components3[0].is_root);
        assert_eq!(components3[1].name, "server");
        assert_eq!(components3[2].name, "share");
        assert_eq!(components3[3].name, "folder");
    }
    
    #[test]
    fn test_to_unc() {
        // Convert drive relative to UNC
        let path1 = WindowsPath::new("C:\\Users\\username");
        let unc1 = path1.to_unc();
        assert_eq!(unc1.to_string(), "\\\\?\\C:\\Users\\username");
        assert_eq!(unc1.path_type(), WindowsPathType::UncPath);
        
        // Convert network path to UNC
        let path2 = WindowsPath::new("\\\\server\\share\\folder");
        let unc2 = path2.to_unc();
        assert_eq!(unc2.to_string(), "\\\\?\\UNC\\server\\share\\folder");
        assert_eq!(unc2.path_type(), WindowsPathType::UncPath);
    }
} 