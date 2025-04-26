# File System Abstraction Overview

The `dougu-essentials` file system abstraction provides a unified interface for interacting with various file systems, both local and cloud-based. This document provides an overview of the core concepts and components.

## Core Concepts

### Entry Types

The abstraction is built around the concept of "entries" which can be either files or folders:

- `Entry`: Base trait for all file system entries
- `FileEntry`: Represents files with read/write operations
- `FolderEntry`: Represents folders with listing and navigation operations

### File Statuses

File entries can exist in different states across providers:

- `Active`: Normal, accessible files
- `Deleted`: Files in trash/recycle bin that can be restored
- `PendingDeletion`: Files marked for deletion but not yet processed
- `PermanentlyDeleted`: Files that have been permanently removed

### Protocol Types

The abstraction defines different protocol or service types to represent what kind of file system is being used:

- `LocalFileSystem`: Native operating system file access
- `WebDAV`: Web-based Distributed Authoring and Versioning protocol
- `FTPSFTP`: File Transfer Protocol and SSH File Transfer Protocol
- `CloudObjectStorage`: Cloud object storage services (S3, Azure, GCP)
- `CloudStorageService`: Cloud storage service with specific provider identification

### Provider Identification

The abstraction uses a structured provider identification system through the `ProviderInfo` struct:

- **Basic Identification**
  - `id`: Unique identifier string (e.g., "dropbox", "google_drive")
  - `display_name`: Human-readable name (e.g., "Dropbox", "Google Drive")

- **Version Information**
  - `version`: Provider implementation version
  - `api_version`: Version of the API used

- **Categorization**
  - `category`: Provider category (e.g., "cloud_storage_service", "local_file_system")
  - `tags`: Collection of feature tags (e.g., "sharing", "versioning", "enterprise")

- **Documentation**
  - `website_url`: Link to the provider's website or documentation

- **Extensibility**
  - `metadata`: Custom key-value pairs for additional information

This flexible identification system allows specific provider implementations to be defined outside the core framework, enabling extensibility and modularity. Each provider implementation can create its own `ProviderInfo` instances with the appropriate details for that specific provider.

### Capabilities

The abstraction uses a capability-based approach to describe what features are supported by each file system:

- `Capability`: Basic file system operations (read, write, delete, etc.)
- `SharedFolderCapability`: Features specific to shared folders
- `ShareLinkCapability`: Features for sharing links/URLs
- `VersioningCapability`: Features for file versioning systems
- `ContentHashCapability`: Features for content integrity verification
- `WebDAVCapability`: Features specific to WebDAV protocol
- `FTPSFTPCapability`: Features specific to FTP/SFTP protocols
- `CloudStorageCapability`: Features specific to cloud object storage services

#### Trash Management Capabilities

The file system abstraction provides specialized capabilities for trash/recycle bin operations:

- `TrashManagement`: Basic trash functionality
- `PermanentDeletion`: Support for bypassing or removing from trash
- `FileRestoration`: Support for restoring files from trash
- `EmptyTrash`: Support for emptying the entire trash
- `ListTrash`: Support for listing deleted items
- `TrashMetadata`: Support for metadata about deleted items

These capabilities allow applications to programmatically handle the entire lifecycle of files, including proper trash management according to each provider's implementation. See the provider-specific documentation for detailed implementation information.

#### Protocol-Specific Capabilities

In addition to general capabilities, the abstraction provides protocol-specific capability sets that describe features unique to particular protocol types:

- **WebDAV Capabilities**: Features specific to WebDAV implementations
  - Compliance levels (Class 1, 2, 3)
  - Extensions like CalDAV, CardDAV for specialized data
  - Access control and search capabilities
  - Network drive mapping for seamless integration

- **FTP/SFTP Capabilities**: Features specific to file transfer protocols
  - Basic and extended FTP commands
  - Secure variants (FTPS, SFTP)
  - Advanced features like file locking, transfer resume
  - Authentication methods including public key

