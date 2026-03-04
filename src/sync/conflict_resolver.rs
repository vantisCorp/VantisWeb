// VantisWeb Browser - Sync Conflict Resolution
// Copyright (c) 2024 VantisCorp
// Conflict detection and resolution for cloud sync

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Types of sync conflicts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncConflict {
    /// Both local and remote have been modified
    BothModified {
        profile_id: String,
        local_modified: DateTime<Utc>,
        remote_modified: DateTime<Utc>,
        local_size: u64,
        remote_size: u64,
    },
    
    /// File deleted locally but modified remotely
    DeletedLocallyModifiedRemotely {
        profile_id: String,
        remote_modified: DateTime<Utc>,
    },
    
    /// File modified locally but deleted remotely
    ModifiedLocallyDeletedRemotely {
        profile_id: String,
        local_modified: DateTime<Utc>,
    },
    
    /// Name collision
    NameCollision {
        local_id: String,
        remote_id: String,
        name: String,
    },
    
    /// Circular modification (complex sync scenarios)
    CircularModification {
        profile_id: String,
        history: Vec<ModificationRecord>,
    },
}

impl SyncConflict {
    /// Get the profile ID involved in the conflict
    pub fn profile_id(&self) -> &str {
        match self {
            Self::BothModified { profile_id, .. } => profile_id,
            Self::DeletedLocallyModifiedRemotely { profile_id, .. } => profile_id,
            Self::ModifiedLocallyDeletedRemotely { profile_id, .. } => profile_id,
            Self::NameCollision { local_id, .. } => local_id,
            Self::CircularModification { profile_id, .. } => profile_id,
        }
    }
    
    /// Get a human-readable description of the conflict
    pub fn description(&self) -> String {
        match self {
            Self::BothModified { profile_id, local_modified, remote_modified, .. } => {
                format!(
                    "Profile '{}' was modified both locally ({}) and remotely ({})",
                    profile_id,
                    local_modified.format("%Y-%m-%d %H:%M:%S"),
                    remote_modified.format("%Y-%m-%d %H:%M:%S")
                )
            }
            Self::DeletedLocallyModifiedRemotely { profile_id, remote_modified } => {
                format!(
                    "Profile '{}' was deleted locally but modified remotely on {}",
                    profile_id,
                    remote_modified.format("%Y-%m-%d %H:%M:%S")
                )
            }
            Self::ModifiedLocallyDeletedRemotely { profile_id, local_modified } => {
                format!(
                    "Profile '{}' was modified locally on {} but deleted remotely",
                    profile_id,
                    local_modified.format("%Y-%m-%d %H:%M:%S")
                )
            }
            Self::NameCollision { name, .. } => {
                format!("Name collision detected for '{}'", name)
            }
            Self::CircularModification { profile_id, history } => {
                format!(
                    "Circular modification detected for '{}' with {} changes",
                    profile_id,
                    history.len()
                )
            }
        }
    }
}

/// Modification record for tracking changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModificationRecord {
    pub timestamp: DateTime<Utc>,
    pub source: ModificationSource,
    pub size: u64,
    pub hash: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModificationSource {
    Local,
    Remote,
}

/// Conflict resolution strategies
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// Keep the local version
    KeepLocal {
        profile_id: String,
        backup_remote: bool,
    },
    
    /// Keep the remote version
    KeepRemote {
        profile_id: String,
        backup_local: bool,
    },
    
    /// Keep the newer version based on timestamp
    KeepNewer {
        profile_id: String,
    },
    
    /// Merge both versions
    Merge {
        profile_id: String,
        merge_strategy: MergeStrategy,
    },
    
    /// Skip this item
    Skip {
        profile_id: String,
    },
    
    /// Rename the local version
    RenameLocal {
        profile_id: String,
        new_name: String,
    },
    
    /// Create a conflict copy
    CreateConflictCopy {
        profile_id: String,
        suffix: String,
    },
}

impl ConflictResolution {
    /// Get the profile ID for this resolution
    pub fn profile_id(&self) -> &str {
        match self {
            Self::KeepLocal { profile_id, .. } => profile_id,
            Self::KeepRemote { profile_id, .. } => profile_id,
            Self::KeepNewer { profile_id, .. } => profile_id,
            Self::Merge { profile_id, .. } => profile_id,
            Self::Skip { profile_id, .. } => profile_id,
            Self::RenameLocal { profile_id, .. } => profile_id,
            Self::CreateConflictCopy { profile_id, .. } => profile_id,
        }
    }
    
