mod local;

pub use local::LocalContentHashProvider;

/// Supported hash algorithms for content hashing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    /// MD5 hash algorithm
    MD5,
    /// SHA-1 hash algorithm
    SHA1,
    /// SHA-256 hash algorithm
    SHA256,
    /// SHA-512 hash algorithm
    SHA512,
}

/// Source to hash (whole file or blocks)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashSource {
    /// Hash the entire file as a single unit
    WholeFile,
    /// Hash the file in blocks, using the specified block size
    Blocks(usize),
}

/// Provider for content hash calculations
pub trait ContentHashProvider: Send + Sync {
    /// Computes a hash for the specified file
    fn compute_hash(&self, path: &dyn AsRef<std::path::Path>, algorithm: HashAlgorithm, source: HashSource) -> crate::core::error::Result<String>;
    
    /// Verifies a file against a provided hash
    fn verify_hash(&self, path: &dyn AsRef<std::path::Path>, algorithm: HashAlgorithm, source: HashSource, hash: &str) -> crate::core::error::Result<bool>;
}

/// Represents version information for a file
#[derive(Debug, Clone)]
pub struct VersionInfo {
    /// Version identifier
    pub id: String,
    /// Optional time when this version was created
    pub created: Option<crate::time::ZonedDateTime>,
    /// Optional user who created this version
    pub author: Option<String>,
    /// Optional size of this version in bytes
    pub size: Option<u64>,
    /// Optional content hash for this version
    pub content_hash: Option<String>,
    /// Optional comment or description for this version
    pub comment: Option<String>,
}

/// Provider for version-related operations
pub trait VersionProvider: Send + Sync {
    /// Gets all versions of a file
    fn get_versions(&self, path: &dyn AsRef<std::path::Path>) -> crate::core::error::Result<Vec<VersionInfo>>;
    
    /// Gets a specific version of a file
    fn get_version(&self, path: &dyn AsRef<std::path::Path>, version_id: &str) -> crate::core::error::Result<Option<VersionInfo>>;
    
    /// Reverts a file to a specific version
    fn revert_to_version(&self, path: &dyn AsRef<std::path::Path>, version_id: &str) -> crate::core::error::Result<()>;
    
    /// Creates a new version of a file
    fn create_version(&self, path: &dyn AsRef<std::path::Path>, comment: Option<&str>) -> crate::core::error::Result<VersionInfo>;
    
    /// Deletes a specific version of a file
    fn delete_version(&self, path: &dyn AsRef<std::path::Path>, version_id: &str) -> crate::core::error::Result<()>;
}

/// Local version provider implementation that uses a .versions directory
pub struct LocalVersionProvider {
    base_dir: std::path::PathBuf,
    hash_provider: LocalContentHashProvider,
}

impl LocalVersionProvider {
    /// Creates a new local version provider with the given base directory
    pub fn new(base_dir: std::path::PathBuf) -> Self {
        Self {
            base_dir,
            hash_provider: LocalContentHashProvider::new(),
        }
    }
} 