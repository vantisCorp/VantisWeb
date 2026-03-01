//! Storage API
//! 
//! Storage implementation:
//! - localStorage
//! - sessionStorage
//! - IndexedDB (placeholder)
//! - Cookie management
//! - Storage events

use anyhow::{Context, Result};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::core::kernel::VantisKernel;

type StorageMap = Arc<RwLock<HashMap<String, StorageEntry>>>;

/// Storage type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StorageType {
    /// Local storage (persists across sessions)
    Local,
    /// Session storage (cleared on session end)
    Session,
}

/// Storage event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageEvent {
    /// Key set
    Set { key: String, value: String },
    /// Key removed
    Removed { key: String },
    /// Storage cleared
    Cleared,
    /// Storage updated
    Updated { key: String, old_value: Option<String>, new_value: String },
}

/// Storage entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageEntry {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Creation timestamp
    pub created_at: i64,
    /// Last modified timestamp
    pub modified_at: i64,
}

impl StorageEntry {
    /// Create a new storage entry
    pub fn new(key: String, value: String) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            key,
            value,
            created_at: now,
            modified_at: now,
        }
    }

    /// Update value
    pub fn update(&mut self, value: String) {
        self.value = value;
        self.modified_at = chrono::Utc::now().timestamp_millis();
    }
}

/// Storage API
pub struct StorageApi {
    kernel: Arc<VantisKernel>,
    /// Local storage
    local_storage: StorageMap,
    /// Session storage
    session_storage: StorageMap,
    /// Storage event listeners
    listeners: Arc<RwLock<Vec<Box<dyn Fn(StorageEvent) + Send + Sync>>>,
    /// Maximum storage size (in bytes)
    max_size: usize,
}

impl StorageApi {
    /// Create a new Storage API
    pub fn new(kernel: Arc<VantisKernel>) -> Self {
        info!("Initializing Storage API...");

        Self {
            kernel,
            local_storage: Arc::new(RwLock::new(HashMap::new())),
            session_storage: Arc::new(RwLock::new(HashMap::new())),
            listeners: Arc::new(RwLock::new(Vec::new())),
            max_size: 5 * 1024 * 1024, // 5 MB
        }
    }

    /// Set a value in storage
    pub async fn set_item(
        &self,
        storage_type: StorageType,
        key: String,
        value: String,
    ) -> Result<()> {
        debug!("Setting item: {} ({:?})", key, storage_type);

        // Check storage size
        self.check_storage_size(storage_type, &key, &value).await?;

        // Get storage
        let storage: StorageMap = match storage_type {
            StorageType::Local => self.local_storage.clone(),
            StorageType::Session => self.session_storage.clone(),
        };

        // Get old value
        let old_value = storage.read().await.get(&key).map(|e| e.value.clone());

        // Set new value
        let mut storage_write: tokio::sync::RwLockWriteGuard<'_, std::collections::HashMap<String, StorageEntry>> = storage.write().await;
        if let Some(entry) = storage_write.get_mut(&key) {
            entry.update(value.clone());
            self.emit_event(StorageEvent::Updated {
                key: key.clone(),
                old_value,
                new_value: value,
            })
            .await;
        } else {
            storage_write.insert(key.clone(), StorageEntry::new(key.clone(), value.clone()));
            self.emit_event(StorageEvent::Set { key, value }).await;
        }

        Ok(())
    }

    /// Get a value from storage
    pub async fn get_item(&self, storage_type: StorageType, key: String) -> Option<String> {
        debug!("Getting item: {} ({:?})", key, storage_type);

        let storage: StorageMap = match storage_type {
            StorageType::Local => self.local_storage.clone(),
            StorageType::Session => self.session_storage.clone(),
        };

        storage.read().await.get(&key).map(|entry| entry.value.clone())
    }

    /// Remove a value from storage
    pub async fn remove_item(&self, storage_type: StorageType, key: String) -> Result<bool> {
        debug!("Removing item: {} ({:?})", key, storage_type);

        let storage: StorageMap = match storage_type {
            StorageType::Local => self.local_storage.clone(),
            StorageType::Session => self.session_storage.clone(),
        };

        let removed = storage.write().await.remove(&key).is_some();

        if removed {
            self.emit_event(StorageEvent::Removed { key }).await;
        }

        Ok(removed)
    }

