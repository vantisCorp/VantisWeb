//! Network Activity Analyzer
//! 
//! This module provides comprehensive network monitoring and analysis capabilities
//! for tracking HTTP/HTTPS requests, WebSocket connections, and network performance.
//! 
//! # Features
//! - HTTP request/response tracking
//! - Network timing and performance metrics
//! - Request/response body inspection
//! - WebSocket message monitoring
//! - Network waterfall visualization
//! - Bandwidth usage tracking

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use url::Url;

/// Network request method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
    CONNECT,
    TRACE,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::GET => write!(f, "GET"),
            HttpMethod::POST => write!(f, "POST"),
            HttpMethod::PUT => write!(f, "PUT"),
            HttpMethod::DELETE => write!(f, "DELETE"),
            HttpMethod::PATCH => write!(f, "PATCH"),
            HttpMethod::HEAD => write!(f, "HEAD"),
            HttpMethod::OPTIONS => write!(f, "OPTIONS"),
            HttpMethod::CONNECT => write!(f, "CONNECT"),
            HttpMethod::TRACE => write!(f, "TRACE"),
        }
    }
}

/// Resource type for network requests
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    Document,
    Stylesheet,
    Image,
    Media,
    Font,
    Script,
    TextTrack,
    XHR,
    Fetch,
    EventSource,
    WebSocket,
    Manifest,
    Other,
}

/// Network request information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequest {
    pub id: String,
    pub url: String,
    pub method: HttpMethod,
    pub resource_type: ResourceType,
    pub initiated_time: DateTime<Utc>,
    pub request_headers: HashMap<String, String>,
    pub post_data: Option<String>,
}

/// Network response information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkResponse {
    pub status: u16,
    pub status_text: String,
    pub response_headers: HashMap<String, String>,
    pub body_size: u64,
    pub encoded_body_size: u64,
}

/// Detailed network timing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTiming {
    pub start_time: Instant,
    pub dns_start: Option<Instant>,
    pub dns_end: Option<Instant>,
    pub tcp_start: Option<Instant>,
    pub tcp_end: Option<Instant>,
    pub tls_start: Option<Instant>,
    pub tls_end: Option<Instant>,
    pub request_start: Option<Instant>,
    pub response_start: Option<Instant>,
    pub end_time: Instant,
}

impl NetworkTiming {
    /// Calculate total duration
    pub fn total_duration(&self) -> Duration {
        self.end_time.duration_since(self.start_time)
    }

    /// Calculate DNS resolution time
    pub fn dns_duration(&self) -> Option<Duration> {
        self.dns_end.and_then(|end| {
            self.dns_start.map(|start| end.duration_since(start))
        })
    }

    /// Calculate TCP connection time
    pub fn tcp_duration(&self) -> Option<Duration> {
        self.tcp_end.and_then(|end| {
            self.tcp_start.map(|start| end.duration_since(start))
        })
    }

    /// Calculate TLS handshake time
    pub fn tls_duration(&self) -> Option<Duration> {
        self.tls_end.and_then(|end| {
            self.tls_start.map(|start| end.duration_since(start))
        })
    }

    /// Calculate time to first byte (TTFB)
    pub fn ttfb(&self) -> Option<Duration> {
        self.response_start.and_then(|start| {
            Some(start.duration_since(self.start_time))
        })
    }

    /// Calculate download time
    pub fn download_duration(&self) -> Option<Duration> {
        self.response_start.and_then(|start| {
            Some(self.end_time.duration_since(start))
        })
    }
}

/// Complete network entry with all timing and data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEntry {
    pub request: NetworkRequest,
    pub response: Option<NetworkResponse>,
    pub timing: NetworkTiming,
    pub blocked_reason: Option<String>,
    pub is_failed: bool,
}

impl NetworkEntry {
    /// Calculate total data transferred
    pub fn total_size(&self) -> u64 {
        self.response.as_ref()
            .map(|r| r.body_size)
            .unwrap_or(0)
    }

    /// Get transfer rate in bytes per second
    pub fn transfer_rate(&self) -> Option<f64> {
        let duration = self.timing.total_duration().as_secs_f64();
        if duration > 0.0 {
            Some(self.total_size() as f64 / duration)
        } else {
            None
        }
    }

