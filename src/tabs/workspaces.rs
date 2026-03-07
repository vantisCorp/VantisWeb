//! Workspace Management
//!
//! Multiple workspaces with separate tab sets and settings.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Workspace manager
pub struct WorkspaceManager {
    workspaces: Arc<RwLock<HashMap<String, Workspace>>>,
}

/// Workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub color: String,
    pub tab_ids: Vec<String>,
    pub settings: WorkspaceSettings,
    pub extensions: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
}

/// Workspace-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSettings {
    pub zoom_level: f32,
    pub enable_dark_mode: bool,
    pub content_blocking: bool,
    pub custom_styles: HashMap<String, String>,
    pub startup_urls: Vec<String>,
    pub session_restore: bool,
}

impl Default for WorkspaceSettings {
    fn default() -> Self {
        Self {
            zoom_level: 1.0,
            enable_dark_mode: false,
            content_blocking: true,
            custom_styles: HashMap::new(),
            startup_urls: vec![],
            session_restore: true,
        }
    }
}

/// Workspace template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceTemplate {
    pub name: String,
    pub icon: String,
    pub settings: WorkspaceSettings,
    pub default_tabs: Vec<String>,
    pub extensions: Vec<String>,
}

impl WorkspaceManager {
    /// Create a new workspace manager
    pub fn new() -> Self {
        Self {
            workspaces: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new workspace
    pub async fn create(&self, name: String) -> Workspace {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now();
        
        let workspace = Workspace {
            id: id.clone(),
            name,
            icon: None,
            color: self.get_next_color().await,
            tab_ids: vec![],
            settings: WorkspaceSettings::default(),
            extensions: vec![],
            created_at: now,
            last_accessed: now,
        };

        let mut workspaces = self.workspaces.write().await;
        workspaces.insert(id.clone(), workspace.clone());
        
        workspace
    }

    /// Get workspace by ID
    pub async fn get(&self, workspace_id: &str) -> Option<Workspace> {
        self.workspaces.read().await.get(workspace_id).cloned()
    }

    /// Get all workspaces
    pub async fn get_all(&self) -> Vec<Workspace> {
        self.workspaces.read().await.values().cloned().collect()
    }

    /// Update workspace
    pub async fn update(&self, workspace_id: &str, update: WorkspaceUpdate) -> Result<(), WorkspaceError> {
        let mut workspaces = self.workspaces.write().await;
        let workspace = workspaces.get_mut(workspace_id)
            .ok_or_else(|| WorkspaceError::NotFound(workspace_id.to_string()))?;

        if let Some(name) = update.name {
            workspace.name = name;
        }
        if let Some(icon) = update.icon {
            workspace.icon = Some(icon);
        }
        if let Some(color) = update.color {
            workspace.color = color;
        }
        if let Some(settings) = update.settings {
            workspace.settings = settings;
        }

        Ok(())
    }

    /// Delete workspace
    pub async fn delete(&self, workspace_id: &str) -> Result<(), WorkspaceError> {
        let mut workspaces = self.workspaces.write().await;
        if workspaces.remove(workspace_id).is_none() {
            return Err(WorkspaceError::NotFound(workspace_id.to_string()));
        }
        Ok(())
    }

    /// Add tab to workspace
    pub async fn add_tab(&self, workspace_id: &str, tab_id: &str) -> Result<(), WorkspaceError> {
        let mut workspaces = self.workspaces.write().await;
        let workspace = workspaces.get_mut(workspace_id)
            .ok_or_else(|| WorkspaceError::NotFound(workspace_id.to_string()))?;

        if !workspace.tab_ids.contains(&tab_id.to_string()) {
            workspace.tab_ids.push(tab_id.to_string());
        }

        Ok(())
    }

    /// Remove tab from workspace
    pub async fn remove_tab(&self, workspace_id: &str, tab_id: &str) -> Result<(), WorkspaceError> {
        let mut workspaces = self.workspaces.write().await;
        let workspace = workspaces.get_mut(workspace_id)
            .ok_or_else(|| WorkspaceError::NotFound(workspace_id.to_string()))?;

        workspace.tab_ids.retain(|id| id != tab_id);

        Ok(())
    }

    /// Update last accessed
    pub async fn touch(&self, workspace_id: &str) -> Result<(), WorkspaceError> {
        let mut workspaces = self.workspaces.write().await;
        let workspace = workspaces.get_mut(workspace_id)
            .ok_or_else(|| WorkspaceError::NotFound(workspace_id.to_string()))?;
        workspace.last_accessed = chrono::Utc::now();
        Ok(())
    }

    /// Create workspace from template
    pub async fn create_from_template(&self, template: &WorkspaceTemplate) -> Workspace {
        let mut workspace = self.create(template.name.clone()).await;
        
        workspace.icon = Some(template.icon.clone());
        workspace.settings = template.settings.clone();
        workspace.extensions = template.extensions.clone();
        
        let mut workspaces = self.workspaces.write().await;
        if let Some(w) = workspaces.get_mut(&workspace.id) {
            w.icon = workspace.icon.clone();
            w.settings = workspace.settings.clone();
            w.extensions = workspace.extensions.clone();
        }
        drop(workspaces);

        workspace
    }

    /// Save workspace as template
    pub async fn save_as_template(&self, workspace_id: &str) -> Result<WorkspaceTemplate, WorkspaceError> {
        let workspace = self.get(workspace_id).await
            .ok_or_else(|| WorkspaceError::NotFound(workspace_id.to_string()))?;

        Ok(WorkspaceTemplate {
            name: workspace.name.clone(),
            icon: workspace.icon.clone().unwrap_or_default(),
            settings: workspace.settings.clone(),
            default_tabs: vec![],
            extensions: workspace.extensions.clone(),
        })
    }

    /// Get workspace settings
    pub async fn get_settings(&self, workspace_id: &str) -> Result<WorkspaceSettings, WorkspaceError> {
        let workspace = self.get(workspace_id).await
            .ok_or_else(|| WorkspaceError::NotFound(workspace_id.to_string()))?;
        Ok(workspace.settings)
    }

    /// Update workspace settings
    pub async fn update_settings(&self, workspace_id: &str, settings: WorkspaceSettings) -> Result<(), WorkspaceError> {
        let mut workspaces = self.workspaces.write().await;
        let workspace = workspaces.get_mut(workspace_id)
            .ok_or_else(|| WorkspaceError::NotFound(workspace_id.to_string()))?;
        workspace.settings = settings;
        Ok(())
    }

    /// Add extension to workspace
    pub async fn add_extension(&self, workspace_id: &str, extension_id: &str) -> Result<(), WorkspaceError> {
        let mut workspaces = self.workspaces.write().await;
        let workspace = workspaces.get_mut(workspace_id)
            .ok_or_else(|| WorkspaceError::NotFound(workspace_id.to_string()))?;

        if !workspace.extensions.contains(&extension_id.to_string()) {
            workspace.extensions.push(extension_id.to_string());
        }

        Ok(())
    }

    /// Remove extension from workspace
    pub async fn remove_extension(&self, workspace_id: &str, extension_id: &str) -> Result<(), WorkspaceError> {
        let mut workspaces = self.workspaces.write().await;
        let workspace = workspaces.get_mut(workspace_id)
            .ok_or_else(|| WorkspaceError::NotFound(workspace_id.to_string()))?;

        workspace.extensions.retain(|id| id != extension_id);

        Ok(())
    }

    /// Export workspace
    pub async fn export(&self, workspace_id: &str) -> Result<String, WorkspaceError> {
        let workspace = self.get(workspace_id).await
            .ok_or_else(|| WorkspaceError::NotFound(workspace_id.to_string()))?;

        let json = serde_json::to_string(&workspace)
            .map_err(|e| WorkspaceError::ExportError(e.to_string()))?;

        Ok(json)
    }

    /// Import workspace
    pub async fn import(&self, json: &str) -> Result<Workspace, WorkspaceError> {
        let workspace: Workspace = serde_json::from_str(json)
            .map_err(|e| WorkspaceError::ImportError(e.to_string()))?;

        let mut workspaces = self.workspaces.write().await;
        workspaces.insert(workspace.id.clone(), workspace.clone());

        Ok(workspace)
    }

    /// Get next color for new workspace
    async fn get_next_color(&self) -> String {
        let colors = vec![
            "#3B82F6".to_string(), // Blue
            "#10B981".to_string(), // Green
            "#F59E0B".to_string(), // Yellow
            "#EF4444".to_string(), // Red
            "#8B5CF6".to_string(), // Purple
            "#EC4899".to_string(), // Pink
        ];

        let workspaces = self.workspaces.read().await;
        let count = workspaces.len();
        colors[count % colors.len()].clone()
    }

    /// Get workspace count
    pub async fn count(&self) -> usize {
        self.workspaces.read().await.len()
    }
}

impl Default for WorkspaceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Workspace update
#[derive(Debug, Clone, Default)]
pub struct WorkspaceUpdate {
    pub name: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub settings: Option<WorkspaceSettings>,
}

/// Workspace errors
#[derive(Debug, thiserror::Error)]
pub enum WorkspaceError {
    #[error("Workspace not found: {0}")]
    NotFound(String),

    #[error("Export error: {0}")]
    ExportError(String),

    #[error("Import error: {0}")]
    ImportError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_workspace_manager_creation() {
        let manager = WorkspaceManager::new();
        assert_eq!(manager.count().await, 0);
    }

    #[tokio::test]
    async fn test_create_workspace() {
        let manager = WorkspaceManager::new();
        let workspace = manager.create("Work".to_string()).await;
        assert_eq!(workspace.name, "Work");
        assert!(!workspace.id.is_empty());
    }

    #[tokio::test]
    async fn test_update_workspace() {
        let manager = WorkspaceManager::new();
        let workspace = manager.create("Test".to_string()).await;
        
        manager.update(&workspace.id, WorkspaceUpdate {
            name: Some("Updated".to_string()),
            ..Default::default()
        }).await.unwrap();
        
        let updated = manager.get(&workspace.id).await.unwrap();
        assert_eq!(updated.name, "Updated");
    }

    #[tokio::test]
    async fn test_add_remove_tab() {
        let manager = WorkspaceManager::new();
        let workspace = manager.create("Test".to_string()).await;
        
        manager.add_tab(&workspace.id, "tab-1").await.unwrap();
        let ws = manager.get(&workspace.id).await.unwrap();
        assert_eq!(ws.tab_ids.len(), 1);
        
        manager.remove_tab(&workspace.id, "tab-1").await.unwrap();
        let ws = manager.get(&workspace.id).await.unwrap();
        assert_eq!(ws.tab_ids.len(), 0);
    }

    #[tokio::test]
    async fn test_create_from_template() {
        let manager = WorkspaceManager::new();
        let template = WorkspaceTemplate {
            name: "Development".to_string(),
            icon: "code".to_string(),
            settings: WorkspaceSettings {
                zoom_level: 1.5,
                ..Default::default()
            },
            default_tabs: vec!["https://github.com".to_string()],
            extensions: vec!["dev-tools".to_string()],
        };
        
        let workspace = manager.create_from_template(&template).await;
        assert_eq!(workspace.name, "Development");
        assert_eq!(workspace.icon, Some("code".to_string()));
        assert_eq!(workspace.settings.zoom_level, 1.5);
    }

    #[tokio::test]
    async fn test_export_import() {
        let manager = WorkspaceManager::new();
        let workspace = manager.create("Test".to_string()).await;
        
        let json = manager.export(&workspace.id).await.unwrap();
        
        manager.delete(&workspace.id).await.unwrap();
        
        let imported = manager.import(&json).await.unwrap();
        assert_eq!(imported.name, "Test");
    }

    #[tokio::test]
    async fn test_extension_management() {
        let manager = WorkspaceManager::new();
        let workspace = manager.create("Test".to_string()).await;
        
        manager.add_extension(&workspace.id, "ext-1").await.unwrap();
        let ws = manager.get(&workspace.id).await.unwrap();
        assert_eq!(ws.extensions.len(), 1);
        
        manager.remove_extension(&workspace.id, "ext-1").await.unwrap();
        let ws = manager.get(&workspace.id).await.unwrap();
        assert_eq!(ws.extensions.len(), 0);
    }
}