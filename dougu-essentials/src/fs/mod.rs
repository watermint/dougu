// fs module is an abstraction layer for file system operations.
//
// ## Objectives
// 
// * Abstract file systems (local file systems, cloud storage, or other pseudo file systems)
// * Make it easier to handle files as client without thinking about differences of file systems
// * Provide implementation for local file systems
//
// This module provides traits and implementations for different types of file systems:
// - Local file systems
// - Cloud file systems
// - Pseudo file systems for services (e.g., task tracking systems)

pub mod path;
pub mod folder;
pub mod entry;
pub mod provider;
pub mod capability;
pub mod local;
pub mod cloud;

// Re-export path types for easier access
pub use path::{
    create_local_path, default_path_type, EssentialPath, LocalPath, LocalPathType, Namespace,
    Path, PathComponents, PathConverter, PathCredentials, PathProvider,
    PathResolver, PathResolverRepository, ServerInfo, StandardServerInfo,
};

// Re-export folder functions
pub use folder::ensure_folder;

// Re-export file system capability types
pub use capability::{Capability, CapabilitySet};

// Re-export entry types
pub use entry::{
    Entry, EntryMetadata, EntryType, FileEntry, FolderEntry,
    ReadWriteFile, ReadableFile, WritableFile,
};

// Re-export provider types
pub use provider::{
    FileSystemProvider, FileSystemProviderRepository,
};

// Re-export local file system implementations
pub use local::{
    LocalEntry, LocalEntryMetadata, LocalFileEntry, LocalFolderEntry,
    LocalFileSystemProvider, LocalReadWriteFile, LocalReadableFile, LocalWritableFile,
};