    /// Check if request is from same origin
    pub fn is_same_origin(&self) -> bool {
        if let Ok(url) = Url::parse(&self.request.url) {
            // Simplified check - in real implementation would compare with document origin
            url.scheme() != "data" && url.scheme() != "about"
        } else {
            false
        }
    }

    /// Get mime type from response
    pub fn mime_type(&self) -> Option<&String> {
        self.response.as_ref()
            .and_then(|r| r.response_headers.get("content-type"))
    }

    /// Check if response was cached
    pub fn is_cached(&self) -> bool {
        self.response.as_ref()
            .and_then(|r| r.response_headers.get("x-from-cache"))
            .is_some()
    }
}

/// WebSocket message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub timestamp: DateTime<Utc>,
    pub direction: MessageDirection,
    pub data: String,
    pub is_binary: bool,
    pub size: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageDirection {
    Sent,
    Received,
}

/// Network bandwidth statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthStats {
    pub upload_total: u64,
    pub download_total: u64,
    pub requests_count: usize,
    pub avg_request_size: f64,
    pub avg_response_size: f64,
    pub peak_upload_rate: f64,
    pub peak_download_rate: f64,
}

/// Network analyzer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAnalyzerConfig {
    pub max_entries: usize,
    pub capture_bodies: bool,
    pub max_body_size: usize,
    pub track_websockets: bool,
    pub record_timing: bool,
}

impl Default for NetworkAnalyzerConfig {
    fn default() -> Self {
        Self {
            max_entries: 10000,
            capture_bodies: true,
            max_body_size: 10 * 1024 * 1024, // 10MB
            track_websockets: true,
            record_timing: true,
        }
    }
}

/// Main network analyzer
pub struct NetworkAnalyzer {
    config: Arc<RwLock<NetworkAnalyzerConfig>>,
    entries: Arc<RwLock<Vec<NetworkEntry>>>,
    websockets: Arc<RwLock<HashMap<String, Vec<WebSocketMessage>>>>,
    bandwidth_history: Arc<RwLock<Vec<(DateTime<Utc>, BandwidthStats)>>>,
    is_recording: Arc<RwLock<bool>>,
}

