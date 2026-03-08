//! Remote Control Manager
//! 
//! Handles remote browser control from mobile devices.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast, mpsc};
use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

use super::{
    RemoteCommand, RemoteCommandResult, MobileEvent,
    models::{RemoteStreamingSession, StreamingQuality},
};

/// Remote control manager
pub struct RemoteControlManager {
    /// Active sessions
    sessions: RwLock<HashMap<String, RemoteSession>>,
    /// Streaming sessions
    streaming_sessions: RwLock<HashMap<String, RemoteStreamingSession>>,
    /// Command history
    command_history: RwLock<Vec<CommandRecord>>,
    /// Event sender
    event_sender: broadcast::Sender<MobileEvent>,
    /// Command executor
    executor: RwLock<Option<mpsc::Sender<InternalCommand>>>,
    /// Running flag
    running: RwLock<bool>,
    /// Configuration
    config: RwLock<RemoteControlConfig>,
}

/// Remote control configuration
#[derive(Debug, Clone)]
pub struct RemoteControlConfig {
    /// Enable remote control
    pub enabled: bool,
    /// Maximum concurrent sessions
    pub max_sessions: usize,
    /// Session timeout in seconds
    pub session_timeout: u64,
    /// Enable streaming
    pub streaming_enabled: bool,
    /// Default frame rate
    pub default_frame_rate: u32,
    /// Default quality
    pub default_quality: StreamingQuality,
    /// Command rate limit per minute
    pub rate_limit: u32,
}

impl Default for RemoteControlConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_sessions: 10,
            session_timeout: 300,
            streaming_enabled: true,
            default_frame_rate: 30,
            default_quality: StreamingQuality::Auto,
            rate_limit: 60,
        }
    }
}

/// Remote session
#[derive(Debug, Clone)]
pub struct RemoteSession {
    /// Session ID
    pub id: String,
    /// Device ID
    pub device_id: String,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Last activity
    pub last_activity: DateTime<Utc>,
    /// Permissions
    pub permissions: Vec<RemotePermission>,
    /// Is active
    pub is_active: bool,
}

/// Remote permissions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RemotePermission {
    ViewTabs,
    ControlTabs,
    ViewBookmarks,
    EditBookmarks,
    ViewHistory,
    ViewSettings,
    EditSettings,
    FullControl,
}

/// Command record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRecord {
    /// Record ID
    pub id: String,
    /// Session ID
    pub session_id: String,
    /// Device ID
    pub device_id: String,
    /// Command
    pub command: RemoteCommand,
    /// Result
    pub result: Option<RemoteCommandResult>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Internal command for processing
#[derive(Debug)]
struct InternalCommand {
    session_id: String,
    command: RemoteCommand,
    response: mpsc::Sender<RemoteCommandResult>,
}

impl RemoteControlManager {
    /// Create a new remote control manager
    pub async fn new(event_sender: broadcast::Sender<MobileEvent>) -> Result<Self> {
        Ok(Self {
            sessions: RwLock::new(HashMap::new()),
            streaming_sessions: RwLock::new(HashMap::new()),
            command_history: RwLock::new(Vec::new()),
            event_sender,
            executor: RwLock::new(None),
            running: RwLock::new(false),
            config: RwLock::new(RemoteControlConfig::default()),
        })
    }
    
    /// Start remote control service
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = true;
        
        // Create command channel
        let (tx, _rx) = mpsc::channel(100);
        let mut executor = self.executor.write().await;
        *executor = Some(tx);
        
