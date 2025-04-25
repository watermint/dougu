// Log module for dougu-essentials
// This module provides a comprehensive logging framework with:
// - Pluggable formatters (LTSV, JSON, CBOR, text)
// - Nestable writers with various destinations
// - Flexible configuration
// - Filtering by log level and module name
// - Support for multiple writers and structured logging
// - Trace logging with source code position

mod formatter;
mod writer;
mod config;
mod filter;
pub mod interface;
pub mod framework;

pub use formatter::{
    CborFormatter,
    Formatter,
    FormatterType,
    JsonFormatter,
    LtsvFormatter,
    TextFormatter,
};

pub use writer::{
    CompositeWriter,
    CompressWriter,
    ConsoleWriter,
    FileWriter,
    RotateWriter,
    Writer,
};

pub use config::{
    Config,
    FilterConfig,
    FormatterConfig,
    WriterConfig,
};

pub use filter::{
    CompositeFilter,
    Filter,
    LevelFilter,
    ModuleFilter,
};

pub use interface::{
    LogLevel,
    LogRecord,
    LogValue,
    Logger,
};

pub use framework::{
    get_logger,
    get_named_logger,
    init,
    init_logger,
    init_named,
    LogFramework,
    TraceInfo,
};
