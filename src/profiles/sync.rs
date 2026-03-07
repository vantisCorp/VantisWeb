//! Profile Synchronization
//!
//! Cloud synchronization for profiles across devices.

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Sync status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncStatus {
    /// Not synced
    NotSynced,
    /// Syncing in progress
    Syncing,
    /// Synced successfully
    Synced,
    /// Sync failed
    Failed(String),
    /// Conflict detected
    Conflict,
}

/// Sync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Sync enabled
    pub enabled: bool,
    /// Sync provider
    pub provider: SyncProvider,
    /// Sync interval in seconds
    pub sync_interval: u64,
    /// Last sync timestamp
    pub last_sync: Option<DateTime<Utc>>,
    /// Auto-sync on changes
    pub auto_sync: bool,
}

/// Sync provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncProvider {
    /// Local storage
    Local,
    /// Custom provider
    Custom {
        /// Provider URL
        url: String,
        /// API key
        api_key: Option<String>,
    },
}

/// Synced profile data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncedProfile {
    /// Profile ID
    pub profile_id: String,
    /// Profile name
    pub name: String,
    /// Profile data
    pub data: serde_json::Value,
    /// Version
    pub version: u64,
    /// Last modified timestamp
    pub last_modified: DateTime<Utc>,
    /// Device ID
    pub device_id: String,
}

/// Sync conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConflict {
    /// Profile ID
    pub profile_id: String,
    /// Local version
    pub local: SyncedProfile,
    /// Remote version
    pub remote: SyncedProfile,
    /// Conflict timestamp
    pub timestamp: DateTime<Utc>,
}

/// Profile sync manager
pub struct ProfileSyncManager {
    /// Sync configuration
    config: SyncConfig,
    /// Device ID
    device_id: String,
    /// Synced profiles
    synced_profiles: HashMap<String, SyncedProfile>,
    /// Conflicts
    conflicts: Vec<SyncConflict>,
    /// Profiles directory
    profiles_dir: PathBuf,
}

impl ProfileSyncManager {
    /// Creates a new profile sync manager
    pub fn new(profiles_dir: PathBuf) -> Self {
        Self {
            config: SyncConfig {
                enabled: false,
                provider: SyncProvider::Local,
                sync_interval: 300, // 5 minutes
                last_sync: None,
                auto_sync: true,
            },
            device_id: uuid::Uuid::new_v4().to_string(),
            synced_profiles: HashMap::new(),
            conflicts: Vec::new(),
            profiles_dir,
        }
    }

    /// Enables sync
    pub fn enable(&mut self, provider: SyncProvider) -> Result<()> {
        self.config.enabled = true;
        self.config.provider = provider;
        self.config.last_sync = None;
        Ok(())
    }

    /// Disables sync
    pub fn disable(&mut self) -> Result<()> {
        self.config.enabled = false;
        self.synced_profiles.clear();
        self.conflicts.clear();
        Ok(())
    }

    /// Checks if sync is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Gets sync status
    pub fn get_status(&self) -> SyncStatus {
        if !self.config.enabled {
            return SyncStatus::NotSynced;
        }

        if !self.conflicts.is_empty() {
            return SyncStatus::Conflict;
        }

        if self.config.last_sync.is_some() {
            SyncStatus::Synced
        } else {
            SyncStatus::NotSynced
        }
    }

    /// Syncs all profiles
    pub fn sync(&mut self) -> Result<SyncResult> {
        if !self.config.enabled {
            return Err(anyhow!("Sync is not enabled"));
        }

        log::info!("Starting profile sync...");

        let mut result = SyncResult {
            synced: 0,
            failed: 0,
            conflicts: 0,
            timestamp: Utc::now(),
        };

        // Load local profiles
        let local_profiles = self.load_local_profiles()?;

        // Sync with remote
        match &self.config.provider {
            SyncProvider::Local => {
                // Local sync - just update last sync time
                for profile in local_profiles {
                    self.synced_profiles.insert(profile.profile_id.clone(), profile);
                    result.synced += 1;
                }
            }
            SyncProvider::Custom { url, api_key } => {
                // Remote sync
                let remote_profiles = self.fetch_remote_profiles(url, api_key)?;

                // Merge profiles
                for local_profile in &local_profiles {
                    if let Some(remote_profile) = remote_profiles.get(&local_profile.profile_id) {
                        // Check for conflicts
                        if local_profile.last_modified != remote_profile.last_modified {
                            // Conflict detected
                            self.conflicts.push(SyncConflict {
                                profile_id: local_profile.profile_id.clone(),
                                local: local_profile.clone(),
                                remote: remote_profile.clone(),
                                timestamp: Utc::now(),
                            });
                            result.conflicts += 1;
                        } else {
                            // No conflict, use local version
                            self.synced_profiles.insert(local_profile.profile_id.clone(), local_profile.clone());
                            result.synced += 1;
                        }
                    } else {
                        // New local profile
                        self.synced_profiles.insert(local_profile.profile_id.clone(), local_profile.clone());
                        result.synced += 1;
                    }
                }

                // Check for new remote profiles
                for (profile_id, remote_profile) in &remote_profiles {
                    if !self.synced_profiles.contains_key(profile_id) {
                        self.synced_profiles.insert(profile_id.clone(), remote_profile.clone());
                        result.synced += 1;
                    }
                }
            }
        }

        self.config.last_sync = Some(Utc::now());
        log::info!("Profile sync completed: {} synced, {} failed, {} conflicts", 
                   result.synced, result.failed, result.conflicts);

        Ok(result)
    }

