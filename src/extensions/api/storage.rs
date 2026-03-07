//! Storage API
//!
//! Provides storage APIs for extensions with local and sync storage support.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Global storage manager for all extensions
static mut EXTENSION_STORAGE: Option<Arc<RwLock<HashMap<String, ExtensionStorage>>>> = None;

fn get_storage() -> Arc<RwLock<HashMap<String, ExtensionStorage>>> {
    unsafe {
        if EXTENSION_STORAGE.is_none() {
            EXTENSION_STORAGE = Some(Arc::new(RwLock::new(HashMap::new())));
        }
        EXTENSION_STORAGE.clone().unwrap()
    }
}

/// Extension-specific storage
#[derive(Debug, Clone, Default)]
struct ExtensionStorage {
    local: HashMap<String, StorageValue>,
    sync: HashMap<String, StorageValue>,
}

/// Storage API
pub struct StorageAPI {
    /// Extension ID
    extension_id: String,
}

impl StorageAPI {
    /// Creates a new storage API
    pub fn new(extension_id: String) -> Self {
        // Initialize storage for this extension
        let storage = get_storage();
        let mut guard = storage.write().unwrap();
        guard.entry(extension_id.clone()).or_insert_with(ExtensionStorage::default);
        
        Self { extension_id }
    }

    /// Gets a value from local storage
    pub fn get_local(&self, key: &str) -> Result<Option<StorageValue>> {
        let storage = get_storage();
        let guard = storage.read().unwrap();
        
        if let Some(ext_storage) = guard.get(&self.extension_id) {
            Ok(ext_storage.local.get(key).cloned())
        } else {
            Ok(None)
        }
    }

    /// Sets a value in local storage
    pub fn set_local(&self, key: &str, value: StorageValue) -> Result<()> {
        let storage = get_storage();
        let mut guard = storage.write().unwrap();
        
        if let Some(ext_storage) = guard.get_mut(&self.extension_id) {
            ext_storage.local.insert(key.to_string(), value);
        }
        
        Ok(())
    }

    /// Removes a value from local storage
    pub fn remove_local(&self, key: &str) -> Result<()> {
        let storage = get_storage();
        let mut guard = storage.write().unwrap();
        
        if let Some(ext_storage) = guard.get_mut(&self.extension_id) {
            ext_storage.local.remove(key);
        }
        
        Ok(())
    }

    /// Clears all local storage
    pub fn clear_local(&self) -> Result<()> {
        let storage = get_storage();
        let mut guard = storage.write().unwrap();
        
        if let Some(ext_storage) = guard.get_mut(&self.extension_id) {
            ext_storage.local.clear();
        }
        
        Ok(())
    }

    /// Gets all local storage
    pub fn get_all_local(&self) -> Result<HashMap<String, StorageValue>> {
        let storage = get_storage();
        let guard = storage.read().unwrap();
        
        if let Some(ext_storage) = guard.get(&self.extension_id) {
            Ok(ext_storage.local.clone())
        } else {
            Ok(HashMap::new())
        }
    }

    /// Gets a value from sync storage
    pub fn get_sync(&self, key: &str) -> Result<Option<StorageValue>> {
        let storage = get_storage();
        let guard = storage.read().unwrap();
        
        if let Some(ext_storage) = guard.get(&self.extension_id) {
            Ok(ext_storage.sync.get(key).cloned())
        } else {
            Ok(None)
        }
    }

    /// Sets a value in sync storage
    pub fn set_sync(&self, key: &str, value: StorageValue) -> Result<()> {
        let storage = get_storage();
        let mut guard = storage.write().unwrap();
        
        if let Some(ext_storage) = guard.get_mut(&self.extension_id) {
            ext_storage.sync.insert(key.to_string(), value);
        }
        
        Ok(())
    }

    /// Removes a value from sync storage
    pub fn remove_sync(&self, key: &str) -> Result<()> {
        let storage = get_storage();
        let mut guard = storage.write().unwrap();
        
        if let Some(ext_storage) = guard.get_mut(&self.extension_id) {
            ext_storage.sync.remove(key);
        }
        
        Ok(())
    }

