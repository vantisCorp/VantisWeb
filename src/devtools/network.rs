// Copyright 2024 Vantis Corporation - All Rights Reserved

//! Network Monitor Module
//! 
//! This module provides network request/response monitoring, HAR export,
//! timeline visualization, and performance metrics for the developer tools.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Network request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequest {
    /// Request ID
    pub id: Uuid,
    /// URL
    pub url: String,
    /// Method
    pub method: HttpMethod,
    /// Headers
    pub headers: HashMap<String, String>,
    /// Request body
    pub body: Option<Vec<u8>>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Initiator
    pub initiator: Option<RequestInitiator>,
    /// Request state
    pub state: RequestState,
    /// Response
    pub response: Option<NetworkResponse>,
    /// Priority
    pub priority: RequestPriority,
    /// Resource type
    pub resource_type: ResourceType,
    /// Request timing
    pub timing: Option<ResourceTiming>,
    /// Size
    pub size: Option<ResourceSize>,
    /// Protocol
    pub protocol: Option<String>,
}

/// HTTP method
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

/// Request initiator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestInitiator {
    /// Initiator type
    pub init_type: InitiatorType,
    /// Source URL
    pub source_url: Option<String>,
    /// Line number
    pub line_number: Option<u32>,
    /// Column number
    pub column: Option<u32>,
    /// Stack trace
    pub stack_trace: Option<String>,
}

/// Initiator type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InitiatorType {
    Parser,
    Script,
    Redirect,
    Other,
}

/// Request state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RequestState {
    Pending,
    Loading,
    Complete,
    Error,
    Cancelled,
}

/// Network response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkResponse {
    /// Status code
    pub status_code: u16,
    /// Status text
    pub status_text: String,
    /// Headers
    pub headers: HashMap<String, String>,
    /// Response body
    pub body: Option<Vec<u8>>,
    /// Content type
    pub content_type: Option<String>,
    /// Size
    pub size: u64,
    /// Encoding
    pub encoding: Option<String>,
    /// Security details
    pub security_details: Option<SecurityDetails>,
}

/// Resource timing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTiming {
    /// Start time
    pub start_time: f64,
    /// DNS lookup
    pub dns_lookup: f64,
    /// TCP connect
    pub tcp_connect: f64,
    /// SSL handshake
    pub ssl_handshake: Option<f64>,
    /// Request start
    pub request_start: f64,
    /// Response start
    pub response_start: f64,
    /// Response end
    pub response_end: f64,
    /// Total duration
    pub duration: f64,
}

/// Resource size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSize {
    /// Request size
    pub request: u64,
    /// Response size
    pub response: u64,
    /// Encoded size
    pub encoded: u64,
    /// Decoded size
    pub decoded: u64,
}

/// Security details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityDetails {
    /// Protocol
    pub protocol: String,
    /// Key exchange
    pub key_exchange: String,
    /// Cipher
    pub cipher: String,
    /// Certificate issuer
    pub issuer: String,
    /// Certificate subject
    pub subject: String,
    /// Valid from
    pub valid_from: DateTime<Utc>,
    /// Valid to
    pub valid_to: DateTime<Utc>,
}

/// Request priority
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RequestPriority {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}

/// Resource type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    SignedExchange,
    Ping,
    CSPViolationReport,
    Other,
}

/// Network filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkFilter {
    /// Filter by URL pattern
    pub url_pattern: Option<String>,
    /// Filter by method
    pub method: Option<HttpMethod>,
    /// Filter by status code
    pub status_code: Option<u16>,
    /// Filter by resource type
    pub resource_type: Option<ResourceType>,
    /// Filter by domain
    pub domain: Option<String>,
    /// Filter by minimum size
    pub min_size: Option<u64>,
    /// Filter by maximum size
    pub max_size: Option<u64>,
    /// Filter by minimum duration
    pub min_duration: Option<f64>,
    /// Filter by maximum duration
    pub max_duration: Option<f64>,
    /// Filter by has errors
    pub has_errors: Option<bool>,
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatistics {
    /// Total requests
    pub total_requests: usize,
    /// Total bytes transferred
    pub total_bytes: u64,
    /// Average response time
    pub avg_response_time: f64,
    /// Requests by type
    pub requests_by_type: HashMap<String, usize>,
    /// Requests by status
    pub requests_by_status: HashMap<u16, usize>,
    /// Failed requests
    pub failed_requests: usize,
    /// Cached requests
    pub cached_requests: usize,
}

