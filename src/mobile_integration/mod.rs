//! Mobile Integration Module
//! 
//! Comprehensive mobile companion app integration for seamless cross-platform experience.
//! Provides device pairing, sync, push notifications, and remote control capabilities.

pub mod models;
pub mod device_manager;
pub mod sync;
pub mod notifications;
pub mod remote_control;
pub mod auth;
pub mod qr_bridge;

use std::sync::Arc;
use tokio::sync::{RwLock, mpsc, broadcast};
use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::cloud_integration::CloudManager;

pub use device_manager::MobileDeviceManager;
pub use sync::MobileSync;
pub use notifications::PushNotificationManager;
pub use remote_control::RemoteControlManager;
pub use auth::MobileAuthManager;
pub use qr_bridge::QRCodeBridge;

/// Main mobile integration manager
pub struct MobileIntegration {
    /// Device manager for paired mobile devices
    device_manager: Arc<MobileDeviceManager>,
    /// Sync engine for mobile data
    sync: Arc<MobileSync>,
    /// Push notification manager
    notifications: Arc<PushNotificationManager>,
    /// Remote control manager
    remote_control: Arc<RemoteControlManager>,
    /// Mobile authentication manager
    auth: Arc<MobileAuthManager>,
    /// QR code bridge for pairing
    qr_bridge: Arc<QRCodeBridge>,
    /// Event broadcaster
    event_sender: broadcast::Sender<MobileEvent>,
    /// Configuration
    config: MobileConfig,
}

/// Mobile integration configuration
#[derive(Debug, Clone)]
pub struct MobileConfig {
    /// Enable push notifications
    pub push_notifications_enabled: bool,
    /// Enable remote control
    pub remote_control_enabled: bool,
    /// Maximum paired devices
    pub max_devices: usize,
    /// Device pairing timeout in seconds
    pub pairing_timeout: u64,
    /// Sync interval in seconds
    pub sync_interval: u64,
    /// Enable biometric authentication
    pub biometric_auth_enabled: bool,
}

impl Default for MobileConfig {
    fn default() -> Self {
        Self {
            push_notifications_enabled: true,
            remote_control_enabled: true,
            max_devices: 10,
            pairing_timeout: 300,
            sync_interval: 60,
            biometric_auth_enabled: true,
        }
    }
}

/// Mobile event types
#[derive(Debug, Clone)]
pub enum MobileEvent {
    /// Device connected
    DeviceConnected(String),
    /// Device disconnected
    DeviceDisconnected(String),
    /// Device paired
    DevicePaired(String),
    /// Device unpaired
    DeviceUnpaired(String),
    /// Sync started
    SyncStarted(String),
    /// Sync completed
    SyncCompleted(String),
    /// Sync failed
    SyncFailed(String, String),
    /// Notification sent
    NotificationSent(String),
    /// Remote command received
    RemoteCommand(String, RemoteCommand),
    /// Pairing requested
    PairingRequested(String),
    /// Authentication successful
    AuthSuccess(String),
    /// Authentication failed
    AuthFailed(String),
}

/// Remote command types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RemoteCommand {
    /// Open URL
    OpenUrl(String),
    /// Close tab
    CloseTab(u64),
    /// Switch tab
    SwitchTab(u64),
    /// Refresh page
    Refresh(u64),
    /// Navigate back
    GoBack(u64),
    /// Navigate forward
    GoForward(u64),
    /// Bookmark page
    Bookmark(u64),
    /// Take screenshot
    Screenshot(u64),
    /// Fill form
    FillForm { tab_id: u64, selector: String, value: String },
    /// Execute script
    ExecuteScript { tab_id: u64, script: String },
    /// Get page info
    GetPageInfo(u64),
    /// List tabs
    ListTabs,
    /// Set theme
    SetTheme(String),
    /// Toggle feature
    ToggleFeature(String, bool),
}

use serde::{Serialize, Deserialize};

