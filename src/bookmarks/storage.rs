use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use super::Bookmark;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkIndex {
    pub by_id: HashMap<Uuid, Bookmark>,
    pub by_url: HashMap<String, Vec<Uuid>>,
    pub by_folder: HashMap<Uuid, Vec<Uuid>>,
    pub by_tag: HashMap<String, Vec<Uuid>>,
    pub favorites: Vec<Uuid>,
    pub read_later: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkStorageConfig {
    pub storage_path: String,
    pub index_path: String,
    pub backup_enabled: bool,
    pub backup_interval_hours: u64,
    pub auto_save: bool,
    pub compression_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkBackup {
    pub backup_id: Uuid,
    pub bookmarks: Vec<Bookmark>,
    pub created_at: DateTime<Utc>,
    pub bookmark_count: usize,
    pub compressed: bool,
}

pub struct BookmarkStorage {
    config: BookmarkStorageConfig,
    index: Arc<RwLock<BookmarkIndex>>,
    cache: Arc<RwLock<HashMap<Uuid, Bookmark>>>,
    backup_path: PathBuf,
}

impl BookmarkStorage {
    pub fn new(config: BookmarkStorageConfig) -> Self {
        let storage_path = PathBuf::from(&config.storage_path);
        let backup_path = storage_path.join("backups");
        
        Self {
            config,
            index: Arc::new(RwLock::new(BookmarkIndex {
                by_id: HashMap::new(),
                by_url: HashMap::new(),
                by_folder: HashMap::new(),
                by_tag: HashMap::new(),
                favorites: Vec::new(),
                read_later: Vec::new(),
            })),
            cache: Arc::new(RwLock::new(HashMap::new())),
            backup_path,
        }
    }

    pub async fn initialize(&self) -> Result<(), String> {
        // Create directories if they don't exist
        tokio::fs::create_dir_all(&self.config.storage_path).await
            .map_err(|e| format!("Failed to create storage directory: {}", e))?;
        
        if self.config.backup_enabled {
            tokio::fs::create_dir_all(&self.backup_path).await
                .map_err(|e| format!("Failed to create backup directory: {}", e))?;
        }
        
        // Load existing bookmarks
        self.load_all().await?;
        
        Ok(())
    }

    pub async fn save(&self, bookmark: &Bookmark) -> Result<(), String> {
        // Save to file
        let file_path = self.get_bookmark_path(bookmark.id);
        let json = serde_json::to_string_pretty(bookmark)
            .map_err(|e| format!("Failed to serialize bookmark: {}", e))?;
        
        if self.config.compression_enabled {
            let compressed = self.compress(&json)
                .map_err(|e| format!("Failed to compress bookmark: {}", e))?;
            tokio::fs::write(file_path.with_extension("json.gz"), compressed).await
                .map_err(|e| format!("Failed to write bookmark file: {}", e))?;
        } else {
            tokio::fs::write(file_path, json).await
                .map_err(|e| format!("Failed to write bookmark file: {}", e))?;
        }
        
        // Update index
        self.update_index(bookmark).await;
        
        // Update cache
        let mut cache = self.cache.write().await;
        cache.insert(bookmark.id, bookmark.clone());
        
        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), String> {
        let file_path = self.get_bookmark_path(id);
        
        // Try compressed first, then regular
        let compressed_path = file_path.with_extension("json.gz");
        if compressed_path.exists() {
            tokio::fs::remove_file(compressed_path).await
                .map_err(|e| format!("Failed to delete bookmark file: {}", e))?;
        } else if file_path.exists() {
            tokio::fs::remove_file(file_path).await
                .map_err(|e| format!("Failed to delete bookmark file: {}", e))?;
        }
        
        // Update index
        let mut index = self.index.write().await;
        let mut cache = self.cache.write().await;
        
        if let Some(bookmark) = index.by_id.remove(&id) {
            // Remove from URL index
            if let Some(urls) = index.by_url.get_mut(&bookmark.url) {
                urls.retain(|&x| x != id);
                if urls.is_empty() {
                    index.by_url.remove(&bookmark.url);
                }
            }
            
            // Remove from folder index
            if let Some(folder_id) = bookmark.folder_id {
                if let Some(folder) = index.by_folder.get_mut(&folder_id) {
                    folder.retain(|&x| x != id);
                    if folder.is_empty() {
                        index.by_folder.remove(&folder_id);
                    }
                }
            }
            
            // Remove from tag index
            for tag in &bookmark.tags {
                if let Some(tags) = index.by_tag.get_mut(tag) {
                    tags.retain(|&x| x != id);
                    if tags.is_empty() {
                        index.by_tag.remove(tag);
                    }
                }
            }
            
            // Remove from favorites/read_later
            index.favorites.retain(|&x| x != id);
            index.read_later.retain(|&x| x != id);
        }
        
        cache.remove(&id);
        
        Ok(())
    }

    pub async fn load(&self, id: Uuid) -> Result<Option<Bookmark>, String> {
        // Check cache first
        let cache = self.cache.read().await;
        if let Some(bookmark) = cache.get(&id) {
            return Ok(Some(bookmark.clone()));
        }
        drop(cache);
        
        // Check index
        let index = self.index.read().await;
        if let Some(bookmark) = index.by_id.get(&id) {
            return Ok(Some(bookmark.clone()));
        }
        drop(index);
        
        // Load from file
        let file_path = self.get_bookmark_path(id);
        
        // Try compressed first
        let compressed_path = file_path.with_extension("json.gz");
        let content = if compressed_path.exists() {
            let bytes = tokio::fs::read(&compressed_path).await
                .map_err(|e| format!("Failed to read bookmark file: {}", e))?;
            self.decompress(&bytes)
                .map_err(|e| format!("Failed to decompress bookmark: {}", e))?
        } else if file_path.exists() {
            tokio::fs::read_to_string(&file_path).await
                .map_err(|e| format!("Failed to read bookmark file: {}", e))?
        } else {
            return Ok(None);
        };
        
        let bookmark: Bookmark = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to deserialize bookmark: {}", e))?;
        
        // Update cache
        let mut cache = self.cache.write().await;
        cache.insert(id, bookmark.clone());
        
        Ok(Some(bookmark))
    }

    pub async fn load_all(&self) -> Result<Vec<Bookmark>, String> {
        let mut bookmarks = Vec::new();
        let storage_path = PathBuf::from(&self.config.storage_path);
        
        let mut entries = tokio::fs::read_dir(&storage_path).await
            .map_err(|e| format!("Failed to read storage directory: {}", e))?;
        
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| format!("Failed to read directory entry: {}", e))?
        {
            let path = entry.path();
            let is_compressed = path.extension().and_then(|s| s.to_str()) == Some("gz");
            
            if is_compressed {
                if let Some(stem) = path.file_stem() {
                    if stem.to_str().map(|s| s.ends_with(".json")).unwrap_or(false) {
                        let id_str = stem.to_str()
                            .and_then(|s| s.strip_suffix(".json"))
                            .ok_or("Invalid bookmark filename")?;
                        let id = Uuid::parse_str(id_str)
                            .map_err(|e| format!("Invalid bookmark UUID: {}", e))?;
                        
                        if let Ok(Some(bookmark)) = self.load(id).await {
                            bookmarks.push(bookmark);
                        }
                    }
                }
            } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(stem) = path.file_stem() {
                    let id_str = stem.to_str()
                        .ok_or("Invalid bookmark filename")?;
                    let id = Uuid::parse_str(id_str)
                        .map_err(|e| format!("Invalid bookmark UUID: {}", e))?;
                    
                    if let Ok(Some(bookmark)) = self.load(id).await {
                        bookmarks.push(bookmark);
                    }
                }
            }
        }
        
        Ok(bookmarks)
    }

    pub async fn get_by_url(&self, url: &str) -> Result<Vec<Bookmark>, String> {
        let index = self.index.read().await;
        if let Some(ids) = index.by_url.get(url) {
            let mut bookmarks = Vec::new();
            for id in ids {
                if let Some(bookmark) = index.by_id.get(id) {
                    bookmarks.push(bookmark.clone());
                }
            }
            Ok(bookmarks)
        } else {
            Ok(Vec::new())
        }
    }

    pub async fn get_by_folder(&self, folder_id: Uuid) -> Result<Vec<Bookmark>, String> {
        let index = self.index.read().await;
        if let Some(ids) = index.by_folder.get(&folder_id) {
            let mut bookmarks = Vec::new();
            for id in ids {
                if let Some(bookmark) = index.by_id.get(id) {
                    bookmarks.push(bookmark.clone());
                }
            }
            Ok(bookmarks)
        } else {
            Ok(Vec::new())
        }
    }

    pub async fn get_by_tag(&self, tag: &str) -> Result<Vec<Bookmark>, String> {
        let index = self.index.read().await;
        if let Some(ids) = index.by_tag.get(tag) {
            let mut bookmarks = Vec::new();
            for id in ids {
                if let Some(bookmark) = index.by_id.get(id) {
                    bookmarks.push(bookmark.clone());
                }
            }
            Ok(bookmarks)
        } else {
            Ok(Vec::new())
        }
    }

    pub async fn get_favorites(&self) -> Result<Vec<Bookmark>, String> {
        let index = self.index.read().await;
        let mut bookmarks = Vec::new();
        for id in &index.favorites {
            if let Some(bookmark) = index.by_id.get(id) {
                bookmarks.push(bookmark.clone());
            }
        }
        bookmarks.sort_by(|a, b| b.visit_count.cmp(&a.visit_count));
        Ok(bookmarks)
    }

    pub async fn get_read_later(&self) -> Result<Vec<Bookmark>, String> {
        let index = self.index.read().await;
        let mut bookmarks = Vec::new();
        for id in &index.read_later {
            if let Some(bookmark) = index.by_id.get(id) {
                bookmarks.push(bookmark.clone());
            }
        }
        Ok(bookmarks)
    }

    pub async fn get_statistics(&self) -> BookmarkStorageStats {
        let index = self.index.read().await;
        BookmarkStorageStats {
            total_bookmarks: index.by_id.len(),
            total_urls: index.by_url.len(),
            total_folders: index.by_folder.len(),
            total_tags: index.by_tag.len(),
            favorites_count: index.favorites.len(),
            read_later_count: index.read_later.len(),
        }
    }

    pub async fn create_backup(&self) -> Result<BookmarkBackup, String> {
        if !self.config.backup_enabled {
            return Err("Backup is not enabled".to_string());
        }
        
        let bookmarks = self.load_all().await?;
        let backup = BookmarkBackup {
            backup_id: Uuid::new_v4(),
            bookmarks: bookmarks.clone(),
            created_at: Utc::now(),
            bookmark_count: bookmarks.len(),
            compressed: self.config.compression_enabled,
        };
        
        let backup_json = serde_json::to_string_pretty(&backup)
            .map_err(|e| format!("Failed to serialize backup: {}", e))?;
        
        let backup_file = self.backup_path.join(format!("backup_{}.json", backup.backup_id));
        tokio::fs::write(backup_file, backup_json).await
            .map_err(|e| format!("Failed to write backup file: {}", e))?;
        
        Ok(backup)
    }

    pub async fn restore_backup(&self, backup_id: Uuid) -> Result<(), String> {
        let backup_file = self.backup_path.join(format!("backup_{}.json", backup_id));
        let backup_json = tokio::fs::read_to_string(&backup_file).await
            .map_err(|e| format!("Failed to read backup file: {}", e))?;
        
        let backup: BookmarkBackup = serde_json::from_str(&backup_json)
            .map_err(|e| format!("Failed to deserialize backup: {}", e))?;
        
        // Clear existing bookmarks
        let current = self.load_all().await?;
        for bookmark in current {
            self.delete(bookmark.id).await?;
        }
        
        // Restore from backup
        for bookmark in backup.bookmarks {
            self.save(&bookmark).await?;
        }
        
        Ok(())
    }

    pub async fn list_backups(&self) -> Result<Vec<BookmarkBackup>, String> {
        let mut backups = Vec::new();
        let mut entries = tokio::fs::read_dir(&self.backup_path).await
            .map_err(|e| format!("Failed to read backup directory: {}", e))?;
        
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| format!("Failed to read directory entry: {}", e))?
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = tokio::fs::read_to_string(&path).await
                    .map_err(|e| format!("Failed to read backup file: {}", e))?;
                
                if let Ok(backup) = serde_json::from_str::<BookmarkBackup>(&content) {
                    backups.push(backup);
                }
            }
        }
        
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(backups)
    }

    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    async fn update_index(&self, bookmark: &Bookmark) {
        let mut index = self.index.write().await;
        
        // Add to by_id
        index.by_id.insert(bookmark.id, bookmark.clone());
        
        // Add to by_url
        index.by_url.entry(bookmark.url.clone())
            .or_insert_with(Vec::new)
            .push(bookmark.id);
        
        // Add to by_folder
        if let Some(folder_id) = bookmark.folder_id {
            index.by_folder.entry(folder_id)
                .or_insert_with(Vec::new)
                .push(bookmark.id);
        }
        
        // Add to by_tag
        for tag in &bookmark.tags {
            index.by_tag.entry(tag.clone())
                .or_insert_with(Vec::new)
                .push(bookmark.id);
        }
        
        // Add to favorites/read_later
        if bookmark.is_favorite {
            if !index.favorites.contains(&bookmark.id) {
                index.favorites.push(bookmark.id);
            }
        } else {
            index.favorites.retain(|&x| x != bookmark.id);
        }
        
        if bookmark.is_read_later {
            if !index.read_later.contains(&bookmark.id) {
                index.read_later.push(bookmark.id);
            }
        } else {
            index.read_later.retain(|&x| x != bookmark.id);
        }
    }

    fn get_bookmark_path(&self, id: Uuid) -> PathBuf {
        PathBuf::from(&self.config.storage_path)
            .join(format!("{}.json", id))
    }

    fn compress(&self, data: &str) -> Result<Vec<u8>, String> {
        // Simple compression placeholder - in production, use flate2 or similar
        Ok(data.as_bytes().to_vec())
    }

    fn decompress(&self, data: &[u8]) -> Result<String, String> {
        // Simple decompression placeholder
        String::from_utf8(data.to_vec())
            .map_err(|e| format!("Failed to decompress: {}", e))
    }
}

