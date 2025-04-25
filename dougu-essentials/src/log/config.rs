use std::path::PathBuf;
use std::sync::Arc;

use crate::core::error::{self, Result};
use crate::log::filter::{CompositeFilter, Filter, LevelFilter, ModuleFilter};
use crate::log::formatter::{CborFormatter, Formatter, JsonFormatter, LtsvFormatter, TextFormatter};
use crate::log::interface::{LogLevel, Logger};
use crate::log::writer::{CompositeWriter, ConsoleWriter, FileWriter, RotateWriter, Writer};

/// Configuration for a formatter
#[derive(Debug, Clone)]
pub enum FormatterConfig {
    /// Text formatter with custom format string
    Text {
        /// Format string with placeholders
        format_string: String,
        /// Date format string
        date_format: String,
    },
    /// LTSV formatter
    Ltsv,
    /// JSON formatter
    Json,
    /// CBOR formatter
    Cbor,
}

impl FormatterConfig {
    /// Create a formatter from the configuration
    pub fn create_formatter(&self) -> Result<Arc<dyn Formatter>> {
        match self {
            FormatterConfig::Text { format_string, date_format } => {
                Ok(Arc::new(TextFormatter::new(format_string.clone(), date_format.clone())))
            }
            FormatterConfig::Ltsv => {
                Ok(Arc::new(LtsvFormatter::new()))
            }
            FormatterConfig::Json => {
                Ok(Arc::new(JsonFormatter::new()))
            }
            FormatterConfig::Cbor => {
                Ok(Arc::new(CborFormatter::new()))
            }
        }
    }
}

/// Configuration for a filter
#[derive(Debug, Clone)]
pub enum FilterConfig {
    /// Level filter
    Level {
        /// Minimum log level
        minimum_level: LogLevel,
    },
    /// Module filter
    Module {
        /// Modules to include
        include: Vec<String>,
        /// Modules to exclude
        exclude: Vec<String>,
    },
    /// Composite filter with AND logic
    All {
        /// Filters to combine with AND logic
        filters: Vec<FilterConfig>,
    },
    /// Composite filter with OR logic
    Any {
        /// Filters to combine with OR logic
        filters: Vec<FilterConfig>,
    },
}

impl FilterConfig {
    /// Create a filter from the configuration
    pub fn create_filter(&self) -> Result<Arc<dyn Filter>> {
        match self {
            FilterConfig::Level { minimum_level } => {
                Ok(Arc::new(LevelFilter::new(*minimum_level)))
            }
            FilterConfig::Module { include, exclude } => {
                Ok(Arc::new(ModuleFilter::new(include.clone(), exclude.clone())))
            }
            FilterConfig::All { filters } => {
                let mut filter_impls = Vec::new();
                for filter in filters {
                    filter_impls.push(filter.create_filter()?);
                }
                Ok(Arc::new(CompositeFilter::all(filter_impls)))
            }
            FilterConfig::Any { filters } => {
                let mut filter_impls = Vec::new();
                for filter in filters {
                    filter_impls.push(filter.create_filter()?);
                }
                Ok(Arc::new(CompositeFilter::any(filter_impls)))
            }
        }
    }
}

/// Configuration for a writer
#[derive(Debug, Clone)]
pub enum WriterConfig {
    /// Console writer
    Console {
        /// Whether to write to stderr
        stderr: bool,
    },
    /// File writer
    File {
        /// Path to the file
        path: PathBuf,
    },
    /// Rotate writer
    Rotate {
        /// Base path for log files
        base_path: PathBuf,
        /// Maximum file size in bytes
        max_size: u64,
        /// Maximum number of files to keep
        max_files: usize,
        /// Rotation period in seconds
        rotation_period: u64,
    },
    /// Compress writer
    Compress {
        /// Compression level (0-9)
        level: u32,
        /// The writer to compress
        writer: Box<WriterConfig>,
    },
    /// Composite writer
    Composite {
        /// The writers to combine
        writers: Vec<WriterConfig>,
    },
}

impl WriterConfig {
    /// Create a writer from the configuration
    pub fn create_writer(&self) -> Result<Box<dyn Writer>> {
        match self {
            WriterConfig::Console { stderr } => Ok(Box::new(ConsoleWriter::new(*stderr))),
            WriterConfig::File { path } => Ok(Box::new(FileWriter::new(path)?)),
            WriterConfig::Rotate {
                base_path,
                max_size,
                max_files,
                rotation_period,
            } => Ok(Box::new(RotateWriter::new(base_path, *max_size, *max_files, *rotation_period)?)),
            WriterConfig::Composite { writers } => {
                let mut composite_writers: Vec<Box<dyn Writer>> = Vec::new();
                for writer_config in writers {
                    composite_writers.push(writer_config.create_writer()?);
                }
                Ok(Box::new(CompositeWriter::from_boxed(composite_writers)))
            }
            WriterConfig::Compress { .. } => {
                Err(error::error("CompressWriter cannot be directly created from config - use it programmatically instead"))
            }
        }
    }
}

/// Logger configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Formatter configuration
    pub formatter: FormatterConfig,
    /// Writer configuration
    pub writer: WriterConfig,
    /// Filter configuration
    pub filter: FilterConfig,
}

impl Config {
    /// Create a default configuration for console logging
    pub fn default_console() -> Self {
        Self {
            formatter: FormatterConfig::Text {
                format_string: "{timestamp} [{level}] {module}: {message}".to_string(),
                date_format: "%Y-%m-%d %H:%M:%S%.3f %z".to_string(),
            },
            writer: WriterConfig::Console { stderr: false },
            filter: FilterConfig::Level { minimum_level: LogLevel::Info },
        }
    }

    /// Create a default configuration for file logging
    pub fn default_file<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            formatter: FormatterConfig::Text {
                format_string: "{timestamp} [{level}] {module}: {message}".to_string(),
                date_format: "%Y-%m-%d %H:%M:%S%.3f %z".to_string(),
            },
            writer: WriterConfig::File { path: path.into() },
            filter: FilterConfig::Level { minimum_level: LogLevel::Info },
        }
    }

    /// Create a default configuration for rotating file logging
    pub fn default_rotate<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            formatter: FormatterConfig::Text {
                format_string: "{timestamp} [{level}] {module}: {message}".to_string(),
                date_format: "%Y-%m-%d %H:%M:%S%.3f %z".to_string(),
            },
            writer: WriterConfig::Rotate {
                base_path: path.into(),
                max_size: 10 * 1024 * 1024, // 10 MB
                max_files: 5,
                rotation_period: 86400, // 1 day
            },
            filter: FilterConfig::Level { minimum_level: LogLevel::Info },
        }
    }

    /// Create a default configuration for JSON logging
    pub fn default_json<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            formatter: FormatterConfig::Json,
            writer: WriterConfig::File { path: path.into() },
            filter: FilterConfig::Level { minimum_level: LogLevel::Info },
        }
    }
} 