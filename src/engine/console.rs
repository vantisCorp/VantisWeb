//! Console API
//! 
//! Console logging and debugging:
//! - Console logging
//! - Console methods (log, warn, error)
//! - Console object
//! - Debug integration
//! - Performance metrics

use anyhow::Result;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::core::kernel::VantisKernel;

/// Log level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogLevel {
    /// Log level
    Log,
    /// Info level
    Info,
    /// Warn level
    Warn,
    /// Error level
    Error,
    /// Debug level
    Debug,
}

/// Console entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleEntry {
    /// Log level
    pub level: LogLevel,
    /// Message
    pub message: String,
    /// Arguments
    pub args: Vec<String>,
    /// Timestamp
    pub timestamp: i64,
    /// Source file (if available)
    pub source: Option<String>,
    /// Line number (if available)
    pub line: Option<u32>,
}

impl ConsoleEntry {
    /// Create a new console entry
    pub fn new(level: LogLevel, message: String) -> Self {
        Self {
            level,
            message,
            args: Vec::new(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            source: None,
            line: None,
        }
    }

    /// Set source information
    pub fn with_source(mut self, source: String, line: u32) -> Self {
        self.source = Some(source);
        self.line = Some(line);
        self
    }

    /// Add argument
    pub fn with_arg(mut self, arg: String) -> Self {
        self.args.push(arg);
        self
    }
}

/// Performance metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    /// Metric name
    pub name: String,
    /// Duration in milliseconds
    pub duration_ms: f64,
    /// Start timestamp
    pub start_time: i64,
    /// End timestamp
    pub end_time: i64,
    /// Additional data
    pub data: Option<serde_json::Value>,
}

impl PerformanceMetric {
    /// Create a new performance metric
    pub fn new(name: String, duration_ms: f64) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            name,
            duration_ms,
            start_time: now,
            end_time: now,
            data: None,
        }
    }

    /// Set additional data
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }
}

/// Console API
pub struct ConsoleApi {
    kernel: Arc<VantisKernel>,
    /// Console entries
    entries: Arc<RwLock<Vec<ConsoleEntry>>>,
    /// Performance metrics
    metrics: Arc<RwLock<HashMap<String, PerformanceMetric>>>,
    /// Active performance timers
    timers: Arc<RwLock<HashMap<String, i64>>>,
    /// Maximum entries to keep
    max_entries: usize,
}

impl ConsoleApi {
    /// Create a new Console API
    pub fn new(kernel: Arc<VantisKernel>) -> Self {
        info!("Initializing Console API...");

        Self {
            kernel,
            entries: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(RwLock::new(HashMap::new())),
            timers: Arc::new(RwLock::new(HashMap::new())),
            max_entries: 1000,
        }
    }

    /// Log a message
    pub async fn log(&self, level: LogLevel, message: String, args: Vec<String>) {
        let entry = ConsoleEntry::new(level.clone(), message.clone());
        
        // Add arguments
        let mut entry_with_args = entry;
        for arg in args {
            entry_with_args = entry_with_args.with_arg(arg);
        }

        // Log based on level
        match level {
            LogLevel::Log => info!("{}", message),
            LogLevel::Info => info!("{}", message),
            LogLevel::Warn => warn!("{}", message),
            LogLevel::Error => log::error!("{}", message),
            LogLevel::Debug => debug!("{}", message),
        }

        // Store entry
        let mut entries = self.entries.write().await;
        entries.push(entry_with_args);

        // Trim if too many entries
        let len = entries.len();
        if len > self.max_entries {
            entries.drain(0..len - self.max_entries);
        }
    }

    /// Log a standard log message
    pub async fn console_log(&self, message: String) {
        self.log(LogLevel::Log, message, Vec::new()).await;
    }

    /// Log an info message
    pub async fn console_info(&self, message: String) {
        self.log(LogLevel::Info, message, Vec::new()).await;
    }

    /// Log a warning message
    pub async fn console_warn(&self, message: String) {
        self.log(LogLevel::Warn, message, Vec::new()).await;
    }

    /// Log an error message
    pub async fn console_error(&self, message: String) {
        self.log(LogLevel::Error, message, Vec::new()).await;
    }

    /// Log a debug message
    pub async fn console_debug(&self, message: String) {
        self.log(LogLevel::Debug, message, Vec::new()).await;
    }

    /// Clear all console entries
    pub async fn clear(&self) {
        info!("Clearing console");
        self.entries.write().await.clear();
    }

    /// Get all console entries
    pub async fn get_entries(&self) -> Vec<ConsoleEntry> {
        self.entries.read().await.clone()
    }

    /// Get entries by log level
    pub async fn get_entries_by_level(&self, level: LogLevel) -> Vec<ConsoleEntry> {
        self.entries
            .read()
            .await
            .iter()
            .filter(|entry| entry.level == level)
            .cloned()
            .collect()
    }

    /// Get entry count
    pub async fn count(&self) -> usize {
        self.entries.read().await.len()
    }

    /// Start a performance timer
    pub async fn time(&self, name: String) {
        let start_time = chrono::Utc::now().timestamp_millis();
        self.timers.write().await.insert(name.clone(), start_time);
        debug!("Timer started: {}", name);
    }

