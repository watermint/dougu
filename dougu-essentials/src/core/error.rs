/// Re-exports anyhow's error handling capabilities with a consistent interface
/// This module provides a unified error handling approach for the dougu crate
/// and hides direct dependency on the anyhow crate.

/// Error type that wraps anyhow::Error
pub type Error = anyhow::Error;

/// Result type that uses our Error type
pub type Result<T> = std::result::Result<T, Error>;

/// Creates a new Error from a message
pub fn error<M>(msg: M) -> Error
where
    M: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
{
    anyhow::anyhow!("{}", msg)
}

/// Creates a new Error with context
pub fn context<C, E>(error: E, context: C) -> Error
where
    C: std::fmt::Display + Send + Sync + 'static,
    E: std::error::Error + Send + Sync + 'static,
{
    anyhow::Error::new(error).context(context)
}

/// Utility to add context to a Result
pub trait ErrorExt<T, E> {
    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

impl<T, E> ErrorExt<T, E> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|err| anyhow::Error::new(err).context(f()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error() {
        let err = error("test error");
        assert!(err.to_string().contains("test error"));
    }

    #[test]
    fn test_context() {
        let err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err_with_context = context(err, "failed to open config");
        assert!(err_with_context.to_string().contains("failed to open config"));
    }

    #[test]
    fn test_with_context() {
        let result: std::result::Result<(), std::io::Error> = 
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"));
        
        let err = result.with_context(|| "failed to open config");
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("failed to open config"));
    }
}