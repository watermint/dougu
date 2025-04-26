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
    /// Depending on the implementation, this may move items to trash or delete them
    /// permanently. See TrashManagement for more details on trash handling.
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
    
    /// Supports sharing folders with specific users or groups
    /// 
    /// This capability indicates that the file system allows sharing folders with
    /// specific users or groups, often with different permission levels. This is
    /// commonly found in cloud storage systems but may not be available in local
    /// file systems.
    SharedFolder,
    
    /// Supports generating sharing links for files/folders
    /// 
    /// This capability indicates that the file system can generate links that allow
    /// access to files or folders without requiring authentication. These links can
    /// typically be shared with others to grant them access to the file or folder.
    ShareLink,
    
    /// Supports versioning of files
    /// 
    /// This capability indicates that the file system maintains a history of versions
    /// for files. This allows accessing previous versions of a file, and possibly
    /// restoring a file to a previous version.
    Versioning,
    
    /// Supports content hashing
    /// 
    /// This capability indicates that the file system provides or supports
    /// cryptographic hashes of file contents. These hashes can be used to verify
    /// file integrity or check for changes.
    ContentHash,
    
    /// Supports access control lists or permissions
    /// 
    /// This capability indicates that the file system supports fine-grained access
    /// control mechanisms beyond basic read/write permissions. This may include
    /// user/group-based permissions, role-based access control, or ACLs.
    AccessControl,
    
    /// Supports extended attributes
    /// 
    /// This capability indicates that the file system supports storing additional
    /// metadata beyond the standard file attributes. These extended attributes are
    /// typically key-value pairs associated with files or folders.
    ExtendedAttributes,
    
    /// Supports locking files for exclusive access
    /// 
    /// This capability indicates that the file system supports mechanisms for obtaining
    /// exclusive access to files. This prevents concurrent modifications by multiple
    /// processes or users, helping to avoid conflicts.
    Locking,
    
    /// Supports encrypted storage
    /// 
    /// This capability indicates that the file system supports storing files in an
    /// encrypted format. This may involve encryption at rest, in transit, or both,
    /// and may use various encryption algorithms and key management approaches.
    Encryption,
    
    /// Supports custom sharing options (expiration, password, permissions)
    /// 
    /// This capability indicates that the file system supports advanced options when
    /// sharing files, such as setting expiration dates, requiring passwords, or
    /// specifying granular permissions. This is commonly found in cloud storage systems.
    SharingOptions,
    
    /// Supports team folders for business/enterprise
    /// 
    /// This capability indicates that the file system supports special folders that
    /// are shared among team members in an organization. These often have different
    /// permission models and administrative controls compared to personal folders.
    TeamFolder,
    
    /// Supports trash/recycle bin functionality for deleted files
    /// 
    /// This capability indicates that the file system provides a way to temporarily store
    /// deleted files before they are permanently removed. Files in trash can typically be 
    /// restored or permanently deleted.
    TrashManagement,
    
    /// Supports permanent deletion of files, bypassing or removing from trash
    /// 
    /// This capability indicates that the file system allows permanently deleting files,
    /// either directly (bypassing trash) or by removing them from the trash/recycle bin.
    PermanentDeletion,
    
    /// Supports restoring files from trash/recycle bin
    /// 
    /// This capability indicates that the file system allows recovering deleted files
    /// from the trash/recycle bin.
    FileRestoration,
    
    /// Supports emptying trash/recycle bin
    /// 
    /// This capability indicates that the file system allows emptying the entire 
    /// trash/recycle bin in one operation.
    EmptyTrash,
    
    /// Supports listing deleted/trashed files
    /// 
    /// This capability indicates that the file system allows viewing and enumerating
    /// files that are in the trash/recycle bin.
    ListTrash,
    
    /// Supports metadata for deleted items including deletion time
    /// 
    /// This capability indicates that the file system provides additional metadata
    /// for deleted files, such as when they were deleted and when they will be
    /// permanently removed.
    TrashMetadata,
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

