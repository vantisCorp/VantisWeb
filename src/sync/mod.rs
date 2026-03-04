// VantisWeb Browser - Cloud Sync Module
// Copyright (c) 2024 VantisCorp
// Advanced cloud sync providers for profile synchronization

pub mod providers;
pub mod oauth;
pub mod sync_manager;
pub mod conflict_resolver;

pub use providers::{
    SyncProvider, SyncProviderType, SyncCredentials, 
    SyncConfig, SyncStatus, SyncResult
};
pub use oauth::{OAuth2Provider, OAuth2Token, OAuth2Config};
pub use sync_manager::CloudSyncManager;
pub use conflict_resolver::{ConflictResolver, ConflictResolution, SyncConflict};

/// Cloud sync error types
#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Sync conflict: {0}")]
    Conflict(String),
    
    #[error("Provider error: {0}")]
    ProviderError(String),
    
    #[error("Storage quota exceeded")]
    QuotaExceeded,
    
    #[error("Invalid configuration")]
    InvalidConfig,
    
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Operation cancelled")]
    Cancelled,
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type SyncResult<T> = Result<T, SyncError>;

/// Sync event for progress tracking
#[derive(Debug, Clone)]
pub enum SyncEvent {
    Started { provider: String },
    Progress { current: usize, total: usize, item: String },
    ConflictDetected { conflict: SyncConflict },
    ConflictResolved { resolution: ConflictResolution },
    Completed { stats: SyncStats },
    Failed { error: String },
    Cancelled,
}

/// Sync statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStats {
    pub profiles_synced: usize,
    pub profiles_created: usize,
    pub profiles_updated: usize,
    pub profiles_deleted: usize,
    pub conflicts_resolved: usize,
    pub bytes_transferred: u64,
    pub duration_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for SyncStats {
    fn default() -> Self {
        Self {
            profiles_synced: 0,
            profiles_created: 0,
            profiles_updated: 0,
            profiles_deleted: 0,
            conflicts_resolved: 0,
            bytes_transferred: 0,
            duration_ms: 0,
            timestamp: chrono::Utc::now(),
        }
    }
}