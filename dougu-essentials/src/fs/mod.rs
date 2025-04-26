// fs module is an abstraction layer for file system operations.
// It provides traits and implementations for different types of file systems:
// - Local file systems
// - Cloud file systems
// - Pseudo file systems for services (e.g., task tracking systems)

pub mod path;

// Re-export path types for easier access
pub use path::{
    Path, PathComponents, Namespace, LocalPath, LocalPathType, PathCredentials,
    PathProvider, EssentialPath, PathConverter, PathResolver, PathResolverRepository,
    ServerInfo, StandardServerInfo, create_local_path, default_path_type,
}; 