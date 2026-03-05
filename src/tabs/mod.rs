//! Tab Module (Enhanced)
//!
//! Tab management with groups, workspaces, hibernation, and search.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod groups;
pub mod workspaces;
pub mod hibernation;
pub mod sync;
pub mod search;

use groups::TabGroupManager;
use workspaces::WorkspaceManager;
use hibernation::TabHibernationManager;
use sync::TabSyncManager;
use search::TabSearchManager;

/// Tab information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tab {
    pub id: String,
    pub url: String,
    pub title: String,
    pub favicon: Option<String>,
    pub is_pinned: bool,
    pub is_hibernated: bool,
    pub group_id: Option<String>,
    pub workspace_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    pub position: u32,
}

/// Enhanced tab manager
pub struct TabManager {
    tabs: Arc<RwLock<HashMap<String, Tab>>>,
    groups: Arc<TabGroupManager>,
    workspaces: Arc<WorkspaceManager>,
    hibernation: Arc<TabHibernationManager>,
    sync: Arc<TabSyncManager>,
    search: Arc<TabSearchManager>,
    current_workspace: Arc<RwLock<String>>,
}

impl TabManager {
    /// Create a new enhanced tab manager
    pub fn new() -> Self {
        let workspaces = Arc::new(WorkspaceManager::new());
        let default_workspace = workspaces.create("Default Workspace".to_string()).await;
        
        Self {
            tabs: Arc::new(RwLock::new(HashMap::new())),
            groups: Arc::new(TabGroupManager::new()),
            workspaces,
            hibernation: Arc::new(TabHibernationManager::new()),
            sync: Arc::new(TabSyncManager::new()),
            search: Arc::new(TabSearchManager::new()),
            current_workspace: Arc::new(RwLock::new(default_workspace.id)),
        }
    }

    /// Create a new tab
    pub async fn create_tab(&self, url: String, title: String) -> Result<Tab, TabError> {
        let workspace_id = self.current_workspace.read().await.clone();
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now();
        
        let tab = Tab {
            id: id.clone(),
            url: url.clone(),
            title: title.clone(),
            favicon: None,
            is_pinned: false,
            is_hibernated: false,
            group_id: None,
            workspace_id: workspace_id.clone(),
            created_at: now,
            last_accessed: now,
            position: self.get_next_position().await,
        };

        let mut tabs = self.tabs.write().await;
        tabs.insert(id.clone(), tab.clone());
        drop(tabs);

        // Auto-group suggestion
        if let Some(group_id) = self.groups.suggest_group(&url).await {
            self.assign_to_group(&id, &group_id).await?;
        }

        Ok(tab)
    }

    /// Close a tab
    pub async fn close_tab(&self, tab_id: &str) -> Result<(), TabError> {
        let mut tabs = self.tabs.write().await;
        if tabs.remove(tab_id).is_none() {
            return Err(TabError::TabNotFound(tab_id.to_string()));
        }
        Ok(())
    }

    /// Get tab by ID
    pub async fn get_tab(&self, tab_id: &str) -> Option<Tab> {
        self.tabs.read().await.get(tab_id).cloned()
    }

    /// Update tab
    pub async fn update_tab(&self, tab_id: &str, update: TabUpdate) -> Result<(), TabError> {
        let mut tabs = self.tabs.write().await;
        let tab = tabs.get_mut(tab_id)
            .ok_or_else(|| TabError::TabNotFound(tab_id.to_string()))?;

        if let Some(url) = update.url {
            tab.url = url;
        }
        if let Some(title) = update.title {
            tab.title = title;
        }
        if let Some(favicon) = update.favicon {
            tab.favicon = Some(favicon);
        }
        tab.last_accessed = chrono::Utc::now();

        Ok(())
    }

    /// Pin tab
    pub async fn pin_tab(&self, tab_id: &str) -> Result<(), TabError> {
        let mut tabs = self.tabs.write().await;
        let tab = tabs.get_mut(tab_id)
            .ok_or_else(|| TabError::TabNotFound(tab_id.to_string()))?;
        tab.is_pinned = true;
        Ok(())
    }

    /// Unpin tab
    pub async fn unpin_tab(&self, tab_id: &str) -> Result<(), TabError> {
        let mut tabs = self.tabs.write().await;
        let tab = tabs.get_mut(tab_id)
            .ok_or_else(|| TabError::TabNotFound(tab_id.to_string()))?;
        tab.is_pinned = false;
        Ok(())
    }

    /// Get all tabs in current workspace
    pub async fn get_tabs(&self) -> Vec<Tab> {
        let workspace_id = self.current_workspace.read().await.clone();
        let tabs = self.tabs.read().await;
        tabs.values()
            .filter(|t| t.workspace_id == workspace_id)
            .cloned()
            .collect()
    }

    /// Get all tabs across all workspaces
    pub async fn get_all_tabs(&self) -> Vec<Tab> {
        self.tabs.read().await.values().cloned().collect()
    }

    /// Assign tab to group
    pub async fn assign_to_group(&self, tab_id: &str, group_id: &str) -> Result<(), TabError> {
        let mut tabs = self.tabs.write().await;
        let tab = tabs.get_mut(tab_id)
            .ok_or_else(|| TabError::TabNotFound(tab_id.to_string()))?;
        tab.group_id = Some(group_id.to_string());
        Ok(())
    }

