use super::error::{AddressError, Result};
use crate::obj::notation::NotationType;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Urn {
    urn: String,
    namespace: String,
    specific_string: String,
    r#type: Option<String>,
}

impl Urn {
    pub fn new<T: Into<String>>(urn: T) -> Result<Self> {
        let urn_str = urn.into();

        if !Self::is_valid(&urn_str) {
            return Err(AddressError::InvalidUrnFormat(urn_str));
        }

        // Parse URN parts
        let (namespace, specific_string, r#type) = Self::parse_urn(&urn_str)?;

        Ok(Self {
            urn: urn_str,
            namespace,
            specific_string,
            r#type,
        })
    }

    pub fn from_parts<N: Into<String>, S: Into<String>, T: Into<Option<String>>>(
        namespace: N,
        specific_string: S,
        r#type: T,
    ) -> Result<Self> {
        let namespace = namespace.into();
        let specific_string = specific_string.into();
        let r#type = r#type.into();

        // Validate the namespace
        if !Self::is_valid_namespace(&namespace) {
            return Err(AddressError::InvalidUrnFormat(format!("Invalid namespace: {}", namespace)));
        }

        // Validate the specific string
        if specific_string.is_empty() {
            return Err(AddressError::InvalidUrnFormat("Specific string cannot be empty".to_string()));
        }

        // Build the full URN
        let urn = if let Some(type_val) = &r#type {
            format!("urn:{}:{}:{}", namespace, specific_string, type_val)
        } else {
            format!("urn:{}:{}", namespace, specific_string)
        };

        Ok(Self {
            urn,
            namespace,
            specific_string,
            r#type,
        })
    }

    pub fn urn(&self) -> &str {
        &self.urn
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn specific_string(&self) -> &str {
        &self.specific_string
    }

    pub fn r#type(&self) -> Option<&str> {
        self.r#type.as_deref()
    }

    pub fn is_valid(urn: &str) -> bool {
        // Basic URN validation (RFC 8141)
        if !urn.starts_with("urn:") {
            return false;
        }

        // Split the URN into parts: "urn:<nid>:<nss>[:<r-component>]"
        let parts: Vec<&str> = urn.split(':').collect();

        // Need at least 3 parts (urn, nid, nss)
        if parts.len() < 3 {
            return false;
        }

        let namespace = parts[1];

        // Validate the namespace identifier (nid)
        if !Self::is_valid_namespace(namespace) {
            return false;
        }

        // Validate the namespace specific string (nss)
        let specific_string = parts[2];
        if specific_string.is_empty() {
            return false;
        }

        true
    }

    fn is_valid_namespace(namespace: &str) -> bool {
        // Namespace identifier must be 2-32 characters long
        if namespace.is_empty() || namespace.len() > 32 {
            return false;
        }

        // Namespace must consist of alphanumeric characters only
        // The first character must be alphabetic
        if !namespace.chars().next().unwrap().is_ascii_alphabetic() {
            return false;
        }

        namespace.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    }

    fn parse_urn(urn: &str) -> Result<(String, String, Option<String>)> {
        let parts: Vec<&str> = urn.split(':').collect();

        // The first part must be "urn"
        if parts[0] != "urn" {
            return Err(AddressError::InvalidUrnFormat(urn.to_string()));
        }

        // Extract the namespace identifier (nid)
        let namespace = parts[1].to_string();

        // Extract the namespace specific string (nss)
        let specific_string = parts[2].to_string();

        // Extract the r-component if present
        let r#type = if parts.len() > 3 {
            Some(parts[3].to_string())
        } else {
            None
        };

        Ok((namespace, specific_string, r#type))
    }
}

impl fmt::Display for Urn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.urn)
    }
}

impl FromStr for Urn {
    type Err = AddressError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Urn::new(s)
    }
}

impl From<Urn> for NotationType {
    fn from(urn: Urn) -> Self {
        let mut obj = HashMap::new();

        obj.insert("urn".to_string(), NotationType::String(urn.urn));
        obj.insert("namespace".to_string(), NotationType::String(urn.namespace));
        obj.insert("specific_string".to_string(), NotationType::String(urn.specific_string));

        if let Some(type_val) = urn.r#type {
            obj.insert("type".to_string(), NotationType::String(type_val));
        }

