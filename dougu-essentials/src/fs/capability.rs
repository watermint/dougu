use std::fmt::Debug;
use std::collections::HashMap;

/// Defines capabilities that a file system implementation may support.
/// Used to determine which operations are available for a given file system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    /// Can read file contents
    /// 
    /// This capability indicates that the file system allows reading the contents of files.
    /// Implementations may provide methods for streaming reads, random access, or
    /// block-based reading depending on the underlying file system.
    Read,
    
    /// Can write to files
    /// 
    /// This capability indicates that the file system allows writing to files.
    /// Implementations may provide methods for streaming writes, random access writes,
    /// truncating writes, or appending writes depending on the underlying file system.
    Write,
    
    /// Can create new files and folders
    /// 
    /// This capability indicates that the file system allows creating new files and folders.
    /// This typically includes the ability to create empty files or folders, but may not
    /// include the ability to write to them (see the Write capability).
    Create,
    
    /// Can delete files and folders
    /// 
    /// This capability indicates that the file system allows deleting files and folders.
    Delete,
    
    /// Can move/rename files and folders
    /// 
    /// This capability indicates that the file system allows moving or renaming files
    /// and folders. This involves changing the path of an entry while preserving its
    /// contents and most of its metadata.
    Move,
    
    /// Can copy files and folders
    /// 
    /// This capability indicates that the file system allows copying files and folders.
    /// This creates a duplicate of an entry at a new location. Implementations may
    /// optimize this operation (e.g., server-side copy) to avoid transferring data
    /// unnecessarily.
    Copy,
    
    /// Supports seeking within files
    /// 
    /// This capability indicates that the file system allows positioning the read/write
    /// cursor at arbitrary positions within a file. This enables random access to file
    /// contents rather than just sequential reads or writes.
    Seek,
    
    /// Supports listing folder contents
    /// 
    /// This capability indicates that the file system allows enumerating the contents of
    /// folders. This typically includes the ability to list files, subfolders, and
    /// possibly special items like symbolic links or mounted volumes.
    List,
    
    /// Supports file/folder metadata retrieval
    /// 
    /// This capability indicates that the file system allows retrieving metadata about
    /// files and folders. This metadata may include size, creation time, modification time,
    /// access permissions, and file system-specific attributes.
    Metadata,
    
    /// Supports file streaming (read/write)
    /// 
    /// This capability indicates that the file system supports efficient streaming of
    /// file contents, either for reading or writing. This is particularly important for
    /// large files that may not fit in memory.
    Stream,
}

/// Defines the protocol or service type of a file system implementation.
/// This is distinct from capabilities as it describes what kind of file system it is,
/// rather than what operations it can perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolType {
    /// Local file system
    /// 
    /// This protocol type indicates that the file system accesses files on the local machine
    /// using the operating system's native file APIs.
    LocalFileSystem,
    
    /// WebDAV protocol
    /// 
    /// This protocol type indicates that the file system supports the WebDAV protocol,
    /// which extends HTTP to allow clients to perform remote web content authoring operations.
    WebDAV,
    
    /// FTP/SFTP protocols
    /// 
    /// This protocol type indicates that the file system supports File Transfer Protocol (FTP)
    /// or SSH File Transfer Protocol (SFTP) for file operations.
    FTPSFTP,
    
    /// Cloud object storage (S3, Azure Blob, GCP Storage)
    /// 
    /// This protocol type indicates that the file system can interact with cloud object
    /// storage services like Amazon S3, Azure Blob Storage, or Google Cloud Storage.
    CloudObjectStorage,
    
    /// Cloud storage service (Dropbox, Google Drive, OneDrive, Box, etc.)
    /// 
    /// This protocol type indicates that the file system interacts with a cloud storage
    /// service API. The specific service is identified through a provider identifier.
    CloudStorageService,
}

/// Defines standardized information about a file system provider.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProviderInfo {
    /// Unique identifier for the provider (e.g., "dropbox", "google_drive")
    id: String,
    
    /// Display name of the provider (e.g., "Dropbox", "Google Drive")
    display_name: String,
    
    /// Version of the provider implementation
    version: Option<String>,
    
    /// Version of the API used by this provider
    api_version: Option<String>,
    
    /// URL to the provider's website or documentation
    website_url: Option<String>,
    
    /// Category of the provider (cloud storage, local filesystem, etc.)
    category: Option<String>,
    
    /// Free-form tags describing the provider's features
    tags: Vec<String>,
    
    /// Additional custom metadata for the provider
    metadata: HashMap<String, String>,
}

