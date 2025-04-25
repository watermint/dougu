// URL-like path implementation for cloud storage

use crate::core::Result;
use crate::core::error::Error;
use super::component::Component;
use super::traits::{PathFormat, PathTrait};
use std::fmt::Display;
use std::path::{Path as StdPath, PathBuf as StdPathBuf};

/// URL-like path for cloud storage
#[derive(Clone, Debug)]
pub struct UrlPath {
    /// Service namespace (e.g., "dropbox", "gdrive")
    namespace: String,
    /// Path components
    components: Vec<Component>,
    /// Whether this path is absolute
    absolute: bool,
}

impl UrlPath {
    /// Create a new URL path
    pub fn new(namespace: &str, path: &str) -> Self {
        let absolute = path.starts_with('/');
        let mut components = Vec::new();
        
        if absolute {
            components.push(Component::root());
        }
        
        for part in path.split('/') {
            if part.is_empty() {
                continue;
            }
            
            if part == "." {
                components.push(Component::current());
            } else if part == ".." {
                components.push(Component::parent());
            } else {
                components.push(Component::new(part));
            }
        }
        
        Self {
            namespace: namespace.to_string(),
            components,
            absolute,
        }
    }
    
    /// Get the namespace
    pub fn namespace(&self) -> &str {
        &self.namespace
    }
    
    /// Create a root path for the namespace
    pub fn root(namespace: &str) -> Self {
        Self {
            namespace: namespace.to_string(),
            components: vec![Component::root()],
            absolute: true,
        }
    }
    
    /// Create an empty path for the namespace
    pub fn empty(namespace: &str) -> Self {
        Self {
            namespace: namespace.to_string(),
            components: Vec::new(),
            absolute: false,
        }
    }
    
    /// Parse a URL-like path string (e.g., "dropbox:/path/to/file")
    pub fn parse(url_path: &str) -> Result<Self> {
        if let Some(idx) = url_path.find(':') {
            let namespace = &url_path[0..idx];
            let path = &url_path[idx+1..];
            Ok(Self::new(namespace, path))
        } else {
            Err(Error::from_string(format!("Invalid URL path: {}", url_path)))
        }
    }
}

impl PathTrait for UrlPath {
    fn format(&self) -> PathFormat {
        PathFormat::UrlLike
    }
    
    fn is_absolute(&self) -> bool {
        self.absolute
    }
    
    fn to_string(&self) -> String {
        let mut path = String::new();
        path.push_str(&self.namespace);
        path.push(':');
        
        if self.absolute {
            path.push('/');
        }
        
        let mut first = true;
        for component in &self.components {
            if component.is_root {
                continue;
            }
            
            if !first {
                path.push('/');
            }
            
            path.push_str(&component.name);
            first = false;
        }
        
        // Handle empty path case (just namespace:)
        if self.components.is_empty() || (self.components.len() == 1 && self.components[0].is_root) {
            // Keep as namespace: or namespace:/
        }
        
        path
    }
    
    fn file_name(&self) -> Option<String> {
        if self.components.is_empty() {
            None
        } else {
            let last = self.components.last().unwrap();
            if last.is_root || last.is_current || last.is_parent {
                None
            } else {
                Some(last.name.clone())
            }
        }
    }
    
    fn parent(&self) -> Option<Self> {
        if self.components.is_empty() {
            return None;
        }
        
        let mut parent = self.clone();
        if parent.components.len() == 1 && parent.components[0].is_root {
            // Root has no parent
            return None;
        }
        
        parent.components.pop();
        Some(parent)
    }
    
    fn has_root(&self) -> bool {
        self.absolute
    }
    
    fn is_empty(&self) -> bool {
        self.components.is_empty()
    }
    
    fn join<P: AsRef<Self>>(&self, path: P) -> Self {
        let path = path.as_ref();
        
        // If the path to join is absolute, return it
        if path.is_absolute() {
            return path.clone();
        }
        
        let mut result = self.clone();
        
        for component in &path.components {
            if component.is_root {
                continue;
            }
            
            result.components.push(component.clone());
        }
        
        result
    }
    
