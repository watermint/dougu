use crate::data::uniqueid::error::{Error, Result};
use crate::data::uniqueid::types::{Uuid, UuidVersion};

/// Formatter for UUID string representations
#[derive(Debug, Clone, Copy)]
pub struct UuidFormatter;

impl UuidFormatter {
    /// Format UUID as a simple string (without hyphens)
    pub fn simple(uuid: &Uuid) -> String {
        uuid.to_simple()
    }
    
    /// Format UUID as a hyphenated string
    pub fn hyphenated(uuid: &Uuid) -> String {
        uuid.to_hyphenated()
    }
    
    /// Format UUID as uppercase
    pub fn uppercase(uuid: &Uuid) -> String {
        uuid.to_string().to_uppercase()
    }
    
    /// Format UUID as URN (Uniform Resource Name)
    pub fn urn(uuid: &Uuid) -> String {
        format!("urn:uuid:{}", uuid)
    }
    
    /// Format UUID with custom separator
    pub fn with_separator(uuid: &Uuid, separator: char) -> String {
        if separator == '-' {
            return uuid.to_string();
        }
        
        uuid.to_string().replace('-', &separator.to_string())
    }
    
    /// Format UUID as ULID if possible
    pub fn ulid(uuid: &Uuid) -> Result<String> {
        if uuid.version() != UuidVersion::Ulid {
            return Err(Error::InvalidFormat("Cannot format non-ULID UUID as ULID".to_string()));
        }
        
        let ulid_bytes = uuid.bytes();
        let ulid = ulid::Ulid::from_bytes(*ulid_bytes);
        
        Ok(ulid.to_string())
    }
    
    /// Try to format any UUID as ULID
    /// This will reinterpret the bytes as a ULID even if it wasn't created as one
    pub fn as_ulid(uuid: &Uuid) -> String {
        let ulid = ulid::Ulid::from_bytes(*uuid.bytes());
        ulid.to_string()
    }
} 