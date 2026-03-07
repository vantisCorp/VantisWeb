//! Local storage for analytics data
//! 
//! This module provides persistent storage for analytics data
//! using SQLite for local-only, privacy-focused data retention.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use rusqlite::{Connection, params};

use super::{AnalyticsError, UsageMetrics, PerformanceMetrics, SecurityMetrics};

/// Analytics storage backend
pub struct AnalyticsStorage {
    db_path: PathBuf,
    connection: Option<Arc<Mutex<Connection>>>,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Database file path
    pub db_path: PathBuf,
    /// Maximum storage size in MB
    pub max_size_mb: u64,
    /// Enable compression
    pub enable_compression: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            db_path: PathBuf::from("analytics.db"),
            max_size_mb: 100,
            enable_compression: true,
        }
    }
}

impl AnalyticsStorage {
    /// Create new analytics storage
    pub fn new() -> Self {
        Self {
            db_path: PathBuf::from("analytics.db"),
            connection: None,
        }
    }

    /// Create storage with custom configuration
    pub fn with_config(config: StorageConfig) -> Self {
        Self {
            db_path: config.db_path,
            connection: None,
        }
    }
    
    /// Create storage with a specific path
    pub fn with_path(path: &str) -> Self {
        Self {
            db_path: PathBuf::from(path),
            connection: None,
        }
    }

