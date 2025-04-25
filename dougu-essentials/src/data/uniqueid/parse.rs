use crate::data::uniqueid::error::{Error, Result};
use crate::data::uniqueid::types::{Uuid, UuidVariant, UuidVersion};
use std::str::FromStr;
use uuid::Uuid as RawUuid;

/// Parser for handling different UUID formats
#[derive(Debug, Clone, Copy)]
pub struct UuidParser;

impl UuidParser {
    /// Parse a UUID string into the Uuid type
    /// 
    /// Supports:
    /// - Standard UUID format (with or without hyphens)
    /// - ULID format (26 characters, base32)
    pub fn parse(s: &str) -> Result<Uuid> {
        // Handle standard UUID format
        if s.len() == 32 || s.len() == 36 {
            return Uuid::from_str(s);
        }

        // Handle ULID format (26 characters, base32)
        if s.len() == 26 && s.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Self::parse_ulid(s);
        }

        Err(Error::InvalidFormat(format!("Unrecognized UUID format: {}", s)))
    }

    /// Parse a UUID from a simple string (without hyphens)
    pub fn parse_simple(s: &str) -> Result<Uuid> {
        if s.len() != 32 {
            return Err(Error::InvalidFormat(
                "Simple UUID must be 32 characters long".to_string(),
            ));
        }

        // Insert hyphens to create standard format
        let formatted = format!(
            "{}-{}-{}-{}-{}",
            &s[0..8],
            &s[8..12],
            &s[12..16],
            &s[16..20],
            &s[20..32]
        );

        Uuid::from_str(&formatted)
    }

    /// Parse a hyphenated UUID string
    pub fn parse_hyphenated(s: &str) -> Result<Uuid> {
        if s.len() != 36 {
            return Err(Error::InvalidFormat(
                "Hyphenated UUID must be 36 characters long".to_string(),
            ));
        }

        Uuid::from_str(s)
    }

    /// Parse a ULID string into a UUID
    fn parse_ulid(s: &str) -> Result<Uuid> {
        if s.len() != 26 || !s.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(Error::InvalidFormat("Invalid ULID format".to_string()));
        }
        
        // Parse the ULID using the ulid crate
        match ulid::Ulid::from_string(s) {
            Ok(ulid) => {
                let bytes = ulid.to_bytes();
                Ok(Uuid::new(bytes, UuidVersion::Ulid, UuidVariant::RFC4122))
            }
            Err(_) => Err(Error::InvalidFormat(format!("Invalid ULID: {}", s))),
        }
    }
} 