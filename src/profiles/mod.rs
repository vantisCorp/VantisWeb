//! Profiles Module
//! 
//! User profile management:
//! - Multiple profiles (Work, Gaming, Private)
//! - Profile isolation (Vantis Shifter)
//! - Vantis ID management
//! - Profile synchronization

use anyhow::Result;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod templates;
pub mod sync;
pub mod analytics;
pub mod security;

use crate::core::kernel::VantisKernel;

pub use templates::{TemplateManager, ProfileTemplate, TemplateCategory, TemplateSettings, PrivacySettings, PerformanceSettings, CPUPriority};
pub use sync::{ProfileSyncManager, SyncConfig, SyncProvider, SyncStatus, SyncedProfile, SyncConflict, SyncResult};
pub use analytics::{AnalyticsManager, ProfileAnalytics, DailyUsage, WebsiteUsage, TabStatistics, PerformanceMetrics, UsageSummary};
pub use security::{ProfileSecurityManager, ProfileSecurity, SecurityLevel, AuthMethod};

/// Profile type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProfileType {
    /// Default profile
    Default,
    /// Work profile
    Work,
    /// Gaming profile
    Gaming,
    /// Private profile
    Private,
    /// Custom profile
    Custom(String),
}

/// Profile configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    /// Profile ID
    pub id: String,
    /// Profile name
    pub name: String,
    /// Profile type
    pub profile_type: ProfileType,
    /// Profile icon
    pub icon: Option<String>,
    /// Profile color
    pub color: Option<String>,
    /// Is profile active
    pub active: bool,
    /// Profile settings
    pub settings: HashMap<String, serde_json::Value>,
    /// Profile bookmarks
    pub bookmarks: Vec<String>,
    /// Profile history
    pub history: Vec<String>,
    /// Profile extensions
    pub extensions: Vec<String>,
    /// Profile theme
    pub theme: Option<String>,
    /// Created at
    pub created_at: i64,
    /// Last used at
    pub last_used_at: i64,
}

impl ProfileConfig {
    /// Create a new profile
    pub fn new(name: String, profile_type: ProfileType) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            profile_type,
            icon: None,
            color: None,
            active: false,
            settings: HashMap::new(),
            bookmarks: Vec::new(),
            history: Vec::new(),
            extensions: Vec::new(),
            theme: None,
            created_at: now,
            last_used_at: now,
        }
    }
}

/// Profile manager
pub struct ProfileManager {
    kernel: Arc<VantisKernel>,
    profiles: Arc<RwLock<HashMap<String, ProfileConfig>>>,
    active_profile: Arc<RwLock<Option<String>>>,
    profiles_dir: PathBuf,
}

impl ProfileManager {
    /// Create a new profile manager
    pub fn new(kernel: Arc<VantisKernel>) -> Result<Self> {
        info!("Initializing Profile Manager...");

        let profiles_dir = std::path::PathBuf::from(".vantisweb/profiles");
        std::fs::create_dir_all(&profiles_dir)?;

        let manager = Self {
            kernel,
            profiles: Arc::new(RwLock::new(HashMap::new())),
            active_profile: Arc::new(RwLock::new(None)),
            profiles_dir,
        };

        // Load existing profiles
        manager.load_profiles()?;

        Ok(manager)
    }

    /// Initialize profile manager (async)
    pub async fn initialize(&self) -> Result<()> {
        // Create default profile if none exists
        let profiles = self.profiles.read().await;
        if profiles.is_empty() {
            drop(profiles);
            self.create_default_profile().await?;
        }
        Ok(())
    }

    /// Create default profile
    async fn create_default_profile(&self) -> Result<()> {
        info!("Creating default profile");
        let default_profile = ProfileConfig::new("Default".to_string(), ProfileType::Default);
        self.add_profile(default_profile).await?;
        Ok(())
    }

