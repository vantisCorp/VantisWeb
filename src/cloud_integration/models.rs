//! Cloud Integration Models
//! 
//! Data structures for cloud integration features.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Cloud status summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudStatus {
    /// Connection state
    pub connection: super::ConnectionState,
    /// Storage usage
    pub storage_usage: StorageUsage,
    /// Sync status
    pub sync_status: SyncStatusInfo,
    /// Active collaborative sessions
    pub active_sessions: usize,
    /// Connected devices count
    pub connected_devices: usize,
}

/// Storage usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageUsage {
    /// Used space in bytes
    pub used_bytes: u64,
    /// Total quota in bytes
    pub quota_bytes: u64,
    /// File count
    pub file_count: usize,
    /// By category
    pub by_category: HashMap<String, u64>,
}

impl StorageUsage {
    pub fn usage_percent(&self) -> f32 {
        if self.quota_bytes == 0 {
            return 0.0;
        }
        (self.used_bytes as f64 / self.quota_bytes as f64 * 100.0) as f32
    }

    pub fn available_bytes(&self) -> u64 {
        self.quota_bytes.saturating_sub(self.used_bytes)
    }
}

/// Sync status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatusInfo {
    /// Last sync time
    pub last_sync: Option<DateTime<Utc>>,
    /// Items pending upload
    pub pending_upload: usize,
    /// Items pending download
    pub pending_download: usize,
    /// Conflicts awaiting resolution
    pub conflicts: usize,
    /// Current sync progress (0-100)
    pub progress: f32,
    /// Sync speed in KB/s
    pub speed_kbps: f32,
    /// Status message
    pub status_message: String,
}

/// Sync result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    /// Items uploaded
    pub uploaded: usize,
    /// Items downloaded
    pub downloaded: usize,
    /// Conflicts detected
    pub conflicts: Vec<SyncConflict>,
    /// Errors encountered
    pub errors: Vec<SyncError>,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Bytes transferred
    pub bytes_transferred: u64,
}

/// Sync conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConflict {
    /// Conflict ID
    pub id: String,
    /// Item path
    pub path: String,
    /// Local version info
    pub local: ItemVersion,
    /// Remote version info
    pub remote: ItemVersion,
    /// Conflict type
    pub conflict_type: ConflictType,
    /// Detected at
    pub detected_at: DateTime<Utc>,
}

/// Item version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemVersion {
    /// Version ID
    pub version: String,
    /// Last modified
    pub modified_at: DateTime<Utc>,
    /// Size in bytes
    pub size: u64,
    /// Checksum
    pub checksum: String,
    /// Modified by
    pub modified_by: Option<String>,
}

/// Conflict type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictType {
    BothModified,
    LocalDeletedRemoteModified,
    LocalModifiedRemoteDeleted,
    BothDeleted,
    TypeMismatch,
}

/// Sync error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncError {
    /// Error ID
    pub id: String,
    /// Item path
    pub path: String,
    /// Error message
    pub message: String,
    /// Error code
    pub code: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Retry count
    pub retry_count: usize,
}

/// Backup information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    /// Backup ID
    pub id: String,
    /// Creation time
    pub created_at: DateTime<Utc>,
    /// Size in bytes
    pub size: u64,
    /// Number of items
    pub item_count: usize,
    /// Backup type
    pub backup_type: BackupType,
    /// Storage location
    pub location: String,
    /// Checksum for verification
    pub checksum: String,
    /// Encryption status
    pub encrypted: bool,
    /// Compression status
    pub compressed: bool,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Backup type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BackupType {
    Full,
    Incremental,
    Differential,
    Scheduled,
    Manual,
}

/// Backup manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    /// Manifest version
    pub version: String,
    /// Backup ID
    pub backup_id: String,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Application version
    pub app_version: String,
    /// Items in backup
    pub items: Vec<BackupItem>,
    /// Total size
    pub total_size: u64,
    /// Checksum
    pub checksum: String,
}

/// Item in backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupItem {
    /// Original path
    pub path: String,
    /// Size in bytes
    pub size: u64,
    /// Checksum
    pub checksum: String,
    /// Last modified
    pub modified_at: DateTime<Utc>,
    /// Item type
    pub item_type: BackupItemType,
    /// Compressed size
    pub compressed_size: u64,
}

/// Backup item type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BackupItemType {
    File,
    Directory,
    Database,
    Configuration,
    Cache,
    Bookmark,
    History,
    Extension,
}

/// Restore result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreResult {
    /// Items restored
    pub items_restored: usize,
    /// Items skipped
    pub items_skipped: usize,
    /// Errors encountered
    pub errors: Vec<RestoreError>,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Bytes restored
    pub bytes_restored: u64,
}

/// Restore error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreError {
    /// Item path
    pub path: String,
    /// Error message
    pub message: String,
    /// Whether it was skipped
    pub skipped: bool,
}

/// Collaborative session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Session ID
    pub id: String,
    /// Session name
    pub name: String,
    /// Owner ID
    pub owner_id: String,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Expires at
    pub expires_at: Option<DateTime<Utc>>,
    /// Participant count
    pub participant_count: usize,
    /// Active tabs being shared
    pub shared_tabs: Vec<SharedTab>,
    /// Session settings
    pub settings: SessionSettings,
    /// Invitation code
    pub invite_code: String,
}

/// Shared tab in session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedTab {
    /// Tab ID
    pub id: String,
    /// URL
    pub url: String,
    /// Title
    pub title: String,
    /// Shared by
    pub shared_by: String,
    /// Shared at
    pub shared_at: DateTime<Utc>,
    /// Cursor position
    pub cursor_position: Option<CursorPosition>,
    /// Annotations
    pub annotations: Vec<Annotation>,
}

