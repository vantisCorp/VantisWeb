//! Password Manager for VantisWeb Browser
//! 
//! This module provides a secure, built-in password manager with:
//! - Encrypted password storage
//! - Password generation
//! - Form autofill
//! - Breach monitoring
//! - Cross-device synchronization

pub mod storage;
pub mod generator;
pub mod autofill;
pub mod crypto;
pub mod sync;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Main password manager
pub struct PasswordManager {
    /// Configuration
    config: Arc<RwLock<PasswordConfig>>,
    /// Storage backend
    storage: Arc<storage::PasswordStorage>,
    /// Password generator
    generator: Arc<generator::PasswordGenerator>,
    /// Autofill manager
    autofill: Arc<autofill::AutofillManager>,
    /// Crypto utilities
    crypto: Arc<crypto::CryptoManager>,
    /// Sync manager
    sync: Arc<sync::PasswordSync>,
}

/// Password manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordConfig {
    /// Enable autofill
    pub enable_autofill: bool,
    /// Auto-lock timeout (seconds)
    pub auto_lock_timeout: u32,
    /// Require master password
    pub require_master_password: bool,
    /// Enable breach monitoring
    pub breach_monitoring: bool,
    /// Enable biometric unlock
    pub biometric_unlock: bool,
    /// Sync passwords across devices
    pub enable_sync: bool,
    /// Default password length
    pub default_password_length: u8,
}

impl Default for PasswordConfig {
    fn default() -> Self {
        Self {
            enable_autofill: true,
            auto_lock_timeout: 300, // 5 minutes
            require_master_password: true,
            breach_monitoring: true,
            biometric_unlock: false,
            enable_sync: false,
            default_password_length: 16,
        }
    }
}

/// A password entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordEntry {
    /// Unique ID
    pub id: String,
    /// Website URL
    pub url: String,
    /// Website name
    pub name: String,
    /// Username
    pub username: String,
    /// Encrypted password
    pub password_encrypted: String,
    /// Additional fields
    pub fields: Vec<PasswordField>,
    /// Notes
    pub notes: Option<String>,
    /// Folder/Category
    pub folder: Option<String>,
    /// When created
    pub created_at: DateTime<Utc>,
    /// Last modified
    pub modified_at: DateTime<Utc>,
    /// Last used
    pub last_used: Option<DateTime<Utc>>,
    /// Times used
    pub use_count: u32,
    /// Strength score (0-100)
    pub strength_score: u8,
    /// Whether password is in a known breach
    pub in_breach: bool,
}

/// Additional password field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordField {
    /// Field name
    pub name: String,
    /// Field value (encrypted)
    pub value: String,
    /// Field type
    pub field_type: FieldType,
}

/// Field types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldType {
    Text,
    Email,
    Phone,
    CreditCard,
    SecurityQuestion,
    Custom,
}

/// Password strength result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordStrength {
    /// Strength score (0-100)
    pub score: u8,
    /// Strength level
    pub level: StrengthLevel,
    /// Suggestions for improvement
    pub suggestions: Vec<String>,
    /// Estimated crack time
    pub crack_time: String,
}

/// Password strength levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrengthLevel {
    VeryWeak,
    Weak,
    Fair,
    Strong,
    VeryStrong,
}

/// Breach check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreachResult {
    /// Whether password is in breach
    pub breached: bool,
    /// Number of breaches found
    pub breach_count: u32,
    /// Breach details
    pub breaches: Vec<BreachInfo>,
}

/// Breach information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreachInfo {
    /// Breach name
    pub name: String,
    /// Breach date
    pub date: String,
    /// Data exposed
    pub exposed_data: Vec<String>,
    /// Breach severity
    pub severity: BreachSeverity,
}

/// Breach severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BreachSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Search query for passwords
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordQuery {
    /// Search term
    pub search_term: Option<String>,
    /// Folder filter
    pub folder: Option<String>,
    /// Breach filter
    pub breached_only: bool,
    /// Sort by
    pub sort_by: SortField,
    /// Sort order
    pub sort_order: SortOrder,
}

/// Sort fields
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortField {
    Name,
    Created,
    Modified,
    Used,
    Strength,
}