        tracing::info!("Remote control service started");
        Ok(())
    }
    
    /// Stop remote control service
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;
        
        // Clear sessions
        let mut sessions = self.sessions.write().await;
        sessions.clear();
        
        // Clear executor
        let mut executor = self.executor.write().await;
        *executor = None;
        
        tracing::info!("Remote control service stopped");
        Ok(())
    }
    
    /// Create remote session
    pub async fn create_session(
        &self,
        device_id: &str,
        permissions: Vec<RemotePermission>,
    ) -> Result<RemoteSession> {
        let config = self.config.read().await;
        
        // Check session limit
        let sessions = self.sessions.read().await;
        let active_count = sessions.values().filter(|s| s.is_active).count();
        drop(sessions);
        
        if active_count >= config.max_sessions {
            return Err(anyhow::anyhow!("Maximum sessions reached"));
        }
        
        let session = RemoteSession {
            id: Uuid::new_v4().to_string(),
            device_id: device_id.to_string(),
            created_at: Utc::now(),
            last_activity: Utc::now(),
            permissions,
            is_active: true,
        };
        
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id.clone(), session.clone());
        
        tracing::info!("Remote session created: {} for device: {}", 
            session.id, device_id);
        
        Ok(session)
    }
    
    /// End remote session
    pub async fn end_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.remove(session_id) {
            // End any streaming sessions
            let mut streaming = self.streaming_sessions.write().await;
            streaming.retain(|_, s| s.device_id != session.device_id);
            
            tracing::info!("Remote session ended: {}", session_id);
        }
        
        Ok(())
    }
    
    /// Get session
    pub async fn get_session(&self, session_id: &str) -> Option<RemoteSession> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }
    
    /// Validate session
    pub async fn validate_session(&self, session_id: &str) -> bool {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).map(|s| s.is_active).unwrap_or(false)
    }
    
    /// Check permission
    pub async fn has_permission(
        &self,
        session_id: &str,
        permission: &RemotePermission,
    ) -> bool {
        let sessions = self.sessions.read().await;
        
        if let Some(session) = sessions.get(session_id) {
            session.permissions.contains(permission) ||
                session.permissions.contains(&RemotePermission::FullControl)
        } else {
            false
        }
    }
    
    /// Execute remote command
    pub async fn execute_command(
        &self,
        device_id: &str,
        command: RemoteCommand,
    ) -> Result<RemoteCommandResult> {
        let start = std::time::Instant::now();
        
        // Emit event
        let _ = self.event_sender.send(MobileEvent::RemoteCommand(
            device_id.to_string(),
            command.clone(),
        ));
        
        // Execute based on command type
        let (success, data, error) = self.execute_internal(&command).await;
        
        let result = RemoteCommandResult {
            command: command.clone(),
            success,
            data,
            error,
            execution_time_ms: start.elapsed().as_millis() as u64,
        };
        
        // Record command
        self.record_command(device_id, &command, &result).await;
        
        Ok(result)
    }
    
    /// Internal command execution
    async fn execute_internal(
        &self,
        command: &RemoteCommand,
    ) -> (bool, serde_json::Value, Option<String>) {
        match command {
            RemoteCommand::OpenUrl(url) => {
                tracing::info!("Remote: Opening URL: {}", url);
                (true, serde_json::json!({"url": url}), None)
            }
            
            RemoteCommand::CloseTab(tab_id) => {
                tracing::info!("Remote: Closing tab: {}", tab_id);
                (true, serde_json::json!({"tab_id": tab_id}), None)
            }
            
            RemoteCommand::SwitchTab(tab_id) => {
                tracing::info!("Remote: Switching to tab: {}", tab_id);
                (true, serde_json::json!({"tab_id": tab_id}), None)
            }
            
            RemoteCommand::Refresh(tab_id) => {
                tracing::info!("Remote: Refreshing tab: {}", tab_id);
                (true, serde_json::json!({"tab_id": tab_id}), None)
            }
            
            RemoteCommand::GoBack(tab_id) => {
                tracing::info!("Remote: Going back on tab: {}", tab_id);
                (true, serde_json::json!({"tab_id": tab_id}), None)
            }
            
            RemoteCommand::GoForward(tab_id) => {
                tracing::info!("Remote: Going forward on tab: {}", tab_id);
                (true, serde_json::json!({"tab_id": tab_id}), None)
            }
            
            RemoteCommand::Bookmark(tab_id) => {
                tracing::info!("Remote: Bookmarking tab: {}", tab_id);
                (true, serde_json::json!({"tab_id": tab_id}), None)
            }
            
            RemoteCommand::Screenshot(tab_id) => {
                tracing::info!("Remote: Taking screenshot of tab: {}", tab_id);
                // Would return actual screenshot
                (true, serde_json::json!({
                    "tab_id": tab_id,
                    "screenshot_url": "data:image/png;base64,..."
                }), None)
            }
            
            RemoteCommand::FillForm { tab_id, selector, value } => {
                tracing::info!("Remote: Filling form on tab {}: {} = {}", 
                    tab_id, selector, value);
                (true, serde_json::json!({
                    "tab_id": tab_id,
                    "selector": selector,
                    "filled": true
                }), None)
            }
            
            RemoteCommand::ExecuteScript { tab_id, script } => {
                tracing::info!("Remote: Executing script on tab: {}", tab_id);
                // Would execute in actual browser context
                (true, serde_json::json!({
                    "tab_id": tab_id,
                    "result": null
                }), None)
            }
            
            RemoteCommand::GetPageInfo(tab_id) => {
                tracing::info!("Remote: Getting page info for tab: {}", tab_id);
                (true, serde_json::json!({
                    "tab_id": tab_id,
                    "title": "Page Title",
                    "url": "https://example.com",
                    "status": "loaded"
                }), None)
            }
            
            RemoteCommand::ListTabs => {
                tracing::info!("Remote: Listing tabs");
                (true, serde_json::json!({
                    "tabs": [
                        {"id": 1, "title": "Tab 1", "url": "https://example.com"},
                        {"id": 2, "title": "Tab 2", "url": "https://example.org"}
                    ],
                    "active_tab": 1
                }), None)
            }
            
            RemoteCommand::SetTheme(theme) => {
                tracing::info!("Remote: Setting theme: {}", theme);
                (true, serde_json::json!({"theme": theme}), None)
            }
            
            RemoteCommand::ToggleFeature(feature, enabled) => {
                tracing::info!("Remote: Toggling feature {}: {}", feature, enabled);
                (true, serde_json::json!({
                    "feature": feature,
                    "enabled": enabled
                }), None)
            }
        }
    }
    
    /// Record command execution
    async fn record_command(
        &self,
        device_id: &str,
        command: &RemoteCommand,
        result: &RemoteCommandResult,
    ) {
        let record = CommandRecord {
            id: Uuid::new_v4().to_string(),
            session_id: String::new(), // Would be set in real implementation
            device_id: device_id.to_string(),
            command: command.clone(),
            result: Some(result.clone()),
            timestamp: Utc::now(),
        };
        
        let mut history = self.command_history.write().await;
        history.push(record);
        
        // Keep only last 500 commands
        if history.len() > 500 {
            history.remove(0);
        }
    }
    
    /// Start streaming session
    pub async fn start_streaming(
        &self,
        device_id: &str,
        tab_id: u64,
        quality: Option<StreamingQuality>,
        frame_rate: Option<u32>,
    ) -> Result<RemoteStreamingSession> {
        let config = self.config.read().await;
        
        if !config.streaming_enabled {
            return Err(anyhow::anyhow!("Streaming is disabled"));
        }
        
        let session = RemoteStreamingSession {
            id: Uuid::new_v4().to_string(),
            device_id: device_id.to_string(),
            tab_id,
            started_at: Utc::now(),
            frame_rate: frame_rate.unwrap_or(config.default_frame_rate),
            quality: quality.unwrap_or_else(|| config.default_quality.clone()),
            is_active: true,
        };
        
        let mut streaming = self.streaming_sessions.write().await;
        streaming.insert(session.id.clone(), session.clone());
        
        tracing::info!("Streaming started: {} for tab: {}", session.id, tab_id);
        
        Ok(session)
    }
    
    /// Stop streaming session
    pub async fn stop_streaming(&self, session_id: &str) -> Result<()> {
        let mut streaming = self.streaming_sessions.write().await;
        
        if let Some(mut session) = streaming.get_mut(session_id) {
            session.is_active = false;
            tracing::info!("Streaming stopped: {}", session_id);
        }
        
        streaming.remove(session_id);
        Ok(())
    }
    
    /// Get streaming session
    pub async fn get_streaming_session(&self, session_id: &str) -> Option<RemoteStreamingSession> {
        let streaming = self.streaming_sessions.read().await;
        streaming.get(session_id).cloned()
    }
    
    /// Get active streaming sessions
    pub async fn get_active_streams(&self) -> Result<Vec<RemoteStreamingSession>> {
        let streaming = self.streaming_sessions.read().await;
        Ok(streaming.values().filter(|s| s.is_active).cloned().collect())
    }
    
    /// Get command history
    pub async fn get_command_history(&self, limit: usize) -> Result<Vec<CommandRecord>> {
        let history = self.command_history.read().await;
        Ok(history.iter().rev().take(limit).cloned().collect())
    }
    
    /// Get command history for device
    pub async fn get_device_command_history(
        &self,
        device_id: &str,
        limit: usize,
    ) -> Result<Vec<CommandRecord>> {
        let history = self.command_history.read().await;
        Ok(history.iter()
            .filter(|r| r.device_id == device_id)
            .rev()
            .take(limit)
            .cloned()
            .collect())
    }
    
    /// Cleanup inactive sessions
    pub async fn cleanup_inactive(&self) -> Result<u32> {
        let config = self.config.read().await;
        let timeout = chrono::Duration::seconds(config.session_timeout as i64);
        let now = Utc::now();
        
        let mut sessions = self.sessions.write().await;
        let initial_count = sessions.len();
        
        sessions.retain(|_, session| {
            let age = now - session.last_activity;
            age < timeout && session.is_active
        });
        
        Ok((initial_count - sessions.len()) as u32)
    }
    
    /// Update session activity
    pub async fn update_activity(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            session.last_activity = Utc::now();
        }
        
        Ok(())
    }
    
    /// Get statistics
    pub async fn get_statistics(&self) -> Result<RemoteControlStatistics> {
        let sessions = self.sessions.read().await;
        let streaming = self.streaming_sessions.read().await;
        let history = self.command_history.read().await;
        
        let active_sessions = sessions.values().filter(|s| s.is_active).count();
        let active_streams = streaming.values().filter(|s| s.is_active).count();
        let total_commands = history.len();
        let successful = history.iter().filter(|r| 
            r.result.as_ref().map(|res| res.success).unwrap_or(false)
        ).count();
        
        Ok(RemoteControlStatistics {
            active_sessions: active_sessions as u64,
            active_streams: active_streams as u64,
            total_commands: total_commands as u64,
            successful_commands: successful as u64,
            failed_commands: (total_commands - successful) as u64,
        })
    }
}

