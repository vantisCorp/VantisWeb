/// # Browser History Module
/// 
/// This module provides browser history tracking, visualization, and analytics.
/// It enables users to view their browsing patterns, search through history,
/// and gain insights into their web usage.
//!
//! ## Features
//!
//! - **History Tracking**: Automatic logging of visited pages
//! - **Search & Filter**: Find specific pages in history
//! - **Visualization**: Timeline and chart-based views
//! - **Analytics**: Usage statistics and patterns
//! - **Export**: Export history data
//! - **Privacy**: Private browsing and deletion controls

pub mod storage;
pub mod analytics;
pub mod visualization;
pub mod search;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur in history operations
#[derive(Error, Debug)]
pub enum HistoryError {
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Invalid query: {0}")]
    InvalidQuery(String),
    #[error("Export failed: {0}")]
    ExportFailed(String),
    #[error("Import failed: {0}")]
    ImportFailed(String),
    #[error("Access denied: {0}")]
    AccessDenied(String),
}

/// Result type for history operations
pub type Result<T> = std::result::Result<T, HistoryError>;

/// History entry representing a visited page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Unique identifier
    pub id: String,
    /// Page URL
    pub url: String,
    /// Page title
    pub title: String,
    /// Visit timestamp
    pub timestamp: DateTime<Utc>,
    /// Number of visits
    pub visit_count: u32,
    /// Last visit timestamp
    pub last_visit: DateTime<Utc>,
    /// Favicon URL
    pub favicon: Option<String>,
    /// Referring URL
    pub referrer: Option<String>,
    /// Page category
    pub category: Option<String>,
    /// Tags
    pub tags: Vec<String>,
    /// Whether the page was visited in private mode
    pub is_private: bool,
}

impl HistoryEntry {
    /// Create a new history entry
    pub fn new(url: String, title: String, is_private: bool) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            url,
            title,
            timestamp: now,
            visit_count: 1,
            last_visit: now,
            favicon: None,
            referrer: None,
            category: None,
            tags: Vec::new(),
            is_private,
        }
    }

    /// Increment visit count
    pub fn increment_visit(&mut self) {
        self.visit_count += 1;
        self.last_visit = Utc::now();
    }
}

/// History query for filtering and searching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryQuery {
    /// Search term (matches title or URL)
    pub search: Option<String>,
    /// Start date filter
    pub start_date: Option<DateTime<Utc>>,
    /// End date filter
    pub end_date: Option<DateTime<Utc>>,
    /// Category filter
    pub category: Option<String>,
    /// Tags filter
    pub tags: Option<Vec<String>>,
    /// Maximum results
    pub limit: Option<usize>,
    /// Sort order
    pub sort: SortOrder,
    /// Include private entries
    pub include_private: bool,
}

impl Default for HistoryQuery {
    fn default() -> Self {
        Self {
            search: None,
            start_date: None,
            end_date: None,
            category: None,
            tags: None,
            limit: None,
            sort: SortOrder::NewestFirst,
            include_private: false,
        }
    }
}

/// Sort order for history results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortOrder {
    /// Sort by newest first
    NewestFirst,
    /// Sort by oldest first
    OldestFirst,
    /// Sort by most visits
    MostVisited,
    /// Sort by least visited
    LeastVisited,
    /// Sort alphabetically by title
    TitleAsc,
    /// Sort alphabetically by title (descending)
    TitleDesc,
}

/// History statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryStatistics {
    /// Total number of entries
    pub total_entries: usize,
    /// Total visits
    pub total_visits: u64,
    /// Unique domains
    pub unique_domains: usize,
    /// Most visited pages
    pub most_visited: Vec<HistoryEntry>,
    /// Category distribution
    pub category_distribution: HashMap<String, usize>,
    /// Daily visit count
    pub daily_visits: HashMap<String, usize>,
    /// Weekly visit count
    pub weekly_visits: HashMap<String, usize>,
    /// Monthly visit count
    pub monthly_visits: HashMap<String, usize>,
}

/// History manager for managing browser history
pub struct HistoryManager {
    /// Storage backend
    storage: Arc<RwLock<storage::HistoryStorage>>,
    /// Analytics engine
    analytics: Arc<RwLock<analytics::HistoryAnalytics>>,
    /// Visualization engine
    visualization: Arc<RwLock<visualization::HistoryVisualization>>,
    /// Search engine
    search: Arc<RwLock<search::HistorySearch>>,
}

