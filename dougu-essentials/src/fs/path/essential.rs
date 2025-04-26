// Essential Path implementation
use std::any::Any;
use std::fmt::{Debug, Display};

use crate::core::error;
use crate::fs::path::core::{Namespace, Path, PathComponents};
use crate::fs::path::default::{DefaultNamespace, DefaultPathComponents};
use crate::fs::path::local::LocalPath;

/// Generic trait for converting between EssentialPath and specific path implementations
pub trait PathConverter<T: Path> {
    /// Convert an EssentialPath to a specific path type
    fn from_essential_path(&self, path: &EssentialPath) -> error::Result<T>;

    /// Convert a specific path type to an EssentialPath
    fn to_essential_path(&self, path: &T) -> error::Result<EssentialPath>;
}

/// EssentialPath represents a generic path that can be used across different file systems.
/// It contains a namespace and components, which are common to all path types.
#[derive(Debug, Clone)]
pub struct EssentialPath {
    namespace: DefaultNamespace,
    components: DefaultPathComponents,
}

impl EssentialPath {
    /// Create a new EssentialPath with empty namespace and components
    pub fn new() -> Self {
        EssentialPath {
            namespace: DefaultNamespace::from_string(""),
            components: DefaultPathComponents::new(),
        }
    }

    /// Parse a string into an EssentialPath
    pub fn from_string(path_str: &str) -> error::Result<Self> {
        Self::parse(path_str)
    }

    /// Convert this path to a specific path type using the provided converter
    pub fn to_specific_path<T: Path>(&self, converter: &dyn PathConverter<T>) -> error::Result<T> {
        converter.from_essential_path(self)
    }
}

impl Path for EssentialPath {
    type ComponentsType = DefaultPathComponents;
    type NamespaceType = DefaultNamespace;

    fn new() -> Self {
        EssentialPath {
            namespace: DefaultNamespace::from_string(""),
            components: DefaultPathComponents::new(),
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

    fn parse(path_str: &str) -> error::Result<Self> {
        // Handle special case for empty path
        if path_str.is_empty() {
            return Ok(Self::new());
        }

        // Special case for absolute paths without namespace
        if path_str.starts_with('/') {
            let path_without_slash = &path_str[1..];
            let mut components = DefaultPathComponents::from_string(path_without_slash);
            components.set_absolute(true);
            let mut path = Self::new();
            path.components = components;
            return Ok(path);
        }

        // Normal case: handle namespace and path parts
        let (namespace_str, path_part) = if path_str.contains(':') {
            let parts: Vec<&str> = path_str.splitn(2, ':').collect();
            (parts[0], parts[1])
        } else {
            ("", path_str)
        };

        let mut components = DefaultPathComponents::from_string(path_part);

        // If the path part starts with a slash, it's absolute
        if path_part.starts_with('/') {
            components.set_absolute(true);
        }

        Ok(EssentialPath {
            namespace: DefaultNamespace::from_string(namespace_str),
            components,
        })
    }

    fn to_string(&self) -> String {
        let ns = if self.namespace.is_empty() {
            String::new()
        } else {
            format!("{}:", self.namespace.as_str())
        };

        let path = self.components.join();

        if path.is_empty() {
            ns
        } else if path.starts_with('/') {
            format!("{}{}", ns, path)
        } else if ns.is_empty() {
            path
        } else {
            format!("{}:{}", ns, path)
        }
    }

    fn join(&self, relative: &str) -> error::Result<Self> {
        // Check if the relative path starts with a namespace (contains :)
        if relative.contains(':') {
            return Err(error::Error::msg(
                format!("Cannot join a path with a namespace")
            ));
        }

        // Special case for absolute path
        if relative.starts_with('/') {
            return Err(error::Error::msg(
                format!("Cannot join an absolute path")
            ));
        }

        // Create a new path with the same namespace
        let mut rel_path = EssentialPath::parse(relative)?;
        if !rel_path.namespace().is_empty() {
            return Err(error::Error::msg(
                format!("Cannot join a path with a namespace")
            ));
        }

        let mut result = self.clone();

        // Add each component from the relative path
        for i in 0..rel_path.components().len() {
            if let Some(component) = rel_path.components().get(i) {
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
        self.components.is_absolute()
    }

    fn to_local_path(&self) -> Option<Box<dyn LocalPath<ComponentsType=DefaultPathComponents, NamespaceType=DefaultNamespace>>> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
