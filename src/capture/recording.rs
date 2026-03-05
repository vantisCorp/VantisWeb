//! Screen Recording Functionality
//!
//! Record tab, window, screen with audio and webcam overlay.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::path::PathBuf;
use crate::capture::{RecordingOptions, CaptureResult, RecordingStatus, CaptureError};

/// Recording manager
pub struct RecordingManager {
    sessions: Arc<RwLock<HashMap<String, RecordingSession>>>,
    temp_dir: PathBuf,
}

/// Active recording session
#[derive(Clone)]
struct RecordingSession {
    id: String,
    options: RecordingOptions,
    started_at: chrono::DateTime<chrono::Utc>,
    paused_at: Option<chrono::DateTime<chrono::Utc>>,
    is_paused: bool,
    frame_count: u32,
    file_path: PathBuf,
}

impl RecordingManager {
    /// Create a new recording manager
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            temp_dir: std::env::temp_dir().join("vantisweb_recordings"),
        }
    }

    /// Start recording
    pub async fn start(&self, options: RecordingOptions) -> Result<String, CaptureError> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now();
        
        let filename = format!("recording_{}.{}", 
            timestamp.timestamp(), 
            options.format.to_extension()
        );
        let file_path = self.temp_dir.join(&filename);

        // In a real implementation, this would:
        // 1. Request screen capture permissions
        // 2. Set up video encoder
        // 3. Set up audio capture if enabled
        // 4. Start webcam overlay if enabled
        // 5. Begin capturing frames

        let session = RecordingSession {
            id: session_id.clone(),
            options: options.clone(),
            started_at: timestamp,
            paused_at: None,
            is_paused: false,
            frame_count: 0,
            file_path,
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);

        log::info!("Started recording session: {}", session_id);
        
        Ok(session_id)
    }

    /// Stop recording
    pub async fn stop(&self, session_id: &str) -> Result<CaptureResult, CaptureError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.remove(session_id)
            .ok_or_else(|| CaptureError::RecordingFailed("Session not found".to_string()))?;

        // In a real implementation, this would:
        // 1. Stop video capture
        // 2. Stop audio capture
        // 3. Finalize encoding
        // 4. Get file metadata

        let duration = if session.is_paused {
            // Account for pause time
            let total_paused = session.paused_at.unwrap_or(session.started_at) - session.started_at;
            (chrono::Utc::now() - session.started_at) - total_paused
        } else {
            chrono::Utc::now() - session.started_at
        };

        let file_size = std::fs::metadata(&session.file_path)
            .map(|m| m.len())
            .unwrap_or(0);

        log::info!("Stopped recording session: {}", session_id);

        Ok(CaptureResult {
            capture_type: crate::capture::CaptureType::Recording,
            format: session.options.format,
            file_path: session.file_path,
            timestamp: session.started_at,
            duration: Some(duration.to_std().unwrap_or_default()),
            width: 1920,
            height: 1080,
            file_size,
        })
    }

    /// Pause recording
    pub async fn pause(&self, session_id: &str) -> Result<(), CaptureError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(session_id)
            .ok_or_else(|| CaptureError::RecordingFailed("Session not found".to_string()))?;

        if session.is_paused {
            return Err(CaptureError::RecordingFailed("Already paused".to_string()));
        }

        session.is_paused = true;
        session.paused_at = Some(chrono::Utc::now());

        log::info!("Paused recording session: {}", session_id);
        Ok(())
    }

    /// Resume recording
    pub async fn resume(&self, session_id: &str) -> Result<(), CaptureError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(session_id)
            .ok_or_else(|| CaptureError::RecordingFailed("Session not found".to_string()))?;

        if !session.is_paused {
            return Err(CaptureError::RecordingFailed("Not paused".to_string()));
        }

        session.is_paused = false;
        session.paused_at = None;

        log::info!("Resumed recording session: {}", session_id);
        Ok(())
    }

    /// Get recording status
    pub async fn status(&self, session_id: &str) -> Result<RecordingStatus, CaptureError> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(session_id)
            .ok_or_else(|| CaptureError::RecordingFailed("Session not found".to_string()))?;

        let duration = if session.is_paused {
            let total_paused = session.paused_at.unwrap_or(session.started_at) - session.started_at;
            (chrono::Utc::now() - session.started_at) - total_paused
        } else {
            chrono::Utc::now() - session.started_at
        };

        let file_size = std::fs::metadata(&session.file_path)
            .map(|m| m.len())
            .unwrap_or(0);

        Ok(RecordingStatus {
            is_recording: true,
            is_paused: session.is_paused,
            duration: duration.to_std().unwrap_or_default(),
            file_size,
            frame_count: session.frame_count,
        })
    }

    /// Get active sessions
    pub async fn active_sessions(&self) -> Vec<String> {
        let sessions = self.sessions.read().await;
        sessions.keys().cloned().collect()
    }

    /// Cancel recording without saving
    pub async fn cancel(&self, session_id: &str) -> Result<(), CaptureError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.remove(session_id)
            .ok_or_else(|| CaptureError::RecordingFailed("Session not found".to_string()))?;

        // Delete partial file
        let _ = std::fs::remove_file(&session.file_path);

        log::info!("Cancelled recording session: {}", session_id);
        Ok(())
    }

    /// Add frame (called by recording loop)
    async fn add_frame(&self, session_id: &str) -> Result<(), CaptureError> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            if !session.is_paused {
                session.frame_count += 1;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::{AudioCaptureOptions, CaptureFormat};

    #[tokio::test]
    async fn test_start_recording() {
        let manager = RecordingManager::new();
        let options = RecordingOptions::default();
        let session_id = manager.start(options).await.unwrap();
        assert!(!session_id.is_empty());
    }

    #[tokio::test]
    async fn test_stop_recording() {
        let manager = RecordingManager::new();
        let options = RecordingOptions::default();
        let session_id = manager.start(options).await.unwrap();
        
        // Wait a bit
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        let result = manager.stop(&session_id).await.unwrap();
        assert_eq!(result.capture_type, crate::capture::CaptureType::Recording);
    }

    #[tokio::test]
    async fn test_pause_resume() {
        let manager = RecordingManager::new();
        let options = RecordingOptions::default();
        let session_id = manager.start(options).await.unwrap();

        manager.pause(&session_id).await.unwrap();
        let status = manager.status(&session_id).await.unwrap();
        assert!(status.is_paused);

        manager.resume(&session_id).await.unwrap();
        let status = manager.status(&session_id).await.unwrap();
        assert!(!status.is_paused);
    }

    #[tokio::test]
    async fn test_cancel_recording() {
        let manager = RecordingManager::new();
        let options = RecordingOptions::default();
        let session_id = manager.start(options).await.unwrap();

        manager.cancel(&session_id).await.unwrap();
        
        // Session should be gone
        let sessions = manager.active_sessions().await;
        assert!(!sessions.contains(&session_id));
    }

    #[tokio::test]
    async fn test_audio_options() {
        let manager = RecordingManager::new();
        let options = RecordingOptions {
            audio: AudioCaptureOptions {
                capture_system_audio: true,
                capture_microphone: true,
                microphone_device_id: Some("mic-1".to_string()),
            },
            ..Default::default()
        };
        
        let session_id = manager.start(options).await.unwrap();
        let status = manager.status(&session_id).await.unwrap();
        assert!(status.is_recording);
        
        manager.cancel(&session_id).await.unwrap();
    }
}