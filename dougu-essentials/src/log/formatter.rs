use std::collections::HashMap;
use std::fmt::Write;

use crate::core::error::Result;
use crate::log::interface::{LogRecord, LogValue};
use crate::obj;
use crate::time::ZonedDateTime;

/// Formatter type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatterType {
    /// Text formatter with customizable format string
    Text,
    /// LTSV (Labeled Tab-separated Values) formatter
    Ltsv,
    /// JSON lines formatter
    Json,
    /// CBOR sequence formatter
    Cbor,
}

/// Trait for formatting log records into bytes
pub trait Formatter: Send + Sync {
    /// Format a log record into bytes
    fn format(&self, record: &LogRecord) -> Result<Vec<u8>>;

    /// Get the formatter type
    fn formatter_type(&self) -> FormatterType;
}

/// Text formatter with customizable format string
pub struct TextFormatter {
    /// Format string with placeholders like {timestamp}, {level}, {module}, {message}
    format_string: String,
    /// Date format string for timestamp formatting
    date_format: String,
}

impl TextFormatter {
    /// Create a new text formatter with the specified format string
    pub fn new(format_string: String, date_format: String) -> Self {
        Self {
            format_string,
            date_format,
        }
    }

    /// Create a default text formatter
    pub fn default() -> Self {
        Self {
            format_string: "{timestamp} [{level}] {module}: {message}".to_string(),
            date_format: "%Y-%m-%d %H:%M:%S%.3f %z".to_string(),
        }
    }
}

impl Formatter for TextFormatter {
    fn format(&self, record: &LogRecord) -> Result<Vec<u8>> {
        let mut result = self.format_string.clone();

        // Replace placeholders
        let timestamp = format_datetime(&record.timestamp);
        result = result.replace("{timestamp}", &timestamp);
        result = result.replace("{level}", &record.level.to_string());
        result = result.replace("{module}", &record.module);
        result = result.replace("{message}", &record.message);

        // Add fields if they exist
        if !record.fields.is_empty() {
            let mut fields_str = String::new();
            for (key, value) in &record.fields {
                write!(&mut fields_str, " {}=", key)?;
                match value {
                    LogValue::String(s) => write!(&mut fields_str, "\"{}\"", s)?,
                    LogValue::Integer(i) => write!(&mut fields_str, "{}", i)?,
                    LogValue::Float(f) => write!(&mut fields_str, "{}", f)?,
                    LogValue::Boolean(b) => write!(&mut fields_str, "{}", b)?,
                    LogValue::DateTime(dt) => {
                        let dt_str = format_datetime(dt);
                        write!(&mut fields_str, "\"{}\"", dt_str)?
                    }
                    // For complex types, just indicate their type
                    LogValue::Array(_) => write!(&mut fields_str, "[...]")?,
                    LogValue::Object(_) => write!(&mut fields_str, "{{...}}")?,
                    LogValue::Null => write!(&mut fields_str, "null")?,
                }
            }
            result.push_str(&fields_str);
        }

        // Add trace info if it exists
        if let Some(trace) = &record.trace_info {
            write!(
                &mut result,
                " [{}:{}:{}]",
                trace.file,
                trace.line,
                trace.column
            )?;
        }

        Ok(result.into_bytes())
    }

    fn formatter_type(&self) -> FormatterType {
        FormatterType::Text
    }
}

/// LTSV (Labeled Tab-separated Values) formatter
pub struct LtsvFormatter;

impl LtsvFormatter {
    /// Create a new LTSV formatter
    pub fn new() -> Self {
        Self
    }
}

impl Formatter for LtsvFormatter {
    fn format(&self, record: &LogRecord) -> Result<Vec<u8>> {
        let mut parts = Vec::new();

        // Add standard fields
        parts.push(format!("time:{}", iso_format_datetime(&record.timestamp)));
        parts.push(format!("level:{}", record.level));
        parts.push(format!("module:{}", record.module));
        parts.push(format!("message:{}", record.message));

        // Add trace info if it exists
        if let Some(trace) = &record.trace_info {
            parts.push(format!("file:{}", trace.file));
            parts.push(format!("line:{}", trace.line));
            parts.push(format!("column:{}", trace.column));
        }

        // Add custom fields
        for (key, value) in &record.fields {
            let value_str = match value {
                LogValue::String(s) => s.clone(),
                LogValue::Integer(i) => i.to_string(),
                LogValue::Float(f) => f.to_string(),
                LogValue::Boolean(b) => b.to_string(),
                LogValue::DateTime(dt) => iso_format_datetime(dt),
                // For complex types, use JSON serialization
                LogValue::Array(arr) => format!("[{}]", arr.len()),
                LogValue::Object(obj) => format!("{{{}}}", obj.len()),
                LogValue::Null => "null".to_string(),
            };
            parts.push(format!("{}:{}", key, value_str));
        }

        // Join with tabs
        let result = parts.join("\t");
        Ok(result.into_bytes())
    }