        NotationType::Object(obj)
    }
}

impl TryFrom<NotationType> for Urn {
    type Error = AddressError;

    fn try_from(value: NotationType) -> std::result::Result<Self, Self::Error> {
        match value {
            NotationType::String(s) => Urn::new(s),
            NotationType::Object(obj) => {
                if let Some(NotationType::String(urn_str)) = obj.get("urn") {
                    Urn::new(urn_str)
                } else if let (Some(NotationType::String(namespace)), Some(NotationType::String(specific_string))) =
                    (obj.get("namespace"), obj.get("specific_string")) {
                    let type_val = obj.get("type").and_then(|t| match t {
                        NotationType::String(s) => Some(s.clone()),
                        _ => None,
                    });

                    Urn::from_parts(namespace, specific_string, type_val)
                } else {
                    Err(AddressError::InvalidUrnFormat("Missing required fields in object".to_string()))
                }
            }
            _ => Err(AddressError::InvalidUrnFormat("Invalid notation type".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urn() {
        assert!(Urn::is_valid("urn:isbn:0451450523"));
        assert!(Urn::is_valid("urn:uuid:f81d4fae-7dec-11d0-a765-00a0c91e6bf6"));
        assert!(Urn::is_valid("urn:example:a123,z456"));
        assert!(Urn::is_valid("urn:ietf:rfc:2648"));
        assert!(Urn::is_valid("urn:oid:2.16.840"));
        assert!(Urn::is_valid("urn:nbn:de:bvb:19-146642"));
    }

    #[test]
    fn test_invalid_urn() {
        assert!(!Urn::is_valid(""));
        assert!(!Urn::is_valid("isbn:0451450523")); // Missing "urn:" prefix
        assert!(!Urn::is_valid("urn:")); // Missing namespace
        assert!(!Urn::is_valid("urn:isbn:")); // Missing specific string
        assert!(!Urn::is_valid("urn:123:456789")); // Namespace starts with a number
        assert!(!Urn::is_valid("urn:abcdefghijklmnopqrstuvwxyz1234567:789")); // Namespace too long (33 chars)
    }

    #[test]
    fn test_urn_components() {
        let urn = Urn::new("urn:isbn:0451450523").unwrap();
        assert_eq!(urn.namespace(), "isbn");
        assert_eq!(urn.specific_string(), "0451450523");
        assert_eq!(urn.r#type(), None);

        let urn_with_type = Urn::new("urn:uuid:f81d4fae-7dec-11d0-a765-00a0c91e6bf6:variant").unwrap();
        assert_eq!(urn_with_type.namespace(), "uuid");
        assert_eq!(urn_with_type.specific_string(), "f81d4fae-7dec-11d0-a765-00a0c91e6bf6");
        assert_eq!(urn_with_type.r#type(), Some("variant"));
    }

    #[test]
    fn test_from_parts() {
        let urn = Urn::from_parts("isbn", "0451450523", None).unwrap();
        assert_eq!(urn.urn(), "urn:isbn:0451450523");

        let urn_with_type = Urn::from_parts("uuid", "f81d4fae-7dec-11d0-a765-00a0c91e6bf6", Some("variant")).unwrap();
        assert_eq!(urn_with_type.urn(), "urn:uuid:f81d4fae-7dec-11d0-a765-00a0c91e6bf6:variant");
    }

    #[test]
    fn test_serialization() {
        let urn = Urn::new("urn:isbn:0451450523").unwrap();
        let notation = NotationType::from(urn.clone());

        assert!(matches!(notation, NotationType::Object(_)));

        if let NotationType::Object(obj) = &notation {
            assert_eq!(obj.get("urn").and_then(|v| v.as_str()), Some("urn:isbn:0451450523"));
            assert_eq!(obj.get("namespace").and_then(|v| v.as_str()), Some("isbn"));
            assert_eq!(obj.get("specific_string").and_then(|v| v.as_str()), Some("0451450523"));
            assert!(obj.get("type").is_none());
        }

        let urn_back = Urn::try_from(notation);
        assert!(urn_back.is_ok());
        assert_eq!(urn_back.unwrap().urn(), urn.urn());
    }
} 