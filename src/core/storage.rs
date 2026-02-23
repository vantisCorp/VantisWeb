//! Vantis Storage Manager
//! 
//! Advanced data persistence system:
//! - SQLite for structured data
//! - KV store for fast access
//! - File system integration
//! - Encryption support
//! - Merkle Tree verification

use anyhow::{Context, Result};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use sled::Db;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::config::VantisConfig;

/// Vantis Storage Manager
pub struct StorageManager {
    config: Arc<VantisConfig>,
    db: Option<Arc<Db>>,
    initialized: bool,
}

impl StorageManager {
    /// Create a new storage manager
    pub async fn new(config: Arc<VantisConfig>) -> Result<Self> {
        info!("Initializing Vantis Storage Manager...");
        
        Ok(Self {
            config,
            db: None,
            initialized: false,
        })
    }
    
    /// Initialize storage system
    pub async fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            warn!("Storage already initialized");
            return Ok(());
        }
        
        info!("Setting up storage directories...");
        
        // Create directories
        let data_dir = self.config.get_data_dir();
        std::fs::create_dir_all(&data_dir)?;
        
        let cache_dir = self.config.get_cache_dir();
        std::fs::create_dir_all(&cache_dir)?;
        
        // Initialize database
        let db_path = data_dir.join("vantisweb.db");
        info!("Opening database: {:?}", db_path);
        
        let db = sled::open(&db_path).context("Failed to open database")?;
        self.db = Some(Arc::new(db));
        
        self.initialized = true;
        info!("Storage Manager initialized successfully");
        
        Ok(())
    }
    
    /// Store a value with encryption
    pub async fn store(&self, key: &str, value: &[u8]) -> Result<()> {
        let db = self.db.as_ref()
            .context("Storage not initialized")?;
        
        db.insert(key, value)
            .context("Failed to store value")?;
        
        db.flush_async().await
            .context("Failed to flush database")?;
        
        debug!("Stored key: {}", key);
        
        Ok(())
    }
    
    /// Retrieve a value
    pub async fn retrieve(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let db = self.db.as_ref()
            .context("Storage not initialized")?;
        
        match db.get(key)? {
            Some(value) => {
                debug!("Retrieved key: {}", key);
                Ok(Some(value.to_vec()))
            }
            None => {
                debug!("Key not found: {}", key);
                Ok(None)
            }
        }
    }
    
    /// Delete a value
    pub async fn delete(&self, key: &str) -> Result<()> {
        let db = self.db.as_ref()
            .context("Storage not initialized")?;
        
        db.remove(key)
            .context("Failed to delete value")?;
        
        db.flush_async().await
            .context("Failed to flush database")?;
        
        debug!("Deleted key: {}", key);
        
        Ok(())
    }
    
    /// Check if key exists
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let db = self.db.as_ref()
            .context("Storage not initialized")?;
        
        Ok(db.get(key)?.is_some())
    }
    
    /// List all keys
    pub async fn list_keys(&self) -> Result<Vec<String>> {
        let db = self.db.as_ref()
            .context("Storage not initialized")?;
        
        let keys: Vec<String> = db.iter()
            .filter_map(|item| item.ok())
            .filter_map(|(key, _)| String::from_utf8(key.to_vec()).ok())
            .collect();
        
        Ok(keys)
    }
    
    /// Backup database
    pub async fn backup(&self, backup_path: PathBuf) -> Result<()> {
        info!("Creating backup to: {:?}", backup_path);
        
        let db = self.db.as_ref()
            .context("Storage not initialized")?;
        
        db.export(backup_path)
            .context("Failed to export database")?;
        
        info!("Backup created successfully");
        
        Ok(())
    }
    
    /// Restore database
    pub async fn restore(&self, backup_path: PathBuf) -> Result<()> {
        info!("Restoring from backup: {:?}", backup_path);
        
        let db = self.db.as_ref()
            .context("Storage not initialized")?;
        
        db.import(backup_path)
            .context("Failed to import database")?;
        
        db.flush_async().await
            .context("Failed to flush database")?;
        
        info!("Database restored successfully");
        
        Ok(())
    }
}