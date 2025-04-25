use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

use crate::core::error::Result;
use crate::log::config::Config;
use crate::log::filter::Filter;
use crate::log::formatter::Formatter;
use crate::log::interface::{LogRecord, Logger};
use crate::log::writer::{BoxedWriter, Writer};

/// Trace information about where a log message was generated
#[derive(Debug, Clone)]
pub struct TraceInfo {
    /// Source file path
    pub file: String,
    /// Line number
    pub line: u32,
    /// Column number
    pub column: u32,
    /// Function name
    pub function: String,
}

impl TraceInfo {
    /// Create a new trace info instance
    pub fn new(file: String, line: u32, column: u32, function: String) -> Self {
        Self {
            file,
            line,
            column,
            function,
        }
    }

    /// Create trace info from the current location
    /// This macro is not implemented here but will be used by macros
    #[allow(unused)]
    pub fn current(file: &str, line: u32, column: u32, function: &str) -> Self {
        Self {
            file: file.to_string(),
            line,
            column,
            function: function.to_string(),
        }
    }
}

/// The main logger implementation
pub struct LogFramework {
    /// Formatter
    formatter: Arc<dyn Formatter>,
    /// Writer
    writer: Arc<dyn Writer>,
    /// Filter
    filter: Arc<dyn Filter>,
}

impl LogFramework {
    /// Create a new logger framework instance
    pub fn new(
        formatter: Arc<dyn Formatter>,
        writer: Arc<dyn Writer>,
        filter: Arc<dyn Filter>,
    ) -> Self {
        Self {
            formatter,
            writer,
            filter,
        }
    }

    /// Create a logger from configuration
    pub fn from_config(config: &Config) -> Result<Self> {
        let formatter = config.formatter.create_formatter()?;
        let writer_box = config.writer.create_writer()?;

        // Convert Box<dyn Writer> to Arc<dyn Writer>
        let writer = Arc::new(BoxedWriter(writer_box));

        let filter = config.filter.create_filter()?;

        Ok(Self::new(formatter, writer, filter))
    }

    /// Get the formatter
    pub fn formatter(&self) -> &dyn Formatter {
        self.formatter.as_ref()
    }

    /// Get the writer
    pub fn writer(&self) -> &dyn Writer {
        self.writer.as_ref()
    }

    /// Get the filter
    pub fn filter(&self) -> &dyn Filter {
        self.filter.as_ref()
    }
}

impl Logger for LogFramework {
    fn log(&self, record: LogRecord) -> Result<()> {
        // Check if the record should be logged
        if !self.filter.should_log(&record)? {
            return Ok(());
        }

        // Write the record
        self.writer.write(&record, self.formatter.as_ref())
    }
}

// Global logger storage
static GLOBAL_LOGGERS: Lazy<RwLock<HashMap<String, Arc<LogFramework>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));
static DEFAULT_LOGGER: Lazy<Mutex<Option<Arc<LogFramework>>>> =
    Lazy::new(|| Mutex::new(None));

/// Initialize the default logger
pub fn init(config: Config) -> Result<()> {
    let logger = LogFramework::from_config(&config)?;
    let mut default_logger = DEFAULT_LOGGER.lock()
        .map_err(|_| crate::core::error::error("Failed to lock default logger"))?;
    *default_logger = Some(Arc::new(logger));
    Ok(())
}

/// Initialize the default logger (alias for init)
pub fn init_logger(config: Config) -> Result<()> {
    init(config)
}

/// Initialize a named logger
pub fn init_named(name: &str, config: Config) -> Result<()> {
    let logger = LogFramework::from_config(&config)?;
    let mut loggers = GLOBAL_LOGGERS.write()
        .map_err(|_| crate::core::error::error("Failed to lock loggers"))?;
    loggers.insert(name.to_string(), Arc::new(logger));
    Ok(())
}

