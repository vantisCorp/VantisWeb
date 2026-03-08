//! Mobile Integration Data Models
//! 
//! Data structures for mobile companion app integration.

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

// Re-export main types from mod.rs
pub use super::{
    MobileDevice, DeviceType, DeviceCapabilities, SyncSettings,
    DevicePairingInfo, QRPairingData, NotificationPayload,
    NotificationType, NotificationPriority, RemoteCommand,
    RemoteCommandResult, SyncResult, MobileIntegrationStatus,
};

/// Mobile message envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileMessage {
    /// Message ID
    pub id: String,
    /// Message type
    pub message_type: MobileMessageType,
    /// Payload
    pub payload: serde_json::Value,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Source device ID
    pub source_device: String,
    /// Target device ID (or None for broadcast)
    pub target_device: Option<String>,
}

impl MobileMessage {
    pub fn new(
        message_type: MobileMessageType,
        payload: serde_json::Value,
        source_device: String,
        target_device: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            message_type,
            payload,
            timestamp: Utc::now(),
            source_device,
            target_device,
        }
    }
}

/// Mobile message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MobileMessageType {
    // Device management
    PairRequest,
    PairResponse,
    UnpairRequest,
    DeviceInfo,
    DeviceList,
    
    // Sync
    SyncRequest,
    SyncResponse,
    SyncData,
    SyncConflict,
    SyncComplete,
    
    // Notifications
    PushNotification,
    NotificationAck,
    NotificationAction,
    
    // Remote control
    RemoteCommand,
    RemoteResponse,
    RemoteStreaming,
    
    // Authentication
    AuthRequest,
    AuthResponse,
    AuthChallenge,
    BiometricAuth,
    
    // Tab management
    TabList,
    TabOpen,
    TabClose,
    TabSwitch,
    TabSend,
    
    // Bookmarks
    BookmarkList,
    BookmarkAdd,
    BookmarkRemove,
    BookmarkUpdate,
    
    // History
    HistoryList,
    HistorySearch,
    
    // Settings
    SettingsSync,
    SettingsUpdate,
    
    // Errors
    Error,
    
    // Heartbeat
    Heartbeat,
    HeartbeatAck,
}

/// Pairing request from mobile device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingRequest {
    /// Device name
    pub device_name: String,
    /// Device type
    pub device_type: DeviceType,
    /// Device model
    pub model: String,
    /// OS version
    pub os_version: String,
    /// App version
    pub app_version: String,
    /// Pairing code from QR
    pub pairing_code: String,
    /// Device capabilities
    pub capabilities: DeviceCapabilities,
    /// Push notification token
    pub push_token: Option<String>,
}

/// Pairing response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingResponse {
    /// Success status
    pub success: bool,
    /// Assigned device ID
    pub device_id: Option<String>,
    /// Error message
    pub error: Option<String>,
    /// Auth token for future communication
    pub auth_token: Option<String>,
    /// Browser name
    pub browser_name: String,
    /// Browser version
    pub browser_version: String,
}

/// Device authentication request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRequest {
    /// Device ID
    pub device_id: String,
    /// Auth token
    pub token: String,
    /// Request timestamp
    pub timestamp: DateTime<Utc>,
    /// Requested permissions
    pub permissions: Vec<Permission>,
}

/// Permission types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Permission {
    /// View tabs
    ViewTabs,
    /// Manage tabs
    ManageTabs,
    /// View bookmarks
    ViewBookmarks,
    /// Manage bookmarks
    ManageBookmarks,
    /// View history
    ViewHistory,
    /// View passwords (requires biometric)
    ViewPasswords,
    /// Manage settings
    ManageSettings,
    /// Remote control
    RemoteControl,
    /// Send notifications
    SendNotifications,
    /// Full access
    FullAccess,
}

/// Authentication response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    /// Success status
    pub success: bool,
    /// Session token
    pub session_token: Option<String>,
    /// Session expiration
    pub expires_at: Option<DateTime<Utc>>,
    /// Granted permissions
    pub permissions: Vec<Permission>,
    /// Error message
    pub error: Option<String>,
    /// Requires biometric auth
    pub requires_biometric: bool,
}

/// Biometric authentication request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiometricAuthRequest {
    /// Device ID
    pub device_id: String,
    /// Session token
    pub session_token: String,
    /// Biometric type
    pub biometric_type: BiometricType,
    /// Challenge response
    pub challenge_response: String,
}

/// Biometric types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BiometricType {
    Fingerprint,
    FaceId,
    Iris,
    Voice,
    Palm,
}

/// Tab information for mobile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileTabInfo {
    /// Tab ID
    pub id: u64,
    /// Tab title
    pub title: String,
    /// Tab URL
    pub url: String,
    /// Favicon URL
    pub favicon: Option<String>,
    /// Is active tab
    pub is_active: bool,
    /// Last accessed
    pub last_accessed: DateTime<Utc>,
    /// Tab group (if any)
    pub group: Option<String>,
    /// Preview image URL
    pub preview: Option<String>,
}

/// Tab list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabListResponse {
    /// List of tabs
    pub tabs: Vec<MobileTabInfo>,
    /// Active tab ID
    pub active_tab_id: u64,
    /// Total count
    pub total: usize,
}

/// Send tab request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendTabRequest {
    /// URL to send
    pub url: String,
    /// Title
    pub title: Option<String>,
    /// Source device ID
    pub source_device: String,
    /// Target device ID
    pub target_device: String,
}

/// Send tab response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendTabResponse {
    /// Success status
    pub success: bool,
    /// Error message
    pub error: Option<String>,
}

