// Copyright 2024 Vantis Corporation - All Rights Reserved

//! Browser Integration Module
//! 
//! This module provides seamless integration between the download manager
//! and the browser UI, including context menus, automatic link detection,
//! bulk downloads, and notifications.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::downloads::Download;
use crate::downloads::DownloadStatus;

/// Browser integration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserIntegrationSettings {
    /// Enable context menu integration
    pub context_menu_enabled: bool,
    /// Enable automatic link detection
    pub auto_detection_enabled: bool,
    /// Enable notifications
    pub notifications_enabled: bool,
    /// Enable clipboard monitoring
    pub clipboard_monitoring_enabled: bool,
    /// Default download folder
    pub default_download_folder: PathBuf,
    /// Ask for download location
    pub ask_for_location: bool,
    /// Open downloads panel on start
    pub open_panel_on_start: bool,
    /// Show download progress in toolbar
    pub show_progress_in_toolbar: bool,
}

impl Default for BrowserIntegrationSettings {
    fn default() -> Self {
        Self {
            context_menu_enabled: true,
            auto_detection_enabled: true,
            notifications_enabled: true,
            clipboard_monitoring_enabled: false,
            default_download_folder: dirs::download_dir().unwrap_or_else(|| PathBuf::from("~/Downloads")),
            ask_for_location: false,
            open_panel_on_start: false,
            show_progress_in_toolbar: true,
        }
    }
}

/// Context menu item for downloads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMenuItem {
    /// Item ID
    pub id: String,
    /// Display text
    pub text: String,
    /// Icon name
    pub icon: String,
    /// Action type
    pub action: ContextMenuAction,
    /// Keyboard shortcut
    pub shortcut: Option<String>,
    /// Enabled state
    pub enabled: bool,
}

/// Context menu action type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextMenuAction {
    /// Download link
    DownloadLink,
    /// Download with custom name
    DownloadWithCustomName,
    /// Download all links on page
    DownloadAllLinks,
    /// Download all images
    DownloadAllImages,
    /// Download all videos
    DownloadAllVideos,
    /// Add to queue
    AddToQueue,
    /// Schedule download
    ScheduleDownload,
    /// Set priority
    SetPriority { priority: u8 },
}

/// Detected downloadable resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadableResource {
    /// Resource ID
    pub id: Uuid,
    /// Resource URL
    pub url: String,
    /// Resource type
    pub resource_type: ResourceType,
    /// File size (bytes)
    pub size: Option<u64>,
    /// MIME type
    pub mime_type: Option<String>,
    /// File name
    pub file_name: Option<String>,
    /// Source page URL
    pub source_url: String,
    /// Resource position
    pub position: ResourcePosition,
    /// Timestamp detected
    pub detected_at: DateTime<Utc>,
    /// Selected state
    pub selected: bool,
}

/// Resource type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResourceType {
    /// Direct download link
    DirectDownload,
    /// Image
    Image,
    /// Video
    Video,
    /// Audio
    Audio,
    /// Document
    Document,
    /// Archive
    Archive,
    /// Unknown
    Unknown,
}

/// Resource position on page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePosition {
    /// X coordinate
    pub x: f64,
    /// Y coordinate
    pub y: f64,
    /// Width
    pub width: f64,
    /// Height
    pub height: f64,
}

/// Bulk download configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkDownloadConfig {
    /// Include images
    pub include_images: bool,
    /// Include videos
    pub include_videos: bool,
    /// Include audio
    pub include_audio: bool,
    /// Include documents
    pub include_documents: bool,
    /// Include archives
    pub include_archives: bool,
    /// Minimum file size (bytes)
    pub min_size: Option<u64>,
    /// Maximum file size (bytes)
    pub max_size: Option<u64>,
    /// Filter by extension
    pub extension_filter: Option<Vec<String>>,
    /// Rename pattern
    pub rename_pattern: Option<String>,
    /// Subfolder pattern
    pub subfolder_pattern: Option<String>,
}