impl MobileIntegration {
    /// Create a new mobile integration instance
    pub async fn new(cloud_manager: Arc<CloudManager>) -> Result<Self> {
        let (event_sender, _) = broadcast::channel(256);
        
        let config = MobileConfig::default();
        
        let device_manager = Arc::new(MobileDeviceManager::new(config.clone(), event_sender.clone()).await?);
        let sync = Arc::new(MobileSync::new(cloud_manager.clone(), config.clone()).await?);
        let notifications = Arc::new(PushNotificationManager::new(event_sender.clone()).await?);
        let remote_control = Arc::new(RemoteControlManager::new(event_sender.clone()).await?);
        let auth = Arc::new(MobileAuthManager::new(config.clone()).await?);
        let qr_bridge = Arc::new(QRCodeBridge::new(event_sender.clone()).await?);
        
        Ok(Self {
            device_manager,
            sync,
            notifications,
            remote_control,
            auth,
            qr_bridge,
            event_sender,
            config,
        })
    }
    
    /// Create with custom configuration
    pub async fn with_config(
        cloud_manager: Arc<CloudManager>,
        config: MobileConfig,
    ) -> Result<Self> {
        let (event_sender, _) = broadcast::channel(256);
        
        let device_manager = Arc::new(MobileDeviceManager::new(config.clone(), event_sender.clone()).await?);
        let sync = Arc::new(MobileSync::new(cloud_manager.clone(), config.clone()).await?);
        let notifications = Arc::new(PushNotificationManager::new(event_sender.clone()).await?);
        let remote_control = Arc::new(RemoteControlManager::new(event_sender.clone()).await?);
        let auth = Arc::new(MobileAuthManager::new(config.clone()).await?);
        let qr_bridge = Arc::new(QRCodeBridge::new(event_sender.clone()).await?);
        
        Ok(Self {
            device_manager,
            sync,
            notifications,
            remote_control,
            auth,
            qr_bridge,
            event_sender,
            config,
        })
    }
    
    /// Start mobile integration services
    pub async fn start(&self) -> Result<()> {
        // Start device discovery
        self.device_manager.start_discovery().await?;
        
        // Start sync service
        self.sync.start().await?;
        
        // Start notification service
        self.notifications.start().await?;
        
        // Start remote control listener
        self.remote_control.start().await?;
        
        tracing::info!("Mobile integration services started");
        Ok(())
    }
    
    /// Stop mobile integration services
    pub async fn stop(&self) -> Result<()> {
        self.device_manager.stop_discovery().await?;
        self.sync.stop().await?;
        self.notifications.stop().await?;
        self.remote_control.stop().await?;
        
        tracing::info!("Mobile integration services stopped");
        Ok(())
    }
    
    /// Subscribe to mobile events
    pub fn subscribe(&self) -> broadcast::Receiver<MobileEvent> {
        self.event_sender.subscribe()
    }
    
    /// Get device manager
    pub fn device_manager(&self) -> Arc<MobileDeviceManager> {
        self.device_manager.clone()
    }
    
    /// Get sync engine
    pub fn sync(&self) -> Arc<MobileSync> {
        self.sync.clone()
    }
    
    /// Get notification manager
    pub fn notifications(&self) -> Arc<PushNotificationManager> {
        self.notifications.clone()
    }
    
    /// Get remote control manager
    pub fn remote_control(&self) -> Arc<RemoteControlManager> {
        self.remote_control.clone()
    }
    
    /// Get auth manager
    pub fn auth(&self) -> Arc<MobileAuthManager> {
        self.auth.clone()
    }
    
    /// Get QR bridge
    pub fn qr_bridge(&self) -> Arc<QRCodeBridge> {
        self.qr_bridge.clone()
    }
    
    /// Get current configuration
    pub fn config(&self) -> &MobileConfig {
        &self.config
    }
    
    /// Update configuration
    pub async fn update_config(&mut self, config: MobileConfig) -> Result<()> {
        self.config = config.clone();
        
        // Update components
        self.device_manager.update_config(config.clone()).await?;
        self.sync.update_config(config.clone()).await?;
        
        tracing::info!("Mobile integration configuration updated");
        Ok(())
    }
    
