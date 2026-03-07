//! Article Storage
//! 
//! This module provides persistent storage for saved articles,
//! enabling offline reading and reading history tracking.
//! 
//! # Features
//! - Save articles for offline reading
//! - Track reading history and progress
//! - Full-text search across saved articles
//! - Export and import capabilities

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{Article, ArticleMetadata, ReadingProgress, Result, ReadingError};

/// Article storage
pub struct ArticleStorage {
    articles: Arc<RwLock<HashMap<String, StoredArticle>>>,
    history: Arc<RwLock<Vec<HistoryEntry>>>,
    storage_path: Option<PathBuf>,
}

/// Stored article with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredArticle {
    pub article: Article,
    pub saved_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub folder: Option<String>,
    pub tags: Vec<String>,
    pub is_favorite: bool,
    pub is_archived: bool,
}

/// History entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub url: String,
    pub title: String,
    pub visited_at: DateTime<Utc>,
    pub time_spent: u32, // seconds
    pub scroll_position: f64,
}

/// Article list options
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListOptions {
    pub folder: Option<String>,
    pub tags: Vec<String>,
    pub favorites_only: bool,
    pub archived_only: bool,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub sort_by: SortBy,
    pub sort_desc: bool,
}

/// Sort options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SortBy {
    #[default]
    SavedDate,
    LastAccessed,
    Title,
    Author,
    ReadingTime,
}

