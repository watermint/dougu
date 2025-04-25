use crate::obj::notation::NotationType;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use super::error::{AddressError, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Url {
    url: String,
    scheme: String,
    host: String,
    path: String,
    query: Option<String>,
    fragment: Option<String>,
}

impl Url {
    pub fn new<T: Into<String>>(url: T) -> Result<Self> {
        let url_str = url.into();
        
        if !Self::is_valid(&url_str) {
            return Err(AddressError::InvalidUrlFormat(url_str));
        }
        
        // Parse URL components
        let (scheme, rest) = match url_str.split_once("://") {
            Some((s, r)) => (s.to_string(), r.to_string()),
            None => return Err(AddressError::InvalidUrlFormat(url_str)),
        };
        
        // Extract host and path
        let (host_part, path_part) = match rest.find('/') {
            Some(idx) => (&rest[..idx], &rest[idx..]),
            None => (rest.as_str(), "/"),
        };
        
        // Extract query and fragment
        let (path, fragment) = match path_part.split_once('#') {
            Some((p, f)) => (p, Some(f.to_string())),
            None => (path_part, None),
        };
        
        let (path, query) = match path.split_once('?') {
            Some((p, q)) => (p, Some(q.to_string())),
            None => (path, None),
        };
        
        Ok(Self {
            url: url_str,
            scheme,
            host: host_part.to_string(),
            path: path.to_string(),
            query,
            fragment,
        })
    }
    
    pub fn url(&self) -> &str {
        &self.url
    }
    
    pub fn scheme(&self) -> &str {
        &self.scheme
    }
    
    pub fn host(&self) -> &str {
        &self.host
    }
    
    pub fn path(&self) -> &str {
        &self.path
    }
    
    pub fn query(&self) -> Option<&str> {
        self.query.as_deref()
    }
    
    pub fn fragment(&self) -> Option<&str> {
        self.fragment.as_deref()
    }
    
    pub fn is_valid(url: &str) -> bool {
        // Basic URL validation
        if url.is_empty() {
            return false;
        }
        
        if !url.contains("://") {
            return false;
        }
        
        let parts: Vec<&str> = url.split("://").collect();
        if parts.len() != 2 {
            return false;
        }
        
        let scheme = parts[0];
        let rest = parts[1];
        
        // Validate scheme
        if scheme.is_empty() || !scheme.chars().all(|c| c.is_alphanumeric() || c == '+' || c == '.' || c == '-') {
            return false;
        }
        
        // Validate host
        if rest.is_empty() {
            return false;
        }
        
        let host_part = match rest.find('/') {
            Some(idx) => &rest[..idx],
            None => rest,
        };
        
        if host_part.is_empty() {
            return false;
        }
        
        true
    }
}

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.url)
    }
}

impl FromStr for Url {
    type Err = AddressError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Url::new(s)
    }
}

impl From<Url> for NotationType {
    fn from(url: Url) -> Self {
        let mut obj = HashMap::new();
        obj.insert("url".to_string(), NotationType::String(url.url));
        obj.insert("scheme".to_string(), NotationType::String(url.scheme));
        obj.insert("host".to_string(), NotationType::String(url.host));
        obj.insert("path".to_string(), NotationType::String(url.path));
        
        if let Some(query) = url.query {
            obj.insert("query".to_string(), NotationType::String(query));
        }
        
        if let Some(fragment) = url.fragment {
            obj.insert("fragment".to_string(), NotationType::String(fragment));
        }
        
        NotationType::Object(obj)
    }
}

impl TryFrom<NotationType> for Url {
    type Error = AddressError;

    fn try_from(value: NotationType) -> std::result::Result<Self, Self::Error> {
        match value {
            NotationType::String(s) => Url::new(s),
            NotationType::Object(obj) => {
                // For complex objects, we expect the full URL to be in the "url" field
                if let Some(NotationType::String(url)) = obj.get("url") {
                    Url::new(url)
                } else {
                    Err(AddressError::InvalidUrlFormat("Missing url field in object".to_string()))
                }
            },
            _ => Err(AddressError::InvalidUrlFormat("Invalid notation type".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_url() {
        assert!(Url::is_valid("https://example.com"));
        assert!(Url::is_valid("https://example.com/path"));
        assert!(Url::is_valid("https://example.com/path?query=value"));
        assert!(Url::is_valid("https://example.com/path?query=value#fragment"));
        assert!(Url::is_valid("ftp://example.com"));
    }

    #[test]
    fn test_invalid_url() {
        assert!(!Url::is_valid(""));
        assert!(!Url::is_valid("example.com"));
        assert!(!Url::is_valid("https://"));
        assert!(!Url::is_valid("://example.com"));
    }

    #[test]
    fn test_url_components() {
        let url = Url::new("https://example.com/path?query=value#fragment").unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host(), "example.com");
        assert_eq!(url.path(), "/path");
        assert_eq!(url.query(), Some("query=value"));
        assert_eq!(url.fragment(), Some("fragment"));
    }

    #[test]
    fn test_serialization() {
        let url = Url::new("https://example.com/path?query=value#fragment").unwrap();
        let notation = NotationType::from(url.clone());
        
        assert!(matches!(notation, NotationType::Object(_)));
        
        if let NotationType::Object(obj) = &notation {
            assert_eq!(obj.get("url").and_then(|v| v.as_str()), Some("https://example.com/path?query=value#fragment"));
            assert_eq!(obj.get("scheme").and_then(|v| v.as_str()), Some("https"));
            assert_eq!(obj.get("host").and_then(|v| v.as_str()), Some("example.com"));
        }
        
        let url_back = Url::try_from(notation);
        assert!(url_back.is_ok());
        assert_eq!(url_back.unwrap().url(), url.url());
    }
} 