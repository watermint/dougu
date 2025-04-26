use std::sync::Arc;
use crate::core::error;
use super::core::{Path, Namespace};
use super::essential::EssentialPath;
use super::local::posix::PosixLocalPath;
use super::local::windows::WindowsLocalPath;
use super::local::unc::UNCLocalPath;
use super::local::nfs::NFSLocalPath;
use super::local::smb::SMBLocalPath;
use std::fmt::Debug;

/// Enum of all possible Path implementations
#[derive(Debug)]
pub enum PathEnum {
    Essential(EssentialPath),
    Posix(PosixLocalPath),
    Windows(WindowsLocalPath),
    UNC(UNCLocalPath),
    NFS(NFSLocalPath),
    SMB(SMBLocalPath),
}

impl PathEnum {
    pub fn to_string(&self) -> String {
        match self {
            PathEnum::Essential(p) => p.to_string(),
            PathEnum::Posix(p) => p.to_string(),
            PathEnum::Windows(p) => p.to_string(),
            PathEnum::UNC(p) => p.to_string(),
            PathEnum::NFS(p) => p.to_string(),
            PathEnum::SMB(p) => p.to_string(),
        }
    }
}

/// PathResolver is responsible for resolving and converting paths for a specific service or system
pub trait PathResolver: Send + Sync {
    /// Get the unique ID for this resolver
    fn id(&self) -> &str;
    
    /// Check if this resolver can handle the given namespace
    fn can_resolve(&self, namespace: &str) -> bool;
    
    /// Convert an EssentialPath to a specific implementation
    fn resolve(&self, path: &EssentialPath) -> error::Result<PathEnum>;
    
    /// Convert a specific path implementation back to an EssentialPath
    fn to_essential_path(&self, path: &PathEnum) -> error::Result<EssentialPath>;
}

/// PathResolverRepository manages a collection of path resolvers and provides path resolution services
pub struct PathResolverRepository {
    resolvers: Vec<Box<dyn PathResolver>>,
    local_resolver: Option<Box<dyn PathResolver>>,
}

impl PathResolverRepository {
    /// Create a new empty repository
    pub fn new() -> Self {
        PathResolverRepository {
            resolvers: Vec::new(),
            local_resolver: None,
        }
    }
    
    /// Register a new path resolver
    pub fn register_resolver(&mut self, resolver: Box<dyn PathResolver>) {
        self.resolvers.push(resolver);
    }
    
    /// Set the local path resolver
    pub fn set_local_resolver(&mut self, resolver: Box<dyn PathResolver>) {
        self.local_resolver = Some(resolver);
    }
    
    /// Resolve an EssentialPath to a specific path implementation
    pub fn resolve(&self, path: &EssentialPath) -> error::Result<PathEnum> {
        let namespace = path.namespace().as_str();
        
        // If namespace is empty or resolvers list is empty, use local resolver if available
        if namespace.is_empty() || self.resolvers.is_empty() {
            if let Some(local_resolver) = &self.local_resolver {
                return local_resolver.resolve(path);
            } else {
                return Err(error::Error::msg(
                    format!("No local resolver registered")
                ));
            }
        }
        
        // Find a resolver that can handle this namespace
        for resolver in &self.resolvers {
            if resolver.can_resolve(namespace) {
                return resolver.resolve(path);
            }
        }
        
        // If no resolver found, try the local resolver as fallback
        if let Some(local_resolver) = &self.local_resolver {
            return local_resolver.resolve(path);
        }
        
        Err(error::Error::msg(
            format!("No resolver found for namespace: {}", namespace)
        ))
    }
    
    /// Convert a specific path implementation back to an EssentialPath
    pub fn to_essential_path(&self, path: &PathEnum) -> error::Result<EssentialPath> {
        // Try to find a resolver that can handle this path type
        for resolver in &self.resolvers {
            if let Ok(essential_path) = resolver.to_essential_path(path) {
                return Ok(essential_path);
            }
        }
        
        // Try local resolver as fallback
        if let Some(local_resolver) = &self.local_resolver {
            return local_resolver.to_essential_path(path);
        }
        
        Err(error::Error::msg(
            format!("No resolver can convert this path")
        ))
    }
    
    /// Get a resolver by ID
    pub fn get_resolver(&self, id: &str) -> Option<&dyn PathResolver> {
        for resolver in &self.resolvers {
            if resolver.id() == id {
                return Some(resolver.as_ref());
            }
        }
        None
    }
} 