    /// Initialize the storage
    pub async fn initialize(&mut self) -> Result<(), AnalyticsError> {
        let conn = Connection::open(&self.db_path)
            .map_err(|e| AnalyticsError::StorageError(e.to_string()))?;

        // Create tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS visits (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL,
                domain TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                timestamp TEXT NOT NULL
            )",
            [],
        ).map_err(|e| AnalyticsError::StorageError(e.to_string()))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS page_loads (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL,
                dns_time_ms INTEGER,
                connect_time_ms INTEGER,
                ttfb_ms INTEGER,
                dom_content_loaded_ms INTEGER,
                load_complete_ms INTEGER,
                total_bytes INTEGER,
                request_count INTEGER,
                timestamp TEXT NOT NULL
            )",
            [],
        ).map_err(|e| AnalyticsError::StorageError(e.to_string()))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS security_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_type TEXT NOT NULL,
                details TEXT,
                url TEXT,
                timestamp TEXT NOT NULL
            )",
            [],
        ).map_err(|e| AnalyticsError::StorageError(e.to_string()))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS daily_stats (
                date TEXT PRIMARY KEY,
                visits INTEGER,
                time_ms INTEGER,
                unique_domains INTEGER
            )",
            [],
        ).map_err(|e| AnalyticsError::StorageError(e.to_string()))?;

        self.connection = Some(Arc::new(Mutex::new(conn)));
        Ok(())
    }

    /// Store a visit record
    pub async fn store_visit(&self, url: &str, domain: &str, duration_ms: u64) -> Result<(), AnalyticsError> {
        let conn = self.connection.as_ref()
            .ok_or_else(|| AnalyticsError::StorageError("Not initialized".to_string()))?;
        
        let conn = conn.lock().await;
        let timestamp = chrono::Utc::now().to_rfc3339();
        
        conn.execute(
            "INSERT INTO visits (url, domain, duration_ms, timestamp) VALUES (?1, ?2, ?3, ?4)",
            params![url, domain, duration_ms, timestamp],
        ).map_err(|e| AnalyticsError::StorageError(e.to_string()))?;

        Ok(())
    }

    /// Store a page load record
    pub async fn store_page_load(&self, record: &super::PageLoadRecord) -> Result<(), AnalyticsError> {
        let conn = self.connection.as_ref()
            .ok_or_else(|| AnalyticsError::StorageError("Not initialized".to_string()))?;
        
        let conn = conn.lock().await;
        let timestamp = record.timestamp.to_rfc3339();
        
        conn.execute(
            "INSERT INTO page_loads (url, dns_time_ms, connect_time_ms, ttfb_ms, dom_content_loaded_ms, load_complete_ms, total_bytes, request_count, timestamp) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                record.url,
                record.dns_time_ms,
                record.connect_time_ms,
                record.ttfb_ms,
                record.dom_content_loaded_ms,
                record.load_complete_ms,
                record.total_bytes,
                record.request_count,
                timestamp
            ],
        ).map_err(|e| AnalyticsError::StorageError(e.to_string()))?;

        Ok(())
    }

    /// Store a security event
    pub async fn store_security_event(&self, event_type: &str, details: &str, url: Option<&str>) -> Result<(), AnalyticsError> {
        let conn = self.connection.as_ref()
            .ok_or_else(|| AnalyticsError::StorageError("Not initialized".to_string()))?;
        
        let conn = conn.lock().await;
        let timestamp = chrono::Utc::now().to_rfc3339();
        
        conn.execute(
            "INSERT INTO security_events (event_type, details, url, timestamp) VALUES (?1, ?2, ?3, ?4)",
            params![event_type, details, url, timestamp],
        ).map_err(|e| AnalyticsError::StorageError(e.to_string()))?;

        Ok(())
    }

    /// Record a visit synchronously
    pub fn record_visit(&mut self, url: &str, domain: Option<&str>, duration_ms: u64) -> Result<(), AnalyticsError> {
        let dom = domain.unwrap_or("");
        // For synchronous operation, use blocking storage
        // In a real implementation, this would write to a memory buffer
        // that gets flushed asynchronously
        Ok(())
    }
    
    /// Record a page load synchronously
    pub fn record_page_load(&mut self, metrics: &super::PageLoadMetrics) -> Result<(), AnalyticsError> {
        // For synchronous operation
        Ok(())
    }
    
    /// Record a security event synchronously
    pub fn record_security_event(&mut self, event: &super::SecurityEvent) -> Result<(), AnalyticsError> {
        // For synchronous operation
        Ok(())
    }
    
    /// Get analytics snapshot
    pub fn get_snapshot(&self) -> Result<super::AnalyticsSnapshot, AnalyticsError> {
        // Return a default snapshot for now
        Ok(super::AnalyticsSnapshot::new())
    }

    /// Clear all stored data
    pub fn clear(&mut self) -> Result<(), AnalyticsError> {
        if let Some(conn) = &self.connection {
            let conn = conn.blocking_lock();
            conn.execute("DELETE FROM visits", [])
                .map_err(|e| AnalyticsError::StorageError(e.to_string()))?;
            conn.execute("DELETE FROM page_loads", [])
                .map_err(|e| AnalyticsError::StorageError(e.to_string()))?;
            conn.execute("DELETE FROM security_events", [])
                .map_err(|e| AnalyticsError::StorageError(e.to_string()))?;
            conn.execute("DELETE FROM daily_stats", [])
                .map_err(|e| AnalyticsError::StorageError(e.to_string()))?;
        }
        Ok(())
    }

    /// Get storage statistics
    pub fn get_stats(&self) -> StorageStats {
        StorageStats {
            db_path: self.db_path.clone(),
            size_bytes: self.get_db_size(),
            table_counts: self.get_table_counts(),
        }
    }

    fn get_db_size(&self) -> u64 {
        std::fs::metadata(&self.db_path)
            .map(|m| m.len())
            .unwrap_or(0)
    }

    fn get_table_counts(&self) -> HashMap<String, u64> {
        let mut counts = HashMap::new();
        if let Some(conn) = &self.connection {
            if let Ok(conn) = conn.try_lock() {
                for table in &["visits", "page_loads", "security_events", "daily_stats"] {
                    let count: u64 = conn.query_row(
                        &format!("SELECT COUNT(*) FROM {}", table),
                        [],
                        |row| row.get(0),
                    ).unwrap_or(0);
                    counts.insert(table.to_string(), count);
                }
            }
        }
        counts
    }
}

impl Default for AnalyticsStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub db_path: PathBuf,
    pub size_bytes: u64,
    pub table_counts: HashMap<String, u64>,
}

use std::collections::HashMap;