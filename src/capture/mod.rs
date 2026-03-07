//! Screen Capture Module
//!
//! Screenshot and screen recording capabilities with annotation tools and sharing options.

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod screenshot;
pub mod recording;
pub mod annotation;
pub mod editor;
pub mod sharing;

use screenshot::ScreenshotManager;
use recording::RecordingManager;
use annotation::AnnotationManager;
use editor::CaptureEditor;
use sharing::SharingManager;

/// Capture type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptureType {
    Screenshot,
    Recording,
}

/// Screenshot mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScreenshotMode {
    VisibleArea,
    FullPage,
    SelectionRegion,
    DelayedCapture,
    ScrollingCapture,
}

/// Recording mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordingMode {
    Tab,
    Window,
    EntireScreen,
    SelectionRegion,
}

/// Capture format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptureFormat {
    PNG,
    JPEG,
    WEBP,
    MP4,
    WEBM,
    GIF,
}

/// Audio capture options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioCaptureOptions {
    pub capture_system_audio: bool,
    pub capture_microphone: bool,
    pub microphone_device_id: Option<String>,
}

impl Default for AudioCaptureOptions {
    fn default() -> Self {
        Self {
            capture_system_audio: true,
            capture_microphone: false,
            microphone_device_id: None,
        }
    }
}

/// Recording options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingOptions {
    pub mode: RecordingMode,
    pub format: CaptureFormat,
    pub quality: u8, // 1-100
    pub frame_rate: u16,
    pub audio: AudioCaptureOptions,
    pub webcam_overlay: bool,
    pub show_mouse: bool,
}

impl Default for RecordingOptions {
    fn default() -> Self {
        Self {
            mode: RecordingMode::Tab,
            format: CaptureFormat::WEBM,
            quality: 80,
            frame_rate: 30,
            audio: AudioCaptureOptions::default(),
            webcam_overlay: false,
            show_mouse: true,
        }
    }
}

/// Screenshot options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotOptions {
    pub mode: ScreenshotMode,
    pub format: CaptureFormat,
    pub quality: u8, // 1-100
    pub delay_seconds: Option<u32>,
    pub capture_cursor: bool,
}

impl Default for ScreenshotOptions {
    fn default() -> Self {
        Self {
            mode: ScreenshotMode::VisibleArea,
            format: CaptureFormat::PNG,
            quality: 90,
            delay_seconds: None,
            capture_cursor: true,
        }
    }
}

/// Capture result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureResult {
    pub capture_type: CaptureType,
    pub format: CaptureFormat,
    pub file_path: PathBuf,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub duration: Option<std::time::Duration>,
    pub width: u32,
    pub height: u32,
    pub file_size: u64,
}

/// Capture configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureConfig {
    pub default_screenshot_format: CaptureFormat,
    pub default_recording_format: CaptureFormat,
    pub default_screenshot_quality: u8,
    pub default_recording_quality: u8,
    pub default_recording_fps: u16,
    pub save_location: PathBuf,
    pub auto_open_editor: bool,
    pub copy_to_clipboard: bool,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            default_screenshot_format: CaptureFormat::PNG,
            default_recording_format: CaptureFormat::WEBM,
            default_screenshot_quality: 90,
            default_recording_quality: 80,
            default_recording_fps: 30,
            save_location: PathBuf::from("~/Pictures/VantisWebCaptures"),
            auto_open_editor: true,
            copy_to_clipboard: false,
        }
    }
}

/// Main capture manager
pub struct CaptureManager {
    config: Arc<RwLock<CaptureConfig>>,
    screenshot: Arc<ScreenshotManager>,
    recording: Arc<RecordingManager>,
    annotation: Arc<AnnotationManager>,
    editor: Arc<CaptureEditor>,
    sharing: Arc<SharingManager>,
    history: Arc<RwLock<Vec<CaptureResult>>>,
}