impl ArticleStorage {
    /// Create a new article storage
    pub fn new() -> Self {
        Self {
            articles: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(Vec::new())),
            storage_path: None,
        }
    }

    /// Create storage with persistence
    pub fn with_path(path: PathBuf) -> Self {
        Self {
            articles: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(Vec::new())),
            storage_path: Some(path),
        }
    }

    /// Save an article
    pub async fn save(&self, article: &Article) -> Result<()> {
        let id = article.id.clone();
        let stored = StoredArticle {
            article: article.clone(),
            saved_at: Utc::now(),
            last_accessed: Utc::now(),
            folder: None,
            tags: Vec::new(),
            is_favorite: false,
            is_archived: false,
        };

        self.articles.write().await.insert(id, stored);

        if let Some(ref path) = self.storage_path {
            self.persist_to_disk(path).await?;
        }

        Ok(())
    }

    /// Load an article by ID
    pub async fn load(&self, id: &str) -> Result<Option<Article>> {
        let mut articles = self.articles.write().await;
        
        if let Some(stored) = articles.get_mut(id) {
            stored.last_accessed = Utc::now();
            
            if let Some(ref path) = self.storage_path {
                self.persist_to_disk(path).await?;
            }
            
            Ok(Some(stored.article.clone()))
        } else {
            Ok(None)
        }
    }

    /// Update an article
    pub async fn update(&self, id: &str, article: &Article) -> Result<()> {
        let mut articles = self.articles.write().await;
        
        if let Some(stored) = articles.get_mut(id) {
            stored.article = article.clone();
            stored.last_accessed = Utc::now();
            
            if let Some(ref path) = self.storage_path {
                drop(articles);
                self.persist_to_disk(path).await?;
            }
            
            Ok(())
        } else {
            Err(ReadingError::StorageError(format!("Article not found: {}", id)))
        }
    }

    /// Delete an article
    pub async fn delete(&self, id: &str) -> Result<()> {
        self.articles.write().await.remove(id);
        
        if let Some(ref path) = self.storage_path {
            self.persist_to_disk(path).await?;
        }
        
        Ok(())
    }

    /// List all articles
    pub async fn list(&self) -> Result<Vec<Article>> {
        let articles = self.articles.read().await;
        Ok(articles.values().map(|s| s.article.clone()).collect())
    }

    /// List articles with options
    pub async fn list_with_options(&self, options: &ListOptions) -> Result<Vec<Article>> {
        let articles = self.articles.read().await;
        
        let mut filtered: Vec<&StoredArticle> = articles.values()
            .filter(|a| {
                // Filter by folder
                if let Some(ref folder) = options.folder {
                    if a.folder.as_ref() != Some(folder) {
                        return false;
                    }
                }
                
                // Filter by tags
                if !options.tags.is_empty() {
                    if !options.tags.iter().all(|t| a.tags.contains(t)) {
                        return false;
                    }
                }
                
                // Filter favorites
                if options.favorites_only && !a.is_favorite {
                    return false;
                }
                
                // Filter archived
                if options.archived_only != a.is_archived {
                    return false;
                }
                
                true
            })
            .collect();

        // Sort
        filtered.sort_by(|a, b| {
            let cmp = match options.sort_by {
                SortBy::SavedDate => a.saved_at.cmp(&b.saved_at),
                SortBy::LastAccessed => a.last_accessed.cmp(&b.last_accessed),
                SortBy::Title => a.article.metadata.title.cmp(&b.article.metadata.title),
                SortBy::Author => {
                    let a_author = a.article.metadata.author.as_ref();
                    let b_author = b.article.metadata.author.as_ref();
                    a_author.cmp(&b_author)
                }
                SortBy::ReadingTime => {
                    a.article.metadata.reading_time_minutes.cmp(&b.article.metadata.reading_time_minutes)
                }
            };
            
            if options.sort_desc {
                cmp.reverse()
            } else {
                cmp
            }
        });

        // Apply offset and limit
        let start = options.offset.unwrap_or(0);
        let result: Vec<Article> = filtered
            .into_iter()
            .skip(start)
            .take(options.limit.unwrap_or(usize::MAX))
            .map(|s| s.article.clone())
            .collect();

        Ok(result)
    }

    /// Search articles
    pub async fn search(&self, query: &str) -> Result<Vec<Article>> {
        let query_lower = query.to_lowercase();
        let articles = self.articles.read().await;
        
        let results: Vec<Article> = articles.values()
            .filter(|stored| {
                let article = &stored.article;
                
                // Search in title
                if article.metadata.title.to_lowercase().contains(&query_lower) {
                    return true;
                }
                
                // Search in content
                if article.content.text.to_lowercase().contains(&query_lower) {
                    return true;
                }
                
                // Search in author
                if let Some(ref author) = article.metadata.author {
                    if author.to_lowercase().contains(&query_lower) {
                        return true;
                    }
                }
                
                // Search in tags
                for tag in &stored.tags {
                    if tag.to_lowercase().contains(&query_lower) {
                        return true;
                    }
                }
                
                false
            })
            .map(|s| s.article.clone())
            .collect();

        Ok(results)
    }

    /// Mark article as favorite
    pub async fn set_favorite(&self, id: &str, favorite: bool) -> Result<()> {
        let mut articles = self.articles.write().await;
        
        if let Some(stored) = articles.get_mut(id) {
            stored.is_favorite = favorite;
            Ok(())
        } else {
            Err(ReadingError::StorageError(format!("Article not found: {}", id)))
        }
    }

    /// Archive article
    pub async fn set_archived(&self, id: &str, archived: bool) -> Result<()> {
        let mut articles = self.articles.write().await;
        
        if let Some(stored) = articles.get_mut(id) {
            stored.is_archived = archived;
            Ok(())
        } else {
            Err(ReadingError::StorageError(format!("Article not found: {}", id)))
        }
    }

    /// Set article folder
    pub async fn set_folder(&self, id: &str, folder: Option<String>) -> Result<()> {
        let mut articles = self.articles.write().await;
        
        if let Some(stored) = articles.get_mut(id) {
            stored.folder = folder;
            Ok(())
        } else {
            Err(ReadingError::StorageError(format!("Article not found: {}", id)))
        }
    }

    /// Add tag to article
    pub async fn add_tag(&self, id: &str, tag: &str) -> Result<()> {
        let mut articles = self.articles.write().await;
        
        if let Some(stored) = articles.get_mut(id) {
            if !stored.tags.contains(&tag.to_string()) {
                stored.tags.push(tag.to_string());
            }
            Ok(())
        } else {
            Err(ReadingError::StorageError(format!("Article not found: {}", id)))
        }
    }

    /// Remove tag from article
    pub async fn remove_tag(&self, id: &str, tag: &str) -> Result<()> {
        let mut articles = self.articles.write().await;
        
        if let Some(stored) = articles.get_mut(id) {
            stored.tags.retain(|t| t != tag);
            Ok(())
        } else {
            Err(ReadingError::StorageError(format!("Article not found: {}", id)))
        }
    }

    /// Get all folders
    pub async fn get_folders(&self) -> Vec<String> {
        let articles = self.articles.read().await;
        let mut folders: Vec<String> = articles.values()
            .filter_map(|a| a.folder.clone())
            .collect();
        
        folders.sort();
        folders.dedup();
        folders
    }

    /// Get all tags
    pub async fn get_tags(&self) -> Vec<String> {
        let articles = self.articles.read().await;
        let mut tags: Vec<String> = articles.values()
            .flat_map(|a| a.tags.clone())
            .collect();
        
        tags.sort();
        tags.dedup();
        tags
    }

    /// Add to reading history
    pub async fn add_to_history(&self, entry: HistoryEntry) {
        let mut history = self.history.write().await;
        
        // Keep only last 1000 entries
        if history.len() >= 1000 {
            history.remove(0);
        }
        
        history.push(entry);
    }

    /// Get reading history
    pub async fn get_history(&self, limit: Option<usize>) -> Vec<HistoryEntry> {
        let history = self.history.read().await;
        
        history.iter()
            .rev()
            .take(limit.unwrap_or(100))
            .cloned()
            .collect()
    }

    /// Clear reading history
    pub async fn clear_history(&self) {
        self.history.write().await.clear();
    }

    /// Get storage statistics
    pub async fn get_stats(&self) -> StorageStats {
        let articles = self.articles.read().await;
        
        let total_articles = articles.len();
        let favorites = articles.values().filter(|a| a.is_favorite).count();
        let archived = articles.values().filter(|a| a.is_archived).count();
        
        let total_words: u32 = articles.values()
            .map(|a| a.article.metadata.word_count)
            .sum();
        
        let total_reading_time: u32 = articles.values()
            .map(|a| a.article.metadata.reading_time_minutes)
            .sum();

        StorageStats {
            total_articles,
            favorites,
            archived,
            total_words,
            total_reading_time,
            folders: self.get_folders().await.len(),
            tags: self.get_tags().await.len(),
        }
    }

    /// Export articles to JSON
    pub async fn export(&self) -> Result<String> {
        let articles = self.articles.read().await;
        let export_data = ExportData {
            articles: articles.values().cloned().collect(),
            exported_at: Utc::now(),
            version: "1.0".to_string(),
        };
        
        serde_json::to_string(&export_data)
            .map_err(|e| ReadingError::StorageError(format!("Export failed: {}", e)))
    }

    /// Import articles from JSON
    pub async fn import(&self, json: &str) -> Result<usize> {
        let import_data: ExportData = serde_json::from_str(json)
            .map_err(|e| ReadingError::StorageError(format!("Import failed: {}", e)))?;
        
        let mut articles = self.articles.write().await;
        let count = import_data.articles.len();
        
        for stored in import_data.articles {
            articles.insert(stored.article.id.clone(), stored);
        }
        
        Ok(count)
    }

    /// Persist to disk
    async fn persist_to_disk(&self, _path: &PathBuf) -> Result<()> {
        // In a real implementation, this would write to disk
        Ok(())
    }

    /// Clear all stored articles
    pub async fn clear(&self) {
        self.articles.write().await.clear();
        self.history.write().await.clear();
    }
}