/// Remote control statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteControlStatistics {
    pub active_sessions: u64,
    pub active_streams: u64,
    pub total_commands: u64,
    pub successful_commands: u64,
    pub failed_commands: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_remote_config_default() {
        let config = RemoteControlConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_sessions, 10);
        assert!(config.streaming_enabled);
    }
    
    #[tokio::test]
    async fn test_create_session() {
        let (tx, _rx) = broadcast::channel(16);
        let manager = RemoteControlManager::new(tx).await.unwrap();
        
        manager.start().await.unwrap();
        
        let session = manager.create_session(
            "device-1",
            vec![RemotePermission::ViewTabs, RemotePermission::ControlTabs],
        ).await.unwrap();
        
        assert!(session.is_active);
        assert_eq!(session.device_id, "device-1");
    }
    
    #[tokio::test]
    async fn test_execute_command() {
        let (tx, _rx) = broadcast::channel(16);
        let manager = RemoteControlManager::new(tx).await.unwrap();
        
        manager.start().await.unwrap();
        
        let result = manager.execute_command(
            "device-1",
            RemoteCommand::OpenUrl("https://example.com".to_string()),
        ).await.unwrap();
        
        assert!(result.success);
    }
    
    #[tokio::test]
    async fn test_permission_check() {
        let (tx, _rx) = broadcast::channel(16);
        let manager = RemoteControlManager::new(tx).await.unwrap();
        
        manager.start().await.unwrap();
        
        let session = manager.create_session(
            "device-1",
            vec![RemotePermission::ViewTabs],
        ).await.unwrap();
        
        let has_view = manager.has_permission(&session.id, &RemotePermission::ViewTabs).await;
        assert!(has_view);
        
        let has_control = manager.has_permission(&session.id, &RemotePermission::ControlTabs).await;
        assert!(!has_control);
    }
}