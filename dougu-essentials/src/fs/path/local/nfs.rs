use std::any::Any;
use std::fmt::Debug;
use std::sync::LazyLock;

use crate::core::error;
use crate::fs::path::core::{Namespace, Path, PathComponents};
use crate::fs::path::default::{DefaultNamespace, DefaultPathComponents};
use crate::fs::path::local::LocalPath;
use crate::fs::path::local::PathCredentials;
use crate::fs::path::local::ServerInfo;

use super::LocalPathType;

static EMPTY_NAMESPACE: LazyLock<DefaultNamespace> =
    LazyLock::new(|| DefaultNamespace::new(String::new()));

/// Information about NFS path server
#[derive(Debug, Clone)]
pub struct NFSServerInfo {
    pub server: String,
    pub export: String,
}

impl ServerInfo for NFSServerInfo {
    fn server(&self) -> &str {
        &self.server
    }

    fn share(&self) -> Option<&str> {
        Some(&self.export)
    }

    fn credentials(&self) -> Option<&PathCredentials> {
        None
    }

    fn properties(&self) -> &[(&str, String)] {
        &[]
    }
}

/// NFS path implementation
#[derive(Debug, Clone)]
pub struct NFSLocalPath {
    pub(crate) components: DefaultPathComponents,
    pub(crate) server: String,
    pub(crate) export: String,
}

impl Path for NFSLocalPath {
    type ComponentsType = DefaultPathComponents;
    type NamespaceType = DefaultNamespace;

    fn new() -> Self {
        NFSLocalPath {
            components: Self::ComponentsType::new(),
            server: String::new(),
            export: String::new(),
        }
    }

    fn namespace(&self) -> &Self::NamespaceType {
        &EMPTY_NAMESPACE
    }

    fn namespace_mut(&mut self) -> &mut Self::NamespaceType {
        panic!("NFS paths don't support namespace modification")
    }

    fn components(&self) -> &Self::ComponentsType {
        &self.components
    }

    fn components_mut(&mut self) -> &mut Self::ComponentsType {
        &mut self.components
    }

    fn parse(path_str: &str) -> error::Result<Self> {
        // NFS paths have format: "//<server>/<share>/path"
        if !path_str.starts_with("//") {
            return Err(error::Error::msg(
                format!("Not a valid NFS path, must start with '//'")
            ));
        }

        // Remove the leading "//"
        let path_without_prefix = &path_str[2..];

        // Split by first slash to get server and path
        let parts: Vec<&str> = path_without_prefix.splitn(2, '/').collect();

        if parts.len() < 2 || parts[0].is_empty() {
            return Err(error::Error::msg(
                format!("Invalid NFS path format. Must include server and share")
            ));
        }

        let server = parts[0].to_string();

        // Get the share and remaining path
        let remaining = parts[1];
        let share_parts: Vec<&str> = remaining.splitn(2, '/').collect();

        if share_parts.is_empty() || share_parts[0].is_empty() {
            return Err(error::Error::msg(
                format!("Invalid NFS path format. Share name is missing")
            ));
        }

        let share = share_parts[0].to_string();

        // Process the remaining path components
        let mut components = Self::ComponentsType::new();
        components.set_absolute(true); // NFS paths are always absolute

        if share_parts.len() > 1 && !share_parts[1].is_empty() {
            let path_parts: Vec<&str> = share_parts[1].split('/').collect();
            for part in path_parts {
                if !part.is_empty() {
                    components.push(part);
                }
            }
        }

        Ok(NFSLocalPath {
            components,
            server,
            export: share,
        })
    }

    fn to_string(&self) -> String {
        let path_part = if self.components.is_empty() {
            String::new()
        } else {
            format!("/{}", self.components.join_with_separator("/"))
        };

        format!("//{}/{}{}", self.server, self.export, path_part)
    }

    fn join(&self, relative: &str) -> error::Result<Self> {
        // Check if the path to join is absolute
        if relative.starts_with('/') || relative.starts_with("//") {
            return Err(error::Error::msg(
                format!("Cannot join an absolute path to an NFS path")
            ));
        }

        let mut result = self.clone();

        // Split on slashes
        let rel_parts: Vec<&str> = relative.split('/').collect();
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
        true // NFS paths are always absolute
    }

    fn to_local_path(&self) -> Option<Box<dyn LocalPath<ComponentsType=DefaultPathComponents, NamespaceType=DefaultNamespace>>> {
        Some(Box::new(self.clone()) as Box<dyn LocalPath<ComponentsType=DefaultPathComponents, NamespaceType=DefaultNamespace>>)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl LocalPath for NFSLocalPath {
    fn path_type(&self) -> LocalPathType {
        LocalPathType::NFSPath
    }

    fn to_path_string(&self, target_type: LocalPathType) -> String {
        match target_type {
            LocalPathType::NFSPath => self.to_string(),
            LocalPathType::UNCPath => {
                // Convert to UNC path
                format!("\\\\{}\\{}\\{}", self.server, self.export, self.components.join_with_separator("\\"))
            }
            LocalPathType::WindowsPath => {
                // Cannot reliably convert to Windows path - use UNC format
                format!("\\\\{}\\{}\\{}", self.server, self.export, self.components.join_with_separator("\\"))
            }
            LocalPathType::PosixPath => {
                // Convert to POSIX path (assuming mounted)
                format!("/{}/{}/{}", "nfs", self.server, self.components.join_with_separator("/"))
            }
            LocalPathType::SMBUrl => {
                // Convert to SMB URL (assuming same export is available via SMB)
                format!("smb://{}/{}/{}", self.server, self.export, self.components.join_with_separator("/"))
            }
        }
    }

    fn create_os_path(path: &str) -> error::Result<Self> {
        Self::parse(path)
    }

    fn os_path_type() -> LocalPathType {
        LocalPathType::NFSPath
    }

    fn validate(&self) -> error::Result<()> {
        // Check server name
        if self.server.is_empty() {
            return Err(error::Error::msg(
                format!("Server name cannot be empty")
            ));
        }

        // Check share name
        if self.export.is_empty() {
            return Err(error::Error::msg(
                format!("Share name cannot be empty")
            ));
        }

        // Check components
        for i in 0..self.components().len() {
            if let Some(component) = self.components().get(i) {
                if component.contains('/') || component.is_empty() {
                    return Err(error::Error::msg(
                        format!("Invalid path component: {}", component)
                    ));
                }
            }
        }

        Ok(())
    }

    fn server_info(&self) -> Option<Box<dyn ServerInfo>> {
        Some(Box::new(NFSServerInfo {
            server: self.server.clone(),
            export: self.export.clone(),
        }))
    }
} 