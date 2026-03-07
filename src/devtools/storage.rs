// Copyright 2024 Vantis Corporation - All Rights Reserved

//! Storage Inspector Module
//! 
//! This module provides inspection capabilities for Local Storage, Session Storage,
//! Cookies, IndexedDB, Service Workers, and Cache API.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Storage type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StorageType {
    LocalStorage,
    SessionStorage,
    Cookies,
    IndexedDB,
    ServiceWorkers,
    CacheStorage,
    WebSQL,
}

/// Storage item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageItem {
    /// Item ID
    pub id: Uuid,
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Storage type
    pub storage_type: StorageType,
    /// Size (bytes)
    pub size: u64,
    /// Created at
    pub created_at: Option<DateTime<Utc>>,
    /// Updated at
    pub updated_at: Option<DateTime<Utc>>,
    /// Expires at
    pub expires_at: Option<DateTime<Utc>>,
    /// Origin
    pub origin: String,
}

/// Cookie item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieItem {
    /// Cookie ID
    pub id: Uuid,
    /// Name
    pub name: String,
    /// Value
    pub value: String,
    /// Domain
    pub domain: String,
    /// Path
    pub path: String,
    /// Expires
    pub expires: Option<DateTime<Utc>>,
    /// Max age
    pub max_age: Option<u64>,
    /// Secure
    pub secure: bool,
    /// HttpOnly
    pub http_only: bool,
    /// SameSite
    pub same_site: SameSite,
    /// Size
    pub size: u64,
    /// Priority
    pub priority: CookiePriority,
}

/// SameSite attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SameSite {
    Strict,
    Lax,
    None,
    Unspecified,
}

/// Cookie priority
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CookiePriority {
    Low,
    Medium,
    High,
}

/// IndexedDB database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedDBDatabase {
    /// Database ID
    pub id: Uuid,
    /// Name
    pub name: String,
    /// Version
    pub version: u32,
    /// Object stores
    pub object_stores: Vec<ObjectStore>,
    /// Origin
    pub origin: String,
}

/// Object store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectStore {
    /// Store ID
    pub id: Uuid,
    /// Name
    pub name: String,
    /// Key path
    pub key_path: Option<String>,
    /// Auto increment
    pub auto_increment: bool,
    /// Indexes
    pub indexes: Vec<Index>,
    /// Entry count
    pub entry_count: u64,
}

/// Index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    /// Index ID
    pub id: Uuid,
    /// Name
    pub name: String,
    /// Key path
    pub key_path: String,
    /// Multi-entry
    pub multi_entry: bool,
    /// Unique
    pub unique: bool,
}

/// IndexedDB entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedDBEntry {
    /// Entry ID
    pub id: Uuid,
    /// Key
    pub key: serde_json::Value,
    /// Value
    pub value: serde_json::Value,
    /// Store ID
    pub store_id: Uuid,
}

/// Service worker registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceWorkerRegistration {
    /// Registration ID
    pub id: Uuid,
    /// Scope URL
    pub scope_url: String,
    /// Script URL
    pub script_url: String,
    /// State
    pub state: ServiceWorkerState,
    /// Last updated
    pub last_updated: DateTime<Utc>,
    /// Running status
    pub running_status: ServiceWorkerRunningStatus,
}

/// Service worker state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceWorkerState {
    Installing,
    Installed,
    Activating,
    Activated,
    Redundant,
}

/// Service worker running status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceWorkerRunningStatus {
    Running,
    Starting,
    Stopped,
    Stopping,
}

/// Cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Entry ID
    pub id: Uuid,
    /// Cache name
    pub cache_name: String,
    /// URL
    pub url: String,
    /// Response
    pub response: CacheResponse,
    /// Created at
    pub created_at: DateTime<Utc>,
}

/// Cache response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheResponse {
    /// Status
    pub status: u16,
    /// Status text
    pub status_text: String,
    /// Headers
    pub headers: HashMap<String, String>,
    /// Body size
    pub body_size: u64,
    /// Content type
    pub content_type: Option<String>,
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStatistics {
    /// Local storage size
    pub local_storage_size: u64,
    /// Session storage size
    pub session_storage_size: u64,
    /// Cookie count
    pub cookie_count: u64,
    /// IndexedDB size
    pub indexed_db_size: u64,
    /// Cache storage size
    pub cache_storage_size: u64,
    /// Service worker count
    pub service_worker_count: u64,
    /// Total size
    pub total_size: u64,
    /// Quota
    pub quota: u64,
    /// Usage percentage
    pub usage_percentage: f64,
}

