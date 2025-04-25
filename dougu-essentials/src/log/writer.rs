use std::fs::{File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::core::error::Result;
use crate::fs::ensure_dir;
use crate::log::formatter::Formatter;
use crate::log::interface::LogRecord;
use crate::time::ZonedDateTime;

/// Writer trait for writing formatted log records
pub trait Writer: Send + Sync {
    /// Write a formatted log record
    fn write(&self, record: &LogRecord, formatter: &dyn Formatter) -> Result<()>;

    /// Flush any buffered data
    fn flush(&self) -> Result<()>;
}

/// Console writer that writes to stdout or stderr
pub struct ConsoleWriter {
    /// Whether to write to stderr (true) or stdout (false)
    stderr: bool,
    /// Buffer for writing
    buffer: Mutex<Vec<u8>>,
}

impl ConsoleWriter {
    /// Create a new console writer
    pub fn new(stderr: bool) -> Self {
        Self {
            stderr,
            buffer: Mutex::new(Vec::with_capacity(4096)),
        }
    }

    /// Create a console writer that writes to stdout
    pub fn stdout() -> Self {
        Self::new(false)
    }

    /// Create a console writer that writes to stderr
    pub fn stderr() -> Self {
        Self::new(true)
    }
}

impl Writer for ConsoleWriter {
    fn write(&self, record: &LogRecord, formatter: &dyn Formatter) -> Result<()> {
        let formatted = formatter.format(record)?;
        let mut buffer = self.buffer.lock()
            .map_err(|_| crate::core::error::error("Failed to lock buffer"))?;

        buffer.extend_from_slice(&formatted);
        buffer.push(b'\n');

        if buffer.len() >= 4096 {
            self.flush_buffer(&mut buffer)?;
        }

        Ok(())
    }

    fn flush(&self) -> Result<()> {
        let mut buffer = self.buffer.lock()
            .map_err(|_| crate::core::error::error("Failed to lock buffer"))?;
        self.flush_buffer(&mut buffer)
    }
}

impl ConsoleWriter {
    fn flush_buffer(&self, buffer: &mut Vec<u8>) -> Result<()> {
        if buffer.is_empty() {
            return Ok(());
        }

        if self.stderr {
            io::stderr().write_all(&buffer)?;
            io::stderr().flush()?;
        } else {
            io::stdout().write_all(&buffer)?;
            io::stdout().flush()?;
        }

        buffer.clear();
        Ok(())
    }
}

/// File writer that writes to a file
pub struct FileWriter {
    /// File path
    path: PathBuf,
    /// File handle
    file: Mutex<BufWriter<File>>,
}

impl FileWriter {
    /// Create a new file writer
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_buf = path.as_ref().to_path_buf();

        // Ensure parent directory exists
        if let Some(parent) = path_buf.parent() {
            ensure_dir(parent)?;
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path_buf)?;

        Ok(Self {
            path: path_buf,
            file: Mutex::new(BufWriter::new(file)),
        })
    }

    /// Get the path of the file
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Writer for FileWriter {
    fn write(&self, record: &LogRecord, formatter: &dyn Formatter) -> Result<()> {
        let formatted = formatter.format(record)?;
        let mut file = self.file.lock()
            .map_err(|_| crate::core::error::error("Failed to lock file"))?;

        file.write_all(&formatted)?;
        file.write_all(b"\n")?;

        Ok(())
    }

    fn flush(&self) -> Result<()> {
        let mut file = self.file.lock()
            .map_err(|_| crate::core::error::error("Failed to lock file"))?;
        file.flush()?;
        Ok(())
    }
}

/// Rotate writer that rotates files based on size or time
pub struct RotateWriter {
    /// Base path for log files
    base_path: PathBuf,
    /// Current writer
    current_writer: Mutex<FileWriter>,
    /// Maximum file size before rotation (in bytes)
    max_size: u64,
    /// Current file size (approximate)
    current_size: Mutex<u64>,
    /// Maximum number of log files to keep
    max_files: usize,
    /// Last rotation time
    last_rotation: Mutex<ZonedDateTime>,
    /// Rotation period (in seconds, 0 means no time-based rotation)
    rotation_period: u64,
}

impl RotateWriter {
    /// Create a new rotate writer
    pub fn new<P: AsRef<Path>>(
        base_path: P,
        max_size: u64,
        max_files: usize,
        rotation_period: u64,
    ) -> Result<Self> {
        let base_path_buf = base_path.as_ref().to_path_buf();
        let current_writer = FileWriter::new(&base_path_buf)?;

        // Get current file size if it exists
        let current_size = match base_path_buf.metadata() {
            Ok(metadata) => metadata.len(),
            Err(_) => 0,
        };

        Ok(Self {
            base_path: base_path_buf,
            current_writer: Mutex::new(current_writer),
            max_size,
            current_size: Mutex::new(current_size),
            max_files,
            last_rotation: Mutex::new(ZonedDateTime::now()),
            rotation_period,
        })
    }

    /// Check if rotation is needed based on size or time
    fn check_rotation(&self) -> Result<bool> {
        // Check size-based rotation
        let current_size = *self.current_size.lock()
            .map_err(|_| crate::core::error::error("Failed to lock current_size"))?;
        if self.max_size > 0 && current_size >= self.max_size {
            return Ok(true);
        }

        // Check time-based rotation (if enabled)
        if self.rotation_period > 0 {
            let last_rotation = self.last_rotation.lock()
                .map_err(|_| crate::core::error::error("Failed to lock last_rotation"))?
                .clone();

            let now = ZonedDateTime::now();

            // Calculate seconds elapsed since last rotation
            let elapsed = now.to_unix() - last_rotation.to_unix();
            if elapsed >= self.rotation_period as i64 {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Rotate log files
    fn rotate(&self) -> Result<()> {
        // Get current time
        let now = ZonedDateTime::now();

        // Create a rotation timestamp for the filename
        let timestamp = now.format();

        // Create a rotated filename with timestamp
        let mut rotated_path = self.base_path.clone();
        let base_filename = self.base_path.file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "log".to_string());

        let extension = self.base_path.extension()
            .map(|s| format!(".{}", s.to_string_lossy()))
            .unwrap_or_else(|| "".to_string());

        // Remove extension from base name if present
        let base_name = if extension.is_empty() {
            base_filename
        } else {
            base_filename[0..base_filename.len() - extension.len()].to_string()
        };

        let rotated_filename = format!("{}.{}{}", base_name, timestamp, extension);
        rotated_path.set_file_name(rotated_filename);

        // Rename current log file to rotated filename
        if self.base_path.exists() {
            std::fs::rename(&self.base_path, &rotated_path)?;
        }

        // Create a new writer for the original filename
        let new_writer = FileWriter::new(&self.base_path)?;
        let mut writer_guard = self.current_writer.lock()
            .map_err(|_| crate::core::error::error("Failed to lock current_writer"))?;
        *writer_guard = new_writer;

        // Reset current size and update last rotation time
        let mut size_guard = self.current_size.lock()
            .map_err(|_| crate::core::error::error("Failed to lock current_size"))?;
        *size_guard = 0;

        let mut time_guard = self.last_rotation.lock()
            .map_err(|_| crate::core::error::error("Failed to lock last_rotation"))?;
        *time_guard = now.clone();

        // Clean up old files if needed
        self.remove_old_files()?;

        Ok(())
    }

    /// Remove old log files based on max_files setting
    fn remove_old_files(&self) -> Result<()> {
        let base_str = self.base_path.to_string_lossy().to_string();
        let dir = self.base_path.parent().unwrap_or_else(|| Path::new("."));

        // Find all rotated files
        let mut rotated_files = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let file_name = path.to_string_lossy().to_string();
            if file_name.starts_with(&base_str) && file_name != base_str {
                rotated_files.push(path);
            }
        }

        // Sort by modification time (oldest first)
        rotated_files.sort_by(|a, b| {
            let a_time = a.metadata().and_then(|m| m.modified()).ok();
            let b_time = b.metadata().and_then(|m| m.modified()).ok();
            a_time.cmp(&b_time)
        });

        // Remove excess files
        if rotated_files.len() >= self.max_files {
            let to_remove = rotated_files.len() - self.max_files + 1;
            for path in rotated_files.iter().take(to_remove) {
                std::fs::remove_file(path)?;
            }
        }

        Ok(())
    }
}

impl Writer for RotateWriter {
    fn write(&self, record: &LogRecord, formatter: &dyn Formatter) -> Result<()> {
        // Check if rotation is needed
        if self.check_rotation()? {
            self.rotate()?;
        }

        // Format and write the record
        let formatted = formatter.format(record)?;
        let writer_guard = self.current_writer.lock()
            .map_err(|_| crate::core::error::error("Failed to lock current_writer"))?;

        writer_guard.write(record, formatter)?;

        // Update current size
        let mut size_guard = self.current_size.lock()
            .map_err(|_| crate::core::error::error("Failed to lock current_size"))?;
        *size_guard += formatted.len() as u64 + 1; // +1 for newline

        Ok(())
    }

    fn flush(&self) -> Result<()> {
        let writer_guard = self.current_writer.lock()
            .map_err(|_| crate::core::error::error("Failed to lock current_writer"))?;
        writer_guard.flush()
    }
}

/// Compress writer that compresses logs before passing them to the next writer
pub struct CompressWriter<W: Writer> {
    /// The next writer in the chain
    next_writer: W,
    /// Compression level (0-9, where 0 is no compression and 9 is maximum compression)
    compression_level: u32,
    /// Buffer for compressed data
    buffer: Mutex<Vec<u8>>,
}

impl<W: Writer> CompressWriter<W> {
    /// Create a new compress writer
    pub fn new(next_writer: W, compression_level: u32) -> Self {
        let compression_level = compression_level.min(9);
        Self {
            next_writer,
            compression_level,
            buffer: Mutex::new(Vec::with_capacity(4096)),
        }
    }
}

impl<W: Writer> Writer for CompressWriter<W> {
    fn write(&self, record: &LogRecord, formatter: &dyn Formatter) -> Result<()> {
        // Compress the data and send it to the next writer
        // Note: In a real implementation, this would use a compression library
        // like flate2 or zstd to compress the data. For simplicity, we'll just
        // pass it through to the next writer.
        self.next_writer.write(record, formatter)
    }

    fn flush(&self) -> Result<()> {
        self.next_writer.flush()
    }
}

/// Composite writer that writes to multiple writers
pub struct CompositeWriter {
    /// The writers to write to
    writers: Vec<Arc<dyn Writer>>,
}

impl CompositeWriter {
    /// Create a new composite writer
    pub fn new(writers: Vec<Arc<dyn Writer>>) -> Self {
        Self { writers }
    }

    /// Alternative constructor that takes Box<dyn Writer> and converts to Arc
    pub fn from_boxed(writers: Vec<Box<dyn Writer>>) -> Self {
        let arc_writers = writers.into_iter()
            .map(|w| {
                let writer: Arc<dyn Writer> = Arc::new(BoxedWriter(w));
                writer
            })
            .collect();

        Self { writers: arc_writers }
    }

    /// Add a writer to the composite
    pub fn add_writer(&mut self, writer: Arc<dyn Writer>) {
        self.writers.push(writer);
    }
}

impl Writer for CompositeWriter {
    fn write(&self, record: &LogRecord, formatter: &dyn Formatter) -> Result<()> {
        let mut last_error = None;

        // Write to all writers
        for writer in &self.writers {
            if let Err(err) = writer.write(record, formatter) {
                last_error = Some(err);
            }
        }

        // Return the last error if any
        if let Some(err) = last_error {
            Err(err)
        } else {
            Ok(())
        }
    }

    fn flush(&self) -> Result<()> {
        let mut last_error = None;

        // Flush all writers
        for writer in &self.writers {
            if let Err(err) = writer.flush() {
                last_error = Some(err);
            }
        }

        // Return the last error if any
        if let Some(err) = last_error {
            Err(err)
        } else {
            Ok(())
        }
    }
}

/// Helper struct to convert Box<dyn Writer> to Arc<dyn Writer>
pub struct BoxedWriter(pub Box<dyn Writer>);

impl Writer for BoxedWriter {
    fn write(&self, record: &LogRecord, formatter: &dyn Formatter) -> Result<()> {
        self.0.write(record, formatter)
    }

    fn flush(&self) -> Result<()> {
        self.0.flush()
    }
} 