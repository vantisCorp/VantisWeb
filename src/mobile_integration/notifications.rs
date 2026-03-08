//! Push Notification Manager
//! 
//! Manages push notifications to mobile devices.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use anyhow::Result;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

use super::{
    NotificationPayload, NotificationType, NotificationPriority,
    MobileEvent,
};

/// Push notification manager
pub struct PushNotificationManager {
    /// Pending notifications
    pending: RwLock<HashMap<String, PendingNotification>>,
    /// Notification history
    history: RwLock<Vec<NotificationRecord>>,
    /// Device tokens
    device_tokens: RwLock<HashMap<String, DeviceTokenInfo>>,
    /// Event sender
    event_sender: broadcast::Sender<MobileEvent>,
    /// Configuration
    config: RwLock<NotificationConfig>,
    /// Running flag
    running: RwLock<bool>,
}

/// Notification configuration
#[derive(Debug, Clone)]
pub struct NotificationConfig {
    /// Enable notifications
    pub enabled: bool,
    /// Maximum pending notifications per device
    pub max_pending_per_device: usize,
    /// Default TTL in seconds
    pub default_ttl: u32,
    /// Batch notifications
    pub batch_enabled: bool,
    /// Batch window in seconds
    pub batch_window: u32,
    /// Retry failed notifications
    pub retry_enabled: bool,
    /// Max retry attempts
    pub max_retries: u32,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_pending_per_device: 50,
            default_ttl: 86400, // 24 hours
            batch_enabled: true,
            batch_window: 5,
            retry_enabled: true,
            max_retries: 3,
        }
    }
}

/// Pending notification
#[derive(Debug, Clone)]
pub struct PendingNotification {
    /// Notification ID
    pub id: String,
    /// Device ID
    pub device_id: String,
    /// Payload
    pub payload: NotificationPayload,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Retry count
    pub retry_count: u32,
    /// Last error
    pub last_error: Option<String>,
}

/// Notification record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRecord {
    /// Record ID
    pub id: String,
    /// Device ID
    pub device_id: String,
    /// Payload
    pub payload: NotificationPayload,
    /// Status
    pub status: NotificationStatus,
    /// Sent at
    pub sent_at: DateTime<Utc>,
    /// Delivered at
    pub delivered_at: Option<DateTime<Utc>>,
    /// Opened at
    pub opened_at: Option<DateTime<Utc>>,
    /// Error
    pub error: Option<String>,
}

/// Notification status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationStatus {
    Pending,
    Sent,
    Delivered,
    Opened,
    Failed,
    Expired,
}

/// Device token info
#[derive(Debug, Clone)]
pub struct DeviceTokenInfo {
    /// Device ID
    pub device_id: String,
    /// Push token
    pub token: String,
    /// Platform
    pub platform: Platform,
    /// Token status
    pub status: TokenStatus,
    /// Registered at
    pub registered_at: DateTime<Utc>,
    /// Last used
    pub last_used: Option<DateTime<Utc>>,
}

/// Platform types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Platform {
    APNS,      // iOS
    FCM,       // Android Firebase
    HMS,       // Huawei
    WN,        // Windows Notification
}

/// Token status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenStatus {
    Active,
    Invalid,
    Expired,
    Unregistered,
}

impl PushNotificationManager {
    /// Create a new notification manager
    pub async fn new(event_sender: broadcast::Sender<MobileEvent>) -> Result<Self> {
        Ok(Self {
            pending: RwLock::new(HashMap::new()),
            history: RwLock::new(Vec::new()),
            device_tokens: RwLock::new(HashMap::new()),
            event_sender,
            config: RwLock::new(NotificationConfig::default()),
            running: RwLock::new(false),
        })
    }
    
    /// Start notification service
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = true;
        