impl Default for BookmarkStorageConfig {
    fn default() -> Self {
        Self {
            storage_path: "./bookmarks".to_string(),
            index_path: "./bookmarks/index.json".to_string(),
            backup_enabled: true,
            backup_interval_hours: 24,
            auto_save: true,
            compression_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkStorageStats {
    pub total_bookmarks: usize,
    pub total_urls: usize,
    pub total_folders: usize,
    pub total_tags: usize,
    pub favorites_count: usize,
    pub read_later_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_creation() {
        let config = BookmarkStorageConfig {
            storage_path: "/tmp/test_bookmarks".to_string(),
            ..Default::default()
        };
        let storage = BookmarkStorage::new(config);
        
        storage.initialize().await.unwrap();
        assert!(PathBuf::from("/tmp/test_bookmarks").exists());
    }

    #[tokio::test]
    async fn test_save_and_load() {
        let config = BookmarkStorageConfig {
            storage_path: "/tmp/test_bookmarks_2".to_string(),
            ..Default::default()
        };
        let storage = BookmarkStorage::new(config);
        storage.initialize().await.unwrap();
        
        let bookmark = Bookmark {
            id: Uuid::new_v4(),
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
            description: Some("Test description".to_string()),
            favicon: None,
            folder_id: None,
            tags: vec!["test".to_string()],
            is_favorite: true,
            is_read_later: false,
            visit_count: 5,
            last_accessed: Some(Utc::now()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        storage.save(&bookmark).await.unwrap();
        
        let loaded = storage.load(bookmark.id).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().title, "Example");
    }

    #[tokio::test]
    async fn test_delete_bookmark() {
        let config = BookmarkStorageConfig {
            storage_path: "/tmp/test_bookmarks_3".to_string(),
            ..Default::default()
        };
        let storage = BookmarkStorage::new(config);
        storage.initialize().await.unwrap();
        
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
        
        storage.save(&bookmark).await.unwrap();
        storage.delete(bookmark.id).await.unwrap();
        
        let loaded = storage.load(bookmark.id).await.unwrap();
        assert!(loaded.is_none());
    }
}