/// Sort order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl PasswordManager {
    /// Create a new password manager
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(PasswordConfig::default())),
            storage: Arc::new(storage::PasswordStorage::new()),
            generator: Arc::new(generator::PasswordGenerator::new()),
            autofill: Arc::new(autofill::AutofillManager::new()),
            crypto: Arc::new(crypto::CryptoManager::new()),
            sync: Arc::new(sync::PasswordSync::new()),
        }
    }

    /// Initialize the password manager
    pub async fn initialize(&self, master_password: Option<String>) -> Result<(), PasswordError> {
        let config = self.config.read().await;
        
        // Initialize crypto with master password
        if let Some(pwd) = master_password {
            self.crypto.set_master_key(&pwd).await?;
        }
        
        // Initialize storage
        self.storage.initialize().await?;
        
        // Initialize sync if enabled
        if config.enable_sync {
            self.sync.initialize().await?;
        }
        
        Ok(())
    }

    /// Save a password
    pub async fn save_password(&self, entry: PasswordEntry) -> Result<String, PasswordError> {
        // Validate entry
        if entry.username.is_empty() || entry.password_encrypted.is_empty() {
            return Err(PasswordError::ValidationError("Username and password required".to_string()));
        }
        
        // Store in storage
        let id = self.storage.save_entry(&entry).await?;
        
        // Update sync
        let config = self.config.read().await;
        if config.enable_sync {
            drop(config);
            self.sync.sync_entry(&id).await?;
        }
        
        Ok(id)
    }

    /// Get a password entry by ID
    pub async fn get_password(&self, id: &str) -> Result<PasswordEntry, PasswordError> {
        self.storage.get_entry(id).await
    }

    /// Get password for a URL (for autofill)
    pub async fn get_password_for_url(&self, url: &str) -> Result<Option<PasswordEntry>, PasswordError> {
        self.autofill.get_password_for_url(url).await
    }

    /// Search passwords
    pub async fn search_passwords(&self, query: PasswordQuery) -> Result<Vec<PasswordEntry>, PasswordError> {
        self.storage.search_entries(query).await
    }

    /// Update a password entry
    pub async fn update_password(&self, entry: PasswordEntry) -> Result<(), PasswordError> {
        self.storage.update_entry(&entry).await?;
        
        let config = self.config.read().await;
        if config.enable_sync {
            drop(config);
            self.sync.sync_entry(&entry.id).await?;
        }
        
        Ok(())
    }

    /// Delete a password entry
    pub async fn delete_password(&self, id: &str) -> Result<(), PasswordError> {
        self.storage.delete_entry(id).await?;
        
        let config = self.config.read().await;
        if config.enable_sync {
            drop(config);
            self.sync.delete_entry(id).await?;
        }
        
        Ok(())
    }

    /// Generate a new password
    pub async fn generate_password(&self, length: u8, options: PasswordGeneratorOptions) -> Result<String, PasswordError> {
        self.generator.generate(length, options).await
    }

    /// Check password strength
    pub async fn check_strength(&self, password: &str) -> PasswordStrength {
        self.generator.check_strength(password)
    }

    /// Check if password is in a breach
    pub async fn check_breach(&self, password: &str) -> Result<BreachResult, PasswordError> {
        let config = self.config.read().await;
        if !config.breach_monitoring {
            return Ok(BreachResult {
                breached: false,
                breach_count: 0,
                breaches: Vec::new(),
            });
        }
        drop(config);
        
        // Check against breach database
        self.storage.check_breach(password).await
    }

    /// Auto-fill login form
    pub async fn autofill_login(&self, url: &str, username: String) -> Result<AutofillResult, PasswordError> {
        let config = self.config.read().await;
        if !config.enable_autofill {
            return Err(PasswordError::AutofillDisabled);
        }
        drop(config);
        
        self.autofill.autofill(url, username).await
    }

    /// Import passwords from another manager
    pub async fn import_passwords(&self, format: ImportFormat, data: String) -> Result<usize, PasswordError> {
        let entries = self.storage.import(format, data).await?;
        let count = entries.len();
        
        for entry in entries {
            self.storage.save_entry(&entry).await?;
        }
        
        Ok(count)
    }

    /// Export passwords
    pub async fn export_passwords(&self, format: ExportFormat) -> Result<String, PasswordError> {
        self.storage.export(format).await
    }

    /// Update configuration
    pub async fn update_config<F>(&self, f: F)
    where
        F: FnOnce(&mut PasswordConfig),
    {
        let mut config = self.config.write().await;
        f(&mut config);
    }

    /// Get configuration
    pub async fn get_config(&self) -> PasswordConfig {
        self.config.read().await.clone()
    }

    /// Lock the password manager
    pub async fn lock(&self) {
        self.crypto.lock().await;
    }

    /// Unlock the password manager
    pub async fn unlock(&self, master_password: &str) -> Result<(), PasswordError> {
        self.crypto.unlock(master_password).await
    }

    /// Check if locked
    pub async fn is_locked(&self) -> bool {
        self.crypto.is_locked().await
    }

    /// Get statistics
    pub async fn get_stats(&self) -> PasswordStats {
        let total = self.storage.get_entry_count().await;
        let breached = self.storage.get_breached_count().await;
        
        PasswordStats {
            total_passwords: total,
            breached_passwords: breached,
            weak_passwords: 0, // TODO: Calculate
            unique_domains: 0, // TODO: Calculate
            average_strength: 0, // TODO: Calculate
        }
    }

    /// Change master password
    pub async fn change_master_password(&self, old_password: &str, new_password: &str) -> Result<(), PasswordError> {
        // Verify old password
        self.crypto.verify_master_password(old_password).await?;
        
        // Re-encrypt all passwords with new key
        self.crypto.reencrypt_all(old_password, new_password).await?;
        
        Ok(())
    }
}