impl ProviderInfo {
    /// Creates a new provider info with the given ID and display name
    pub fn new(id: String, display_name: String) -> Self {
        Self {
            id,
            display_name,
            version: None,
            api_version: None,
            website_url: None,
            category: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Sets the version of the provider implementation
    pub fn with_version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }
    
    /// Sets the API version used by this provider
    pub fn with_api_version(mut self, api_version: String) -> Self {
        self.api_version = Some(api_version);
        self
    }
    
    /// Sets the URL to the provider's website or documentation
    pub fn with_website_url(mut self, url: String) -> Self {
        self.website_url = Some(url);
        self
    }
    
    /// Sets the category of the provider
    pub fn with_category(mut self, category: String) -> Self {
        self.category = Some(category);
        self
    }
    
    /// Adds a tag to the provider
    pub fn with_tag(mut self, tag: String) -> Self {
        self.tags.push(tag);
        self
    }
    
    /// Adds multiple tags to the provider
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags.extend(tags);
        self
    }
    
    /// Adds custom metadata to the provider
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// Gets the unique identifier for the provider
    pub fn id(&self) -> &str {
        &self.id
    }
    
    /// Gets the display name of the provider
    pub fn display_name(&self) -> &str {
        &self.display_name
    }
    
    /// Gets the version of the provider implementation
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }
    
    /// Gets the API version used by this provider
    pub fn api_version(&self) -> Option<&str> {
        self.api_version.as_deref()
    }
    
    /// Gets the URL to the provider's website or documentation
    pub fn website_url(&self) -> Option<&str> {
        self.website_url.as_deref()
    }
    
    /// Gets the category of the provider
    pub fn category(&self) -> Option<&str> {
        self.category.as_deref()
    }
    
    /// Gets the tags describing the provider's features
    pub fn tags(&self) -> &[String] {
        &self.tags
    }
    
    /// Checks if the provider has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
    
    /// Gets a metadata value by key
    pub fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(|s| s.as_str())
    }
    
    /// Adds or updates a metadata value
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

/// Defines a capability set that describes what operations are supported
/// by a specific file system implementation.
#[derive(Debug, Clone, Default)]
pub struct CapabilitySet {
    capabilities: Vec<Capability>,
    protocol_types: Vec<ProtocolType>,
    provider_info: Option<ProviderInfo>,
    feature_tags: Vec<String>,
}

impl CapabilitySet {
    /// Creates a new empty capability set
    pub fn new() -> Self {
        Self {
            capabilities: Vec::new(),
            protocol_types: Vec::new(),
            provider_info: None,
            feature_tags: Vec::new(),
        }
    }

    /// Creates a new capability set with all capabilities
    pub fn all() -> Self {
        Self {
            capabilities: vec![
                Capability::Read,
                Capability::Write,
                Capability::Create,
                Capability::Delete, 
                Capability::Move,
                Capability::Copy,
                Capability::Seek,
                Capability::List,
                Capability::Metadata,
                Capability::Stream,
            ],
            protocol_types: vec![
                ProtocolType::LocalFileSystem,
                ProtocolType::WebDAV,
                ProtocolType::FTPSFTP,
                ProtocolType::CloudObjectStorage,
                ProtocolType::CloudStorageService,
            ],
            provider_info: None,
            feature_tags: Vec::new(),
        }
    }

    /// Creates a read-only capability set
    pub fn read_only() -> Self {
        Self {
            capabilities: vec![
                Capability::Read,
                Capability::Seek,
                Capability::List,
                Capability::Metadata,
                Capability::Stream,
            ],
            protocol_types: Vec::new(),
            provider_info: None,
            feature_tags: Vec::new(),
        }
    }

    /// Adds a capability to this set
    pub fn add(&mut self, capability: Capability) {
        if !self.has(capability) {
            self.capabilities.push(capability);
        }
    }

    /// Adds a protocol type to this set
    pub fn add_protocol_type(&mut self, protocol_type: ProtocolType) {
        if !self.has_protocol_type(protocol_type) {
            self.protocol_types.push(protocol_type);
        }
    }

    /// Adds a feature tag to this set
    pub fn add_feature_tag(&mut self, tag: String) {
        if !self.has_feature_tag(&tag) {
            self.feature_tags.push(tag);
        }
    }

    /// Checks if this set has the specified capability
    pub fn has(&self, capability: Capability) -> bool {
        self.capabilities.contains(&capability)
    }