/// Defines capabilities specific to shared folders
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SharedFolderCapability {
    /// Supports business/enterprise team folder functionality
    /// 
    /// This capability indicates that the file system supports specialized folders
    /// that belong to an organization rather than an individual. Team folders typically
    /// have centralized administration, different permission models, and may support
    /// features like domain-restricted sharing.
    TeamFolder,
    
    /// Supports fine-grained access control for shared folders
    /// 
    /// This capability indicates that the file system allows setting detailed permissions
    /// for shared folders. This may include role-based access controls, user/group-based
    /// permissions, or full access control lists (ACLs).
    AccessControl,
    
    /// Supports folder permissions inheritance
    /// 
    /// This capability indicates that the file system supports propagating permissions
    /// from parent folders to child items. When a permission is set on a folder, it
    /// automatically applies to all items within that folder, possibly with options to
    /// control how inheritance works.
    InheritedPermissions,
    
    /// Supports adding/removing users from shared folders
    /// 
    /// This capability indicates that the file system allows modifying the set of users
    /// who have access to a shared folder. This typically includes adding new users,
    /// removing existing users, and possibly modifying their permission levels.
    MemberManagement,
    
    /// Supports setting different permission levels for different members
    /// 
    /// This capability indicates that the file system allows assigning different
    /// permission levels to different users of a shared folder. Common permission
    /// levels include view-only, comment, edit, and manage (admin).
    DifferentiatedAccess,
    
    /// Supports ownership transfer of shared folders
    /// 
    /// This capability indicates that the file system allows changing the owner of
    /// a shared folder. The owner typically has full control over the folder and
    /// may be responsible for storage quotas or billing in some systems.
    OwnershipTransfer,
    
    /// Supports group-based permissions (like Dropbox Business, Google Workspace)
    /// 
    /// This capability indicates that the file system supports assigning permissions
    /// to groups rather than just individual users. This simplifies permission
    /// management for larger organizations by allowing permissions to be managed
    /// at the group level.
    GroupPermissions,
    
    /// Supports external collaboration with users outside the organization
    /// 
    /// This capability indicates that the file system allows sharing folders with
    /// users who are not part of the same organization. This may include additional
    /// security controls, different permission levels, or restrictions on what
    /// external users can do.
    ExternalCollaboration,
    
    /// Supports content approval workflows before publishing (Box, SharePoint)
    /// 
    /// This capability indicates that the file system supports review processes for
    /// content before it is published or made available to a wider audience. This
    /// may include approval workflows, content moderation, or publishing schedules.
    ContentApproval,
    
    /// Supports automatic expiration of shared folder access
    /// 
    /// This capability indicates that the file system allows setting time limits on
    /// shared folder access. Access permissions can be configured to automatically
    /// expire after a specified time period, requiring renewal for continued access.
    AccessExpiration,
    
    /// Supports compliance features like legal holds, retention policies
    /// 
    /// This capability indicates that the file system supports features for regulatory
    /// compliance or legal requirements. This may include legal holds that prevent
    /// deletion, retention policies that enforce minimum storage periods, or audit
    /// logs for compliance tracking.
    ComplianceFeatures,
    
    /// Supports detailed audit logging of all sharing activities
    /// 
    /// This capability indicates that the file system maintains detailed logs of all
    /// actions related to shared folders. This typically includes who shared what with
    /// whom, permission changes, access events, and other security-relevant activities.
    AuditLogging,
    
    /// Supports comments and annotations on shared files
    /// 
    /// This capability indicates that the file system allows users to add comments
    /// or annotations to files in a shared folder. This enables collaboration
    /// through discussions about specific content without modifying the content itself.
    Comments,
    
    /// Supports setting folder-level storage quotas (Box, OneDrive Business)
    /// 
    /// This capability indicates that the file system allows setting storage limits
    /// for specific shared folders. This helps organizations manage storage allocation
    /// and prevent individual shared folders from consuming excessive resources.
    StorageQuotas,
    
    /// Supports folder-level encryption keys (Box KeySafe, Google Workspace)
    /// 
    /// This capability indicates that the file system supports encryption that is
    /// specific to individual folders. This may include customer-managed encryption
    /// keys, different encryption settings for sensitive content, or specialized
    /// encryption for regulated data.
    FolderEncryption,
    
    /// Supports sharing with distribution lists or security groups
    /// 
    /// This capability indicates that the file system supports sharing with
    /// distribution lists, security groups, or similar collections of users.
    /// This simplifies sharing with predefined groups of users without having
    /// to select each user individually.
    DistributionLists,
    
    /// Supports automated workflows for shared folders (Box Relay, Microsoft Flow)
    /// 
    /// This capability indicates that the file system supports automated processes
    /// or workflows triggered by actions within shared folders. Examples include
    /// automatic notifications, approval workflows, content processing, or
    /// integration with other business systems.
    AutomatedWorkflows,
    
    /// Supports viewer information for seeing who has accessed shared content
    /// 
    /// This capability indicates that the file system provides information about who
    /// has viewed shared content. This may include access logs, view counts, or
    /// detailed analytics about how users interact with shared content.
    ViewerTracking,
}

/// Defines capabilities specific to sharing links
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShareLinkCapability {
    /// Supports password protection for shared links
    /// 
    /// This capability indicates that the file system allows requiring a password
    /// to access content via a shared link. This adds an additional layer of
    /// security beyond just possessing the link URL.
    PasswordProtection,
    
    /// Supports expiration dates for shared links
    /// 
    /// This capability indicates that the file system allows setting a time limit
    /// for shared links. After the expiration date, the link becomes invalid and
    /// no longer provides access to the content.
    Expiration,
    
    /// Supports view-only access (no download) for shared links
    /// 
    /// This capability indicates that the file system allows creating links that
    /// permit viewing content but not downloading the original files. This helps
    /// protect sensitive content while still allowing it to be shared.
    ViewOnlyAccess,
    
    /// Supports download limits for shared links
    /// 
    /// This capability indicates that the file system allows restricting the number
    /// of times content can be downloaded via a shared link. This may include setting
    /// a maximum number of downloads or limiting downloads per user.
    DownloadLimits,
    
    /// Supports custom permissions for shared links
    /// 
    /// This capability indicates that the file system allows setting specific
    /// permissions for shared links beyond basic access. This may include allowing
    /// comments, edits, or other actions depending on the system.
    CustomPermissions,
    
    /// Supports watermarking for shared document previews
    /// 
    /// This capability indicates that the file system supports adding visible
    /// watermarks to document previews accessed via shared links. This helps
    /// identify the source of leaked documents and discourage unauthorized sharing.
    Watermarking,
    
    /// Supports domain restrictions for shared links
    /// 
    /// This capability indicates that the file system allows limiting access to
    /// shared links to users within specific email domains. This helps ensure that
    /// shared content remains within an organization or trusted partners.
    DomainRestriction,
    
    /// Supports email verification before accessing shared links
    /// 
    /// This capability indicates that the file system requires users to verify their
    /// email address before accessing content via a shared link. This adds authentication
    /// without requiring full account creation.
    EmailVerification,
    
    /// Supports branded sharing pages (Box, Dropbox Business)
    /// 
    /// This capability indicates that the file system allows customizing the appearance
    /// of shared link pages with organization branding. This typically includes logos,
    /// colors, and possibly custom domains for a more professional appearance.
    BrandedSharing,
    
    /// Supports file requests (Dropbox, Box)
    /// 
    /// This capability indicates that the file system allows creating links that
    /// enable others to upload files to a specific location. This is useful for
    /// collecting submissions or contributions from external users.
    FileRequests,
    
    /// Supports analytics and reporting for link usage
    /// 
    /// This capability indicates that the file system provides statistics and reports
    /// about shared link usage. This may include view counts, download statistics,
    /// geographic data, or user tracking.
    LinkAnalytics,
    
    /// Supports preview-only access without downloading original files
    /// 
    /// This capability indicates that the file system allows creating links that
    /// permit viewing file previews but not accessing the original files. This is
    /// similar to view-only access but may be more restrictive.
    PreviewOnly,
    
    /// Supports selective file access within a shared folder link
    /// 
    /// This capability indicates that the file system allows creating links to
    /// specific files within a shared folder, rather than sharing the entire folder.
    /// This provides more granular control over what content is accessible.
    SelectiveAccess,
    
    /// Supports integration with social media platforms for sharing
    /// 
    /// This capability indicates that the file system provides features to easily
    /// share links on social media platforms. This may include specialized formatting,
    /// preview generation, or direct posting to social networks.
    SocialSharing,
    
    /// Supports access approval workflow (request access feature)
    /// 
    /// This capability indicates that the file system allows setting up access
    /// requests for shared content. Users without direct access can request permission,
    /// triggering an approval workflow before access is granted.
    AccessRequest,
    
    /// Supports direct integration with collaboration tools (MS Teams, Slack)
    /// 
    /// This capability indicates that the file system offers specialized integration
    /// with collaboration platforms. This may include rich previews, notifications,
    /// or interactive features when shared in collaboration tools.
    CollaborationIntegration,
    
    /// Supports link-specific notification settings
    /// 
    /// This capability indicates that the file system allows configuring notification
    /// preferences for shared links. This may include alerts for access, downloads,
    /// or other activities specific to content shared via links.
    NotificationSettings,
    
    /// Supports embedding content in other websites
    /// 
    /// This capability indicates that the file system provides features for embedding
    /// shared content in external websites. This typically includes specialized embed
    /// codes, player interfaces, or interactive widgets.
    Embedding,
    
    /// Supports enhanced security controls like geographic restrictions
    /// 
    /// This capability indicates that the file system allows restricting access to
    /// shared links based on geographic location. This helps prevent access from
    /// untrusted regions or comply with geographic data sovereignty requirements.
    GeoRestriction,
    
    /// Supports direct editing in web applications (Office Online, Google Docs)
    /// 
    /// This capability indicates that the file system allows editing content directly
    /// in the browser when accessed via a shared link. This enables collaboration
    /// without requiring users to download, edit, and re-upload files.
    WebEditing,
}