/// Get the default logger
pub fn get_logger() -> Result<Arc<LogFramework>> {
    let mut default_logger = DEFAULT_LOGGER.lock()
        .map_err(|_| crate::core::error::error("Failed to lock default logger"))?;
    match &*default_logger {
        Some(logger) => Ok(logger.clone()),
        None => {
            // Create a default logger that logs to console
            let logger = LogFramework::from_config(&Config::default_console())?;
            let logger_arc = Arc::new(logger);
            *default_logger = Some(logger_arc.clone());
            Ok(logger_arc)
        }
    }
}

/// Get a named logger
pub fn get_named_logger(name: &str) -> Result<Arc<LogFramework>> {
    let loggers = GLOBAL_LOGGERS.read()
        .map_err(|_| crate::core::error::error("Failed to lock loggers"))?;
    match loggers.get(name) {
        Some(logger) => Ok(logger.clone()),
        None => Err(crate::core::error::error(format!("Logger '{}' not found", name))),
    }
}

/// Define macros for easy logging
/// These should be used through macros defined at the crate level
#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {{
        let logger = $crate::log::get_logger().unwrap();
        let module = module_path!();
        let message = format!($($arg)*);
        let trace = $crate::log::TraceInfo::current(file!(), line!(), 0, "");
        let mut record = $crate::log::interface::LogRecord::new(
            $crate::log::interface::LogLevel::Trace,
            module.to_string(),
            message,
            std::collections::HashMap::new(),
            Some(trace),
        );
        let _ = logger.log(record);
    }};
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        let logger = $crate::log::get_logger().unwrap();
        let module = module_path!();
        let message = format!($($arg)*);
        let trace = $crate::log::TraceInfo::current(file!(), line!(), 0, "");
        let mut record = $crate::log::interface::LogRecord::new(
            $crate::log::interface::LogLevel::Debug,
            module.to_string(),
            message,
            std::collections::HashMap::new(),
            Some(trace),
        );
        let _ = logger.log(record);
    }};
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        let logger = $crate::log::get_logger().unwrap();
        let module = module_path!();
        let message = format!($($arg)*);
        let trace = $crate::log::TraceInfo::current(file!(), line!(), 0, "");
        let mut record = $crate::log::interface::LogRecord::new(
            $crate::log::interface::LogLevel::Info,
            module.to_string(),
            message,
            std::collections::HashMap::new(),
            Some(trace),
        );
        let _ = logger.log(record);
    }};
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        let logger = $crate::log::get_logger().unwrap();
        let module = module_path!();
        let message = format!($($arg)*);
        let trace = $crate::log::TraceInfo::current(file!(), line!(), 0, "");
        let mut record = $crate::log::interface::LogRecord::new(
            $crate::log::interface::LogLevel::Warn,
            module.to_string(),
            message,
            std::collections::HashMap::new(),
            Some(trace),
        );
        let _ = logger.log(record);
    }};
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        let logger = $crate::log::get_logger().unwrap();
        let module = module_path!();
        let message = format!($($arg)*);
        let trace = $crate::log::TraceInfo::current(file!(), line!(), 0, "");
        let mut record = $crate::log::interface::LogRecord::new(
            $crate::log::interface::LogLevel::Error,
            module.to_string(),
            message,
            std::collections::HashMap::new(),
            Some(trace),
        );
        let _ = logger.log(record);
    }};
}

#[macro_export]
macro_rules! fatal {
    ($($arg:tt)*) => {{
        let logger = $crate::log::get_logger().unwrap();
        let module = module_path!();
        let message = format!($($arg)*);
        let trace = $crate::log::TraceInfo::current(file!(), line!(), 0, "");
        let mut record = $crate::log::interface::LogRecord::new(
            $crate::log::interface::LogLevel::Fatal,
            module.to_string(),
            message,
            std::collections::HashMap::new(),
            Some(trace),
        );
        let _ = logger.log(record);
    }};
} 