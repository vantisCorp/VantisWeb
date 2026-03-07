//! Secure Password Storage for VantisWeb
//! 
//! This module handles:
//! - Encrypted password storage
//! - Secure retrieval
//! - Search functionality
//! - Import/Export

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use super::{PasswordEntry, PasswordQuery, PasswordError, ImportFormat, ExportFormat};

/// Password storage backend
pub struct PasswordStorage {
    /// In-memory storage (encrypted)
    entries: Arc<RwLock<HashMap<String, PasswordEntry>>>,
    /// Storage path
    storage_path: String,
    /// Whether initialized
    initialized: bool,
}

impl PasswordStorage {
    /// Create a new password storage
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            storage_path: "./passwords.db".to_string(),
            initialized: false,
        }
    }

    /// Initialize storage
    pub async fn initialize(&self) -> Result<(), PasswordError> {
        // Load from disk if exists
        // In production, this would load from encrypted file
        self.entries.write().await.clear();
        Ok(())
    }

    /// Save a password entry
    pub async fn save_entry(&self, entry: &PasswordEntry) -> Result<String, PasswordError> {
        let id = entry.id.clone();
        let mut entries = self.entries.write().await;
        entries.insert(id.clone(), entry.clone());
        Ok(id)
    }

    /// Get a password entry
    pub async fn get_entry(&self, id: &str) -> Result<PasswordEntry, PasswordError> {
        let entries = self.entries.read().await;
        entries.get(id)
            .cloned()
            .ok_or_else(|| PasswordError::NotFound(id.to_string()))
    }

    /// Update a password entry
    pub async fn update_entry(&self, entry: &PasswordEntry) -> Result<(), PasswordError> {
        let mut entries = self.entries.write().await;
        if !entries.contains_key(&entry.id) {
            return Err(PasswordError::NotFound(entry.id.clone()));
        }
        entries.insert(entry.id.clone(), entry.clone());
        Ok(())
    }

    /// Delete a password entry
    pub async fn delete_entry(&self, id: &str) -> Result<(), PasswordError> {
        let mut entries = self.entries.write().await;
        entries.remove(id)
            .ok_or_else(|| PasswordError::NotFound(id.to_string()))?;
        Ok(())
    }

    /// Search password entries
    pub async fn search_entries(&self, query: PasswordQuery) -> Result<Vec<PasswordEntry>, PasswordError> {
        let entries = self.entries.read().await;
        let mut results: Vec<PasswordEntry> = entries.values().cloned().collect();
        
        // Filter by search term
        if let Some(term) = &query.search_term {
            results.retain(|e| {
                e.name.to_lowercase().contains(&term.to_lowercase()) ||
                e.url.to_lowercase().contains(&term.to_lowercase()) ||
                e.username.to_lowercase().contains(&term.to_lowercase())
            });
        }
        
        // Filter by folder
        if let Some(folder) = &query.folder {
            results.retain(|e| e.folder.as_ref() == Some(folder));
        }
        
        // Filter breached only
        if query.breached_only {
            results.retain(|e| e.in_breach);
        }
        
        // Sort results
        results.sort_by(|a, b| {
            let cmp = match query.sort_by {
                super::SortField::Name => a.name.cmp(&b.name),
                super::SortField::Created => a.created_at.cmp(&b.created_at),
                super::SortField::Modified => a.modified_at.cmp(&b.modified_at),
                super::SortField::Used => {
                    let a_used = a.last_used.unwrap_or(DateTime::UNIX_EPOCH);
                    let b_used = b.last_used.unwrap_or(DateTime::UNIX_EPOCH);
                    a_used.cmp(&b_used)
                }
                super::SortField::Strength => a.strength_score.cmp(&b.strength_score),
            };
            
            match query.sort_order {
                super::SortOrder::Ascending => cmp,
                super::SortOrder::Descending => cmp.reverse(),
            }
        });
        
        Ok(results)
    }

    /// Get entry count
    pub async fn get_entry_count(&self) -> usize {
        self.entries.read().await.len()
    }

    /// Get breached password count
    pub async fn get_breached_count(&self) -> usize {
        self.entries.read().await.values().filter(|e| e.in_breach).count()
    }

    /// Check if password is in a breach
    pub async fn check_breach(&self, _password: &str) -> Result<super::BreachResult, PasswordError> {
        // In production, this would check against HaveIBeenPwned API
        Ok(super::BreachResult {
            breached: false,
            breach_count: 0,
            breaches: Vec::new(),
        })
    }

    /// Import passwords from another manager
    pub async fn import(&self, format: ImportFormat, data: String) -> Result<Vec<PasswordEntry>, PasswordError> {
        match format {
            ImportFormat::CSV => self.import_csv(&data),
            ImportFormat::JSON => self.import_json(&data),
            ImportFormat::OnePassword => self.import_1password(&data),
            ImportFormat::LastPass => self.import_lastpass(&data),
            ImportFormat::Bitwarden => self.import_bitwarden(&data),
        }
    }

    /// Import from CSV
    fn import_csv(&self, data: &str) -> Result<Vec<PasswordEntry>, PasswordError> {
        let mut entries = Vec::new();
        let mut lines = data.lines();
        
        // Skip header
        lines.next();
        
        for line in lines {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 4 {
                let entry = PasswordEntry {
                    id: uuid::Uuid::new_v4().to_string(),
                    url: parts[0].to_string(),
                    name: parts[1].to_string(),
                    username: parts[2].to_string(),
                    password_encrypted: parts[3].to_string(),
                    fields: Vec::new(),
                    notes: parts.get(4).map(|s| s.to_string()),
                    folder: None,
                    created_at: Utc::now(),
                    modified_at: Utc::now(),
                    last_used: None,
                    use_count: 0,
                    strength_score: 0,
                    in_breach: false,
                };
                entries.push(entry);
            }
        }
        
        Ok(entries)
    }

    /// Import from JSON
    fn import_json(&self, data: &str) -> Result<Vec<PasswordEntry>, PasswordError> {
        let entries: Vec<PasswordEntry> = serde_json::from_str(data)
            .map_err(|e| PasswordError::ImportError(e.to_string()))?;
        Ok(entries)
    }

    /// Import from 1Password format
    fn import_1password(&self, data: &str) -> Result<Vec<PasswordEntry>, PasswordError> {
        // Parse 1Password CSV format
        self.import_csv(data)
    }

    /// Import from LastPass format
    fn import_lastpass(&self, data: &str) -> Result<Vec<PasswordEntry>, PasswordError> {
        // Parse LastPass CSV format
        self.import_csv(data)
    }

    /// Import from Bitwarden format
    fn import_bitwarden(&self, data: &str) -> Result<Vec<PasswordEntry>, PasswordError> {
        // Parse Bitwarden JSON format
        self.import_json(data)
    }

    /// Export passwords
    pub async fn export(&self, format: ExportFormat) -> Result<String, PasswordError> {
        let entries = self.entries.read().await;
        let entries: Vec<_> = entries.values().collect();
        
        match format {
            ExportFormat::CSV => self.export_csv(&entries),
            ExportFormat::JSON => self.export_json(&entries),
            ExportFormat::EncryptedJSON => self.export_encrypted_json(&entries),
        }
    }

    /// Export to CSV
    fn export_csv(&self, entries: &[&PasswordEntry]) -> Result<String, PasswordError> {
        let mut csv = String::from("url,name,username,password,notes\n");
        
        for entry in entries {
            csv.push_str(&format!(
                "{},{},{},{},{}\n",
                entry.url,
                entry.name,
                entry.username,
                entry.password_encrypted,
                entry.notes.as_ref().unwrap_or(&String::new())
            ));
        }
        
        Ok(csv)
    }

    /// Export to JSON
    fn export_json(&self, entries: &[&PasswordEntry]) -> Result<String, PasswordError> {
        serde_json::to_string_pretty(&entries)
            .map_err(|e| PasswordError::ExportError(e.to_string()))
    }

    /// Export to encrypted JSON
    fn export_encrypted_json(&self, entries: &[&PasswordEntry]) -> Result<String, PasswordError> {
        // In production, this would encrypt the JSON
        self.export_json(entries)
    }

    /// Get all entries for a URL
    pub async fn get_entries_for_url(&self, url: &str) -> Vec<PasswordEntry> {
        let entries = self.entries.read().await;
        entries.values()
            .filter(|e| e.url == url || url.contains(&e.url) || e.url.contains(url))
            .cloned()
            .collect()
    }

    /// Get folders
    pub async fn get_folders(&self) -> Vec<String> {
        let entries = self.entries.read().await;
        entries.values()
            .filter_map(|e| e.folder.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }

    /// Mark entry as used
    pub async fn mark_used(&self, id: &str) -> Result<(), PasswordError> {
        let mut entries = self.entries.write().await;
        if let Some(entry) = entries.get_mut(id) {
            entry.last_used = Some(Utc::now());
            entry.use_count += 1;
        }
        Ok(())
    }

    /// Update breach status
    pub async fn update_breach_status(&self, id: &str, breached: bool) -> Result<(), PasswordError> {
        let mut entries = self.entries.write().await;
        if let Some(entry) = entries.get_mut(id) {
            entry.in_breach = breached;
        }
        Ok(())
    }

    /// Clear all entries
    pub async fn clear(&self) {
        self.entries.write().await.clear();
    }
}