        tracing::info!("Push notification service started");
        Ok(())
    }
    
    /// Stop notification service
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;
        
        tracing::info!("Push notification service stopped");
        Ok(())
    }
    
    /// Register device token
    pub async fn register_token(
        &self,
        device_id: &str,
        token: &str,
        platform: Platform,
    ) -> Result<()> {
        let mut tokens = self.device_tokens.write().await;
        
        tokens.insert(device_id.to_string(), DeviceTokenInfo {
            device_id: device_id.to_string(),
            token: token.to_string(),
            platform,
            status: TokenStatus::Active,
            registered_at: Utc::now(),
            last_used: None,
        });
        
        tracing::info!("Registered push token for device: {}", device_id);
        Ok(())
    }
    
    /// Unregister device token
    pub async fn unregister_token(&self, device_id: &str) -> Result<()> {
        let mut tokens = self.device_tokens.write().await;
        tokens.remove(device_id);
        
        tracing::info!("Unregistered push token for device: {}", device_id);
        Ok(())
    }
    
    /// Get token for device
    pub async fn get_token(&self, device_id: &str) -> Option<DeviceTokenInfo> {
        let tokens = self.device_tokens.read().await;
        tokens.get(device_id).cloned()
    }
    
    /// Send notification to device
    pub async fn send_to_device(
        &self,
        device_id: &str,
        notification: NotificationPayload,
    ) -> Result<()> {
        let config = self.config.read().await;
        
        if !config.enabled {
            return Ok(());
        }
        
        // Get device token
        let token_info = self.get_token(device_id).await;
        
        match token_info {
            Some(info) if info.status == TokenStatus::Active => {
                // Send notification
                let id = Uuid::new_v4().to_string();
                
                self.send_via_platform(&info, &id, &notification).await?;
                
                // Record in history
                let record = NotificationRecord {
                    id: id.clone(),
                    device_id: device_id.to_string(),
                    payload: notification.clone(),
                    status: NotificationStatus::Sent,
                    sent_at: Utc::now(),
                    delivered_at: None,
                    opened_at: None,
                    error: None,
                };
                
                self.add_to_history(record).await;
                
                // Update last used
                self.update_last_used(device_id).await;
                
                // Emit event
                let _ = self.event_sender.send(MobileEvent::NotificationSent(id));
            }
            Some(info) => {
                tracing::warn!("Device token not active: {:?} for device: {}", 
                    info.status, device_id);
            }
            None => {
                tracing::warn!("No push token registered for device: {}", device_id);
            }
        }
        
        Ok(())
    }
    
    /// Broadcast notification to all devices
    pub async fn broadcast(&self, notification: NotificationPayload) -> Result<()> {
        let tokens = self.device_tokens.read().await;
        let device_ids: Vec<String> = tokens.keys().cloned().collect();
        drop(tokens);
        
        for device_id in device_ids {
            let _ = self.send_to_device(&device_id, notification.clone()).await;
        }
        
        Ok(())
    }
    
    /// Send via platform-specific service
    async fn send_via_platform(
        &self,
        token_info: &DeviceTokenInfo,
        id: &str,
        notification: &NotificationPayload,
    ) -> Result<()> {
        match token_info.platform {
            Platform::APNS => self.send_via_apns(token_info, id, notification).await,
            Platform::FCM => self.send_via_fcm(token_info, id, notification).await,
            Platform::HMS => self.send_via_hms(token_info, id, notification).await,
            Platform::WN => self.send_via_wn(token_info, id, notification).await,
        }
    }
    
    /// Send via APNS (iOS)
    async fn send_via_apns(
        &self,
        token_info: &DeviceTokenInfo,
        _id: &str,
        notification: &NotificationPayload,
    ) -> Result<()> {
        // Placeholder for APNS integration
        tracing::debug!("Sending APNS notification to device: {}", token_info.device_id);
        
        // Would integrate with a2 library for APNS
        let apns_payload = APNSPayload {
            aps: APS {
                alert: Alert {
                    title: notification.title.clone(),
                    body: notification.body.clone(),
                },
                badge: None,
                sound: notification.priority == NotificationPriority::High,
            },
            data: notification.data.clone(),
        };
        
        // Serialize and send
        let _ = serde_json::to_string(&apns_payload)?;
        
        Ok(())
    }
    
    /// Send via FCM (Android)
    async fn send_via_fcm(
        &self,
        token_info: &DeviceTokenInfo,
        _id: &str,
        notification: &NotificationPayload,
    ) -> Result<()> {
        // Placeholder for FCM integration
        tracing::debug!("Sending FCM notification to device: {}", token_info.device_id);
        
        let fcm_payload = FCMPayload {
            to: token_info.token.clone(),
            notification: FCMNotification {
                title: notification.title.clone(),
                body: notification.body.clone(),
                icon: notification.icon.clone(),
            },
            data: notification.data.clone(),
            priority: match notification.priority {
                NotificationPriority::High => "high".to_string(),
                NotificationPriority::Normal => "normal".to_string(),
            },
        };
        
        let _ = serde_json::to_string(&fcm_payload)?;
        
        Ok(())
    }
    
    /// Send via HMS (Huawei)
    async fn send_via_hms(
        &self,
        token_info: &DeviceTokenInfo,
        _id: &str,
        notification: &NotificationPayload,
    ) -> Result<()> {
        tracing::debug!("Sending HMS notification to device: {}", token_info.device_id);
        Ok(())
    }
    
    /// Send via Windows Notification
    async fn send_via_wn(
        &self,
        token_info: &DeviceTokenInfo,
        _id: &str,
        notification: &NotificationPayload,
    ) -> Result<()> {
        tracing::debug!("Sending Windows notification to device: {}", token_info.device_id);
        Ok(())
    }
    
    /// Add to history
    async fn add_to_history(&self, record: NotificationRecord) {
        let mut history = self.history.write().await;
        history.push(record);
        
        // Keep only last 1000 entries
        if history.len() > 1000 {
            history.remove(0);
        }
    }
    
    /// Update last used timestamp
    async fn update_last_used(&self, device_id: &str) {
        let mut tokens = self.device_tokens.write().await;
        if let Some(info) = tokens.get_mut(device_id) {
            info.last_used = Some(Utc::now());
        }
    }
    
    /// Mark notification as delivered
    pub async fn mark_delivered(&self, notification_id: &str) -> Result<()> {
        let mut history = self.history.write().await;
        
        for record in history.iter_mut() {
            if record.id == notification_id {
                record.status = NotificationStatus::Delivered;
                record.delivered_at = Some(Utc::now());
                break;
            }
        }
        
        Ok(())
    }
    
    /// Mark notification as opened
    pub async fn mark_opened(&self, notification_id: &str) -> Result<()> {
        let mut history = self.history.write().await;
        
        for record in history.iter_mut() {
            if record.id == notification_id {
                record.status = NotificationStatus::Opened;
                record.opened_at = Some(Utc::now());
                break;
            }
        }
        
        Ok(())
    }
    
    /// Get notification history
    pub async fn get_history(&self, limit: usize) -> Result<Vec<NotificationRecord>> {
        let history = self.history.read().await;
        Ok(history.iter().rev().take(limit).cloned().collect())
    }
    
    /// Get device notification history
    pub async fn get_device_history(
        &self,
        device_id: &str,
        limit: usize,
    ) -> Result<Vec<NotificationRecord>> {
        let history = self.history.read().await;
        Ok(history.iter()
            .filter(|r| r.device_id == device_id)
            .rev()
            .take(limit)
            .cloned()
            .collect())
    }
    
    /// Get pending notifications
    pub async fn get_pending(&self) -> Result<Vec<PendingNotification>> {
        let pending = self.pending.read().await;
        Ok(pending.values().cloned().collect())
    }
    
    /// Clear pending notifications for device
    pub async fn clear_pending(&self, device_id: &str) -> Result<()> {
        let mut pending = self.pending.write().await;
        pending.retain(|_, n| n.device_id != device_id);
        Ok(())
    }
    
    /// Retry failed notifications
    pub async fn retry_failed(&self) -> Result<u32> {
        let config = self.config.read().await;
        
        if !config.retry_enabled {
            return Ok(0);
        }
        
        let pending = self.pending.read().await;
        let to_retry: Vec<_> = pending.values()
            .filter(|n| n.retry_count < config.max_retries)
            .cloned()
            .collect();
        drop(pending);
        
        let mut retried = 0u32;
        
        for notification in to_retry {
            if self.send_to_device(&notification.device_id, notification.payload.clone()).await.is_ok() {
                retried += 1;
            }
        }
        
        Ok(retried)
    }
    
    /// Clean expired notifications
    pub async fn clean_expired(&self) -> Result<u32> {
        let now = Utc::now();
        let mut history = self.history.write().await;
        
        let initial_len = history.len();
        history.retain(|r| {
            if r.status == NotificationStatus::Pending {
                let age = (now - r.sent_at).num_seconds() as u32;
                age < 86400 // Keep for 24 hours
            } else {
                true
            }
        });
        
        Ok((initial_len - history.len()) as u32)
    }
    
    /// Get statistics
    pub async fn get_statistics(&self) -> Result<NotificationStatistics> {
        let history = self.history.read().await;
        let tokens = self.device_tokens.read().await;
        let pending = self.pending.read().await;
        
        let total = history.len();
        let sent = history.iter().filter(|r| r.status == NotificationStatus::Sent).count();
        let delivered = history.iter().filter(|r| r.status == NotificationStatus::Delivered).count();
        let opened = history.iter().filter(|r| r.status == NotificationStatus::Opened).count();
        let failed = history.iter().filter(|r| r.status == NotificationStatus::Failed).count();
        
        Ok(NotificationStatistics {
            total_notifications: total as u64,
            sent: sent as u64,
            delivered: delivered as u64,
            opened: opened as u64,
            failed: failed as u64,
            pending: pending.len() as u64,
            registered_devices: tokens.len() as u64,
        })
    }
}

