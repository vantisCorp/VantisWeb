//! Bookmarks Module
//! 
//! Bookmark management:
//! - Add/remove bookmarks
//! - Bookmark folders
//! - Search bookmarks
//! - Import/export

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Bookmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: String,
    pub url: String,
    pub title: String,
    pub folder: String,
    pub created_at: DateTime<Utc>,
    pub favicon: Option<String>,
}

/// Bookmark manager
pub struct BookmarkManager {
    bookmarks: HashMap<String, Bookmark>,
    folders: Vec<String>,
}

impl BookmarkManager {
    /// Create a new bookmark manager
    pub fn new() -> Self {
        info!("Initializing Bookmark Manager...");
        
        let folders = vec![
            "General".to_string(),
            "Work".to_string(),
            "Personal".to_string(),
            "Reading".to_string(),
        ];
        
        Self {
            bookmarks: HashMap::new(),
            folders,
        }
    }
    
    /// Add bookmark
    pub fn add(&mut self, url: String, title: String, folder: Option<String>) -> Result<()> {
        let bookmark = Bookmark {
            id: uuid::Uuid::new_v4().to_string(),
            url: url.clone(),
            title,
            folder: folder.unwrap_or_else(|| "General".to_string()),
            created_at: Utc::now(),
            favicon: None,
        };
        
        self.bookmarks.insert(bookmark.url.clone(), bookmark);
        info!("Added bookmark: {} (total: {})", url, self.bookmarks.len());
        
        Ok(())
    }
    
    /// Remove bookmark
    pub fn remove(&mut self, url: &str) -> Result<()> {
        if self.bookmarks.remove(url).is_some() {
            info!("Removed bookmark: {}", url);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Bookmark not found: {}", url))
        }
    }
    
    /// Get all bookmarks
    pub fn get_all(&self) -> Vec<Bookmark> {
        self.bookmarks.values().cloned().collect()
    }
    
    /// Get bookmarks in folder
    pub fn get_by_folder(&self, folder: &str) -> Vec<Bookmark> {
        self.bookmarks
            .values()
            .filter(|b| b.folder == folder)
            .cloned()
            .collect()
    }
    
    /// Search bookmarks
    pub fn search(&self, query: &str) -> Vec<Bookmark> {
        let query_lower = query.to_lowercase();
        
        self.bookmarks
            .values()
            .filter(|bookmark| {
                bookmark.url.to_lowercase().contains(&query_lower)
                    || bookmark.title.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect()
    }
    
    /// Create folder
    pub fn create_folder(&mut self, name: String) -> Result<()> {
        if self.folders.contains(&name) {
            return Err(anyhow::anyhow!("Folder already exists: {}", name));
        }
        
        self.folders.push(name.clone());
        info!("Created folder: {}", name);
        
        Ok(())
    }
    
    /// Get all folders
    pub fn get_folders(&self) -> &[String] {
        &self.folders
    }
    
    /// Check if URL is bookmarked
    pub fn is_bookmarked(&self, url: &str) -> bool {
        self.bookmarks.contains_key(url)
    }
    
    /// Get bookmark count
    pub fn count(&self) -> usize {
        self.bookmarks.len()
    }
}