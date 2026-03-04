//! Profiles Module
//! 
//! User profile management:
//! - Multiple profiles (Work, Gaming, Private)
//! - Profile isolation (Vantis Shifter)
//! - Vantis ID management
//! - Profile synchronization
//! - Profile import/export

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
pub mod import_export;
pub mod analytics_visualization;
pub mod community_templates;
pub mod template_manager;
pub mod profile_comparison;

use crate::core::kernel::VantisKernel;

pub use templates::{TemplateManager, ProfileTemplate, TemplateCategory, TemplateSettings, PrivacySettings, PerformanceSettings, CPUPriority};
pub use sync::{ProfileSyncManager, SyncConfig, SyncProvider, SyncStatus, SyncedProfile, SyncConflict, SyncResult};
pub use analytics::{AnalyticsManager, ProfileAnalytics, DailyUsage, WebsiteUsage, TabStatistics, PerformanceMetrics, UsageSummary};
pub use analytics_visualization::{
    VisualizationManager, WebsiteCategory, HeatmapCell, TrendDataPoint,
    ProfileComparison, ComparisonMetrics, ExportFormat, DateRange, AnalyticsReport
};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_profile_creation() {
        let profile = ProfileConfig::new("Test Profile".to_string(), ProfileType::Custom("Test".to_string()));
        assert_eq!(profile.name, "Test Profile");
        assert!(!profile.active);
    }
}
pub use security::{ProfileSecurityManager, ProfileSecurity, SecurityLevel, AuthMethod};
pub use import_export::{
    ProfileExport, ProfilesExport, ImportOptions, ExportOptions,
    ImportResult, export_profile_to_file, export_profiles_to_file,
    import_profile_from_file, import_profiles_from_file, validate_import_file
};
pub use community_templates::{
    TemplateMetadata, CommunityTemplate, TemplateReview, TemplateFilter,
    TemplateSortOrder, TemplateSubmission, ModerationReport, ModerationStatus,
    CreatorInfo, TemplateApiResponse, TemplateListResponse, TemplateStatistics
};
pub use template_manager::{TemplateManager, TemplateError};
pub use profile_comparison::{
    ProfileComparison, ProfileComparisonManager, SettingsDiff, BookmarksDiff,
    ExtensionsDiff, SecurityDiff, HistoryDiff, ComparisonOptions, ComparisonReport,
    MergeOperation, MergeItem, MergeItemType, MergeStrategy
};

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
    /// Profile order position (for drag and drop reordering)
    pub order: i32,
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
            order: 0,
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

    /// Export a single profile to a file
    pub fn export_profile(&self, profile_id: &str, path: &Path, options: &ExportOptions) -> Result<()> {
        let profile = self.profiles.blocking_read()
            .get(profile_id)
            .context("Profile not found")?
            .clone();
        
        export_profile_to_file(&profile, path, options)
            .context("Failed to export profile")?;
        
        Ok(())
    }

    /// Export all profiles to a file
    pub fn export_all_profiles(&self, path: &Path, options: &ExportOptions) -> Result<()> {
        let profiles: Vec<ProfileConfig> = self.profiles.blocking_read()
            .values()
            .cloned()
            .collect();
        
        export_profiles_to_file(&profiles, path, options)
            .context("Failed to export profiles")?;
        
        Ok(())
    }

    /// Import a single profile from a file
    pub async fn import_profile(&self, path: &Path, password: Option<&str>, options: &ImportOptions) -> Result<ImportResult> {
        let (profile, mut result) = import_profile_from_file(path, password, options)
            .context("Failed to import profile")?;
        
        // Check if profile with same name exists
        let existing_profile = self.profiles.read().await
            .values()
            .find(|p| p.name == profile.name);
        
        if let Some(existing) = existing_profile {
            if options.overwrite {
                // Delete existing profile
                self.delete_profile(&existing.id).await?;
                debug!("Deleted existing profile: {}", existing.name);
            } else {
                warn!("Profile '{}' already exists, skipping", profile.name);
                result.skipped_count += 1;
                return Ok(result);
            }
        }
        
        // Add imported profile
        self.add_profile(profile).await?;
        
        Ok(result)
    }

    /// Import multiple profiles from a file
    pub async fn import_profiles(&self, path: &Path, password: Option<&str>, options: &ImportOptions) -> Result<ImportResult> {
        let (profiles, mut result) = import_profiles_from_file(path, password, options)
            .context("Failed to import profiles")?;
        
        let mut imported_count = 0;
        for profile in profiles {
            // Check if profile with same name exists
            let existing_profile = self.profiles.read().await
                .values()
                .find(|p| p.name == profile.name);
            
            if let Some(existing) = existing_profile {
                if options.overwrite {
                    // Delete existing profile
                    self.delete_profile(&existing.id).await?;
                    debug!("Deleted existing profile: {}", existing.name);
                } else {
                    warn!("Profile '{}' already exists, skipping", profile.name);
                    result.skipped_count += 1;
                    continue;
                }
            }
            
            // Add imported profile
            self.add_profile(profile).await?;
            imported_count += 1;
        }
        
        Ok(result)
    }

    /// Validate import file without importing
    pub fn validate_import(&self, path: &Path, password: Option<&str>) -> Result<ProfilesExport> {
        validate_import_file(path, password)
            .context("Failed to validate import file")?;
        
        Ok(validate_import_file(path, password)?)
    }

    /// Reorder profiles - move a profile to a new position
    pub async fn reorder_profile(&self, profile_id: &str, new_position: i32) -> Result<()> {
        info!("Reordering profile {} to position {}", profile_id, new_position);
        
        let mut profiles = self.profiles.write().await;
        let total_profiles = profiles.len() as i32;
        
        // Validate new position
        if new_position < 0 || new_position >= total_profiles {
            return Err(anyhow::anyhow!("Invalid position: {}", new_position));
        }
        
        // Find the profile and its current position
        let current_index = profiles.iter().position(|p| p.id == profile_id)
            .ok_or_else(|| anyhow::anyhow!("Profile not found: {}", profile_id))?;
        
        let current_position = profiles[current_index].order;
        
        // If already at the target position, nothing to do
        if current_position == new_position {
            return Ok(());
        }
        
        // Update order values for affected profiles
        if new_position < current_position {
            // Moving up: shift profiles between new_position and current_position down
            for profile in profiles.iter_mut() {
                if profile.order >= new_position && profile.order < current_position {
                    profile.order += 1;
                }
            }
        } else {
            // Moving down: shift profiles between current_position and new_position up
            for profile in profiles.iter_mut() {
                if profile.order > current_position && profile.order <= new_position {
                    profile.order -= 1;
                }
            }
        }
        
        // Set the new position for the moved profile
        profiles[current_index].order = new_position;
        
        // Persist changes
        drop(profiles);
        self.save_profiles().await?;
        
        info!("Profile {} reordered to position {}", profile_id, new_position);
        Ok(())
    }

    /// Swap two profiles' positions
    pub async fn swap_profiles(&self, profile_id_1: &str, profile_id_2: &str) -> Result<()> {
        info!("Swapping profiles {} and {}", profile_id_1, profile_id_2);
        
        let mut profiles = self.profiles.write().await;
        
        // Find both profiles
        let index1 = profiles.iter().position(|p| p.id == profile_id_1)
            .ok_or_else(|| anyhow::anyhow!("Profile not found: {}", profile_id_1))?;
        let index2 = profiles.iter().position(|p| p.id == profile_id_2)
            .ok_or_else(|| anyhow::anyhow!("Profile not found: {}", profile_id_2))?;
        
        // Swap their order values
        let order1 = profiles[index1].order;
        let order2 = profiles[index2].order;
        profiles[index1].order = order2;
        profiles[index2].order = order1;
        
        // Persist changes
        drop(profiles);
        self.save_profiles().await?;
        
        info!("Profiles {} and {} swapped", profile_id_1, profile_id_2);
        Ok(())
    }

    /// Get profiles sorted by their order
    pub async fn get_ordered_profiles(&self) -> Vec<ProfileConfig> {
        let profiles = self.profiles.read().await;
        let mut sorted: Vec<ProfileConfig> = profiles.values().cloned().collect();
        sorted.sort_by_key(|p| p.order);
        sorted
    }

    /// Reorder all profiles to ensure consecutive order values
    pub async fn normalize_order(&self) -> Result<()> {
        info!("Normalizing profile order");
        
        let mut profiles = self.profiles.write().await;
        let mut sorted: Vec<&mut ProfileConfig> = profiles.values_mut().collect();
        sorted.sort_by_key(|p| p.order);
        
        // Assign consecutive order values
        for (index, profile) in sorted.iter_mut().enumerate() {
            profile.order = index as i32;
        }
        
        // Persist changes
        drop(profiles);
        self.save_profiles().await?;
        
        info!("Profile order normalized");
        Ok(())
    }

    /// Clone a profile with optional data selection
    pub async fn clone_profile(&self, profile_id: &str, new_name: Option<String>, include_bookmarks: bool, include_history: bool) -> Result<String> {
        info!("Cloning profile {} with options: bookmarks={}, history={}", 
              profile_id, include_bookmarks, include_history);
        
        let profiles = self.profiles.read().await;
        let original = profiles.get(profile_id)
            .ok_or_else(|| anyhow::anyhow!("Profile not found: {}", profile_id))?
            .clone();
        drop(profiles);
        
        // Create cloned profile with new UUID
        let now = chrono::Utc::now().timestamp_millis();
        let cloned_id = uuid::Uuid::new_v4().to_string();
        let cloned_profile = ProfileConfig {
            id: cloned_id.clone(),
            name: new_name.unwrap_or_else(|| format!("{} (Copy)", original.name)),
            profile_type: original.profile_type.clone(),
            icon: original.icon.clone(),
            color: original.color.clone(),
            active: false, // Cloned profile is never active
            order: original.order, // Same order initially, can be reordered
            settings: original.settings.clone(),
            bookmarks: if include_bookmarks { original.bookmarks.clone() } else { Vec::new() },
            history: if include_history { original.history.clone() } else { Vec::new() },
            extensions: original.extensions.clone(),
            theme: original.theme.clone(),
            created_at: now,
            last_used_at: original.created_at, // Keep original last_used_at
        };
        
        // Add cloned profile to manager
        let mut profiles = self.profiles.write().await;
        profiles.insert(cloned_profile.id.clone(), cloned_profile);
        drop(profiles);
        
        // Persist changes
        self.save_profiles().await?;
        
        info!("Profile cloned successfully as {}", cloned_id);
        Ok(cloned_id)
    }

    /// Clone options structure
    #[derive(Debug, Clone)]
    pub struct CloneOptions {
        pub new_name: Option<String>,
        pub include_bookmarks: bool,
        pub include_history: bool,
        pub clone_settings: bool,
        pub clone_theme: bool,
    }

    impl Default for CloneOptions {
        fn default() -> Self {
            Self {
                new_name: None,
                include_bookmarks: true,
                include_history: false,
                clone_settings: true,
                clone_theme: true,
            }
        }
    }

    /// Clone a profile with detailed options
    pub async fn clone_profile_with_options(&self, profile_id: &str, options: &CloneOptions) -> Result<String> {
        info!("Cloning profile {} with detailed options: {:?}", profile_id, options);
        
        let profiles = self.profiles.read().await;
        let original = profiles.get(profile_id)
            .ok_or_else(|| anyhow::anyhow!("Profile not found: {}", profile_id))?
            .clone();
        drop(profiles);
        
        // Create cloned profile with new UUID
        let now = chrono::Utc::now().timestamp_millis();
        let cloned_profile = ProfileConfig {
            id: uuid::Uuid::new_v4().to_string(),
            name: options.new_name.clone()
                .unwrap_or_else(|| format!("{} (Copy)", original.name)),
            profile_type: original.profile_type.clone(),
            icon: original.icon.clone(),
            color: original.color.clone(),
            active: false, // Cloned profile is never active
            order: original.order, // Same order initially, can be reordered
            settings: if options.clone_settings { original.settings.clone() } else { HashMap::new() },
            bookmarks: if options.include_bookmarks { original.bookmarks.clone() } else { Vec::new() },
            history: if options.include_history { original.history.clone() } else { Vec::new() },
            extensions: original.extensions.clone(),
            theme: if options.clone_theme { original.theme.clone() } else { None },
            created_at: now,
            last_used_at: original.created_at, // Keep original last_used_at
        };
        
        let cloned_id = cloned_profile.id.clone();
        
        // Add cloned profile to manager
        let mut profiles = self.profiles.write().await;
        profiles.insert(cloned_id.clone(), cloned_profile);
        drop(profiles);
        
        // Persist changes
        self.save_profiles().await?;
        
        info!("Profile cloned successfully as {}", cloned_id);
        Ok(cloned_id)
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