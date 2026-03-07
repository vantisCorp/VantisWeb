//! Tab Group Management
//!
//! Group tabs by topic/website with color coding and drag-drop.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Tab group manager
pub struct TabGroupManager {
    groups: Arc<RwLock<HashMap<String, TabGroup>>>,
}

/// Tab group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabGroup {
    pub id: String,
    pub name: String,
    pub color: String,
    pub icon: Option<String>,
    pub tab_ids: Vec<String>,
    pub is_collapsed: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub workspace_id: String,
}

/// Group color options
pub struct GroupColors;

impl GroupColors {
    pub fn blue() -> String { "#3B82F6".to_string() }
    pub fn green() -> String { "#10B981".to_string() }
    pub fn yellow() -> String { "#F59E0B".to_string() }
    pub fn red() -> String { "#EF4444".to_string() }
    pub fn purple() -> String { "#8B5CF6".to_string() }
    pub fn pink() -> String { "#EC4899".to_string() }
    pub fn gray() -> String { "#6B7280".to_string() }
    
    pub fn all() -> Vec<String> {
        vec![
            Self::blue(),
            Self::green(),
            Self::yellow(),
            Self::red(),
            Self::purple(),
            Self::pink(),
            Self::gray(),
        ]
    }
}

impl TabGroupManager {
    /// Create a new tab group manager
    pub fn new() -> Self {
        Self {
            groups: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new group
    pub async fn create(&self, name: String, workspace_id: String) -> TabGroup {
        let id = uuid::Uuid::new_v4().to_string();
        let color = Self::get_next_color(workspace_id.clone()).await;
        
        let group = TabGroup {
            id: id.clone(),
            name,
            color,
            icon: None,
            tab_ids: vec![],
            is_collapsed: false,
            created_at: chrono::Utc::now(),
            workspace_id,
        };

        let mut groups = self.groups.write().await;
        groups.insert(id.clone(), group.clone());
        
        group
    }

    /// Get group by ID
    pub async fn get(&self, group_id: &str) -> Option<TabGroup> {
        self.groups.read().await.get(group_id).cloned()
    }

    /// Get all groups for workspace
    pub async fn get_all(&self) -> Vec<TabGroup> {
        self.groups.read().await.values().cloned().collect()
    }

    /// Get groups for workspace
    pub async fn get_for_workspace(&self, workspace_id: &str) -> Vec<TabGroup> {
        let groups = self.groups.read().await;
        groups.values()
            .filter(|g| g.workspace_id == workspace_id)
            .cloned()
            .collect()
    }

    /// Update group
    pub async fn update(&self, group_id: &str, update: GroupUpdate) -> Result<(), crate::tabs::TabError> {
        let mut groups = self.groups.write().await;
        let group = groups.get_mut(group_id)
            .ok_or_else(|| crate::tabs::TabError::GroupNotFound(group_id.to_string()))?;

        if let Some(name) = update.name {
            group.name = name;
        }
        if let Some(color) = update.color {
            group.color = color;
        }
        if let Some(icon) = update.icon {
            group.icon = Some(icon);
        }
        if let Some(is_collapsed) = update.is_collapsed {
            group.is_collapsed = is_collapsed;
        }

        Ok(())
    }

    /// Delete group
    pub async fn delete(&self, group_id: &str) -> Result<(), crate::tabs::TabError> {
        let mut groups = self.groups.write().await;
        if groups.remove(group_id).is_none() {
            return Err(crate::tabs::TabError::GroupNotFound(group_id.to_string()));
        }
        Ok(())
    }

    /// Add tab to group
    pub async fn add_tab(&self, group_id: &str, tab_id: &str) -> Result<(), crate::tabs::TabError> {
        let mut groups = self.groups.write().await;
        let group = groups.get_mut(group_id)
            .ok_or_else(|| crate::tabs::TabError::GroupNotFound(group_id.to_string()))?;

        if !group.tab_ids.contains(&tab_id.to_string()) {
            group.tab_ids.push(tab_id.to_string());
        }

        Ok(())
    }

    /// Remove tab from group
    pub async fn remove_tab(&self, group_id: &str, tab_id: &str) -> Result<(), crate::tabs::TabError> {
        let mut groups = self.groups.write().await;
        let group = groups.get_mut(group_id)
            .ok_or_else(|| crate::tabs::TabError::GroupNotFound(group_id.to_string()))?;

        group.tab_ids.retain(|id| id != tab_id);

        Ok(())
    }

    /// Move tab between groups
    pub async fn move_tab(&self, tab_id: &str, from_group: &str, to_group: &str) -> Result<(), crate::tabs::TabError> {
        self.remove_tab(from_group, tab_id).await?;
        self.add_tab(to_group, tab_id).await?;
        Ok(())
    }

    /// Collapse group
    pub async fn collapse(&self, group_id: &str) -> Result<(), crate::tabs::TabError> {
        let mut groups = self.groups.write().await;
        let group = groups.get_mut(group_id)
            .ok_or_else(|| crate::tabs::TabError::GroupNotFound(group_id.to_string()))?;
        group.is_collapsed = true;
        Ok(())
    }

    /// Expand group
    pub async fn expand(&self, group_id: &str) -> Result<(), crate::tabs::TabError> {
        let mut groups = self.groups.write().await;
        let group = groups.get_mut(group_id)
            .ok_or_else(|| crate::tabs::TabError::GroupNotFound(group_id.to_string()))?;
        group.is_collapsed = false;
        Ok(())
    }

    /// Suggest group for URL
    pub async fn suggest_group(&self, url: &str) -> Option<String> {
        let groups = self.groups.read().await;
        
        // Suggest based on domain matching
        let domain = Self::extract_domain(url);
        
        for group in groups.values() {
            if let Some(existing_url) = group.tab_ids.first() {
                // In a real implementation, would check tab URLs
                if group.name.contains(&domain) {
                    return Some(group.id.clone());
                }
            }
        }

        // Auto-create suggestion for common domains
        let suggestions = vec![
            ("github.com", "Development"),
            ("stackoverflow.com", "Development"),
            ("reddit.com", "Social"),
            ("twitter.com", "Social"),
            ("youtube.com", "Media"),
            ("netflix.com", "Media"),
        ];
        
        for (site, name) in suggestions {
            if url.contains(site) {
                // Return suggestion to create group
                return None;
            }
        }

        None
    }

    /// Auto-group tabs by domain
    pub async fn auto_group(&self, tab_ids: Vec<String>, url_map: HashMap<String, String>) -> Vec<String> {
        let mut domain_groups: HashMap<String, Vec<String>> = HashMap::new();

        for tab_id in &tab_ids {
            if let Some(url) = url_map.get(tab_id) {
                let domain = Self::extract_domain(url);
                domain_groups.entry(domain).or_insert_with(Vec::new).push(tab_id.clone());
            }
        }

        let mut created_groups = vec![];
        for (domain, tabs) in domain_groups {
            if tabs.len() > 1 {
                let workspace_id = "default".to_string(); // In real impl, get from context
                let group = self.create(domain.clone(), workspace_id).await;
                for tab_id in tabs {
                    let _ = self.add_tab(&group.id, &tab_id).await;
                }
                created_groups.push(group.id);
            }
        }

        created_groups
    }

    /// Get next available color
    async fn get_next_color(workspace_id: String) -> String {
        // In a real implementation, would check existing groups and rotate colors
        GroupColors::all()[0].clone()
    }

    /// Extract domain from URL
    fn extract_domain(url: &str) -> String {
        url.replace("https://", "")
           .replace("http://", "")
           .replace("www.", "")
           .split('/')
           .next()
           .unwrap_or("unknown")
           .to_string()
    }

    /// Save groups
    pub async fn save(&self) -> Result<String, crate::tabs::TabError> {
        let groups = self.groups.read().await;
        let json = serde_json::to_string(&*groups)
            .map_err(|e| crate::tabs::TabError::InvalidOperation(e.to_string()))?;
        Ok(json)
    }

    /// Load groups
    pub async fn load(&self, json: &str) -> Result<(), crate::tabs::TabError> {
        let loaded: HashMap<String, TabGroup> = serde_json::from_str(json)
            .map_err(|e| crate::tabs::TabError::InvalidOperation(e.to_string()))?;
        
        let mut groups = self.groups.write().await;
        for (id, group) in loaded {
            groups.insert(id, group);
        }
        Ok(())
    }
}

impl Default for TabGroupManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Group update
#[derive(Debug, Clone, Default)]
pub struct GroupUpdate {
    pub name: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub is_collapsed: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_group_manager_creation() {
        let manager = TabGroupManager::new();
        assert_eq!(manager.get_all().await.len(), 0);
    }

    #[tokio::test]
    async fn test_create_group() {
        let manager = TabGroupManager::new();
        let group = manager.create("Development".to_string(), "workspace-1".to_string()).await;
        assert_eq!(group.name, "Development");
        assert!(!group.tab_ids.is_empty());
    }

    #[tokio::test]
    async fn test_add_remove_tab() {
        let manager = TabGroupManager::new();
        let group = manager.create("Test".to_string(), "workspace-1".to_string()).await;
        
        manager.add_tab(&group.id, "tab-1").await.unwrap();
        let updated = manager.get(&group.id).await.unwrap();
        assert_eq!(updated.tab_ids.len(), 1);
        
        manager.remove_tab(&group.id, "tab-1").await.unwrap();
        let updated = manager.get(&group.id).await.unwrap();
        assert_eq!(updated.tab_ids.len(), 0);
    }

    #[tokio::test]
    async fn test_collapse_expand() {
        let manager = TabGroupManager::new();
        let group = manager.create("Test".to_string(), "workspace-1".to_string()).await;
        
        manager.collapse(&group.id).await.unwrap();
        let updated = manager.get(&group.id).await.unwrap();
        assert!(updated.is_collapsed);
        
        manager.expand(&group.id).await.unwrap();
        let updated = manager.get(&group.id).await.unwrap();
        assert!(!updated.is_collapsed);
    }

    #[tokio::test]
    async fn test_move_tab() {
        let manager = TabGroupManager::new();
        let group1 = manager.create("Group 1".to_string(), "workspace-1".to_string()).await;
        let group2 = manager.create("Group 2".to_string(), "workspace-1".to_string()).await;
        
        manager.add_tab(&group1.id, "tab-1").await.unwrap();
        manager.move_tab("tab-1", &group1.id, &group2.id).await.unwrap();
        
        let g1 = manager.get(&group1.id).await.unwrap();
        let g2 = manager.get(&group2.id).await.unwrap();
        assert!(!g1.tab_ids.iter().any(|id| id == "tab-1"));
        assert!(g2.tab_ids.iter().any(|id| id == "tab-1"));
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(TabGroupManager::extract_domain("https://www.example.com/page"), "example.com");
        assert_eq!(TabGroupManager::extract_domain("http://github.com/user/repo"), "github.com");
    }
}