/// Password generator options
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PasswordGeneratorOptions {
    /// Include uppercase letters
    pub uppercase: bool,
    /// Include lowercase letters
    pub lowercase: bool,
    /// Include numbers
    pub numbers: bool,
    /// Include symbols
    pub symbols: bool,
    /// Exclude similar characters (i, l, 1, L, o, 0, O)
    pub exclude_similar: bool,
}

impl Default for PasswordGeneratorOptions {
    fn default() -> Self {
        Self {
            uppercase: true,
            lowercase: true,
            numbers: true,
            symbols: true,
            exclude_similar: true,
        }
    }
}

/// Autofill result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutofillResult {
    /// Username filled
    pub username: String,
    /// Password filled
    pub password: String,
    /// Additional fields filled
    pub additional_fields: HashMap<String, String>,
    /// Form submitted automatically
    pub auto_submitted: bool,
}

/// Import formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportFormat {
    /// CSV export
    CSV,
    /// JSON export
    JSON,
    /// 1Password export
    OnePassword,
    /// LastPass export
    LastPass,
    /// Bitwarden export
    Bitwarden,
}

/// Export formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    /// CSV format
    CSV,
    /// JSON format
    JSON,
    /// Encrypted JSON
    EncryptedJSON,
}

/// Password statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordStats {
    pub total_passwords: usize,
    pub breached_passwords: usize,
    pub weak_passwords: usize,
    pub unique_domains: usize,
    pub average_strength: u8,
}

/// Password errors
#[derive(Debug, thiserror::Error)]
pub enum PasswordError {
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Crypto error: {0}")]
    CryptoError(String),
    
    #[error("Sync error: {0}")]
    SyncError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Already locked")]
    AlreadyLocked,
    
    #[error("Already unlocked")]
    AlreadyUnlocked,
    
    #[error("Invalid master password")]
    InvalidMasterPassword,
    
    #[error("Autofill disabled")]
    AutofillDisabled,
    
    #[error("Import error: {0}")]
    ImportError(String),
    
    #[error("Export error: {0}")]
    ExportError(String),
}

impl Default for PasswordManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_manager() {
        let manager = PasswordManager::new();
        assert!(!manager.is_locked().await);
    }

    #[tokio::test]
    async fn test_generate_password() {
        let manager = PasswordManager::new();
        let options = PasswordGeneratorOptions::default();
        let password = manager.generate_password(16, options).await.unwrap();
        
        assert_eq!(password.len(), 16);
    }

    #[tokio::test]
    async fn test_check_strength() {
        let manager = PasswordManager::new();
        let weak = manager.check_strength("password").await;
        assert!(weak.score < 50);
        
        let strong = manager.check_strength("P@ssw0rd!123456").await;
        assert!(strong.score > 80);
    }

    #[tokio::test]
    async fn test_lock_unlock() {
        let manager = PasswordManager::new();
        manager.lock().await;
        assert!(manager.is_locked().await);
        
        manager.unlock("test123").await.ok();
        assert!(!manager.is_locked().await);
    }

    #[tokio::test]
    async fn test_config_update() {
        let manager = PasswordManager::new();
        manager.update_config(|config| {
            config.enable_autofill = false;
        }).await;
        
        let config = manager.get_config().await;
        assert!(!config.enable_autofill);
    }
}