// Copyright 2024 Vantis Corporation - All Rights Reserved

//! Developer Tools Module
//! 
//! This module provides comprehensive developer tools for the VantisWeb browser,
//! including DOM inspection, network monitoring, JavaScript console, performance
//! profiling, storage inspection, and device emulation.

pub mod inspector;
pub mod network;
pub mod console;
pub mod profiler;
pub mod storage;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use inspector::ElementsInspector;
use network::NetworkMonitor;
use console::JSConsole;
use profiler::PerformanceProfiler;
use storage::StorageInspector;

/// Developer tools configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevToolsConfig {
    /// Enable developer tools
    pub enabled: bool,
    /// Auto-open on error
    pub auto_open_on_error: bool,
    /// Preserve console logs
    pub preserve_console_logs: bool,
    /// Enable source maps
    pub enable_source_maps: bool,
    /// Default panel
    pub default_panel: DevToolsPanel,
    /// Theme
    pub theme: DevToolsTheme,
    /// Window position
    pub window_position: WindowPosition,
    /// Log level
    pub log_level: LogLevel,
}

impl Default for DevToolsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_open_on_error: false,
            preserve_console_logs: false,
            enable_source_maps: true,
            default_panel: DevToolsPanel::Elements,
            theme: DevToolsTheme::Dark,
            window_position: WindowPosition::Right,
            log_level: LogLevel::Info,
        }
    }
}

/// Developer tools panel
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DevToolsPanel {
    /// Elements inspector
    Elements,
    /// Console
    Console,
    /// Sources
    Sources,
    /// Network
    Network,
    /// Performance
    Performance,
    /// Memory
    Memory,
    /// Application
    Application,
    /// Security
    Security,
    /// Accessibility
    Accessibility,
}

/// Developer tools theme
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DevToolsTheme {
    /// Light theme
    Light,
    /// Dark theme
    Dark,
    /// High contrast
    HighContrast,
}

/// Window position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WindowPosition {
    /// Right side
    Right,
    /// Bottom
    Bottom,
    /// Floating
    Floating { x: i32, y: i32, width: u32, height: u32 },
}

/// Log level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum LogLevel {
    /// Verbose
    Verbose,
    /// Debug
    Debug,
    /// Info
    Info,
    /// Warning
    Warning,
    /// Error
    Error,
    /// None
    None,
}

/// Developer tools message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DevToolsMessage {
    /// Panel activated
    PanelActivated { panel: DevToolsPanel },
    /// Panel deactivated
    PanelDeactivated { panel: DevToolsPanel },
    /// Tool opened
    ToolOpened { tool: String },
    /// Tool closed
    ToolClosed { tool: String },
    /// Configuration changed
    ConfigChanged { config: DevToolsConfig },
    /// Error occurred
    Error { message: String },
    /// Warning
    Warning { message: String },
    /// Info
    Info { message: String },
}

/// Main developer tools manager
pub struct DevToolsManager {
    /// Configuration
    config: Arc<RwLock<DevToolsConfig>>,
    /// Elements inspector
    inspector: Arc<RwLock<ElementsInspector>>,
    /// Network monitor
    network: Arc<RwLock<NetworkMonitor>>,
    /// JavaScript console
    console: Arc<RwLock<JSConsole>>,
    /// Performance profiler
    profiler: Arc<RwLock<PerformanceProfiler>>,
    /// Storage inspector
    storage: Arc<RwLock<StorageInspector>>,
    /// Active panel
    active_panel: Arc<RwLock<DevToolsPanel>>,
    /// Event handlers
    event_handlers: Arc<RwLock<Vec<Box<dyn DevToolsEventHandler>>>>,
    /// State
    state: Arc<RwLock<DevToolsState>>,
}

/// Developer tools state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevToolsState {
    /// Session ID
    pub session_id: Uuid,
    /// Open panels
    pub open_panels: Vec<DevToolsPanel>,
    /// Pinned elements
    pub pinned_elements: Vec<String>,
    /// Watch expressions
    pub watch_expressions: Vec<String>,
    /// Breakpoints
    pub breakpoints: Vec<Breakpoint>,
    /// Session start time
    pub session_start: DateTime<Utc>,
    /// Last activity
    pub last_activity: DateTime<Utc>,
}

/// Breakpoint location
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Breakpoint {
    /// Source URL
    pub source_url: String,
    /// Line number
    pub line_number: u32,
    /// Column number
    pub column: Option<u32>,
    /// Enabled
    pub enabled: bool,
    /// Condition
    pub condition: Option<String>,
}

/// Event handler trait
pub trait DevToolsEventHandler: Send + Sync {
    /// Handle devtools message
    fn handle_message(&self, message: &DevToolsMessage);
}

