use super::resolver::PathEnum;
use crate::core::error;
use crate::fs::path::local::posix::PosixLocalPath;
use crate::fs::path::local::{LocalPath, LocalPathType};
use crate::fs::path::default::{DefaultNamespace, DefaultPathComponents};

/// Create a path in the current OS format
pub fn create_os_path(path: &str) -> error::Result<PathEnum> {
    match std::env::consts::OS {
        "windows" => {
            let path = PosixLocalPath::create_os_path(path)?;
            Ok(PathEnum::Posix(path))
        }
        _ => {
            let path = PosixLocalPath::create_os_path(path)?;
            Ok(PathEnum::Posix(path))
        }
    }
}

/// Factory function for creating a local path with the OS-specific implementation
pub fn create_local_path(path: &str) -> error::Result<PathEnum> {
    match std::env::consts::OS {
        "windows" => {
            let path = PosixLocalPath::create_os_path(path)?;
            Ok(PathEnum::Posix(path))
        }
        _ => {
            let path = PosixLocalPath::create_os_path(path)?;
            Ok(PathEnum::Posix(path))
        }
    }
}

/// Convert a PathEnum to a boxed LocalPath
pub fn path_enum_to_boxed_local_path(path_enum: PathEnum) -> Box<dyn LocalPath<ComponentsType=DefaultPathComponents, NamespaceType=DefaultNamespace> + 'static> {
    match path_enum {
        PathEnum::Posix(path) => Box::new(path),
        PathEnum::Windows(path) => Box::new(path),
        PathEnum::UNC(path) => Box::new(path),
        PathEnum::NFS(path) => Box::new(path),
        PathEnum::SMB(path) => Box::new(path),
        PathEnum::Essential(path) => {
            // Use the Path trait method to_string instead of calling directly
            let path_str = super::core::Path::to_string(&path);
            match std::env::consts::OS {
                "windows" => {
                    let path = PosixLocalPath::create_os_path(&path_str).unwrap();
                    Box::new(path) 
                },
                _ => {
                    let path = PosixLocalPath::create_os_path(&path_str).unwrap();
                    Box::new(path)
                }
            }
        }
    }
}

/// Get the default local path type for the current OS
pub fn default_path_type() -> LocalPathType {
    #[cfg(target_family = "unix")]
    {
        LocalPathType::PosixPath
    }

    #[cfg(target_family = "windows")]
    {
        LocalPathType::WindowsPath
    }

    #[cfg(not(any(target_family = "unix", target_family = "windows")))]
    {
        // Default to POSIX paths as a fallback
        LocalPathType::PosixPath
    }
} 