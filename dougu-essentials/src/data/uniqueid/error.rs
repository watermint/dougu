use crate::core::error::ErrorTrait;

/// Error types for UniqueId operations
#[derive(ErrorTrait, Debug)]
pub enum Error {
    /// Error parsing a UUID from string
    #[error("Failed to parse UUID: {0}")]
    UuidParse(#[from] uuid::Error),

    /// Error parsing a ULID from string
    #[error("Failed to parse ULID: {0}")]
    UlidParse(String),

    /// Invalid identifier format
    #[error("Invalid identifier format: {0}")]
    InvalidFormat(String),

    /// Unsupported UUID version
    #[error("Unsupported UUID version: {0}")]
    UnsupportedVersion(String),

    /// Unable to extract timestamp
    #[error("Unable to extract timestamp: {0}")]
    TimestampExtraction(String),

    /// Operation not supported for this identifier type
    #[error("Operation not supported for this identifier type: {0}")]
    UnsupportedOperation(String),

    /// General error
    #[error("UniqueId error: {0}")]
    Other(String),
}

/// Result type for UniqueId operations
pub type Result<T> = std::result::Result<T, Error>; 