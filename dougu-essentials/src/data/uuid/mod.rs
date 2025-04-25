mod error;
mod format;
mod parse;
mod timestamp;
mod types;

#[cfg(test)]
mod tests;

pub use error::{Error, Result};
pub use format::UuidFormatter;
pub use parse::UuidParser;
pub use timestamp::UuidTimestamp;
pub use types::{Uuid, UuidVersion, UuidVariant};

// Re-export the standard Uuid type from uuid crate for compatibility when needed
pub use uuid::Uuid as RawUuid; 