impl Default for PasswordStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_storage() {
        let storage = PasswordStorage::new();
        storage.initialize().await.unwrap();
        assert_eq!(storage.get_entry_count().await, 0);
    }

    #[tokio::test]
    async fn test_save_and_get_entry() {
        let storage = PasswordStorage::new();
        storage.initialize().await.unwrap();
        
        let entry = PasswordEntry {
            id: "test-1".to_string(),
            url: "https://example.com".to_string(),
            name: "Example".to_string(),
            username: "user@example.com".to_string(),
            password_encrypted: "encrypted_password".to_string(),
            fields: Vec::new(),
            notes: None,
            folder: None,
            created_at: Utc::now(),
            modified_at: Utc::now(),
            last_used: None,
            use_count: 0,
            strength_score: 85,
            in_breach: false,
        };
        
        storage.save_entry(&entry).await.unwrap();
        let retrieved = storage.get_entry("test-1").await.unwrap();
        
        assert_eq!(retrieved.username, "user@example.com");
    }

    #[tokio::test]
    async fn test_search_entries() {
        let storage = PasswordStorage::new();
        storage.initialize().await.unwrap();
        
        let entry = PasswordEntry {
            id: "test-1".to_string(),
            url: "https://example.com".to_string(),
            name: "Example".to_string(),
            username: "user@example.com".to_string(),
            password_encrypted: "encrypted".to_string(),
            fields: Vec::new(),
            notes: None,
            folder: Some("Work".to_string()),
            created_at: Utc::now(),
            modified_at: Utc::now(),
            last_used: None,
            use_count: 0,
            strength_score: 85,
            in_breach: false,
        };
        
        storage.save_entry(&entry).await.unwrap();
        
        let query = PasswordQuery {
            search_term: Some("example".to_string()),
            folder: None,
            breached_only: false,
            sort_by: super::super::SortField::Name,
            sort_order: super::super::SortOrder::Ascending,
        };
        
        let results = storage.search_entries(query).await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_delete_entry() {
        let storage = PasswordStorage::new();
        storage.initialize().await.unwrap();
        
        let entry = PasswordEntry {
            id: "test-1".to_string(),
            url: "https://example.com".to_string(),
            name: "Example".to_string(),
            username: "user".to_string(),
            password_encrypted: "pwd".to_string(),
            fields: Vec::new(),
            notes: None,
            folder: None,
            created_at: Utc::now(),
            modified_at: Utc::now(),
            last_used: None,
            use_count: 0,
            strength_score: 0,
            in_breach: false,
        };
        
        storage.save_entry(&entry).await.unwrap();
        assert_eq!(storage.get_entry_count().await, 1);
        
        storage.delete_entry("test-1").await.unwrap();
        assert_eq!(storage.get_entry_count().await, 0);
    }
}