- **Cloud Object Storage Capabilities**: Features specific to object storage services
  - Common features shared across providers:
    - Multipart uploads for large files
    - Server-side encryption and customer-managed keys
    - Object lifecycle management
    - Tagging and versioning
  - Provider-specific implementations for:
    - Amazon S3 and S3-compatible services
    - Microsoft Azure Blob Storage
    - Google Cloud Storage

## Provider Implementations

This abstraction supports multiple file system providers, each with their own implementation details:

- [Dropbox](./dropbox.md): Block-based storage with strong sharing features
- [Google Drive](./google-drive.md): Document-focused with collaborative editing
- [OneDrive](./onedrive.md): Microsoft ecosystem integration with SharePoint features
- [Box](./box.md): Enterprise-focused with strong security and compliance
- [Local File System](./local.md): Access to native operating system file operations

## Protocol Support

The abstraction supports standard file transfer and management protocols:

- [WebDAV](./webdav.md): Web-based Distributed Authoring and Versioning protocol
- [FTP/SFTP](./ftp-sftp.md): File Transfer Protocol and SSH File Transfer Protocol

## Cloud Object Storage Support

In addition to traditional file systems, the abstraction supports cloud object storage services:

- [Cloud Storage](./cloud-storage.md): Support for S3, Azure Blob Storage, and Google Cloud Storage

Each provider's documentation includes detailed information about how it implements the various capabilities, including:

- Trash management and retention policies
- File addressing mechanisms
- Content hashing approaches
- Sharing features and controls

Refer to the provider-specific documentation to understand how each capability is implemented by that provider.

## Common Operations

The abstraction provides unified methods for common operations:

- File reading/writing
- Directory listing
- Metadata access
- Share link creation
- Trash management (delete, restore, permanent deletion)
- Protocol-specific operations (WebDAV, FTP/SFTP)
- Cloud storage operations (object manipulation, lifecycle management)

By abstracting these operations, applications can work with different file systems without needing to understand the specific implementation details of each provider.

## Integration

To check for protocol support and capabilities, the abstraction provides helper methods:

```rust
// Create a capability set with provider information
let mut capabilities = CapabilitySet::new();
capabilities.add_protocol_type(ProtocolType::CloudStorageService);

// Set up provider info
let provider_info = ProviderInfo::new(
    "dropbox".to_string(),
    "Dropbox".to_string()
)
.with_category("cloud_storage_service".to_string())
.with_tags(vec!["block_based".to_string(), "sharing".to_string()])
.with_website_url("https://www.dropbox.com".to_string());

capabilities.set_provider_info(provider_info);

// Check protocol type
if capabilities.supports_webdav() {
    // WebDAV protocol is supported
}

if capabilities.supports_cloud_storage() {
    // Cloud object storage is supported
}

if capabilities.supports_cloud_storage_service() {
    // Some cloud storage service is supported
    
    // Check specific provider
    if capabilities.supports_dropbox() {
        // Dropbox is supported
    }
    
    // Access provider information
    if let Some(provider_info) = capabilities.provider_info() {
        println!("Using provider: {} ({})", 
            provider_info.display_name(),
            provider_info.id());
            
        // Access provider tags
        if provider_info.has_tag("enterprise") {
            // This is an enterprise-grade provider
        }
        
        // Access provider metadata
        if let Some(region) = provider_info.get_metadata("region") {
            println!("Provider region: {}", region);
        }
    }
}

// Check specific capabilities
if capabilities.has(Capability::Read) {
    // File reading is supported
}

if capabilities.has_webdav_capability(WebDAVCapability::CalDAV) {
    // Calendar operations are supported via WebDAV
}

if capabilities.has_cloud_storage_capability(CloudStorageCapability::MultipartUpload) {
    // Large file uploads can be done in parts
}
```

The separation between the capability framework and concrete provider implementations allows for a modular and extensible design, where specific file system providers can be implemented in separate modules without modifying the core framework. 