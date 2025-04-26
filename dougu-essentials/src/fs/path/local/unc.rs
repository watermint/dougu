use std::any::Any;
use std::fmt::Debug;
use std::sync::LazyLock;

use crate::core::error;
use crate::fs::path::core::{Namespace, Path, PathComponents};
use crate::fs::path::default::{DefaultNamespace, DefaultPathComponents};
use crate::fs::path::local::LocalPath;
use crate::fs::path::local::ServerInfo;

use super::LocalPathType;

static EMPTY_NAMESPACE: LazyLock<DefaultNamespace> =
    LazyLock::new(|| DefaultNamespace::new(String::new()));

/// Information about UNC path server
#[derive(Debug, Clone)]
pub struct UNCServerInfo {
    pub server: String,
    pub share: String,
}

impl ServerInfo for UNCServerInfo {
    fn server(&self) -> &str {
        &self.server
    }

    fn share(&self) -> Option<&str> {
        Some(&self.share)
    }

    fn credentials(&self) -> Option<&crate::fs::path::local::PathCredentials> {
        None
    }

    fn properties(&self) -> &[(&str, String)] {
        &[]
    }
}

/// UNC path implementation
#[derive(Debug, Clone)]
pub struct UNCLocalPath {
    pub(crate) components: DefaultPathComponents,
    pub(crate) server: String,
    pub(crate) share: String,
}

impl Path for UNCLocalPath {
    type ComponentsType = DefaultPathComponents;
    type NamespaceType = DefaultNamespace;

    fn new() -> Self {
        UNCLocalPath {
            components: Self::ComponentsType::new(),
            server: String::new(),
            share: String::new(),
        }
    }

    fn namespace(&self) -> &Self::NamespaceType {
        &EMPTY_NAMESPACE
    }

    fn namespace_mut(&mut self) -> &mut Self::NamespaceType {
        panic!("UNC paths don't support namespace modification")
    }

    fn components(&self) -> &Self::ComponentsType {
        &self.components
    }

    fn components_mut(&mut self) -> &mut Self::ComponentsType {
        &mut self.components
    }

    fn parse(path_str: &str) -> error::Result<Self> {
        if !path_str.starts_with("\\\\") {
            return Err(error::Error::msg(
                format!("Not a valid UNC path, must start with '\\\\'")
            ));
        }

        // Remove the leading "\\"
        let path_without_prefix = &path_str[2..];

        // Split by first backslash to get server and path
        let parts: Vec<&str> = path_without_prefix.splitn(2, '\\').collect();

        if parts.len() < 2 || parts[0].is_empty() {
            return Err(error::Error::msg(
                format!("Invalid UNC path format. Must include server and share")
            ));
        }

        let server = parts[0].to_string();

        // Get the share and remaining path
        let remaining = parts[1];
        let share_parts: Vec<&str> = remaining.splitn(2, '\\').collect();

        if share_parts.is_empty() || share_parts[0].is_empty() {
            return Err(error::Error::msg(
                format!("Invalid UNC path format. Share name is missing")
            ));
        }

        let share = share_parts[0].to_string();

        // Process the remaining path components
        let mut components = Self::ComponentsType::new();
        components.set_absolute(true); // UNC paths are always absolute

        if share_parts.len() > 1 && !share_parts[1].is_empty() {
            let path_parts: Vec<&str> = share_parts[1].split('\\').collect();
            for part in path_parts {
                if !part.is_empty() {
                    components.push(part);
                }
            }
        }

        Ok(UNCLocalPath {
            components,
            server,
            share,
        })
    }

    fn to_string(&self) -> String {
        let path_part = if self.components.is_empty() {
            String::new()
        } else {
            format!("\\{}", self.components.join_with_separator("\\"))
        };

        format!("\\\\{}\\{}{}", self.server, self.share, path_part)
    }

    fn join(&self, relative: &str) -> error::Result<Self> {
        // Check if the path to join is absolute
        if relative.starts_with('\\') {
            return Err(error::Error::msg(
                format!("Cannot join an absolute path to a UNC path")
            ));
        }

        let mut result = self.clone();

        // Split on backslashes
        let rel_parts: Vec<&str> = relative.split('\\').collect();
        for part in rel_parts {
            if !part.is_empty() {
                result.components_mut().push(part);
            }
        }

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
        let len = self.components().len();
        if len == 0 {
            None
        } else {
            self.components().get(len - 1).map(|s| s.to_string())
        }
    }

    fn normalize(&mut self) {
        self.components_mut().normalize();
    }

    fn is_absolute(&self) -> bool {
        true // UNC paths are always absolute
    }

    fn to_local_path(&self) -> Option<Box<dyn LocalPath<ComponentsType=DefaultPathComponents, NamespaceType=DefaultNamespace>>> {
        Some(Box::new(self.clone()) as Box<dyn LocalPath<ComponentsType=DefaultPathComponents, NamespaceType=DefaultNamespace>>)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl LocalPath for UNCLocalPath {
    fn path_type(&self) -> LocalPathType {
        LocalPathType::UNCPath
    }

    fn to_path_string(&self, target_type: LocalPathType) -> String {
        match target_type {
            LocalPathType::UNCPath => self.to_string(),
            LocalPathType::WindowsPath => {
                // Convert to Windows path
                // Only possible for localhost with a single-letter share (mapped drive)
                if self.server == "localhost" || self.server == "127.0.0.1" {
                    if self.share.len() == 1 && self.share.chars().next().unwrap().is_ascii_alphabetic() {
                        // Use share as drive letter
                        let drive = self.share.to_uppercase();
                        return format!("{}:\\{}", drive, self.components.join_with_separator("\\"));
                    }
                }
                // For non-local servers or multi-character shares, cannot convert reliably
                self.to_string()
            }
            LocalPathType::PosixPath => {
                // Convert to POSIX path
                format!("/{}", self.components.join_with_separator("/"))
            }
            LocalPathType::NFSPath => {
                // Convert to NFS path
                format!("//{}/{}/{}", self.server, self.share, self.components.join_with_separator("/"))
            }
            LocalPathType::SMBUrl => {
                // Convert to SMB URL
                format!("smb://{}/{}/{}", self.server, self.share, self.components.join_with_separator("/"))
            }
        }
    }

    fn create_os_path(path: &str) -> error::Result<Self> {
        Self::parse(path)
    }

    fn os_path_type() -> LocalPathType {
        LocalPathType::UNCPath
    }

    fn validate(&self) -> error::Result<()> {
        // Check server name
        if self.server.is_empty() {
            return Err(error::Error::msg(
                format!("Server name cannot be empty")
            ));
        }

        // Check share name
        if self.share.is_empty() {
            return Err(error::Error::msg(
                format!("Share name cannot be empty")
            ));
        }

        // Check components
        for i in 0..self.components().len() {
            if let Some(component) = self.components().get(i) {
                if component.contains('\\') || component.is_empty() {
                    return Err(error::Error::msg(
                        format!("Invalid path component: {}", component)
                    ));
                }
            }
        }

        Ok(())
    }

    fn server_info(&self) -> Option<Box<dyn ServerInfo>> {
        Some(Box::new(UNCServerInfo {
            server: self.server.clone(),
            share: self.share.clone(),
        }))
    }
} 