/// Notification statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationStatistics {
    pub total_notifications: u64,
    pub sent: u64,
    pub delivered: u64,
    pub opened: u64,
    pub failed: u64,
    pub pending: u64,
    pub registered_devices: u64,
}

/// APNS Payload structure
#[derive(Debug, Serialize)]
struct APNSPayload {
    aps: APS,
    data: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct APS {
    alert: Alert,
    badge: Option<u32>,
    sound: bool,
}

#[derive(Debug, Serialize)]
struct Alert {
    title: String,
    body: String,
}

/// FCM Payload structure
#[derive(Debug, Serialize)]
struct FCMPayload {
    to: String,
    notification: FCMNotification,
    data: serde_json::Value,
    priority: String,
}

#[derive(Debug, Serialize)]
struct FCMNotification {
    title: String,
    body: String,
    icon: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_notification_config_default() {
        let config = NotificationConfig::default();
        assert!(config.enabled);
        assert!(config.retry_enabled);
        assert_eq!(config.max_retries, 3);
    }
    
    #[test]
    fn test_platform_types() {
        assert_ne!(Platform::APNS, Platform::FCM);
        assert_ne!(Platform::FCM, Platform::HMS);
    }
    
    #[tokio::test]
    async fn test_notification_manager_creation() {
        let (tx, _rx) = broadcast::channel(16);
        let manager = PushNotificationManager::new(tx).await.unwrap();
        
        let stats = manager.get_statistics().await.unwrap();
        assert_eq!(stats.total_notifications, 0);
    }
}