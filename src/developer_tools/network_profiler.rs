//! Network Profiler
//! 
//! Captures and analyzes network requests with detailed timing information.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use anyhow::Result;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

use super::{
    DevToolsEvent, NetworkRequestInfo,
    NetworkAnalysis,
};
use super::models::{
    NetworkRequest, HTTPHeader, RequestTiming, ResourceType,
    RequestPriority, RequestInitiator, InitiatorType,
};

/// Network profiler for request analysis
pub struct NetworkProfiler {
    /// Captured requests
    requests: RwLock<HashMap<String, NetworkRequest>>,
    /// Request order
    request_order: RwLock<Vec<String>>,
    /// Event sender
    event_sender: broadcast::Sender<DevToolsEvent>,
    /// Configuration
    config: RwLock<NetworkProfilerConfig>,
    /// Filter settings
    filters: RwLock<NetworkFilters>,
}

/// Network profiler configuration
#[derive(Debug, Clone)]
pub struct NetworkProfilerConfig {
    /// Capture request bodies
    pub capture_request_bodies: bool,
    /// Capture response bodies
    pub capture_response_bodies: bool,
    /// Maximum body size to capture (bytes)
    pub max_body_size: usize,
    /// Enable timing capture
    pub capture_timing: bool,
    /// Enable initiator capture
    pub capture_initiators: bool,
    /// Preserve log on navigation
    pub preserve_log: bool,
}

impl Default for NetworkProfilerConfig {
    fn default() -> Self {
        Self {
            capture_request_bodies: true,
            capture_response_bodies: true,
            max_body_size: 1024 * 1024, // 1MB
            capture_timing: true,
            capture_initiators: true,
            preserve_log: false,
        }
    }
}

/// Network filters
#[derive(Debug, Clone, Default)]
pub struct NetworkFilters {
    /// Filter by resource type
    pub resource_types: Vec<ResourceType>,
    /// Filter by URL pattern
    pub url_pattern: Option<String>,
    /// Filter by status code range
    pub status_range: Option<(u16, u16)>,
    /// Filter by method
    pub methods: Vec<String>,
    /// Show only XHR/fetch
    pub xhr_only: bool,
    /// Show only blocked requests
    pub blocked_only: bool,
}

impl NetworkProfiler {
    /// Create a new network profiler
    pub async fn new(event_sender: broadcast::Sender<DevToolsEvent>) -> Result<Self> {
        Ok(Self {
            requests: RwLock::new(HashMap::new()),
            request_order: RwLock::new(Vec::new()),
            event_sender,
            config: RwLock::new(NetworkProfilerConfig::default()),
            filters: RwLock::new(NetworkFilters::default()),
        })
    }
    
    /// Start capturing request
    pub async fn start_request(
        &self,
        url: &str,
        method: &str,
        resource_type: ResourceType,
        priority: RequestPriority,
        initiator: Option<RequestInitiator>,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        
        let request = NetworkRequest {
            id: id.clone(),
            url: url.to_string(),
            method: method.to_uppercase(),
            request_headers: Vec::new(),
            request_body: None,
            response_status: None,
            response_status_text: None,
            response_headers: Vec::new(),
            response_body: None,
            response_preview: None,
            timing: RequestTiming {
                dns_time_ms: 0,
                connect_time_ms: 0,
                ssl_time_ms: 0,
                send_time_ms: 0,
                wait_time_ms: 0,
                receive_time_ms: 0,
                total_time_ms: 0,
                start_time: Utc::now(),
                end_time: Utc::now(),
            },
            resource_type,
            priority,
            from_cache: false,
            initiator,
            error: None,
        };
        
        let mut requests = self.requests.write().await;
        requests.insert(id.clone(), request);
        
        let mut order = self.request_order.write().await;
        order.push(id.clone());
        
        // Emit event
        let _ = self.event_sender.send(DevToolsEvent::NetworkRequest(id.clone()));
        
        Ok(id)
    }
    