    /// Clear all values from storage
    pub async fn clear(&self, storage_type: StorageType) -> Result<()> {
        debug!("Clearing storage: {:?}", storage_type);

        let storage: StorageMap = match storage_type {
            StorageType::Local => self.local_storage.clone(),
            StorageType::Session => self.session_storage.clone(),
        };

        storage.write().await.clear();

        self.emit_event(StorageEvent::Cleared).await;

        Ok(())
    }

    /// Get all keys from storage
    pub async fn keys(&self, storage_type: StorageType) -> Vec<String> {
        let storage: StorageMap = match storage_type {
            StorageType::Local => self.local_storage.clone(),
            StorageType::Session => self.session_storage.clone(),
        };

        storage.read().await.keys().cloned().collect()
    }

    /// Get number of items in storage
    pub async fn length(&self, storage_type: StorageType) -> usize {
        let storage: StorageMap = match storage_type {
            StorageType::Local => self.local_storage.clone(),
            StorageType::Session => self.session_storage.clone(),
        };

        storage.read().await.len()
    }

    /// Check if key exists in storage
    pub async fn has_item(&self, storage_type: StorageType, key: String) -> bool {
        let storage: StorageMap = match storage_type {
            StorageType::Local => self.local_storage.clone(),
            StorageType::Session => self.session_storage.clone(),
        };

        storage.read().await.contains_key(&key)
    }

    /// Get storage size in bytes
    pub async fn size(&self, storage_type: StorageType) -> usize {
        let storage: StorageMap = match storage_type {
            StorageType::Local => self.local_storage.clone(),
            StorageType::Session => self.session_storage.clone(),
        };

        storage
            .read()
            .await
            .values()
            .map(|entry| entry.key.len() + entry.value.len())
            .sum()
    }

    /// Check storage size limit
    async fn check_storage_size(
        &self,
        storage_type: StorageType,
        key: &str,
        value: &str,
    ) -> Result<()> {
        let current_size = self.size(storage_type).await;
        let new_size = current_size + key.len() + value.len();

        if new_size > self.max_size {
            return Err(anyhow::anyhow!(
                "Storage quota exceeded. Maximum size: {} bytes",
                self.max_size
            ));
        }

        Ok(())
    }

    /// Add storage event listener
    pub async fn add_listener<F>(&self, listener: F)
    where
        F: Fn(StorageEvent) + Send + Sync + 'static,
    {
        self.listeners.write().await.push(Box::new(listener));
    }

    /// Remove all listeners
    pub async fn clear_listeners(&self) {
        self.listeners.write().await.clear();
    }

    /// Emit storage event
    async fn emit_event(&self, event: StorageEvent) {
        let listeners = self.listeners.read().await;
        for listener in listeners.iter() {
            listener(event.clone());
        }
    }

    /// Clear session storage (called on session end)
    pub async fn clear_session(&self) -> Result<()> {
        info!("Clearing session storage");
        self.clear(StorageType::Session).await
    }
}

/// Cookie
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookie {
    /// Cookie name
    pub name: String,
    /// Cookie value
    pub value: String,
    /// Domain
    pub domain: Option<String>,
    /// Path
    pub path: Option<String>,
    /// Expiration timestamp (None = session cookie)
    pub expires: Option<i64>,
    /// Secure flag
    pub secure: bool,
    /// HttpOnly flag
    pub http_only: bool,
    /// SameSite attribute
    pub same_site: Option<String>,
}

impl Cookie {
    /// Create a new cookie
    pub fn new(name: String, value: String) -> Self {
        Self {
            name,
            value,
            domain: None,
            path: Some("/".to_string()),
            expires: None,
            secure: false,
            http_only: false,
            same_site: None,
        }
    }