    /// Get all paired devices
    pub async fn get_paired_devices(&self) -> Result<Vec<MobileDevice>> {
        self.device_manager.get_paired_devices().await
    }
    
    /// Initiate pairing with QR code
    pub async fn initiate_qr_pairing(&self) -> Result<QRPairingData> {
        self.qr_bridge.generate_pairing_qr().await
    }
    
    /// Complete pairing with mobile device
    pub async fn complete_pairing(&self, device_info: DevicePairingInfo) -> Result<MobileDevice> {
        self.device_manager.pair_device(device_info).await
    }
    
    /// Unpair a device
    pub async fn unpair_device(&self, device_id: &str) -> Result<()> {
        self.device_manager.unpair_device(device_id).await
    }
    
    /// Send notification to device
    pub async fn send_notification(
        &self,
        device_id: &str,
        notification: NotificationPayload,
    ) -> Result<()> {
        self.notifications.send_to_device(device_id, notification).await
    }
    
    /// Broadcast notification to all devices
    pub async fn broadcast_notification(&self, notification: NotificationPayload) -> Result<()> {
        self.notifications.broadcast(notification).await
    }
    
    /// Execute remote command on browser
    pub async fn execute_remote_command(
        &self,
        device_id: &str,
        command: RemoteCommand,
    ) -> Result<RemoteCommandResult> {
        self.remote_control.execute_command(device_id, command).await
    }
    
    /// Sync data with device
    pub async fn sync_with_device(&self, device_id: &str) -> Result<SyncResult> {
        self.sync.sync_device(device_id).await
    }
    
    /// Sync all devices
    pub async fn sync_all_devices(&self) -> Result<Vec<SyncResult>> {
        self.sync.sync_all().await
    }
    
    /// Get mobile integration status
    pub async fn get_status(&self) -> Result<MobileIntegrationStatus> {
        let devices = self.device_manager.get_paired_devices().await?;
        let connected = devices.iter().filter(|d| d.is_connected).count();
        
        Ok(MobileIntegrationStatus {
            total_devices: devices.len(),
            connected_devices: connected,
            sync_enabled: self.config.sync_interval > 0,
            notifications_enabled: self.config.push_notifications_enabled,
            remote_control_enabled: self.config.remote_control_enabled,
            last_sync: self.sync.last_sync_time().await,
        })
    }
}

/// Mobile device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileDevice {
    /// Device unique identifier
    pub id: String,
    /// Device name
    pub name: String,
    /// Device type (iOS, Android)
    pub device_type: DeviceType,
    /// Device model
    pub model: String,
    /// Operating system version
    pub os_version: String,
    /// App version
    pub app_version: String,
    /// Is device currently connected
    pub is_connected: bool,
    /// Last seen timestamp
    pub last_seen: DateTime<Utc>,
    /// Paired timestamp
    pub paired_at: DateTime<Utc>,
    /// Device capabilities
    pub capabilities: DeviceCapabilities,
    /// Push notification token
    pub push_token: Option<String>,
    /// Sync settings
    pub sync_settings: SyncSettings,
}

/// Device type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceType {
    IOS,
    Android,
    IPadOS,
    Tablet,
    Wearable,
}

/// Device capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    /// Supports push notifications
    pub push_notifications: bool,
    /// Supports remote control
    pub remote_control: bool,
    /// Supports biometric auth
    pub biometric_auth: bool,
    /// Supports QR scanning
    pub qr_scanning: bool,
    /// Supports NFC
    pub nfc: bool,
    /// Supports Bluetooth
    pub bluetooth: bool,
    /// Supports location services
    pub location_services: bool,
    /// Maximum notification payload size
    pub max_payload_size: usize,
}

impl Default for DeviceCapabilities {
    fn default() -> Self {
        Self {
            push_notifications: true,
            remote_control: true,
            biometric_auth: true,
            qr_scanning: true,
            nfc: false,
            bluetooth: true,
            location_services: true,
            max_payload_size: 4096,
        }
    }
}

