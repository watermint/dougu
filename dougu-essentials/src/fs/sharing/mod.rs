use crate::time::ZonedDateTime;
use std::fmt::Debug;

/// Defines the privacy level for a shared resource
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivacyLevel {
    /// Only accessible by authenticated users who have been explicitly granted access
    Private,
    /// Accessible by anyone with the link, but not discoverable
    LinkOnly,
    /// Publicly discoverable and accessible by anyone
    Public,
}

/// Defines the access permission level for a shared resource
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessPermission {
    /// Users can only view the content
    ViewOnly,
    /// Users can comment on the content
    Comment,
    /// Users can edit the content
    Edit,
    /// Users have full control over the content
    Owner,
}

/// Defines download permissions for a shared resource
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadPermission {
    /// No downloads allowed, preview only
    NoDownload,
    /// Allow downloading the file/folder
    AllowDownload,
    /// Allow downloading with a limit on number of downloads
    LimitedDownloads(u32),
}

/// Contains advanced sharing options for files/folders
#[derive(Debug, Clone)]
pub struct SharingOptions {
    /// Privacy level for this shared resource
    pub privacy: PrivacyLevel,
    
    /// Access permission level for this shared resource
    pub permission: AccessPermission,
    
    /// Password for accessing the shared resource (if enabled)
    pub password: Option<String>,
    
    /// Expiration time for the shared resource (if enabled)
    pub expiration: Option<ZonedDateTime>,
    
    /// Specifies if and how the resource can be downloaded
    pub download: DownloadPermission,
    
    /// Optional custom message to display to recipients
    pub message: Option<String>,
    
    /// If true, recipients are notified when the resource is shared with them
    pub notify_recipients: bool,
    
    /// Domain restrictions on accessing the shared resource
    pub domain_restriction: Option<String>,
    
    /// Watermark documents when viewed (if applicable)
    pub watermark: bool,
}

impl Default for SharingOptions {
    fn default() -> Self {
        Self {
            privacy: PrivacyLevel::LinkOnly,
            permission: AccessPermission::ViewOnly,
            password: None,
            expiration: None,
            download: DownloadPermission::AllowDownload,
            message: None,
            notify_recipients: false,
            domain_restriction: None,
            watermark: false,
        }
    }
}

impl SharingOptions {
    /// Creates a new instance with default settings
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Creates a new instance with the specified privacy level
    pub fn with_privacy(privacy: PrivacyLevel) -> Self {
        let mut options = Self::default();
        options.privacy = privacy;
        options
    }
    
    /// Creates a new instance with the specified access permission level
    pub fn with_permission(permission: AccessPermission) -> Self {
        let mut options = Self::default();
        options.permission = permission;
        options
    }
    
    /// Creates a password-protected shared resource
    pub fn with_password(password: &str) -> Self {
        let mut options = Self::default();
        options.password = Some(password.to_string());
        options
    }
    
    /// Creates a shared resource with an expiration date
    pub fn with_expiration(expiration: ZonedDateTime) -> Self {
        let mut options = Self::default();
        options.expiration = Some(expiration);
        options
    }
    
    /// Sets whether downloads are allowed
    pub fn set_download_permission(&mut self, permission: DownloadPermission) -> &mut Self {
        self.download = permission;
        self
    }
    
    /// Adds a password to this shared resource
    pub fn set_password(&mut self, password: Option<&str>) -> &mut Self {
        self.password = password.map(|s| s.to_string());
        self
    }
    
    /// Sets an expiration date for this shared resource
    pub fn set_expiration(&mut self, expiration: Option<ZonedDateTime>) -> &mut Self {
        self.expiration = expiration;
        self
    }
    
    /// Sets the privacy level for this shared resource
    pub fn set_privacy(&mut self, privacy: PrivacyLevel) -> &mut Self {
        self.privacy = privacy;
        self
    }
    
    /// Sets the access permission level for this shared resource
    pub fn set_permission(&mut self, permission: AccessPermission) -> &mut Self {
        self.permission = permission;
        self
    }
    
    /// Sets a custom message to display to recipients
    pub fn set_message(&mut self, message: Option<&str>) -> &mut Self {
        self.message = message.map(|s| s.to_string());
        self
    }
    
    /// Sets whether to notify recipients when the resource is shared
    pub fn set_notify_recipients(&mut self, notify: bool) -> &mut Self {
        self.notify_recipients = notify;
        self
    }
    
    /// Sets domain restrictions for accessing the shared resource
    pub fn set_domain_restriction(&mut self, domain: Option<&str>) -> &mut Self {
        self.domain_restriction = domain.map(|s| s.to_string());
        self
    }
    
    /// Sets whether to apply a watermark when the document is viewed
    pub fn set_watermark(&mut self, watermark: bool) -> &mut Self {
        self.watermark = watermark;
        self
    }
}

/// Represents the result of creating a shared resource
#[derive(Debug, Clone)]
pub struct SharingResult {
    /// The unique URL for accessing the shared resource
    pub url: String,
    
    /// The applied sharing options
    pub options: SharingOptions,
    
    /// Identifier for this shared resource
    pub id: String,
    
    /// When this shared resource was created
    pub created_at: ZonedDateTime,
    
    /// Statistics about this shared resource
    pub stats: SharingStats,
}

/// Statistics about a shared resource
#[derive(Debug, Clone, Default)]
pub struct SharingStats {
    /// Number of times the resource has been viewed
    pub views: u64,
    
    /// Number of times the resource has been downloaded
    pub downloads: u64,
    
    /// Last time the resource was accessed
    pub last_accessed: Option<ZonedDateTime>,
} 