    /// Load profiles from disk
    fn load_profiles(&self) -> Result<()> {
        debug!("Loading profiles from disk");

        let mut profiles = HashMap::new();

        if let Ok(entries) = std::fs::read_dir(&self.profiles_dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(file_name) = entry.file_name().to_str() {
                            if file_name.ends_with(".json") {
                                let path = entry.path();
                                if let Ok(content) = std::fs::read_to_string(&path) {
                                    if let Ok(profile) = serde_json::from_str::<ProfileConfig>(&content) {
                                        profiles.insert(profile.id.clone(), profile);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Update profiles
        let profiles_arc = self.profiles.clone();
        tokio::spawn(async move {
            *profiles_arc.write().await = profiles;
        });

        Ok(())
    }

    /// Save profile to disk
    fn save_profile(&self, profile: &ProfileConfig) -> Result<()> {
        let profile_path = self.profiles_dir.join(format!("{}.json", profile.id));
        let content = serde_json::to_string_pretty(profile)?;
        std::fs::write(profile_path, content)?;
        Ok(())
    }

    /// Add a new profile
    pub async fn add_profile(&self, profile: ProfileConfig) -> Result<()> {
        info!("Adding profile: {}", profile.name);

        let profile_id = profile.id.clone();

        // Save to disk
        self.save_profile(&profile)?;

        // Add to memory
        self.profiles.write().await.insert(profile_id.clone(), profile);

        Ok(())
    }

    /// Get profile by ID
    pub async fn get_profile(&self, profile_id: &str) -> Option<ProfileConfig> {
        self.profiles.read().await.get(profile_id).cloned()
    }

    /// Get all profiles
    pub async fn get_all_profiles(&self) -> Vec<ProfileConfig> {
        self.profiles.read().await.values().cloned().collect()
    }

    /// Get active profile
    pub async fn get_active_profile(&self) -> Option<ProfileConfig> {
        let active_id = self.active_profile.read().await.clone()?;
        self.get_profile(&active_id).await
    }

    /// Set active profile
    pub async fn set_active_profile(&self, profile_id: &str) -> Result<()> {
        info!("Setting active profile: {}", profile_id);

        // Check if profile exists
        if !self.profiles.read().await.contains_key(profile_id) {
            return Err(anyhow::anyhow!("Profile not found: {}", profile_id));
        }

        // Deactivate current profile
        if let Some(current_id) = self.active_profile.read().await.as_ref() {
            if let Some(profile) = self.profiles.write().await.get_mut(current_id) {
                profile.active = false;
                self.save_profile(profile)?;
            }
        }

        // Activate new profile
        if let Some(profile) = self.profiles.write().await.get_mut(profile_id) {
            profile.active = true;
            profile.last_used_at = chrono::Utc::now().timestamp_millis();
            self.save_profile(profile)?;
        }

        *self.active_profile.write().await = Some(profile_id.to_string());

        Ok(())
    }

    /// Delete profile
    pub async fn delete_profile(&self, profile_id: &str) -> Result<()> {
        info!("Deleting profile: {}", profile_id);

        // Check if profile is active
        if let Some(active_id) = self.active_profile.read().await.as_ref() {
            if active_id == profile_id {
                return Err(anyhow::anyhow!("Cannot delete active profile"));
            }
        }

        // Remove from memory
        self.profiles.write().await.remove(profile_id);

        // Remove from disk
        let profile_path = self.profiles_dir.join(format!("{}.json", profile_id));
        std::fs::remove_file(profile_path)?;

        Ok(())
    }

    /// Update profile settings
    pub async fn update_profile_settings(&self, profile_id: &str, settings: HashMap<String, serde_json::Value>) -> Result<()> {
        debug!("Updating profile settings: {}", profile_id);

        if let Some(profile) = self.profiles.write().await.get_mut(profile_id) {
            profile.settings = settings;
            self.save_profile(profile)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_profile_creation() {
        let profile = ProfileConfig::new("Test Profile".to_string(), ProfileType::Custom("Test".to_string()));
        assert_eq!(profile.name, "Test Profile");
        assert!(!profile.active);
    }

    #[tokio::test]
    async fn test_profile_manager() {
        let kernel = Arc::new(VantisKernel::new().await.unwrap());
        let manager = ProfileManager::new(kernel).unwrap();
        let profiles = manager.get_all_profiles().await;
        assert!(!profiles.is_empty());
    }
}