/// Defines capabilities specific to versioning systems
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersioningCapability {
    /// Supports listing all versions of a file
    /// 
    /// This capability indicates that the file system allows retrieving a history
    /// of all versions of a file. This typically includes metadata about each version
    /// such as when it was created and by whom.
    ListVersions,
    
    /// Supports retrieving specific versions of a file
    /// 
    /// This capability indicates that the file system allows accessing the content
    /// of specific previous versions of a file. This enables viewing or working with
    /// historical content without affecting the current version.
    GetVersion,
    
    /// Supports reverting to a previous version
    /// 
    /// This capability indicates that the file system allows restoring a file to a
    /// previous version. This typically makes the selected historical version become
    /// the current version, possibly preserving the version history.
    RevertVersion,
    
    /// Supports deleting specific versions
    /// 
    /// This capability indicates that the file system allows removing specific versions
    /// from a file's version history. This can be useful for eliminating versions with
    /// errors or sensitive content while preserving other history.
    DeleteVersion,
    
    /// Supports version tagging or labeling
    /// 
    /// This capability indicates that the file system allows adding descriptive tags
    /// or labels to specific versions. This helps identify important versions or
    /// milestones in a file's development history.
    VersionLabels,
    
    /// Supports version comments or descriptions
    /// 
    /// This capability indicates that the file system allows adding descriptive text
    /// to specific versions. This helps document the changes made in each version or
    /// provide context about why changes were made.
    VersionComments,
    
    /// Supports retention policies for versions
    /// 
    /// This capability indicates that the file system allows configuring how long
    /// version history is preserved. This may include settings for maximum version
    /// count, maximum age, or different policies for different file types.
    RetentionPolicies,
    
    /// Supports automatic versioning on changes
    /// 
    /// This capability indicates that the file system automatically creates new versions
    /// when files are modified. This provides a complete history without requiring users
    /// to manually create versions or check in changes.
    AutoVersioning,
    
    /// Supports versioning for directories/folders
    /// 
    /// This capability indicates that the file system extends versioning to entire
    /// folders, not just individual files. This allows tracking changes to folder
    /// structure, permissions, or other folder-level properties.
    FolderVersioning,
    
    /// Supports version comparison
    /// 
    /// This capability indicates that the file system provides features for comparing
    /// different versions of a file. This may include visual diff tools, content
    /// comparison, or highlighting changes between versions.
    VersionComparison,
}

