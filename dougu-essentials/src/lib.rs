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

pub use data::address::{AddressType, Email, Uri, Url};
// Data module
pub use data::encoding::BinaryTextCodec;
pub use data::uniqueid::{IdFormatter, IdParser, IdTimestamp, IdVariant, IdVersion, UniqueId};
pub use data::version::Version;

// Text module
pub use text::case::{Case, CaseConverter, CaseExt};

// Time module
pub use time::{LocalDate, LocalTime, TimeError, ZonedDateTime};

// Core module - these macros are exported at the crate root because of #[macro_export]
// No need to re-export them here

pub mod prelude {
    pub use crate::core::error::{context, error, into_error};
    pub use crate::core::error::{ChainableError, Error, ErrorExt, ErrorTrait, Result};
    pub use crate::data::address::{AddressType, Email, Uri, Url};
    pub use crate::data::encoding::BinaryTextCodec;
    pub use crate::data::uniqueid::{IdFormatter, IdParser, IdTimestamp, IdVariant, IdVersion, UniqueId};
    pub use crate::data::version::Version;
    pub use crate::text::case::{Case, CaseConverter, CaseExt};
    pub use crate::time::{LocalDate, LocalTime, TimeError, ZonedDateTime};
    pub use crate::{bail, ensure};
}