    /// Update request with response data
    pub async fn complete_request(
        &self,
        request_id: &str,
        status: u16,
        status_text: &str,
        response_headers: Vec<HTTPHeader>,
        response_body: Option<Vec<u8>>,
    ) -> Result<()> {
        let mut requests = self.requests.write().await;
        
        if let Some(request) = requests.get_mut(request_id) {
            request.response_status = Some(status);
            request.response_status_text = Some(status_text.to_string());
            request.response_headers = response_headers;
            
            // Capture response body if enabled
            let config = self.config.read().await;
            if config.capture_response_bodies {
                if let Some(body) = response_body {
                    if body.len() <= config.max_body_size {
                        request.response_body = Some(body);
                    }
                    
                    // Create preview
                    request.response_preview = Some(self.create_preview(&request.response_body));
                }
            }
            
            // Update timing
            request.timing.end_time = Utc::now();
            request.timing.total_time_ms = (request.timing.end_time - request.timing.start_time)
                .num_milliseconds() as u64;
        }
        
        Ok(())
    }
    
    /// Mark request as cached
    pub async fn mark_cached(&self, request_id: &str) -> Result<()> {
        let mut requests = self.requests.write().await;
        
        if let Some(request) = requests.get_mut(request_id) {
            request.from_cache = true;
        }
        
        Ok(())
    }
    
    /// Mark request as failed
    pub async fn mark_failed(&self, request_id: &str, error: &str) -> Result<()> {
        let mut requests = self.requests.write().await;
        
        if let Some(request) = requests.get_mut(request_id) {
            request.error = Some(error.to_string());
            request.timing.end_time = Utc::now();
        }
        
        Ok(())
    }
    
    /// Set request timing
    pub async fn set_timing(
        &self,
        request_id: &str,
        dns_time_ms: u64,
        connect_time_ms: u64,
        ssl_time_ms: u64,
        send_time_ms: u64,
        wait_time_ms: u64,
        receive_time_ms: u64,
    ) -> Result<()> {
        let mut requests = self.requests.write().await;
        
        if let Some(request) = requests.get_mut(request_id) {
            request.timing.dns_time_ms = dns_time_ms;
            request.timing.connect_time_ms = connect_time_ms;
            request.timing.ssl_time_ms = ssl_time_ms;
            request.timing.send_time_ms = send_time_ms;
            request.timing.wait_time_ms = wait_time_ms;
            request.timing.receive_time_ms = receive_time_ms;
        }
        
        Ok(())
    }
    
    /// Get all requests
    pub async fn get_requests(&self) -> Result<Vec<NetworkRequest>> {
        let requests = self.requests.read().await;
        let order = self.request_order.read().await;
        
        let result: Vec<NetworkRequest> = order.iter()
            .filter_map(|id| requests.get(id).cloned())
            .collect();
        
        Ok(result)
    }
    
    /// Get filtered requests
    pub async fn get_filtered_requests(&self) -> Result<Vec<NetworkRequest>> {
        let requests = self.requests.read().await;
        let order = self.request_order.read().await;
        let filters = self.filters.read().await;
        
        let result: Vec<NetworkRequest> = order.iter()
            .filter_map(|id| requests.get(id))
            .filter(|req| self.matches_filters(req, &filters))
            .cloned()
            .collect();
        
        Ok(result)
    }
    
    /// Check if request matches filters
    fn matches_filters(&self, request: &NetworkRequest, filters: &NetworkFilters) -> bool {
        // Resource type filter
        if !filters.resource_types.is_empty() && 
           !filters.resource_types.contains(&request.resource_type) {
            return false;
        }
        
        // URL pattern filter
        if let Some(pattern) = &filters.url_pattern {
            if !request.url.contains(pattern) {
                return false;
            }
        }
        
        // Status range filter
        if let Some((min, max)) = filters.status_range {
            if let Some(status) = request.response_status {
                if status < min || status > max {
                    return false;
                }
            }
        }
        
        // Method filter
        if !filters.methods.is_empty() && 
           !filters.methods.contains(&request.method) {
            return false;
        }
        
        // XHR only filter
        if filters.xhr_only && 
           request.resource_type != ResourceType::XHR && 
           request.resource_type != ResourceType::Fetch {
            return false;
        }
        
        true
    }
    
