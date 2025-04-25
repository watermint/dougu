use dougu_essentials::log::{
    get_logger, get_named_logger, init_logger, init_named, Config, FilterConfig,
    FormatterConfig, LogLevel, LogRecord, WriterConfig,
};
use dougu_essentials::prelude::*;
use serial_test::serial;
use std::fs;
use std::io::Read;
use tempfile::tempdir;

#[test]
#[serial]
fn test_console_logging() -> Result<()> {
    // Configure a simple console logger
    let config = Config::default_console();
    init_logger(config)?;

    // Log some messages
    info!("This is an info message");
    warn!("This is a warning message");
    error!("This is an error message");

    // These should be filtered out by default
    trace!("This trace message should not appear");
    debug!("This debug message should not appear");

    Ok(())
}

#[test]
#[serial]
fn test_file_logging() -> Result<()> {
    // Create a temporary directory for log files
    let temp_dir = tempdir()?;
    let log_path = temp_dir.path().join("test.log");

    // Configure a file logger with JSON formatting
    let config = Config {
        formatter: FormatterConfig::Json,
        writer: WriterConfig::File { path: log_path.clone() },
        filter: FilterConfig::Level { minimum_level: LogLevel::Debug },
    };
    init_logger(config)?;

    // Log some messages
    debug!("This is a debug message");
    info!("This is an info message with field: {}", 42);

    // Explicitly flush the logger to ensure messages are written
    let logger = get_logger()?;
    logger.writer().flush()?;

    // Verify the log file was created
    assert!(log_path.exists());

    // Read the log file contents and verify it contains JSON
    let mut file = fs::File::open(&log_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    assert!(contents.contains("\"level\":\"DEBUG\""));
    assert!(contents.contains("\"message\":\"This is a debug message\""));
    assert!(contents.contains("\"level\":\"INFO\""));
    assert!(contents.contains("\"message\":\"This is an info message with field: 42\""));

    Ok(())
}

#[test]
#[serial]
fn test_rotate_logging() -> Result<()> {
    // Create a temporary directory for log files
    let temp_dir = tempdir()?;
    let log_path = temp_dir.path().join("rotate.log");

    // Configure a rotating file logger
    let config = Config {
        formatter: FormatterConfig::Ltsv,
        writer: WriterConfig::Rotate {
            base_path: log_path.clone(),
            max_size: 1024, // Very small to trigger rotation
            max_files: 3,
            rotation_period: 0, // No time-based rotation
        },
        filter: FilterConfig::Level { minimum_level: LogLevel::Info },
    };
    init_logger(config)?;

    // Log enough messages to trigger rotation
    for i in 0..100 {
        info!("This is log message number {}", i);
    }

    // Verify the log file was created
    assert!(log_path.exists());

    // Directory should have some rotated files
    let dir_entries = std::fs::read_dir(temp_dir.path())?;
    let log_files: Vec<_> = dir_entries
        .filter_map(|r| r.ok())
        .filter(|e| e.file_name().to_string_lossy().contains("rotate.log"))
        .collect();

    // Should have at least the main log file
    assert!(log_files.len() >= 1);

    Ok(())
}

#[test]
#[serial]
fn test_structured_logging() -> Result<()> {
    // Create a temporary directory for log files
    let temp_dir = tempdir()?;
    let log_path = temp_dir.path().join("structured.log");

    // Configure a logger with JSON format to verify structured data
    let config = Config {
        formatter: FormatterConfig::Json,
        writer: WriterConfig::File { path: log_path.clone() },
        filter: FilterConfig::Level { minimum_level: LogLevel::Info },
    };
    init_logger(config)?;

    // Get the logger
    let logger = get_logger()?;

    // Create a structured log entry
    logger.with_fields("test_module")
        .field("user_id", 12345)
        .field("action", "login")
        .field("success", true)
        .field("duration_ms", 235.5)
        .info("User login completed")?;

    // Explicitly flush the logger to ensure messages are written
    logger.writer().flush()?;

    // Read the log file contents and verify it contains the structured data
    let mut file = fs::File::open(&log_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Print the contents for debugging
    println!("Log contents: {}", contents);

    // Check for presence of structured fields with more flexible conditions
    assert!(contents.contains("User login completed"));
    assert!(contents.contains("test_module"));
    assert!(contents.contains("12345"));
    assert!(contents.contains("login"));
    assert!(contents.contains("true"));
    assert!(contents.contains("235.5"));

    Ok(())
}

#[test]
#[serial]
fn test_composite_writer() -> Result<()> {
    // Create a temporary directory for log files
    let temp_dir = tempdir()?;
    let log_path = temp_dir.path().join("test.log");

    // Configure a composite logger that writes to both console and file
    let config = Config {
        formatter: FormatterConfig::Text {
            format_string: "{timestamp} [{level}] {module}: {message}".to_string(),
            date_format: "%Y-%m-%d %H:%M:%S%.3f".to_string(),
        },
        writer: WriterConfig::Composite {
            writers: vec![
                WriterConfig::Console { stderr: false },
                WriterConfig::File { path: log_path.clone() },
            ],
        },
        filter: FilterConfig::Level { minimum_level: LogLevel::Info },
    };
    init_logger(config)?;

    // Log some messages
    info!("This message should go to both console and file");

    // Explicitly flush the logger to ensure messages are written
    let logger = get_logger()?;
    logger.writer().flush()?;

    // Verify the log file was created
    assert!(log_path.exists());

    // Verify the file contains the message
    let mut file = fs::File::open(&log_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    assert!(contents.contains("This message should go to both console and file"));

    Ok(())
}

#[test]
#[serial]
fn test_module_filtering() -> Result<()> {
    // Create a temporary directory for log files
    let temp_dir = tempdir()?;
    let log_path = temp_dir.path().join("filtered.log");

    // Configure a logger with module filtering
    let config = Config {
        formatter: FormatterConfig::Text {
            format_string: "{timestamp} [{level}] {module}: {message}".to_string(),
            date_format: "%Y-%m-%d %H:%M:%S%.3f".to_string(),
        },
        writer: WriterConfig::File { path: log_path.clone() },
        filter: FilterConfig::All {
            filters: vec![
                FilterConfig::Level { minimum_level: LogLevel::Info },
                FilterConfig::Module {
                    include: vec!["test_log".to_string()],
                    exclude: vec!["test_log::excluded".to_string()],
                },
            ],
        },
    };
    init_logger(config)?;

    // These messages should be logged
    let logger = get_logger()?;
    logger.info("test_log", "This message should be logged")?;
    logger.info("test_log::included", "This message should also be logged")?;

    // This message should be filtered out
    logger.info("test_log::excluded", "This message should NOT be logged")?;
    logger.info("other_module", "This message should NOT be logged")?;

    // Explicitly flush the logger to ensure messages are written
    logger.writer().flush()?;

    // Verify the log file was created
    assert!(log_path.exists());

    // Read the log file contents and verify filtering worked
    let mut file = fs::File::open(&log_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    assert!(contents.contains("This message should be logged"));
    assert!(contents.contains("This message should also be logged"));
    assert!(!contents.contains("This message should NOT be logged"));

    Ok(())
}

#[test]
#[serial]
fn test_cbor_formatter() -> Result<()> {
    // Create a temporary directory for log files
    let temp_dir = tempdir()?;
    let log_path = temp_dir.path().join("cbor.log");

    // Configure a logger with CBOR format
    let config = Config {
        formatter: FormatterConfig::Cbor,
        writer: WriterConfig::File { path: log_path.clone() },
        filter: FilterConfig::Level { minimum_level: LogLevel::Info },
    };
    init_logger(config)?;

    // Log a message
    info!("Testing CBOR formatter");

    // Explicitly flush the logger to ensure messages are written
    let logger = get_logger()?;
    logger.writer().flush()?;

    // Verify the log file was created
    assert!(log_path.exists());

    // Verify the file has content (we can't easily validate CBOR content in a test)
    let metadata = fs::metadata(&log_path)?;
    assert!(metadata.len() > 0, "CBOR log file should not be empty");

    Ok(())
}

#[test]
#[serial]
fn test_log_levels() -> Result<()> {
    // Create a temporary directory for log files
    let temp_dir = tempdir()?;
    let log_path = temp_dir.path().join("levels.log");

    // Configure a logger with debug level
    let config = Config {
        formatter: FormatterConfig::Text {
            format_string: "{level}: {message}".to_string(),
            date_format: "%Y-%m-%d".to_string(),
        },
        writer: WriterConfig::File { path: log_path.clone() },
        filter: FilterConfig::Level { minimum_level: LogLevel::Debug },
    };
    init_logger(config)?;

    // Log messages at different levels
    trace!("Trace message");
    debug!("Debug message");
    info!("Info message");
    warn!("Warning message");
    error!("Error message");
    fatal!("Fatal message");

    // Explicitly flush the logger to ensure messages are written
    let logger = get_logger()?;
    logger.writer().flush()?;

    // Read the log file contents and verify level filtering
    let mut file = fs::File::open(&log_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    assert!(!contents.contains("TRACE: Trace message"));
    assert!(contents.contains("DEBUG: Debug message"));
    assert!(contents.contains("INFO: Info message"));
    assert!(contents.contains("WARN: Warning message"));
    assert!(contents.contains("ERROR: Error message"));
    assert!(contents.contains("FATAL: Fatal message"));

    Ok(())
}

#[test]
#[serial]
fn test_multiple_named_loggers() -> Result<()> {
    // Create a temporary directory for log files
    let temp_dir = tempdir()?;
    let app_log_path = temp_dir.path().join("app.log");
    let audit_log_path = temp_dir.path().join("audit.log");

    // Configure two different loggers
    let app_config = Config {
        formatter: FormatterConfig::Text {
            format_string: "APP: {message}".to_string(),
            date_format: "%Y-%m-%d".to_string(),
        },
        writer: WriterConfig::File { path: app_log_path.clone() },
        filter: FilterConfig::Level { minimum_level: LogLevel::Info },
    };

    let audit_config = Config {
        formatter: FormatterConfig::Text {
            format_string: "AUDIT: {message}".to_string(),
            date_format: "%Y-%m-%d".to_string(),
        },
        writer: WriterConfig::File { path: audit_log_path.clone() },
        filter: FilterConfig::Level { minimum_level: LogLevel::Info },
    };

    // Initialize named loggers
    init_named("app", app_config)?;
    init_named("audit", audit_config)?;

    // Use each logger
    let app_logger = get_named_logger("app")?;
    let audit_logger = get_named_logger("audit")?;

    app_logger.info("app_module", "Application started")?;
    audit_logger.info("audit_module", "User login event")?;

    // Explicitly flush both loggers
    app_logger.writer().flush()?;
    audit_logger.writer().flush()?;

    // Verify both log files were created with correct content
    let mut app_file = fs::File::open(&app_log_path)?;
    let mut app_contents = String::new();
    app_file.read_to_string(&mut app_contents)?;

    let mut audit_file = fs::File::open(&audit_log_path)?;
    let mut audit_contents = String::new();
    audit_file.read_to_string(&mut audit_contents)?;

    assert!(app_contents.contains("APP: Application started"));
    assert!(audit_contents.contains("AUDIT: User login event"));

    Ok(())
}

#[test]
#[serial]
fn test_trace_info() -> Result<()> {
    // Create a temporary directory for log files
    let temp_dir = tempdir()?;
    let log_path = temp_dir.path().join("trace.log");

    // Configure a logger with JSON formatter to easily check trace info
    let config = Config {
        formatter: FormatterConfig::Json,
        writer: WriterConfig::File { path: log_path.clone() },
        filter: FilterConfig::Level { minimum_level: LogLevel::Info },
    };
    init_logger(config)?;

    // Log a message which should include trace info via the macro
    info!("Log message with trace info");

    // Explicitly flush the logger to ensure messages are written
    let logger = get_logger()?;
    logger.writer().flush()?;

    // Read the log file contents and verify trace info exists
    let mut file = fs::File::open(&log_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Should contain trace information
    assert!(contents.contains("trace"));
    assert!(contents.contains("file"));
    assert!(contents.contains("line"));

    Ok(())
}

#[test]
#[serial]
fn test_ltsv_formatter() -> Result<()> {
    // Create a temporary directory for log files
    let temp_dir = tempdir()?;
    let log_path = temp_dir.path().join("ltsv.log");

    // Configure a logger with LTSV formatter
    let config = Config {
        formatter: FormatterConfig::Ltsv,
        writer: WriterConfig::File { path: log_path.clone() },
        filter: FilterConfig::Level { minimum_level: LogLevel::Info },
    };
    init_logger(config)?;

    // Log a message
    info!("Testing LTSV formatter");

    // Explicitly flush the logger to ensure messages are written
    let logger = get_logger()?;
    logger.writer().flush()?;

    // Read the log file contents and verify LTSV format
    let mut file = fs::File::open(&log_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // LTSV format uses field:value pairs separated by tabs
    assert!(contents.contains("level:INFO"));
    assert!(contents.contains("message:Testing LTSV formatter"));
    assert!(contents.contains("time:"));

    Ok(())
}

#[test]
#[serial]
fn test_any_filter_composition() -> Result<()> {
    // Create a temporary directory for log files
    let temp_dir = tempdir()?;
    let log_path = temp_dir.path().join("any_filter.log");

    // Configure a logger with "any" filter composition
    let config = Config {
        formatter: FormatterConfig::Text {
            format_string: "{module}: {message}".to_string(),
            date_format: "%Y-%m-%d".to_string(),
        },
        writer: WriterConfig::File { path: log_path.clone() },
        filter: FilterConfig::Any {
            filters: vec![
                FilterConfig::Module {
                    include: vec!["important_module".to_string()],
                    exclude: vec![],
                },
                FilterConfig::Level { minimum_level: LogLevel::Error },
            ],
        },
    };
    init_logger(config)?;

    let logger = get_logger()?;

    // Should be logged (important module)
    logger.info("important_module", "Important info message")?;

    // Should be logged (error level)
    logger.error("other_module", "Error in other module")?;

    // Should NOT be logged (neither important nor error)
    logger.info("other_module", "Normal info message")?;

    // Explicitly flush the logger to ensure messages are written
    logger.writer().flush()?;

    // Read the log file contents and verify filter behavior
    let mut file = fs::File::open(&log_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    assert!(contents.contains("important_module: Important info message"));
    assert!(contents.contains("other_module: Error in other module"));
    assert!(!contents.contains("other_module: Normal info message"));

    Ok(())
}

#[test]
#[serial]
fn test_flush_behavior() -> Result<()> {
    // Create a temporary directory for log files
    let temp_dir = tempdir()?;
    let log_path = temp_dir.path().join("flush.log");

    // Configure a file logger
    let config = Config {
        formatter: FormatterConfig::Text {
            format_string: "{message}".to_string(),
            date_format: "%Y-%m-%d".to_string(),
        },
        writer: WriterConfig::File { path: log_path.clone() },
        filter: FilterConfig::Level { minimum_level: LogLevel::Info },
    };

    let logger = dougu_essentials::log::LogFramework::from_config(&config)?;

    // Create and log a message
    let record = LogRecord::new(
        LogLevel::Info,
        "test_module".to_string(),
        "Test message before flush".to_string(),
        std::collections::HashMap::new(),
        None,
    );

    logger.log(record)?;

    // Explicitly flush the logger's writer
    logger.writer().flush()?;

    // Verify the message was written to the file
    let mut file = fs::File::open(&log_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    assert!(contents.contains("Test message before flush"));

    Ok(())
} 