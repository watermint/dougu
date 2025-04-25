/// Core module for dougu-essentials
pub mod error;

pub use error::{Error, Result, ErrorExt, ErrorTrait, ChainableError, error, into_error, context};
// Macros defined with #[macro_export] are available at the crate root
// Re-export them here for convenience
pub use crate::{bail, ensure};
