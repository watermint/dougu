// Essentials
pub mod obj;
pub mod build;
pub mod core;
pub mod data;
pub mod text;
pub mod fs;
pub mod time;

// Object module
pub use obj::notation::{Notation, NotationType};
pub use obj::query::Query;


// Build module
pub use build::{get_build_info, BuildInfo};

// Data module
pub use data::encoding::BinaryTextCodec;
pub use data::version::Version;
pub use data::uniqueid::{Uuid, UuidVersion, UuidVariant, UuidParser, UuidFormatter, UuidTimestamp};

// Text module
pub use text::case::{Case, CaseExt, CaseConverter};

// Time module
pub use time::{ZonedDateTime, LocalDate, LocalTime, TimeError};

// Core module - these macros are exported at the crate root because of #[macro_export]
// No need to re-export them here

pub mod prelude {
    pub use crate::core::error::{Error, Result, ErrorExt, ErrorTrait, ChainableError};
    pub use crate::core::error::{error, into_error, context};
    pub use crate::{bail, ensure};
    pub use crate::data::encoding::BinaryTextCodec;
    pub use crate::data::version::Version;
    pub use crate::data::uniqueid::{Uuid, UuidVersion, UuidVariant, UuidParser, UuidFormatter, UuidTimestamp};
    pub use crate::text::case::{Case, CaseExt, CaseConverter};
    pub use crate::time::{ZonedDateTime, LocalDate, LocalTime, TimeError};
}
