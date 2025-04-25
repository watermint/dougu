// Essentials
pub mod obj;
pub mod runtime;
pub mod core;
pub mod data;
pub mod text;
pub mod fs;
pub mod time;
pub mod i18n;
pub mod math;
pub mod log;

// Object module
pub use obj::notation::{Notation, NotationType};
pub use obj::query::Query;


// Runtime module
pub use runtime::{get_build_info, BuildInfo};

pub use data::address::{AddressType, Email, Uri, Url};
// Data module
pub use data::encoding::BinaryTextCodec;
pub use data::uniqueid::{IdFormatter, IdParser, IdTimestamp, IdVariant, IdVersion, UniqueId};
pub use data::version::Version;

// Text module
pub use text::case::{Case, CaseConverter, CaseExt};

// Filesystem module
pub use fs::path::{Path, PathBuf, PathComponents, Namespace};

// Time module
pub use time::{LocalDate, LocalTime, TimeError, ZonedDateTime};

// i18n module
pub use i18n::{CldrDataFactory, LanguageId, LocaleDataProvider, LocaleId, RegionId};
pub use i18n::{MessageBundle, MessageFormat, MessageFormatter, MsgArgs, ResourceManager};

// Math module
pub use math::FixedDecimal;

// Log module
pub use log::{
    get_logger, get_named_logger, init_logger, init_named,
    CborFormatter, CompositeFilter, CompositeWriter, CompressWriter, Config, ConsoleWriter,
    FileWriter, Filter, FilterConfig, Formatter, FormatterConfig, FormatterType,
    JsonFormatter, LevelFilter, LogFramework, LogLevel,
    LogRecord, LogValue, Logger, LtsvFormatter,
    ModuleFilter, RotateWriter, TextFormatter, TraceInfo, Writer, WriterConfig,
};

// The log macros are exported at the crate root by #[macro_export]
// We don't need to re-export them here

// Core module - these macros are exported at the crate root because of #[macro_export]
// No need to re-export them here

pub mod prelude {
    pub use crate::core::error::{context, error, into_error};
    pub use crate::core::error::{ChainableError, Error, ErrorExt, ErrorTrait, Result};
    pub use crate::data::address::{AddressType, Email, Uri, Url};
    pub use crate::data::encoding::BinaryTextCodec;
    pub use crate::data::uniqueid::{IdFormatter, IdParser, IdTimestamp, IdVariant, IdVersion, UniqueId};
    pub use crate::data::version::Version;
    pub use crate::fs::path::{Namespace, Path, PathBuf, PathComponents};
    pub use crate::i18n::{CldrDataFactory, LanguageId, LocaleDataProvider, LocaleId, RegionId};
    pub use crate::i18n::{MessageBundle, MessageFormat, MessageFormatter, MsgArgs, ResourceManager};
    pub use crate::log::interface::LoggerExt;
    pub use crate::log::{
        get_logger, get_named_logger, init_logger, init_named,
        Config, ConsoleWriter, FileWriter,
        Filter, LevelFilter, LogLevel,
        LogRecord, LogValue, Logger, ModuleFilter, Writer,
    };
    pub use crate::math::FixedDecimal;
    pub use crate::text::case::{Case, CaseConverter, CaseExt};
    pub use crate::time::{LocalDate, LocalTime, TimeError, ZonedDateTime};
    pub use crate::{bail, ensure};
    pub use crate::{debug, error, fatal, info, trace, warn};
}