    /// Get a human-readable description
    pub fn description(&self) -> String {
        match self {
            Self::KeepLocal { backup_remote, .. } => {
                if *backup_remote {
                    "Keep local version, backup remote".to_string()
                } else {
                    "Keep local version, discard remote".to_string()
                }
            }
            Self::KeepRemote { backup_local, .. } => {
                if *backup_local {
                    "Keep remote version, backup local".to_string()
                } else {
                    "Keep remote version, discard local".to_string()
                }
            }
            Self::KeepNewer { .. } => "Keep the newer version".to_string(),
            Self::Merge { merge_strategy, .. } => {
                format!("Merge using {:?}", merge_strategy)
            }
            Self::Skip { .. } => "Skip this item".to_string(),
            Self::RenameLocal { new_name, .. } => {
                format!("Rename local to '{}'", new_name)
            }
            Self::CreateConflictCopy { suffix, .. } => {
                format!("Create conflict copy with suffix '{}'", suffix)
            }
        }
    }
}

/// Merge strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeStrategy {
    /// Prefer local values for conflicts
    PreferLocal,
    
    /// Prefer remote values for conflicts
    PreferRemote,
    
    /// Merge arrays by combining unique values
    Combine,
    
    /// Use smart merging based on field types
    Smart,
}

/// Conflict resolver configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolverConfig {
    /// Default resolution strategy
    pub default_strategy: DefaultResolution,
    
    /// Automatically resolve conflicts using default strategy
    pub auto_resolve: bool,
    
    /// Create backups before applying resolutions
    pub create_backups: bool,
    
    /// Maximum number of conflict copies to keep
    pub max_conflict_copies: usize,
}

impl Default for ConflictResolverConfig {
    fn default() -> Self {
        Self {
            default_strategy: DefaultResolution::KeepNewer,
            auto_resolve: false,
            create_backups: true,
            max_conflict_copies: 10,
        }
    }
}

/// Default resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DefaultResolution {
    KeepLocal,
    KeepRemote,
    KeepNewer,
    AskUser,
}

/// Conflict resolver for managing sync conflicts
pub struct ConflictResolver {
    config: ConflictResolverConfig,
    conflicts: Vec<SyncConflict>,
    resolutions: HashMap<String, ConflictResolution>,
    conflict_history: Vec<ConflictHistoryEntry>,
}

impl ConflictResolver {
    /// Create a new conflict resolver
    pub fn new(config: ConflictResolverConfig) -> Self {
        Self {
            config,
            conflicts: Vec::new(),
            resolutions: HashMap::new(),
            conflict_history: Vec::new(),
        }
    }
    
    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(ConflictResolverConfig::default())
    }
    
    /// Add a detected conflict
    pub fn add_conflict(&mut self, conflict: SyncConflict) {
        self.conflicts.push(conflict);
    }
    
    /// Get all pending conflicts
    pub fn get_pending_conflicts(&self) -> &[SyncConflict] {
        &self.conflicts
    }
    
    /// Set resolution for a conflict
    pub fn set_resolution(&mut self, resolution: ConflictResolution) {
        let profile_id = resolution.profile_id().to_string();
        self.resolutions.insert(profile_id, resolution);
    }
    
    /// Get resolution for a profile
    pub fn get_resolution(&self, profile_id: &str) -> Option<&ConflictResolution> {
        self.resolutions.get(profile_id)
    }
    
    /// Apply default resolution to all conflicts
    pub fn apply_default_resolutions(&mut self) -> Vec<ConflictResolution> {
        let mut resolutions = Vec::new();
        
        for conflict in self.conflicts.clone() {
            let resolution = self.resolve_with_default(conflict.clone());
            resolutions.push(resolution.clone());
            self.set_resolution(resolution);
        }
        
        resolutions
    }
    
    /// Resolve a conflict using the default strategy
    fn resolve_with_default(&self, conflict: SyncConflict) -> ConflictResolution {
        let profile_id = conflict.profile_id().to_string();
        
        match self.config.default_strategy {
            DefaultResolution::KeepLocal => ConflictResolution::KeepLocal {
                profile_id,
                backup_remote: self.config.create_backups,
            },
            DefaultResolution::KeepRemote => ConflictResolution::KeepRemote {
                profile_id,
                backup_local: self.config.create_backups,
            },
            DefaultResolution::KeepNewer => ConflictResolution::KeepNewer { profile_id },
            DefaultResolution::AskUser => ConflictResolution::CreateConflictCopy {
                profile_id,
                suffix: format!("_conflict_{}", Utc::now().format("%Y%m%d_%H%M%S")),
            },
        }
    }
    
    /// Apply resolution to a conflict
    pub fn apply_resolution(&mut self, resolution: &ConflictResolution) -> Result<ResolutionResult, String> {
        let profile_id = resolution.profile_id().to_string();
        
        // Find the conflict
        let conflict_idx = self.conflicts.iter().position(|c| c.profile_id() == profile_id);
        
        if let Some(idx) = conflict_idx {
            let conflict = self.conflicts.remove(idx);
            
            // Record in history
            self.conflict_history.push(ConflictHistoryEntry {
                conflict: conflict.clone(),
                resolution: resolution.clone(),
                resolved_at: Utc::now(),
            });
            
            // Return the result
            Ok(ResolutionResult {
                profile_id,
                action: resolution_to_action(resolution),
            })
        } else {
            Err(format!("No conflict found for profile {}", profile_id))
        }
    }
    
    /// Get conflict history
    pub fn get_history(&self) -> &[ConflictHistoryEntry] {
        &self.conflict_history
    }
    
    /// Clear all pending conflicts
    pub fn clear_pending(&mut self) {
        self.conflicts.clear();
        self.resolutions.clear();
    }
    
    /// Check if there are any pending conflicts
    pub fn has_pending_conflicts(&self) -> bool {
        !self.conflicts.is_empty()
    }
    
    /// Get the number of pending conflicts
    pub fn pending_count(&self) -> usize {
        self.conflicts.len()
    }
}