/// Storage inspector
pub struct StorageInspector {
    /// Local storage items
    local_storage: Arc<RwLock<HashMap<String, StorageItem>>>,
    /// Session storage items
    session_storage: Arc<RwLock<HashMap<String, StorageItem>>>,
    /// Cookies
    cookies: Arc<RwLock<HashMap<String, CookieItem>>>,
    /// IndexedDB databases
    indexed_db: Arc<RwLock<HashMap<String, IndexedDBDatabase>>>,
    /// Service workers
    service_workers: Arc<RwLock<HashMap<String, ServiceWorkerRegistration>>>,
    /// Cache storage
    cache_storage: Arc<RwLock<Vec<CacheEntry>>>,
    /// Current origin
    current_origin: Arc<RwLock<String>>,
}

impl StorageInspector {
    /// Create a new storage inspector
    pub fn new() -> Self {
        Self {
            local_storage: Arc::new(RwLock::new(HashMap::new())),
            session_storage: Arc::new(RwLock::new(HashMap::new())),
            cookies: Arc::new(RwLock::new(HashMap::new())),
            indexed_db: Arc::new(RwLock::new(HashMap::new())),
            service_workers: Arc::new(RwLock::new(HashMap::new())),
            cache_storage: Arc::new(RwLock::new(Vec::new())),
            current_origin: Arc::new(RwLock::new(String::new())),
        }
    }

    /// Initialize the inspector
    pub fn initialize(&self) -> Result<(), StorageError> {
        Ok(())
    }

    /// Set current origin
    pub async fn set_origin(&self, origin: String) {
        *self.current_origin.write().await = origin;
    }

    // Local Storage

    /// Get all local storage items
    pub async fn get_local_storage(&self) -> Vec<StorageItem> {
        self.local_storage.read().await.values().cloned().collect()
    }

    /// Get local storage item
    pub async fn get_local_storage_item(&self, key: &str) -> Option<StorageItem> {
        self.local_storage.read().await.get(key).cloned()
    }

    /// Set local storage item
    pub async fn set_local_storage_item(&self, key: String, value: String) -> StorageItem {
        let origin = self.current_origin.read().await.clone();
        let item = StorageItem {
            id: Uuid::new_v4(),
            key: key.clone(),
            value: value.clone(),
            storage_type: StorageType::LocalStorage,
            size: (key.len() + value.len()) as u64,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
            expires_at: None,
            origin,
        };
        
        self.local_storage.write().await.insert(key, item.clone());
        item
    }

    /// Remove local storage item
    pub async fn remove_local_storage_item(&self, key: &str) -> bool {
        self.local_storage.write().await.remove(key).is_some()
    }

    /// Clear local storage
    pub async fn clear_local_storage(&self) {
        self.local_storage.write().await.clear();
    }

    // Session Storage

    /// Get all session storage items
    pub async fn get_session_storage(&self) -> Vec<StorageItem> {
        self.session_storage.read().await.values().cloned().collect()
    }

    /// Set session storage item
    pub async fn set_session_storage_item(&self, key: String, value: String) -> StorageItem {
        let origin = self.current_origin.read().await.clone();
        let item = StorageItem {
            id: Uuid::new_v4(),
            key: key.clone(),
            value: value.clone(),
            storage_type: StorageType::SessionStorage,
            size: (key.len() + value.len()) as u64,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
            expires_at: None,
            origin,
        };
        
        self.session_storage.write().await.insert(key, item.clone());
        item
    }

    /// Remove session storage item
    pub async fn remove_session_storage_item(&self, key: &str) -> bool {
        self.session_storage.write().await.remove(key).is_some()
    }

    /// Clear session storage
    pub async fn clear_session_storage(&self) {
        self.session_storage.write().await.clear();
    }

    // Cookies

    /// Get all cookies
    pub async fn get_cookies(&self) -> Vec<CookieItem> {
        self.cookies.read().await.values().cloned().collect()
    }

    /// Get cookie by name
    pub async fn get_cookie(&self, name: &str) -> Option<CookieItem> {
        self.cookies.read().await.get(name).cloned()
    }

    /// Set cookie
    pub async fn set_cookie(&self, cookie: CookieItem) -> Result<(), StorageError> {
        // Validate cookie
        if cookie.name.is_empty() {
            return Err(StorageError::InvalidCookieName);
        }
        
        if cookie.value.contains(['\n', '\r', '\0']) {
            return Err(StorageError::InvalidCookieValue);
        }
        
        self.cookies.write().await.insert(cookie.name.clone(), cookie);
        Ok(())
    }