    /// Get request by ID
    pub async fn get_request(&self, request_id: &str) -> Result<Option<NetworkRequest>> {
        let requests = self.requests.read().await;
        Ok(requests.get(request_id).cloned())
    }
    
    /// Get request summary info
    pub async fn capture_requests(&self) -> Result<Vec<NetworkRequestInfo>> {
        let requests = self.requests.read().await;
        let order = self.request_order.read().await;
        
        let result: Vec<NetworkRequestInfo> = order.iter()
            .filter_map(|id| {
                requests.get(id).map(|r| NetworkRequestInfo {
                    id: r.id.clone(),
                    url: r.url.clone(),
                    method: r.method.clone(),
                    status: r.response_status.unwrap_or(0),
                    duration_ms: r.timing.total_time_ms,
                    request_size: r.request_body.as_ref().map(|b| b.len() as u64).unwrap_or(0),
                    response_size: r.response_body.as_ref().map(|b| b.len() as u64).unwrap_or(0),
                    timestamp: r.timing.start_time,
                })
            })
            .collect();
        
        Ok(result)
    }
    
    /// Clear all requests
    pub async fn clear(&self) -> Result<()> {
        let mut requests = self.requests.write().await;
        let mut order = self.request_order.write().await;
        
        requests.clear();
        order.clear();
        
        Ok(())
    }
    
    /// Set filters
    pub async fn set_filters(&self, filters: NetworkFilters) -> Result<()> {
        let mut current = self.filters.write().await;
        *current = filters;
        Ok(())
    }
    
    /// Get statistics
    pub async fn get_statistics(&self) -> Result<NetworkStats> {
        let requests = self.requests.read().await;
        
        let total = requests.len() as u64;
        let cached = requests.values().filter(|r| r.from_cache).count() as u64;
        let failed = requests.values().filter(|r| r.error.is_some()).count() as u64;
        
        let total_transferred: u64 = requests.values()
            .filter_map(|r| r.response_body.as_ref().map(|b| b.len() as u64))
            .sum();
        
        let avg_response_time = if total > 0 {
            requests.values()
                .map(|r| r.timing.total_time_ms)
                .sum::<u64>() as f64 / total as f64
        } else {
            0.0
        };
        
        let cache_hit_rate = if total > 0 {
            cached as f64 / total as f64
        } else {
            0.0
        };
        
        Ok(NetworkStats {
            total_requests: total,
            cached_requests: cached,
            failed_requests: failed,
            total_transferred,
            avg_response_time_ms: avg_response_time,
            cache_hit_rate,
        })
    }
    
    /// Analyze network performance
    pub async fn analyze(&self) -> Result<NetworkAnalysis> {
        let stats = self.get_statistics().await?;
        
        Ok(NetworkAnalysis {
            total_requests: stats.total_requests,
            failed_requests: stats.failed_requests,
            avg_response_time_ms: stats.avg_response_time_ms,
            total_transferred: stats.total_transferred,
            cache_hit_rate: stats.cache_hit_rate,
        })
    }
    
    /// Export as HAR format
    pub async fn export_har(&self) -> Result<serde_json::Value> {
        let requests = self.requests.read().await;
        let order = self.request_order.read().await;
        
        let entries: Vec<serde_json::Value> = order.iter()
            .filter_map(|id| {
                requests.get(id).map(|r| self.request_to_har_entry(r))
            })
            .collect();
        
        Ok(serde_json::json!({
            "log": {
                "version": "1.2",
                "creator": {
                    "name": "VantisWeb Developer Tools",
                    "version": "2.3.0"
                },
                "entries": entries
            }
        }))
    }
    
