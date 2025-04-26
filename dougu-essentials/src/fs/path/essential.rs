// Essential Path implementation
use std::any::Any;
use crate::core::error::{Error, Result};
use super::core::Path;
use super::default::{DefaultNamespace, DefaultPathComponents};
use super::local::LocalPath;

/// EssentialPath is the central path abstraction that can be converted to and from other path types.
/// It serves as the common format for path representation across different backends.
#[derive(Debug, Clone)]
pub struct EssentialPath {
    namespace: DefaultNamespace,
    components: DefaultPathComponents,
    is_absolute: bool,
}

/// PathConverter provides conversion between EssentialPath and specific path types
pub trait PathConverter<T: Path> {
    /// Convert an EssentialPath to a specific path type
    fn from_essential_path(&self, path: &EssentialPath) -> Result<T>;
    
    /// Convert a specific path type to an EssentialPath
    fn to_essential_path(&self, path: &T) -> Result<EssentialPath>;
}

impl EssentialPath {
    /// Create a new empty EssentialPath
    pub fn new() -> Self {
        EssentialPath {
            namespace: DefaultNamespace::from_string(""),
            components: DefaultPathComponents::new(),
            is_absolute: false,
        }
    }
    
    /// Create an EssentialPath from a string representation
    pub fn from_string(path_str: &str) -> Result<Self> {
        Self::parse(path_str)
    }
    
    /// Convert this EssentialPath to a specific path type using a resolver
    pub fn to_specific_path<T: Path>(&self, converter: &dyn PathConverter<T>) -> Result<T> {
        converter.from_essential_path(self)
    }
}

impl Path for EssentialPath {
    type ComponentsType = DefaultPathComponents;
    type NamespaceType = DefaultNamespace;
    
    fn new() -> Self {
        EssentialPath::new()
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
    
    fn parse(path_str: &str) -> Result<Self> {
        // Handle empty paths
        if path_str.is_empty() {
            return Ok(Self::new());
        }
        
        let is_absolute = path_str.starts_with('/') || path_str.starts_with('\\');
        
        // Parse namespace and path components
        let (namespace_str, path_part) = if path_str.contains(':') {
            let parts: Vec<&str> = path_str.splitn(2, ':').collect();
            (parts[0], parts.get(1).unwrap_or(&""))
        } else {
            ("", path_str)
        };
        
        // Create the path components
        let mut components = if is_absolute {
            // Remove leading slash for absolute paths when parsing components
            let path_without_slash = if path_part.starts_with('/') || path_part.starts_with('\\') {
                &path_part[1..]
            } else {
                path_part
            };
            DefaultPathComponents::from_string(path_without_slash)
        } else {
            DefaultPathComponents::from_string(path_part)
        };
        
        // Normalize the components
        components.normalize();
        
        Ok(EssentialPath {
            namespace: DefaultNamespace::from_string(namespace_str),
            components,
            is_absolute,
        })
    }
    
    fn to_string(&self) -> String {
        let ns = if self.namespace.is_empty() {
            String::new()
        } else {
            format!("{}:", self.namespace.as_str())
        };
        
        let path = self.components.join();
        
        if self.is_absolute {
            if path.is_empty() {
                format!("{}/", ns)
            } else {
                format!("{}/{}", ns, path)
            }
        } else {
            format!("{}{}", ns, path)
        }
    }
    
    fn join(&self, relative: &str) -> Result<Self> {
        // Cannot join an absolute path to another path
        if relative.starts_with('/') || relative.starts_with('\\') {
            return Err(Error::InvalidArgument(
                "Cannot join an absolute path".to_string()
            ));
        }
        
        let rel_path = Self::parse(relative)?;
        
        // If rel_path has a namespace, it's a different path type
        if !rel_path.namespace().is_empty() {
            return Err(Error::InvalidArgument(
                "Cannot join paths with different namespaces".to_string()
            ));
        }
        
        let mut result = self.clone();
        
        // Add components from the relative path
        for i in 0..rel_path.components().len() {
            if let Some(component) = rel_path.components().get(i) {
                result.components_mut().push(component);
            }
        }
        
        // Normalize the result
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
        self.is_absolute
    }
    
    fn to_local_path(&self) -> Option<Box<dyn LocalPath>> {
        // This would be implemented using the resolver repository
        // For now, return None as we haven't implemented local paths yet
        None
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}
