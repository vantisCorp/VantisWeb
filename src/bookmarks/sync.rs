use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use super::Bookmark;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncedBookmark {
    pub bookmark: Bookmark,
    pub sync_id: String,
    pub device_id: String,
    pub last_modified: DateTime<Utc>,
    pub version: u32,
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    pub device_id: String,
    pub last_sync: Option<DateTime<Utc>>,
    pub bookmarks: HashMap<Uuid, SyncedBookmark>,
    pub sync_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConflict {
    pub bookmark_id: Uuid,
    pub local: SyncedBookmark,
    pub remote: SyncedBookmark,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ConflictResolution {
    KeepLocal,
    KeepRemote,
    KeepNewest,
    Merge,
    Manual,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SyncStatus {
    NotSynced,
    Syncing,
    Synced,
    Error,
    Conflict,
}

#[derive(Debug, Clone)]
pub struct SyncSettings {
    pub auto_sync_enabled: bool,
    pub sync_interval_seconds: u64,
    pub conflict_resolution: ConflictResolution,
    pub sync_on_startup: bool,
    pub sync_on_change: bool,
    pub compress_data: bool,
    pub encrypt_data: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncProvider {
    pub name: String,
    pub endpoint: String,
    pub is_authenticated: bool,
    pub last_sync: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatistics {
    pub total_bookmarks: usize,
    pub synced_bookmarks: usize,
    pub pending_bookmarks: usize,
    pub conflicts_count: usize,
    pub last_sync: Option<DateTime<Utc>>,
    pub sync_errors: usize,
}

pub struct BookmarkSyncManager {
    device_id: String,
    state: Arc<RwLock<SyncState>>,
    settings: Arc<RwLock<SyncSettings>>,
    conflicts: Arc<RwLock<Vec<SyncConflict>>>,
    status: Arc<RwLock<SyncStatus>>,
    providers: Arc<RwLock<Vec<SyncProvider>>>,
}

impl BookmarkSyncManager {
    pub fn new(device_id: String) -> Self {
        Self {
            device_id: device_id.clone(),
            state: Arc::new(RwLock::new(SyncState {
                device_id,
                last_sync: None,
                bookmarks: HashMap::new(),
                sync_version: 1,
            })),
            settings: Arc::new(RwLock::new(SyncSettings::default())),
            conflicts: Arc::new(RwLock::new(Vec::new())),
            status: Arc::new(RwLock::new(SyncStatus::NotSynced)),
            providers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn get_device_id(&self) -> String {
        self.device_id.clone()
    }

    pub async fn add_bookmark_to_sync(&self, bookmark: Bookmark) {
        let sync_id = format!("{}-{}", self.device_id, bookmark.id);
        let synced = SyncedBookmark {
            bookmark: bookmark.clone(),
            sync_id,
            device_id: self.device_id.clone(),
            last_modified: Utc::now(),
            version: 1,
            is_deleted: false,
        };
        
        let mut state = self.state.write().await;
        state.bookmarks.insert(bookmark.id, synced);
    }

    pub async fn update_bookmark_sync(&self, bookmark: Bookmark) {
        let mut state = self.state.write().await;
        if let Some(synced) = state.bookmarks.get_mut(&bookmark.id) {
            synced.bookmark = bookmark;
            synced.last_modified = Utc::now();
            synced.version += 1;
        } else {
            // Add if not exists
            drop(state);
            self.add_bookmark_to_sync(bookmark).await;
        }
    }

    pub async fn mark_bookmark_deleted(&self, id: Uuid) {
        let mut state = self.state.write().await;
        if let Some(synced) = state.bookmarks.get_mut(&id) {
            synced.is_deleted = true;
            synced.last_modified = Utc::now();
            synced.version += 1;
        }
    }

    pub async fn remove_bookmark_from_sync(&self, id: Uuid) {
        let mut state = self.state.write().await;
        state.bookmarks.remove(&id);
    }

    pub async fn get_sync_status(&self) -> SyncStatus {
        *self.status.read().await
    }

    pub async fn get_synced_bookmarks(&self) -> Vec<Bookmark> {
        let state = self.state.read().await;
        state.bookmarks.values()
            .filter(|s| !s.is_deleted)
            .map(|s| s.bookmark.clone())
            .collect()
    }

    pub async fn get_pending_bookmarks(&self) -> Vec<SyncedBookmark> {
        let state = self.state.read().await;
        let last_sync = state.last_sync;
        
        state.bookmarks.values()
            .filter(|s| {
                last_sync.map_or(true, |ls| s.last_modified > ls)
            })
            .cloned()
            .collect()
    }

    pub async fn get_conflicts(&self) -> Vec<SyncConflict> {
        self.conflicts.read().await.clone()
    }

    pub async fn resolve_conflict(&self, bookmark_id: Uuid, resolution: ConflictResolution) -> Result<Option<Bookmark>, String> {
        let mut conflicts = self.conflicts.write().await;
        
        if let Some(pos) = conflicts.iter().position(|c| c.bookmark_id == bookmark_id) {
            let conflict = conflicts.remove(pos);
            
            let resolved_bookmark = match resolution {
                ConflictResolution::KeepLocal => conflict.local.bookmark,
                ConflictResolution::KeepRemote => conflict.remote.bookmark,
                ConflictResolution::KeepNewest => {
                    if conflict.local.last_modified > conflict.remote.last_modified {
                        conflict.local.bookmark
                    } else {
                        conflict.remote.bookmark
                    }
                }
                ConflictResolution::Merge => {
                    // Simple merge: prefer remote values for non-empty fields
                    let mut merged = conflict.local.bookmark.clone();
                    if !conflict.remote.bookmark.title.is_empty() {
                        merged.title = conflict.remote.bookmark.title;
                    }
                    if conflict.remote.bookmark.description.is_some() {
                        merged.description = conflict.remote.bookmark.description;
                    }
                    merged.tags.extend(conflict.remote.bookmark.tags);
                    merged.tags.sort();
                    merged.tags.dedup();
                    merged
                }
                ConflictResolution::Manual => {
                    // Put back the conflict for manual resolution
                    conflicts.push(conflict);
                    return Ok(None);
                }
            };
            
            // Update state with resolved bookmark
            let mut state = self.state.write().await;
            if let Some(synced) = state.bookmarks.get_mut(&bookmark_id) {
                synced.bookmark = resolved_bookmark.clone();
                synced.last_modified = Utc::now();
                synced.version += 1;
            }
            
            return Ok(Some(resolved_bookmark));
        }
        
        Ok(None)
    }

    pub async fn sync_with_remote(&self, remote_state: SyncState) -> Result<Vec<SyncConflict>, String> {
        *self.status.write().await = SyncStatus::Syncing;
        
        let mut new_conflicts = Vec::new();
        let mut state = self.state.write().await;
        let settings = self.settings.read().await;
        
        for (id, remote_synced) in &remote_state.bookmarks {
            if let Some(local_synced) = state.bookmarks.get(id) {
                // Both exist - check for conflict
                if local_synced.version != remote_synced.version
                    && local_synced.last_modified != remote_synced.last_modified
                {
                    new_conflicts.push(SyncConflict {
                        bookmark_id: *id,
                        local: local_synced.clone(),
                        remote: remote_synced.clone(),
                        detected_at: Utc::now(),
                    });
                    
                    // Auto-resolve based on settings
                    match settings.conflict_resolution {
                        ConflictResolution::KeepLocal => {}
                        ConflictResolution::KeepRemote => {
                            state.bookmarks.insert(*id, remote_synced.clone());
                        }
                        ConflictResolution::KeepNewest => {
                            if remote_synced.last_modified > local_synced.last_modified {
                                state.bookmarks.insert(*id, remote_synced.clone());
                            }
                        }
                        ConflictResolution::Merge => {
                            let mut merged = local_synced.clone();
                            if !remote_synced.bookmark.title.is_empty() {
                                merged.bookmark.title = remote_synced.bookmark.title.clone();
                            }
                            merged.last_modified = Utc::now();
                            merged.version += 1;
                            state.bookmarks.insert(*id, merged);
                        }
                        ConflictResolution::Manual => {}
                    }
                }
            } else {
                // Only remote - add to local
                state.bookmarks.insert(*id, remote_synced.clone());
            }
        }
        
        state.last_sync = Some(Utc::now());
        state.sync_version = remote_state.sync_version.max(state.sync_version);
        
        drop(state);
        
        // Add new conflicts
        self.conflicts.write().await.extend(new_conflicts.clone());
        
        *self.status.write().await = if new_conflicts.is_empty() {
            SyncStatus::Synced
        } else {
            SyncStatus::Conflict
        };
        
        Ok(new_conflicts)
    }

    pub async fn update_settings(&self, settings: SyncSettings) {
        *self.settings.write().await = settings;
    }

    pub async fn get_settings(&self) -> SyncSettings {
        self.settings.read().await.clone()
    }

    pub async fn add_provider(&self, provider: SyncProvider) {
        self.providers.write().await.push(provider);
    }

    pub async fn remove_provider(&self, name: &str) {
        let mut providers = self.providers.write().await;
        providers.retain(|p| p.name != name);
    }

    pub async fn get_providers(&self) -> Vec<SyncProvider> {
        self.providers.read().await.clone()
    }

    pub async fn get_statistics(&self) -> SyncStatistics {
        let state = self.state.read().await;
        let conflicts = self.conflicts.read().await;
        
        let synced_count = state.bookmarks.values()
            .filter(|s| !s.is_deleted && s.version > 0)
            .count();
        
        let pending_count = state.bookmarks.values()
            .filter(|s| {
                state.last_sync.map_or(true, |ls| s.last_modified > ls)
            })
            .count();
        
        SyncStatistics {
            total_bookmarks: state.bookmarks.len(),
            synced_bookmarks: synced_count,
            pending_bookmarks: pending_count,
            conflicts_count: conflicts.len(),
            last_sync: state.last_sync,
            sync_errors: 0,
        }
    }

    pub async fn export_sync_state(&self) -> Result<String, String> {
        let state = self.state.read().await;
        serde_json::to_string_pretty(&*state)
            .map_err(|e| format!("Failed to serialize sync state: {}", e))
    }

    pub async fn import_sync_state(&self, json: &str) -> Result<(), String> {
        let imported: SyncState = serde_json::from_str(json)
            .map_err(|e| format!("Failed to deserialize sync state: {}", e))?;
        
        let mut state = self.state.write().await;
        *state = imported;
        
        *self.status.write().await = SyncStatus::Synced;
        
        Ok(())
    }

    pub async fn clear_sync_data(&self) {
        let mut state = self.state.write().await;
        state.bookmarks.clear();
        state.last_sync = None;
        
        self.conflicts.write().await.clear();
        
        *self.status.write().await = SyncStatus::NotSynced;
    }

    pub async fn force_full_sync(&self) -> Result<(), String> {
        *self.status.write().await = SyncStatus::Syncing;
        
        // In production, this would trigger a full sync with the remote server
        // For now, we just update the timestamp
        
        let mut state = self.state.write().await;
        state.last_sync = Some(Utc::now());
        
        *self.status.write().await = SyncStatus::Synced;
        
        Ok(())
    }
}

impl Default for SyncSettings {
    fn default() -> Self {
        Self {
            auto_sync_enabled: true,
            sync_interval_seconds: 300,
            conflict_resolution: ConflictResolution::KeepNewest,
            sync_on_startup: true,
            sync_on_change: true,
            compress_data: true,
            encrypt_data: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sync_manager_creation() {
        let manager = BookmarkSyncManager::new("device-123".to_string());
        assert_eq!(manager.get_device_id().await, "device-123");
    }

    #[tokio::test]
    async fn test_add_bookmark_to_sync() {
        let manager = BookmarkSyncManager::new("device-123".to_string());
        
        let bookmark = Bookmark {
            id: Uuid::new_v4(),
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
            description: None,
            favicon: None,
            folder_id: None,
            tags: Vec::new(),
            is_favorite: false,
            is_read_later: false,
            visit_count: 0,
            last_accessed: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        manager.add_bookmark_to_sync(bookmark).await;
        
        let synced = manager.get_synced_bookmarks().await;
        assert_eq!(synced.len(), 1);
    }

    #[tokio::test]
    async fn test_sync_conflict_resolution() {
        let manager = BookmarkSyncManager::new("device-123".to_string());
        
        let bookmark_id = Uuid::new_v4();
        let local_bookmark = Bookmark {
            id: bookmark_id,
            url: "https://example.com".to_string(),
            title: "Local Title".to_string(),
            description: None,
            favicon: None,
            folder_id: None,
            tags: Vec::new(),
            is_favorite: false,
            is_read_later: false,
            visit_count: 0,
            last_accessed: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let mut remote_bookmark = local_bookmark.clone();
        remote_bookmark.title = "Remote Title".to_string();
        
        manager.add_bookmark_to_sync(local_bookmark.clone()).await;
        
        // Simulate conflict
        let conflict = SyncConflict {
            bookmark_id,
            local: SyncedBookmark {
                bookmark: local_bookmark,
                sync_id: "local".to_string(),
                device_id: "device-123".to_string(),
                last_modified: Utc::now() - chrono::Duration::hours(1),
                version: 1,
                is_deleted: false,
            },
            remote: SyncedBookmark {
                bookmark: remote_bookmark,
                sync_id: "remote".to_string(),
                device_id: "device-456".to_string(),
                last_modified: Utc::now(),
                version: 2,
                is_deleted: false,
            },
            detected_at: Utc::now(),
        };
        
        manager.conflicts.write().await.push(conflict);
        
        let resolved = manager.resolve_conflict(bookmark_id, ConflictResolution::KeepRemote).await.unwrap();
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap().title, "Remote Title");
    }

    #[tokio::test]
    async fn test_sync_statistics() {
        let manager = BookmarkSyncManager::new("device-123".to_string());
        
        let bookmark = Bookmark {
            id: Uuid::new_v4(),
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
            description: None,
            favicon: None,
            folder_id: None,
            tags: Vec::new(),
            is_favorite: false,
            is_read_later: false,
            visit_count: 0,
            last_accessed: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        manager.add_bookmark_to_sync(bookmark).await;
        
        let stats = manager.get_statistics().await;
        assert_eq!(stats.total_bookmarks, 1);
    }

    #[tokio::test]
    async fn test_export_import_sync_state() {
        let manager = BookmarkSyncManager::new("device-123".to_string());
        
        let bookmark = Bookmark {
            id: Uuid::new_v4(),
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
            description: None,
            favicon: None,
            folder_id: None,
            tags: Vec::new(),
            is_favorite: false,
            is_read_later: false,
            visit_count: 0,
            last_accessed: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        manager.add_bookmark_to_sync(bookmark).await;
        
        let exported = manager.export_sync_state().await.unwrap();
        assert!(exported.contains("example.com"));
        
        let new_manager = BookmarkSyncManager::new("device-456".to_string());
        new_manager.import_sync_state(&exported).await.unwrap();
        
        let synced = new_manager.get_synced_bookmarks().await;
        assert_eq!(synced.len(), 1);
    }
}