impl DevToolsManager {
    /// Create a new developer tools manager
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(DevToolsConfig::default())),
            inspector: Arc::new(RwLock::new(ElementsInspector::new())),
            network: Arc::new(RwLock::new(NetworkMonitor::new())),
            console: Arc::new(RwLock::new(JSConsole::new())),
            profiler: Arc::new(RwLock::new(PerformanceProfiler::new())),
            storage: Arc::new(RwLock::new(StorageInspector::new())),
            active_panel: Arc::new(RwLock::new(DevToolsPanel::Elements)),
            event_handlers: Arc::new(RwLock::new(Vec::new())),
            state: Arc::new(RwLock::new(DevToolsState {
                session_id: Uuid::new_v4(),
                open_panels: vec![DevToolsPanel::Elements],
                pinned_elements: Vec::new(),
                watch_expressions: Vec::new(),
                breakpoints: Vec::new(),
                session_start: Utc::now(),
                last_activity: Utc::now(),
            })),
        }
    }

    /// Initialize developer tools
    pub async fn initialize(&self) -> Result<(), DevToolsError> {
        let config = self.config.read().await;
        if !config.enabled {
            return Err(DevToolsError::Disabled);
        }

        // Initialize all tools
        self.inspector.write().await.initialize()?;
        self.network.write().await.initialize()?;
        self.console.write().await.initialize()?;
        self.profiler.write().await.initialize()?;
        self.storage.write().await.initialize()?;

        Ok(())
    }

    /// Open developer tools
    pub async fn open(&self) -> Result<(), DevToolsError> {
        let config = self.config.read().await;
        if !config.enabled {
            return Err(DevToolsError::Disabled);
        }

        self.send_message(DevToolsMessage::ToolOpened {
            tool: "devtools".to_string(),
        });

        Ok(())
    }

    /// Close developer tools
    pub async fn close(&self) -> Result<(), DevToolsError> {
        self.send_message(DevToolsMessage::ToolClosed {
            tool: "devtools".to_string(),
        });

        Ok(())
    }

    /// Activate panel
    pub async fn activate_panel(&self, panel: DevToolsPanel) -> Result<(), DevToolsError> {
        *self.active_panel.write().await = panel.clone();

        self.send_message(DevToolsMessage::PanelActivated { panel });

        Ok(())
    }

    /// Get active panel
    pub async fn get_active_panel(&self) -> DevToolsPanel {
        self.active_panel.read().await.clone()
    }

    /// Get configuration
    pub async fn get_config(&self) -> DevToolsConfig {
        self.config.read().await.clone()
    }

    /// Update configuration
    pub async fn update_config(&self, config: DevToolsConfig) -> Result<(), DevToolsError> {
        *self.config.write().await = config.clone();
        
        self.send_message(DevToolsMessage::ConfigChanged { config });
        
        Ok(())
    }

    /// Get elements inspector
    pub async fn get_inspector(&self) -> Arc<RwLock<ElementsInspector>> {
        self.inspector.clone()
    }

    /// Get network monitor
    pub async fn get_network_monitor(&self) -> Arc<RwLock<NetworkMonitor>> {
        self.network.clone()
    }

    /// Get console
    pub async fn get_console(&self) -> Arc<RwLock<JSConsole>> {
        self.console.clone()
    }

    /// Get profiler
    pub async fn get_profiler(&self) -> Arc<RwLock<PerformanceProfiler>> {
        self.profiler.clone()
    }

    /// Get storage inspector
    pub async fn get_storage(&self) -> Arc<RwLock<StorageInspector>> {
        self.storage.clone()
    }

    /// Add event handler
    pub async fn add_event_handler(&self, handler: Box<dyn DevToolsEventHandler>) {
        self.event_handlers.write().await.push(handler);
    }

    /// Remove event handler
    pub async fn remove_event_handler(&self, index: usize) {
        let mut handlers = self.event_handlers.write().await;
        if index < handlers.len() {
            handlers.remove(index);
        }
    }

    /// Send message to event handlers
    fn send_message(&self, message: DevToolsMessage) {
        let handlers = self.event_handlers.clone();
        tokio::spawn(async move {
            let handlers = handlers.read().await;
            for handler in handlers.iter() {
                handler.handle_message(&message);
            }
        });
    }

    /// Add breakpoint
    pub async fn add_breakpoint(&self, breakpoint: Breakpoint) {
        let mut state = self.state.write().await;
        state.breakpoints.push(breakpoint);
    }

    /// Remove breakpoint
    pub async fn remove_breakpoint(&self, source_url: &str, line_number: u32) -> bool {
        let mut state = self.state.write().await;
        let initial_len = state.breakpoints.len();
        state.breakpoints.retain(|b| {
            !(b.source_url == source_url && b.line_number == line_number)
        });
        state.breakpoints.len() < initial_len
    }

    /// Get breakpoints
    pub async fn get_breakpoints(&self) -> Vec<Breakpoint> {
        self.state.read().await.breakpoints.clone()
    }

    /// Add watch expression
    pub async fn add_watch_expression(&self, expression: String) {
        let mut state = self.state.write().await;
        state.watch_expressions.push(expression);
    }

    /// Remove watch expression
    pub async fn remove_watch_expression(&self, index: usize) -> bool {
        let mut state = self.state.write().await;
        if index < state.watch_expressions.len() {
            state.watch_expressions.remove(index);
            true
        } else {
            false
        }
    }

    /// Get watch expressions
    pub async fn get_watch_expressions(&self) -> Vec<String> {
        self.state.read().await.watch_expressions.clone()
    }

    /// Get state
    pub async fn get_state(&self) -> DevToolsState {
        self.state.read().await.clone()
    }

    /// Reset session
    pub async fn reset_session(&self) {
        let mut state = self.state.write().await;
        state.session_id = Uuid::new_v4();
        state.session_start = Utc::now();
        state.last_activity = Utc::now();
    }

    /// Get statistics
    pub async fn get_statistics(&self) -> DevToolsStatistics {
        let inspector = self.inspector.read().await;
        let network = self.network.read().await;
        let console = self.console.read().await;
        let profiler = self.profiler.read().await;

        DevToolsStatistics {
            elements_inspected: inspector.get_elements_count().await,
            network_requests: network.get_request_count().await,
            console_messages: console.get_message_count().await,
            profile_snapshots: profiler.get_snapshot_count().await,
            session_duration: (Utc::now() - self.state.read().await.session_start).num_seconds(),
        }
    }

    /// Export data
    pub async fn export_data(&self, format: ExportFormat) -> Result<String, DevToolsError> {
        match format {
            ExportFormat::JSON => {
                let state = self.state.read().await;
                let config = self.config.read().await;
                let data = serde_json::to_string_pretty(&(&*state, &*config))?;
                Ok(data)
            }
            ExportFormat::HAR => {
                let network = self.network.read().await;
                network.export_har().await
            }
        }
    }

    /// Clear all data
    pub async fn clear_all_data(&self) -> Result<(), DevToolsError> {
        self.console.write().await.clear()?;
        self.network.write().await.clear()?;
        self.profiler.write().await.clear()?;
        
        Ok(())
    }
}

