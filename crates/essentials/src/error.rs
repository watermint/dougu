use std::fmt;

#[derive(Debug)]
pub enum Error {
    ResourceNotFound(String),
    CommandFailed(String),
    ConfigError(String),
    IoError(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ResourceNotFound(res) => write!(f, "Resource not found: {}", res),
            Error::CommandFailed(cmd) => write!(f, "Command failed: {}", cmd),
            Error::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            Error::IoError(err) => write!(f, "I/O error: {}", err),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
} 