    /// Loads local profiles
    fn load_local_profiles(&self) -> Result<Vec<SyncedProfile>> {
        let mut profiles = Vec::new();

        if !self.profiles_dir.exists() {
            return Ok(profiles);
        }

        for entry in std::fs::read_dir(&self.profiles_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let profile_file = path.join("profile.json");
                if profile_file.exists() {
                    let content = std::fs::read_to_string(&profile_file)?;
                    let data: serde_json::Value = serde_json::from_str(&content)?;

                    let profile = SyncedProfile {
                        profile_id: path.file_name().unwrap().to_string_lossy().to_string(),
                        name: data["name"].as_str().unwrap_or("Unknown").to_string(),
                        data,
                        version: 1,
                        last_modified: Utc::now(),
                        device_id: self.device_id.clone(),
                    };

                    profiles.push(profile);
                }
            }
        }

        Ok(profiles)
    }

    /// Fetches remote profiles from sync server
    fn fetch_remote_profiles(&self, url: &str, api_key: &Option<String>) -> Result<HashMap<String, SyncedProfile>> {
        let mut profiles = HashMap::new();
        
        // Build the client
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("VantisWeb-Sync/1.0")
            .build()
            .map_err(|e| anyhow!("Failed to build HTTP client: {}", e))?;
        
        // Build request
        let mut request = client.get(format!("{}/api/v1/profiles", url.trim_end_matches('/')));
        
        // Add authentication if API key is provided
        if let Some(key) = api_key {
            request = request.bearer_auth(key);
        }
        
        // Add device ID header for tracking
        request = request.header("X-Device-ID", &self.device_id);
        
        // Execute request
        let response = request.send()
            .map_err(|e| anyhow!("Failed to fetch remote profiles: {}", e))?;
        
        // Check response status
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(anyhow!("Remote sync failed with status {}: {}", status, body));
        }
        
        // Parse response
        let response_data: SyncListResponse = response.json()
            .map_err(|e| anyhow!("Failed to parse sync response: {}", e))?;
        
        // Convert to HashMap
        for profile in response_data.profiles {
            profiles.insert(profile.profile_id.clone(), profile);
        }
        
        log::info!("Fetched {} remote profiles from {}", profiles.len(), url);
        Ok(profiles)
    }
    
    /// Pushes local profile changes to remote server
    pub fn push_profile(&self, profile: &SyncedProfile) -> Result<()> {
        if !self.config.enabled {
            return Err(anyhow!("Sync is not enabled"));
        }
        
        let (url, api_key) = match &self.config.provider {
            SyncProvider::Local => {
                log::debug!("Local sync - skipping push");
                return Ok(());
            }
            SyncProvider::Custom { url, api_key } => (url, api_key),
        };
        
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("VantisWeb-Sync/1.0")
            .build()
            .map_err(|e| anyhow!("Failed to build HTTP client: {}", e))?;
        
        let mut request = client.post(format!("{}/api/v1/profiles/{}", url.trim_end_matches('/'), profile.profile_id));
        
        if let Some(key) = api_key {
            request = request.bearer_auth(key);
        }
        
        request = request.header("X-Device-ID", &self.device_id);
        request = request.json(profile);
        
        let response = request.send()
            .map_err(|e| anyhow!("Failed to push profile: {}", e))?;
        
        if !response.status().is_success() {
            let status = response.status();
            return Err(anyhow!("Failed to push profile: status {}", status));
        }
        
        log::info!("Pushed profile {} to remote", profile.profile_id);
        Ok(())
    }
    
    /// Deletes a profile from remote server
    pub fn delete_remote_profile(&self, profile_id: &str) -> Result<()> {
        if !self.config.enabled {
            return Err(anyhow!("Sync is not enabled"));
        }
        
        let (url, api_key) = match &self.config.provider {
            SyncProvider::Local => return Ok(()),
            SyncProvider::Custom { url, api_key } => (url, api_key),
        };
        
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| anyhow!("Failed to build HTTP client: {}", e))?;
        
        let mut request = client.delete(format!("{}/api/v1/profiles/{}", url.trim_end_matches('/'), profile_id));
        
        if let Some(key) = api_key {
            request = request.bearer_auth(key);
        }
        
        let response = request.send()
            .map_err(|e| anyhow!("Failed to delete remote profile: {}", e))?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Failed to delete remote profile: status {}", response.status()));
        }
        
        log::info!("Deleted profile {} from remote", profile_id);
        Ok(())
    }

    /// Gets a synced profile
    pub fn get_synced_profile(&self, profile_id: &str) -> Option<&SyncedProfile> {
        self.synced_profiles.get(profile_id)
    }

    /// Gets all synced profiles
    pub fn get_all_synced_profiles(&self) -> Vec<&SyncedProfile> {
        self.synced_profiles.values().collect()
    }

    /// Gets conflicts
    pub fn get_conflicts(&self) -> &[SyncConflict] {
        &self.conflicts
    }

    /// Resolves a conflict
    pub fn resolve_conflict(&mut self, profile_id: &str, use_local: bool) -> Result<()> {
        let conflict_index = self.conflicts
            .iter()
            .position(|c| c.profile_id == profile_id)
            .ok_or_else(|| anyhow!("Conflict not found for profile: {}", profile_id))?;

        let conflict = self.conflicts.remove(conflict_index);

        let profile = if use_local {
            conflict.local
        } else {
            conflict.remote
        };

        self.synced_profiles.insert(profile_id.to_string(), profile);

        Ok(())
    }

    /// Exports profiles to backup
    pub fn export_backup(&self) -> Result<String> {
        let backup = BackupData {
            version: 1,
            timestamp: Utc::now(),
            device_id: self.device_id.clone(),
            profiles: self.synced_profiles.values().cloned().collect(),
        };

        serde_json::to_string_pretty(&backup)
            .map_err(|e| anyhow!("Failed to export backup: {}", e))
    }

    /// Imports profiles from backup
    pub fn import_backup(&mut self, backup: &str) -> Result<()> {
        let backup_data: BackupData = serde_json::from_str(backup)
            .map_err(|e| anyhow!("Failed to import backup: {}", e))?;

        for profile in backup_data.profiles {
            self.synced_profiles.insert(profile.profile_id.clone(), profile);
        }

        Ok(())
    }

    /// Gets sync configuration
    pub fn get_config(&self) -> &SyncConfig {
        &self.config
    }

    /// Sets sync configuration
    pub fn set_config(&mut self, config: SyncConfig) -> Result<()> {
        self.config = config;
        Ok(())
    }
}