/// HAR entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HAREntry {
    /// Start time (ISO 8601)
    pub startedDateTime: String,
    /// Request
    pub request: HARRequest,
    /// Response
    pub response: HARResponse,
    /// Cache info
    pub cache: HARCache,
    /// Timings
    pub timings: HARTimings,
    /// Time in ms
    pub time: f64,
}

/// HAR request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HARRequest {
    /// Method
    pub method: String,
    /// URL
    pub url: String,
    /// HTTP version
    pub httpVersion: String,
    /// Headers
    pub headers: Vec<HARNameValuePair>,
    /// Query string
    pub queryString: Vec<HARNameValuePair>,
    /// Post data
    pub postData: Option<HARPostData>,
    /// Header size
    pub headerSize: i64,
    /// Body size
    pub bodySize: i64,
}

/// HAR response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HARResponse {
    /// Status
    pub status: u16,
    /// Status text
    pub statusText: String,
    /// HTTP version
    pub httpVersion: String,
    /// Headers
    pub headers: Vec<HARNameValuePair>,
    /// Content
    pub content: HARContent,
    /// Redirect URL
    pub redirectURL: String,
    /// Header size
    pub headerSize: i64,
    /// Body size
    pub bodySize: i64,
}

/// HAR content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HARContent {
    /// Size
    pub size: u64,
    /// Compression
    pub compression: Option<i64>,
    /// MIME type
    pub mimeType: String,
    /// Text
    pub text: Option<String>,
    /// Encoding
    pub encoding: Option<String>,
}

/// HAR cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HARCache {
    /// Before request
    pub beforeRequest: Option<HARCacheEntry>,
    /// After request
    pub afterRequest: Option<HARCacheEntry>,
}

/// HAR cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HARCacheEntry {
    /// Expires
    pub expires: Option<String>,
    /// Last accessed
    pub lastAccess: String,
    /// ETag
    pub eTag: String,
    /// Hit count
    pub hitCount: u32,
}

/// HAR timings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HARTimings {
    /// DNS
    pub dns: f64,
    /// Connect
    pub connect: f64,
    /// SSL
    pub ssl: Option<f64>,
    /// Send
    pub send: f64,
    /// Wait
    pub wait: f64,
    /// Receive
    pub receive: f64,
}

/// HAR name-value pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HARNameValuePair {
    /// Name
    pub name: String,
    /// Value
    pub value: String,
}

/// HAR log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HARLog {
    /// Version
    pub version: String,
    /// Creator
    pub creator: HARCreator,
    /// Browser
    pub browser: Option<HARCreator>,
    /// Pages
    pub pages: Vec<HARPage>,
    /// Entries
    pub entries: Vec<HAREntry>,
}

/// HAR creator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HARCreator {
    /// Name
    pub name: String,
    /// Version
    pub version: String,
}

/// HAR page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HARPage {
    /// Start time
    pub startedDateTime: String,
    /// ID
    pub id: String,
    /// Title
    pub title: String,
    /// Page timings
    pub pageTimings: HARPageTimings,
}

/// HAR page timings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HARPageTimings {
    /// On content load
    pub onContentLoad: Option<f64>,
    /// On load
    pub onLoad: Option<f64>,
}

/// Network monitor
pub struct NetworkMonitor {
    /// Requests
    requests: Arc<RwLock<HashMap<Uuid, NetworkRequest>>>,
    /// Request order
    request_order: Arc<RwLock<Vec<Uuid>>>,
    /// Filters
    filters: Arc<RwLock<Vec<NetworkFilter>>>,
    /// Blocked URLs
    blocked_urls: Arc<RwLock<Vec<String>>>,
    /// Throttling settings
    throttling: Arc<RwLock<Option<ThrottlingSettings>>>,
    /// Preserve log
    preserve_log: Arc<RwLock<bool>>,
}

