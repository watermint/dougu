// fs module is an abstraction layer for file system operations.
// It provides traits and implementations for different types of file systems:
// - Local file systems
// - Cloud file systems
// - Pseudo file systems for services (e.g., task tracking systems)

pub mod path;
pub mod folder;

// Re-export path types for easier access
pub use path::{
    create_local_path, default_path_type, EssentialPath, LocalPath, LocalPathType, Namespace,
    Path, PathComponents, PathConverter, PathCredentials, PathProvider,
    PathResolver, PathResolverRepository, ServerInfo, StandardServerInfo,
};

// Re-export folder functions
pub use folder::ensure_folder;
