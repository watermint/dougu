/// Core module for dougu-essentials
pub mod error;

// Macros defined with #[macro_export] are available at the crate root
// Re-export them here for convenience
pub use crate::{bail, ensure};
pub use error::{context, error, into_error, ChainableError, Error, ErrorExt, ErrorTrait, Result};