impl NetworkAnalyzer {
    /// Create a new network analyzer
    pub fn new(config: NetworkAnalyzerConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            entries: Arc::new(RwLock::new(Vec::new())),
            websockets: Arc::new(RwLock::new(HashMap::new())),
            bandwidth_history: Arc::new(RwLock::new(Vec::new())),
            is_recording: Arc::new(RwLock::new(false)),
        }
    }

    /// Start recording network activity
    pub async fn start_recording(&self) {
        *self.is_recording.write().await = true;
        self.entries.write().await.clear();
        self.websockets.write().await.clear();
    }

    /// Stop recording network activity
    pub async fn stop_recording(&self) {
        *self.is_recording.write().await = false;
    }

    /// Check if currently recording
    pub async fn is_recording(&self) -> bool {
        *self.is_recording.read().await
    }

    /// Add a network request
    pub async fn add_request(&self, request: NetworkRequest) -> Result<(), String> {
        if !self.is_recording().await {
            return Ok(());
        }

        let timing = NetworkTiming {
            start_time: Instant::now(),
            dns_start: None,
            dns_end: None,
            tcp_start: None,
            tcp_end: None,
            tls_start: None,
            tls_end: None,
            request_start: None,
            response_start: None,
            end_time: Instant::now(),
        };

        let entry = NetworkEntry {
            request,
            response: None,
            timing,
            blocked_reason: None,
            is_failed: false,
        };

        let mut entries = self.entries.write().await;
        if entries.len() >= self.config.read().await.max_entries {
            entries.remove(0);
        }
        entries.push(entry);

        Ok(())
    }

    /// Update network entry with response
    pub async fn update_response(
        &self,
        id: &str,
        response: NetworkResponse,
        timing: NetworkTiming,
    ) -> Result<(), String> {
        let mut entries = self.entries.write().await;
        if let Some(entry) = entries.iter_mut().find(|e| e.request.id == id) {
            entry.response = Some(response);
            entry.timing = timing;
            Ok(())
        } else {
            Err(format!("Network entry not found: {}", id))
        }
    }

    /// Mark entry as failed
    pub async fn mark_failed(&self, id: &str, reason: String) -> Result<(), String> {
        let mut entries = self.entries.write().await;
        if let Some(entry) = entries.iter_mut().find(|e| e.request.id == id) {
            entry.is_failed = true;
            entry.blocked_reason = Some(reason);
            Ok(())
        } else {
            Err(format!("Network entry not found: {}", id))
        }
    }

    /// Add WebSocket message
    pub async fn add_websocket_message(&self, url: &str, message: WebSocketMessage) {
        if !self.is_recording().await {
            return;
        }

        let mut websockets = self.websockets.write().await;
        websockets
            .entry(url.to_string())
            .or_insert_with(Vec::new)
            .push(message);
    }

    /// Get all network entries
    pub async fn get_entries(&self) -> Vec<NetworkEntry> {
        self.entries.read().await.clone()
    }

    /// Get entries filtered by URL
    pub async fn get_entries_by_url(&self, url_pattern: &str) -> Vec<NetworkEntry> {
        self.entries.read().await
            .iter()
            .filter(|e| e.request.url.contains(url_pattern))
            .cloned()
            .collect()
    }

    /// Get entries filtered by resource type
    pub async fn get_entries_by_type(&self, resource_type: ResourceType) -> Vec<NetworkEntry> {
        self.entries.read().await
            .iter()
            .filter(|e| e.request.resource_type == resource_type)
            .cloned()
            .collect()
    }

    /// Get failed requests
    pub async fn get_failed_requests(&self) -> Vec<NetworkEntry> {
        self.entries.read().await
            .iter()
            .filter(|e| e.is_failed)
            .cloned()
            .collect()
    }

    /// Get WebSocket messages for a URL
    pub async fn get_websocket_messages(&self, url: &str) -> Vec<WebSocketMessage> {
        self.websockets.read().await
            .get(url)
            .cloned()
            .unwrap_or_default()
    }

    /// Calculate current bandwidth statistics
    pub async fn calculate_bandwidth(&self) -> BandwidthStats {
        let entries = self.entries.read().await;
        let requests_count = entries.len();

        if requests_count == 0 {
            return BandwidthStats {
                upload_total: 0,
                download_total: 0,
                requests_count: 0,
                avg_request_size: 0.0,
                avg_response_size: 0.0,
                peak_upload_rate: 0.0,
                peak_download_rate: 0.0,
            };
        }

        let upload_total: u64 = 0; // Would need request body tracking
        let download_total: u64 = entries.iter()
            .map(|e| e.total_size())
            .sum();

        let avg_response_size = download_total as f64 / requests_count as f64;

        // Calculate peak rates from transfer rates
        let mut peak_upload_rate = 0.0;
        let mut peak_download_rate = 0.0;
        for entry in entries.iter() {
            if let Some(rate) = entry.transfer_rate() {
                if rate > peak_download_rate {
                    peak_download_rate = rate;
                }
            }
        }

        BandwidthStats {
            upload_total,
            download_total,
            requests_count,
            avg_request_size: 0.0,
            avg_response_size,
            peak_upload_rate,
            peak_download_rate,
        }
    }

    /// Get bandwidth history
    pub async fn get_bandwidth_history(&self) -> Vec<(DateTime<Utc>, BandwidthStats)> {
        self.bandwidth_history.read().await.clone()
    }

    /// Generate waterfall data for visualization
    pub async fn generate_waterfall(&self) -> Vec<WaterfallEntry> {
        self.entries.read().await
            .iter()
            .map(|entry| WaterfallEntry {
                name: self.extract_resource_name(&entry.request.url),
                url: entry.request.url.clone(),
                method: entry.request.method,
                start_time: entry.timing.start_time.elapsed().as_millis() as f64,
                total_time: entry.timing.total_duration().as_millis() as f64,
                dns_time: entry.timing.dns_duration().map(|d| d.as_millis() as f64),
                tcp_time: entry.timing.tcp_duration().map(|d| d.as_millis() as f64),
                tls_time: entry.timing.tls_duration().map(|d| d.as_millis() as f64),
                ttfb: entry.timing.ttfb().map(|d| d.as_millis() as f64),
                download_time: entry.timing.download_duration().map(|d| d.as_millis() as f64),
                size: entry.total_size(),
                status: entry.response.as_ref().map(|r| r.status),
                is_failed: entry.is_failed,
            })
            .collect()
    }

    fn extract_resource_name(&self, url: &str) -> String {
        if let Ok(parsed) = Url::parse(url) {
            parsed.path_segments()
                .and_then(|segments| segments.last())
                .unwrap_or("unknown")
                .to_string()
        } else {
            "unknown".to_string()
        }
    }

    /// Clear all recorded data
    pub async fn clear(&self) {
        self.entries.write().await.clear();
        self.websockets.write().await.clear();
        self.bandwidth_history.write().await.clear();
    }

    /// Get statistics summary
    pub async fn get_summary(&self) -> NetworkSummary {
        let entries = self.entries.read().await;

        let total_requests = entries.len();
        let failed_requests = entries.iter().filter(|e| e.is_failed).count();
        let total_size: u64 = entries.iter()
            .map(|e| e.total_size())
            .sum();

        let avg_response_time = if total_requests > 0 {
            entries.iter()
                .map(|e| e.timing.total_duration().as_millis() as f64)
                .sum::<f64>() / total_requests as f64
        } else {
            0.0
        };

        // Count by resource type
        let mut by_type: HashMap<ResourceType, usize> = HashMap::new();
        for entry in entries.iter() {
            *by_type.entry(entry.request.resource_type).or_insert(0) += 1;
        }

        // Count by status code
        let mut by_status: HashMap<u16, usize> = HashMap::new();
        for entry in entries.iter() {
            if let Some(ref response) = entry.response {
                *by_status.entry(response.status).or_insert(0) += 1;
            }
        }

        NetworkSummary {
            total_requests,
            failed_requests,
            total_size,
            avg_response_time,
            by_type,
            by_status,
        }
    }
}

