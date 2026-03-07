//! Requests API
//!
//! Provides APIs for extensions to make HTTP requests.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Requests API
pub struct RequestsAPI {
    /// Extension ID
    extension_id: String,
    /// HTTP client
    client: reqwest::Client,
}

impl RequestsAPI {
    /// Creates a new requests API
    pub fn new(extension_id: String) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(format!("VantisWeb-Extension/{}", extension_id))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        
        Self { extension_id, client }
    }

    /// Makes an HTTP GET request
    pub async fn get(&self, url: &str, options: Option<RequestOptions>) -> Result<Response> {
        self.request("GET", url, None, options).await
    }

    /// Makes an HTTP POST request
    pub async fn post(&self, url: &str, data: Option<RequestData>, options: Option<RequestOptions>) -> Result<Response> {
        self.request("POST", url, data, options).await
    }

    /// Makes an HTTP PUT request
    pub async fn put(&self, url: &str, data: Option<RequestData>, options: Option<RequestOptions>) -> Result<Response> {
        self.request("PUT", url, data, options).await
    }

    /// Makes an HTTP DELETE request
    pub async fn delete(&self, url: &str, options: Option<RequestOptions>) -> Result<Response> {
        self.request("DELETE", url, None, options).await
    }

    /// Makes an HTTP PATCH request
    pub async fn patch(&self, url: &str, data: Option<RequestData>, options: Option<RequestOptions>) -> Result<Response> {
        self.request("PATCH", url, data, options).await
    }

    /// Makes a custom HTTP request
    pub async fn request(&self, method: &str, url: &str, data: Option<RequestData>, options: Option<RequestOptions>) -> Result<Response> {
        let mut request = match method.to_uppercase().as_str() {
            "GET" => self.client.get(url),
            "POST" => self.client.post(url),
            "PUT" => self.client.put(url),
            "DELETE" => self.client.delete(url),
            "PATCH" => self.client.patch(url),
            "HEAD" => self.client.head(url),
            _ => {
                let method = reqwest::Method::try_from(method)
                    .map_err(|e| anyhow::anyhow!("Invalid HTTP method: {}", e))?;
                self.client.request(method, url)
            }
        };

        // Apply options
        if let Some(opts) = options {
            // Add headers
            for (key, value) in &opts.headers {
                request = request.header(key, value);
            }
            
            // Set timeout
            if let Some(timeout) = opts.timeout {
                request = request.timeout(std::time::Duration::from_millis(timeout));
            }
            
            // Set user agent
            if let Some(user_agent) = &opts.user_agent {
                request = request.header("User-Agent", user_agent);
            }
        }

        // Add body data
        if let Some(data) = data {
            request = match data {
                RequestData::Json(json) => request.json(&json),
                RequestData::Form(form) => request.form(&form),
                RequestData::Text(text) => request.body(text),
                RequestData::Bytes(bytes) => request.body(bytes),
            };
        }

        // Execute request
        let response = request.send().await?;
        
        // Build Response
        let status = response.status().as_u16();
        let status_text = response.status().canonical_reason().unwrap_or("Unknown").to_string();
        
        let headers: HashMap<String, String> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        
        let body = response.text().await.ok();

        Ok(Response {
            status,
            status_text,
            headers,
            body,
        })
    }
    
    /// Create a synchronous blocking client
    pub fn new_blocking(extension_id: String) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(format!("VantisWeb-Extension/{}", extension_id))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        
        Self { extension_id, client }
    }
}

/// Request options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestOptions {
    /// Request headers
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Request timeout in milliseconds
    pub timeout: Option<u64>,
    /// Whether to follow redirects
    #[serde(default = "default_follow_redirects")]
    pub follow_redirects: bool,
    /// User agent
    pub user_agent: Option<String>,
}

fn default_follow_redirects() -> bool {
    true
}

impl Default for RequestOptions {
    fn default() -> Self {
        Self {
            headers: HashMap::new(),
            timeout: Some(30_000),
            follow_redirects: true,
            user_agent: None,
        }
    }
}

/// Request data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestData {
    /// JSON data
    Json(serde_json::Value),
    /// Form data
    Form(HashMap<String, String>),
    /// Raw text data
    Text(String),
    /// Raw bytes data
    Bytes(Vec<u8>),
}

/// HTTP response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// HTTP status code
    pub status: u16,
    /// HTTP status text
    pub status_text: String,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body
    pub body: Option<String>,
}

impl Response {
    /// Checks if the response was successful (2xx status code)
    pub fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    /// Gets the response body as JSON
    pub fn json(&self) -> Result<serde_json::Value> {
        match &self.body {
            Some(body) => {
                serde_json::from_str(body)
                    .map_err(|e| anyhow::anyhow!("Failed to parse JSON: {}", e))
            }
            None => Err(anyhow::anyhow!("Response body is empty")),
        }
    }

    /// Gets the response body as text
    pub fn text(&self) -> Result<String> {
        self.body.clone()
            .ok_or_else(|| anyhow::anyhow!("Response body is empty"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_requests_api_creation() {
        let api = RequestsAPI::new("test-extension".to_string());
        assert_eq!(api.extension_id, "test-extension");
    }

    #[test]
    fn test_response_is_success() {
        let response = Response {
            status: 200,
            status_text: "OK".to_string(),
            headers: HashMap::new(),
            body: None,
        };

        assert!(response.is_success());
    }

    #[test]
    fn test_response_is_not_success() {
        let response = Response {
            status: 404,
            status_text: "Not Found".to_string(),
            headers: HashMap::new(),
            body: None,
        };

        assert!(!response.is_success());
    }

    #[test]
    fn test_response_text() {
        let response = Response {
            status: 200,
            status_text: "OK".to_string(),
            headers: HashMap::new(),
            body: Some("Hello, World!".to_string()),
        };

        let text = response.text().unwrap();
        assert_eq!(text, "Hello, World!");
    }

    #[test]
    fn test_response_json() {
        let body = r#"{"key": "value"}"#;
        let response = Response {
            status: 200,
            status_text: "OK".to_string(),
            headers: HashMap::new(),
            body: Some(body.to_string()),
        };

        let json = response.json().unwrap();
        assert_eq!(json["key"], "value");
    }
    
    #[test]
    fn test_request_options_default() {
        let opts = RequestOptions::default();
        assert!(opts.follow_redirects);
        assert_eq!(opts.timeout, Some(30_000));
    }
}