impl CaptureManager {
    /// Create a new capture manager
    pub fn new(config: CaptureConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            screenshot: Arc::new(ScreenshotManager::new()),
            recording: Arc::new(RecordingManager::new()),
            annotation: Arc::new(AnnotationManager::new()),
            editor: Arc::new(CaptureEditor::new()),
            sharing: Arc::new(SharingManager::new()),
            history: Arc::new(RwLock::new(vec![])),
        }
    }

    /// Take a screenshot
    pub async fn take_screenshot(&self, options: ScreenshotOptions) -> Result<CaptureResult, CaptureError> {
        let result = match options.mode {
            ScreenshotMode::VisibleArea => {
                self.screenshot.capture_visible_area(options).await?
            }
            ScreenshotMode::FullPage => {
                self.screenshot.capture_full_page(options).await?
            }
            ScreenshotMode::SelectionRegion => {
                self.screenshot.capture_selection(options).await?
            }
            ScreenshotMode::DelayedCapture => {
                self.screenshot.capture_with_delay(options).await?
            }
            ScreenshotMode::ScrollingCapture => {
                self.screenshot.capture_scrolling_page(options).await?
            }
        };

        self.add_to_history(result.clone()).await;
        
        let config = self.config.read().await;
        if config.copy_to_clipboard {
            if let Err(e) = self.copy_to_clipboard(&result).await {
                log::warn!("Failed to copy to clipboard: {:?}", e);
            }
        }
        
        Ok(result)
    }

    /// Start recording
    pub async fn start_recording(&self, options: RecordingOptions) -> Result<String, CaptureError> {
        let session_id = self.recording.start(options).await?;
        Ok(session_id)
    }

    /// Stop recording
    pub async fn stop_recording(&self, session_id: &str) -> Result<CaptureResult, CaptureError> {
        let result = self.recording.stop(session_id).await?;
        self.add_to_history(result.clone()).await;
        Ok(result)
    }

    /// Pause recording
    pub async fn pause_recording(&self, session_id: &str) -> Result<(), CaptureError> {
        self.recording.pause(session_id).await
    }

    /// Resume recording
    pub async fn resume_recording(&self, session_id: &str) -> Result<(), CaptureError> {
        self.recording.resume(session_id).await
    }

    /// Get recording status
    pub async fn get_recording_status(&self, session_id: &str) -> Result<RecordingStatus, CaptureError> {
        self.recording.status(session_id).await
    }

    /// Open capture in editor
    pub async fn open_in_editor(&self, capture: &CaptureResult) -> Result<(), CaptureError> {
        self.editor.open(capture).await
    }

    /// Annotate capture
    pub async fn annotate(&self, capture_path: &PathBuf, annotations: Vec<Annotation>) -> Result<PathBuf, CaptureError> {
        self.annotation.apply(capture_path, annotations).await
    }

    /// Share capture
    pub async fn share(&self, capture: &CaptureResult, method: ShareMethod) -> Result<String, CaptureError> {
        self.sharing.share(capture, method).await
    }

    /// Get capture history
    pub async fn history(&self) -> Vec<CaptureResult> {
        self.history.read().await.clone()
    }

    /// Clear history
    pub async fn clear_history(&self) {
        self.history.write().await.clear();
    }

    /// Update configuration
    pub async fn update_config(&self, config: CaptureConfig) {
        let mut cfg = self.config.write().await;
        *cfg = config;
    }

    /// Get configuration
    pub async fn config(&self) -> CaptureConfig {
        self.config.read().await.clone()
    }

    /// Get available screenshot modes
    pub fn screenshot_modes() -> Vec<ScreenshotMode> {
        vec![
            ScreenshotMode::VisibleArea,
            ScreenshotMode::FullPage,
            ScreenshotMode::SelectionRegion,
            ScreenshotMode::DelayedCapture,
            ScreenshotMode::ScrollingCapture,
        ]
    }

    /// Get available recording modes
    pub fn recording_modes() -> Vec<RecordingMode> {
        vec![
            RecordingMode::Tab,
            RecordingMode::Window,
            RecordingMode::EntireScreen,
            RecordingMode::SelectionRegion,
        ]
    }

    /// Get supported formats for screenshots
    pub fn screenshot_formats() -> Vec<CaptureFormat> {
        vec![
            CaptureFormat::PNG,
            CaptureFormat::JPEG,
            CaptureFormat::WEBP,
        ]
    }

    /// Get supported formats for recordings
    pub fn recording_formats() -> Vec<CaptureFormat> {
        vec![
            CaptureFormat::MP4,
            CaptureFormat::WEBM,
            CaptureFormat::GIF,
        ]
    }

    /// Add capture to history
    async fn add_to_history(&self, capture: CaptureResult) {
        let mut history = self.history.write().await;
        history.push(capture);
        
        // Keep only last 100 captures
        if history.len() > 100 {
            history.remove(0);
        }
    }

    /// Copy capture to clipboard
    async fn copy_to_clipboard(&self, capture: &CaptureResult) -> Result<(), CaptureError> {
        // In a real implementation, this would copy to system clipboard
        Ok(())
    }

    /// Delete capture
    pub async fn delete_capture(&self, path: &PathBuf) -> Result<(), CaptureError> {
        // In a real implementation, this would delete the file
        let mut history = self.history.write().await;
        history.retain(|c| &c.file_path != path);
        Ok(())
    }
}

/// Recording status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingStatus {
    pub is_recording: bool,
    pub is_paused: bool,
    pub duration: std::time::Duration,
    pub file_size: u64,
    pub frame_count: u32,
}

/// Annotation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Annotation {
    Pen {
        points: Vec<(f32, f32)>,
        color: String,
        width: f32,
    },
    Arrow {
        start: (f32, f32),
        end: (f32, f32),
        color: String,
        width: f32,
    },
    Rectangle {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: String,
        filled: bool,
    },
    Circle {
        x: f32,
        y: f32,
        radius: f32,
        color: String,
        filled: bool,
    },
    Text {
        x: f32,
        y: f32,
        text: String,
        font_size: f32,
        color: String,
    },
    Blur {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        intensity: f32,
    },
    Highlight {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: String,
    },
}

/// Share methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShareMethod {
    Link,
    Email,
    CopyToClipboard,
    SaveToDevice,
    Custom,
}

/// Capture errors
#[derive(Debug, thiserror::Error)]
pub enum CaptureError {
    #[error("Capture failed: {0}")]
    CaptureFailed(String),

    #[error("Recording failed: {0}")]
    RecordingFailed(String),

    #[error("Editor failed: {0}")]
    EditorFailed(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid format")]
    InvalidFormat,

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Not supported on this platform")]
    NotSupported,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_capture_manager_creation() {
        let config = CaptureConfig::default();
        let manager = CaptureManager::new(config);
        assert_eq!(manager.history().await.len(), 0);
    }

    #[test]
    fn test_screenshot_modes() {
        let modes = CaptureManager::screenshot_modes();
        assert_eq!(modes.len(), 5);
    }

    #[test]
    fn test_recording_modes() {
        let modes = CaptureManager::recording_modes();
        assert_eq!(modes.len(), 4);
    }

    #[test]
    fn test_screenshot_formats() {
        let formats = CaptureManager::screenshot_formats();
        assert_eq!(formats.len(), 3);
    }

    #[test]
    fn test_recording_formats() {
        let formats = CaptureManager::recording_formats();
        assert_eq!(formats.len(), 3);
    }
}