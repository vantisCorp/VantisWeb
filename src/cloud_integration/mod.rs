//! Cloud Integration Module
//! 
//! Comprehensive cloud-native features for seamless cross-device experience
//! including cloud storage, real-time sync, and collaborative browsing.

pub mod storage;
pub mod sync;
pub mod collaboration;
pub mod backup;
pub mod cross_device;
pub mod models;

use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

pub use storage::{CloudStorageProvider, StorageManager, StorageConfig};
pub use sync::{RealTimeSync, SyncEngine, SyncStatus};
pub use collaboration::{CollaborativeSession, SessionManager, Participant};
pub use backup::{CloudBackup, BackupScheduler, RestoreManager};
pub use cross_device::{CrossDeviceManager, DeviceRegistry, Device};
pub use models::*;

/// Cloud Integration Manager - Central hub for cloud features
pub struct CloudManager {
    /// Storage manager
    storage: Arc<StorageManager>,
    /// Real-time sync engine
    sync: Arc<RealTimeSync>,
    /// Session manager for collaboration
    sessions: Arc<SessionManager>,
    /// Backup manager
    backup: Arc<CloudBackup>,
    /// Cross-device manager
    devices: Arc<CrossDeviceManager>,
    /// Configuration
    config: CloudConfig,
    /// Connection state
    connection: RwLock<ConnectionState>,
}

/// Cloud configuration
#[derive(Debug, Clone)]
pub struct CloudConfig {
    /// Enable cloud storage
    pub storage_enabled: bool,
    /// Enable real-time sync
    pub sync_enabled: bool,
    /// Enable collaboration features
    pub collaboration_enabled: bool,
    /// Enable automatic backups
    pub auto_backup: bool,
    /// Sync interval in seconds
    pub sync_interval: u64,
    /// Maximum storage quota in MB
    pub storage_quota_mb: usize,
    /// Encryption enabled
    pub encryption_enabled: bool,
    /// Compression enabled
    pub compression_enabled: bool,
    /// Offline mode support
    pub offline_mode: bool,
    /// Conflict resolution strategy
    pub conflict_strategy: ConflictStrategy,
}

impl Default for CloudConfig {
    fn default() -> Self {
        Self {
            storage_enabled: true,
            sync_enabled: true,
            collaboration_enabled: true,
            auto_backup: true,
            sync_interval: 30,
            storage_quota_mb: 1024,
            encryption_enabled: true,
            compression_enabled: true,
            offline_mode: true,
            conflict_strategy: ConflictStrategy::LatestWins,
        }
    }
}

/// Conflict resolution strategy
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictStrategy {
    /// Keep the latest version
    LatestWins,
    /// Keep the local version
    LocalWins,
    /// Keep the remote version
    RemoteWins,
    /// Merge changes when possible
    Merge,
    /// Ask user for decision
    Manual,
}

/// Connection state
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Connected,
    Disconnected,
    Syncing,
    Error(String),
}

impl CloudManager {
    pub fn new(config: CloudConfig) -> Self {
        Self {
            storage: Arc::new(StorageManager::new(config.clone())),
            sync: Arc::new(RealTimeSync::new(config.clone())),
            sessions: Arc::new(SessionManager::new()),
            backup: Arc::new(CloudBackup::new(config.clone())),
            devices: Arc::new(CrossDeviceManager::new()),
            config,
            connection: RwLock::new(ConnectionState::Disconnected),
        }
    }

    /// Initialize cloud connection
    pub async fn initialize(&self, credentials: CloudCredentials) -> Result<(), CloudError> {
        // Authenticate with cloud provider
        self.storage.authenticate(&credentials).await?;
        
        // Register current device
        self.devices.register_current_device().await?;
        
        // Start sync engine
        if self.config.sync_enabled {
            self.sync.start().await?;
        }
        
        // Schedule automatic backups
        if self.config.auto_backup {
            self.backup.schedule_automatic().await?;
        }
        
        *self.connection.write().await = ConnectionState::Connected;
        
        Ok(())
    }

    /// Get connection status
    pub async fn get_status(&self) -> CloudStatus {
        let connection = self.connection.read().await.clone();
        let storage_usage = self.storage.get_usage().await;
        let sync_status = self.sync.get_status().await;
        let active_sessions = self.sessions.active_count().await;
        let devices = self.devices.count().await;
        
        CloudStatus {
            connection,
            storage_usage,
            sync_status,
            active_sessions,
            connected_devices: devices,
        }
    }

    /// Sync all data
    pub async fn sync_all(&self) -> Result<SyncResult, CloudError> {
        *self.connection.write().await = ConnectionState::Syncing;
        
        let result = self.sync.sync_all().await;
        
        *self.connection.write().await = ConnectionState::Connected;
        
        result
    }

    /// Create a collaborative session
    pub async fn create_session(&self, name: &str) -> Result<CollaborativeSession, CloudError> {
        if !self.config.collaboration_enabled {
            return Err(CloudError::FeatureDisabled);
        }
        
        self.sessions.create(name).await
    }

    /// Join a collaborative session
    pub async fn join_session(&self, session_id: &str) -> Result<CollaborativeSession, CloudError> {
        if !self.config.collaboration_enabled {
            return Err(CloudError::FeatureDisabled);
        }
        
        self.sessions.join(session_id).await
    }

    /// Create a backup
    pub async fn create_backup(&self) -> Result<BackupInfo, CloudError> {
        self.backup.create().await
    }

    /// Restore from backup
    pub async fn restore_backup(&self, backup_id: &str) -> Result<(), CloudError> {
        self.backup.restore(backup_id).await
    }

    /// List all devices
    pub async fn list_devices(&self) -> Vec<Device> {
        self.devices.list().await
    }

    /// Send a command to another device
    pub async fn send_command(&self, device_id: &str, command: DeviceCommand) -> Result<(), CloudError> {
        self.devices.send_command(device_id, command).await
    }

    /// Disconnect from cloud
    pub async fn disconnect(&self) -> Result<(), CloudError> {
        // Stop sync
        self.sync.stop().await?;
        
        // Leave all sessions
        self.sessions.leave_all().await?;
        
        // Update connection state
        *self.connection.write().await = ConnectionState::Disconnected;
        
        Ok(())
    }
}

/// Cloud credentials
#[derive(Debug, Clone)]
pub struct CloudCredentials {
    pub provider: CloudProvider,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub user_id: String,
}

/// Cloud provider
#[derive(Debug, Clone, PartialEq)]
pub enum CloudProvider {
    VantisCloud,
    GoogleDrive,
    OneDrive,
    Dropbox,
    iCloud,
    AWS,
    Custom(String),
}

/// Cloud error
#[derive(Debug, thiserror::Error)]
pub enum CloudError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Sync error: {0}")]
    SyncError(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Session error: {0}")]
    SessionError(String),
    #[error("Backup error: {0}")]
    BackupError(String),
    #[error("Device error: {0}")]
    DeviceError(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Quota exceeded")]
    QuotaExceeded,
    #[error("Feature disabled")]
    FeatureDisabled,
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Permission denied")]
    PermissionDenied,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloud_config_defaults() {
        let config = CloudConfig::default();
        assert!(config.storage_enabled);
        assert!(config.sync_enabled);
        assert_eq!(config.sync_interval, 30);
    }
}