impl HistoryManager {
    /// Create a new history manager
    pub async fn new() -> Result<Self> {
        let storage = Arc::new(RwLock::new(storage::HistoryStorage::new().await?));
        let analytics = Arc::new(RwLock::new(analytics::HistoryAnalytics::new()));
        let visualization = Arc::new(RwLock::new(visualization::HistoryVisualization::new()));
        let search = Arc::new(RwLock::new(search::HistorySearch::new()));

        Ok(Self {
            storage,
            analytics,
            visualization,
            search,
        })
    }

    /// Add a history entry
    pub async fn add_entry(&self, entry: HistoryEntry) -> Result<()> {
        self.storage.write().await.add_entry(entry).await
    }

    /// Get history entries matching a query
    pub async fn query(&self, query: HistoryQuery) -> Result<Vec<HistoryEntry>> {
        self.search.read().await.query(&self.storage.read().await, query).await
    }

    /// Search history
    pub async fn search(&self, term: &str, limit: usize) -> Result<Vec<HistoryEntry>> {
        let query = HistoryQuery {
            search: Some(term.to_string()),
            limit: Some(limit),
            ..Default::default()
        };
        self.query(query).await
    }

    /// Get history statistics
    pub async fn get_statistics(&self) -> Result<HistoryStatistics> {
        let storage = self.storage.read().await;
        let entries = storage.get_all_entries().await?;
        self.analytics.read().await.calculate_statistics(&entries).await
    }

    /// Get visualization data
    pub async fn get_visualization(&self, viz_type: visualization::VisualizationType) -> Result<visualization::VisualizationData> {
        let storage = self.storage.read().await;
        let entries = storage.get_all_entries().await?;
        self.visualization.read().await.generate(viz_type, &entries).await
    }

    /// Delete history entries
    pub async fn delete_entries(&self, ids: Vec<String>) -> Result<()> {
        self.storage.write().await.delete_entries(ids).await
    }

    /// Clear all history
    pub async fn clear_all(&self) -> Result<()> {
        self.storage.write().await.clear_all().await
    }

    /// Clear history by date range
    pub async fn clear_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<()> {
        self.storage.write().await.clear_by_date_range(start, end).await
    }

    /// Export history
    pub async fn export(&self, format: ExportFormat) -> Result<Vec<u8>> {
        let storage = self.storage.read().await;
        let entries = storage.get_all_entries().await?;
        
        match format {
            ExportFormat::Json => {
                serde_json::to_vec_pretty(&entries)
                    .map_err(|e| HistoryError::ExportFailed(e.to_string()))
            }
            ExportFormat::Csv => {
                let mut csv = String::new();
                csv.push_str("id,url,title,timestamp,visit_count,last_visit,is_private\n");
                for entry in entries {
                    csv.push_str(&format!(
                        "{},{},{},{},{},{},{}\n",
                        entry.id,
                        entry.url,
                        entry.title.replace(',', ';'),
                        entry.timestamp,
                        entry.visit_count,
                        entry.last_visit,
                        entry.is_private
                    ));
                }
                Ok(csv.into_bytes())
            }
        }
    }

    /// Import history
    pub async fn import(&self, data: Vec<u8>, format: ExportFormat) -> Result<()> {
        let entries: Vec<HistoryEntry> = match format {
            ExportFormat::Json => {
                serde_json::from_slice(&data)
                    .map_err(|e| HistoryError::ImportFailed(e.to_string()))?
            }
            ExportFormat::Csv => {
                return Err(HistoryError::ImportFailed("CSV import not yet implemented".to_string()));
            }
        };

        for entry in entries {
            self.storage.write().await.add_entry(entry).await?;
        }

        Ok(())
    }
}

impl Default for HistoryManager {
    async fn default() -> Self {
        Self::new().await.unwrap()
    }
}

/// Export format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_entry_creation() {
        let entry = HistoryEntry::new(
            "https://example.com".to_string(),
            "Example".to_string(),
            false,
        );

        assert_eq!(entry.url, "https://example.com");
        assert_eq!(entry.visit_count, 1);
        assert!(!entry.is_private);
    }

    #[test]
    fn test_history_entry_increment() {
        let mut entry = HistoryEntry::new(
            "https://example.com".to_string(),
            "Example".to_string(),
            false,
        );

        entry.increment_visit();
        assert_eq!(entry.visit_count, 2);
    }

    #[test]
    fn test_query_default() {
        let query = HistoryQuery::default();
        assert!(query.search.is_none());
        assert_eq!(query.sort, SortOrder::NewestFirst);
        assert!(!query.include_private);
    }
}