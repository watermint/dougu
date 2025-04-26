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

/// POSIX path implementation
#[derive(Debug, Clone)]
pub struct PosixLocalPath {
    pub(crate) components: DefaultPathComponents,
}

impl Path for PosixLocalPath {
    type ComponentsType = DefaultPathComponents;
    type NamespaceType = DefaultNamespace;

    fn new() -> Self {
        PosixLocalPath {
            components: Self::ComponentsType::new(),
        }
    }

    fn namespace(&self) -> &Self::NamespaceType {
        &EMPTY_NAMESPACE
    }

    fn namespace_mut(&mut self) -> &mut Self::NamespaceType {
        panic!("POSIX paths don't support namespace modification")
    }

    fn components(&self) -> &Self::ComponentsType {
        &self.components
    }

    fn components_mut(&mut self) -> &mut Self::ComponentsType {
        &mut self.components
    }

    fn parse(path_str: &str) -> error::Result<Self> {
        // Check if this is a POSIX path (not checking OS)
        if path_str.contains('\\') {
            return Err(error::Error::msg(format!("Not a valid POSIX path")));
        }

        // Handle relative or absolute paths
        let components = Self::ComponentsType::from_string(path_str);

        Ok(PosixLocalPath {
            components,
        })
    }

    fn to_string(&self) -> String {
        if self.is_absolute() {
            format!("/{}", self.components.join())
        } else {
            self.components.join()
        }
    }

    fn join(&self, relative: &str) -> error::Result<Self> {
        if relative.starts_with('/') {
            return Err(error::Error::msg(format!("Cannot join an absolute path")));
        }

        let mut result = self.clone();
        let rel_components = Self::ComponentsType::from_string(relative);

        for i in 0..rel_components.len() {
            if let Some(component) = rel_components.get(i) {
                result.components_mut().push(component);
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
        self.components.is_absolute()
    }

    fn to_local_path(&self) -> Option<Box<dyn LocalPath<ComponentsType=DefaultPathComponents, NamespaceType=DefaultNamespace>>> {
        Some(Box::new(self.clone()) as Box<dyn LocalPath<ComponentsType=DefaultPathComponents, NamespaceType=DefaultNamespace>>)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl LocalPath for PosixLocalPath {
    fn path_type(&self) -> LocalPathType {
        LocalPathType::PosixPath
    }

    fn to_path_string(&self, target_type: LocalPathType) -> String {
        match target_type {
            LocalPathType::PosixPath => self.to_string(),
            LocalPathType::WindowsPath => {
                // Convert to Windows path
                // For an absolute path, map to C: drive
                let path_part = self.components().join().replace('/', "\\");
                if self.is_absolute() {
                    format!("C:\\{}", path_part)
                } else {
                    path_part
                }
            }
            LocalPathType::UNCPath => {
                // Cannot reliably convert to UNC without server information
                format!("\\\\server\\share\\{}", self.components().join().replace('/', "\\"))
            }
            LocalPathType::NFSPath => {
                // Cannot reliably convert to NFS without server information
                format!("//server/{}", self.components().join())
            }
            LocalPathType::SMBUrl => {
                // Cannot reliably convert to SMB URL without server information
                format!("smb://server/{}", self.components().join())
            }
        }
    }

    fn create_os_path(path: &str) -> error::Result<Self> {
        Self::parse(path)
    }

    fn os_path_type() -> LocalPathType {
        LocalPathType::PosixPath
    }

    fn validate(&self) -> error::Result<()> {
        // Check components
        for i in 0..self.components().len() {
            if let Some(component) = self.components().get(i) {
                if component.contains('/') || component.is_empty() {
                    return Err(error::Error::msg(format!("Invalid path component: {}", component)));
                }
            }
        }

        Ok(())
    }

    fn server_info(&self) -> Option<Box<dyn ServerInfo>> {
        None // POSIX paths don't have server information
    }
} 