// dougu-essentials::fs module
//
// This module provides an abstraction layer for file system operations,
// supporting various types of file systems including:
// - Local file systems (full CRUD operations)
// - Cloud file systems (with link support)
// - Read-only file systems (like CD-ROM or service abstractions)

pub mod path;

// Re-exports for common types
pub use path::{Path, PathBuf, PathComponents, Namespace}; 