    /// Remove cookie
    pub async fn remove_cookie(&self, name: &str) -> bool {
        self.cookies.write().await.remove(name).is_some()
    }

    /// Clear all cookies
    pub async fn clear_cookies(&self) {
        self.cookies.write().await.clear();
    }

    /// Block cookie
    pub async fn block_cookie(&self, _name: &str) -> Result<(), StorageError> {
        Ok(())
    }

    // IndexedDB

    /// Get all IndexedDB databases
    pub async fn get_indexed_db_databases(&self) -> Vec<IndexedDBDatabase> {
        self.indexed_db.read().await.values().cloned().collect()
    }

    /// Get IndexedDB database
    pub async fn get_indexed_db_database(&self, name: &str) -> Option<IndexedDBDatabase> {
        self.indexed_db.read().await.get(name).cloned()
    }

    /// Create IndexedDB database
    pub async fn create_indexed_db_database(&self, name: String, version: u32) -> IndexedDBDatabase {
        let origin = self.current_origin.read().await.clone();
        let db = IndexedDBDatabase {
            id: Uuid::new_v4(),
            name: name.clone(),
            version,
            object_stores: Vec::new(),
            origin,
        };
        
        self.indexed_db.write().await.insert(name, db.clone());
        db
    }

    /// Delete IndexedDB database
    pub async fn delete_indexed_db_database(&self, name: &str) -> bool {
        self.indexed_db.write().await.remove(name).is_some()
    }

    /// Get object store entries
    pub async fn get_object_store_entries(&self, _db_name: &str, _store_name: &str) -> Vec<IndexedDBEntry> {
        // Simplified - in production, read from actual IndexedDB
        Vec::new()
    }

    // Service Workers

    /// Get all service workers
    pub async fn get_service_workers(&self) -> Vec<ServiceWorkerRegistration> {
        self.service_workers.read().await.values().cloned().collect()
    }

    /// Register service worker
    pub async fn register_service_worker(&self, script_url: String, scope_url: String) -> ServiceWorkerRegistration {
        let registration = ServiceWorkerRegistration {
            id: Uuid::new_v4(),
            scope_url,
            script_url: script_url.clone(),
            state: ServiceWorkerState::Installing,
            last_updated: Utc::now(),
            running_status: ServiceWorkerRunningStatus::Starting,
        };
        
        self.service_workers.write().await.insert(script_url, registration.clone());
        registration
    }

    /// Unregister service worker
    pub async fn unregister_service_worker(&self, script_url: &str) -> bool {
        self.service_workers.write().await.remove(script_url).is_some()
    }

    /// Update service worker
    pub async fn update_service_worker(&self, script_url: &str) -> Result<(), StorageError> {
        let mut workers = self.service_workers.write().await;
        if let Some(worker) = workers.get_mut(script_url) {
            worker.last_updated = Utc::now();
            worker.state = ServiceWorkerState::Installing;
            Ok(())
        } else {
            Err(StorageError::ServiceWorkerNotFound)
        }
    }

    // Cache Storage

