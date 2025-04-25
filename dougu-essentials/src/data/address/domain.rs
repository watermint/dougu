use super::error::{AddressError, Result};
use crate::obj::notation::NotationType;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Domain {
    domain: String,
    parts: Vec<String>,
}

impl Domain {
    pub fn new<T: Into<String>>(domain: T) -> Result<Self> {
        let domain_str = domain.into();

        if !Self::is_valid(&domain_str) {
            return Err(AddressError::InvalidDomainFormat(domain_str));
        }

        // Split by dots to get domain parts
        let parts: Vec<String> = domain_str
            .split('.')
            .map(|s| s.to_string())
            .collect();

        Ok(Self {
            domain: domain_str,
            parts,
        })
    }

    pub fn domain(&self) -> &str {
        &self.domain
    }

    pub fn parts(&self) -> &[String] {
        &self.parts
    }

    pub fn tld(&self) -> Option<&str> {
        self.parts.last().map(|s| s.as_str())
    }

    pub fn is_valid(domain: &str) -> bool {
        // Basic domain validation rules
        if domain.is_empty() || domain.len() > 253 {
            return false;
        }

        // Split by dots
        let parts: Vec<&str> = domain.split('.').collect();

        // A domain must have at least two parts
        if parts.len() < 2 {
            return false;
        }

        // Each part must be valid
        for part in &parts {
            // Length check for each label
            if part.is_empty() || part.len() > 63 {
                return false;
            }

            // Character validation
            let first_char = part.chars().next().unwrap();
            let last_char = part.chars().last().unwrap();

            // First and last character must be alphanumeric
            if !first_char.is_alphanumeric() || !last_char.is_alphanumeric() {
                return false;
            }

            // Check that all characters are valid
            if !part.chars().all(|c| {
                c.is_alphanumeric() || c == '-'
            }) {
                return false;
            }
        }

        true
    }
}

impl fmt::Display for Domain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.domain)
    }
}

impl FromStr for Domain {
    type Err = AddressError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Domain::new(s)
    }
}

impl From<Domain> for NotationType {
    fn from(domain: Domain) -> Self {
        NotationType::String(domain.domain)
    }
}

impl TryFrom<NotationType> for Domain {
    type Error = AddressError;

    fn try_from(value: NotationType) -> std::result::Result<Self, Self::Error> {
        match value {
            NotationType::String(s) => Domain::new(s),
            _ => Err(AddressError::InvalidDomainFormat("Invalid notation type".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_domain() {
        assert!(Domain::is_valid("example.com"));
        assert!(Domain::is_valid("sub.example.com"));
        assert!(Domain::is_valid("sub-domain.example.com"));
        assert!(Domain::is_valid("example.co.uk"));
        assert!(Domain::is_valid("123example.com"));
    }

    #[test]
    fn test_invalid_domain() {
        assert!(!Domain::is_valid(""));
        assert!(!Domain::is_valid("example"));  // No TLD
        assert!(!Domain::is_valid(".com"));     // No domain name
        assert!(!Domain::is_valid("example..com")); // Double dot
        assert!(!Domain::is_valid("-example.com")); // Starts with hyphen
        assert!(!Domain::is_valid("example-.com")); // Ends with hyphen
        assert!(!Domain::is_valid("exam@ple.com")); // Invalid character
    }

    #[test]
    fn test_domain_parts() {
        let domain = Domain::new("sub.example.com").unwrap();
        assert_eq!(domain.parts(), &["sub", "example", "com"]);
        assert_eq!(domain.tld(), Some("com"));

        let domain2 = Domain::new("example.co.uk").unwrap();
        assert_eq!(domain2.parts(), &["example", "co", "uk"]);
        assert_eq!(domain2.tld(), Some("uk"));
    }

    #[test]
    fn test_serialization() {
        let domain = Domain::new("example.com").unwrap();
        let notation = NotationType::from(domain.clone());

        assert!(matches!(notation, NotationType::String(_)));

        if let NotationType::String(ref s) = notation {
            assert_eq!(s, "example.com");
        } else {
            panic!("Expected String notation type");
        }

        let domain_back = Domain::try_from(notation);
        assert!(domain_back.is_ok());
        assert_eq!(domain_back.unwrap().domain(), domain.domain());
    }
} 