impl Default for ArticleStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_articles: usize,
    pub favorites: usize,
    pub archived: usize,
    pub total_words: u32,
    pub total_reading_time: u32,
    pub folders: usize,
    pub tags: usize,
}

/// Export data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub articles: Vec<StoredArticle>,
    pub exported_at: DateTime<Utc>,
    pub version: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_save_and_load() {
        let storage = ArticleStorage::new();
        
        let article = Article {
            id: "test-1".to_string(),
            metadata: ArticleMetadata {
                title: "Test Article".to_string(),
                url: "https://example.com".to_string(),
                author: Some("Test Author".to_string()),
                published_date: None,
                excerpt: None,
                featured_image: None,
                reading_time_minutes: 5,
                word_count: 1000,
                domain: "example.com".to_string(),
                language: None,
                tags: vec![],
                extracted_at: Utc::now(),
            },
            content: super::super::ArticleContent {
                html: "<p>Test content</p>".to_string(),
                text: "Test content".to_string(),
                images: vec![],
                links: vec![],
                videos: vec![],
            },
            reading_progress: ReadingProgress::new(),
        };
        
        storage.save(&article).await.unwrap();
        
        let loaded = storage.load("test-1").await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().metadata.title, "Test Article");
    }

    #[tokio::test]
    async fn test_search() {
        let storage = ArticleStorage::new();
        
        let article = Article {
            id: "test-search".to_string(),
            metadata: ArticleMetadata {
                title: "Rust Programming Guide".to_string(),
                url: "https://example.com/rust".to_string(),
                author: Some("Jane Doe".to_string()),
                published_date: None,
                excerpt: None,
                featured_image: None,
                reading_time_minutes: 10,
                word_count: 2000,
                domain: "example.com".to_string(),
                language: None,
                tags: vec![],
                extracted_at: Utc::now(),
            },
            content: super::super::ArticleContent {
                html: "<p>Learn Rust programming</p>".to_string(),
                text: "Learn Rust programming".to_string(),
                images: vec![],
                links: vec![],
                videos: vec![],
            },
            reading_progress: ReadingProgress::new(),
        };
        
        storage.save(&article).await.unwrap();
        
        let results = storage.search("Rust").await.unwrap();
        assert_eq!(results.len(), 1);
        
        let results = storage.search("Python").await.unwrap();
        assert_eq!(results.len(), 0);
    }
}