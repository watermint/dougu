use std::any::Any;
use std::fmt::Debug;
use std::sync::LazyLock;

use crate::core::error;
use crate::fs::path::core::{Namespace, Path, PathComponents};
use crate::fs::path::default::{DefaultNamespace, DefaultPathComponents};
use crate::fs::path::local::LocalPath;

use super::{LocalPathType, ServerInfo};

static EMPTY_NAMESPACE: LazyLock<DefaultNamespace> =
    LazyLock::new(|| DefaultNamespace::new(String::new()));

/// Windows path implementation
#[derive(Debug, Clone)]
pub struct WindowsLocalPath {
    pub(crate) components: DefaultPathComponents,
    pub(crate) drive: Option<String>,
}

impl Path for WindowsLocalPath {
    type ComponentsType = DefaultPathComponents;
    type NamespaceType = DefaultNamespace;

    fn new() -> Self {
        WindowsLocalPath {
            components: Self::ComponentsType::new(),
            drive: None,
        }
    }

    fn namespace(&self) -> &Self::NamespaceType {
        &EMPTY_NAMESPACE
    }

    fn namespace_mut(&mut self) -> &mut Self::NamespaceType {
        panic!("Windows paths don't support namespace modification")
    }

    fn components(&self) -> &Self::ComponentsType {
        &self.components
    }

    fn components_mut(&mut self) -> &mut Self::ComponentsType {
        &mut self.components
    }

    fn parse(path_str: &str) -> error::Result<Self> {
        // Validate whether this is a Windows path
        if path_str.contains('/') {
            return Err(error::Error::msg(format!("Not a valid Windows path, contains '/'")));
        }

        // Extract drive letter if present
        let (drive, path_without_drive) = if path_str.len() >= 2 && path_str.chars().nth(1) == Some(':') {
            let drive_letter = path_str.chars().next().unwrap().to_string();
            if !drive_letter.chars().next().unwrap().is_ascii_alphabetic() {
                return Err(error::Error::msg(format!("Invalid drive letter")));
            }

            // Extract the path part (skip drive letter and colon)
            let path_part = if path_str.len() > 2 {
                &path_str[2..]
            } else {
                ""
            };

            (Some(drive_letter), path_part)
        } else {
            (None, path_str)
        };

        // Handle components
        let mut components = Self::ComponentsType::new();

        if !path_without_drive.is_empty() {
            // Windows paths use backslash as separator
            let parts: Vec<&str> = path_without_drive.split('\\').collect();

            // Skip the first element if it's empty (indicating a root path)
            let start_idx = if !parts.is_empty() && parts[0].is_empty() { 1 } else { 0 };

            for i in start_idx..parts.len() {
                if !parts[i].is_empty() {
                    components.push(parts[i]);
                }
            }

            if start_idx == 1 || (drive.is_some() && path_without_drive.starts_with('\\')) {
                components.set_absolute(true);
            }
        }

        Ok(WindowsLocalPath {
            components,
            drive,
        })
    }

    fn to_string(&self) -> String {
        let path_part = if self.components.is_empty() {
            String::new()
        } else if self.is_absolute() {
            format!("\\{}", self.components.join_with_separator("\\"))
        } else {
            self.components.join_with_separator("\\")
        };

        if let Some(drive) = &self.drive {
            format!("{}:{}", drive, path_part)
        } else {
            path_part
        }
    }

    fn join(&self, relative: &str) -> error::Result<Self> {
        // Check if the path to join is absolute
        if let Ok(rel_path) = Self::parse(relative) {
            if rel_path.is_absolute() || rel_path.drive.is_some() {
                return Err(error::Error::msg(format!("Cannot join an absolute or drive-qualified path")));
            }
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
        self.components.is_absolute() || self.drive.is_some()
    }

    fn to_local_path(&self) -> Option<Box<dyn LocalPath<ComponentsType=DefaultPathComponents, NamespaceType=DefaultNamespace>>> {
        Some(Box::new(self.clone()) as Box<dyn LocalPath<ComponentsType=DefaultPathComponents, NamespaceType=DefaultNamespace>>)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl LocalPath for WindowsLocalPath {
    fn path_type(&self) -> LocalPathType {
        LocalPathType::WindowsPath
    }

    fn to_path_string(&self, target_type: LocalPathType) -> String {
        match target_type {
            LocalPathType::WindowsPath => self.to_string(),
            LocalPathType::PosixPath => {
                // Convert to POSIX path
                let path_part = self.components().join_with_separator("/");
                if self.is_absolute() {
                    format!("/{}", path_part)
                } else {
                    path_part
                }
            }
            LocalPathType::UNCPath => {
                // Convert to UNC path
                if let Some(drive) = &self.drive {
                    // Map drive letter to a server share
                    format!("\\\\localhost\\{}\\{}", drive.to_lowercase(), self.components().join_with_separator("\\"))
                } else {
                    // Cannot reliably convert without drive
                    format!("\\\\server\\share\\{}", self.components().join_with_separator("\\"))
                }
            }
            LocalPathType::NFSPath => {
                // Convert to NFS path
                if let Some(drive) = &self.drive {
                    // Map drive letter to a server share
                    format!("//localhost/{}/{}", drive.to_lowercase(), self.components().join_with_separator("/"))
                } else {
                    // Cannot reliably convert without drive
                    format!("//server/{}", self.components().join_with_separator("/"))
                }
            }
            LocalPathType::SMBUrl => {
                // Convert to SMB URL
                if let Some(drive) = &self.drive {
                    // Map drive letter to a server share
                    format!("smb://localhost/{}/{}", drive.to_lowercase(), self.components().join_with_separator("/"))
                } else {
                    // Cannot reliably convert without drive
                    format!("smb://server/{}", self.components().join_with_separator("/"))
                }
            }
        }
    }

    fn create_os_path(path: &str) -> error::Result<Self> {
        Self::parse(path)
    }

    fn os_path_type() -> LocalPathType {
        LocalPathType::WindowsPath
    }

    fn validate(&self) -> error::Result<()> {
        // Check drive letter
        if let Some(drive) = &self.drive {
            if drive.len() != 1 || !drive.chars().next().unwrap().is_ascii_alphabetic() {
                return Err(error::Error::msg(format!("Invalid drive letter")));
            }
        }

        // Check components
        for i in 0..self.components().len() {
            if let Some(component) = self.components().get(i) {
                if component.contains('\\') || component.contains(':') || component.is_empty() {
                    return Err(error::Error::msg(format!("Invalid path component: {}", component)));
                }
            }
        }

        Ok(())
    }

    fn server_info(&self) -> Option<Box<dyn ServerInfo>> {
        None // Windows paths don't have server information
    }
} 