/// Sync result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    /// Number of profiles synced
    pub synced: usize,
    /// Number of profiles that failed to sync
    pub failed: usize,
    /// Number of conflicts detected
    pub conflicts: usize,
    /// Sync timestamp
    pub timestamp: DateTime<Utc>,
}

/// Sync list response from remote server
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SyncListResponse {
    /// List of profiles
    profiles: Vec<SyncedProfile>,
    /// Server timestamp
    timestamp: Option<DateTime<Utc>>,
    /// Server version
    server_version: Option<String>,
}

/// Backup data
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BackupData {
    /// Backup version
    version: u64,
    /// Backup timestamp
    timestamp: DateTime<Utc>,
    /// Device ID
    device_id: String,
    /// Profiles
    profiles: Vec<SyncedProfile>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_sync_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ProfileSyncManager::new(temp_dir.path().to_path_buf());

        assert!(!manager.is_enabled());
        assert_eq!(manager.get_status(), SyncStatus::NotSynced);
    }

    #[test]
    fn test_enable_sync() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = ProfileSyncManager::new(temp_dir.path().to_path_buf());

        manager.enable(SyncProvider::Local).unwrap();

        assert!(manager.is_enabled());
    }

    #[test]
    fn test_disable_sync() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = ProfileSyncManager::new(temp_dir.path().to_path_buf());

        manager.enable(SyncProvider::Local).unwrap();
        manager.disable().unwrap();

        assert!(!manager.is_enabled());
    }

    #[test]
    fn test_export_import_backup() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = ProfileSyncManager::new(temp_dir.path().to_path_buf());

        let backup = manager.export_backup().unwrap();

        let mut new_manager = ProfileSyncManager::new(temp_dir.path().to_path_buf());
        new_manager.import_backup(&backup).unwrap();

        assert_eq!(new_manager.device_id, manager.device_id);
    }
}