/// Sync settings per device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSettings {
    /// Sync bookmarks
    pub sync_bookmarks: bool,
    /// Sync history
    pub sync_history: bool,
    /// Sync tabs
    pub sync_tabs: bool,
    /// Sync passwords
    pub sync_passwords: bool,
    /// Sync settings
    pub sync_settings: bool,
    /// Sync extensions
    pub sync_extensions: bool,
    /// Sync on WiFi only
    pub wifi_only: bool,
    /// Sync interval in minutes
    pub sync_interval: u32,
}

impl Default for SyncSettings {
    fn default() -> Self {
        Self {
            sync_bookmarks: true,
            sync_history: true,
            sync_tabs: true,
            sync_passwords: false,
            sync_settings: true,
            sync_extensions: false,
            wifi_only: false,
            sync_interval: 15,
        }
    }
}

/// Device pairing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevicePairingInfo {
    /// Device name
    pub name: String,
    /// Device type
    pub device_type: DeviceType,
    /// Device model
    pub model: String,
    /// OS version
    pub os_version: String,
    /// App version
    pub app_version: String,
    /// Pairing code
    pub pairing_code: String,
    /// Push token (optional)
    pub push_token: Option<String>,
    /// Device capabilities
    pub capabilities: DeviceCapabilities,
}

/// QR code pairing data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRPairingData {
    /// Pairing code
    pub code: String,
    /// QR code data URL
    pub qr_data_url: String,
    /// Expiration time
    pub expires_at: DateTime<Utc>,
    /// Device ID that scanned (if any)
    pub scanned_by: Option<String>,
}

/// Notification payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPayload {
    /// Notification title
    pub title: String,
    /// Notification body
    pub body: String,
    /// Icon URL
    pub icon: Option<String>,
    /// URL to open on tap
    pub url: Option<String>,
    /// Additional data
    pub data: serde_json::Value,
    /// Notification type
    pub notification_type: NotificationType,
    /// Priority (normal or high)
    pub priority: NotificationPriority,
    /// Time to live in seconds
    pub ttl: u32,
}

/// Notification types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    /// Tab shared
    TabShared,
    /// Bookmark added
    BookmarkAdded,
    /// Download complete
    DownloadComplete,
    /// Sync complete
    SyncComplete,
    /// Security alert
    SecurityAlert,
    /// Update available
    UpdateAvailable,
    /// Reminder
    Reminder,
    /// Custom
    Custom(String),
}

/// Notification priority
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationPriority {
    Normal,
    High,
}

/// Remote command result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteCommandResult {
    /// Command that was executed
    pub command: RemoteCommand,
    /// Success status
    pub success: bool,
    /// Result data
    pub data: serde_json::Value,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Execution time in ms
    pub execution_time_ms: u64,
}

/// Sync result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    /// Device ID
    pub device_id: String,
    /// Success status
    pub success: bool,
    /// Items synced
    pub items_synced: u64,
    /// Items failed
    pub items_failed: u64,
    /// Sync duration in ms
    pub duration_ms: u64,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Mobile integration status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileIntegrationStatus {
    /// Total paired devices
    pub total_devices: usize,
    /// Currently connected devices
    pub connected_devices: usize,
    /// Sync enabled
    pub sync_enabled: bool,
    /// Notifications enabled
    pub notifications_enabled: bool,
    /// Remote control enabled
    pub remote_control_enabled: bool,
    /// Last sync time
    pub last_sync: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mobile_config_default() {
        let config = MobileConfig::default();
        assert!(config.push_notifications_enabled);
        assert!(config.remote_control_enabled);
        assert_eq!(config.max_devices, 10);
    }
    
    #[test]
    fn test_sync_settings_default() {
        let settings = SyncSettings::default();
        assert!(settings.sync_bookmarks);
        assert!(settings.sync_history);
        assert!(!settings.sync_passwords);
    }
    
    #[test]
    fn test_device_capabilities_default() {
        let caps = DeviceCapabilities::default();
        assert!(caps.push_notifications);
        assert!(caps.remote_control);
        assert!(caps.biometric_auth);
    }
}