    /// Get cache names
    pub async fn get_cache_names(&self) -> Vec<String> {
        let cache = self.cache_storage.read().await;
        cache.iter()
            .map(|entry| entry.cache_name.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }

    /// Get cache entries
    pub async fn get_cache_entries(&self, cache_name: &str) -> Vec<CacheEntry> {
        let cache = self.cache_storage.read().await;
        cache.iter()
            .filter(|entry| entry.cache_name == cache_name)
            .cloned()
            .collect()
    }

    /// Add cache entry
    pub async fn add_cache_entry(&self, entry: CacheEntry) {
        self.cache_storage.write().await.push(entry);
    }

    /// Delete cache
    pub async fn delete_cache(&self, cache_name: &str) -> usize {
        let mut cache = self.cache_storage.write().await;
        let initial_len = cache.len();
        cache.retain(|entry| entry.cache_name != cache_name);
        initial_len - cache.len()
    }

    /// Delete cache entry
    pub async fn delete_cache_entry(&self, cache_name: &str, url: &str) -> bool {
        let mut cache = self.cache_storage.write().await;
        let initial_len = cache.len();
        cache.retain(|entry| !(entry.cache_name == cache_name && entry.url == url));
        cache.len() < initial_len
    }

    // Statistics and Management

    /// Get storage statistics
    pub async fn get_statistics(&self) -> StorageStatistics {
        let local_storage = self.local_storage.read().await;
        let session_storage = self.session_storage.read().await;
        let cookies = self.cookies.read().await;
        let indexed_db = self.indexed_db.read().await;
        let cache = self.cache_storage.read().await;
        let service_workers = self.service_workers.read().await;
        
        let local_storage_size: u64 = local_storage.values().map(|i| i.size).sum();
        let session_storage_size: u64 = session_storage.values().map(|i| i.size).sum();
        let cookie_count = cookies.len() as u64;
        let cache_storage_size: u64 = cache.iter().map(|e| e.response.body_size).sum();
        let service_worker_count = service_workers.len() as u64;
        
        // Estimate IndexedDB size
        let indexed_db_size: u64 = indexed_db.values()
            .map(|db| db.object_stores.iter().map(|s| s.entry_count * 100).sum::<u64>())
            .sum();
        
        let total_size = local_storage_size + session_storage_size + indexed_db_size + cache_storage_size;
        let quota = 1024 * 1024 * 1024 * 50; // 50 MB default quota
        let usage_percentage = (total_size as f64 / quota as f64) * 100.0;
        
        StorageStatistics {
            local_storage_size,
            session_storage_size,
            cookie_count,
            indexed_db_size,
            cache_storage_size,
            service_worker_count,
            total_size,
            quota,
            usage_percentage,
        }
    }

    /// Clear all storage
    pub async fn clear_all_storage(&self) {
        self.clear_local_storage().await;
        self.clear_session_storage().await;
        self.clear_cookies().await;
        self.indexed_db.write().await.clear();
        self.service_workers.write().await.clear();
        self.cache_storage.write().await.clear();
    }

    /// Export storage data
    pub async fn export_storage(&self, storage_type: StorageType) -> Result<String, StorageError> {
        match storage_type {
            StorageType::LocalStorage => {
                let items = self.get_local_storage().await;
                serde_json::to_string_pretty(&items).map_err(StorageError::from)
            }
            StorageType::SessionStorage => {
                let items = self.get_session_storage().await;
                serde_json::to_string_pretty(&items).map_err(StorageError::from)
            }
            StorageType::Cookies => {
                let items = self.get_cookies().await;
                serde_json::to_string_pretty(&items).map_err(StorageError::from)
            }
            StorageType::IndexedDB => {
                let items = self.get_indexed_db_databases().await;
                serde_json::to_string_pretty(&items).map_err(StorageError::from)
            }
            _ => Err(StorageError::ExportNotSupported),
        }
    }

    /// Import storage data
    pub async fn import_storage(&self, storage_type: StorageType, data: &str) -> Result<(), StorageError> {
        match storage_type {
            StorageType::LocalStorage => {
                let items: Vec<StorageItem> = serde_json::from_str(data)?;
                for item in items {
                    self.set_local_storage_item(item.key, item.value).await;
                }
                Ok(())
            }
            StorageType::SessionStorage => {
                let items: Vec<StorageItem> = serde_json::from_str(data)?;
                for item in items {
                    self.set_session_storage_item(item.key, item.value).await;
                }
                Ok(())
            }
            StorageType::Cookies => {
                let cookies: Vec<CookieItem> = serde_json::from_str(data)?;
                for cookie in cookies {
                    self.set_cookie(cookie).await?;
                }
                Ok(())
            }
            _ => Err(StorageError::ImportNotSupported),
        }
    }
}

/// Storage error
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Invalid cookie name")]
    InvalidCookieName,
    #[error("Invalid cookie value")]
    InvalidCookieValue,
    #[error("Service worker not found")]
    ServiceWorkerNotFound,
    #[error("Object store not found")]
    ObjectStoreNotFound,
    #[error("Database not found")]
    DatabaseNotFound,
    #[error("Export not supported for this storage type")]
    ExportNotSupported,
    #[error("Import not supported for this storage type")]
    ImportNotSupported,
    #[error("Quota exceeded")]
    QuotaExceeded,
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Other error: {0}")]
    Other(String),
}

