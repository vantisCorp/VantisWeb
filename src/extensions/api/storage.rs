//! Storage API
//!
//! Provides storage APIs for extensions.

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Storage API
pub struct StorageAPI {
    /// Extension ID
    extension_id: String,
}

impl StorageAPI {
    /// Creates a new storage API
    pub fn new(extension_id: String) -> Self {
        Self { extension_id }
    }

    /// Gets a value from local storage
    pub fn get_local(&self, key: &str) -> Result<Option<StorageValue>> {
        // TODO: Implement local storage retrieval
        Ok(None)
    }

    /// Sets a value in local storage
    pub fn set_local(&self, key: &str, value: StorageValue) -> Result<()> {
        // TODO: Implement local storage setting
        Ok(())
    }

    /// Removes a value from local storage
    pub fn remove_local(&self, key: &str) -> Result<()> {
        // TODO: Implement local storage removal
        Ok(())
    }

    /// Clears all local storage
    pub fn clear_local(&self) -> Result<()> {
        // TODO: Implement local storage clearing
        Ok(())
    }

    /// Gets all local storage
    pub fn get_all_local(&self) -> Result<std::collections::HashMap<String, StorageValue>> {
        // TODO: Implement getting all local storage
        Ok(std::collections::HashMap::new())
    }

    /// Gets a value from sync storage
    pub fn get_sync(&self, key: &str) -> Result<Option<StorageValue>> {
        // TODO: Implement sync storage retrieval
        Ok(None)
    }

    /// Sets a value in sync storage
    pub fn set_sync(&self, key: &str, value: StorageValue) -> Result<()> {
        // TODO: Implement sync storage setting
        Ok(())
    }

    /// Removes a value from sync storage
    pub fn remove_sync(&self, key: &str) -> Result<()> {
        // TODO: Implement sync storage removal
        Ok(())
    }

    /// Clears all sync storage
    pub fn clear_sync(&self) -> Result<()> {
        // TODO: Implement sync storage clearing
        Ok(())
    }

    /// Gets all sync storage
    pub fn get_all_sync(&self) -> Result<std::collections::HashMap<String, StorageValue>> {
        // TODO: Implement getting all sync storage
        Ok(std::collections::HashMap::new())
    }
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