    /// Check if cookie is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires {
            expires < chrono::Utc::now().timestamp()
        } else {
            false
        }
    }

    /// Set domain
    pub fn with_domain(mut self, domain: String) -> Self {
        self.domain = Some(domain);
        self
    }

    /// Set path
    pub fn with_path(mut self, path: String) -> Self {
        self.path = Some(path);
        self
    }

    /// Set expiration (in seconds from now)
    pub fn with_expires(mut self, seconds: i64) -> Self {
        self.expires = Some(chrono::Utc::now().timestamp() + seconds);
        self
    }

    /// Set secure flag
    pub fn with_secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    /// Set http_only flag
    pub fn with_http_only(mut self, http_only: bool) -> Self {
        self.http_only = http_only;
        self
    }

    /// Set same_site attribute
    pub fn with_same_site(mut self, same_site: String) -> Self {
        self.same_site = Some(same_site);
        self
    }
}

/// Cookie Manager
pub struct CookieManager {
    kernel: Arc<VantisKernel>,
    /// Cookies storage
    cookies: Arc<RwLock<HashMap<String, Cookie>>>,
}

impl CookieManager {
    /// Create a new cookie manager
    pub fn new(kernel: Arc<VantisKernel>) -> Self {
        info!("Initializing Cookie Manager...");

        Self {
            kernel,
            cookies: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set a cookie
    pub async fn set_cookie(&self, cookie: Cookie) -> Result<()> {
        debug!("Setting cookie: {}", cookie.name);

        let key = format!("{}:{}", cookie.domain.clone().unwrap_or_else(|| "*".to_string()), cookie.name);
        self.cookies.write().await.insert(key, cookie);

        Ok(())
    }

    /// Get a cookie
    pub async fn get_cookie(&self, name: String, domain: Option<String>) -> Option<Cookie> {
        let key = format!("{}:{}", domain.unwrap_or_else(|| "*".to_string()), name);
        self.cookies.read().await.get(&key).cloned()
    }

    /// Remove a cookie
    pub async fn remove_cookie(&self, name: String, domain: Option<String>) -> Result<bool> {
        debug!("Removing cookie: {}", name);

        let key = format!("{}:{}", domain.unwrap_or_else(|| "*".to_string()), name);
        let removed = self.cookies.write().await.remove(&key).is_some();

        Ok(removed)
    }

    /// Get all cookies for a domain
    pub async fn get_cookies_for_domain(&self, domain: String) -> Vec<Cookie> {
        self.cookies
            .read()
            .await
            .values()
            .filter(|cookie| {
                if let Some(cookie_domain) = &cookie.domain {
                    domain.ends_with(cookie_domain) || cookie_domain == "*"
                } else {
                    true
                }
            })
            .filter(|cookie| !cookie.is_expired())
            .cloned()
            .collect()
    }

    /// Clear all cookies
    pub async fn clear_all(&self) -> Result<()> {
        info!("Clearing all cookies");
        self.cookies.write().await.clear();
        Ok(())
    }

    /// Clear expired cookies
    pub async fn clear_expired(&self) -> Result<()> {
        debug!("Clearing expired cookies");

        let mut cookies = self.cookies.write().await;
        cookies.retain(|_, cookie| !cookie.is_expired());

        Ok(())
    }

    /// Get cookie count
    pub async fn count(&self) -> usize {
        self.cookies.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_storage_api() -> StorageApi {
        let kernel = Arc::new(VantisKernel::new().await.unwrap());
        StorageApi::new(kernel)
    }

    async fn create_cookie_manager() -> CookieManager {
        let kernel = Arc::new(VantisKernel::new().await.unwrap());
        CookieManager::new(kernel)
    }

    #[tokio::test]
    async fn test_storage_api_creation() {
        let storage_api = create_storage_api().await;
        assert_eq!(storage_api.length(StorageType::Local).await, 0);
    }

    #[tokio::test]
    async fn test_set_get_item() {
        let storage_api = create_storage_api().await;

        storage_api
            .set_item(StorageType::Local, "key1".to_string(), "value1".to_string())
            .await
            .unwrap();

        let value = storage_api.get_item(StorageType::Local, "key1".to_string()).await;
        assert_eq!(value, Some("value1".to_string()));
    }

    #[tokio::test]
    async fn test_remove_item() {
        let storage_api = create_storage_api().await;

        storage_api
            .set_item(StorageType::Local, "key1".to_string(), "value1".to_string())
            .await
            .unwrap();

        let removed = storage_api
            .remove_item(StorageType::Local, "key1".to_string())
            .await
            .unwrap();

        assert!(removed);
        assert_eq!(
            storage_api.get_item(StorageType::Local, "key1".to_string()).await,
            None
        );
    }

    #[tokio::test]
    async fn test_clear_storage() {
        let storage_api = create_storage_api().await;

        storage_api
            .set_item(StorageType::Local, "key1".to_string(), "value1".to_string())
            .await
            .unwrap();

        storage_api
            .set_item(StorageType::Local, "key2".to_string(), "value2".to_string())
            .await
            .unwrap();

        storage_api.clear(StorageType::Local).await.unwrap();

        assert_eq!(storage_api.length(StorageType::Local).await, 0);
    }

    #[tokio::test]
    async fn test_session_storage() {
        let storage_api = create_storage_api().await;

        storage_api
            .set_item(StorageType::Session, "key1".to_string(), "value1".to_string())
            .await
            .unwrap();

        let value = storage_api.get_item(StorageType::Session, "key1".to_string()).await;
        assert_eq!(value, Some("value1".to_string()));

        // Session storage should be separate from local storage
        let local_value = storage_api.get_item(StorageType::Local, "key1".to_string()).await;
        assert_eq!(local_value, None);
    }

    #[tokio::test]
    async fn test_storage_keys() {
        let storage_api = create_storage_api().await;

        storage_api
            .set_item(StorageType::Local, "key1".to_string(), "value1".to_string())
            .await
            .unwrap();

        storage_api
            .set_item(StorageType::Local, "key2".to_string(), "value2".to_string())
            .await
            .unwrap();

        let keys = storage_api.keys(StorageType::Local).await;
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
    }

    #[tokio::test]
    async fn test_storage_event_listener() {
        let storage_api = create_storage_api().await;
        let events = Arc::new(RwLock::new(Vec::new()));
        let events_clone = events.clone();

        storage_api
            .add_listener(move |event| {
                let mut events = events_clone.blocking_write();
                events.push(event);
            })
            .await;

        storage_api
            .set_item(StorageType::Local, "key1".to_string(), "value1".to_string())
            .await
            .unwrap();

        let events = events.read().await;
        assert!(events.len() > 0);
    }

    #[tokio::test]
    async fn test_cookie_creation() {
        let cookie = Cookie::new("test".to_string(), "value".to_string());
        assert_eq!(cookie.name, "test");
        assert_eq!(cookie.value, "value");
    }

    #[tokio::test]
    async fn test_cookie_manager() {
        let cookie_manager = create_cookie_manager().await;

        let cookie = Cookie::new("test".to_string(), "value".to_string())
            .with_domain("example.com".to_string());

        cookie_manager.set_cookie(cookie).await.unwrap();

        let retrieved = cookie_manager
            .get_cookie("test".to_string(), Some("example.com".to_string()))
            .await;

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().value, "value");
    }

    #[tokio::test]
    async fn test_cookie_expiration() {
        let cookie = Cookie::new("test".to_string(), "value".to_string())
            .with_expires(-1); // Expired

        assert!(cookie.is_expired());
    }

    #[tokio::test]
    async fn test_cookie_clear_expired() {
        let cookie_manager = create_cookie_manager().await;

        let expired_cookie = Cookie::new("expired".to_string(), "value".to_string())
            .with_expires(-1);

        let valid_cookie = Cookie::new("valid".to_string(), "value".to_string())
            .with_expires(3600);

        cookie_manager.set_cookie(expired_cookie).await.unwrap();
        cookie_manager.set_cookie(valid_cookie).await.unwrap();

        cookie_manager.clear_expired().await.unwrap();

        assert_eq!(cookie_manager.count().await, 1);
    }
}