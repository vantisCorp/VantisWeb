//! Requests API
//!
//! Provides APIs for extensions to make HTTP requests.

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Requests API
pub struct RequestsAPI {
    /// Extension ID
    extension_id: String,
}

impl RequestsAPI {
    /// Creates a new requests API
    pub fn new(extension_id: String) -> Self {
        Self { extension_id }
    }

    /// Makes an HTTP GET request
    pub fn get(&self, url: &str, options: Option<RequestOptions>) -> Result<Response> {
        // TODO: Implement GET request
        Ok(Response {
            status: 200,
            status_text: "OK".to_string(),
            headers: std::collections::HashMap::new(),
            body: None,
        })
    }

    /// Makes an HTTP POST request
    pub fn post(&self, url: &str, data: Option<RequestData>, options: Option<RequestOptions>) -> Result<Response> {
        // TODO: Implement POST request
        Ok(Response {
            status: 200,
            status_text: "OK".to_string(),
            headers: std::collections::HashMap::new(),
            body: None,
        })
    }

    /// Makes an HTTP PUT request
    pub fn put(&self, url: &str, data: Option<RequestData>, options: Option<RequestOptions>) -> Result<Response> {
        // TODO: Implement PUT request
        Ok(Response {
            status: 200,
            status_text: "OK".to_string(),
            headers: std::collections::HashMap::new(),
            body: None,
        })
    }

    /// Makes an HTTP DELETE request
    pub fn delete(&self, url: &str, options: Option<RequestOptions>) -> Result<Response> {
        // TODO: Implement DELETE request
        Ok(Response {
            status: 200,
            status_text: "OK".to_string(),
            headers: std::collections::HashMap::new(),
            body: None,
        })
    }

    /// Makes an HTTP PATCH request
    pub fn patch(&self, url: &str, data: Option<RequestData>, options: Option<RequestOptions>) -> Result<Response> {
        // TODO: Implement PATCH request
        Ok(Response {
            status: 200,
            status_text: "OK".to_string(),
            headers: std::collections::HashMap::new(),
            body: None,
        })
    }

    /// Makes a custom HTTP request
    pub fn request(&self, method: &str, url: &str, data: Option<RequestData>, options: Option<RequestOptions>) -> Result<Response> {
        // TODO: Implement custom request
        Ok(Response {
            status: 200,
            status_text: "OK".to_string(),
            headers: std::collections::HashMap::new(),
            body: None,
        })
    }
}

/// Request options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestOptions {
    /// Request headers
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
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

/// Request data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestData {
    /// JSON data
    Json(serde_json::Value),
    /// Form data
    Form(std::collections::HashMap<String, String>),
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
    pub headers: std::collections::HashMap<String, String>,
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
    fn test_get_request() {
        let api = RequestsAPI::new("test-extension".to_string());
        let response = api.get("https://example.com", None).unwrap();

        assert_eq!(response.status, 200);
        assert_eq!(response.status_text, "OK");
    }

    #[test]
    fn test_response_is_success() {
        let response = Response {
            status: 200,
            status_text: "OK".to_string(),
            headers: std::collections::HashMap::new(),
            body: None,
        };

        assert!(response.is_success());
    }

    #[test]
    fn test_response_is_not_success() {
        let response = Response {
            status: 404,
            status_text: "Not Found".to_string(),
            headers: std::collections::HashMap::new(),
            body: None,
        };

        assert!(!response.is_success());
    }

    #[test]
    fn test_response_text() {
        let response = Response {
            status: 200,
            status_text: "OK".to_string(),
            headers: std::collections::HashMap::new(),
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
            headers: std::collections::HashMap::new(),
            body: Some(body.to_string()),
        };

        let json = response.json().unwrap();
        assert_eq!(json["key"], "value");
    }
}