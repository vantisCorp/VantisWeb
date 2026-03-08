//! Cross-Device Module
//! 
//! Device management and cross-device synchronization for
//! seamless browsing experience across multiple devices.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{CloudError, DeviceCommand};
use super::models::*;

/// Cross-Device Manager
pub struct CrossDeviceManager {
    registry: DeviceRegistry,
    sync_state: RwLock<DeviceSyncState>,
    command_queue: RwLock<Vec<PendingCommand>>,
    event_sender: tokio::sync::broadcast::Sender<DeviceEvent>,
}

/// Device Registry
pub struct DeviceRegistry {
    devices: RwLock<HashMap<String, Device>>,
    current_device: RwLock<Option<String>>,
}

/// Device representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    /// Unique device ID
    pub id: String,
    /// Device name
    pub name: String,
    /// Device type
    pub device_type: DeviceType,
    /// Operating system
    pub os: String,
    /// Browser version
    pub version: String,
    /// Last seen timestamp
    pub last_seen: DateTime<Utc>,
    /// Registration date
    pub registered_at: DateTime<Utc>,
    /// Device capabilities
    pub capabilities: DeviceCapabilities,
    /// Push notification token
    pub push_token: Option<String>,
    /// Device status
    pub status: DeviceStatus,
    /// Sync settings
    pub sync_settings: DeviceSyncSettings,
}

/// Device capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    /// Supports push notifications
    pub push_notifications: bool,
    /// Supports WebRTC
    pub webrtc: bool,
    /// Supports background sync
    pub background_sync: bool,
    /// Supports screen sharing
    pub screen_sharing: bool,
    /// Maximum storage (MB)
    pub storage_mb: usize,
    /// Screen resolution
    pub screen_resolution: Option<(u32, u32)>,
    /// Has camera
    pub has_camera: bool,
    /// Has microphone
    pub has_microphone: bool,
}

/// Device status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceStatus {
    Online,
    Offline,
    Syncing,
    Busy,
    Suspended,
}

/// Device sync settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSyncSettings {
    /// Auto-sync enabled
    pub auto_sync: bool,
    /// Sync history
    pub sync_history: bool,
    /// Sync bookmarks
    pub sync_bookmarks: bool,
    /// Sync passwords
    pub sync_passwords: bool,
    /// Sync extensions
    pub sync_extensions: bool,
    /// Sync settings
    pub sync_settings: bool,
    /// Sync tabs
    pub sync_tabs: bool,
}

impl Default for DeviceSyncSettings {
    fn default() -> Self {
        Self {
            auto_sync: true,
            sync_history: true,
            sync_bookmarks: true,
            sync_passwords: false,
            sync_extensions: true,
            sync_settings: true,
            sync_tabs: true,
        }
    }
}

/// Device sync state
#[derive(Debug, Default)]
struct DeviceSyncState {
    last_sync: Option<DateTime<Utc>>,
    sync_in_progress: bool,
    pending_items: usize,
}

/// Pending command to send to device
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PendingCommand {
    id: String,
    device_id: String,
    command: DeviceCommand,
    created_at: DateTime<Utc>,
    attempts: usize,
    max_attempts: usize,
}

/// Device event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceEvent {
    DeviceRegistered { device: Device },
    DeviceRemoved { device_id: String },
    DeviceOnline { device_id: String },
    DeviceOffline { device_id: String },
    CommandSent { device_id: String, command: String },
    CommandFailed { device_id: String, command: String, error: String },
    SyncStarted { device_id: String },
    SyncCompleted { device_id: String },
    TabReceived { device_id: String, url: String },
}

/// Device pairing
pub struct DevicePairing {
    code: RwLock<Option<String>>,
    expires_at: RwLock<Option<DateTime<Utc>>>,
}

