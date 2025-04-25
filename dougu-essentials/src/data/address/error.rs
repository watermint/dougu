use thiserror::Error;

pub type Result<T> = std::result::Result<T, AddressError>;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum AddressError {
    #[error("invalid email format: {0}")]
    InvalidEmailFormat(String),
    
    #[error("invalid URL format: {0}")]
    InvalidUrlFormat(String),
    
    #[error("invalid URI format: {0}")]
    InvalidUriFormat(String),
    
    #[error("invalid address type: {0}")]
    InvalidAddressType(String),
} 