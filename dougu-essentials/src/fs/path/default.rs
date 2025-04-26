use super::core::{Namespace, PathComponents};

/// Default implementation of PathComponents that uses a vector of strings
#[derive(Debug, Clone)]
pub struct DefaultPathComponents {
    components: Vec<String>,
    delimiter: char,
    is_absolute: bool,
}

impl DefaultPathComponents {
    /// Create a new DefaultPathComponents with the specified delimiter
    pub fn with_delimiter(delimiter: char) -> Self {
        DefaultPathComponents {
            components: Vec::new(),
            delimiter,
            is_absolute: false,
        }
    }
    
    /// Join all components with the specified delimiter
    pub fn join_with_separator(&self, separator: &str) -> String {
        self.components.join(separator)
    }
    
    /// Set if this path is absolute
    pub fn set_absolute(&mut self, is_absolute: bool) {
        self.is_absolute = is_absolute;
    }
    
    /// Check if this path is absolute
    pub fn is_absolute(&self) -> bool {
        self.is_absolute
    }

    /// Get an iterator over all components
    pub fn all(&self) -> impl Iterator<Item = &str> {
        self.components.iter().map(|s| s.as_str())
    }
}

impl PathComponents for DefaultPathComponents {
    fn new() -> Self {
        // Default to '/' as the delimiter
        DefaultPathComponents {
            components: Vec::new(),
            delimiter: '/',
            is_absolute: false,
        }
    }

    fn len(&self) -> usize {
        self.components.len()
    }

    fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    fn get(&self, index: usize) -> Option<&str> {
        self.components.get(index).map(|s| s.as_str())
    }

    fn push(&mut self, component: &str) {
        self.components.push(component.to_string());
    }

    fn pop(&mut self) -> Option<String> {
        self.components.pop()
    }

    fn join(&self) -> String {
        self.components.join(&self.delimiter.to_string())
    }

    fn normalize(&mut self) {
        let mut normalized = Vec::new();
        
        for component in &self.components {
            match component.as_str() {
                "." => continue, // Skip "." components
                ".." => {
                    if !normalized.is_empty() && normalized.last().unwrap() != ".." {
                        normalized.pop(); // Go up one level
                    } else {
                        normalized.push(component.clone()); // Keep ".." if we're at the top
                    }
                }
                _ => normalized.push(component.clone()),
            }
        }
        
        self.components = normalized;
    }

    fn from_string(path: &str) -> Self {
        let delimiter = if path.contains('\\') { '\\' } else { '/' };
        
        let components = path.split(delimiter)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
            
        DefaultPathComponents {
            components,
            delimiter,
            is_absolute: false,
        }
    }
}

/// Default implementation of Namespace
#[derive(Debug, Clone)]
pub struct DefaultNamespace {
    value: String,
}

impl DefaultNamespace {
    /// Create a new DefaultNamespace with the specified value
    pub fn new(value: String) -> Self {
        DefaultNamespace { value }
    }
    
    /// Set the namespace value
    pub fn set(&mut self, value: &str) {
        self.value = value.to_string();
    }
}

impl Namespace for DefaultNamespace {
    fn as_str(&self) -> &str {
        &self.value
    }

    fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    fn from_string(s: &str) -> Self {
        DefaultNamespace {
            value: s.to_string(),
        }
    }
} 