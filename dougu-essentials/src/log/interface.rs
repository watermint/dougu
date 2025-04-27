use std::collections::HashMap;
use std::fmt;

use crate::core::error::Result;
use crate::time::ZonedDateTime;

/// Log level enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Trace level logging for fine-grained information
    /// Used for function entry/exit points and detailed execution flow
    Trace,
    /// Debug level logging for detailed diagnostic information
    /// Used during development to help diagnose issues
    Debug,
    /// Info level logging for general operational information
    /// Used to track normal application flow
    Info,
    /// Warning level logging for potentially harmful situations
    /// Used to notify users about situations that require attention
    Warn,
    /// Error level logging for situations that require user action
    /// Used when operations fail and users need to fix, wait, or retry
    Error,
    /// Fatal level logging for system-level fatal situations
    /// Used for unrecoverable errors, should be followed by a panic
    Fatal,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Fatal => write!(f, "FATAL"),
        }
    }
}

/// Log value that can represent different types of data
#[derive(Debug, Clone)]
pub enum LogValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    DateTime(ZonedDateTime),
    Array(Vec<LogValue>),
    Object(HashMap<String, LogValue>),
    Null,
}

impl From<String> for LogValue {
    fn from(value: String) -> Self {
        LogValue::String(value)
    }
}

impl From<&str> for LogValue {
    fn from(value: &str) -> Self {
        LogValue::String(value.to_string())
    }
}

impl From<i64> for LogValue {
    fn from(value: i64) -> Self {
        LogValue::Integer(value)
    }
}

impl From<i32> for LogValue {
    fn from(value: i32) -> Self {
        LogValue::Integer(value as i64)
    }
}

impl From<f64> for LogValue {
    fn from(value: f64) -> Self {
        LogValue::Float(value)
    }
}

impl From<bool> for LogValue {
    fn from(value: bool) -> Self {
        LogValue::Boolean(value)
    }
}

impl From<ZonedDateTime> for LogValue {
    fn from(value: ZonedDateTime) -> Self {
        LogValue::DateTime(value)
    }
}

impl<T: Into<LogValue>> From<Vec<T>> for LogValue {
    fn from(values: Vec<T>) -> Self {
        LogValue::Array(values.into_iter().map(|v| v.into()).collect())
    }
}

impl<T: Into<LogValue>> From<HashMap<String, T>> for LogValue {
    fn from(values: HashMap<String, T>) -> Self {
        LogValue::Object(values.into_iter().map(|(k, v)| (k, v.into())).collect())
    }
}

/// A log record containing all information about a log event
#[derive(Debug, Clone)]
pub struct LogRecord {
    /// Timestamp when the log was created
    pub timestamp: ZonedDateTime,
    /// Log level
    pub level: LogLevel,
    /// Module name
    pub module: String,
    /// Main message
    pub message: String,
    /// Additional structured data
    pub fields: HashMap<String, LogValue>,
    /// Trace information (file, line, column)
    pub trace_info: Option<crate::log::framework::TraceInfo>,
}

impl LogRecord {
    /// Create a new log record
    pub fn new(
        level: LogLevel,
        module: String,
        message: String,
        fields: HashMap<String, LogValue>,
        trace_info: Option<crate::log::framework::TraceInfo>,
    ) -> Self {
        Self {
            timestamp: ZonedDateTime::now(),
            level,
            module,
            message,
            fields,
            trace_info,
        }
    }

    /// Add a field to the log record
    pub fn with_field<K: Into<String>, V: Into<LogValue>>(mut self, key: K, value: V) -> Self {
        self.fields.insert(key.into(), value.into());
        self
    }
}

/// Logger interface - core trait that can be used with dyn
pub trait Logger: Send + Sync {
    /// Log a message at the specified level
    fn log(&self, record: LogRecord) -> Result<()>;
}

/// Extended logger functionality
pub trait LoggerExt: Logger {
    /// Log a trace message
    /// Used for fine-grained information such as function entry/exit points
    fn trace<M: Into<String>>(&self, module: &str, message: M) -> Result<()> {
        self.log(LogRecord::new(
            LogLevel::Trace,
            module.to_string(),
            message.into(),
            HashMap::new(),
            None,
        ))
    }