impl Default for BulkDownloadConfig {
    fn default() -> Self {
        Self {
            include_images: true,
            include_videos: true,
            include_audio: true,
            include_documents: true,
            include_archives: true,
            min_size: None,
            max_size: None,
            extension_filter: None,
            rename_pattern: None,
            subfolder_pattern: None,
        }
    }
}

/// Download notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadNotification {
    /// Notification ID
    pub id: Uuid,
    /// Notification type
    pub notification_type: NotificationType,
    /// Download ID
    pub download_id: Uuid,
    /// File name
    pub file_name: String,
    /// Notification message
    pub message: String,
    /// Notification timestamp
    pub timestamp: DateTime<Utc>,
    /// Notification priority
    pub priority: NotificationPriority,
    /// Read state
    pub read: bool,
}

/// Notification type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    /// Download started
    DownloadStarted,
    /// Download completed
    DownloadCompleted,
    /// Download failed
    DownloadFailed,
    /// Download paused
    DownloadPaused,
    /// Download resumed
    DownloadResumed,
    /// Download cancelled
    DownloadCancelled,
    /// All downloads completed
    AllCompleted,
}

/// Notification priority
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationPriority {
    /// Low priority
    Low,
    /// Normal priority
    Normal,
    /// High priority
    High,
    /// Critical priority
    Critical,
}

/// Video download options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoDownloadOptions {
    /// Stream quality
    pub quality: VideoQuality,
    /// Include audio
    pub include_audio: bool,
    /// Extract subtitles
    pub extract_subtitles: bool,
    /// Audio-only mode
    pub audio_only: bool,
    /// Output format
    pub output_format: String,
}

/// Video quality options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VideoQuality {
    /// Auto (best available)
    Auto,
    /// 4K resolution
    UHD4K,
    /// 1080p
    FullHD,
    /// 720p
    HD,
    /// 480p
    SD,
    /// 360p
    Low,
    /// Audio only
    AudioOnly,
}

/// Clipboard monitor event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEvent {
    /// Event ID
    pub id: Uuid,
    /// Detected URL
    pub url: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Auto-download flag
    pub auto_download: bool,
}

/// Browser integration manager
pub struct BrowserIntegration {
    /// Integration settings
    settings: Arc<RwLock<BrowserIntegrationSettings>>,
    /// Detected resources
    resources: Arc<RwLock<HashMap<Uuid, DownloadableResource>>>,
    /// Notifications
    notifications: Arc<RwLock<Vec<DownloadNotification>>>,
    /// Clipboard events
    clipboard_events: Arc<RwLock<Vec<ClipboardEvent>>>,
    /// Video download options
    video_options: Arc<RwLock<HashMap<Uuid, VideoDownloadOptions>>>,
}

