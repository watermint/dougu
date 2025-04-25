// Essentials
pub mod obj;
pub mod build;
pub mod core;
pub mod data;

// Object module
pub use obj::notation::{Notation, NotationType};
pub use obj::query::Query;


// Build module
pub use build::{get_build_info, BuildInfo};

// Data module
pub use data::encoding::BinaryTextCodec;
pub use data::version::Version;

// Archive module

pub mod prelude {
    pub use crate::core::error::{Error, Result, ErrorExt};
    pub use crate::data::encoding::BinaryTextCodec;
    pub use crate::data::version::Version;
}
