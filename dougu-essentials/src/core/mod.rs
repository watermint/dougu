/// Core module for dougu-essentials
pub mod error;

pub use crate::{bail, ensure};
pub use error::{context, error, into_error, ChainableError, Error, ErrorExt, ErrorTrait, Result};