/// Result of applying a resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionResult {
    pub profile_id: String,
    pub action: ResolutionAction,
}

/// Actions to take for resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionAction {
    UploadLocal,
    DownloadRemote,
    Skip,
    CreateBackup,
    DeleteLocal,
    DeleteRemote,
    RenameAndUpload,
}

/// Conflict history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictHistoryEntry {
    pub conflict: SyncConflict,
    pub resolution: ConflictResolution,
    pub resolved_at: DateTime<Utc>,
}

/// Convert resolution to action
fn resolution_to_action(resolution: &ConflictResolution) -> ResolutionAction {
    match resolution {
        ConflictResolution::KeepLocal { .. } => ResolutionAction::UploadLocal,
        ConflictResolution::KeepRemote { .. } => ResolutionAction::DownloadRemote,
        ConflictResolution::KeepNewer { .. } => ResolutionAction::DownloadRemote, // Will be determined by timestamp
        ConflictResolution::Skip { .. } => ResolutionAction::Skip,
        ConflictResolution::Merge { .. } => ResolutionAction::DownloadRemote, // Merge happens after download
        ConflictResolution::RenameLocal { .. } => ResolutionAction::RenameAndUpload,
        ConflictResolution::CreateConflictCopy { .. } => ResolutionAction::CreateBackup,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_conflict_creation() {
        let conflict = SyncConflict::BothModified {
            profile_id: "profile1".to_string(),
            local_modified: Utc::now(),
            remote_modified: Utc::now(),
            local_size: 1024,
            remote_size: 2048,
        };
        
        assert_eq!(conflict.profile_id(), "profile1");
        assert!(conflict.description().contains("profile1"));
    }
    
    #[test]
    fn test_conflict_resolver() {
        let mut resolver = ConflictResolver::with_defaults();
        
        let conflict = SyncConflict::BothModified {
            profile_id: "profile1".to_string(),
            local_modified: Utc::now(),
            remote_modified: Utc::now(),
            local_size: 1024,
            remote_size: 2048,
        };
        
        resolver.add_conflict(conflict);
        
        assert!(resolver.has_pending_conflicts());
        assert_eq!(resolver.pending_count(), 1);
        
        let resolutions = resolver.apply_default_resolutions();
        assert_eq!(resolutions.len(), 1);
    }
    
    #[test]
    fn test_resolution_description() {
        let resolution = ConflictResolution::KeepLocal {
            profile_id: "profile1".to_string(),
            backup_remote: true,
        };
        
        assert!(resolution.description().contains("backup"));
    }
}