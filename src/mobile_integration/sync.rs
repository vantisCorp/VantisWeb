//! Mobile Sync Engine
//! 
//! Handles synchronization between browser and mobile devices.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc, broadcast};
use anyhow::Result;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

use super::{
    SyncSettings, SyncResult, MobileConfig, MobileEvent,
    models::{SyncDataPacket, SyncDataType, SyncOperation, SyncConflict},
};
use crate::cloud_integration::CloudManager;

/// Mobile sync engine
pub struct MobileSync {
    /// Cloud manager reference
    cloud_manager: Arc<CloudManager>,
    /// Sync queues per device
    sync_queues: RwLock<HashMap<String, mpsc::Sender<SyncJob>>>,
    /// Last sync times
    last_sync: RwLock<HashMap<String, DateTime<Utc>>>,
    /// Sync history
    sync_history: RwLock<Vec<SyncHistoryEntry>>,
    /// Event sender
    event_sender: broadcast::Sender<MobileEvent>,
    /// Configuration
    config: RwLock<MobileConfig>,
    /// Running flag
    running: RwLock<bool>,
}

/// Sync job
#[derive(Debug, Clone)]
pub struct SyncJob {
    /// Job ID
    pub id: String,
    /// Device ID
    pub device_id: String,
    /// Data type to sync
    pub data_type: SyncDataType,
    /// Priority
    pub priority: SyncPriority,
    /// Created at
    pub created_at: DateTime<Utc>,
}

/// Sync priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SyncPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Sync history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncHistoryEntry {
    /// Entry ID
    pub id: String,
    /// Device ID
    pub device_id: String,
    /// Sync type
    pub sync_type: SyncType,
    /// Success
    pub success: bool,
    /// Items synced
    pub items_synced: u64,
    /// Items failed
    pub items_failed: u64,
    /// Duration in ms
    pub duration_ms: u64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Error message
    pub error: Option<String>,
}

/// Sync types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncType {
    Full,
    Incremental,
    Delta,
    Manual,
}

impl MobileSync {
    /// Create a new sync engine
    pub async fn new(
        cloud_manager: Arc<CloudManager>,
        config: MobileConfig,
    ) -> Result<Self> {
        let (event_sender, _) = broadcast::channel(256);
        
        Ok(Self {
            cloud_manager,
            sync_queues: RwLock::new(HashMap::new()),
            last_sync: RwLock::new(HashMap::new()),
            sync_history: RwLock::new(Vec::new()),
            event_sender,
            config: RwLock::new(config),
            running: RwLock::new(false),
        })
    }
    
    /// Create with event sender
    pub async fn with_event_sender(
        cloud_manager: Arc<CloudManager>,
        config: MobileConfig,
        event_sender: broadcast::Sender<MobileEvent>,
    ) -> Result<Self> {
        Ok(Self {
            cloud_manager,
            sync_queues: RwLock::new(HashMap::new()),
            last_sync: RwLock::new(HashMap::new()),
            sync_history: RwLock::new(Vec::new()),
            event_sender,
            config: RwLock::new(config),
            running: RwLock::new(false),
        })
    }
    
    /// Start sync service
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = true;
        
