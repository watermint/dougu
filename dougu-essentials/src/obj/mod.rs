mod resources;
pub mod query;
pub mod notation;

pub use notation::{Notation, NotationType};
pub use query::Query;

use crate::core::error::Result;
use serde::Serialize;

/// Serialize a value to JSON string
pub fn to_json<T: Serialize>(value: &T) -> Result<String> {
    Ok(serde_json::to_string(value)?)
}

/// Re-export commonly used types and traits
pub mod prelude {
    pub use super::to_json;
    pub use super::Notation;
    pub use super::NotationType;
    pub use super::Query;
}