    /// Clears all sync storage
    pub fn clear_sync(&self) -> Result<()> {
        let storage = get_storage();
        let mut guard = storage.write().unwrap();
        
        if let Some(ext_storage) = guard.get_mut(&self.extension_id) {
            ext_storage.sync.clear();
        }
        
        Ok(())
    }

    /// Gets all sync storage
    pub fn get_all_sync(&self) -> Result<HashMap<String, StorageValue>> {
        let storage = get_storage();
        let guard = storage.read().unwrap();
        
        if let Some(ext_storage) = guard.get(&self.extension_id) {
            Ok(ext_storage.sync.clone())
        } else {
            Ok(HashMap::new())
        }
    }
    
    /// Get storage usage information
    pub fn get_usage(&self) -> Result<StorageUsage> {
        let storage = get_storage();
        let guard = storage.read().unwrap();
        
        if let Some(ext_storage) = guard.get(&self.extension_id) {
            let local_bytes: usize = ext_storage.local.values()
                .map(|v| std::mem::size_of_val(v))
                .sum();
            let sync_bytes: usize = ext_storage.sync.values()
                .map(|v| std::mem::size_of_val(v))
                .sum();
            
            Ok(StorageUsage {
                local_bytes,
                sync_bytes,
                local_items: ext_storage.local.len(),
                sync_items: ext_storage.sync.len(),
            })
        } else {
            Ok(StorageUsage::default())
        }
    }
}

/// Storage usage information
#[derive(Debug, Clone, Default)]
pub struct StorageUsage {
    pub local_bytes: usize,
    pub sync_bytes: usize,
    pub local_items: usize,
    pub sync_items: usize,
}

/// Storage value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StorageValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Object(serde_json::Value),
    Array(Vec<serde_json::Value>),
}

impl From<String> for StorageValue {
    fn from(s: String) -> Self {
        StorageValue::String(s)
    }
}

impl From<&str> for StorageValue {
    fn from(s: &str) -> Self {
        StorageValue::String(s.to_string())
    }
}

impl From<f64> for StorageValue {
    fn from(n: f64) -> Self {
        StorageValue::Number(n)
    }
}

impl From<i32> for StorageValue {
    fn from(n: i32) -> Self {
        StorageValue::Number(n as f64)
    }
}

impl From<bool> for StorageValue {
    fn from(b: bool) -> Self {
        StorageValue::Boolean(b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_api_creation() {
        let api = StorageAPI::new("test-extension".to_string());
        assert_eq!(api.extension_id, "test-extension");
    }

    #[test]
    fn test_local_storage_operations() {
        let api = StorageAPI::new("test-local".to_string());
        
        // Test set and get
        api.set_local("key1", "value1".into()).unwrap();
        let value = api.get_local("key1").unwrap();
        assert!(matches!(value, Some(StorageValue::String(s)) if s == "value1"));
        
        // Test get all
        let all = api.get_all_local().unwrap();
        assert_eq!(all.len(), 1);
        
        // Test remove
        api.remove_local("key1").unwrap();
        let value = api.get_local("key1").unwrap();
        assert!(value.is_none());
    }

    #[test]
    fn test_sync_storage_operations() {
        let api = StorageAPI::new("test-sync".to_string());
        
        // Test set and get
        api.set_sync("key1", 42.into()).unwrap();
        let value = api.get_sync("key1").unwrap();
        assert!(matches!(value, Some(StorageValue::Number(n)) if n == 42.0));
        
        // Test clear
        api.clear_sync().unwrap();
        let all = api.get_all_sync().unwrap();
        assert!(all.is_empty());
    }

    #[test]
    fn test_storage_value_from_string() {
        let value: StorageValue = "test".into();
        assert!(matches!(value, StorageValue::String(_)));
    }

    #[test]
    fn test_storage_value_from_number() {
        let value: StorageValue = 42.into();
        assert!(matches!(value, StorageValue::Number(_)));
    }

    #[test]
    fn test_storage_value_from_boolean() {
        let value: StorageValue = true.into();
        assert!(matches!(value, StorageValue::Boolean(_)));
    }
}