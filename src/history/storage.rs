/// # History Storage Module
/// 
/// Provides persistent storage for browser history entries.
/// Supports efficient querying, indexing, and retrieval.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{HistoryEntry, HistoryError, Result};

/// Storage errors
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Entry not found: {0}")]
    NotFound(String),
    #[error("Invalid data: {0}")]
    InvalidData(String),
}

/// History storage backend
pub struct HistoryStorage {
    /// In-memory storage (would be replaced with SQLite in production)
    entries: Arc<RwLock<HashMap<String, HistoryEntry>>>,
    /// URL index for fast lookups
    url_index: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Domain index for domain-based queries
    domain_index: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Date index for time-based queries
    date_index: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Maximum entries to store
    max_entries: usize,
}

impl HistoryStorage {
    /// Create a new history storage
    pub async fn new() -> Result<Self> {
        Ok(Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            url_index: Arc::new(RwLock::new(HashMap::new())),
            domain_index: Arc::new(RwLock::new(HashMap::new())),
            date_index: Arc::new(RwLock::new(HashMap::new())),
            max_entries: 100000,
        })
    }

    /// Add a history entry
    pub async fn add_entry(&self, entry: HistoryEntry) -> Result<()> {
        // Check if entry with same URL exists
        let existing_id = {
            let entries = self.entries.read().await;
            entries.values()
                .find(|e| e.url == entry.url)
                .map(|e| e.id.clone())
        };

        if let Some(id) = existing_id {
            // Update existing entry
            let mut entries = self.entries.write().await;
            if let Some(existing) = entries.get_mut(&id) {
                existing.increment_visit();
                existing.last_visit = Utc::now();
            }
        } else {
            // Add new entry
            let id = entry.id.clone();
            let url = entry.url.clone();
            let domain = extract_domain(&url);
            let date_key = entry.timestamp.format("%Y-%m-%d").to_string();

            // Add to main storage
            self.entries.write().await.insert(id.clone(), entry);

            // Update URL index
            self.url_index.write().await
                .entry(url)
                .or_insert_with(Vec::new)
                .push(id.clone());

            // Update domain index
            self.domain_index.write().await
                .entry(domain)
                .or_insert_with(Vec::new)
                .push(id.clone());

            // Update date index
            self.date_index.write().await
                .entry(date_key)
                .or_insert_with(Vec::new)
                .push(id);

            // Enforce max entries limit
            self.enforce_limit().await?;
        }

        Ok(())
    }

    /// Get entry by ID
    pub async fn get_entry(&self, id: &str) -> Option<HistoryEntry> {
        self.entries.read().await.get(id).cloned()
    }

    /// Get all entries
    pub async fn get_all_entries(&self) -> Result<Vec<HistoryEntry>> {
        let entries = self.entries.read().await;
        Ok(entries.values().cloned().collect())
    }

    /// Get entries by domain
    pub async fn get_entries_by_domain(&self, domain: &str) -> Result<Vec<HistoryEntry>> {
        let domain_index = self.domain_index.read().await;
        let entries = self.entries.read().await;

        let ids = domain_index.get(domain).cloned().unwrap_or_default();
        Ok(ids.iter()
            .filter_map(|id| entries.get(id).cloned())
            .collect())
    }

    /// Get entries by date
    pub async fn get_entries_by_date(&self, date: &str) -> Result<Vec<HistoryEntry>> {
        let date_index = self.date_index.read().await;
        let entries = self.entries.read().await;

        let ids = date_index.get(date).cloned().unwrap_or_default();
        Ok(ids.iter()
            .filter_map(|id| entries.get(id).cloned())
            .collect())
    }

    /// Delete entries by IDs
    pub async fn delete_entries(&self, ids: Vec<String>) -> Result<()> {
        let mut entries = self.entries.write().await;
        let mut url_index = self.url_index.write().await;
        let mut domain_index = self.domain_index.write().await;
        let mut date_index = self.date_index.write().await;

        for id in ids {
            if let Some(entry) = entries.remove(&id) {
                // Remove from URL index
                if let Some(url_ids) = url_index.get_mut(&entry.url) {
                    url_ids.retain(|x| x != &id);
                }

                // Remove from domain index
                let domain = extract_domain(&entry.url);
                if let Some(domain_ids) = domain_index.get_mut(&domain) {
                    domain_ids.retain(|x| x != &id);
                }

                // Remove from date index
                let date_key = entry.timestamp.format("%Y-%m-%d").to_string();
                if let Some(date_ids) = date_index.get_mut(&date_key) {
                    date_ids.retain(|x| x != &id);
                }
            }
        }

        Ok(())
    }

    /// Clear all entries
    pub async fn clear_all(&self) -> Result<()> {
        self.entries.write().await.clear();
        self.url_index.write().await.clear();
        self.domain_index.write().await.clear();
        self.date_index.write().await.clear();
        Ok(())
    }

    /// Clear entries by date range
    pub async fn clear_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<()> {
        let entries = self.entries.read().await;
        let ids_to_delete: Vec<String> = entries.values()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .map(|e| e.id.clone())
            .collect();
        drop(entries);

        self.delete_entries(ids_to_delete).await
    }

    /// Get total entry count
    pub async fn count(&self) -> usize {
        self.entries.read().await.len()
    }

    /// Enforce maximum entries limit
    async fn enforce_limit(&self) -> Result<()> {
        let mut entries = self.entries.write().await;
        
        if entries.len() > self.max_entries {
            // Remove oldest entries
            let mut entries_vec: Vec<_> = entries.iter().collect();
            entries_vec.sort_by_key(|(_, e)| e.timestamp);

            let to_remove = entries.len() - self.max_entries;
            for (id, _) in entries_vec.iter().take(to_remove) {
                // Remove from indices
                if let Some(entry) = entries.get(*id) {
                    let url = &entry.url;
                    let domain = extract_domain(url);
                    let date_key = entry.timestamp.format("%Y-%m-%d").to_string();
                    
                    // We'll clean up indices later
                }
            }

            // Actually remove entries
            let ids: Vec<String> = entries_vec.iter()
                .take(to_remove)
                .map(|(id, _)| (*id).clone())
                .collect();
            
            for id in ids {
                entries.remove(&id);
            }
        }

        Ok(())
    }

    /// Rebuild all indices
    pub async fn rebuild_indices(&self) -> Result<()> {
        let entries = self.entries.read().await;
        
        let mut url_index = HashMap::new();
        let mut domain_index = HashMap::new();
        let mut date_index = HashMap::new();

        for (id, entry) in entries.iter() {
            let url = &entry.url;
            let domain = extract_domain(url);
            let date_key = entry.timestamp.format("%Y-%m-%d").to_string();

            url_index.entry(url.clone())
                .or_insert_with(Vec::new)
                .push(id.clone());

            domain_index.entry(domain)
                .or_insert_with(Vec::new)
                .push(id.clone());

            date_index.entry(date_key)
                .or_insert_with(Vec::new)
                .push(id.clone());
        }

        *self.url_index.write().await = url_index;
        *self.domain_index.write().await = domain_index;
        *self.date_index.write().await = date_index;

        Ok(())
    }
}

