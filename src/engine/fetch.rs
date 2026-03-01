//! Fetch API
//! 
//! HTTP/HTTPS client implementation:
//! - Request/Response handling
//! - Headers management
//! - Streaming support
//! - CORS handling

use anyhow::{Context, Result};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::core::kernel::VantisKernel;

/// HTTP methods
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

impl HttpMethod {
    /// Parse from string
    pub fn from_str(method: &str) -> Result<Self> {
        match method.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::GET),
            "POST" => Ok(HttpMethod::POST),
            "PUT" => Ok(HttpMethod::PUT),
            "DELETE" => Ok(HttpMethod::DELETE),
            "PATCH" => Ok(HttpMethod::PATCH),
            "HEAD" => Ok(HttpMethod::HEAD),
            "OPTIONS" => Ok(HttpMethod::OPTIONS),
            "CONNECT" => Ok(HttpMethod::CONNECT),
            "TRACE" => Ok(HttpMethod::TRACE),
            _ => Err(anyhow::anyhow!("Invalid HTTP method: {}", method)),
        }
    }

    /// Convert to string
    pub fn as_str(&self) -> &str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::OPTIONS => "OPTIONS",
            HttpMethod::CONNECT => "CONNECT",
            HttpMethod::TRACE => "TRACE",
        }
    }
}

/// HTTP headers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpHeaders {
    headers: HashMap<String, String>,
}

impl HttpHeaders {
    /// Create new headers
    pub fn new() -> Self {
        Self {
            headers: HashMap::new(),
        }
    }

    /// Set a header
    pub fn set(&mut self, name: String, value: String) {
        self.headers.insert(name.to_lowercase(), value);
    }

    /// Get a header
    pub fn get(&self, name: &str) -> Option<&String> {
        self.headers.get(&name.to_lowercase())
    }

    /// Remove a header
    pub fn remove(&mut self, name: &str) -> Option<String> {
        self.headers.remove(&name.to_lowercase())
    }

    /// Check if header exists
    pub fn has(&self, name: &str) -> bool {
        self.headers.contains_key(&name.to_lowercase())
    }

    /// Get all headers
    pub fn all(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// Get headers as iterator
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.headers.iter()
    }
}

impl Default for HttpHeaders {
    fn default() -> Self {
        Self::new()
    }
}

/// Request body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestBody {
    /// No body
    None,
    /// Text body
    Text(String),
    /// JSON body
    Json(serde_json::Value),
    /// Binary body
    Binary(Vec<u8>),
    /// Form data
    Form(HashMap<String, String>),
}

impl RequestBody {
    /// Get content type
    pub fn content_type(&self) -> Option<String> {
        match self {
            RequestBody::None => None,
            RequestBody::Text(_) => Some("text/plain".to_string()),
            RequestBody::Json(_) => Some("application/json".to_string()),
            RequestBody::Binary(_) => Some("application/octet-stream".to_string()),
            RequestBody::Form(_) => Some("application/x-www-form-urlencoded".to_string()),
        }
    }

    /// Get body as bytes
    pub fn as_bytes(&self) -> Option<Vec<u8>> {
        match self {
            RequestBody::None => None,
            RequestBody::Text(text) => Some(text.as_bytes().to_vec()),
            RequestBody::Json(value) => Some(value.to_string().as_bytes().to_vec()),
            RequestBody::Binary(data) => Some(data.clone()),
            RequestBody::Form(form) => {
                let body = form
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("&");
                Some(body.as_bytes().to_vec())
            }
        }
    }
}

/// HTTP Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    /// Request URL
    pub url: String,
    /// HTTP method
    pub method: HttpMethod,
    /// Request headers
    pub headers: HttpHeaders,
    /// Request body
    pub body: RequestBody,
    /// Request timeout (seconds)
    pub timeout: u64,
    /// Follow redirects
    pub follow_redirects: bool,
    /// User agent
    pub user_agent: String,
}

impl HttpRequest {
    /// Create a new request
    pub fn new(url: String, method: HttpMethod) -> Self {
        let mut headers = HttpHeaders::new();
        headers.set("User-Agent".to_string(), "VantisWeb/0.2.0".to_string());
        headers.set("Accept".to_string(), "*/*".to_string());

        Self {
            url,
            method,
            headers,
            body: RequestBody::None,
            timeout: 30,
            follow_redirects: true,
            user_agent: "VantisWeb/0.2.0".to_string(),
        }
    }

    /// Set request body
    pub fn with_body(mut self, body: RequestBody) -> Self {
        if let Some(content_type) = body.content_type() {
            self.headers.set("Content-Type".to_string(), content_type);
        }
        self.body = body;
        self
    }

