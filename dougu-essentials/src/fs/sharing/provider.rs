use crate::core::Result;
use crate::fs::sharing::{SharingOptions, SharingResult, SharingStats};
use crate::fs::FsPath;
use std::fmt::Debug;

/// Trait that defines operations related to file/folder sharing
pub trait SharingProvider: Debug {
    /// Creates a new share link for the specified path with the given options
    fn create_share_link(&self, path: &FsPath, options: SharingOptions) -> Result<SharingResult>;
    
    /// Updates the sharing options for an existing share
    fn update_share_options(&self, share_id: &str, options: SharingOptions) -> Result<SharingResult>;
    
    /// Lists all active shares for the specified path
    fn list_shares(&self, path: &FsPath) -> Result<Vec<SharingResult>>;
    
    /// Gets information about a specific share
    fn get_share_info(&self, share_id: &str) -> Result<SharingResult>;
    
    /// Gets the latest statistics for a shared resource
    fn get_share_stats(&self, share_id: &str) -> Result<SharingStats>;
    
    /// Revokes/deletes an existing share link
    fn revoke_share(&self, share_id: &str) -> Result<()>;
    
    /// Checks if a specific path has any active shares
    fn has_shares(&self, path: &FsPath) -> Result<bool> {
        Ok(!self.list_shares(path)?.is_empty())
    }
    
    /// Revokes all shares for a specific path
    fn revoke_all_shares(&self, path: &FsPath) -> Result<()> {
        let shares = self.list_shares(path)?;
        for share in shares {
            self.revoke_share(&share.id)?;
        }
        Ok(())
    }
} 