/// Throttling settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThrottlingSettings {
    /// Download speed (Kbps)
    pub download_speed: u64,
    /// Upload speed (Kbps)
    pub upload_speed: u64,
    /// Latency (ms)
    pub latency: u64,
    /// Enabled
    pub enabled: bool,
}

impl NetworkMonitor {
    /// Create a new network monitor
    pub fn new() -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            request_order: Arc::new(RwLock::new(Vec::new())),
            filters: Arc::new(RwLock::new(Vec::new())),
            blocked_urls: Arc::new(RwLock::new(Vec::new())),
            throttling: Arc::new(RwLock::new(None)),
            preserve_log: Arc::new(RwLock::new(false)),
        }
    }

    /// Initialize the monitor
    pub fn initialize(&self) -> Result<(), NetworkError> {
        Ok(())
    }

    /// Start request
    pub async fn start_request(&self, request: NetworkRequest) -> Result<Uuid, NetworkError> {
        let id = request.id;
        
        self.requests.write().await.insert(id, request);
        self.request_order.write().await.push(id);
        
        Ok(id)
    }

    /// Complete request with response
    pub async fn complete_request(
        &self,
        id: Uuid,
        response: NetworkResponse,
        timing: Option<ResourceTiming>,
    ) -> Result<(), NetworkError> {
        let mut requests = self.requests.write().await;
        
        if let Some(mut req) = requests.remove(&id) {
            req.state = RequestState::Complete;
            req.response = Some(response);
            req.timing = timing;
            requests.insert(id, req);
            Ok(())
        } else {
            Err(NetworkError::RequestNotFound)
        }
    }

    /// Fail request
    pub async fn fail_request(&self, id: Uuid, error: String) -> Result<(), NetworkError> {
        let mut requests = self.requests.write().await;
        
        if let Some(mut req) = requests.remove(&id) {
            req.state = RequestState::Error;
            requests.insert(id, req);
            Ok(())
        } else {
            Err(NetworkError::RequestNotFound)
        }
    }

    /// Cancel request
    pub async fn cancel_request(&self, id: Uuid) -> Result<(), NetworkError> {
        let mut requests = self.requests.write().await;
        
        if let Some(mut req) = requests.remove(&id) {
            req.state = RequestState::Cancelled;
            requests.insert(id, req);
            Ok(())
        } else {
            Err(NetworkError::RequestNotFound)
        }
    }

    /// Get request by ID
    pub async fn get_request(&self, id: Uuid) -> Option<NetworkRequest> {
        self.requests.read().await.get(&id).cloned()
    }

    /// Get all requests
    pub async fn get_all_requests(&self) -> Vec<NetworkRequest> {
        let order = self.request_order.read().await;
        let requests = self.requests.read().await;
        
        order.iter()
            .filter_map(|id| requests.get(id).cloned())
            .collect()
    }

    /// Get filtered requests
    pub async fn get_filtered_requests(&self) -> Vec<NetworkRequest> {
        let filters = self.filters.read().await;
        let all = self.get_all_requests().await;
        
        if filters.is_empty() {
            return all;
        }
        
        all.into_iter()
            .filter(|req| {
                filters.iter().all(|f| self.matches_filter(req, f))
            })
            .collect()
    }

    /// Get request count
    pub async fn get_request_count(&self) -> usize {
        self.requests.read().await.len()
    }

    /// Add filter
    pub async fn add_filter(&self, filter: NetworkFilter) {
        self.filters.write().await.push(filter);
    }

    /// Clear filters
    pub async fn clear_filters(&self) {
        self.filters.write().await.clear();
    }

    /// Block URL
    pub async fn block_url(&self, url_pattern: String) {
        self.blocked_urls.write().await.push(url_pattern);
    }

    /// Unblock URL
    pub async fn unblock_url(&self, url_pattern: &str) -> bool {
        let mut blocked = self.blocked_urls.write().await;
        let initial_len = blocked.len();
        blocked.retain(|u| u != url_pattern);
        blocked.len() < initial_len
    }

    /// Set throttling
    pub async fn set_throttling(&self, settings: ThrottlingSettings) {
        *self.throttling.write().await = Some(settings);
    }

    /// Clear throttling
    pub async fn clear_throttling(&self) {
        *self.throttling.write().await = None;
    }

    /// Get statistics
    pub async fn get_statistics(&self) -> NetworkStatistics {
        let requests = self.requests.read().await;
        
        let mut stats = NetworkStatistics {
            total_requests: requests.len(),
            total_bytes: 0,
            avg_response_time: 0.0,
            requests_by_type: HashMap::new(),
            requests_by_status: HashMap::new(),
            failed_requests: 0,
            cached_requests: 0,
        };
        
        let mut total_duration = 0.0;
        let mut duration_count = 0;
        
        for req in requests.values() {
            // Count bytes
            if let Some(ref size) = req.size {
                stats.total_bytes += size.response;
            }
            
            // Count by type
            let type_name = match req.resource_type {
                ResourceType::Document => "Document",
                ResourceType::Stylesheet => "Stylesheet",
                ResourceType::Image => "Image",
                ResourceType::Media => "Media",
                ResourceType::Font => "Font",
                ResourceType::Script => "Script",
                ResourceType::XHR => "XHR",
                ResourceType::Fetch => "Fetch",
                ResourceType::WebSocket => "WebSocket",
                _ => "Other",
            };
            *stats.requests_by_type.entry(type_name.to_string()).or_insert(0) += 1;
            
            // Count by status
            if let Some(ref response) = req.response {
                *stats.requests_by_status.entry(response.status_code).or_insert(0) += 1;
            }
            
            // Track duration
            if let Some(ref timing) = req.timing {
                total_duration += timing.duration;
                duration_count += 1;
            }
            
            // Count errors
            if req.state == RequestState::Error {
                stats.failed_requests += 1;
            }
        }
        
        if duration_count > 0 {
            stats.avg_response_time = total_duration / duration_count as f64;
        }
        
        stats
    }

    /// Export HAR
    pub async fn export_har(&self) -> Result<String, NetworkError> {
        let requests = self.get_all_requests().await;
        
        let entries: Vec<HAREntry> = requests.iter()
            .filter_map(|req| self.request_to_har_entry(req))
            .collect();
        
        let log = HARLog {
            version: "1.2".to_string(),
            creator: HARCreator {
                name: "VantisWeb DevTools".to_string(),
                version: "1.0.0".to_string(),
            },
            browser: Some(HARCreator {
                name: "VantisWeb".to_string(),
                version: "1.0.0".to_string(),
            }),
            pages: vec![],
            entries,
        };
        
        serde_json::to_string(&log).map_err(NetworkError::from)
    }

    /// Clear all requests
    pub async fn clear(&self) {
        if !*self.preserve_log.read().await {
            self.requests.write().await.clear();
            self.request_order.write().await.clear();
        }
    }

    /// Set preserve log
    pub async fn set_preserve_log(&self, preserve: bool) {
        *self.preserve_log.write().await = preserve;
    }

    // Helper methods

    fn matches_filter(&self, req: &NetworkRequest, filter: &NetworkFilter) -> bool {
        if let Some(ref pattern) = filter.url_pattern {
            if !req.url.contains(pattern) {
                return false;
            }
        }
        
        if let Some(ref method) = filter.method {
            if req.method != *method {
                return false;
            }
        }
        
        if let Some(ref resource_type) = filter.resource_type {
            if req.resource_type != *resource_type {
                return false;
            }
        }
        
        if let Some(ref domain) = filter.domain {
            if !req.url.contains(domain) {
                return false;
            }
        }
        
        if let Some(min_size) = filter.min_size {
            if let Some(ref size) = req.size {
                if size.response < min_size {
                    return false;
                }
            }
        }
        
        if let Some(max_size) = filter.max_size {
            if let Some(ref size) = req.size {
                if size.response > max_size {
                    return false;
                }
            }
        }
        
        if let Some(min_duration) = filter.min_duration {
            if let Some(ref timing) = req.timing {
                if timing.duration < min_duration {
                    return false;
                }
            }
        }
        
        if let Some(max_duration) = filter.max_duration {
            if let Some(ref timing) = req.timing {
                if timing.duration > max_duration {
                    return false;
                }
            }
        }
        
        if let Some(has_errors) = filter.has_errors {
            let has_error = req.state == RequestState::Error;
            if has_errors != has_error {
                return false;
            }
        }
        
        true
    }

    fn request_to_har_entry(&self, req: &NetworkRequest) -> Option<HAREntry> {
        let response = req.response.as_ref()?;
        let timing = req.timing.as_ref()?;
        
        let har_request = HARRequest {
            method: match req.method {
                HttpMethod::GET => "GET",
                HttpMethod::POST => "POST",
                HttpMethod::PUT => "PUT",
                HttpMethod::DELETE => "DELETE",
                HttpMethod::PATCH => "PATCH",
                HttpMethod::HEAD => "HEAD",
                HttpMethod::OPTIONS => "OPTIONS",
                HttpMethod::CONNECT => "CONNECT",
                HttpMethod::TRACE => "TRACE",
            }.to_string(),
            url: req.url.clone(),
            httpVersion: "HTTP/1.1".to_string(),
            headers: req.headers.iter()
                .map(|(k, v)| HARNameValuePair { name: k.clone(), value: v.clone() })
                .collect(),
            queryString: vec![],
            postData: None,
            headerSize: -1,
            bodySize: -1,
        };
        
        let har_response = HARResponse {
            status: response.status_code,
            statusText: response.status_text.clone(),
            httpVersion: "HTTP/1.1".to_string(),
            headers: response.headers.iter()
                .map(|(k, v)| HARNameValuePair { name: k.clone(), value: v.clone() })
                .collect(),
            content: HARContent {
                size: response.size,
                compression: None,
                mimeType: response.content_type.clone().unwrap_or_default(),
                text: None,
                encoding: None,
            },
            redirectURL: String::new(),
            headerSize: -1,
            bodySize: response.size as i64,
        };
        
        Some(HAREntry {
            startedDateTime: req.timestamp.to_rfc3339(),
            request: har_request,
            response: har_response,
            cache: HARCache {
                beforeRequest: None,
                afterRequest: None,
            },
            timings: HARTimings {
                dns: timing.dns_lookup,
                connect: timing.tcp_connect,
                ssl: timing.ssl_handshake,
                send: 0.0,
                wait: timing.response_start - timing.request_start,
                receive: timing.response_end - timing.response_start,
            },
            time: timing.duration,
        })
    }
}