    /// Remove from group
    pub async fn remove_from_group(&self, tab_id: &str) -> Result<(), TabError> {
        let mut tabs = self.tabs.write().await;
        let tab = tabs.get_mut(tab_id)
            .ok_or_else(|| TabError::TabNotFound(tab_id.to_string()))?;
        tab.group_id = None;
        Ok(())
    }

    /// Get next position for new tab
    async fn get_next_position(&self) -> u32 {
        let tabs = self.tabs.read().await;
        tabs.values().map(|t| t.position).max().unwrap_or(0) + 1
    }

    /// Get groups manager
    pub fn groups(&self) -> &TabGroupManager {
        &self.groups
    }

    /// Get workspaces manager
    pub fn workspaces(&self) -> &WorkspaceManager {
        &self.workspaces
    }

    /// Get hibernation manager
    pub fn hibernation(&self) -> &TabHibernationManager {
        &self.hibernation
    }

    /// Get sync manager
    pub fn sync(&self) -> &TabSyncManager {
        &self.sync
    }

    /// Get search manager
    pub fn search(&self) -> &TabSearchManager {
        &self.search
    }

    /// Switch workspace
    pub async fn switch_workspace(&self, workspace_id: &str) -> Result<(), TabError> {
        // Verify workspace exists
        let workspaces = self.workspaces.get_all().await;
        if !workspaces.iter().any(|w| w.id == workspace_id) {
            return Err(TabError::WorkspaceNotFound(workspace_id.to_string()));
        }

        let mut current = self.current_workspace.write().await;
        *current = workspace_id.to_string();

        log::info!("Switched to workspace: {}", workspace_id);
        Ok(())
    }

    /// Get current workspace ID
    pub async fn current_workspace_id(&self) -> String {
        self.current_workspace.read().await.clone()
    }

    /// Detect duplicate tabs
    pub async fn detect_duplicates(&self) -> Vec<(Tab, Tab)> {
        let tabs = self.get_all_tabs().await;
        let mut duplicates = vec![];
        let mut seen = HashMap::new();

        for tab in tabs {
            if let Some(existing) = seen.get(&tab.url) {
                duplicates.push((existing.clone(), tab.clone()));
            } else {
                seen.insert(tab.url.clone(), tab);
            }
        }

        duplicates
    }

    /// Get tab count by status
    pub async fn get_tab_count(&self) -> TabCount {
        let tabs = self.tabs.read().await;
        let total = tabs.len();
        let pinned = tabs.values().filter(|t| t.is_pinned).count();
        let hibernated = tabs.values().filter(|t| t.is_hibernated).count();
        let grouped = tabs.values().filter(|t| t.group_id.is_some()).count();

        TabCount {
            total,
            pinned,
            hibernated,
            grouped,
            active: total - hibernated,
        }
    }
}

impl Default for TabManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tab update
#[derive(Debug, Clone, Default)]
pub struct TabUpdate {
    pub url: Option<String>,
    pub title: Option<String>,
    pub favicon: Option<String>,
}

/// Tab count statistics
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TabCount {
    pub total: usize,
    pub pinned: usize,
    pub hibernated: usize,
    pub grouped: usize,
    pub active: usize,
}

/// Tab errors
#[derive(Debug, thiserror::Error)]
pub enum TabError {
    #[error("Tab not found: {0}")]
    TabNotFound(String),

    #[error("Group not found: {0}")]
    GroupNotFound(String),

    #[error("Workspace not found: {0}")]
    WorkspaceNotFound(String),

    #[error("Tab limit exceeded")]
    TabLimitExceeded,

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tab_manager_creation() {
        let manager = TabManager::new();
        assert!(!manager.get_tabs().await.is_empty());
    }

    #[tokio::test]
    async fn test_create_tab() {
        let manager = TabManager::new();
        let tab = manager.create_tab("https://example.com".to_string(), "Example".to_string()).await.unwrap();
        assert_eq!(tab.url, "https://example.com");
        assert!(!tab.is_pinned);
    }

    #[tokio::test]
    async fn test_close_tab() {
        let manager = TabManager::new();
        let tab = manager.create_tab("https://example.com".to_string(), "Example".to_string()).await.unwrap();
        manager.close_tab(&tab.id).await.unwrap();
        assert!(manager.get_tab(&tab.id).await.is_none());
    }

    #[tokio::test]
    async fn test_pin_tab() {
        let manager = TabManager::new();
        let tab = manager.create_tab("https://example.com".to_string(), "Example".to_string()).await.unwrap();
        manager.pin_tab(&tab.id).await.unwrap();
        
        let updated = manager.get_tab(&tab.id).await.unwrap();
        assert!(updated.is_pinned);
    }

    #[tokio::test]
    async fn test_detect_duplicates() {
        let manager = TabManager::new();
        manager.create_tab("https://example.com".to_string(), "Example".to_string()).await.unwrap();
        manager.create_tab("https://example.com".to_string(), "Example".to_string()).await.unwrap();
        
        let duplicates = manager.detect_duplicates().await;
        assert_eq!(duplicates.len(), 1);
    }
}