/// Waterfall entry for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaterfallEntry {
    pub name: String,
    pub url: String,
    pub method: HttpMethod,
    pub start_time: f64,
    pub total_time: f64,
    pub dns_time: Option<f64>,
    pub tcp_time: Option<f64>,
    pub tls_time: Option<f64>,
    pub ttfb: Option<f64>,
    pub download_time: Option<f64>,
    pub size: u64,
    pub status: Option<u16>,
    pub is_failed: bool,
}

/// Network summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSummary {
    pub total_requests: usize,
    pub failed_requests: usize,
    pub total_size: u64,
    pub avg_response_time: f64,
    pub by_type: HashMap<ResourceType, usize>,
    pub by_status: HashMap<u16, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_method_display() {
        assert_eq!(HttpMethod::GET.to_string(), "GET");
        assert_eq!(HttpMethod::POST.to_string(), "POST");
    }

    #[test]
    fn test_network_timing_calculations() {
        let start = Instant::now();
        let timing = NetworkTiming {
            start_time: start,
            dns_start: Some(start),
            dns_end: Some(start + Duration::from_millis(10)),
            tcp_start: Some(start + Duration::from_millis(15)),
            tcp_end: Some(start + Duration::from_millis(30)),
            tls_start: Some(start + Duration::from_millis(30)),
            tls_end: Some(start + Duration::from_millis(45)),
            request_start: Some(start + Duration::from_millis(45)),
            response_start: Some(start + Duration::from_millis(50)),
            end_time: start + Duration::from_millis(100),
        };

        assert_eq!(timing.total_duration().as_millis(), 100);
        assert_eq!(timing.dns_duration().unwrap().as_millis(), 10);
        assert_eq!(timing.tcp_duration().unwrap().as_millis(), 15);
        assert_eq!(timing.tls_duration().unwrap().as_millis(), 15);
        assert_eq!(timing.ttfb().unwrap().as_millis(), 50);
        assert_eq!(timing.download_duration().unwrap().as_millis(), 50);
    }
}