    /// Checks if this set has the specified protocol type
    pub fn has_protocol_type(&self, protocol_type: ProtocolType) -> bool {
        self.protocol_types.contains(&protocol_type)
    }

    /// Checks if this set has the specified feature tag
    pub fn has_feature_tag(&self, tag: &str) -> bool {
        self.feature_tags.iter().any(|t| t == tag)
    }

    /// Checks if this set contains all the capabilities in the specified set
    pub fn has_all(&self, capabilities: &[Capability]) -> bool {
        capabilities.iter().all(|cap| self.has(*cap))
    }

    /// Checks if this set contains any of the capabilities in the specified set
    pub fn has_any(&self, capabilities: &[Capability]) -> bool {
        capabilities.iter().any(|cap| self.has(*cap))
    }

    /// Checks if this set has all of the specified feature tags
    pub fn has_all_feature_tags(&self, tags: &[&str]) -> bool {
        tags.iter().all(|&tag| self.has_feature_tag(tag))
    }

    /// Checks if this set has any of the specified feature tags
    pub fn has_any_feature_tags(&self, tags: &[&str]) -> bool {
        tags.iter().any(|&tag| self.has_feature_tag(tag))
    }

    /// Checks if this set supports any S3-compatible operations
    pub fn supports_s3(&self) -> bool {
        self.has_protocol_type(ProtocolType::CloudObjectStorage) && 
        self.has_feature_tag("s3-compatible")
    }

    /// Checks if this set supports any Azure Blob Storage operations
    pub fn supports_azure_blob(&self) -> bool {
        self.has_protocol_type(ProtocolType::CloudObjectStorage) && 
        self.has_feature_tag("azure-blob-compatible")
    }

    /// Checks if this set supports any Google Cloud Storage operations
    pub fn supports_gcp_storage(&self) -> bool {
        self.has_protocol_type(ProtocolType::CloudObjectStorage) && 
        self.has_feature_tag("gcp-storage-compatible")
    }

    /// Checks if this set supports WebDAV protocol
    pub fn supports_webdav(&self) -> bool {
        self.has_protocol_type(ProtocolType::WebDAV)
    }

    /// Checks if this set supports FTP/SFTP protocols
    pub fn supports_ftpsftp(&self) -> bool {
        self.has_protocol_type(ProtocolType::FTPSFTP)
    }

    /// Checks if this set supports cloud object storage
    pub fn supports_cloud_storage(&self) -> bool {
        self.has_protocol_type(ProtocolType::CloudObjectStorage)
    }

    /// Checks if this set supports cloud storage service
    pub fn supports_cloud_storage_service(&self) -> bool {
        self.has_protocol_type(ProtocolType::CloudStorageService)
    }

    /// Checks if this set supports Dropbox
    pub fn supports_dropbox(&self) -> bool {
        self.is_provider("dropbox")
    }
    
    /// Checks if this set supports Google Drive
    pub fn supports_google_drive(&self) -> bool {
        self.is_provider("google_drive")
    }
    
    /// Checks if this set supports OneDrive
    pub fn supports_onedrive(&self) -> bool {
        self.is_provider("onedrive")
    }
    
    /// Checks if this set supports Box
    pub fn supports_box(&self) -> bool {
        self.is_provider("box")
    }
    
    /// Checks if this set supports local file system
    pub fn supports_local_file_system(&self) -> bool {
        self.has_protocol_type(ProtocolType::LocalFileSystem)
    }

    /// Sets the provider information for this capability set
    pub fn set_provider_info(&mut self, provider_info: ProviderInfo) {
        self.provider_info = Some(provider_info);
    }

    /// Gets the provider information for this capability set
    pub fn provider_info(&self) -> Option<&ProviderInfo> {
        self.provider_info.as_ref()
    }
    
    /// Gets the provider ID if available
    pub fn provider_id(&self) -> Option<&str> {
        self.provider_info.as_ref().map(|info| info.id())
    }
    
    /// Checks if the provider matches the given ID
    pub fn is_provider(&self, provider_id: &str) -> bool {
        self.provider_info.as_ref().map_or(false, |info| info.id() == provider_id)
    }
    
    /// Gets provider metadata by key
    pub fn get_provider_metadata(&self, key: &str) -> Option<&str> {
        self.provider_info.as_ref().and_then(|info| info.get_metadata(key))
    }
    
    /// Checks if the provider has a specific tag
    pub fn provider_has_tag(&self, tag: &str) -> bool {
        self.provider_info.as_ref().map_or(false, |info| info.has_tag(tag))
    }
} 