/// Send Tab functionality
pub struct SendTabManager {
    pending_tabs: RwLock<Vec<PendingTab>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PendingTab {
    id: String,
    from_device: String,
    to_device: String,
    url: String,
    title: String,
    sent_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

/// Device activity
pub struct DeviceActivityTracker {
    activities: RwLock<HashMap<String, Vec<Activity>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub device_id: String,
    pub activity_type: ActivityType,
    pub timestamp: DateTime<Utc>,
    pub details: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityType {
    TabOpened,
    TabClosed,
    PageVisited,
    BookmarkAdded,
    DownloadStarted,
    DownloadCompleted,
    SessionStarted,
    SessionEnded,
}

/// Device notification manager
pub struct DeviceNotificationManager {
    notifications: RwLock<HashMap<String, Vec<DeviceNotification>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceNotification {
    pub id: String,
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
    pub url: Option<String>,
    pub sent_at: DateTime<Utc>,
    pub read: bool,
}

impl CrossDeviceManager {
    pub fn new() -> Self {
        let (event_sender, _) = tokio::sync::broadcast::channel(1000);
        
        Self {
            registry: DeviceRegistry::new(),
            sync_state: RwLock::new(DeviceSyncState::default()),
            command_queue: RwLock::new(Vec::new()),
            event_sender,
        }
    }

    /// Register current device
    pub async fn register_current_device(&self) -> Result<Device, CloudError> {
        let device = Device {
            id: Uuid::new_v4().to_string(),
            name: "Current Device".to_string(),
            device_type: DeviceType::Desktop,
            os: std::env::consts::OS.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            last_seen: Utc::now(),
            registered_at: Utc::now(),
            capabilities: DeviceCapabilities::detect(),
            push_token: None,
            status: DeviceStatus::Online,
            sync_settings: DeviceSyncSettings::default(),
        };
        
        self.registry.register(device.clone()).await?;
        
        let _ = self.event_sender.send(DeviceEvent::DeviceRegistered {
            device: device.clone(),
        });
        
        Ok(device)
    }

    /// Get all registered devices
    pub async fn list(&self) -> Vec<Device> {
        self.registry.list().await
    }

    /// Get device count
    pub async fn count(&self) -> usize {
        self.registry.count().await
    }

    /// Get a specific device
    pub async fn get(&self, device_id: &str) -> Option<Device> {
        self.registry.get(device_id).await
    }

    /// Remove a device
    pub async fn remove(&self, device_id: &str) -> Result<(), CloudError> {
        self.registry.remove(device_id).await?;
        
        let _ = self.event_sender.send(DeviceEvent::DeviceRemoved {
            device_id: device_id.to_string(),
        });
        
        Ok(())
    }

    /// Send a command to a device
    pub async fn send_command(&self, device_id: &str, command: DeviceCommand) -> Result<(), CloudError> {
        let device = self.registry.get(device_id).await
            .ok_or_else(|| CloudError::NotFound(format!("Device {} not found", device_id)))?;
        
        if device.status == DeviceStatus::Offline {
            // Queue the command for later
            let pending = PendingCommand {
                id: Uuid::new_v4().to_string(),
                device_id: device_id.to_string(),
                command: command.clone(),
                created_at: Utc::now(),
                attempts: 0,
                max_attempts: 3,
            };
            
            let mut queue = self.command_queue.write().await;
            queue.push(pending);
            
            return Ok(());
        }
        
        // Send the command immediately
        self.execute_command(device_id, &command).await?;
        
        let command_name = format!("{:?}", command);
        let _ = self.event_sender.send(DeviceEvent::CommandSent {
            device_id: device_id.to_string(),
            command: command_name,
        });
        
        Ok(())
    }

    /// Send a tab to another device
    pub async fn send_tab(&self, device_id: &str, url: &str, title: &str) -> Result<(), CloudError> {
        self.send_command(device_id, DeviceCommand::SendTab {
            url: url.to_string(),
            title: title.to_string(),
        }).await
    }

    /// Generate pairing code
    pub async fn generate_pairing_code(&self) -> Result<String, CloudError> {
        let code = Self::generate_code();
        
        let _ = self.event_sender.send(DeviceEvent::DeviceRegistered {
            device: Device {
                id: "pending".to_string(),
                name: "Pending Device".to_string(),
                device_type: DeviceType::Other,
                os: String::new(),
                version: String::new(),
                last_seen: Utc::now(),
                registered_at: Utc::now(),
                capabilities: DeviceCapabilities::default(),
                push_token: None,
                status: DeviceStatus::Offline,
                sync_settings: DeviceSyncSettings::default(),
            },
        });
        
        Ok(code)
    }

    /// Pair with a device using code
    pub async fn pair_with_code(&self, _code: &str) -> Result<Device, CloudError> {
        // Would verify the code and pair the device
        Err(CloudError::DeviceError("Pairing not implemented".to_string()))
    }

    /// Sync with a device
    pub async fn sync(&self, device_id: &str) -> Result<(), CloudError> {
        let mut state = self.sync_state.write().await;
        state.sync_in_progress = true;
        drop(state);
        
        let _ = self.event_sender.send(DeviceEvent::SyncStarted {
            device_id: device_id.to_string(),
        });
        
        // Perform sync
        // Would sync actual data
        
        let mut state = self.sync_state.write().await;
        state.sync_in_progress = false;
        state.last_sync = Some(Utc::now());
        
        let _ = self.event_sender.send(DeviceEvent::SyncCompleted {
            device_id: device_id.to_string(),
        });
        
        Ok(())
    }

    /// Get sync status
    pub async fn get_sync_status(&self) -> (bool, Option<DateTime<Utc>>) {
        let state = self.sync_state.read().await;
        (state.sync_in_progress, state.last_sync)
    }

    /// Update device status
    pub async fn update_status(&self, device_id: &str, status: DeviceStatus) {
        self.registry.update_status(device_id, status.clone()).await;
        
        let event = if status == DeviceStatus::Online {
            DeviceEvent::DeviceOnline { device_id: device_id.to_string() }
        } else {
            DeviceEvent::DeviceOffline { device_id: device_id.to_string() }
        };
        
        let _ = self.event_sender.send(event);
    }

    /// Subscribe to device events
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<DeviceEvent> {
        self.event_sender.subscribe()
    }

    // Private methods
    async fn execute_command(&self, device_id: &str, command: &DeviceCommand) -> Result<(), CloudError> {
        // Would send command via push notification or WebSocket
        match command {
            DeviceCommand::OpenUrl { url } => {
                // Would open URL on the device
            }
            DeviceCommand::CloseTab { tab_id } => {
                // Would close tab on the device
            }
            DeviceCommand::Notify { title, message } => {
                // Would send notification
            }
            DeviceCommand::RequestSync => {
                // Would trigger sync
            }
            DeviceCommand::Lock => {
                // Would lock browser
            }
            DeviceCommand::ClearCache => {
                // Would clear cache
            }
            DeviceCommand::SendTab { url, title } => {
                let _ = self.event_sender.send(DeviceEvent::TabReceived {
                    device_id: device_id.to_string(),
                    url: url.clone(),
                });
            }
            DeviceCommand::Custom { command, payload } => {
                // Handle custom command
            }
        }
        
        Ok(())
    }

    fn generate_code() -> String {
        use std::iter;
        const CHARSET: &[u8] = b"0123456789";
        let mut rng = rand::thread_rng();
        (0..6)
            .map(|_| {
                let idx = (rng.next_u32() as usize) % CHARSET.len();
                CHARSET[idx] as char
            })
            .collect()
    }
}

impl DeviceRegistry {
    pub fn new() -> Self {
        Self {
            devices: RwLock::new(HashMap::new()),
            current_device: RwLock::new(None),
        }
    }

    pub async fn register(&self, device: Device) -> Result<(), CloudError> {
        let mut devices = self.devices.write().await;
        
        // Limit number of devices
        if devices.len() >= 10 {
            return Err(CloudError::DeviceError("Maximum devices reached".to_string()));
        }
        
        let device_id = device.id.clone();
        devices.insert(device_id.clone(), device);
        
        let mut current = self.current_device.write().await;
        if current.is_none() {
            *current = Some(device_id);
        }
        
        Ok(())
    }

    pub async fn get(&self, device_id: &str) -> Option<Device> {
        self.devices.read().await.get(device_id).cloned()
    }

    pub async fn list(&self) -> Vec<Device> {
        self.devices.read().await.values().cloned().collect()
    }

    pub async fn count(&self) -> usize {
        self.devices.read().await.len()
    }

    pub async fn remove(&self, device_id: &str) -> Result<(), CloudError> {
        let mut devices = self.devices.write().await;
        
        if devices.remove(device_id).is_none() {
            return Err(CloudError::NotFound(format!("Device {} not found", device_id)));
        }
        
        Ok(())
    }

    pub async fn update_status(&self, device_id: &str, status: DeviceStatus) {
        let mut devices = self.devices.write().await;
        if let Some(device) = devices.get_mut(device_id) {
            device.status = status;
            device.last_seen = Utc::now();
        }
    }

    pub async fn get_current(&self) -> Option<Device> {
        let current_id = self.current_device.read().await.clone();
        if let Some(id) = current_id {
            self.get(&id).await
        } else {
            None
        }
    }
}

impl DeviceCapabilities {
    fn detect() -> Self {
        Self {
            push_notifications: true,
            webrtc: true,
            background_sync: true,
            screen_sharing: true,
            storage_mb: 100,
            screen_resolution: Some((1920, 1080)),
            has_camera: true,
            has_microphone: true,
        }
    }
}

impl Default for DeviceCapabilities {
    fn default() -> Self {
        Self::detect()
    }
}

impl SendTabManager {
    pub fn new() -> Self {
        Self {
            pending_tabs: RwLock::new(Vec::new()),
        }
    }

    pub async fn send(&self, from: &str, to: &str, url: &str, title: &str) -> Result<(), CloudError> {
        let tab = PendingTab {
            id: Uuid::new_v4().to_string(),
            from_device: from.to_string(),
            to_device: to.to_string(),
            url: url.to_string(),
            title: title.to_string(),
            sent_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::hours(24),
        };
        
        let mut pending = self.pending_tabs.write().await;
        pending.push(tab);
        
        Ok(())
    }

    pub async fn receive(&self, device_id: &str) -> Vec<PendingTab> {
        let mut pending = self.pending_tabs.write().await;
        let (received, remaining): (Vec<_>, Vec<_>) = pending
            .drain(..)
            .partition(|t| t.to_device == device_id);
        
        *pending = remaining;
        received
    }

    pub async fn clear_expired(&self) {
        let mut pending = self.pending_tabs.write().await;
        let now = Utc::now();
        pending.retain(|t| t.expires_at > now);
    }
}

impl DeviceActivityTracker {
    pub fn new() -> Self {
        Self {
            activities: RwLock::new(HashMap::new()),
        }
    }

    pub async fn record(&self, device_id: &str, activity: Activity) {
        let mut activities = self.activities.write().await;
        activities
            .entry(device_id.to_string())
            .or_insert_with(Vec::new)
            .push(activity);
        
        // Keep only last 100 activities per device
        if let Some(device_activities) = activities.get_mut(device_id) {
            if device_activities.len() > 100 {
                device_activities.remove(0);
            }
        }
    }

    pub async fn get_activities(&self, device_id: &str) -> Vec<Activity> {
        self.activities.read().await
            .get(device_id)
            .cloned()
            .unwrap_or_default()
    }
}

impl DeviceNotificationManager {
    pub fn new() -> Self {
        Self {
            notifications: RwLock::new(HashMap::new()),
        }
    }

    pub async fn send(&self, device_id: &str, notification: DeviceNotification) {
        let mut notifications = self.notifications.write().await;
        notifications
            .entry(device_id.to_string())
            .or_insert_with(Vec::new)
            .push(notification);
    }

    pub async fn get_unread(&self, device_id: &str) -> Vec<DeviceNotification> {
        self.notifications.read().await
            .get(device_id)
            .map(|n| n.iter().filter(|n| !n.read).cloned().collect())
            .unwrap_or_default()
    }

    pub async fn mark_read(&self, device_id: &str, notification_id: &str) {
        let mut notifications = self.notifications.write().await;
        if let Some(device_notifications) = notifications.get_mut(device_id) {
            for notification in device_notifications {
                if notification.id == notification_id {
                    notification.read = true;
                }
            }
        }
    }
}

// Rand implementation for code generation
mod rand {
    use std::cell::Cell;
    
    pub struct ThreadRng(Cell<u32>);
    
    pub fn thread_rng() -> ThreadRng {
        ThreadRng(Cell::new(1))
    }
    
    impl ThreadRng {
        pub fn next_u32(&self) -> u32 {
            let mut state = self.0.get();
            state = state.wrapping_mul(1103515245).wrapping_add(12345);
            self.0.set(state);
            state
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_device() {
        let manager = CrossDeviceManager::new();
        let device = manager.register_current_device().await.unwrap();
        
        assert!(!device.id.is_empty());
        assert_eq!(device.status, DeviceStatus::Online);
    }

    #[tokio::test]
    async fn test_list_devices() {
        let manager = CrossDeviceManager::new();
        manager.register_current_device().await.unwrap();
        
        let devices = manager.list().await;
        assert_eq!(devices.len(), 1);
    }

    #[tokio::test]
    async fn test_send_tab() {
        let manager = CrossDeviceManager::new();
        let device = manager.register_current_device().await.unwrap();
        
        // Create another device
        let device2 = Device {
            id: "device2".to_string(),
            name: "Device 2".to_string(),
            device_type: DeviceType::Mobile,
            os: "iOS".to_string(),
            version: "1.0.0".to_string(),
            last_seen: Utc::now(),
            registered_at: Utc::now(),
            capabilities: DeviceCapabilities::default(),
            push_token: None,
            status: DeviceStatus::Online,
            sync_settings: DeviceSyncSettings::default(),
        };
        manager.registry.register(device2).await.unwrap();
        
        let result = manager.send_tab("device2", "https://example.com", "Example").await;
        assert!(result.is_ok());
    }
}