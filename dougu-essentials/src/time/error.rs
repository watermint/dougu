use crate::core::error::ErrorTrait;

/// Errors that can occur when working with time operations
#[derive(ErrorTrait, Debug)]
pub enum TimeError {
    #[error("Invalid date format: {0}")]
    InvalidDateFormat(String),
    #[error("Invalid time format: {0}")]
    InvalidTimeFormat(String),
    #[error("Invalid datetime format: {0}")]
    InvalidDateTimeFormat(String),
    #[error("Date/time operation failed: {0}")]
    OperationFailed(String),
    #[error("Invalid Unix timestamp: {0}")]
    InvalidUnixTimestamp(String),
    #[error("Invalid duration: {0}")]
    InvalidDuration(String),
    #[error("Clock error: {0}")]
    ClockError(String),
} 