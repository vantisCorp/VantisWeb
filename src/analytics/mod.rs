//! Analytics Module for VantisWeb Browser
//! 
//! This module provides comprehensive analytics and telemetry functionality
//! for tracking browsing habits, performance metrics, and security statistics.
//! All data is stored locally to respect user privacy.

pub mod dashboard;
pub mod metrics;
pub mod storage;
pub mod exporter;
pub mod privacy;

pub use dashboard::{DashboardBuilder, DashboardConfig, DashboardData, DashboardComponent};
pub use metrics::{UsageMetrics, PerformanceMetrics, SecurityMetrics, AnalyticsSnapshot};
pub use storage::AnalyticsStorage;
pub use exporter::{AnalyticsExporter, ExportFormat, TimeRange};
pub use privacy::{PrivacyFilter, PrivacyConfig};

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Main analytics engine that coordinates all analytics functionality
pub struct AnalyticsEngine {
    storage: Arc<RwLock<AnalyticsStorage>>,
    privacy_filter: PrivacyFilter,
    config: AnalyticsConfig,
}

/// Configuration for analytics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    /// Enable/disable analytics collection
    pub enabled: bool,
    /// Collect browsing history statistics
    pub track_browsing: bool,
    /// Collect performance metrics
    pub track_performance: bool,
    /// Collect security statistics
    pub track_security: bool,
    /// Data retention period in days
    pub retention_days: u32,
    /// Minimum data points before aggregation
    pub min_data_points: u32,
    /// Database path for storage
    pub db_path: Option<String>,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            track_browsing: true,
            track_performance: true,
            track_security: true,
            retention_days: 90,
            min_data_points: 10,
            db_path: None,
        }
    }
}

impl AnalyticsEngine {
    /// Create a new analytics engine with default configuration
    pub fn new() -> Self {
        Self::with_config(AnalyticsConfig::default())
    }

    /// Create a new analytics engine with custom configuration
    pub fn with_config(config: AnalyticsConfig) -> Self {
        let storage = match &config.db_path {
            Some(path) => AnalyticsStorage::with_path(path),
            None => AnalyticsStorage::new(),
        };
        
        Self {
            storage: Arc::new(RwLock::new(storage)),
            privacy_filter: PrivacyFilter::with_defaults(),
            config,
        }
    }

    /// Record a page visit
    pub async fn record_visit(&self, url: &str, duration_ms: u64) -> Result<(), AnalyticsError> {
        if !self.config.enabled || !self.config.track_browsing {
            return Ok(());
        }

        if !self.privacy_filter.should_track_url(url) {
            return Ok(());
        }

        let sanitized_url = self.privacy_filter.sanitize_url(url);
        let domain = self.privacy_filter.extract_domain(url);
        
        let mut storage = self.storage.write().await;
        storage.record_visit(&sanitized_url, domain.as_deref(), duration_ms)?;
        
        Ok(())
    }

    /// Record performance metrics for a page load
    pub async fn record_performance(&self, metrics: PageLoadMetrics) -> Result<(), AnalyticsError> {
        if !self.config.enabled || !self.config.track_performance {
            return Ok(());
        }

        let mut storage = self.storage.write().await;
        storage.record_page_load(&metrics)?;
        
        Ok(())
    }

    /// Record a security event
    pub async fn record_security_event(&self, event: SecurityEvent) -> Result<(), AnalyticsError> {
        if !self.config.enabled || !self.config.track_security {
            return Ok(());
        }

        let mut storage = self.storage.write().await;
        storage.record_security_event(&event)?;
        
        Ok(())
    }

    /// Get current analytics snapshot
    pub async fn get_snapshot(&self) -> Result<AnalyticsSnapshot, AnalyticsError> {
        let storage = self.storage.read().await;
        storage.get_snapshot()
    }

    /// Export analytics data in the specified format
    pub async fn export(
        &self, 
        format: ExportFormat, 
        path: &str
    ) -> Result<(), AnalyticsError> {
        let storage = self.storage.read().await;
        let snapshot = storage.get_snapshot()?;
        
        let exporter = AnalyticsExporter::with_defaults();
        exporter.export(&snapshot, format, path)?;
        
        Ok(())
    }

    /// Clear all analytics data
    pub async fn clear_data(&self) -> Result<(), AnalyticsError> {
        let mut storage = self.storage.write().await;
        storage.clear()?;
        
        Ok(())
    }

    /// Get the dashboard data for UI rendering
    pub async fn get_dashboard_data(&self) -> Result<DashboardData, AnalyticsError> {
        let storage = self.storage.read().await;
        let snapshot = storage.get_snapshot()?;
        
        let builder = DashboardBuilder::new(DashboardConfig::default());
        Ok(builder.build(&snapshot))
    }

    /// Update configuration
    pub fn update_config(&mut self, config: AnalyticsConfig) {
        self.config = config;
    }

