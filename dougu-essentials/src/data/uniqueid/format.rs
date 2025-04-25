use crate::data::uniqueid::error::Result;
use crate::data::uniqueid::types::Uuid;

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
} 