    /// Set header
    pub fn with_header(mut self, name: String, value: String) -> Self {
        self.headers.set(name, value);
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set follow redirects
    pub fn with_redirects(mut self, follow: bool) -> Self {
        self.follow_redirects = follow;
        self
    }
}

/// HTTP Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    /// Response status code
    pub status: u16,
    /// Response status text
    pub status_text: String,
    /// Response headers
    pub headers: HttpHeaders,
    /// Response body
    pub body: Vec<u8>,
    /// Response URL (after redirects)
    pub url: String,
    /// Response time (milliseconds)
    pub response_time_ms: u64,
}

impl HttpResponse {
    /// Create a new response
    pub fn new(status: u16, status_text: String, url: String) -> Self {
        Self {
            status,
            status_text,
            headers: HttpHeaders::new(),
            body: Vec::new(),
            url,
            response_time_ms: 0,
        }
    }

    /// Get body as text
    pub fn text(&self) -> Result<String> {
        String::from_utf8(self.body.clone())
            .context("Failed to decode response body as UTF-8")
    }

    /// Get body as JSON
    pub fn json(&self) -> Result<serde_json::Value> {
        serde_json::from_slice(&self.body)
            .context("Failed to parse response body as JSON")
    }

    /// Check if response is successful
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// Check if response is redirect
    pub fn is_redirect(&self) -> bool {
        (300..400).contains(&self.status)
    }

    /// Check if response is client error
    pub fn is_client_error(&self) -> bool {
        (400..500).contains(&self.status)
    }

    /// Check if response is server error
    pub fn is_server_error(&self) -> bool {
        (500..600).contains(&self.status)
    }
}

/// CORS mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CorsMode {
    /// Same-origin
    SameOrigin,
    /// CORS with credentials
    CorsWithCredentials,
    /// CORS without credentials
    CorsNoCredentials,
    /// No CORS
    NoCors,
}

/// Fetch API
pub struct FetchApi {
    kernel: Arc<VantisKernel>,
    /// Default timeout
    default_timeout: u64,
    /// Default user agent
    default_user_agent: String,
    /// CORS mode
    cors_mode: Arc<RwLock<CorsMode>>,
}

impl FetchApi {
    /// Create a new Fetch API
    pub fn new(kernel: Arc<VantisKernel>) -> Self {
        info!("Initializing Fetch API...");

        Self {
            kernel,
            default_timeout: 30,
            default_user_agent: "VantisWeb/0.2.0".to_string(),
            cors_mode: Arc::new(RwLock::new(CorsMode::CorsNoCredentials)),
        }
    }

    /// Execute a fetch request
    pub async fn fetch(&self, request: HttpRequest) -> Result<HttpResponse> {
        info!("Fetching: {} {}", request.method.as_str(), request.url);

        let start_time = std::time::Instant::now();

        // In production: Use actual HTTP client (reqwest)
        // For MVP: Simulate fetch

        // Validate URL
        self.validate_url(&request.url)?;

        // Check CORS
        self.check_cors(&request.url).await?;

        // Simulate network delay
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Create mock response
        let mut response = HttpResponse::new(200, "OK".to_string(), request.url.clone());

        // Set response headers
        response.headers.set("Content-Type".to_string(), "text/html".to_string());
        response.headers.set("Content-Length".to_string(), "0".to_string());

        // Set response time
        response.response_time_ms = start_time.elapsed().as_millis() as u64;

        info!("Fetch completed: {} ({}ms)", request.url, response.response_time_ms);

        Ok(response)
    }

    /// Execute a simple GET request
    pub async fn get(&self, url: String) -> Result<HttpResponse> {
        let request = HttpRequest::new(url, HttpMethod::GET);
        self.fetch(request).await
    }

    /// Execute a POST request
    pub async fn post(&self, url: String, body: RequestBody) -> Result<HttpResponse> {
        let request = HttpRequest::new(url, HttpMethod::POST).with_body(body);
        self.fetch(request).await
    }

    /// Execute a PUT request
    pub async fn put(&self, url: String, body: RequestBody) -> Result<HttpResponse> {
        let request = HttpRequest::new(url, HttpMethod::PUT).with_body(body);
        self.fetch(request).await
    }

    /// Execute a DELETE request
    pub async fn delete(&self, url: String) -> Result<HttpResponse> {
        let request = HttpRequest::new(url, HttpMethod::DELETE);
        self.fetch(request).await
    }

    /// Validate URL
    fn validate_url(&self, url: &str) -> Result<()> {
        if url.is_empty() {
            return Err(anyhow::anyhow!("URL cannot be empty"));
        }

        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(anyhow::anyhow!("URL must start with http:// or https://"));
        }