/// Developer tools statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevToolsStatistics {
    /// Number of elements inspected
    pub elements_inspected: usize,
    /// Number of network requests
    pub network_requests: usize,
    /// Number of console messages
    pub console_messages: usize,
    /// Number of profile snapshots
    pub profile_snapshots: usize,
    /// Session duration in seconds
    pub session_duration: i64,
}

/// Export format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    /// JSON format
    JSON,
    /// HAR (HTTP Archive) format
    HAR,
}

/// Developer tools error
#[derive(Debug, thiserror::Error)]
pub enum DevToolsError {
    #[error("Developer tools are disabled")]
    Disabled,
    #[error("Panel not found: {0}")]
    PanelNotFound(String),
    #[error("Tool error: {0}")]
    ToolError(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Other error: {0}")]
    Other(String),
}

impl Default for DevToolsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_devtools_initialization() {
        let devtools = DevToolsManager::new();
        let result = devtools.initialize().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_panel_activation() {
        let devtools = DevToolsManager::new();
        devtools.initialize().await.unwrap();
        
        devtools.activate_panel(DevToolsPanel::Console).await.unwrap();
        let active = devtools.get_active_panel().await;
        assert_eq!(active, DevToolsPanel::Console);
    }

    #[tokio::test]
    async fn test_breakpoints() {
        let devtools = DevToolsManager::new();
        
        let breakpoint = Breakpoint {
            source_url: "https://example.com/script.js".to_string(),
            line_number: 42,
            column: Some(0),
            enabled: true,
            condition: None,
        };
        
        devtools.add_breakpoint(breakpoint.clone()).await;
        let breakpoints = devtools.get_breakpoints().await;
        assert_eq!(breakpoints.len(), 1);
        assert_eq!(breakpoints[0], breakpoint);
    }

    #[tokio::test]
    async fn test_watch_expressions() {
        let devtools = DevToolsManager::new();
        
        devtools.add_watch_expression("document.title".to_string()).await;
        devtools.add_watch_expression("window.location".to_string()).await;
        
        let expressions = devtools.get_watch_expressions().await;
        assert_eq!(expressions.len(), 2);
    }

    #[tokio::test]
    async fn test_statistics() {
        let devtools = DevToolsManager::new();
        devtools.initialize().await.unwrap();
        
        let stats = devtools.get_statistics().await;
        assert!(stats.session_duration >= 0);
    }

    #[tokio::test]
    async fn test_export_data() {
        let devtools = DevToolsManager::new();
        devtools.initialize().await.unwrap();
        
        let json = devtools.export_data(ExportFormat::JSON).await.unwrap();
        assert!(json.contains("session_id"));
    }
}