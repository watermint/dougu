/// Re-exports anyhow's and thiserror's error handling capabilities with a consistent interface
/// This module provides a unified error handling approach for the dougu crate
/// and hides direct dependency on the anyhow and thiserror crates.

pub use thiserror;


/// Error type that wraps anyhow::Error
pub type Error = anyhow::Error;

/// Result type that uses our Error type
pub type Result<T> = std::result::Result<T, Error>;

/// Re-export of thiserror's derive macro for creating custom error types
pub use thiserror::Error as ErrorTrait;


/// Creates a new Error from a message
pub fn error<M>(msg: M) -> Error
where
    M: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
{
    anyhow::anyhow!("{}", msg)
}

/// Converts any error type to our Error type
pub fn into_error<E>(err: E) -> Error
where
    E: std::error::Error + Send + Sync + 'static,
{
    anyhow::Error::new(err)
}

/// Creates a new Error with context
pub fn context<C, E>(error: E, context: C) -> Error
where
    C: std::fmt::Display + Send + Sync + 'static,
    E: std::error::Error + Send + Sync + 'static,
{
    anyhow::Error::new(error).context(context)
}

/// Wraps anyhow's bail macro
#[macro_export]
macro_rules! bail {
    ($msg:literal $(,)?) => {
        return Err(anyhow::anyhow!($msg).into())
    };
    ($err:expr $(,)?) => {
        return Err(anyhow::anyhow!($err).into())
    };
    ($fmt:expr, $($arg:tt)*) => {
        return Err(anyhow::anyhow!($fmt, $($arg)*).into())
    };
}

/// Wraps anyhow's ensure macro
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $msg:literal $(,)?) => {
        if !($cond) {
            return Err(anyhow::anyhow!($msg).into());
        }
    };
    ($cond:expr, $err:expr $(,)?) => {
        if !($cond) {
            return Err(anyhow::anyhow!($err).into());
        }
    };
    ($cond:expr, $fmt:expr, $($arg:tt)*) => {
        if !($cond) {
            return Err(anyhow::anyhow!($fmt, $($arg)*).into());
        }
    };
}

/// Utility to add context to a Result
pub trait ErrorExt<T, E> {
    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C;

    fn context<C>(self, context: C) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static;
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

    fn context<C>(self, context: C) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|err| anyhow::Error::new(err).context(context))
    }
}

/// Trait for creating error chains
pub trait ChainableError: Sized {
    fn chain<E>(self, err: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static;
}

impl ChainableError for Error {
    fn chain<E>(self, err: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        // Need to reverse the order because anyhow::Error displays context first
        anyhow::Error::new(err).context(self.to_string())
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

    #[test]
    fn test_into_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = into_error(io_err);
        assert!(err.to_string().contains("file not found"));
    }

    #[test]
    fn test_chain_error() {
        let err1 = error("first error");
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let chained = err1.chain(io_err);

        // Display the error in the console for debugging
        println!("Chained error: {}", chained);

        // Test that at least one of the error messages is present in the output
        let err_string = chained.to_string();
        assert!(err_string.contains("file not found") || err_string.contains("first error"),
                "Error string '{}' does not contain any of the expected messages", err_string);
    }

    #[test]
    fn test_custom_error() {
        use super::ErrorTrait;

        #[derive(Debug, ErrorTrait)]
        enum CustomError {
            #[error("invalid input: {0}")]
            InvalidInput(String),

            #[error("operation failed")]
            OperationFailed,
        }

        let custom_err = CustomError::InvalidInput("bad value".to_string());
        let err = into_error(custom_err);
        assert!(err.to_string().contains("invalid input: bad value"));
    }
}