        tracing::info!("Mobile sync service started");
        Ok(())
    }
    
    /// Stop sync service
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;
        
        // Clear queues
        let mut queues = self.sync_queues.write().await;
        queues.clear();
        
        tracing::info!("Mobile sync service stopped");
        Ok(())
    }
    
    /// Check if running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
    
    /// Sync specific device
    pub async fn sync_device(&self, device_id: &str) -> Result<SyncResult> {
        let start = std::time::Instant::now();
        
        let _ = self.event_sender.send(MobileEvent::SyncStarted(device_id.to_string()));
        
        // Get sync settings for device
        let settings = self.get_device_sync_settings(device_id).await?;
        
        // Perform sync for each enabled data type
        let mut total_synced = 0u64;
        let mut total_failed = 0u64;
        let mut errors = Vec::new();
        
        if settings.sync_bookmarks {
            match self.sync_data_type(device_id, SyncDataType::Bookmarks).await {
                Ok(count) => total_synced += count,
                Err(e) => {
                    total_failed += 1;
                    errors.push(format!("Bookmarks: {}", e));
                }
            }
        }
        
        if settings.sync_history {
            match self.sync_data_type(device_id, SyncDataType::History).await {
                Ok(count) => total_synced += count,
                Err(e) => {
                    total_failed += 1;
                    errors.push(format!("History: {}", e));
                }
            }
        }
        
        if settings.sync_tabs {
            match self.sync_data_type(device_id, SyncDataType::Tabs).await {
                Ok(count) => total_synced += count,
                Err(e) => {
                    total_failed += 1;
                    errors.push(format!("Tabs: {}", e));
                }
            }
        }
        
        if settings.sync_passwords {
            match self.sync_data_type(device_id, SyncDataType::Passwords).await {
                Ok(count) => total_synced += count,
                Err(e) => {
                    total_failed += 1;
                    errors.push(format!("Passwords: {}", e));
                }
            }
        }
        
        if settings.sync_settings {
            match self.sync_data_type(device_id, SyncDataType::Settings).await {
                Ok(count) => total_synced += count,
                Err(e) => {
                    total_failed += 1;
                    errors.push(format!("Settings: {}", e));
                }
            }
        }
        
        let duration_ms = start.elapsed().as_millis() as u64;
        let success = total_failed == 0;
        
        // Update last sync time
        self.update_last_sync(device_id).await;
        
        // Add to history
        let history_entry = SyncHistoryEntry {
            id: Uuid::new_v4().to_string(),
            device_id: device_id.to_string(),
            sync_type: SyncType::Incremental,
            success,
            items_synced: total_synced,
            items_failed: total_failed,
            duration_ms,
            timestamp: Utc::now(),
            error: if errors.is_empty() { None } else { Some(errors.join("; ")) },
        };
        
        self.add_history_entry(history_entry).await;
        
        let result = SyncResult {
            device_id: device_id.to_string(),
            success,
            items_synced: total_synced,
            items_failed: total_failed,
            duration_ms,
            error: if errors.is_empty() { None } else { Some(errors.join("; ")) },
            timestamp: Utc::now(),
        };
        
        if success {
            let _ = self.event_sender.send(MobileEvent::SyncCompleted(device_id.to_string()));
        } else {
            let _ = self.event_sender.send(MobileEvent::SyncFailed(
                device_id.to_string(),
                errors.join("; "),
            ));
        }
        
        Ok(result)
    }
    
    /// Sync all devices
    pub async fn sync_all(&self) -> Result<Vec<SyncResult>> {
        let queues = self.sync_queues.read().await;
        let device_ids: Vec<String> = queues.keys().cloned().collect();
        drop(queues);
        
        let mut results = Vec::new();
        
        for device_id in device_ids {
            let result = self.sync_device(&device_id).await?;
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// Sync specific data type
    async fn sync_data_type(&self, device_id: &str, data_type: SyncDataType) -> Result<u64> {
        // Get changes since last sync
        let changes = self.get_changes_since_last_sync(device_id, &data_type).await?;
        
        // Send to cloud
        self.upload_changes(device_id, &data_type, &changes).await?;
        
        // Download remote changes
        let remote_changes = self.download_changes(device_id, &data_type).await?;
        
        // Apply remote changes
        self.apply_changes(&remote_changes).await?;
        
        // Check for conflicts
        let conflicts = self.detect_conflicts(&changes, &remote_changes).await?;
        
        if !conflicts.is_empty() {
            self.resolve_conflicts(&conflicts).await?;
        }
        
        Ok(changes.len() as u64 + remote_changes.len() as u64)
    }
    
    /// Get changes since last sync
    async fn get_changes_since_last_sync(
        &self,
        device_id: &str,
        data_type: &SyncDataType,
    ) -> Result<Vec<SyncDataPacket>> {
        // Placeholder - would integrate with actual data stores
        Ok(Vec::new())
    }
    
    /// Upload changes to cloud
    async fn upload_changes(
        &self,
        device_id: &str,
        data_type: &SyncDataType,
        changes: &[SyncDataPacket],
    ) -> Result<()> {
        tracing::debug!("Uploading {} changes for {:?}", changes.len(), data_type);
        Ok(())
    }
    
    /// Download changes from cloud
    async fn download_changes(
        &self,
        device_id: &str,
        data_type: &SyncDataType,
    ) -> Result<Vec<SyncDataPacket>> {
        // Placeholder - would integrate with cloud storage
        Ok(Vec::new())
    }
    
    /// Apply changes locally
    async fn apply_changes(&self, changes: &[SyncDataPacket]) -> Result<()> {
        for change in changes {
            match change.operation {
                SyncOperation::Create => {
                    tracing::debug!("Creating: {:?}", change.data_type);
                }
                SyncOperation::Update => {
                    tracing::debug!("Updating: {:?}", change.data_type);
                }
                SyncOperation::Delete => {
                    tracing::debug!("Deleting: {:?}", change.data_type);
                }
                SyncOperation::Move => {
                    tracing::debug!("Moving: {:?}", change.data_type);
                }
            }
        }
        Ok(())
    }
    
    /// Detect sync conflicts
    async fn detect_conflicts(
        &self,
        local_changes: &[SyncDataPacket],
        remote_changes: &[SyncDataPacket],
    ) -> Result<Vec<SyncConflict>> {
        let mut conflicts = Vec::new();
        
        // Simple conflict detection - same ID modified both sides
        for local in local_changes {
            for remote in remote_changes {
                if local.id == remote.id && local.version != remote.version {
                    conflicts.push(SyncConflict {
                        id: Uuid::new_v4().to_string(),
                        data_type: local.data_type.clone(),
                        local_version: local.clone(),
                        remote_version: remote.clone(),
                        detected_at: Utc::now(),
                    });
                }
            }
        }
        
        Ok(conflicts)
    }
    
    /// Resolve sync conflicts
    async fn resolve_conflicts(&self, conflicts: &[SyncConflict]) -> Result<()> {
        for conflict in conflicts {
            // Default: latest wins
            tracing::warn!("Sync conflict detected for {:?}: {}", 
                conflict.data_type, conflict.id);
        }
        Ok(())
    }
    
    /// Register device for sync
    pub async fn register_device(&self, device_id: &str) -> Result<()> {
        let (tx, _rx) = mpsc::channel(100);
        
        let mut queues = self.sync_queues.write().await;
        queues.insert(device_id.to_string(), tx);
        
        tracing::info!("Device registered for sync: {}", device_id);
        Ok(())
    }
    
    /// Unregister device from sync
    pub async fn unregister_device(&self, device_id: &str) -> Result<()> {
        let mut queues = self.sync_queues.write().await;
        queues.remove(device_id);
        
        tracing::info!("Device unregistered from sync: {}", device_id);
        Ok(())
    }
    
    /// Queue sync job
    pub async fn queue_sync_job(&self, job: SyncJob) -> Result<()> {
        let queues = self.sync_queues.read().await;
        
        if let Some(sender) = queues.get(&job.device_id) {
            let _ = sender.send(job).await;
        }
        
        Ok(())
    }
    
    /// Get last sync time
    pub async fn last_sync_time(&self) -> Option<DateTime<Utc>> {
        let last_sync = self.last_sync.read().await;
        last_sync.values().max().copied()
    }
    
    /// Get last sync time for device
    pub async fn last_sync_for_device(&self, device_id: &str) -> Option<DateTime<Utc>> {
        let last_sync = self.last_sync.read().await;
        last_sync.get(device_id).copied()
    }
    
    /// Update last sync time
    async fn update_last_sync(&self, device_id: &str) {
        let mut last_sync = self.last_sync.write().await;
        last_sync.insert(device_id.to_string(), Utc::now());
    }
    
    /// Get sync history
    pub async fn get_sync_history(&self, limit: usize) -> Result<Vec<SyncHistoryEntry>> {
        let history = self.sync_history.read().await;
        Ok(history.iter().rev().take(limit).cloned().collect())
    }
    
    /// Get sync history for device
    pub async fn get_device_sync_history(
        &self,
        device_id: &str,
        limit: usize,
    ) -> Result<Vec<SyncHistoryEntry>> {
        let history = self.sync_history.read().await;
        Ok(history.iter()
            .filter(|e| e.device_id == device_id)
            .rev()
            .take(limit)
            .cloned()
            .collect())
    }
    
    /// Add history entry
    async fn add_history_entry(&self, entry: SyncHistoryEntry) {
        let mut history = self.sync_history.write().await;
        history.push(entry);
        
        // Keep only last 1000 entries
        if history.len() > 1000 {
            history.remove(0);
        }
    }
    
    /// Get device sync settings (placeholder)
    async fn get_device_sync_settings(&self, device_id: &str) -> Result<SyncSettings> {
        // Would fetch from device manager
        Ok(SyncSettings::default())
    }
    
    /// Update configuration
    pub async fn update_config(&self, config: MobileConfig) -> Result<()> {
        let mut current = self.config.write().await;
        *current = config;
        Ok(())
    }
    
    /// Get sync statistics
    pub async fn get_statistics(&self) -> Result<SyncStatistics> {
        let history = self.sync_history.read().await;
        let last_sync = self.last_sync.read().await;
        
        let total_syncs = history.len();
        let successful = history.iter().filter(|e| e.success).count();
        let failed = total_syncs - successful;
        
        let avg_duration = if total_syncs > 0 {
            history.iter().map(|e| e.duration_ms).sum::<u64>() / total_syncs as u64
        } else {
            0
        };
        
        Ok(SyncStatistics {
            total_syncs: total_syncs as u64,
            successful_syncs: successful as u64,
            failed_syncs: failed as u64,
            average_duration_ms: avg_duration,
            last_sync: last_sync.values().max().copied(),
            registered_devices: last_sync.len(),
        })
    }
}

/// Sync statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatistics {
    /// Total sync operations
    pub total_syncs: u64,
    /// Successful syncs
    pub successful_syncs: u64,
    /// Failed syncs
    pub failed_syncs: u64,
    /// Average duration in ms
    pub average_duration_ms: u64,
    /// Last sync time
    pub last_sync: Option<DateTime<Utc>>,
    /// Registered devices
    pub registered_devices: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sync_priority_order() {
        assert!(SyncPriority::Critical > SyncPriority::High);
        assert!(SyncPriority::High > SyncPriority::Normal);
        assert!(SyncPriority::Normal > SyncPriority::Low);
    }
    
    #[test]
    fn test_sync_job_creation() {
        let job = SyncJob {
            id: "job-1".to_string(),
            device_id: "device-1".to_string(),
            data_type: SyncDataType::Bookmarks,
            priority: SyncPriority::Normal,
            created_at: Utc::now(),
        };
        
        assert_eq!(job.id, "job-1");
        assert_eq!(job.priority, SyncPriority::Normal);
    }
}