/// Bookmark info for mobile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileBookmarkInfo {
    /// Bookmark ID
    pub id: String,
    /// Title
    pub title: String,
    /// URL
    pub url: String,
    /// Folder path
    pub folder: Option<String>,
    /// Favicon URL
    pub favicon: Option<String>,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Modified at
    pub modified_at: DateTime<Utc>,
}

/// Bookmark list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkListResponse {
    /// List of bookmarks
    pub bookmarks: Vec<MobileBookmarkInfo>,
    /// Folders
    pub folders: Vec<String>,
    /// Total count
    pub total: usize,
}

/// History entry for mobile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileHistoryEntry {
    /// Entry ID
    pub id: String,
    /// Title
    pub title: String,
    /// URL
    pub url: String,
    /// Visit count
    pub visit_count: u32,
    /// Last visit
    pub last_visit: DateTime<Utc>,
    /// Favicon URL
    pub favicon: Option<String>,
}

/// History list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryListResponse {
    /// List of entries
    pub entries: Vec<MobileHistoryEntry>,
    /// Total count
    pub total: usize,
    /// Has more
    pub has_more: bool,
}

/// Sync data packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncDataPacket {
    /// Sync ID
    pub sync_id: String,
    /// Data type
    pub data_type: SyncDataType,
    /// Operation
    pub operation: SyncOperation,
    /// Data payload
    pub data: serde_json::Value,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Version for conflict detection
    pub version: u64,
}

/// Sync data types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncDataType {
    Bookmarks,
    History,
    Tabs,
    Passwords,
    Settings,
    Extensions,
    Cookies,
    FormData,
}

/// Sync operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncOperation {
    Create,
    Update,
    Delete,
    Move,
}

/// Sync conflict info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConflict {
    /// Conflict ID
    pub id: String,
    /// Data type
    pub data_type: SyncDataType,
    /// Local version
    pub local_version: SyncDataPacket,
    /// Remote version
    pub remote_version: SyncDataPacket,
    /// Detected at
    pub detected_at: DateTime<Utc>,
}

/// Settings sync data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsSyncData {
    /// Theme
    pub theme: Option<String>,
    /// Font size
    pub font_size: Option<u32>,
    /// Homepage URL
    pub homepage: Option<String>,
    /// Search engine
    pub search_engine: Option<String>,
    /// Privacy settings
    pub privacy: Option<PrivacySettings>,
    /// Notification settings
    pub notifications: Option<NotificationSettings>,
    /// Custom settings
    pub custom: serde_json::Value,
}

/// Privacy settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    /// Block trackers
    pub block_trackers: bool,
    /// Block ads
    pub block_ads: bool,
    /// Do not track
    pub do_not_track: bool,
    /// Cookie handling
    pub cookie_handling: CookieHandling,
}

/// Cookie handling modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CookieHandling {
    AllowAll,
    BlockThirdParty,
    BlockAll,
    IncognitoOnly,
}

/// Notification settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    /// Enabled
    pub enabled: bool,
    /// Sound
    pub sound: bool,
    /// Vibration
    pub vibration: bool,
    /// Show preview
    pub show_preview: bool,
}

/// Remote streaming session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteStreamingSession {
    /// Session ID
    pub id: String,
    /// Device ID
    pub device_id: String,
    /// Tab ID being streamed
    pub tab_id: u64,
    /// Started at
    pub started_at: DateTime<Utc>,
    /// Frame rate
    pub frame_rate: u32,
    /// Quality
    pub quality: StreamingQuality,
    /// Is active
    pub is_active: bool,
}

/// Streaming quality levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamingQuality {
    Low,
    Medium,
    High,
    Auto,
}

/// Error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileError {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Additional details
    pub details: Option<serde_json::Value>,
    /// Retry after seconds
    pub retry_after: Option<u32>,
}

impl MobileError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
            retry_after: None,
        }
    }
    
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
    
    pub fn with_retry_after(mut self, seconds: u32) -> Self {
        self.retry_after = Some(seconds);
        self
    }
}

/// Heartbeat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    /// Device ID
    pub device_id: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Battery level (0-100)
    pub battery_level: Option<u8>,
    /// Network type
    pub network_type: Option<NetworkType>,
    /// Is charging
    pub is_charging: Option<bool>,
}

/// Network types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkType {
    WiFi,
    Cellular4G,
    Cellular5G,
    Cellular3G,
    Offline,
}

/// Device state update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStateUpdate {
    /// Device ID
    pub device_id: String,
    /// Connection status
    pub connected: bool,
    /// Battery level
    pub battery_level: Option<u8>,
    /// Network type
    pub network_type: Option<NetworkType>,
    /// App state
    pub app_state: AppState,
}

/// App states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppState {
    Foreground,
    Background,
    Terminated,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mobile_message_creation() {
        let msg = MobileMessage::new(
            MobileMessageType::Heartbeat,
            serde_json::json!({}),
            "device-1".to_string(),
            None,
        );
        
        assert!(!msg.id.is_empty());
        assert_eq!(msg.source_device, "device-1");
    }
    
    #[test]
    fn test_mobile_error() {
        let error = MobileError::new("ERR_001", "Device not found")
            .with_retry_after(60);
        
        assert_eq!(error.code, "ERR_001");
        assert_eq!(error.retry_after, Some(60));
    }
    
    #[test]
    fn test_heartbeat() {
        let heartbeat = Heartbeat {
            device_id: "device-1".to_string(),
            timestamp: Utc::now(),
            battery_level: Some(85),
            network_type: Some(NetworkType::WiFi),
            is_charging: Some(true),
        };
        
        assert_eq!(heartbeat.battery_level, Some(85));
    }
}