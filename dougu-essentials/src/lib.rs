// Essentials
pub mod obj;
pub mod build;
pub mod core;

// Object module
pub use obj::notation::{Notation, NotationType};
pub use obj::query::Query;


// Build module
pub use build::{get_build_info, BuildInfo};

// Archive module

pub mod prelude {
    pub use crate::core::error::{Error, Result, ErrorExt};
}
