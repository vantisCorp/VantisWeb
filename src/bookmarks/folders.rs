use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkFolder {
    pub id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub is_smart: bool,
    pub smart_filter: Option<SmartFilter>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartFilter {
    pub filter_type: SmartFilterType,
    pub value: String,
    pub match_case: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SmartFilterType {
    Domain,
    Tag,
    TitleContains,
    UrlContains,
    DateAfter,
    DateBefore,
    IsFavorite,
    IsReadLater,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderUpdate {
    pub name: Option<String>,
    pub parent_id: Option<Option<Uuid>>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderStats {
    pub folder_id: Uuid,
    pub total_bookmarks: usize,
    pub subfolders_count: usize,
    pub depth: usize,
}

#[derive(Debug, Clone)]
pub struct FolderColors;

impl FolderColors {
    pub const RED: &'static str = "#ef4444";
    pub const ORANGE: &'static str = "#f97316";
    pub const YELLOW: &'static str = "#eab308";
    pub const GREEN: &'static str = "#22c55e";
    pub const BLUE: &'static str = "#3b82f6";
    pub const INDIGO: &'static str = "#6366f1";
    pub const PURPLE: &'static str = "#a855f7";
    pub const PINK: &'static str = "#ec4899";
    pub const GRAY: &'static str = "#6b7280";
    
    pub fn all() -> Vec<&'static str> {
        vec![
            Self::RED, Self::ORANGE, Self::YELLOW, Self::GREEN,
            Self::BLUE, Self::INDIGO, Self::PURPLE, Self::PINK, Self::GRAY,
        ]
    }
    
    pub fn get_name(color: &str) -> &str {
        match color {
            Self::RED => "Red",
            Self::ORANGE => "Orange",
            Self::YELLOW => "Yellow",
            Self::GREEN => "Green",
            Self::BLUE => "Blue",
            Self::INDIGO => "Indigo",
            Self::PURPLE => "Purple",
            Self::PINK => "Pink",
            Self::GRAY => "Gray",
            _ => "Custom",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderTree {
    pub folder: BookmarkFolder,
    pub children: Vec<FolderTree>,
}

pub struct BookmarkFolderManager {
    folders: Arc<RwLock<HashMap<Uuid, BookmarkFolder>>>,
    root_folder_id: Uuid,
}

impl BookmarkFolderManager {
    pub fn new() -> Self {
        let root_id = Uuid::new_v4();
        let root_folder = BookmarkFolder {
            id: root_id,
            name: "Bookmarks".to_string(),
            parent_id: None,
            description: Some("Root bookmark folder".to_string()),
            color: Some(FolderColors::BLUE.to_string()),
            icon: Some("folder".to_string()),
            is_smart: false,
            smart_filter: None,
            sort_order: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let mut folders = HashMap::new();
        folders.insert(root_id, root_folder);
        
        Self {
            folders: Arc::new(RwLock::new(folders)),
            root_folder_id: root_id,
        }
    }

    pub async fn create_folder(
        &self,
        name: String,
        parent_id: Option<Uuid>,
    ) -> Result<BookmarkFolder, String> {
        self.create_folder_with_options(
            name,
            parent_id,
            None,
            None,
            None,
            false,
            None,
        ).await
    }

    pub async fn create_folder_with_options(
        &self,
        name: String,
        parent_id: Option<Uuid>,
        description: Option<String>,
        color: Option<String>,
        icon: Option<String>,
        is_smart: bool,
        smart_filter: Option<SmartFilter>,
    ) -> Result<BookmarkFolder, String> {
        // Validate parent exists
        if let Some(pid) = parent_id {
            let folders = self.folders.read().await;
            if !folders.contains_key(&pid) {
                return Err("Parent folder not found".to_string());
            }
        }
        
        let now = Utc::now();
        let folder = BookmarkFolder {
            id: Uuid::new_v4(),
            name,
            parent_id,
            description,
            color,
            icon,
            is_smart,
            smart_filter,
            sort_order: 0,
            created_at: now,
            updated_at: now,
        };
        
        let mut folders = self.folders.write().await;
        folders.insert(folder.id, folder.clone());
        
        Ok(folder)
    }

    pub async fn get_folder(&self, id: Uuid) -> Option<BookmarkFolder> {
        let folders = self.folders.read().await;
        folders.get(&id).cloned()
    }

    pub async fn get_root_folder(&self) -> BookmarkFolder {
        self.get_folder(self.root_folder_id).await.unwrap()
    }

    pub async fn update_folder(&self, id: Uuid, update: FolderUpdate) -> Result<Option<BookmarkFolder>, String> {
        let mut folders = self.folders.write().await;
        if let Some(folder) = folders.get_mut(&id) {
            if let Some(name) = update.name {
                folder.name = name;
            }
            if let Some(parent_id) = update.parent_id {
                // Check for circular reference
                if parent_id.is_some() {
                    if self.would_create_cycle(&folders, id, parent_id.unwrap()) {
                        return Err("Cannot create circular folder structure".to_string());
                    }
                }
                folder.parent_id = parent_id;
            }
            if let Some(description) = update.description {
                folder.description = Some(description);
            }
            if let Some(color) = update.color {
                folder.color = Some(color);
            }
            if let Some(icon) = update.icon {
                folder.icon = Some(icon);
            }
            if let Some(sort_order) = update.sort_order {
                folder.sort_order = sort_order;
            }
            folder.updated_at = Utc::now();
            
            return Ok(Some(folder.clone()));
        }
        Ok(None)
    }

    fn would_create_cycle(
        &self,
        folders: &HashMap<Uuid, BookmarkFolder>,
        folder_id: Uuid,
        new_parent_id: Uuid,
    ) -> bool {
        if folder_id == new_parent_id {
            return true;
        }
        
        let mut current = new_parent_id;
        while let Some(folder) = folders.get(&current) {
            if let Some(parent_id) = folder.parent_id {
                if parent_id == folder_id {
                    return true;
                }
                current = parent_id;
            } else {
                break;
            }
        }
        
        false
    }

    pub async fn delete_folder(&self, id: Uuid) -> Result<bool, String> {
        if id == self.root_folder_id {
            return Err("Cannot delete root folder".to_string());
        }
        
        let mut folders = self.folders.write().await;
        
        // Check if folder has subfolders
        let has_subfolders = folders.values()
            .any(|f| f.parent_id == Some(id));
        
        if has_subfolders {
            return Err("Cannot delete folder with subfolders. Move or delete subfolders first.".to_string());
        }
        
        Ok(folders.remove(&id).is_some())
    }

    pub async fn get_subfolders(&self, parent_id: Uuid) -> Vec<BookmarkFolder> {
        let folders = self.folders.read().await;
        let mut subfolders: Vec<_> = folders.values()
            .filter(|f| f.parent_id == Some(parent_id))
            .cloned()
            .collect();
        
        subfolders.sort_by(|a, b| a.sort_order.cmp(&b.sort_order));
        subfolders
    }

    pub async fn get_folder_path(&self, id: Uuid) -> Vec<BookmarkFolder> {
        let folders = self.folders.read().await;
        let mut path = Vec::new();
        
        let mut current_id = Some(id);
        while let Some(id) = current_id {
            if let Some(folder) = folders.get(&id) {
                path.push(folder.clone());
                current_id = folder.parent_id;
            } else {
                break;
            }
        }
        
        path.reverse();
        path
    }

    pub async fn get_all_folders(&self) -> Vec<BookmarkFolder> {
        let folders = self.folders.read().await;
        let mut all: Vec<_> = folders.values().cloned().collect();
        all.sort_by(|a, b| {
            a.sort_order.cmp(&b.sort_order)
                .then_with(|| a.name.cmp(&b.name))
        });
        all
    }

    pub async fn get_folder_tree(&self, parent_id: Option<Uuid>) -> Vec<FolderTree> {
        let folders = self.folders.read().await;
        let parent = parent_id.unwrap_or(self.root_folder_id);
        
        self.build_tree(&folders, parent).await
    }

    async fn build_tree(
        &self,
        folders: &HashMap<Uuid, BookmarkFolder>,
        parent_id: Uuid,
    ) -> Vec<FolderTree> {
        let mut children: Vec<_> = folders.values()
            .filter(|f| f.parent_id == Some(parent_id))
            .cloned()
            .collect();
        
        children.sort_by(|a, b| a.sort_order.cmp(&b.sort_order));
        
        children.into_iter()
            .map(|folder| {
                let sub_children = futures::executor::block_on(
                    self.build_tree(folders, folder.id)
                );
                FolderTree {
                    folder,
                    children: sub_children,
                }
            })
            .collect()
    }

    pub async fn move_folder(&self, id: Uuid, new_parent_id: Option<Uuid>) -> Result<(), String> {
        if id == self.root_folder_id {
            return Err("Cannot move root folder".to_string());
        }
        
        // Validate new parent exists
        if let Some(pid) = new_parent_id {
            let folders = self.folders.read().await;
            if !folders.contains_key(&pid) {
                return Err("Target folder not found".to_string());
            }
            drop(folders);
            
            // Check for cycle
            let folders = self.folders.read().await;
            if self.would_create_cycle(&folders, id, pid) {
                return Err("Cannot move folder into itself or its descendants".to_string());
            }
        }
        
        let mut folders = self.folders.write().await;
        if let Some(folder) = folders.get_mut(&id) {
            folder.parent_id = new_parent_id;
            folder.updated_at = Utc::now();
            Ok(())
        } else {
            Err("Folder not found".to_string())
        }
    }

    pub async fn rename_folder(&self, id: Uuid, new_name: String) -> Result<BookmarkFolder, String> {
        let mut folders = self.folders.write().await;
        if let Some(folder) = folders.get_mut(&id) {
            folder.name = new_name;
            folder.updated_at = Utc::now();
            Ok(folder.clone())
        } else {
            Err("Folder not found".to_string())
        }
    }

    pub async fn set_folder_color(&self, id: Uuid, color: String) -> Result<BookmarkFolder, String> {
        let mut folders = self.folders.write().await;
        if let Some(folder) = folders.get_mut(&id) {
            folder.color = Some(color);
            folder.updated_at = Utc::now();
            Ok(folder.clone())
        } else {
            Err("Folder not found".to_string())
        }
    }

    pub async fn get_folder_depth(&self, id: Uuid) -> usize {
        let folders = self.folders.read().await;
        let mut depth = 0;
        let mut current_id = Some(id);
        
        while let Some(id) = current_id {
            if let Some(folder) = folders.get(&id) {
                depth += 1;
                current_id = folder.parent_id;
            } else {
                break;
            }
        }
        
        depth
    }

    pub async fn create_smart_folder(
        &self,
        name: String,
        parent_id: Option<Uuid>,
        filter: SmartFilter,
    ) -> Result<BookmarkFolder, String> {
        self.create_folder_with_options(
            name,
            parent_id,
            Some("Smart folder - auto-updated".to_string()),
            Some(FolderColors::PURPLE.to_string()),
            Some("smart-folder".to_string()),
            true,
            Some(filter),
        ).await
    }

    pub async fn get_folder_stats(&self, id: Uuid) -> FolderStats {
        let folders = self.folders.read().await;
        let depth = self.get_folder_depth(id).await;
        let subfolders_count = folders.values()
            .filter(|f| f.parent_id == Some(id))
            .count();
        
        FolderStats {
            folder_id: id,
            total_bookmarks: 0, // Would need bookmark manager reference
            subfolders_count,
            depth,
        }
    }

    pub async fn save(&self) -> Result<String, String> {
        let folders = self.folders.read().await;
        let folders_vec: Vec<_> = folders.values().cloned().collect();
        
        serde_json::to_string_pretty(&folders_vec)
            .map_err(|e| format!("Failed to serialize folders: {}", e))
    }

    pub async fn load(&self, json: &str) -> Result<(), String> {
        let folders_vec: Vec<BookmarkFolder> = serde_json::from_str(json)
            .map_err(|e| format!("Failed to deserialize folders: {}", e))?;
        
        let mut folders = self.folders.write().await;
        folders.clear();
        
        for folder in folders_vec {
            folders.insert(folder.id, folder);
        }
        
        Ok(())
    }
}

impl Default for BookmarkFolderManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_folder_manager_creation() {
        let manager = BookmarkFolderManager::new();
        let root = manager.get_root_folder().await;
        assert_eq!(root.name, "Bookmarks");
    }

    #[tokio::test]
    async fn test_create_folder() {
        let manager = BookmarkFolderManager::new();
        let root = manager.get_root_folder().await;
        
        let folder = manager.create_folder(
            "Tech Sites".to_string(),
            Some(root.id),
        ).await.unwrap();
        
        assert_eq!(folder.name, "Tech Sites");
        assert_eq!(folder.parent_id, Some(root.id));
    }

    #[tokio::test]
    async fn test_get_subfolders() {
        let manager = BookmarkFolderManager::new();
        let root = manager.get_root_folder().await;
        
        manager.create_folder("Folder 1".to_string(), Some(root.id)).await.unwrap();
        manager.create_folder("Folder 2".to_string(), Some(root.id)).await.unwrap();
        
        let subfolders = manager.get_subfolders(root.id).await;
        assert_eq!(subfolders.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_folder() {
        let manager = BookmarkFolderManager::new();
        let root = manager.get_root_folder().await;
        
        let folder = manager.create_folder(
            "To Delete".to_string(),
            Some(root.id),
        ).await.unwrap();
        
        let deleted = manager.delete_folder(folder.id).await.unwrap();
        assert!(deleted);
        
        let retrieved = manager.get_folder(folder.id).await;
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_folder_path() {
        let manager = BookmarkFolderManager::new();
        let root = manager.get_root_folder().await;
        
        let parent = manager.create_folder(
            "Parent".to_string(),
            Some(root.id),
        ).await.unwrap();
        
        let child = manager.create_folder(
            "Child".to_string(),
            Some(parent.id),
        ).await.unwrap();
        
        let path = manager.get_folder_path(child.id).await;
        assert_eq!(path.len(), 3); // root -> parent -> child
        assert_eq!(path[0].name, "Bookmarks");
        assert_eq!(path[1].name, "Parent");
        assert_eq!(path[2].name, "Child");
    }

    #[tokio::test]
    async fn test_cannot_delete_root() {
        let manager = BookmarkFolderManager::new();
        let root = manager.get_root_folder().await;
        
        let result = manager.delete_folder(root.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_circular_reference_prevention() {
        let manager = BookmarkFolderManager::new();
        let root = manager.get_root_folder().await;
        
        let folder1 = manager.create_folder(
            "Folder 1".to_string(),
            Some(root.id),
        ).await.unwrap();
        
        let folder2 = manager.create_folder(
            "Folder 2".to_string(),
            Some(folder1.id),
        ).await.unwrap();
        
        // Try to move folder1 into folder2 (would create cycle)
        let result = manager.move_folder(folder1.id, Some(folder2.id)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_smart_folder() {
        let manager = BookmarkFolderManager::new();
        let root = manager.get_root_folder().await;
        
        let smart = manager.create_smart_folder(
            "GitHub Bookmarks".to_string(),
            Some(root.id),
            SmartFilter {
                filter_type: SmartFilterType::Domain,
                value: "github.com".to_string(),
                match_case: false,
            },
        ).await.unwrap();
        
        assert!(smart.is_smart);
        assert!(smart.smart_filter.is_some());
    }
}