use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: Uuid,
    pub url: String,
    pub title: String,
    pub description: Option<String>,
    pub favicon: Option<String>,
    pub folder_id: Option<Uuid>,
    pub tags: Vec<String>,
    pub is_favorite: bool,
    pub is_read_later: bool,
    pub visit_count: usize,
    pub last_accessed: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub folder_id: Option<Option<Uuid>>,
    pub tags: Option<Vec<String>>,
    pub is_favorite: Option<bool>,
    pub is_read_later: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkCount {
    pub total: usize,
    pub favorites: usize,
    pub read_later: usize,
    pub recent: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum BookmarkSortOrder {
    CreatedDesc,
    CreatedAsc,
    UpdatedDesc,
    UpdatedAsc,
    TitleAsc,
    TitleDesc,
    VisitCountDesc,
    LastAccessedDesc,
}

pub struct BookmarkManager {
    bookmarks: Arc<RwLock<HashMap<Uuid, Bookmark>>>,
    storage: Arc<BookmarkStorage>,
}

impl BookmarkManager {
    pub fn new(storage_path: Option<String>) -> Self {
        let storage = Arc::new(BookmarkStorage::new(storage_path));
        Self {
            bookmarks: Arc::new(RwLock::new(HashMap::new())),
            storage,
        }
    }

    pub async fn initialize(&self) -> Result<(), String> {
        let loaded = self.storage.load_all().await?;
        let mut bookmarks = self.bookmarks.write().await;
        for bookmark in loaded {
            bookmarks.insert(bookmark.id, bookmark);
        }
        Ok(())
    }

    pub async fn add_bookmark(&self, url: String, title: String) -> Result<Bookmark, String> {
        self.add_bookmark_with_options(
            url,
            title,
            None,
            None,
            Vec::new(),
            false,
            false,
        ).await
    }

    pub async fn add_bookmark_with_options(
        &self,
        url: String,
        title: String,
        description: Option<String>,
        folder_id: Option<Uuid>,
        tags: Vec<String>,
        is_favorite: bool,
        is_read_later: bool,
    ) -> Result<Bookmark, String> {
        let now = Utc::now();
        let bookmark = Bookmark {
            id: Uuid::new_v4(),
            url,
            title,
            description,
            favicon: None,
            folder_id,
            tags,
            is_favorite,
            is_read_later,
            visit_count: 0,
            last_accessed: None,
            created_at: now,
            updated_at: now,
        };

        let mut bookmarks = self.bookmarks.write().await;
        bookmarks.insert(bookmark.id, bookmark.clone());
        
        self.storage.save(&bookmark).await?;
        Ok(bookmark)
    }

    pub async fn get_bookmark(&self, id: Uuid) -> Option<Bookmark> {
        let bookmarks = self.bookmarks.read().await;
        bookmarks.get(&id).cloned()
    }

    pub async fn get_all_bookmarks(&self) -> Vec<Bookmark> {
        let bookmarks = self.bookmarks.read().await;
        bookmarks.values().cloned().collect()
    }

    pub async fn update_bookmark(&self, id: Uuid, update: BookmarkUpdate) -> Result<Option<Bookmark>, String> {
        let mut bookmarks = self.bookmarks.write().await;
        if let Some(bookmark) = bookmarks.get_mut(&id) {
            if let Some(title) = update.title {
                bookmark.title = title;
            }
            if let Some(description) = update.description {
                bookmark.description = Some(description);
            }
            if let Some(folder_id) = update.folder_id {
                bookmark.folder_id = folder_id;
            }
            if let Some(tags) = update.tags {
                bookmark.tags = tags;
            }
            if let Some(is_favorite) = update.is_favorite {
                bookmark.is_favorite = is_favorite;
            }
            if let Some(is_read_later) = update.is_read_later {
                bookmark.is_read_later = is_read_later;
            }
            bookmark.updated_at = Utc::now();
            
            let bookmark_clone = bookmark.clone();
            self.storage.save(&bookmark_clone).await?;
            return Ok(Some(bookmark_clone));
        }
        Ok(None)
    }

    pub async fn delete_bookmark(&self, id: Uuid) -> Result<bool, String> {
        let mut bookmarks = self.bookmarks.write().await;
        if bookmarks.remove(&id).is_some() {
            self.storage.delete(id).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn visit_bookmark(&self, id: Uuid) -> Result<(), String> {
        let mut bookmarks = self.bookmarks.write().await;
        if let Some(bookmark) = bookmarks.get_mut(&id) {
            bookmark.visit_count += 1;
            bookmark.last_accessed = Some(Utc::now());
            bookmark.updated_at = Utc::now();
            
            let bookmark_clone = bookmark.clone();
            self.storage.save(&bookmark_clone).await?;
        }
        Ok(())
    }

    pub async fn toggle_favorite(&self, id: Uuid) -> Result<Option<bool>, String> {
        let mut bookmarks = self.bookmarks.write().await;
        if let Some(bookmark) = bookmarks.get_mut(&id) {
            bookmark.is_favorite = !bookmark.is_favorite;
            bookmark.updated_at = Utc::now();
            
            let bookmark_clone = bookmark.clone();
            self.storage.save(&bookmark_clone).await?;
            return Ok(Some(bookmark.is_favorite));
        }
        Ok(None)
    }

    pub async fn toggle_read_later(&self, id: Uuid) -> Result<Option<bool>, String> {
        let mut bookmarks = self.bookmarks.write().await;
        if let Some(bookmark) = bookmarks.get_mut(&id) {
            bookmark.is_read_later = !bookmark.is_read_later;
            bookmark.updated_at = Utc::now();
            
            let bookmark_clone = bookmark.clone();
            self.storage.save(&bookmark_clone).await?;
            return Ok(Some(bookmark.is_read_later));
        }
        Ok(None)
    }

    pub async fn get_favorites(&self, limit: Option<usize>) -> Vec<Bookmark> {
        let bookmarks = self.bookmarks.read().await;
        let mut favorites: Vec<_> = bookmarks.values()
            .filter(|b| b.is_favorite)
            .cloned()
            .collect();
        
        favorites.sort_by(|a, b| b.visit_count.cmp(&a.visit_count));
        
        if let Some(limit) = limit {
            favorites.truncate(limit);
        }
        
        favorites
    }

    pub async fn get_read_later(&self) -> Vec<Bookmark> {
        let bookmarks = self.bookmarks.read().await;
        bookmarks.values()
            .filter(|b| b.is_read_later)
            .cloned()
            .collect()
    }

    pub async fn get_recent(&self, limit: usize) -> Vec<Bookmark> {
        let bookmarks = self.bookmarks.read().await;
        let mut recent: Vec<_> = bookmarks.values()
            .filter(|b| b.last_accessed.is_some())
            .cloned()
            .collect();
        
        recent.sort_by(|a, b| {
            b.last_accessed.unwrap()
                .cmp(&a.last_accessed.unwrap())
        });
        
        recent.truncate(limit);
        recent
    }

    pub async fn get_most_visited(&self, limit: usize) -> Vec<Bookmark> {
        let bookmarks = self.bookmarks.read().await;
        let mut visited: Vec<_> = bookmarks.values()
            .cloned()
            .collect();
        
        visited.sort_by(|a, b| b.visit_count.cmp(&a.visit_count));
        visited.truncate(limit);
        visited
    }

    pub async fn get_by_folder(&self, folder_id: Uuid) -> Vec<Bookmark> {
        let bookmarks = self.bookmarks.read().await;
        bookmarks.values()
            .filter(|b| b.folder_id == Some(folder_id))
            .cloned()
            .collect()
    }

    pub async fn get_by_tag(&self, tag: &str) -> Vec<Bookmark> {
        let bookmarks = self.bookmarks.read().await;
        bookmarks.values()
            .filter(|b| b.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)))
            .cloned()
            .collect()
    }

    pub async fn search(&self, query: &str, limit: Option<usize>) -> Vec<Bookmark> {
        let bookmarks = self.bookmarks.read().await;
        let query_lower = query.to_lowercase();
        
        let mut results: Vec<_> = bookmarks.values()
            .filter(|b| {
                b.title.to_lowercase().contains(&query_lower)
                    || b.url.to_lowercase().contains(&query_lower)
                    || b.description.as_ref().map_or(false, |d| d.to_lowercase().contains(&query_lower))
                    || b.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect();
        
        results.sort_by(|a, b| b.visit_count.cmp(&a.visit_count));
        
        if let Some(limit) = limit {
            results.truncate(limit);
        }
        
        results
    }

    pub async fn get_bookmarks_sorted(&self, order: BookmarkSortOrder, limit: Option<usize>) -> Vec<Bookmark> {
        let mut bookmarks = self.get_all_bookmarks().await;
        
        match order {
            BookmarkSortOrder::CreatedDesc => {
                bookmarks.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            }
            BookmarkSortOrder::CreatedAsc => {
                bookmarks.sort_by(|a, b| a.created_at.cmp(&b.created_at));
            }
            BookmarkSortOrder::UpdatedDesc => {
                bookmarks.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
            }
            BookmarkSortOrder::UpdatedAsc => {
                bookmarks.sort_by(|a, b| a.updated_at.cmp(&b.updated_at));
            }
            BookmarkSortOrder::TitleAsc => {
                bookmarks.sort_by(|a, b| a.title.cmp(&b.title));
            }
            BookmarkSortOrder::TitleDesc => {
                bookmarks.sort_by(|a, b| b.title.cmp(&a.title));
            }
            BookmarkSortOrder::VisitCountDesc => {
                bookmarks.sort_by(|a, b| b.visit_count.cmp(&a.visit_count));
            }
            BookmarkSortOrder::LastAccessedDesc => {
                bookmarks.sort_by(|a, b| {
                    b.last_accessed.unwrap_or(DateTime::UNIX_EPOCH)
                        .cmp(&a.last_accessed.unwrap_or(DateTime::UNIX_EPOCH))
                });
            }
        }
        
        if let Some(limit) = limit {
            bookmarks.truncate(limit);
        }
        
        bookmarks
    }

    pub async fn get_statistics(&self) -> BookmarkCount {
        let bookmarks = self.bookmarks.read().await;
        let now = Utc::now();
        let week_ago = now - chrono::Duration::weeks(1);
        
        BookmarkCount {
            total: bookmarks.len(),
            favorites: bookmarks.values().filter(|b| b.is_favorite).count(),
            read_later: bookmarks.values().filter(|b| b.is_read_later).count(),
            recent: bookmarks.values()
                .filter(|b| b.created_at > week_ago)
                .count(),
        }
    }

    pub async fn duplicate_bookmark(&self, id: Uuid) -> Result<Option<Bookmark>, String> {
        if let Some(original) = self.get_bookmark(id).await {
            let mut bookmark = original.clone();
            bookmark.id = Uuid::new_v4();
            bookmark.title = format!("{} (Copy)", bookmark.title);
            bookmark.visit_count = 0;
            bookmark.created_at = Utc::now();
            bookmark.updated_at = Utc::now();
            
            let mut bookmarks = self.bookmarks.write().await;
            bookmarks.insert(bookmark.id, bookmark.clone());
            
            self.storage.save(&bookmark).await?;
            Ok(Some(bookmark))
        } else {
            Ok(None)
        }
    }

    pub async fn bulk_delete(&self, ids: Vec<Uuid>) -> Result<usize, String> {
        let mut deleted = 0;
        for id in ids {
            if self.delete_bookmark(id).await? {
                deleted += 1;
            }
        }
        Ok(deleted)
    }

    pub async fn bulk_add_tag(&self, ids: Vec<Uuid>, tag: String) -> Result<usize, String> {
        let mut updated = 0;
        for id in ids {
            if let Some(mut bookmark) = self.get_bookmark(id).await {
                if !bookmark.tags.contains(&tag) {
                    bookmark.tags.push(tag.clone());
                    self.update_bookmark(id, BookmarkUpdate {
                        tags: Some(bookmark.tags),
                        ..Default::default()
                    }).await?;
                    updated += 1;
                }
            }
        }
        Ok(updated)
    }
}

impl Default for BookmarkUpdate {
    fn default() -> Self {
        Self {
            title: None,
            description: None,
            folder_id: None,
            tags: None,
            is_favorite: None,
            is_read_later: None,
        }
    }
}

pub struct BookmarkStorage {
    storage_path: Option<String>,
}

impl BookmarkStorage {
    pub fn new(storage_path: Option<String>) -> Self {
        Self { storage_path }
    }

    pub async fn save(&self, bookmark: &Bookmark) -> Result<(), String> {
        if let Some(path) = &self.storage_path {
            let json = serde_json::to_string_pretty(bookmark)
                .map_err(|e| format!("Failed to serialize bookmark: {}", e))?;
            
            let filename = format!("{}/{}.json", path, bookmark.id);
            tokio::fs::write(&filename, json)
                .await
                .map_err(|e| format!("Failed to save bookmark: {}", e))?;
        }
        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), String> {
        if let Some(path) = &self.storage_path {
            let filename = format!("{}/{}.json", path, id);
            tokio::fs::remove_file(&filename)
                .await
                .ok();
        }
        Ok(())
    }

    pub async fn load_all(&self) -> Result<Vec<Bookmark>, String> {
        if let Some(path) = &self.storage_path {
            let mut bookmarks = Vec::new();
            let mut entries = tokio::fs::read_dir(path)
                .await
                .map_err(|e| format!("Failed to read bookmarks directory: {}", e))?;
            
            while let Some(entry) = entries.next_entry().await
                .map_err(|e| format!("Failed to read directory entry: {}", e))?
            {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    let content = tokio::fs::read_to_string(&path).await
                        .map_err(|e| format!("Failed to read bookmark file: {}", e))?;
                    
                    if let Ok(bookmark) = serde_json::from_str::<Bookmark>(&content) {
                        bookmarks.push(bookmark);
                    }
                }
            }
            Ok(bookmarks)
        } else {
            Ok(Vec::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bookmark_manager_creation() {
        let manager = BookmarkManager::new(None);
        let bookmark = manager.add_bookmark(
            "https://example.com".to_string(),
            "Example".to_string(),
        ).await.unwrap();
        
        assert_eq!(bookmark.url, "https://example.com");
        assert_eq!(bookmark.title, "Example");
    }

    #[tokio::test]
    async fn test_get_bookmark() {
        let manager = BookmarkManager::new(None);
        let bookmark = manager.add_bookmark(
            "https://example.com".to_string(),
            "Example".to_string(),
        ).await.unwrap();
        
        let retrieved = manager.get_bookmark(bookmark.id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().url, "https://example.com");
    }

    #[tokio::test]
    async fn test_update_bookmark() {
        let manager = BookmarkManager::new(None);
        let bookmark = manager.add_bookmark(
            "https://example.com".to_string(),
            "Example".to_string(),
        ).await.unwrap();
        
        let updated = manager.update_bookmark(bookmark.id, BookmarkUpdate {
            title: Some("Updated Title".to_string()),
            ..Default::default()
        }).await.unwrap();
        
        assert!(updated.is_some());
        assert_eq!(updated.unwrap().title, "Updated Title");
    }

    #[tokio::test]
    async fn test_delete_bookmark() {
        let manager = BookmarkManager::new(None);
        let bookmark = manager.add_bookmark(
            "https://example.com".to_string(),
            "Example".to_string(),
        ).await.unwrap();
        
        let deleted = manager.delete_bookmark(bookmark.id).await.unwrap();
        assert!(deleted);
        
        let retrieved = manager.get_bookmark(bookmark.id).await;
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_toggle_favorite() {
        let manager = BookmarkManager::new(None);
        let bookmark = manager.add_bookmark(
            "https://example.com".to_string(),
            "Example".to_string(),
        ).await.unwrap();
        
        assert!(!bookmark.is_favorite);
        
        let favorited = manager.toggle_favorite(bookmark.id).await.unwrap();
        assert_eq!(favorited, Some(true));
        
        let retrieved = manager.get_bookmark(bookmark.id).await.unwrap();
        assert!(retrieved.is_favorite);
    }

    #[tokio::test]
    async fn test_search_bookmarks() {
        let manager = BookmarkManager::new(None);
        manager.add_bookmark(
            "https://rust-lang.org".to_string(),
            "Rust Programming".to_string(),
        ).await.unwrap();
        
        manager.add_bookmark(
            "https://python.org".to_string(),
            "Python Language".to_string(),
        ).await.unwrap();
        
        let results = manager.search("programming", None).await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust Programming");
    }

    #[tokio::test]
    async fn test_bookmark_statistics() {
        let manager = BookmarkManager::new(None);
        let bookmark1 = manager.add_bookmark(
            "https://example1.com".to_string(),
            "Example 1".to_string(),
        ).await.unwrap();
        
        let bookmark2 = manager.add_bookmark_with_options(
            "https://example2.com".to_string(),
            "Example 2".to_string(),
            None,
            None,
            Vec::new(),
            true,
            false,
        ).await.unwrap();
        
        let stats = manager.get_statistics().await;
        assert_eq!(stats.total, 2);
        assert_eq!(stats.favorites, 1);
    }

    #[tokio::test]
    async fn test_duplicate_bookmark() {
        let manager = BookmarkManager::new(None);
        let bookmark = manager.add_bookmark(
            "https://example.com".to_string(),
            "Example".to_string(),
        ).await.unwrap();
        
        let duplicate = manager.duplicate_bookmark(bookmark.id).await.unwrap();
        assert!(duplicate.is_some());
        
        let dup = duplicate.unwrap();
        assert_ne!(dup.id, bookmark.id);
        assert!(dup.title.contains("Copy"));
    }
}