    fn to_std_path(&self) -> Option<StdPathBuf> {
        // URL paths can't be directly converted to standard paths
        None
    }
    
    fn components(&self) -> Vec<Component> {
        self.components.clone()
    }
    
    fn starts_with<P: AsRef<Self>>(&self, base: P) -> bool {
        let base = base.as_ref();
        
        if self.namespace != base.namespace {
            return false;
        }
        
        if self.components.len() < base.components.len() {
            return false;
        }
        
        for (i, component) in base.components.iter().enumerate() {
            if &self.components[i] != component {
                return false;
            }
        }
        
        true
    }
    
    fn ends_with<P: AsRef<Self>>(&self, child: P) -> bool {
        let child = child.as_ref();
        
        if self.namespace != child.namespace {
            return false;
        }
        
        if self.components.len() < child.components.len() {
            return false;
        }
        
        let offset = self.components.len() - child.components.len();
        
        for (i, component) in child.components.iter().enumerate() {
            if &self.components[i + offset] != component {
                return false;
            }
        }
        
        true
    }
    
    fn normalize(&self) -> Self {
        let mut result = Self {
            namespace: self.namespace.clone(),
            components: Vec::new(),
            absolute: self.absolute,
        };
        
        for component in &self.components {
            if component.is_current {
                // Skip . components
                continue;
            } else if component.is_parent {
                // Handle .. by removing the last non-root component
                if !result.components.is_empty() && !result.components.last().unwrap().is_root {
                    result.components.pop();
                }
            } else {
                result.components.push(component.clone());
            }
        }
        
        result
    }
    
    fn extension(&self) -> Option<String> {
        self.file_name().and_then(|name| {
            let parts: Vec<&str> = name.rsplitn(2, '.').collect();
            if parts.len() == 2 {
                Some(parts[0].to_string())
            } else {
                None
            }
        })
    }
    
    fn file_stem(&self) -> Option<String> {
        self.file_name().and_then(|name| {
            let parts: Vec<&str> = name.rsplitn(2, '.').collect();
            if parts.len() == 2 {
                Some(parts[1].to_string())
            } else {
                Some(name)
            }
        })
    }
    
    fn pop(&mut self) -> bool {
        if self.components.is_empty() {
            return false;
        }
        
        // Don't pop root
        if self.components.len() == 1 && self.components[0].is_root {
            return false;
        }
        
        self.components.pop();
        true
    }
    
    fn push<P: AsRef<Self>>(&mut self, path: P) {
        let path = path.as_ref();
        
        // If path has root, it replaces everything
        if path.has_root() {
            *self = path.clone();
            return;
        }
        
        for component in &path.components {
            if component.is_root {
                continue;
            }
            
            self.components.push(component.clone());
        }
    }
    
    fn with_extension(&self, extension: &str) -> Self {
        let mut result = self.clone();
        
        if let Some(last_idx) = result.components.len().checked_sub(1) {
            let last = &result.components[last_idx];
            if !last.is_root && !last.is_current && !last.is_parent {
                let name = last.name.clone();
                let stem = if let Some(dot_pos) = name.rfind('.') {
                    name[..dot_pos].to_string()
                } else {
                    name
                };
                
                result.components[last_idx] = Component::new(&format!("{}.{}", stem, extension));
            }
        }
        
        result
    }
    
    fn with_file_name(&self, file_name: &str) -> Self {
        let mut result = self.clone();
        
        if let Some(last_idx) = result.components.len().checked_sub(1) {
            let last = &result.components[last_idx];
            if !last.is_root && !last.is_current && !last.is_parent {
                result.components[last_idx] = Component::new(file_name);
            }
        }
        
        result
    }
}

impl AsRef<UrlPath> for UrlPath {
    fn as_ref(&self) -> &UrlPath {
        self
    }
}

impl Display for UrlPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
} 