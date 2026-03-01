//! Browsing History Module
//! 
//! History management:
//! - Page navigation history
//! - Search in history
//! - Import/export
//! - Privacy mode support

use anyhow::Result;
use chrono::{DateTime, Utc};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// History entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub url: String,
    pub title: String,
    pub visit_time: DateTime<Utc>,
    pub visit_count: u32,
}

/// History manager
pub struct HistoryManager {
    entries: HashMap<String, HistoryEntry>,
    max_entries: usize,
}

impl HistoryManager {
    /// Create a new history manager
    pub fn new(max_entries: usize) -> Self {
        info!("Initializing History Manager...");
        
        Self {
            entries: HashMap::new(),
            max_entries,
        }
    }
    
    /// Add history entry
    pub fn add_entry(&mut self, url: String, title: String) -> Result<()> {
        let now = Utc::now();
        
        // Check if entry already exists
        if let Some(entry) = self.entries.get(&url) {
            // Update existing entry
            let mut updated_entry = entry.clone();
            updated_entry.visit_time = now;
            updated_entry.visit_count += 1;
            updated_entry.title = title;
            self.entries.insert(url.clone(), updated_entry);
            
            debug!("Updated history entry: {}", url);
        } else {
            // Create new entry
            let entry = HistoryEntry {
                id: uuid::Uuid::new_v4().to_string(),
                url: url.clone(),
                title,
                visit_time: now,
                visit_count: 1,
            };
            
            self.entries.insert(url, entry);
            debug!("Added history entry (total: {})", self.entries.len());
            
            // Enforce max entries limit
            self.enforce_limit();
        }
        
        Ok(())
    }
    
    /// Enforce maximum entries limit
    fn enforce_limit(&mut self) {
        if self.entries.len() > self.max_entries {
            // Remove oldest entries
            let mut entries: Vec<_> = self.entries.values().cloned().collect();
            entries.sort_by(|a, b| a.visit_time.cmp(&b.visit_time));
            
            let to_remove = self.entries.len() - self.max_entries;
            for entry in entries.iter().take(to_remove) {
                self.entries.remove(&entry.url);
            }
            
            info!("Removed {} oldest entries to enforce limit", to_remove);
        }
    }
    
    /// Get all history entries
    pub fn get_all(&self) -> Vec<HistoryEntry> {
        let mut entries: Vec<_> = self.entries.values().cloned().collect();
        entries.sort_by(|a, b| b.visit_time.cmp(&a.visit_time));
        entries
    }
    
    /// Search history
    pub fn search(&self, query: &str) -> Vec<HistoryEntry> {
        let query_lower = query.to_lowercase();
        
        self.entries
            .values()
            .filter(|entry| {
                entry.url.to_lowercase().contains(&query_lower)
                    || entry.title.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect()
    }
    
    /// Clear all history
    pub fn clear(&mut self) {
        info!("Clearing all history");
        self.entries.clear();
    }
    
    /// Remove specific entry
    pub fn remove(&mut self, url: &str) {
        info!("Removing history entry: {}", url);
        self.entries.remove(url);
    }
    
    /// Get entry count
    pub fn count(&self) -> usize {
        self.entries.len()
    }
}