/// Extract domain from URL
fn extract_domain(url: &str) -> String {
    // Simple domain extraction (would use url crate in production)
    let url = url.trim_start_matches("https://")
                .trim_start_matches("http://")
                .trim_start_matches("www.");
    
    url.split('/').next().unwrap_or(url).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_creation() {
        let storage = HistoryStorage::new().await.unwrap();
        assert_eq!(storage.count().await, 0);
    }

    #[tokio::test]
    async fn test_add_entry() {
        let storage = HistoryStorage::new().await.unwrap();
        
        let entry = HistoryEntry::new(
            "https://example.com".to_string(),
            "Example".to_string(),
            false,
        );

        storage.add_entry(entry).await.unwrap();
        assert_eq!(storage.count().await, 1);
    }

    #[tokio::test]
    async fn test_get_entry() {
        let storage = HistoryStorage::new().await.unwrap();
        
        let entry = HistoryEntry::new(
            "https://example.com".to_string(),
            "Example".to_string(),
            false,
        );
        let id = entry.id.clone();

        storage.add_entry(entry).await.unwrap();
        
        let retrieved = storage.get_entry(&id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().url, "https://example.com");
    }

    #[tokio::test]
    async fn test_delete_entry() {
        let storage = HistoryStorage::new().await.unwrap();
        
        let entry = HistoryEntry::new(
            "https://example.com".to_string(),
            "Example".to_string(),
            false,
        );
        let id = entry.id.clone();

        storage.add_entry(entry).await.unwrap();
        storage.delete_entries(vec![id]).await.unwrap();
        
        assert_eq!(storage.count().await, 0);
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(extract_domain("https://example.com/page"), "example.com");
        assert_eq!(extract_domain("http://www.test.org"), "test.org");
        assert_eq!(extract_domain("https://sub.domain.com/path"), "sub.domain.com");
    }
}