/// Cursor position for collaborative browsing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    pub x: f32,
    pub y: f32,
    pub page_x: f32,
    pub page_y: f32,
    pub selector: Option<String>,
}

/// Annotation on shared content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub id: String,
    pub annotation_type: AnnotationType,
    pub content: String,
    pub position: AnnotationPosition,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub color: Option<String>,
}

/// Annotation type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnnotationType {
    Highlight,
    Comment,
    Drawing,
    Sticker,
    Arrow,
    Text,
}

/// Annotation position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationPosition {
    pub x: f32,
    pub y: f32,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub selector: Option<String>,
}

/// Session settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSettings {
    /// Allow anyone to join
    pub open_join: bool,
    /// Maximum participants
    pub max_participants: usize,
    /// Allow screen sharing
    pub screen_sharing: bool,
    /// Allow annotations
    pub annotations: bool,
    /// Allow chat
    pub chat_enabled: bool,
    /// Auto-close when owner leaves
    pub auto_close: bool,
}

/// Participant information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantInfo {
    /// Participant ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Avatar URL
    pub avatar: Option<String>,
    /// Role
    pub role: ParticipantRole,
    /// Joined at
    pub joined_at: DateTime<Utc>,
    /// Last active
    pub last_active: DateTime<Utc>,
    /// Is online
    pub online: bool,
    /// Device type
    pub device_type: DeviceType,
}

/// Participant role
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParticipantRole {
    Owner,
    Admin,
    Editor,
    Viewer,
}

/// Device type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceType {
    Desktop,
    Mobile,
    Tablet,
    TV,
    Other,
}

/// Device command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceCommand {
    /// Open URL on device
    OpenUrl { url: String },
    /// Close tab on device
    CloseTab { tab_id: String },
    /// Send a notification
    Notify { title: String, message: String },
    /// Request sync
    RequestSync,
    /// Lock the browser
    Lock,
    /// Clear cache
    ClearCache,
    /// Send tab to device
    SendTab { url: String, title: String },
    /// Custom command
    Custom { command: String, payload: String },
}

/// File metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    /// File ID
    pub id: String,
    /// Name
    pub name: String,
    /// Path
    pub path: String,
    /// Size in bytes
    pub size: u64,
    /// MIME type
    pub mime_type: String,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Modified at
    pub modified_at: DateTime<Utc>,
    /// ETag for caching
    pub etag: String,
    /// Is directory
    pub is_directory: bool,
    /// Version ID
    pub version_id: String,
    /// Share status
    pub shared: bool,
    /// Custom metadata
    pub custom_metadata: HashMap<String, String>,
}

/// Upload request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadRequest {
    /// File path
    pub path: String,
    /// Content type
    pub content_type: String,
    /// Size
    pub size: u64,
    /// Overwrite existing
    pub overwrite: bool,
    /// Custom metadata
    pub metadata: HashMap<String, String>,
    /// Parent folder ID
    pub parent_id: Option<String>,
}

/// Upload progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadProgress {
    /// Upload ID
    pub upload_id: String,
    /// Bytes uploaded
    pub bytes_uploaded: u64,
    /// Total bytes
    pub total_bytes: u64,
    /// Progress percentage
    pub progress: f32,
    /// Speed in bytes/second
    pub speed_bps: u64,
    /// Status
    pub status: UploadStatus,
}

/// Upload status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UploadStatus {
    Pending,
    Uploading,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

/// Download request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRequest {
    /// File ID or path
    pub file_id: String,
    /// Destination path
    pub destination: String,
    /// Version to download
    pub version: Option<String>,
    /// Overwrite existing
    pub overwrite: bool,
}

/// Download progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    /// Download ID
    pub download_id: String,
    /// Bytes downloaded
    pub bytes_downloaded: u64,
    /// Total bytes
    pub total_bytes: u64,
    /// Progress percentage
    pub progress: f32,
    /// Speed in bytes/second
    pub speed_bps: u64,
    /// Status
    pub status: DownloadStatus,
}

/// Download status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DownloadStatus {
    Queued,
    Downloading,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

/// Shared item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedItem {
    /// Share ID
    pub id: String,
    /// Item type
    pub item_type: SharedItemType,
    /// Item ID
    pub item_id: String,
    /// Shared by
    pub shared_by: String,
    /// Shared at
    pub shared_at: DateTime<Utc>,
    /// Expires at
    pub expires_at: Option<DateTime<Utc>>,
    /// Access level
    pub access: ShareAccess,
    /// Share link
    pub link: Option<String>,
    /// Password protected
    pub password: Option<String>,
    /// Recipients
    pub recipients: Vec<ShareRecipient>,
}

/// Shared item type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SharedItemType {
    File,
    Folder,
    Tab,
    Session,
    Bookmark,
    Collection,
}

/// Share access level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShareAccess {
    View,
    Edit,
    Comment,
    Full,
}

/// Share recipient
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareRecipient {
    pub id: String,
    pub email: Option<String>,
    pub user_id: Option<String>,
    pub name: String,
    pub access: ShareAccess,
    pub notified: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_usage() {
        let usage = StorageUsage {
            used_bytes: 500,
            quota_bytes: 1000,
            file_count: 10,
            by_category: HashMap::new(),
        };
        
        assert_eq!(usage.usage_percent(), 50.0);
        assert_eq!(usage.available_bytes(), 500);
    }
}