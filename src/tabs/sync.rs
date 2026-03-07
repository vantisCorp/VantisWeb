use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncedTab {
    pub id: Uuid,
    pub url: String,
    pub title: String,
    pub favicon: Option<String>,
    pub group_id: Option<Uuid>,
    pub workspace_id: Option<Uuid>,
    pub is_pinned: bool,
    pub is_hibernated: bool,
    pub position: usize,
    pub last_modified: DateTime<Utc>,
    pub device_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncedGroup {
    pub id: Uuid,
    pub name: String,
    pub color: String,
    pub is_collapsed: bool,
    pub last_modified: DateTime<Utc>,
    pub device_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncedWorkspace {
    pub id: Uuid,
    pub name: String,
    pub tab_ids: Vec<Uuid>,
    pub settings: WorkspaceSettings,
    pub last_modified: DateTime<Utc>,
    pub device_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSettings {
    pub extensions_enabled: bool,
    pub ad_blocking: bool,
    pub tracking_protection: bool,
    pub cookie_settings: CookieSettings,
    pub theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieSettings {
    pub allow_third_party: bool,
    pub block_trackers: bool,
    pub clear_on_exit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    pub device_id: String,
    pub last_sync: Option<DateTime<Utc>>,
    pub tabs: HashMap<Uuid, SyncedTab>,
    pub groups: HashMap<Uuid, SyncedGroup>,
    pub workspaces: HashMap<Uuid, SyncedWorkspace>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConflict {
    pub item_type: String,
    pub item_id: Uuid,
    pub local_version: DateTime<Utc>,
    pub remote_version: DateTime<Utc>,
    pub device_id: String,
}

#[derive(Debug, Clone)]
pub struct SyncSettings {
    pub auto_sync_enabled: bool,
    pub sync_interval_seconds: u64,
    pub conflict_resolution: ConflictResolution,
    pub sync_enabled_types: SyncEnabledTypes,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ConflictResolution {
    KeepLocal,
    KeepRemote,
    KeepNewest,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEnabledTypes {
    pub sync_tabs: bool,
    pub sync_groups: bool,
    pub sync_workspaces: bool,
}

pub struct TabSyncManager {
    device_id: String,
    state: Arc<RwLock<SyncState>>,
    settings: Arc<RwLock<SyncSettings>>,
    conflicts: Arc<RwLock<Vec<SyncConflict>>>,
}

impl TabSyncManager {
    pub fn new(device_id: String) -> Self {
        let settings = SyncSettings {
            auto_sync_enabled: true,
            sync_interval_seconds: 300,
            conflict_resolution: ConflictResolution::KeepNewest,
            sync_enabled_types: SyncEnabledTypes {
                sync_tabs: true,
                sync_groups: true,
                sync_workspaces: true,
            },
        };

        Self {
            device_id,
            state: Arc::new(RwLock::new(SyncState {
                device_id: device_id.clone(),
                last_sync: None,
                tabs: HashMap::new(),
                groups: HashMap::new(),
                workspaces: HashMap::new(),
            })),
            settings: Arc::new(RwLock::new(settings)),
            conflicts: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn get_device_id(&self) -> String {
        self.device_id.clone()
    }

    pub async fn add_tab_to_sync(&self, tab: SyncedTab) {
        let mut state = self.state.write().await;
        state.tabs.insert(tab.id, tab);
    }

    pub async fn add_group_to_sync(&self, group: SyncedGroup) {
        let mut state = self.state.write().await;
        state.groups.insert(group.id, group);
    }

    pub async fn add_workspace_to_sync(&self, workspace: SyncedWorkspace) {
        let mut state = self.state.write().await;
        state.workspaces.insert(workspace.id, workspace);
    }

    pub async fn remove_tab_from_sync(&self, tab_id: Uuid) {
        let mut state = self.state.write().await;
        state.tabs.remove(&tab_id);
    }

    pub async fn remove_group_from_sync(&self, group_id: Uuid) {
        let mut state = self.state.write().await;
        state.groups.remove(&group_id);
    }

    pub async fn remove_workspace_from_sync(&self, workspace_id: Uuid) {
        let mut state = self.state.write().await;
        state.workspaces.remove(&workspace_id);
    }

    pub async fn get_synced_tabs(&self) -> Vec<SyncedTab> {
        let state = self.state.read().await;
        state.tabs.values().cloned().collect()
    }

    pub async fn get_synced_groups(&self) -> Vec<SyncedGroup> {
        let state = self.state.read().await;
        state.groups.values().cloned().collect()
    }

    pub async fn get_synced_workspaces(&self) -> Vec<SyncedWorkspace> {
        let state = self.state.read().await;
        state.workspaces.values().cloned().collect()
    }

    pub async fn get_last_sync(&self) -> Option<DateTime<Utc>> {
        let state = self.state.read().await;
        state.last_sync
    }

    pub async fn update_sync_settings(&self, settings: SyncSettings) {
        let mut sync_settings = self.settings.write().await;
        *sync_settings = settings;
    }

    pub async fn get_sync_settings(&self) -> SyncSettings {
        let settings = self.settings.read().await;
        settings.clone()
    }

    pub async fn get_conflicts(&self) -> Vec<SyncConflict> {
        let conflicts = self.conflicts.read().await;
        conflicts.clone()
    }

    pub async fn resolve_conflict(&self, conflict_id: Uuid, resolution: ConflictResolution) -> Result<(), String> {
        let mut conflicts = self.conflicts.write().await;
        conflicts.retain(|c| c.item_id != conflict_id);
        Ok(())
    }

    pub async fn sync_with_remote(&self, remote_state: SyncState) -> Result<Vec<SyncConflict>, String> {
        let mut conflicts = Vec::new();
        let mut state = self.state.write().await;
        let settings = self.settings.read().await;

        // Sync tabs
        if settings.sync_enabled_types.sync_tabs {
            for (tab_id, remote_tab) in remote_state.tabs {
                if let Some(local_tab) = state.tabs.get(&tab_id) {
                    if local_tab.last_modified != remote_tab.last_modified {
                        conflicts.push(SyncConflict {
                            item_type: "tab".to_string(),
                            item_id: tab_id,
                            local_version: local_tab.last_modified,
                            remote_version: remote_tab.last_modified,
                            device_id: remote_tab.device_id.clone(),
                        });

                        match settings.conflict_resolution {
                            ConflictResolution::KeepLocal => {}
                            ConflictResolution::KeepRemote => {
                                state.tabs.insert(tab_id, remote_tab);
                            }
                            ConflictResolution::KeepNewest => {
                                if remote_tab.last_modified > local_tab.last_modified {
                                    state.tabs.insert(tab_id, remote_tab);
                                }
                            }
                            ConflictResolution::Manual => {}
                        }
                    }
                } else {
                    state.tabs.insert(tab_id, remote_tab);
                }
            }
        }

        // Sync groups
        if settings.sync_enabled_types.sync_groups {
            for (group_id, remote_group) in remote_state.groups {
                if let Some(local_group) = state.groups.get(&group_id) {
                    if local_group.last_modified != remote_group.last_modified {
                        conflicts.push(SyncConflict {
                            item_type: "group".to_string(),
                            item_id: group_id,
                            local_version: local_group.last_modified,
                            remote_version: remote_group.last_modified,
                            device_id: remote_group.device_id.clone(),
                        });

                        match settings.conflict_resolution {
                            ConflictResolution::KeepLocal => {}
                            ConflictResolution::KeepRemote => {
                                state.groups.insert(group_id, remote_group);
                            }
                            ConflictResolution::KeepNewest => {
                                if remote_group.last_modified > local_group.last_modified {
                                    state.groups.insert(group_id, remote_group);
                                }
                            }
                            ConflictResolution::Manual => {}
                        }
                    }
                } else {
                    state.groups.insert(group_id, remote_group);
                }
            }
        }

        // Sync workspaces
        if settings.sync_enabled_types.sync_workspaces {
            for (workspace_id, remote_workspace) in remote_state.workspaces {
                if let Some(local_workspace) = state.workspaces.get(&workspace_id) {
                    if local_workspace.last_modified != remote_workspace.last_modified {
                        conflicts.push(SyncConflict {
                            item_type: "workspace".to_string(),
                            item_id: workspace_id,
                            local_version: local_workspace.last_modified,
                            remote_version: remote_workspace.last_modified,
                            device_id: remote_workspace.device_id.clone(),
                        });

                        match settings.conflict_resolution {
                            ConflictResolution::KeepLocal => {}
                            ConflictResolution::KeepRemote => {
                                state.workspaces.insert(workspace_id, remote_workspace);
                            }
                            ConflictResolution::KeepNewest => {
                                if remote_workspace.last_modified > local_workspace.last_modified {
                                    state.workspaces.insert(workspace_id, remote_workspace);
                                }
                            }
                            ConflictResolution::Manual => {}
                        }
                    }
                } else {
                    state.workspaces.insert(workspace_id, remote_workspace);
                }
            }
        }

        state.last_sync = Some(Utc::now());
        Ok(conflicts)
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
        Ok(())
    }

    pub async fn clear_sync_data(&self) {
        let mut state = self.state.write().await;
        state.tabs.clear();
        state.groups.clear();
        state.workspaces.clear();
        state.last_sync = None;
    }

    pub async fn get_sync_statistics(&self) -> SyncStatistics {
        let state = self.state.read().await;
        SyncStatistics {
            total_tabs: state.tabs.len(),
            total_groups: state.groups.len(),
            total_workspaces: state.workspaces.len(),
            last_sync: state.last_sync,
            device_id: state.device_id.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatistics {
    pub total_tabs: usize,
    pub total_groups: usize,
    pub total_workspaces: usize,
    pub last_sync: Option<DateTime<Utc>>,
    pub device_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sync_manager_creation() {
        let manager = TabSyncManager::new("test-device".to_string());
        assert_eq!(manager.get_device_id().await, "test-device");
    }

    #[tokio::test]
    async fn test_add_sync_tab() {
        let manager = TabSyncManager::new("test-device".to_string());
        let tab = SyncedTab {
            id: Uuid::new_v4(),
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
            favicon: None,
            group_id: None,
            workspace_id: None,
            is_pinned: false,
            is_hibernated: false,
            position: 0,
            last_modified: Utc::now(),
            device_id: "test-device".to_string(),
        };

        manager.add_tab_to_sync(tab.clone()).await;
        let tabs = manager.get_synced_tabs().await;
        assert_eq!(tabs.len(), 1);
        assert_eq!(tabs[0].url, "https://example.com");
    }

    #[tokio::test]
    async fn test_sync_conflict_resolution() {
        let manager = TabSyncManager::new("test-device".to_string());
        let tab_id = Uuid::new_v4();
        
        let local_tab = SyncedTab {
            id: tab_id,
            url: "https://local.com".to_string(),
            title: "Local".to_string(),
            favicon: None,
            group_id: None,
            workspace_id: None,
            is_pinned: false,
            is_hibernated: false,
            position: 0,
            last_modified: Utc::now() - chrono::Duration::hours(1),
            device_id: "test-device".to_string(),
        };

        let mut remote_tab = local_tab.clone();
        remote_tab.url = "https://remote.com".to_string();
        remote_tab.last_modified = Utc::now();

        manager.add_tab_to_sync(local_tab).await;

        let mut remote_state = SyncState {
            device_id: "remote-device".to_string(),
            last_sync: None,
            tabs: HashMap::new(),
            groups: HashMap::new(),
            workspaces: HashMap::new(),
        };
        remote_state.tabs.insert(tab_id, remote_tab);

        let conflicts = manager.sync_with_remote(remote_state).await.unwrap();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].item_type, "tab");
    }

    #[tokio::test]
    async fn test_export_import_sync_state() {
        let manager = TabSyncManager::new("test-device".to_string());
        let tab = SyncedTab {
            id: Uuid::new_v4(),
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
            favicon: None,
            group_id: None,
            workspace_id: None,
            is_pinned: false,
            is_hibernated: false,
            position: 0,
            last_modified: Utc::now(),
            device_id: "test-device".to_string(),
        };

        manager.add_tab_to_sync(tab).await;

        let exported = manager.export_sync_state().await.unwrap();
        assert!(exported.contains("example.com"));

        let new_manager = TabSyncManager::new("new-device".to_string());
        new_manager.import_sync_state(&exported).await.unwrap();
        
        let tabs = new_manager.get_synced_tabs().await;
        assert_eq!(tabs.len(), 1);
    }

    #[tokio::test]
    async fn test_sync_statistics() {
        let manager = TabSyncManager::new("test-device".to_string());
        let stats = manager.get_sync_statistics().await;
        assert_eq!(stats.total_tabs, 0);
        assert_eq!(stats.total_groups, 0);
        assert_eq!(stats.total_workspaces, 0);
    }
}