    fn formatter_type(&self) -> FormatterType {
        FormatterType::Ltsv
    }
}

/// JSON lines formatter
pub struct JsonFormatter;

impl JsonFormatter {
    /// Create a new JSON formatter
    pub fn new() -> Self {
        Self
    }
}

impl Formatter for JsonFormatter {
    fn format(&self, record: &LogRecord) -> Result<Vec<u8>> {
        let mut log_object = HashMap::new();

        // Add standard fields
        log_object.insert("timestamp".to_string(), iso_format_datetime(&record.timestamp));
        log_object.insert("level".to_string(), record.level.to_string());
        log_object.insert("module".to_string(), record.module.clone());
        log_object.insert("message".to_string(), record.message.clone());

        // Add trace info if it exists
        if let Some(trace) = &record.trace_info {
            let mut trace_object = HashMap::new();
            trace_object.insert("file".to_string(), trace.file.clone());
            trace_object.insert("line".to_string(), trace.line.to_string());
            trace_object.insert("column".to_string(), trace.column.to_string());
            log_object.insert("trace".to_string(), obj::to_json(&trace_object)?);
        }

        // Add custom fields
        for (key, value) in &record.fields {
            let json_value = log_value_to_json(value)?;
            log_object.insert(key.clone(), json_value);
        }

        // Serialize to JSON
        let json_string = obj::to_json(&log_object)?;
        Ok(json_string.into_bytes())
    }

    fn formatter_type(&self) -> FormatterType {
        FormatterType::Json
    }
}

/// Helper function to convert LogValue to JSON string
fn log_value_to_json(value: &LogValue) -> Result<String> {
    match value {
        LogValue::String(s) => Ok(obj::to_json(s)?),
        LogValue::Integer(i) => Ok(i.to_string()),
        LogValue::Float(f) => Ok(f.to_string()),
        LogValue::Boolean(b) => Ok(b.to_string()),
        LogValue::DateTime(dt) => Ok(obj::to_json(&iso_format_datetime(dt))?),
        LogValue::Array(arr) => {
            let json_values: Result<Vec<String>> = arr.iter().map(log_value_to_json).collect();
            Ok(format!("[{}]", json_values?.join(", ")))
        }
        LogValue::Object(obj) => {
            let mut json_pairs = Vec::new();
            for (k, v) in obj {
                let json_value = log_value_to_json(v)?;
                json_pairs.push(format!("\"{}\":{}", k, json_value));
            }
            Ok(format!("{{{}}}", json_pairs.join(", ")))
        }
        LogValue::Null => Ok("null".to_string()),
    }
}

/// CBOR sequence formatter
pub struct CborFormatter;

impl CborFormatter {
    /// Create a new CBOR formatter
    pub fn new() -> Self {
        Self
    }
}

impl Formatter for CborFormatter {
    fn format(&self, record: &LogRecord) -> Result<Vec<u8>> {
        let mut log_map = HashMap::new();

        // Add standard fields
        log_map.insert("timestamp".to_string(), iso_format_datetime(&record.timestamp));
        log_map.insert("level".to_string(), record.level.to_string());
        log_map.insert("module".to_string(), record.module.clone());
        log_map.insert("message".to_string(), record.message.clone());

        // Add trace info if it exists
        if let Some(trace) = &record.trace_info {
            let mut trace_map = HashMap::new();
            trace_map.insert("file".to_string(), trace.file.clone());
            trace_map.insert("line".to_string(), trace.line.to_string());
            trace_map.insert("column".to_string(), trace.column.to_string());
            log_map.insert("trace".to_string(), format!("{:?}", trace_map));
        }

        // Add custom fields
        for (key, value) in &record.fields {
            let value_str = match value {
                LogValue::String(s) => s.clone(),
                LogValue::Integer(i) => i.to_string(),
                LogValue::Float(f) => f.to_string(),
                LogValue::Boolean(b) => b.to_string(),
                LogValue::DateTime(dt) => iso_format_datetime(dt),
                LogValue::Array(arr) => format!("{:?}", arr),
                LogValue::Object(obj) => format!("{:?}", obj),
                LogValue::Null => "null".to_string(),
            };
            log_map.insert(key.clone(), value_str);
        }

        // Serialize to CBOR
        let mut cbor_bytes = Vec::new();
        ciborium::ser::into_writer(&log_map, &mut cbor_bytes)
            .map_err(|e| crate::core::error::error(format!("CBOR serialization error: {}", e)))?;

        Ok(cbor_bytes)
    }

    fn formatter_type(&self) -> FormatterType {
        FormatterType::Cbor
    }
}

/// Format a ZonedDateTime with a standard format
fn format_datetime(dt: &ZonedDateTime) -> String {
    dt.format()
}

/// Format a ZonedDateTime with ISO 8601 format
fn iso_format_datetime(dt: &ZonedDateTime) -> String {
    dt.format()
} 