/// Defines capabilities specific to content hashing
/// 
/// # Examples
/// 
/// For a Dropbox implementation, you would add capabilities like:
/// 
/// ```
/// let mut capabilities = CapabilitySet::new();
/// // Add other capabilities...
/// capabilities.add(Capability::ContentHash);
/// 
/// // Add Dropbox's specific content hash capabilities
/// capabilities.add_content_hash_capability(ContentHashCapability::SHA256Blocks {
///     provider: "dropbox".to_string(),
///     block_size: Some(4 * 1024 * 1024), // Dropbox uses 4MB blocks
/// });
/// 
/// // Add Dropbox's content hash (a custom hash format based on SHA-256)
/// capabilities.add_content_hash_capability(ContentHashCapability::CustomHash {
///     provider: "dropbox".to_string(),
///     algorithm: "content_hash".to_string(),
/// });
/// ```
/// 
/// Then, to check for Dropbox's specific hashing capabilities:
/// 
/// ```
/// if provider.capabilities().has_sha256_blocks_for_provider("dropbox") {
///     // Use Dropbox's block-based hashing
///     let block_size = provider.capabilities().get_sha256_blocks_size("dropbox").unwrap_or(4 * 1024 * 1024);
///     // ...
/// }
/// 
/// if provider.capabilities().has_custom_hash("dropbox", "content_hash") {
///     // Use Dropbox's custom content hash format
///     // ...
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum ContentHashCapability {
    /// SHA-256 hash of the entire file
    /// 
    /// This capability indicates that the file system supports calculating or verifying
    /// SHA-256 cryptographic hashes of complete file contents. SHA-256 is a widely used
    /// secure hash algorithm that produces a 256-bit (32-byte) hash value.
    SHA256Whole,
    
    /// SHA-256 hash using block-based approach with identifier for specific implementation
    /// 
    /// This capability indicates that the file system supports a block-based approach to
    /// SHA-256 hashing, where files are divided into blocks, each block is hashed separately,
    /// and the results are combined in a provider-specific way.
    SHA256Blocks {
        /// Provider identifier (e.g., "dropbox", "onedrive", "custom")
        provider: &'static str,
        /// Optional block size in bytes, if known at compile time
        block_size: Option<usize>,
    },
    
    /// MD5 hash of the entire file
    /// 
    /// This capability indicates that the file system supports calculating or verifying
    /// MD5 cryptographic hashes of complete file contents. While MD5 is considered
    /// cryptographically weak, it remains widely used for non-security-critical integrity checking.
    MD5Whole,
    
    /// SHA-1 hash of the entire file
    /// 
    /// This capability indicates that the file system supports calculating or verifying
    /// SHA-1 cryptographic hashes of complete file contents. While SHA-1 is considered
    /// cryptographically weak, it remains in use in some systems for compatibility.
    SHA1Whole,
    
    /// Support for custom or proprietary hash algorithms
    /// 
    /// This capability indicates that the file system supports a non-standard or
    /// proprietary hashing algorithm specific to a particular provider or use case.
    CustomHash {
        /// Provider identifier (e.g., "dropbox", "onedrive", "s3")
        provider: &'static str,
        /// Algorithm name or identifier
        algorithm: &'static str,
    },
    
    /// Supports verifying content against a hash
    /// 
    /// This capability indicates that the file system provides mechanisms to verify
    /// file integrity by comparing calculated hashes against expected values. This
    /// helps detect corruption or tampering with file contents.
    Verification,
    
    /// Supports hashing for directories/folders
    /// 
    /// This capability indicates that the file system extends hashing to entire
    /// directories, not just individual files. This allows verifying the integrity
    /// of a complete directory structure, including file names and organization.
    DirectoryHashing,
    
    /// Supports incremental hashing for large files
    /// 
    /// This capability indicates that the file system supports efficient hash calculation
    /// for large files without requiring the entire file to be loaded into memory. This
    /// typically involves streaming or chunk-based hashing approaches.
    IncrementalHashing,
    
    /// Supports content-based deduplication
    /// 
    /// This capability indicates that the file system uses content hashing for
    /// deduplication purposes. This allows storing identical content only once, even
    /// if it appears in multiple files, saving storage space.
    Deduplication,
}

/// Defines capabilities specific to WebDAV protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDAVCapability {
    /// Supports WebDAV Class 1 compliance (RFC 4918)
    /// 
    /// This capability indicates that the file system supports basic WebDAV operations
    /// including GET, PUT, DELETE, PROPFIND, PROPPATCH, MKCOL, COPY, and MOVE methods.
    Class1Compliance,
    
    /// Supports WebDAV Class 2 compliance (includes locking)
    /// 
    /// This capability indicates that the file system supports advanced WebDAV operations
    /// including locking (LOCK and UNLOCK methods) to prevent the "lost update problem."
    Class2Compliance,
    
    /// Supports WebDAV Class 3 compliance (includes versioning)
    /// 
    /// This capability indicates that the file system supports WebDAV versioning
    /// capabilities as specified in RFC 3253, allowing access to previous versions of resources.
    Class3Compliance,
    
    /// Supports WebDAV Access Control Protocol (RFC 3744)
    /// 
    /// This capability indicates that the file system supports the WebDAV Access Control
    /// Protocol extension for defining access control lists and user privileges.
    AccessControl,
    
    /// Supports WebDAV SEARCH method (RFC 5323)
    /// 
    /// This capability indicates that the file system supports the WebDAV SEARCH method,
    /// allowing clients to search for resources matching specified criteria.
    Search,
    
    /// Supports CalDAV (RFC 4791)
    /// 
    /// This capability indicates that the file system supports CalDAV, a WebDAV extension
    /// for calendar access, allowing clients to access and manage calendar data.
    CalDAV,
    
    /// Supports CardDAV (RFC 6352)
    /// 
    /// This capability indicates that the file system supports CardDAV, a WebDAV extension
    /// for address book access, allowing clients to access and manage contact data.
    CardDAV,
    
    /// Supports Microsoft MS-WDVME extensions
    /// 
    /// This capability indicates that the file system supports Microsoft's WebDAV
    /// extensions, which provide additional functionality in Windows environments.
    MicrosoftExtensions,
    
    /// Supports mounting as a network drive
    /// 
    /// This capability indicates that the WebDAV file system can be mounted as a
    /// network drive in supported operating systems, providing seamless integration.
    NetworkDriveMapping,
    
    /// Supports chunked file uploads
    /// 
    /// This capability indicates that the WebDAV server supports uploading files
    /// in chunks, which is useful for large file transfers that might be interrupted.
    ChunkedUploads,
}

