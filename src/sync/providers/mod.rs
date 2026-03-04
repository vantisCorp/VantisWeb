// VantisWeb Browser - Cloud Sync Providers
// Copyright (c) 2024 VantisCorp
// Sync provider implementations for various cloud services

pub mod google_drive;
pub mod dropbox;
pub mod icloud;
pub mod webdav;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use async_trait::async_trait;
use crate::sync::{SyncError, SyncResult, SyncStats};

pub use google_drive::GoogleDriveProvider;
pub use dropbox::DropboxProvider;
pub use icloud::ICloudProvider;
pub use webdav::WebDAVProvider;

/// Types of supported cloud sync providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncProviderType {
    GoogleDrive,
    Dropbox,
    ICloud,
    WebDAV,
}

impl std::fmt::Display for SyncProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GoogleDrive => write!(f, "Google Drive"),
            Self::Dropbox => write!(f, "Dropbox"),
            Self::ICloud => write!(f, "iCloud"),
            Self::WebDAV => write!(f, "WebDAV"),
        }
    }
}

/// Credentials for cloud sync providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncCredentials {
    OAuth2 {
        access_token: String,
        refresh_token: String,
        expires_at: chrono::DateTime<chrono::Utc>,
    },
    BasicAuth {
        username: String,
        password: String,
    },
    ApiKey {
        key: String,
    },
    AppleId {
        user_identifier: String,
        device_token: String,
    },
}

/// Sync provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub provider_type: SyncProviderType,
    pub credentials: SyncCredentials,
    pub root_folder: String,
    pub auto_sync: bool,
    pub sync_interval_minutes: u32,
    pub conflict_resolution: ConflictResolutionStrategy,
    pub selective_sync: SelectiveSyncConfig,
    pub encryption: EncryptionConfig,
}

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictResolutionStrategy {
    KeepLocal,
    KeepRemote,
    KeepNewer,
    Merge,
    AskUser,
}

/// Selective sync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectiveSyncConfig {
    pub sync_profiles: bool,
    pub sync_bookmarks: bool,
    pub sync_history: bool,
    pub sync_extensions: bool,
    pub sync_settings: bool,
    pub sync_themes: bool,
    pub excluded_profile_ids: Vec<String>,
}

impl Default for SelectiveSyncConfig {
    fn default() -> Self {
        Self {
            sync_profiles: true,
            sync_bookmarks: true,
            sync_history: true,
            sync_extensions: true,
            sync_settings: true,
            sync_themes: true,
            excluded_profile_ids: Vec::new(),
        }
    }
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub enabled: bool,
    pub algorithm: EncryptionAlgorithm,
    pub key_derivation: KeyDerivation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EncryptionAlgorithm {
    AES256GCM,
    ChaCha20Poly1305,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyDerivation {
    Argon2,
    PBKDF2,
    Scrypt,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            algorithm: EncryptionAlgorithm::AES256GCM,
            key_derivation: KeyDerivation::Argon2,
        }
    }
}

/// Sync status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub provider: SyncProviderType,
    pub connected: bool,
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
    pub last_sync_status: Option<String>,
    pub pending_changes: usize,
    pub storage_used: u64,
    pub storage_limit: Option<u64>,
    pub sync_in_progress: bool,
    pub error_count: usize,
    pub last_error: Option<String>,
}

/// Remote file metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteFile {
    pub id: String,
    pub name: String,
    pub path: String,
    pub size: u64,
    pub modified: chrono::DateTime<chrono::Utc>,
    pub created: chrono::DateTime<chrono::Utc>,
    pub hash: String,
    pub version: Option<String>,
}

/// Sync item for tracking sync operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncItem {
    pub local_id: String,
    pub remote_id: Option<String>,
    pub name: String,
    pub item_type: SyncItemType,
    pub local_modified: Option<chrono::DateTime<chrono::Utc>>,
    pub remote_modified: Option<chrono::DateTime<chrono::Utc>>,
    pub local_hash: Option<String>,
    pub remote_hash: Option<String>,
    pub status: ItemSyncStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncItemType {
    Profile,
    BookmarkFolder,
    History,
    Extension,
    Setting,
    Theme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemSyncStatus {
    Synced,
    PendingUpload,
    PendingDownload,
    Conflict,
    Error,
    Excluded,
}

/// Main sync provider trait
#[async_trait]
pub trait SyncProvider: Send + Sync {
    /// Get the provider type
    fn provider_type(&self) -> SyncProviderType;
    
    /// Authenticate with the provider
    async fn authenticate(&mut self, credentials: &SyncCredentials) -> SyncResult<()>;
    
    /// Check if currently authenticated
    fn is_authenticated(&self) -> bool;
    
    /// Refresh authentication tokens
    async fn refresh_auth(&mut self) -> SyncResult<()>;
    
    /// Disconnect and revoke tokens
    async fn disconnect(&mut self) -> SyncResult<()>;
    
    /// Get current sync status
    async fn get_status(&self) -> SyncResult<SyncStatus>;
    
    /// List remote files
    async fn list_remote_files(&self, path: &str) -> SyncResult<Vec<RemoteFile>>;
    
    /// Upload a file
    async fn upload_file(&self, local_path: &str, remote_path: &str, data: &[u8]) -> SyncResult<RemoteFile>;
    
    /// Download a file
    async fn download_file(&self, remote_id: &str) -> SyncResult<Vec<u8>>;
    
    /// Delete a remote file
    async fn delete_file(&self, remote_id: &str) -> SyncResult<()>;
    
    /// Create a folder
    async fn create_folder(&self, path: &str) -> SyncResult<RemoteFile>;
    
    /// Get storage quota information
    async fn get_storage_info(&self) -> SyncResult<StorageInfo>;
    
    /// Subscribe to file changes (for real-time sync)
    async fn subscribe_changes(&self) -> SyncResult<()>;
    
    /// Get file changes since last sync
    async fn get_changes(&self, since: chrono::DateTime<chrono::Utc>) -> SyncResult<Vec<FileChange>>;
}

/// Storage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub used_bytes: u64,
    pub limit_bytes: Option<u64>,
    pub plan_name: Option<String>,
}

/// File change notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub file: RemoteFile,
    pub change_type: FileChangeType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileChangeType {
    Created,
    Modified,
    Deleted,
    Moved,
}

/// Provider capability flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderCapabilities {
    pub supports_realtime: bool,
    pub supports_versioning: bool,
    pub supports_sharing: bool,
    pub supports_selective_sync: bool,
    pub max_file_size: u64,
}