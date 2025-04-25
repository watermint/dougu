use super::error::{AddressError, Result};
use crate::obj::notation::NotationType;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Uri {
    uri: String,
    scheme: Option<String>,
    authority: Option<String>,
    path: String,
    query: Option<String>,
    fragment: Option<String>,
}

impl Uri {
    pub fn new<T: Into<String>>(uri: T) -> Result<Self> {
        let uri_str = uri.into();

        if !Self::is_valid(&uri_str) {
            return Err(AddressError::InvalidUriFormat(uri_str));
        }

        // Extract scheme if present
        let (scheme, rest) = match uri_str.split_once(':') {
            Some((s, r)) => {
                if s.chars().all(|c| c.is_alphanumeric() || c == '+' || c == '.' || c == '-') {
                    (Some(s.to_string()), r.to_string())
                } else {
                    (None, uri_str.clone())
                }
            }
            None => (None, uri_str.clone()),
        };

        // Extract authority if present
        let (authority, path_part) = if let Some(_) = &scheme {
            if rest.starts_with("//") {
                match rest[2..].find('/') {
                    Some(idx) => (Some(rest[2..2 + idx].to_string()), rest[2 + idx..].to_string()),
                    None => {
                        if !rest[2..].is_empty() {
                            (Some(rest[2..].to_string()), "/".to_string())
                        } else {
                            (None, rest[2..].to_string())
                        }
                    }
                }
            } else {
                (None, rest)
            }
        } else {
            (None, rest)
        };

        // Extract path, query, and fragment
        let (path, fragment) = match path_part.split_once('#') {
            Some((p, f)) => (p, Some(f.to_string())),
            None => (path_part.as_str(), None),
        };

        let (path, query) = match path.split_once('?') {
            Some((p, q)) => (p, Some(q.to_string())),
            None => (path, None),
        };

        // For absolute URIs without a scheme-specific part, the path must start with a /
        let path = if path.is_empty() && scheme.is_some() && authority.is_none() {
            "/".to_string()
        } else {
            path.to_string()
        };

        Ok(Self {
            uri: uri_str,
            scheme,
            authority,
            path,
            query,
            fragment,
        })
    }

    pub fn uri(&self) -> &str {
        &self.uri
    }

    pub fn scheme(&self) -> Option<&str> {
        self.scheme.as_deref()
    }

    pub fn authority(&self) -> Option<&str> {
        self.authority.as_deref()
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

    pub fn is_valid(uri: &str) -> bool {
        // Basic URI validation - more permissive than URL validation
        if uri.is_empty() {
            return false;
        }

        // RFC 3986 allows URIs to be a path-reference, absolute-path, or complete URI
        // For simplicity, we'll allow any non-empty string that doesn't have invalid characters

        // Check for invalid characters
        if uri.contains(' ') || uri.contains('\n') || uri.contains('\r') || uri.contains('\t') {
            return false;
        }

        true
    }
}

impl fmt::Display for Uri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.uri)
    }
}

impl FromStr for Uri {
    type Err = AddressError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Uri::new(s)
    }
}

impl From<Uri> for NotationType {
    fn from(uri: Uri) -> Self {
        let mut obj = HashMap::new();
        obj.insert("uri".to_string(), NotationType::String(uri.uri));

        if let Some(scheme) = uri.scheme {
            obj.insert("scheme".to_string(), NotationType::String(scheme));
        }

        if let Some(authority) = uri.authority {
            obj.insert("authority".to_string(), NotationType::String(authority));
        }

        obj.insert("path".to_string(), NotationType::String(uri.path));

        if let Some(query) = uri.query {
            obj.insert("query".to_string(), NotationType::String(query));
        }

        if let Some(fragment) = uri.fragment {
            obj.insert("fragment".to_string(), NotationType::String(fragment));
        }

        NotationType::Object(obj)
    }
}

impl TryFrom<NotationType> for Uri {
    type Error = AddressError;

    fn try_from(value: NotationType) -> std::result::Result<Self, Self::Error> {
        match value {
            NotationType::String(s) => Uri::new(s),
            NotationType::Object(obj) => {
                // For complex objects, we expect the full URI to be in the "uri" field
                if let Some(NotationType::String(uri)) = obj.get("uri") {
                    Uri::new(uri)
                } else {
                    Err(AddressError::InvalidUriFormat("Missing uri field in object".to_string()))
                }
            }
            _ => Err(AddressError::InvalidUriFormat("Invalid notation type".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_uri() {
        assert!(Uri::is_valid("https://example.com"));
        assert!(Uri::is_valid("/path/to/resource"));
        assert!(Uri::is_valid("path/to/resource"));
        assert!(Uri::is_valid("mailto:user@example.com"));
        assert!(Uri::is_valid("tel:+1-234-567-8901"));
        assert!(Uri::is_valid("urn:isbn:0451450523"));
    }

    #[test]
    fn test_invalid_uri() {
        assert!(!Uri::is_valid(""));
        assert!(!Uri::is_valid("https://example.com with spaces"));
    }

    #[test]
    fn test_uri_components() {
        let uri = Uri::new("https://example.com/path?query=value#fragment").unwrap();
        assert_eq!(uri.scheme(), Some("https"));
        assert_eq!(uri.authority(), Some("example.com"));
        assert_eq!(uri.path(), "/path");
        assert_eq!(uri.query(), Some("query=value"));
        assert_eq!(uri.fragment(), Some("fragment"));

        let path_uri = Uri::new("/path/to/resource").unwrap();
        assert_eq!(path_uri.scheme(), None);
        assert_eq!(path_uri.authority(), None);
        assert_eq!(path_uri.path(), "/path/to/resource");
        assert_eq!(path_uri.query(), None);
        assert_eq!(path_uri.fragment(), None);
    }

    #[test]
    fn test_serialization() {
        let uri = Uri::new("https://example.com/path?query=value#fragment").unwrap();
        let notation = NotationType::from(uri.clone());

        assert!(matches!(notation, NotationType::Object(_)));

        if let NotationType::Object(obj) = &notation {
            assert_eq!(obj.get("uri").and_then(|v| v.as_str()), Some("https://example.com/path?query=value#fragment"));
            assert_eq!(obj.get("scheme").and_then(|v| v.as_str()), Some("https"));
            assert_eq!(obj.get("authority").and_then(|v| v.as_str()), Some("example.com"));
        }

        let uri_back = Uri::try_from(notation);
        assert!(uri_back.is_ok());
        assert_eq!(uri_back.unwrap().uri(), uri.uri());
    }
} 