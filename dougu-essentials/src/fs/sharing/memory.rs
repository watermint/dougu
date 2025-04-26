use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::core::Result;
use crate::fs::sharing::{SharingOptions, SharingResult, SharingStats};
use crate::fs::sharing::provider::SharingProvider;
use crate::fs::FsPath;
use crate::time::now_utc;

/// In-memory implementation of the `SharingProvider` trait for testing purposes
#[derive(Debug)]
pub struct MemorySharingProvider {
    shares: Arc<Mutex<HashMap<String, SharingEntry>>>,
}

#[derive(Debug, Clone)]
struct SharingEntry {
    id: String,
    path: FsPath,
    options: SharingOptions,
    url: String,
    created_at: i64,
    stats: SharingStats,
}

impl MemorySharingProvider {
    /// Creates a new in-memory sharing provider
    pub fn new() -> Self {
        Self {
            shares: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Generate a fake sharing URL for testing
    fn generate_url(&self, path: &FsPath, id: &str) -> String {
        format!("https://example.com/share/{}/{}", id, path.display())
    }
}

impl Default for MemorySharingProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SharingProvider for MemorySharingProvider {
    fn create_share_link(&self, path: &FsPath, options: SharingOptions) -> Result<SharingResult> {
        let mut shares = self.shares.lock().unwrap();
        let id = Uuid::new_v4().to_string();
        let url = self.generate_url(path, &id);
        let created_at = now_utc().timestamp();
        
        let entry = SharingEntry {
            id: id.clone(),
            path: path.clone(),
            options: options.clone(),
            url: url.clone(),
            created_at,
            stats: SharingStats::default(),
        };
        
        shares.insert(id.clone(), entry);
        
        Ok(SharingResult {
            id,
            url,
            options,
            created_at,
            stats: SharingStats::default(),
        })
    }
    
    fn update_share_options(&self, share_id: &str, options: SharingOptions) -> Result<SharingResult> {
        let mut shares = self.shares.lock().unwrap();
        
        if let Some(entry) = shares.get_mut(share_id) {
            entry.options = options.clone();
            
            Ok(SharingResult {
                id: entry.id.clone(),
                url: entry.url.clone(),
                options,
                created_at: entry.created_at,
                stats: entry.stats.clone(),
            })
        } else {
            Err(crate::core::Error::msg("Share not found"))
        }
    }
    
    fn list_shares(&self, path: &FsPath) -> Result<Vec<SharingResult>> {
        let shares = self.shares.lock().unwrap();
        
        let results = shares
            .iter()
            .filter(|(_, entry)| entry.path == *path)
            .map(|(_, entry)| SharingResult {
                id: entry.id.clone(),
                url: entry.url.clone(),
                options: entry.options.clone(),
                created_at: entry.created_at,
                stats: entry.stats.clone(),
            })
            .collect();
        
        Ok(results)
    }
    
    fn get_share_info(&self, share_id: &str) -> Result<SharingResult> {
        let shares = self.shares.lock().unwrap();
        
        if let Some(entry) = shares.get(share_id) {
            Ok(SharingResult {
                id: entry.id.clone(),
                url: entry.url.clone(),
                options: entry.options.clone(),
                created_at: entry.created_at,
                stats: entry.stats.clone(),
            })
        } else {
            Err(crate::core::Error::msg("Share not found"))
        }
    }
    
    fn get_share_stats(&self, share_id: &str) -> Result<SharingStats> {
        let shares = self.shares.lock().unwrap();
        
        if let Some(entry) = shares.get(share_id) {
            Ok(entry.stats.clone())
        } else {
            Err(crate::core::Error::msg("Share not found"))
        }
    }
    
    fn revoke_share(&self, share_id: &str) -> Result<()> {
        let mut shares = self.shares.lock().unwrap();
        
        if shares.remove(share_id).is_some() {
            Ok(())
        } else {
            Err(crate::core::Error::msg("Share not found"))
        }
    }
    
    fn has_shares(&self, path: &FsPath) -> Result<bool> {
        let shares = self.shares.lock().unwrap();
        
        Ok(shares.values().any(|entry| entry.path == *path))
    }
    
    fn revoke_all_shares(&self, path: &FsPath) -> Result<()> {
        let mut shares = self.shares.lock().unwrap();
        
        let ids_to_remove: Vec<String> = shares
            .iter()
            .filter(|(_, entry)| entry.path == *path)
            .map(|(id, _)| id.clone())
            .collect();
        
        for id in ids_to_remove {
            shares.remove(&id);
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_sharing_provider() -> Result<()> {
        let provider = MemorySharingProvider::new();
        let path = FsPath::new("test/path/file.txt")?;
        let options = SharingOptions::default();
        
        // Test creating a share
        let share_info = provider.create_share_link(&path, options.clone())?;
        assert_eq!(share_info.options, options);
        
        // Test has_shares
        assert!(provider.has_shares(&path)?);
        assert!(!provider.has_shares(&FsPath::new("non/existent/path")?)?);
        
        // Test listing shares
        let shares = provider.list_shares(&path)?;
        assert_eq!(shares.len(), 1);
        
        // Test getting share info
        let retrieved_info = provider.get_share_info(&shares[0].id)?;
        assert_eq!(retrieved_info.id, shares[0].id);
        
        // Test updating share options
        let mut updated_options = SharingOptions::default();
        updated_options.set_expiration(Some(crate::time::now_utc() + crate::time::Duration::days(30)));
        
        provider.update_share_options(&shares[0].id, updated_options.clone())?;
        let updated_info = provider.get_share_info(&shares[0].id)?;
        assert!(updated_info.options.expiration.is_some());
        
        // Test revoking a specific share
        provider.revoke_share(&shares[0].id)?;
        assert!(provider.get_share_info(&shares[0].id).is_err());
        
        // Test revoking all shares
        let share1 = provider.create_share_link(&path, SharingOptions::default())?;
        let share2 = provider.create_share_link(&path, SharingOptions::default())?;
        assert_eq!(provider.list_shares(&path)?.len(), 2);
        
        provider.revoke_all_shares(&path)?;
        assert_eq!(provider.list_shares(&path)?.len(), 0);
        
        Ok(())
    }
} 