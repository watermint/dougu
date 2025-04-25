use crate::data::uniqueid::error::{Error, Result};
use crate::data::uniqueid::types::{UniqueId, IdType};

/// Formatter for unique identifier string representations
#[derive(Debug, Clone, Copy)]
pub struct IdFormatter;

impl IdFormatter {
    /// Format identifier as a simple string (without hyphens for UUID, base32 for ULID)
    pub fn simple(id: &UniqueId) -> String {
        id.to_simple()
    }
    
    /// Format UUID as a hyphenated string (or ULID in standard format)
    pub fn hyphenated(id: &UniqueId) -> String {
        id.to_hyphenated()
    }
    
    /// Format identifier as uppercase
    pub fn uppercase(id: &UniqueId) -> String {
        id.to_string().to_uppercase()
    }
    
    /// Format UUID as URN (Uniform Resource Name)
    /// Only valid for UUID types
    pub fn urn(id: &UniqueId) -> Result<String> {
        if !id.is_uuid() {
            return Err(Error::UnsupportedOperation("URN format is only valid for UUID types".to_string()));
        }
        
        Ok(format!("urn:uuid:{}", id))
    }
    
    /// Format UUID with custom separator
    /// Only valid for UUID types
    pub fn with_separator(id: &UniqueId, separator: char) -> Result<String> {
        if !id.is_uuid() {
            return Err(Error::UnsupportedOperation("Custom separator is only valid for UUID types".to_string()));
        }
        
        if separator == '-' {
            return Ok(id.to_hyphenated());
        }
        
        Ok(id.to_hyphenated().replace('-', &separator.to_string()))
    }
    
    /// Format as ULID string
    /// Returns the original ULID if the ID is already a ULID,
    /// or tries to interpret the UUID bytes as a ULID
    pub fn as_ulid(id: &UniqueId) -> String {
        match id.to_ulid_string() {
            Some(s) => s,
            None => {
                let ulid = ulid::Ulid::from_bytes(*id.bytes());
                ulid.to_string()
            }
        }
    }
    
    /// Format as UUID string
    /// Returns the original UUID if the ID is already a UUID,
    /// or tries to interpret the ULID bytes as a UUID
    pub fn as_uuid(id: &UniqueId) -> String {
        match id.to_uuid_string() {
            Some(s) => s,
            None => {
                let uuid = uuid::Uuid::from_bytes(*id.bytes());
                uuid.hyphenated().to_string()
            }
        }
    }
} 