        Ok(())
    }

    /// Check CORS
    async fn check_cors(&self, url: &str) -> Result<()> {
        let cors_mode = *self.cors_mode.read().await;

        match cors_mode {
            CorsMode::NoCors => {
                // No CORS check needed
                Ok(())
            }
            CorsMode::SameOrigin => {
                // Check if same origin
                // For MVP: Allow all
                Ok(())
            }
            CorsMode::CorsWithCredentials | CorsMode::CorsNoCredentials => {
                // Perform CORS check
                // For MVP: Allow all
                Ok(())
            }
        }
    }

    /// Set CORS mode
    pub async fn set_cors_mode(&self, mode: CorsMode) {
        *self.cors_mode.write().await = mode;
    }

    /// Get CORS mode
    pub async fn get_cors_mode(&self) -> CorsMode {
        *self.cors_mode.read().await
    }

    /// Set default timeout
    pub fn set_default_timeout(&mut self, timeout: u64) {
        self.default_timeout = timeout;
    }

    /// Get default timeout
    pub fn get_default_timeout(&self) -> u64 {
        self.default_timeout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_fetch_api() -> FetchApi {
        let kernel = Arc::new(VantisKernel::new().await.unwrap());
        FetchApi::new(kernel)
    }

    #[tokio::test]
    async fn test_fetch_api_creation() {
        let fetch_api = create_fetch_api().await;
        assert_eq!(fetch_api.get_default_timeout(), 30);
    }

    #[tokio::test]
    async fn test_http_request_creation() {
        let request = HttpRequest::new("https://example.com".to_string(), HttpMethod::GET);
        assert_eq!(request.url, "https://example.com");
        assert_eq!(request.method, HttpMethod::GET);
    }

    #[tokio::test]
    async fn test_http_request_with_body() {
        let body = RequestBody::Text("Hello, World!".to_string());
        let request = HttpRequest::new("https://example.com".to_string(), HttpMethod::POST)
            .with_body(body);

        assert!(matches!(request.body, RequestBody::Text(_)));
        assert!(request.headers.has("Content-Type"));
    }

    #[tokio::test]
    async fn test_http_headers() {
        let mut headers = HttpHeaders::new();
        headers.set("Content-Type".to_string(), "application/json".to_string());
        headers.set("Authorization".to_string(), "Bearer token".to_string());

        assert_eq!(
            headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
        assert_eq!(
            headers.get("Authorization"),
            Some(&"Bearer token".to_string())
        );
        assert!(headers.has("content-type")); // Case insensitive
    }

    #[tokio::test]
    async fn test_http_method_from_str() {
        assert_eq!(HttpMethod::from_str("GET").unwrap(), HttpMethod::GET);
        assert_eq!(HttpMethod::from_str("post").unwrap(), HttpMethod::POST);
        assert!(HttpMethod::from_str("INVALID").is_err());
    }

    #[tokio::test]
    async fn test_http_response() {
        let response = HttpResponse::new(200, "OK".to_string(), "https://example.com".to_string());
        assert_eq!(response.status, 200);
        assert!(response.is_success());
        assert!(!response.is_redirect());
        assert!(!response.is_client_error());
        assert!(!response.is_server_error());
    }

    #[tokio::test]
    async fn test_fetch_get() {
        let fetch_api = create_fetch_api().await;

        let response = fetch_api
            .get("https://example.com".to_string())
            .await
            .unwrap();

        assert_eq!(response.status, 200);
        assert!(response.is_success());
    }

    #[tokio::test]
    async fn test_fetch_post() {
        let fetch_api = create_fetch_api().await;

        let body = RequestBody::Json(serde_json::json!({"key": "value"}));
        let response = fetch_api
            .post("https://example.com".to_string(), body)
            .await
            .unwrap();

        assert_eq!(response.status, 200);
    }

    #[tokio::test]
    async fn test_cors_mode() {
        let fetch_api = create_fetch_api().await;

        fetch_api.set_cors_mode(CorsMode::CorsWithCredentials).await;
        assert_eq!(
            fetch_api.get_cors_mode().await,
            CorsMode::CorsWithCredentials
        );
    }

    #[tokio::test]
    async fn test_invalid_url() {
        let fetch_api = create_fetch_api().await;

        let result = fetch_api.get("invalid-url".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_request_body_content_type() {
        let text_body = RequestBody::Text("test".to_string());
        assert_eq!(text_body.content_type(), Some("text/plain".to_string()));

        let json_body = RequestBody::Json(serde_json::json!({}));
        assert_eq!(json_body.content_type(), Some("application/json".to_string()));

        let none_body = RequestBody::None;
        assert_eq!(none_body.content_type(), None);
    }
}