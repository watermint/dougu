pub mod i18n;
pub mod log;
pub mod obj;
pub mod fs;
pub mod kvs;
pub mod sql;
pub mod build;
pub mod archive;

// Re-export public types from each module
// I18n module
pub use i18n::{I18n, Locale, LocaleError, TranslationMessage};

// Log module
pub use log::{init, log_error, log_info, log_warning};

// Object module
pub use obj::{Converter, Decoder, Encoder, Format, Query};

// FS module
pub use fs::{FileMetadata, FileSystem, FileSystemEntry, FileSystemProvider, ReadOptions, WriteOptions};

// KVS module
pub use kvs::{KeyValuePair, KvsProvider, RedbKvsProvider};

// SQL module
pub use sql::{SqlProvider, SqlRow, SqlValue, SqliteProvider};

// Build module
pub use build::{get_build_info, BuildInfo};

// Archive module
pub use archive::{Archive, ArchiveEntry, ArchiveMetadata, ArchiveProvider, EntryOptions, ExtractOptions};