    /// End a performance timer
    pub async fn time_end(&self, name: String) -> Result<f64> {
        let end_time = chrono::Utc::now().timestamp_millis();
        
        if let Some(start_time) = self.timers.write().await.remove(&name) {
            let duration = (end_time - start_time) as f64;
            
            let metric = PerformanceMetric::new(name.clone(), duration)
                .with_data(serde_json::json!({
                    "start_time": start_time,
                    "end_time": end_time
                }));

            self.metrics.write().await.insert(name.clone(), metric);
            
            info!("Timer ended: {} ({}ms)", name, duration);
            
            Ok(duration)
        } else {
            warn!("Timer not found: {}", name);
            Err(anyhow::anyhow!("Timer not found: {}", name))
        }
    }

    /// Record a performance metric
    pub async fn record_metric(&self, metric: PerformanceMetric) {
        self.metrics.write().await.insert(metric.name.clone(), metric);
    }

    /// Get a performance metric
    pub async fn get_metric(&self, name: String) -> Option<PerformanceMetric> {
        self.metrics.read().await.get(&name).cloned()
    }

    /// Get all performance metrics
    pub async fn get_metrics(&self) -> Vec<PerformanceMetric> {
        self.metrics.read().await.values().cloned().collect()
    }

    /// Clear all performance metrics
    pub async fn clear_metrics(&self) {
        info!("Clearing performance metrics");
        self.metrics.write().await.clear();
        self.timers.write().await.clear();
    }

    /// Assert that a condition is true
    pub async fn assert(&self, condition: bool, message: Option<String>) -> bool {
        if condition {
            self.console_log("Assertion passed".to_string()).await;
            true
        } else {
            let msg = message.unwrap_or_else(|| "Assertion failed".to_string());
            self.console_error(format!("Assertion failed: {}", msg)).await;
            false
        }
    }

    /// Count entries by level
    pub async fn count_by_level(&self, level: LogLevel) -> usize {
        self.entries
            .read()
            .await
            .iter()
            .filter(|entry| entry.level == level)
            .count()
    }

    /// Get the last N entries
    pub async fn get_last_entries(&self, n: usize) -> Vec<ConsoleEntry> {
        let entries = self.entries.read().await;
        let start = if entries.len() > n { entries.len() - n } else { 0 };
        entries[start..].to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_console_api() -> ConsoleApi {
        let kernel = Arc::new(VantisKernel::new().await.unwrap());
        ConsoleApi::new(kernel)
    }

    #[tokio::test]
    async fn test_console_api_creation() {
        let console_api = create_console_api().await;
        assert_eq!(console_api.count().await, 0);
    }

    #[tokio::test]
    async fn test_console_log() {
        let console_api = create_console_api().await;

        console_api.console_log("Test message".to_string()).await;

        assert_eq!(console_api.count().await, 1);
        
        let entries = console_api.get_entries().await;
        assert_eq!(entries[0].message, "Test message");
        assert_eq!(entries[0].level, LogLevel::Log);
    }

    #[tokio::test]
    async fn test_console_error() {
        let console_api = create_console_api().await;

        console_api.console_error("Error message".to_string()).await;

        assert_eq!(console_api.count().await, 1);
        
        let entries = console_api.get_entries().await;
        assert_eq!(entries[0].level, LogLevel::Error);
    }

    #[tokio::test]
    async fn test_console_clear() {
        let console_api = create_console_api().await;

        console_api.console_log("Message 1".to_string()).await;
        console_api.console_log("Message 2".to_string()).await;

        assert_eq!(console_api.count().await, 2);

        console_api.clear().await;

        assert_eq!(console_api.count().await, 0);
    }

    #[tokio::test]
    async fn test_console_timer() {
        let console_api = create_console_api().await;

        console_api.time("test_timer".to_string()).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        let duration = console_api.time_end("test_timer".to_string()).await.unwrap();

        assert!(duration >= 100.0);
    }

    #[tokio::test]
    async fn test_console_metrics() {
        let console_api = create_console_api().await;

        console_api.time("test_timer".to_string()).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        console_api.time_end("test_timer".to_string()).await.unwrap();

        let metric = console_api.get_metric("test_timer".to_string()).await;
        assert!(metric.is_some());
        assert_eq!(metric.unwrap().name, "test_timer");
    }

    #[tokio::test]
    async fn test_console_assert() {
        let console_api = create_console_api().await;

        let result = console_api.assert(true, None).await;
        assert!(result);

        let result = console_api.assert(false, Some("Test failed".to_string())).await;
        assert!(!result);
    }

    #[tokio::test]
    async fn test_console_get_by_level() {
        let console_api = create_console_api().await;

        console_api.console_log("Log message".to_string()).await;
        console_api.console_error("Error message".to_string()).await;
        console_api.console_warn("Warning message".to_string()).await;

        let error_entries = console_api.get_entries_by_level(LogLevel::Error).await;
        assert_eq!(error_entries.len(), 1);
        assert_eq!(error_entries[0].message, "Error message");
    }

    #[tokio::test]
    async fn test_console_max_entries() {
        let mut console_api = create_console_api();
        console_api.max_entries = 5;

        for i in 0..10 {
            console_api.console_log(format!("Message {}", i)).await;
        }

        // Should only keep last 5 entries
        assert_eq!(console_api.count().await, 5);
    }

    #[tokio::test]
    async fn test_console_get_last_entries() {
        let console_api = create_console_api().await;

        for i in 0..10 {
            console_api.console_log(format!("Message {}", i)).await;
        }

        let last_3 = console_api.get_last_entries(3).await;
        assert_eq!(last_3.len(), 3);
        assert_eq!(last_3[0].message, "Message 7");
        assert_eq!(last_3[2].message, "Message 9");
    }
}