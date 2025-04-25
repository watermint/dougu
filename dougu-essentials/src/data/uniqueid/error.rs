use crate::core::error::ErrorTrait;

/// Error types for UUID operations
#[derive(ErrorTrait, Debug)]
pub enum Error {
    /// Error parsing a UUID from string
    #[error("Failed to parse UUID: {0}")]
    Parse(#[from] uuid::Error),

    /// Invalid UUID format
    #[error("Invalid UUID format: {0}")]
    InvalidFormat(String),

    /// Unsupported UUID version
    #[error("Unsupported UUID version: {0}")]
    UnsupportedVersion(String),

    /// Unable to extract timestamp
    #[error("Unable to extract timestamp: {0}")]
    TimestampExtraction(String),

    /// General error
    #[error("UUID error: {0}")]
    Other(String),
}

/// Result type for UUID operations
pub type Result<T> = std::result::Result<T, Error>; 