/// Defines capabilities specific to FTP/SFTP protocols
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FTPSFTPCapability {
    /// Supports basic FTP commands (RFC 959)
    /// 
    /// This capability indicates that the file system supports basic FTP operations
    /// including file upload, download, listing, and navigation.
    BasicFTP,
    
    /// Supports FTP over SSL/TLS (FTPS)
    /// 
    /// This capability indicates that the file system supports secure FTP connections
    /// using SSL/TLS encryption to protect data in transit.
    FTPS,
    
    /// Supports FTP extension commands (RFC 3659)
    /// 
    /// This capability indicates that the file system supports extended FTP commands
    /// such as MLSD, MLST, SIZE, and MDTM for more detailed file information.
    ExtendedFTP,
    
    /// Supports SFTP protocol (SSH File Transfer Protocol)
    /// 
    /// This capability indicates that the file system supports the SFTP protocol,
    /// which provides secure file transfer and manipulation over SSH.
    SFTP,
    
    /// Supports SFTP version 3
    /// 
    /// This capability indicates that the file system supports SFTP protocol version 3,
    /// which includes basic file operations and attributes.
    SFTPv3,
    
    /// Supports SFTP version 4 or higher
    /// 
    /// This capability indicates that the file system supports SFTP protocol version 4
    /// or higher, which includes extended attributes and file locking.
    SFTPv4Plus,
    
    /// Supports SFTP file locking extensions
    /// 
    /// This capability indicates that the file system supports SFTP extensions for
    /// locking files to prevent concurrent modification.
    SFTPLocking,
    
    /// Supports public key authentication
    /// 
    /// This capability indicates that the file system supports authentication using
    /// public key cryptography rather than just passwords.
    PublicKeyAuth,
    
    /// Supports FXP (server-to-server transfer)
    /// 
    /// This capability indicates that the file system supports FXP (File eXchange Protocol),
    /// allowing direct transfers between FTP servers without passing through the client.
    ServerToServerTransfer,
    
    /// Supports transfer resume
    /// 
    /// This capability indicates that the file system supports resuming interrupted
    /// transfers, starting from where they left off rather than from the beginning.
    ResumeTransfer,
}

/// Defines capabilities specific to cloud object storage services
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloudStorageCapability {
    /// Supports Amazon S3 compatibility
    /// 
    /// This capability indicates that the file system can interact with Amazon S3
    /// or S3-compatible storage services using the S3 API.
    S3Compatible,
    
    /// Supports Azure Blob Storage compatibility
    /// 
    /// This capability indicates that the file system can interact with Microsoft
    /// Azure Blob Storage using the Azure Storage API.
    AzureBlobCompatible,
    
    /// Supports Google Cloud Storage compatibility
    /// 
    /// This capability indicates that the file system can interact with Google
    /// Cloud Storage using the GCS API.
    GCPStorageCompatible,
    
    /// Supports multipart uploads
    /// 
    /// This capability indicates that the file system supports uploading large files
    /// in multiple parts, which is important for reliability and performance with large objects.
    MultipartUpload,
    
    /// Supports server-side encryption
    /// 
    /// This capability indicates that the file system supports encryption of data
    /// at rest, managed by the storage service rather than the client.
    ServerSideEncryption,
    
    /// Supports customer-provided encryption keys
    /// 
    /// This capability indicates that the file system supports using customer-provided
    /// keys for server-side encryption rather than service-managed keys.
    CustomerProvidedKeys,
    
    /// Supports object lifecycle management
    /// 
    /// This capability indicates that the file system supports automated management
    /// of object lifecycles, including transitioning between storage tiers and expiration.
    LifecycleManagement,
    
    /// Supports object tagging
    /// 
    /// This capability indicates that the file system supports adding metadata tags
    /// to objects for organization, billing, or access control purposes.
    ObjectTagging,
    
    /// Supports object versioning
    /// 
    /// This capability indicates that the file system supports maintaining multiple
    /// versions of objects, providing protection against accidental deletion or overwrite.
    ObjectVersioning,
    
    /// Supports requester pays model
    /// 
    /// This capability indicates that the file system supports a billing model where
    /// the requester rather than the bucket owner pays for data transfer costs.
    RequesterPays,
    
    /// Supports object-level ACLs
    /// 
    /// This capability indicates that the file system supports setting access control
    /// lists at the individual object level for fine-grained access control.
    ObjectACLs,
    
    /// Supports object lock (WORM - Write Once Read Many)
    /// 
    /// This capability indicates that the file system supports preventing objects
    /// from being deleted or overwritten for a specified period, useful for compliance.
    ObjectLock,
    
    /// Supports inventory reporting
    /// 
    /// This capability indicates that the file system supports generating inventory
    /// reports of objects and their metadata for analysis or verification.
    InventoryReporting,
    
    /// Supports batch operations
    /// 
    /// This capability indicates that the file system supports performing operations
    /// on multiple objects in a single request, improving efficiency for bulk actions.
    BatchOperations,
    
    /// Supports cross-region replication
    /// 
    /// This capability indicates that the file system supports automatically replicating
    /// objects between different geographic regions for redundancy or performance.
    CrossRegionReplication,
    
    /// Supports storage tier transitions
    /// 
    /// This capability indicates that the file system supports moving objects between
    /// different storage tiers to balance performance and cost based on access patterns.
    StorageTierTransitions,
    
    /// Supports pre-signed URLs
    /// 
    /// This capability indicates that the file system supports generating time-limited
    /// URLs that grant temporary access to objects without requiring authentication.
    PreSignedURLs,
    
    /// Supports server access logging
    /// 
    /// This capability indicates that the file system supports detailed logging of
    /// access requests for security auditing or usage analysis.
    ServerAccessLogging,
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
    shared_folder_capabilities: Vec<SharedFolderCapability>,
    share_link_capabilities: Vec<ShareLinkCapability>,
    versioning_capabilities: Vec<VersioningCapability>,
    content_hash_capabilities: Vec<ContentHashCapability>,
    webdav_capabilities: Vec<WebDAVCapability>,
    ftpsftp_capabilities: Vec<FTPSFTPCapability>,
    cloud_storage_capabilities: Vec<CloudStorageCapability>,
}

