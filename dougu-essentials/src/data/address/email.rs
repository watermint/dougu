use super::error::{AddressError, Result};
use crate::obj::notation::NotationType;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Email {
    address: String,
}

impl Email {
    pub fn new<T: Into<String>>(address: T) -> Result<Self> {
        let address = address.into();
        if Self::is_valid(&address) {
            Ok(Self { address })
        } else {
            Err(AddressError::InvalidEmailFormat(address))
        }
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn is_valid(address: &str) -> bool {
        // Simple regex check for valid email format
        let parts: Vec<&str> = address.split('@').collect();

        if parts.len() != 2 {
            return false;
        }

        let local_part = parts[0];
        let domain_part = parts[1];

        // Check if local part and domain part are not empty
        if local_part.is_empty() || domain_part.is_empty() {
            return false;
        }

        // Check if domain has at least one dot and has valid domain parts
        if !domain_part.contains('.') {
            return false;
        }

        // Check that domain doesn't start or end with a dot
        if domain_part.starts_with('.') || domain_part.ends_with('.') {
            return false;
        }

        // Check for consecutive dots in domain
        if domain_part.contains("..") {
            return false;
        }

        // Basic character check
        if local_part.chars().any(|c| !c.is_alphanumeric() && !".!#$%&'*+-/=?^_`{|}~".contains(c)) {
            return false;
        }

        if domain_part.chars().any(|c| !c.is_alphanumeric() && c != '.' && c != '-') {
            return false;
        }

        true
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.address)
    }
}

impl FromStr for Email {
    type Err = AddressError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Email::new(s)
    }
}

impl From<Email> for NotationType {
    fn from(email: Email) -> Self {
        NotationType::String(email.address)
    }
}

impl TryFrom<NotationType> for Email {
    type Error = AddressError;

    fn try_from(value: NotationType) -> std::result::Result<Self, Self::Error> {
        match value {
            NotationType::String(s) => Email::new(s),
            _ => Err(AddressError::InvalidEmailFormat("Invalid notation type".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        assert!(Email::is_valid("user@example.com"));
        assert!(Email::is_valid("user.name@example.com"));
        assert!(Email::is_valid("user+tag@example.com"));
        assert!(Email::is_valid("user_name@example.co.uk"));
    }

    #[test]
    fn test_invalid_email() {
        assert!(!Email::is_valid(""));
        assert!(!Email::is_valid("user"));
        assert!(!Email::is_valid("user@"));
        assert!(!Email::is_valid("@example.com"));
        assert!(!Email::is_valid("user@example"));
        assert!(!Email::is_valid("user@.com"));
    }

    #[test]
    fn test_new_valid_email() {
        let email = Email::new("user@example.com");
        assert!(email.is_ok());
        assert_eq!(email.unwrap().address(), "user@example.com");
    }

    #[test]
    fn test_new_invalid_email() {
        let email = Email::new("not-an-email");
        assert!(email.is_err());
    }

    #[test]
    fn test_serialization() {
        let email = Email::new("user@example.com").unwrap();
        let notation = NotationType::from(email.clone());

        if let NotationType::String(ref s) = notation {
            assert_eq!(s, "user@example.com");
        } else {
            panic!("Expected String notation type");
        }

        let email_back = Email::try_from(notation);
        assert!(email_back.is_ok());
        assert_eq!(email_back.unwrap().address(), email.address());
    }
} 