    /// Check if analytics is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

impl Default for AnalyticsEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Page load performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageLoadMetrics {
    pub url: String,
    pub dns_time_ms: u64,
    pub connect_time_ms: u64,
    pub ttfb_ms: u64,
    pub dom_content_loaded_ms: u64,
    pub load_complete_ms: u64,
    pub total_bytes: u64,
    pub request_count: u32,
    pub timestamp: DateTime<Utc>,
}

impl PageLoadMetrics {
    /// Create a new page load metrics with current timestamp
    pub fn new(url: String) -> Self {
        Self {
            url,
            dns_time_ms: 0,
            connect_time_ms: 0,
            ttfb_ms: 0,
            dom_content_loaded_ms: 0,
            load_complete_ms: 0,
            total_bytes: 0,
            request_count: 0,
            timestamp: Utc::now(),
        }
    }
    
    /// Get total page load time
    pub fn total_load_time_ms(&self) -> u64 {
        self.load_complete_ms
    }
    
    /// Check if this was a slow page load (>3 seconds)
    pub fn is_slow(&self) -> bool {
        self.load_complete_ms > 3000
    }
}

/// Security event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEvent {
    /// A threat was blocked
    ThreatBlocked { 
        threat_type: String, 
        url: String 
    },
    /// HTTP was upgraded to HTTPS
    HttpsUpgrade { 
        url: String 
    },
    /// A tracking attempt was blocked
    TrackingAttempt { 
        tracker: String, 
        url: String 
    },
    /// A malicious site was blocked
    MaliciousSiteBlocked { 
        url: String, 
        reason: String 
    },
    /// A phishing attempt was blocked
    PhishingAttemptBlocked { 
        url: String 
    },
    /// A cookie was blocked
    CookieBlocked { 
        domain: String 
    },
    /// A fingerprinting attempt was detected
    FingerprintingAttempt { 
        technique: String 
    },
}

impl SecurityEvent {
    /// Get the severity level of this event
    pub fn severity(&self) -> SecuritySeverity {
        match self {
            SecurityEvent::MaliciousSiteBlocked(_) |
            SecurityEvent::PhishingAttemptBlocked(_) => SecuritySeverity::Critical,
            SecurityEvent::ThreatBlocked { .. } |
            SecurityEvent::FingerprintingAttempt { .. } => SecuritySeverity::High,
            SecurityEvent::TrackingAttempt { .. } => SecuritySeverity::Medium,
            SecurityEvent::HttpsUpgrade { .. } |
            SecurityEvent::CookieBlocked { .. } => SecuritySeverity::Low,
        }
    }
    
    /// Get a human-readable description
    pub fn description(&self) -> String {
        match self {
            SecurityEvent::ThreatBlocked { threat_type, url } => 
                format!("{} threat blocked from {}", threat_type, url),
            SecurityEvent::HttpsUpgrade { url } => 
                format!("Upgraded to HTTPS: {}", url),
            SecurityEvent::TrackingAttempt { tracker, url } => 
                format!("Tracker {} blocked on {}", tracker, url),
            SecurityEvent::MaliciousSiteBlocked { url, reason } => 
                format!("Malicious site {} blocked: {}", url, reason),
            SecurityEvent::PhishingAttemptBlocked { url } => 
                format!("Phishing attempt blocked: {}", url),
            SecurityEvent::CookieBlocked { domain } => 
                format!("Third-party cookie blocked from {}", domain),
            SecurityEvent::FingerprintingAttempt { technique } => 
                format!("Browser fingerprinting attempt detected: {}", technique),
        }
    }
}

/// Security event severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Analytics error types
#[derive(Debug, thiserror::Error)]
pub enum AnalyticsError {
    #[error("Storage error: {0}")]
    StorageError(#[from] std::io::Error),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Export error: {0}")]
    ExportError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Privacy filter error: {0}")]
    PrivacyError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_config_default() {
        let config = AnalyticsConfig::default();
        assert!(config.enabled);
        assert!(config.track_browsing);
        assert_eq!(config.retention_days, 90);
    }

    #[test]
    fn test_analytics_engine_creation() {
        let engine = AnalyticsEngine::new();
        assert!(engine.is_enabled());
    }

    #[test]
    fn test_page_load_metrics() {
        let metrics = PageLoadMetrics::new("https://example.com".to_string());
        assert!(!metrics.is_slow());
        
        let mut slow_metrics = metrics.clone();
        slow_metrics.load_complete_ms = 5000;
        assert!(slow_metrics.is_slow());
    }

    #[test]
    fn test_security_event_severity() {
        let phishing = SecurityEvent::PhishingAttemptBlocked { 
            url: "https://evil.com".to_string() 
        };
        assert_eq!(phishing.severity(), SecuritySeverity::Critical);
        
        let cookie = SecurityEvent::CookieBlocked { 
            domain: "tracker.com".to_string() 
        };
        assert_eq!(cookie.severity(), SecuritySeverity::Low);
    }
}

#[cfg(test)]
mod tests_module;