    /// Log a debug message
    /// Used for detailed diagnostic information during development
    fn debug<M: Into<String>>(&self, module: &str, message: M) -> Result<()> {
        self.log(LogRecord::new(
            LogLevel::Debug,
            module.to_string(),
            message.into(),
            HashMap::new(),
            None,
        ))
    }

    /// Log an info message
    /// Used for general operational information and normal application flow
    fn info<M: Into<String>>(&self, module: &str, message: M) -> Result<()> {
        self.log(LogRecord::new(
            LogLevel::Info,
            module.to_string(),
            message.into(),
            HashMap::new(),
            None,
        ))
    }

    /// Log a warning message
    /// Used for potentially harmful situations that require user attention
    fn warn<M: Into<String>>(&self, module: &str, message: M) -> Result<()> {
        self.log(LogRecord::new(
            LogLevel::Warn,
            module.to_string(),
            message.into(),
            HashMap::new(),
            None,
        ))
    }

    /// Log an error message
    /// Used when operations fail and users need to fix, wait, or retry
    fn error<M: Into<String>>(&self, module: &str, message: M) -> Result<()> {
        self.log(LogRecord::new(
            LogLevel::Error,
            module.to_string(),
            message.into(),
            HashMap::new(),
            None,
        ))
    }

    /// Log a fatal message
    /// Used for system-level fatal situations, should be followed by a panic
    fn fatal<M: Into<String>>(&self, module: &str, message: M) -> Result<()> {
        self.log(LogRecord::new(
            LogLevel::Fatal,
            module.to_string(),
            message.into(),
            HashMap::new(),
            None,
        ))
    }

    /// Create a structured log entry builder
    fn with_fields(&self, module: &str) -> LogBuilder<'_>
    where
        Self: Sized,
    {
        LogBuilder {
            logger: self,
            module: module.to_string(),
            fields: HashMap::new(),
        }
    }
}

// Implement LoggerExt for all Logger types
impl<T: Logger> LoggerExt for T {}

/// Helper for building structured log entries
pub struct LogBuilder<'a> {
    logger: &'a dyn Logger,
    module: String,
    fields: HashMap<String, LogValue>,
}

impl<'a> LogBuilder<'a> {
    /// Add a field to the log record
    pub fn field<K: Into<String>, V: Into<LogValue>>(mut self, key: K, value: V) -> Self {
        self.fields.insert(key.into(), value.into());
        self
    }

    /// Log a trace message with the accumulated fields
    /// Used for fine-grained information such as function entry/exit points
    pub fn trace<M: Into<String>>(&self, message: M) -> Result<()> {
        self.logger.log(LogRecord::new(
            LogLevel::Trace,
            self.module.clone(),
            message.into(),
            self.fields.clone(),
            None,
        ))
    }

    /// Log a debug message with the accumulated fields
    /// Used for detailed diagnostic information during development
    pub fn debug<M: Into<String>>(&self, message: M) -> Result<()> {
        self.logger.log(LogRecord::new(
            LogLevel::Debug,
            self.module.clone(),
            message.into(),
            self.fields.clone(),
            None,
        ))
    }

    /// Log an info message with the accumulated fields
    /// Used for general operational information and normal application flow
    pub fn info<M: Into<String>>(&self, message: M) -> Result<()> {
        self.logger.log(LogRecord::new(
            LogLevel::Info,
            self.module.clone(),
            message.into(),
            self.fields.clone(),
            None,
        ))
    }

    /// Log a warning message with the accumulated fields
    /// Used for potentially harmful situations that require user attention
    pub fn warn<M: Into<String>>(&self, message: M) -> Result<()> {
        self.logger.log(LogRecord::new(
            LogLevel::Warn,
            self.module.clone(),
            message.into(),
            self.fields.clone(),
            None,
        ))
    }

    /// Log an error message with the accumulated fields
    /// Used when operations fail and users need to fix, wait, or retry
    pub fn error<M: Into<String>>(&self, message: M) -> Result<()> {
        self.logger.log(LogRecord::new(
            LogLevel::Error,
            self.module.clone(),
            message.into(),
            self.fields.clone(),
            None,
        ))
    }

    /// Log a fatal message with the accumulated fields
    /// Used for system-level fatal situations, should be followed by a panic
    pub fn fatal<M: Into<String>>(&self, message: M) -> Result<()> {
        self.logger.log(LogRecord::new(
            LogLevel::Fatal,
            self.module.clone(),
            message.into(),
            self.fields.clone(),
            None,
        ))
    }
} 