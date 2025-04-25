use crate::data::uniqueid::error::{Error, Result};
use crate::data::uniqueid::types::{UniqueId, IdVariant, IdVersion, IdType};
use std::str::FromStr;
use uuid::Uuid as RawUuid;

/// Parser for handling different unique identifier formats
#[derive(Debug, Clone, Copy)]
pub struct IdParser;

impl IdParser {
    /// Parse a string into the UniqueId type
    /// 
    /// Supports:
    /// - Standard UUID format (with or without hyphens)
    /// - ULID format (26 characters, base32)
    pub fn parse(s: &str) -> Result<UniqueId> {
        // Handle ULID format (26 characters, base32)
        if s.len() == 26 && s.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Self::parse_ulid(s);
        }

        // Handle standard UUID format
        if s.len() == 32 || s.len() == 36 {
            return UniqueId::from_str(s);
        }

        Err(Error::InvalidFormat(format!("Unrecognized identifier format: {}", s)))
    }

    /// Parse a UUID from a simple string (without hyphens)
    pub fn parse_simple(s: &str) -> Result<UniqueId> {
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

        Self::parse_uuid(&formatted)
    }

    /// Parse a hyphenated UUID string
    pub fn parse_hyphenated(s: &str) -> Result<UniqueId> {
        if s.len() != 36 {
            return Err(Error::InvalidFormat(
                "Hyphenated UUID must be 36 characters long".to_string(),
            ));
        }

        Self::parse_uuid(s)
    }

    /// Parse a UUID string into a UniqueId
    fn parse_uuid(s: &str) -> Result<UniqueId> {
        match RawUuid::parse_str(s) {
            Ok(raw) => {
                // Determine version
                let version = match raw.get_version_num() {
                    1 => IdVersion::V1,
                    2 => IdVersion::V2,
                    3 => IdVersion::V3,
                    4 => IdVersion::V4,
                    5 => IdVersion::V5,
                    6 => IdVersion::V6,
                    7 => IdVersion::V7,
                    8 => IdVersion::V8,
                    _ => IdVersion::Unknown,
                };
                
                // Determine variant
                let variant = match raw.get_variant() {
                    uuid::Variant::RFC4122 => IdVariant::RFC4122,
                    uuid::Variant::NCS => IdVariant::NCS,
                    uuid::Variant::Microsoft => IdVariant::Microsoft,
                    uuid::Variant::Future => IdVariant::Future,
                    _ => IdVariant::Unknown,
                };
                
                Ok(UniqueId::new(raw.into_bytes(), IdType::Uuid(version, variant)))
            },
            Err(e) => Err(Error::UuidParse(e)),
        }
    }

    /// Parse a ULID string into a UniqueId
    fn parse_ulid(s: &str) -> Result<UniqueId> {
        if s.len() != 26 || !s.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(Error::InvalidFormat("Invalid ULID format".to_string()));
        }
        
        // Parse the ULID using the ulid crate
        match ulid::Ulid::from_string(s) {
            Ok(ulid) => {
                let bytes = ulid.to_bytes();
                Ok(UniqueId::new(bytes, IdType::Ulid))
            }
            Err(e) => Err(Error::UlidParse(format!("Invalid ULID: {}", e))),
        }
    }
} 