/// Network error
#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("Request not found")]
    RequestNotFound,
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Other error: {0}")]
    Other(String),
}

impl Default for NetworkMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_monitor_initialization() {
        let monitor = NetworkMonitor::new();
        assert!(monitor.initialize().is_ok());
    }

    #[tokio::test]
    async fn test_start_and_complete_request() {
        let monitor = NetworkMonitor::new();
        
        let request = NetworkRequest {
            id: Uuid::new_v4(),
            url: "https://example.com/api/data".to_string(),
            method: HttpMethod::GET,
            headers: HashMap::new(),
            body: None,
            timestamp: Utc::now(),
            initiator: None,
            state: RequestState::Pending,
            response: None,
            priority: RequestPriority::Medium,
            resource_type: ResourceType::XHR,
            timing: None,
            size: None,
            protocol: None,
        };
        
        let id = monitor.start_request(request).await.unwrap();
        
        let response = NetworkResponse {
            status_code: 200,
            status_text: "OK".to_string(),
            headers: HashMap::new(),
            body: Some(b"test data".to_vec()),
            content_type: Some("application/json".to_string()),
            size: 9,
            encoding: None,
            security_details: None,
        };
        
        monitor.complete_request(id, response, None).await.unwrap();
        
        let completed = monitor.get_request(id).await.unwrap();
        assert_eq!(completed.state, RequestState::Complete);
    }

    #[tokio::test]
    async fn test_get_all_requests() {
        let monitor = NetworkMonitor::new();
        
        for i in 0..3 {
            let request = NetworkRequest {
                id: Uuid::new_v4(),
                url: format!("https://example.com/api/{}", i),
                method: HttpMethod::GET,
                headers: HashMap::new(),
                body: None,
                timestamp: Utc::now(),
                initiator: None,
                state: RequestState::Pending,
                response: None,
                priority: RequestPriority::Medium,
                resource_type: ResourceType::XHR,
                timing: None,
                size: None,
                protocol: None,
            };
            monitor.start_request(request).await.unwrap();
        }
        
        let all = monitor.get_all_requests().await;
        assert_eq!(all.len(), 3);
    }

    #[tokio::test]
    async fn test_statistics() {
        let monitor = NetworkMonitor::new();
        
        let request = NetworkRequest {
            id: Uuid::new_v4(),
            url: "https://example.com/api/data".to_string(),
            method: HttpMethod::GET,
            headers: HashMap::new(),
            body: None,
            timestamp: Utc::now(),
            initiator: None,
            state: RequestState::Complete,
            response: Some(NetworkResponse {
                status_code: 200,
                status_text: "OK".to_string(),
                headers: HashMap::new(),
                body: None,
                content_type: None,
                size: 1024,
                encoding: None,
                security_details: None,
            }),
            priority: RequestPriority::Medium,
            resource_type: ResourceType::XHR,
            timing: Some(ResourceTiming {
                start_time: 0.0,
                dns_lookup: 10.0,
                tcp_connect: 20.0,
                ssl_handshake: Some(30.0),
                request_start: 40.0,
                response_start: 50.0,
                response_end: 100.0,
                duration: 100.0,
            }),
            size: Some(ResourceSize {
                request: 100,
                response: 1024,
                encoded: 512,
                decoded: 1024,
            }),
            protocol: Some("HTTP/2".to_string()),
        };
        
        monitor.start_request(request).await.unwrap();
        
        let stats = monitor.get_statistics().await;
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.total_bytes, 1024);
    }

    #[tokio::test]
    async fn test_export_har() {
        let monitor = NetworkMonitor::new();
        
        let request = NetworkRequest {
            id: Uuid::new_v4(),
            url: "https://example.com/api/data".to_string(),
            method: HttpMethod::GET,
            headers: HashMap::new(),
            body: None,
            timestamp: Utc::now(),
            initiator: None,
            state: RequestState::Complete,
            response: Some(NetworkResponse {
                status_code: 200,
                status_text: "OK".to_string(),
                headers: HashMap::new(),
                body: None,
                content_type: Some("application/json".to_string()),
                size: 1024,
                encoding: None,
                security_details: None,
            }),
            priority: RequestPriority::Medium,
            resource_type: ResourceType::XHR,
            timing: Some(ResourceTiming {
                start_time: 0.0,
                dns_lookup: 10.0,
                tcp_connect: 20.0,
                ssl_handshake: Some(30.0),
                request_start: 40.0,
                response_start: 50.0,
                response_end: 100.0,
                duration: 100.0,
            }),
            size: None,
            protocol: None,
        };
        
        monitor.start_request(request).await.unwrap();
        
        let har = monitor.export_har().await.unwrap();
        assert!(har.contains("example.com"));
        assert!(har.contains("GET"));
    }

    #[tokio::test]
    async fn test_filter_requests() {
        let monitor = NetworkMonitor::new();
        
        let request = NetworkRequest {
            id: Uuid::new_v4(),
            url: "https://api.example.com/data".to_string(),
            method: HttpMethod::POST,
            headers: HashMap::new(),
            body: None,
            timestamp: Utc::now(),
            initiator: None,
            state: RequestState::Pending,
            response: None,
            priority: RequestPriority::High,
            resource_type: ResourceType::Fetch,
            timing: None,
            size: None,
            protocol: None,
        };
        
        monitor.start_request(request).await.unwrap();
        
        monitor.add_filter(NetworkFilter {
            url_pattern: Some("api".to_string()),
            method: Some(HttpMethod::POST),
            resource_type: None,
            ..Default::default()
        }).await;
        
        let filtered = monitor.get_filtered_requests().await;
        assert_eq!(filtered.len(), 1);
    }
}

impl Default for NetworkFilter {
    fn default() -> Self {
        Self {
            url_pattern: None,
            method: None,
            status_code: None,
            resource_type: None,
            domain: None,
            min_size: None,
            max_size: None,
            min_duration: None,
            max_duration: None,
            has_errors: None,
        }
    }
}