impl Default for StorageInspector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_inspector_initialization() {
        let inspector = StorageInspector::new();
        assert!(inspector.initialize().is_ok());
    }

    #[tokio::test]
    async fn test_local_storage() {
        let inspector = StorageInspector::new();
        inspector.set_origin("https://example.com".to_string()).await;
        
        inspector.set_local_storage_item("key1".to_string(), "value1".to_string()).await;
        
        let item = inspector.get_local_storage_item("key1").await.unwrap();
        assert_eq!(item.value, "value1");
        
        let items = inspector.get_local_storage().await;
        assert_eq!(items.len(), 1);
        
        inspector.remove_local_storage_item("key1").await;
        assert!(inspector.get_local_storage_item("key1").await.is_none());
    }

    #[tokio::test]
    async fn test_session_storage() {
        let inspector = StorageInspector::new();
        inspector.set_origin("https://example.com".to_string()).await;
        
        inspector.set_session_storage_item("session_key".to_string(), "session_value".to_string()).await;
        
        let item = inspector.get_session_storage_item("session_key").await.unwrap();
        assert_eq!(item.value, "session_value");
    }

    #[tokio::test]
    async fn test_cookies() {
        let inspector = StorageInspector::new();
        
        let cookie = CookieItem {
            id: Uuid::new_v4(),
            name: "test_cookie".to_string(),
            value: "cookie_value".to_string(),
            domain: "example.com".to_string(),
            path: "/".to_string(),
            expires: None,
            max_age: None,
            secure: true,
            http_only: false,
            same_site: SameSite::Lax,
            size: 24,
            priority: CookiePriority::Medium,
        };
        
        inspector.set_cookie(cookie).await.unwrap();
        
        let retrieved = inspector.get_cookie("test_cookie").await.unwrap();
        assert_eq!(retrieved.value, "cookie_value");
    }

    #[tokio::test]
    async fn test_indexed_db() {
        let inspector = StorageInspector::new();
        inspector.set_origin("https://example.com".to_string()).await;
        
        let db = inspector.create_indexed_db_database("test_db".to_string(), 1).await;
        assert_eq!(db.name, "test_db");
        
        let databases = inspector.get_indexed_db_databases().await;
        assert_eq!(databases.len(), 1);
        
        inspector.delete_indexed_db_database("test_db").await;
        assert!(inspector.get_indexed_db_database("test_db").await.is_none());
    }

    #[tokio::test]
    async fn test_service_workers() {
        let inspector = StorageInspector::new();
        
        let sw = inspector.register_service_worker(
            "https://example.com/sw.js".to_string(),
            "https://example.com/".to_string()
        ).await;
        
        assert_eq!(sw.state, ServiceWorkerState::Installing);
        
        let workers = inspector.get_service_workers().await;
        assert_eq!(workers.len(), 1);
    }

    #[tokio::test]
    async fn test_cache_storage() {
        let inspector = StorageInspector::new();
        
        let entry = CacheEntry {
            id: Uuid::new_v4(),
            cache_name: "v1".to_string(),
            url: "https://example.com/style.css".to_string(),
            response: CacheResponse {
                status: 200,
                status_text: "OK".to_string(),
                headers: HashMap::new(),
                body_size: 1024,
                content_type: Some("text/css".to_string()),
            },
            created_at: Utc::now(),
        };
        
        inspector.add_cache_entry(entry).await;
        
        let entries = inspector.get_cache_entries("v1").await;
        assert_eq!(entries.len(), 1);
    }

    #[tokio::test]
    async fn test_statistics() {
        let inspector = StorageInspector::new();
        inspector.set_origin("https://example.com".to_string()).await;
        
        inspector.set_local_storage_item("key".to_string(), "value".to_string()).await;
        
        let stats = inspector.get_statistics().await;
        assert!(stats.local_storage_size > 0);
        assert!(stats.total_size > 0);
    }

    #[tokio::test]
    async fn test_export_import() {
        let inspector = StorageInspector::new();
        inspector.set_origin("https://example.com".to_string()).await;
        
        inspector.set_local_storage_item("export_key".to_string(), "export_value".to_string()).await;
        
        let exported = inspector.export_storage(StorageType::LocalStorage).await.unwrap();
        assert!(exported.contains("export_key"));
        
        inspector.clear_local_storage().await;
        
        inspector.import_storage(StorageType::LocalStorage, &exported).await.unwrap();
        let items = inspector.get_local_storage().await;
        assert_eq!(items.len(), 1);
    }

    #[tokio::test]
    async fn test_clear_all() {
        let inspector = StorageInspector::new();
        inspector.set_origin("https://example.com".to_string()).await;
        
        inspector.set_local_storage_item("key".to_string(), "value".to_string()).await;
        inspector.clear_all_storage().await;
        
        let stats = inspector.get_statistics().await;
        assert_eq!(stats.total_size, 0);
    }
}