    /// Convert request to HAR entry
    fn request_to_har_entry(&self, request: &NetworkRequest) -> serde_json::Value {
        let headers: Vec<serde_json::Value> = request.request_headers.iter()
            .map(|h| serde_json::json!({
                "name": h.name,
                "value": h.value
            }))
            .collect();
        
        let response_headers: Vec<serde_json::Value> = request.response_headers.iter()
            .map(|h| serde_json::json!({
                "name": h.name,
                "value": h.value
            }))
            .collect();
        
        serde_json::json!({
            "startedDateTime": request.timing.start_time.to_rfc3339(),
            "time": request.timing.total_time_ms,
            "request": {
                "method": request.method,
                "url": request.url,
                "headers": headers
            },
            "response": {
                "status": request.response_status.unwrap_or(0),
                "statusText": request.response_status_text,
                "headers": response_headers,
                "content": {
                    "size": request.response_body.as_ref().map(|b| b.len()).unwrap_or(0),
                    "mimeType": self.get_mime_type(request)
                }
            },
            "timings": {
                "dns": request.timing.dns_time_ms,
                "connect": request.timing.connect_time_ms,
                "ssl": request.timing.ssl_time_ms,
                "send": request.timing.send_time_ms,
                "wait": request.timing.wait_time_ms,
                "receive": request.timing.receive_time_ms
            }
        })
    }
    
    /// Get MIME type for request
    fn get_mime_type(&self, request: &NetworkRequest) -> String {
        request.response_headers.iter()
            .find(|h| h.name.eq_ignore_ascii_case("content-type"))
            .map(|h| h.value.split(';').next().unwrap_or("application/octet-stream").to_string())
            .unwrap_or_else(|| match request.resource_type {
                ResourceType::Document => "text/html",
                ResourceType::Stylesheet => "text/css",
                ResourceType::Script => "application/javascript",
                ResourceType::Image => "image/png",
                ResourceType::Font => "font/woff2",
                _ => "application/octet-stream",
            }.to_string())
    }
    
    /// Create preview string
    fn create_preview(&self, body: &Option<Vec<u8>>) -> String {
        if let Some(data) = body {
            // Try to decode as UTF-8
            if let Ok(text) = String::from_utf8(data.clone()) {
                if text.len() > 1000 {
                    format!("{}... ({} bytes)", &text[..1000], data.len())
                } else {
                    text
                }
            } else {
                format!("[Binary data: {} bytes]", data.len())
            }
        } else {
            String::new()
        }
    }
    
    /// Find slow requests
    pub async fn find_slow_requests(&self, threshold_ms: u64) -> Result<Vec<NetworkRequest>> {
        let requests = self.requests.read().await;
        
        let slow: Vec<NetworkRequest> = requests.values()
            .filter(|r| r.timing.total_time_ms > threshold_ms)
            .cloned()
            .collect();
        
        Ok(slow)
    }
    
    /// Find large requests
    pub async fn find_large_requests(&self, threshold_bytes: u64) -> Result<Vec<NetworkRequest>> {
        let requests = self.requests.read().await;
        
        let large: Vec<NetworkRequest> = requests.values()
            .filter(|r| {
                r.response_body.as_ref().map(|b| b.len() as u64).unwrap_or(0) > threshold_bytes
            })
            .cloned()
            .collect();
        
        Ok(large)
    }
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub total_requests: u64,
    pub cached_requests: u64,
    pub failed_requests: u64,
    pub total_transferred: u64,
    pub avg_response_time_ms: f64,
    pub cache_hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_network_profiler_creation() {
        let (tx, _rx) = broadcast::channel(16);
        let profiler = NetworkProfiler::new(tx).await.unwrap();
        
        let requests = profiler.get_requests().await.unwrap();
        assert!(requests.is_empty());
    }
    
    #[tokio::test]
    async fn test_start_and_complete_request() {
        let (tx, _rx) = broadcast::channel(16);
        let profiler = NetworkProfiler::new(tx).await.unwrap();
        
        let id = profiler.start_request(
            "https://example.com/api/test",
            "GET",
            ResourceType::XHR,
            RequestPriority::High,
            None,
        ).await.unwrap();
        
        profiler.complete_request(
            &id,
            200,
            "OK",
            vec![HTTPHeader { name: "Content-Type".to_string(), value: "application/json".to_string() }],
            Some(br##"{"status":"ok"}"##.to_vec()),
        ).await.unwrap();
        
        let requests = profiler.get_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].response_status, Some(200));
    }
}