impl BrowserIntegration {
    /// Create a new browser integration instance
    pub fn new() -> Self {
        Self {
            settings: Arc::new(RwLock::new(BrowserIntegrationSettings::default())),
            resources: Arc::new(RwLock::new(HashMap::new())),
            notifications: Arc::new(RwLock::new(Vec::new())),
            clipboard_events: Arc::new(RwLock::new(Vec::new())),
            video_options: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get context menu items for current page
    pub async fn get_context_menu_items(&self, url: &str, selected_text: Option<&str>) -> Vec<ContextMenuItem> {
        let settings = self.settings.read().await;
        
        if !settings.context_menu_enabled {
            return Vec::new();
        }

        let mut items = vec![
            ContextMenuItem {
                id: "download-link".to_string(),
                text: "Download Link".to_string(),
                icon: "download".to_string(),
                action: ContextMenuAction::DownloadLink,
                shortcut: Some("Ctrl+S".to_string()),
                enabled: url.starts_with("http"),
            },
            ContextMenuItem {
                id: "download-as".to_string(),
                text: "Download As...".to_string(),
                icon: "save-as".to_string(),
                action: ContextMenuAction::DownloadWithCustomName,
                shortcut: None,
                enabled: url.starts_with("http"),
            },
        ];

        // Add bulk download options
        items.push(ContextMenuItem {
            id: "separator-1".to_string(),
            text: "-".to_string(),
            icon: "".to_string(),
            action: ContextMenuAction::DownloadLink,
            shortcut: None,
            enabled: false,
        });

        items.push(ContextMenuItem {
            id: "download-all".to_string(),
            text: "Download All Links".to_string(),
            icon: "download-multiple".to_string(),
            action: ContextMenuAction::DownloadAllLinks,
            shortcut: None,
            enabled: true,
        });

        items.push(ContextMenuItem {
            id: "download-images".to_string(),
            text: "Download All Images".to_string(),
            icon: "image-download".to_string(),
            action: ContextMenuAction::DownloadAllImages,
            shortcut: None,
            enabled: true,
        });

        items.push(ContextMenuItem {
            id: "download-videos".to_string(),
            text: "Download All Videos".to_string(),
            icon: "video-download".to_string(),
            action: ContextMenuAction::DownloadAllVideos,
            shortcut: None,
            enabled: true,
        });

        // Add queue options
        items.push(ContextMenuItem {
            id: "separator-2".to_string(),
            text: "-".to_string(),
            icon: "".to_string(),
            action: ContextMenuAction::DownloadLink,
            shortcut: None,
            enabled: false,
        });

        items.push(ContextMenuItem {
            id: "add-to-queue".to_string(),
            text: "Add to Queue".to_string(),
            icon: "queue".to_string(),
            action: ContextMenuAction::AddToQueue,
            shortcut: None,
            enabled: url.starts_with("http"),
        });

        items.push(ContextMenuItem {
            id: "schedule".to_string(),
            text: "Schedule Download".to_string(),
            icon: "schedule".to_string(),
            action: ContextMenuAction::ScheduleDownload,
            shortcut: None,
            enabled: url.starts_with("http"),
        });

        items
    }

    /// Detect downloadable resources on a page
    pub async fn detect_resources(&self, page_url: &str, html_content: &str) -> Vec<DownloadableResource> {
        let mut resources = Vec::new();
        let mut resource_map = self.resources.write().await;

        // Detect image resources
        for (index, url) in self.extract_image_urls(html_content) {
            let resource = DownloadableResource {
                id: Uuid::new_v4(),
                url,
                resource_type: ResourceType::Image,
                size: None,
                mime_type: Some("image/*".to_string()),
                file_name: self.extract_filename_from_url(&url),
                source_url: page_url.to_string(),
                position: ResourcePosition {
                    x: 0.0,
                    y: index as f64 * 10.0,
                    width: 100.0,
                    height: 100.0,
                },
                detected_at: Utc::now(),
                selected: true,
            };
            resources.push(resource.clone());
            resource_map.insert(resource.id, resource);
        }

        // Detect video resources
        for (index, url) in self.extract_video_urls(html_content) {
            let resource = DownloadableResource {
                id: Uuid::new_v4(),
                url,
                resource_type: ResourceType::Video,
                size: None,
                mime_type: Some("video/*".to_string()),
                file_name: self.extract_filename_from_url(&url),
                source_url: page_url.to_string(),
                position: ResourcePosition {
                    x: 0.0,
                    y: index as f64 * 10.0,
                    width: 100.0,
                    height: 100.0,
                },
                detected_at: Utc::now(),
                selected: true,
            };
            resources.push(resource.clone());
            resource_map.insert(resource.id, resource);
        }

        // Detect audio resources
        for (index, url) in self.extract_audio_urls(html_content) {
            let resource = DownloadableResource {
                id: Uuid::new_v4(),
                url,
                resource_type: ResourceType::Audio,
                size: None,
                mime_type: Some("audio/*".to_string()),
                file_name: self.extract_filename_from_url(&url),
                source_url: page_url.to_string(),
                position: ResourcePosition {
                    x: 0.0,
                    y: index as f64 * 10.0,
                    width: 100.0,
                    height: 100.0,
                },
                detected_at: Utc::now(),
                selected: true,
            };
            resources.push(resource.clone());
            resource_map.insert(resource.id, resource);
        }

        resources
    }

    /// Extract image URLs from HTML
    fn extract_image_urls(&self, html: &str) -> Vec<(usize, String)> {
        let mut urls = Vec::new();
        
        // Simple regex-based extraction (in production, use proper HTML parser)
        let re = regex::Regex::new(r##"src=["']([^"']+\.(?:jpg|jpeg|png|gif|webp|svg|bmp|ico))["']"##).unwrap();
        for (index, cap) in re.captures_iter(html).enumerate() {
            if let Some(url) = cap.get(1) {
                urls.push((index, url.as_str().to_string()));
            }
        }
        
        urls
    }

    /// Extract video URLs from HTML
    fn extract_video_urls(&self, html: &str) -> Vec<(usize, String)> {
        let mut urls = Vec::new();
        
        // Extract from video tags
        let re = regex::Regex::new(r##"src=["']([^"']+\.(?:mp4|webm|ogg|avi|mkv|mov))["']"##).unwrap();
        for (index, cap) in re.captures_iter(html).enumerate() {
            if let Some(url) = cap.get(1) {
                urls.push((index, url.as_str().to_string()));
            }
        }
        
        // Extract from iframe/embed (YouTube, etc.)
        let iframe_re = regex::Regex::new(r##"src=["']([^"']*(?:youtube|vimeo|dailymotion)[^"']*)["']"##).unwrap();
        for (index, cap) in iframe_re.captures_iter(html).enumerate() {
            if let Some(url) = cap.get(1) {
                urls.push((index, url.as_str().to_string()));
            }
        }
        
        urls
    }

    /// Extract audio URLs from HTML
    fn extract_audio_urls(&self, html: &str) -> Vec<(usize, String)> {
        let mut urls = Vec::new();
        
        let re = regex::Regex::new(r##"src=["']([^"']+\.(?:mp3|wav|ogg|flac|aac|m4a))["']"##).unwrap();
        for (index, cap) in re.captures_iter(html).enumerate() {
            if let Some(url) = cap.get(1) {
                urls.push((index, url.as_str().to_string()));
            }
        }
        
        urls
    }

    /// Extract filename from URL
    fn extract_filename_from_url(&self, url: &str) -> Option<String> {
        let parsed = url::Url::parse(url).ok()?;
        let path = parsed.path();
        path.split('/')
            .last()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
    }

    /// Filter resources based on bulk download config
    pub async fn filter_resources(&self, config: &BulkDownloadConfig) -> Vec<DownloadableResource> {
        let resources = self.resources.read().await;
        
        resources.values()
            .filter(|r| {
                // Filter by resource type
                let type_match = match r.resource_type {
                    ResourceType::Image => config.include_images,
                    ResourceType::Video => config.include_videos,
                    ResourceType::Audio => config.include_audio,
                    ResourceType::Document => config.include_documents,
                    ResourceType::Archive => config.include_archives,
                    _ => false,
                };

                if !type_match {
                    return false;
                }

                // Filter by extension
                if let Some(ref extensions) = config.extension_filter {
                    if let Some(ref filename) = r.file_name {
                        let ext = filename.split('.').last().unwrap_or("");
                        if !extensions.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
                            return false;
                        }
                    }
                }

                // Filter by size (if size is known)
                if let (Some(min_size), Some(size)) = (config.min_size, r.size) {
                    if size < min_size {
                        return false;
                    }
                }

                if let (Some(max_size), Some(size)) = (config.max_size, r.size) {
                    if size > max_size {
                        return false;
                    }
                }

                // Filter by selected state
                r.selected
            })
            .cloned()
            .collect()
    }

    /// Add download notification
    pub async fn add_notification(
        &self,
        download: &Download,
        notification_type: NotificationType,
    ) -> DownloadNotification {
        let notification = DownloadNotification {
            id: Uuid::new_v4(),
            notification_type,
            download_id: download.id,
            file_name: download.file_name.clone(),
            message: self.get_notification_message(&notification_type, &download.file_name),
            timestamp: Utc::now(),
            priority: self.get_notification_priority(&notification_type),
            read: false,
        };

        self.notifications.write().await.push(notification.clone());
        notification
    }

    /// Get notification message
    fn get_notification_message(&self, notification_type: &NotificationType, file_name: &str) -> String {
        match notification_type {
            NotificationType::DownloadStarted => format!("Download started: {}", file_name),
            NotificationType::DownloadCompleted => format!("Download completed: {}", file_name),
            NotificationType::DownloadFailed => format!("Download failed: {}", file_name),
            NotificationType::DownloadPaused => format!("Download paused: {}", file_name),
            NotificationType::DownloadResumed => format!("Download resumed: {}", file_name),
            NotificationType::DownloadCancelled => format!("Download cancelled: {}", file_name),
            NotificationType::AllCompleted => "All downloads completed".to_string(),
        }
    }

    /// Get notification priority
    fn get_notification_priority(&self, notification_type: &NotificationType) -> NotificationPriority {
        match notification_type {
            NotificationType::DownloadStarted => NotificationPriority::Low,
            NotificationType::DownloadCompleted => NotificationPriority::Normal,
            NotificationType::DownloadFailed => NotificationPriority::High,
            NotificationType::DownloadPaused => NotificationPriority::Normal,
            NotificationType::DownloadResumed => NotificationPriority::Normal,
            NotificationType::DownloadCancelled => NotificationPriority::Low,
            NotificationType::AllCompleted => NotificationPriority::Normal,
        }
    }

    /// Get unread notifications
    pub async fn get_unread_notifications(&self) -> Vec<DownloadNotification> {
        let notifications = self.notifications.read().await;
        notifications.iter()
            .filter(|n| !n.read)
            .cloned()
            .collect()
    }

    /// Mark notification as read
    pub async fn mark_notification_read(&self, notification_id: Uuid) -> bool {
        let mut notifications = self.notifications.write().await;
        if let Some(notification) = notifications.iter_mut().find(|n| n.id == notification_id) {
            notification.read = true;
            return true;
        }
        false
    }

    /// Clear all notifications
    pub async fn clear_notifications(&self) {
        self.notifications.write().await.clear();
    }

    /// Add clipboard event
    pub async fn add_clipboard_event(&self, url: String, auto_download: bool) -> ClipboardEvent {
        let event = ClipboardEvent {
            id: Uuid::new_v4(),
            url,
            timestamp: Utc::now(),
            auto_download,
        };

        self.clipboard_events.write().await.push(event.clone());
        event
    }

    /// Get recent clipboard events
    pub async fn get_recent_clipboard_events(&self, limit: usize) -> Vec<ClipboardEvent> {
        let events = self.clipboard_events.read().await;
        events.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Set video download options
    pub async fn set_video_options(&self, url: String, options: VideoDownloadOptions) {
        let video_id = Uuid::new_v4();
        self.video_options.write().await.insert(video_id, options);
    }

    /// Get settings
    pub async fn get_settings(&self) -> BrowserIntegrationSettings {
        self.settings.read().await.clone()
    }

    /// Update settings
    pub async fn update_settings(&self, settings: BrowserIntegrationSettings) {
        *self.settings.write().await = settings;
    }

    /// Get download completion badge count
    pub async fn get_badge_count(&self) -> usize {
        let notifications = self.notifications.read().await;
        notifications.iter()
            .filter(|n| !n.read && matches!(n.notification_type, NotificationType::DownloadCompleted))
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_context_menu_items() {
        let integration = BrowserIntegration::new();
        let items = integration.get_context_menu_items("https://example.com/file.pdf", None).await;
        
        assert!(!items.is_empty());
        assert!(items.iter().any(|i| i.id == "download-link"));
    }

    #[tokio::test]
    async fn test_detect_resources() {
        let integration = BrowserIntegration::new();
        let html = r##"
            <html>
                <img src="https://example.com/image1.jpg" />
                <img src="https://example.com/image2.png" />
                <video src="https://example.com/video1.mp4" />
                <audio src="https://example.com/audio1.mp3" />
            </html>
        "##;
        
        let resources = integration.detect_resources("https://example.com", html).await;
        assert!(!resources.is_empty());
        assert!(resources.iter().any(|r| r.resource_type == ResourceType::Image));
    }

    #[tokio::test]
    async fn test_filter_resources() {
        let integration = BrowserIntegration::new();
        let config = BulkDownloadConfig {
            include_images: true,
            include_videos: false,
            ..Default::default()
        };
        
        // Add test resources
        let mut resources = integration.resources.write().await;
        resources.insert(Uuid::new_v4(), DownloadableResource {
            id: Uuid::new_v4(),
            url: "https://example.com/image.jpg".to_string(),
            resource_type: ResourceType::Image,
            size: None,
            mime_type: None,
            file_name: None,
            source_url: "https://example.com".to_string(),
            position: ResourcePosition { x: 0.0, y: 0.0, width: 0.0, height: 0.0 },
            detected_at: Utc::now(),
            selected: true,
        });
        
        drop(resources);
        let filtered = integration.filter_resources(&config).await;
        assert!(!filtered.is_empty());
    }

    #[tokio::test]
    async fn test_notifications() {
        let integration = BrowserIntegration::new();
        
        let download = Download {
            id: Uuid::new_v4(),
            url: "https://example.com/file.pdf".to_string(),
            file_name: "file.pdf".to_string(),
            destination: PathBuf::from("/tmp/file.pdf"),
            size: 1024,
            downloaded: 0,
            speed: 0.0,
            status: DownloadStatus::Pending,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            error: None,
            priority: 0,
            retry_count: 0,
        };
        
        let notification = integration.add_notification(&download, NotificationType::DownloadStarted).await;
        assert_eq!(notification.notification_type, NotificationType::DownloadStarted);
        
        let unread = integration.get_unread_notifications().await;
        assert_eq!(unread.len(), 1);
    }

    #[tokio::test]
    async fn test_clipboard_events() {
        let integration = BrowserIntegration::new();
        integration.add_clipboard_event("https://example.com/file.pdf".to_string(), false).await;
        
        let events = integration.get_recent_clipboard_events(10).await;
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn test_settings() {
        let integration = BrowserIntegration::new();
        let settings = integration.get_settings().await;
        assert!(settings.context_menu_enabled);
        
        let mut new_settings = settings.clone();
        new_settings.context_menu_enabled = false;
        integration.update_settings(new_settings).await;
        
        let updated = integration.get_settings().await;
        assert!(!updated.context_menu_enabled);
    }

    #[tokio::test]
    async fn test_badge_count() {
        let integration = BrowserIntegration::new();
        
        let download = Download {
            id: Uuid::new_v4(),
            url: "https://example.com/file.pdf".to_string(),
            file_name: "file.pdf".to_string(),
            destination: PathBuf::from("/tmp/file.pdf"),
            size: 1024,
            downloaded: 0,
            speed: 0.0,
            status: DownloadStatus::Completed,
            created_at: Utc::now(),
            started_at: Some(Utc::now()),
            completed_at: Some(Utc::now()),
            error: None,
            priority: 0,
            retry_count: 0,
        };
        
        integration.add_notification(&download, NotificationType::DownloadCompleted).await;
        let count = integration.get_badge_count().await;
        assert_eq!(count, 1);
    }
}