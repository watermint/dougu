mod resources;
pub mod query;
pub mod notation;

pub use notation::{Notation, NotationType};
pub use query::Query;

/// Re-export commonly used types and traits
pub mod prelude {
    pub use super::Notation;
    pub use super::NotationType;
    pub use super::Query;
}