impl CapabilitySet {
    /// Creates a new empty capability set
    pub fn new() -> Self {
        Self {
            capabilities: Vec::new(),
            protocol_types: Vec::new(),
            provider_info: None,
            shared_folder_capabilities: Vec::new(),
            share_link_capabilities: Vec::new(),
            versioning_capabilities: Vec::new(),
            content_hash_capabilities: Vec::new(),
            webdav_capabilities: Vec::new(),
            ftpsftp_capabilities: Vec::new(),
            cloud_storage_capabilities: Vec::new(),
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
                Capability::SharedFolder,
                Capability::ShareLink,
                Capability::Versioning,
                Capability::ContentHash,
                Capability::AccessControl,
                Capability::ExtendedAttributes,
                Capability::Locking,
                Capability::Encryption,
                Capability::SharingOptions,
                Capability::TeamFolder,
                Capability::TrashManagement,
                Capability::PermanentDeletion,
                Capability::FileRestoration,
                Capability::EmptyTrash,
                Capability::ListTrash,
                Capability::TrashMetadata,
            ],
            protocol_types: vec![
                ProtocolType::LocalFileSystem,
                ProtocolType::WebDAV,
                ProtocolType::FTPSFTP,
                ProtocolType::CloudObjectStorage,
                ProtocolType::CloudStorageService,
            ],
            provider_info: None,
            shared_folder_capabilities: vec![
                SharedFolderCapability::TeamFolder,
                SharedFolderCapability::AccessControl,
                SharedFolderCapability::InheritedPermissions,
                SharedFolderCapability::MemberManagement,
                SharedFolderCapability::DifferentiatedAccess,
                SharedFolderCapability::OwnershipTransfer,
                SharedFolderCapability::GroupPermissions,
                SharedFolderCapability::ExternalCollaboration,
                SharedFolderCapability::ContentApproval,
                SharedFolderCapability::AccessExpiration,
                SharedFolderCapability::ComplianceFeatures,
                SharedFolderCapability::AuditLogging,
                SharedFolderCapability::Comments,
                SharedFolderCapability::StorageQuotas,
                SharedFolderCapability::FolderEncryption,
                SharedFolderCapability::DistributionLists,
                SharedFolderCapability::AutomatedWorkflows,
                SharedFolderCapability::ViewerTracking,
            ],
            share_link_capabilities: vec![
                ShareLinkCapability::PasswordProtection,
                ShareLinkCapability::Expiration,
                ShareLinkCapability::ViewOnlyAccess,
                ShareLinkCapability::DownloadLimits,
                ShareLinkCapability::CustomPermissions,
                ShareLinkCapability::Watermarking,
                ShareLinkCapability::DomainRestriction,
                ShareLinkCapability::EmailVerification,
                ShareLinkCapability::BrandedSharing,
                ShareLinkCapability::FileRequests,
                ShareLinkCapability::LinkAnalytics,
                ShareLinkCapability::PreviewOnly,
                ShareLinkCapability::SelectiveAccess,
                ShareLinkCapability::SocialSharing,
                ShareLinkCapability::AccessRequest,
                ShareLinkCapability::CollaborationIntegration,
                ShareLinkCapability::NotificationSettings,
                ShareLinkCapability::Embedding,
                ShareLinkCapability::GeoRestriction,
                ShareLinkCapability::WebEditing,
            ],
            versioning_capabilities: vec![
                VersioningCapability::ListVersions,
                VersioningCapability::GetVersion,
                VersioningCapability::RevertVersion,
                VersioningCapability::DeleteVersion,
                VersioningCapability::VersionLabels,
                VersioningCapability::VersionComments,
                VersioningCapability::RetentionPolicies,
                VersioningCapability::AutoVersioning,
                VersioningCapability::FolderVersioning,
                VersioningCapability::VersionComparison,
            ],
            content_hash_capabilities: vec![
                ContentHashCapability::SHA256Whole,
                ContentHashCapability::SHA256Blocks {
                    provider: "dropbox",
                    block_size: None,
                },
                ContentHashCapability::MD5Whole,
                ContentHashCapability::SHA1Whole,
                ContentHashCapability::CustomHash {
                    provider: "dropbox",
                    algorithm: "content_hash",
                },
                ContentHashCapability::Verification,
                ContentHashCapability::DirectoryHashing,
                ContentHashCapability::IncrementalHashing,
                ContentHashCapability::Deduplication,
            ],
            webdav_capabilities: vec![
                WebDAVCapability::Class1Compliance,
                WebDAVCapability::Class2Compliance,
                WebDAVCapability::Class3Compliance,
                WebDAVCapability::AccessControl,
                WebDAVCapability::Search,
                WebDAVCapability::CalDAV,
                WebDAVCapability::CardDAV,
                WebDAVCapability::MicrosoftExtensions,
                WebDAVCapability::NetworkDriveMapping,
                WebDAVCapability::ChunkedUploads,
            ],
            ftpsftp_capabilities: vec![
                FTPSFTPCapability::BasicFTP,
                FTPSFTPCapability::FTPS,
                FTPSFTPCapability::ExtendedFTP,
                FTPSFTPCapability::SFTP,
                FTPSFTPCapability::SFTPv3,
                FTPSFTPCapability::SFTPv4Plus,
                FTPSFTPCapability::SFTPLocking,
                FTPSFTPCapability::PublicKeyAuth,
                FTPSFTPCapability::ServerToServerTransfer,
                FTPSFTPCapability::ResumeTransfer,
            ],
            cloud_storage_capabilities: vec![
                CloudStorageCapability::S3Compatible,
                CloudStorageCapability::AzureBlobCompatible,
                CloudStorageCapability::GCPStorageCompatible,
                CloudStorageCapability::MultipartUpload,
                CloudStorageCapability::ServerSideEncryption,
                CloudStorageCapability::CustomerProvidedKeys,
                CloudStorageCapability::LifecycleManagement,
                CloudStorageCapability::ObjectTagging,
                CloudStorageCapability::ObjectVersioning,
                CloudStorageCapability::RequesterPays,
                CloudStorageCapability::ObjectACLs,
                CloudStorageCapability::ObjectLock,
                CloudStorageCapability::InventoryReporting,
                CloudStorageCapability::BatchOperations,
                CloudStorageCapability::CrossRegionReplication,
                CloudStorageCapability::StorageTierTransitions,
                CloudStorageCapability::PreSignedURLs,
                CloudStorageCapability::ServerAccessLogging,
            ],
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
            shared_folder_capabilities: Vec::new(),
            share_link_capabilities: Vec::new(),
            versioning_capabilities: Vec::new(),
            content_hash_capabilities: Vec::new(),
            webdav_capabilities: Vec::new(),
            ftpsftp_capabilities: Vec::new(),
            cloud_storage_capabilities: Vec::new(),
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

    /// Adds a shared folder capability to this set
    pub fn add_shared_folder_capability(&mut self, capability: SharedFolderCapability) {
        if !self.has_shared_folder_capability(capability) {
            self.shared_folder_capabilities.push(capability);
        }
    }

    /// Adds a share link capability to this set
    pub fn add_share_link_capability(&mut self, capability: ShareLinkCapability) {
        if !self.has_share_link_capability(capability) {
            self.share_link_capabilities.push(capability);
        }
    }

    /// Adds a versioning capability to this set
    pub fn add_versioning_capability(&mut self, capability: VersioningCapability) {
        if !self.versioning_capabilities.contains(&capability) {
            self.versioning_capabilities.push(capability);
        }
    }
    
    /// Adds a content hash capability to this set
    pub fn add_content_hash_capability(&mut self, capability: ContentHashCapability) {
        if !self.content_hash_capabilities.contains(&capability) {
            self.content_hash_capabilities.push(capability);
        }
    }

    /// Adds a WebDAV capability to this set
    pub fn add_webdav_capability(&mut self, capability: WebDAVCapability) {
        if !self.has_webdav_capability(capability) {
            self.webdav_capabilities.push(capability);
        }
    }

    /// Adds an FTP/SFTP capability to this set
    pub fn add_ftpsftp_capability(&mut self, capability: FTPSFTPCapability) {
        if !self.has_ftpsftp_capability(capability) {
            self.ftpsftp_capabilities.push(capability);
        }
    }

    /// Adds a cloud storage capability to this set
    pub fn add_cloud_storage_capability(&mut self, capability: CloudStorageCapability) {
        if !self.has_cloud_storage_capability(capability) {
            self.cloud_storage_capabilities.push(capability);
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

    /// Checks if this set has the specified shared folder capability
    pub fn has_shared_folder_capability(&self, capability: SharedFolderCapability) -> bool {
        self.shared_folder_capabilities.contains(&capability)
    }

    /// Checks if this set has the specified share link capability
    pub fn has_share_link_capability(&self, capability: ShareLinkCapability) -> bool {
        self.share_link_capabilities.contains(&capability)
    }

    /// Checks if this set contains all the capabilities in the specified set
    pub fn has_all(&self, capabilities: &[Capability]) -> bool {
        capabilities.iter().all(|cap| self.has(*cap))
    }

    /// Checks if this set contains any of the capabilities in the specified set
    pub fn has_any(&self, capabilities: &[Capability]) -> bool {
        capabilities.iter().any(|cap| self.has(*cap))
    }

    /// Checks if this set contains all the shared folder capabilities in the specified set
    pub fn has_all_shared_folder_capabilities(&self, capabilities: &[SharedFolderCapability]) -> bool {
        capabilities.iter().all(|cap| self.has_shared_folder_capability(*cap))
    }

    /// Checks if this set contains any of the shared folder capabilities in the specified set
    pub fn has_any_shared_folder_capabilities(&self, capabilities: &[SharedFolderCapability]) -> bool {
        capabilities.iter().any(|cap| self.has_shared_folder_capability(*cap))
    }

    /// Checks if this set contains all the share link capabilities in the specified set
    pub fn has_all_share_link_capabilities(&self, capabilities: &[ShareLinkCapability]) -> bool {
        capabilities.iter().all(|cap| self.has_share_link_capability(*cap))
    }

    /// Checks if this set contains any of the share link capabilities in the specified set
    pub fn has_any_share_link_capabilities(&self, capabilities: &[ShareLinkCapability]) -> bool {
        capabilities.iter().any(|cap| self.has_share_link_capability(*cap))
    }

    /// Checks if this set has a specific versioning capability
    pub fn has_versioning_capability(&self, capability: VersioningCapability) -> bool {
        self.versioning_capabilities.contains(&capability)
    }
    
    /// Checks if this set has a specific content hash capability
    pub fn has_content_hash_capability(&self, capability: ContentHashCapability) -> bool {
        self.content_hash_capabilities.contains(&capability)
    }
    
    /// Checks if this set has all of the specified versioning capabilities
    pub fn has_all_versioning_capabilities(&self, capabilities: &[VersioningCapability]) -> bool {
        capabilities.iter().all(|&c| self.has_versioning_capability(c))
    }
    
    /// Checks if this set has any of the specified versioning capabilities
    pub fn has_any_versioning_capabilities(&self, capabilities: &[VersioningCapability]) -> bool {
        capabilities.iter().any(|&c| self.has_versioning_capability(c))
    }
    
    /// Checks if this set has all of the specified content hash capabilities
    pub fn has_all_content_hash_capabilities(&self, capabilities: &[ContentHashCapability]) -> bool {
        capabilities.iter().all(|c| self.has_content_hash_capability(*c))
    }
    
    /// Checks if this set has any of the specified content hash capabilities
    pub fn has_any_content_hash_capabilities(&self, capabilities: &[ContentHashCapability]) -> bool {
        capabilities.iter().any(|c| self.has_content_hash_capability(*c))
    }

    /// Checks if this set has SHA-256 block hashing for the specified provider
    pub fn has_sha256_blocks_for_provider(&self, provider: &str) -> bool {
        self.content_hash_capabilities.iter().any(|&cap| {
            if let ContentHashCapability::SHA256Blocks { provider: cap_provider, .. } = cap {
                *cap_provider == *provider
            } else {
                false
            }
        })
    }
    
    /// Checks if this set has custom hash support for the specified provider and algorithm
    pub fn has_custom_hash(&self, provider: &str, algorithm: &str) -> bool {
        self.content_hash_capabilities.iter().any(|&cap| {
            if let ContentHashCapability::CustomHash { provider: cap_provider, algorithm: cap_algorithm } = cap {
                *cap_provider == *provider && *cap_algorithm == *algorithm
            } else {
                false
            }
        })
    }
    
    /// Gets the block size for a specific provider's SHA-256 blocks implementation, if available
    pub fn get_sha256_blocks_size(&self, provider: &str) -> Option<usize> {
        self.content_hash_capabilities.iter().find_map(|cap| {
            if let ContentHashCapability::SHA256Blocks { provider: cap_provider, block_size } = cap {
                if cap_provider == &provider {
                    *block_size
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    /// Checks if this set has any trash management capabilities
    pub fn has_any_trash_capabilities(&self) -> bool {
        self.has_any(&[
            Capability::TrashManagement,
            Capability::PermanentDeletion,
            Capability::FileRestoration,
            Capability::EmptyTrash,
            Capability::ListTrash,
            Capability::TrashMetadata,
        ])
    }
    
    /// Checks if this set has all trash management capabilities
    pub fn has_all_trash_capabilities(&self) -> bool {
        self.has_all(&[
            Capability::TrashManagement,
            Capability::PermanentDeletion,
            Capability::FileRestoration,
            Capability::EmptyTrash,
            Capability::ListTrash,
            Capability::TrashMetadata,
        ])
    }
    
    /// Checks if this set supports permanent deletion of files
    pub fn supports_permanent_deletion(&self) -> bool {
        self.has(Capability::PermanentDeletion)
    }
    
    /// Checks if this set supports restoring files from trash
    pub fn supports_file_restoration(&self) -> bool {
        self.has(Capability::FileRestoration)
    }
    
    /// Checks if this set supports emptying the trash
    pub fn supports_empty_trash(&self) -> bool {
        self.has(Capability::EmptyTrash)
    }
    
    /// Checks if this set supports listing trashed files
    pub fn supports_list_trash(&self) -> bool {
        self.has(Capability::ListTrash)
    }
    
    /// Checks if this set supports metadata for trashed files
    pub fn supports_trash_metadata(&self) -> bool {
        self.has(Capability::TrashMetadata)
    }

    /// Checks if this set has a specific WebDAV capability
    pub fn has_webdav_capability(&self, capability: WebDAVCapability) -> bool {
        self.webdav_capabilities.contains(&capability)
    }

    /// Checks if this set has a specific FTP/SFTP capability
    pub fn has_ftpsftp_capability(&self, capability: FTPSFTPCapability) -> bool {
        self.ftpsftp_capabilities.contains(&capability)
    }

    /// Checks if this set has a specific cloud storage capability
    pub fn has_cloud_storage_capability(&self, capability: CloudStorageCapability) -> bool {
        self.cloud_storage_capabilities.contains(&capability)
    }

    /// Checks if this set has all of the specified WebDAV capabilities
    pub fn has_all_webdav_capabilities(&self, capabilities: &[WebDAVCapability]) -> bool {
        capabilities.iter().all(|&c| self.has_webdav_capability(c))
    }

    /// Checks if this set has any of the specified WebDAV capabilities
    pub fn has_any_webdav_capabilities(&self, capabilities: &[WebDAVCapability]) -> bool {
        capabilities.iter().any(|&c| self.has_webdav_capability(c))
    }

    /// Checks if this set has all of the specified FTP/SFTP capabilities
    pub fn has_all_ftpsftp_capabilities(&self, capabilities: &[FTPSFTPCapability]) -> bool {
        capabilities.iter().all(|&c| self.has_ftpsftp_capability(c))
    }

    /// Checks if this set has any of the specified FTP/SFTP capabilities
    pub fn has_any_ftpsftp_capabilities(&self, capabilities: &[FTPSFTPCapability]) -> bool {
        capabilities.iter().any(|&c| self.has_ftpsftp_capability(c))
    }

    /// Checks if this set has all of the specified cloud storage capabilities
    pub fn has_all_cloud_storage_capabilities(&self, capabilities: &[CloudStorageCapability]) -> bool {
        capabilities.iter().all(|&c| self.has_cloud_storage_capability(c))
    }

    /// Checks if this set has any of the specified cloud storage capabilities
    pub fn has_any_cloud_storage_capabilities(&self, capabilities: &[CloudStorageCapability]) -> bool {
        capabilities.iter().any(|&c| self.has_cloud_storage_capability(c))
    }

    /// Checks if this set supports any S3-compatible operations
    pub fn supports_s3(&self) -> bool {
        self.has_cloud_storage_capability(CloudStorageCapability::S3Compatible)
    }

    /// Checks if this set supports any Azure Blob Storage operations
    pub fn supports_azure_blob(&self) -> bool {
        self.has_cloud_storage_capability(CloudStorageCapability::AzureBlobCompatible)
    }

    /// Checks if this set supports any Google Cloud Storage operations
    pub fn supports_gcp_storage(&self) -> bool {
        self.has_cloud_storage_capability(CloudStorageCapability::GCPStorageCompatible)
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