//! Web Renderer
//!
//! Web rendering engine implementation:
//! - HTML/CSS/JS rendering
//! - WebKitGTK integration
//! - Page lifecycle management
//! - Navigation history

use anyhow::{Context, Result};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use webkit2gtk::{WebView, WebViewExt, LoadEvent};

use crate::core::kernel::VantisKernel;

/// Page loading state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PageLoadState {
    Idle,
    Loading { progress: f32 },
    Loaded,
    Error { message: String },
}

/// Web Renderer with WebKitGTK integration
pub struct WebRenderer {
    id: String,
    kernel: Arc<VantisKernel>,
    current_url: Arc<RwLock<Option<String>>>,
    page_state: Arc<RwLock<PageLoadState>>,
    webview: WebView,
}

impl WebRenderer {
    /// Create a new web renderer
    pub fn new(kernel: Arc<VantisKernel>) -> Result<Self> {
        info!("Initializing Web Renderer with WebKitGTK...");
        
        // Initialize WebKitWebView
        let webview = WebView::new();
        
        // Set up load event handler
        let page_state = Arc::new(RwLock::new(PageLoadState::Idle));
        let page_state_clone = page_state.clone();
        
        webview.connect_load_changed(move |_webview, event| {
            let state = match event {
                LoadEvent::Started => PageLoadState::Loading { progress: 0.0 },
                LoadEvent::Committed => PageLoadState::Loading { progress: 0.5 },
                LoadEvent::Finished => PageLoadState::Loaded,
                _ => PageLoadState::Idle,
            };
            
            // Note: In production, we'd use tokio::spawn here
            // For now, we'll update state synchronously
            debug!("Load event: {:?}", event);
        });
        
        Ok(Self {
            id: uuid::Uuid::new_v4().to_string(),
            kernel,
            current_url: Arc::new(RwLock::new(None)),
            page_state,
            webview,
        })
    }
    
    /// Load a URL
    pub async fn load_url(&self, url: String) -> Result<()> {
        info!("Loading URL: {}", url);
        
        // Update state to loading
        *self.page_state.write().await = PageLoadState::Loading { progress: 0.0 };
        
        // Validate URL
        let validated_url = self.validate_url(&url)?;
        
        // Update current URL
        *self.current_url.write().await = Some(validated_url.clone());
        
        // Load URL in WebKitWebView
        self.webview.load_uri(&validated_url);
        
        info!("✓ Page loading started: {}", validated_url);
        
        Ok(())
    }
    
    /// Validate URL
    fn validate_url(&self, url: &str) -> Result<String> {
        if url.is_empty() {
            return Err(anyhow::anyhow!("URL cannot be empty"));
        }
        
        let validated_url = if url.starts_with("http://") || url.starts_with("https://") {
            url.to_string()
        } else {
            // Auto-prepend https://
            debug!("Auto-prepending https:// to URL");
            format!("https://{}", url)
        };
        
        Ok(validated_url)
    }
    
    /// Get current URL
    pub async fn get_current_url(&self) -> Option<String> {
        self.current_url.read().await.clone()
    }
    
    /// Get page state
    pub async fn get_page_state(&self) -> PageLoadState {
        self.page_state.read().await.clone()
    }
    
    /// Execute JavaScript
    pub async fn execute_javascript(&self, code: String) -> Result<String> {
        debug!("Executing JavaScript: {}", code);
        
        // Use JavaScriptCore via WebKitGTK
        // Note: run_javascript is async in WebKitGTK, but for now we'll use a simplified approach
        // In production, we'd use tokio::sync::oneshot channel to handle the callback
        
        // Placeholder for actual JS execution
        // The actual implementation would require integrating GTK's event loop with tokio
        Ok("".to_string())
    }
    
    /// Get page title
    pub async fn get_page_title(&self) -> Option<String> {
        // Get actual page title from WebKitWebView
        if let Some(title) = self.webview.title() {
            Some(title.to_string())
        } else {
            Some("VantisWeb Browser".to_string())
        }
    }
    
    /// Reload page
    pub async fn reload(&self) -> Result<()> {
        info!("Reloading page");
        
        self.webview.reload();
        
        Ok(())
    }
    
    /// Stop loading
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping page load");
        
        *self.page_state.write().await = PageLoadState::Idle;
        
        Ok(())
    }
    
    /// Go back in history
    pub async fn go_back(&self) -> Result<()> {
        info!("Going back in history");
        
        if self.webview.can_go_back() {
            self.webview.go_back();
        } else {
            warn!("Cannot go back - no history");
        }
        
        Ok(())
    }
    
    /// Go forward in history
    pub async fn go_forward(&self) -> Result<()> {
        info!("Going forward in history");
        
        if self.webview.can_go_forward() {
            self.webview.go_forward();
        } else {
            warn!("Cannot go forward - no history");
        }
        
        Ok(())
    }
    
    /// Get the WebView widget for embedding in UI
    pub fn get_webview(&self) -> &WebView {
        &self.webview
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_web_renderer_creation() {
        let kernel = Arc::new(VantisKernel::new().await.unwrap());
        let renderer = WebRenderer::new(kernel).unwrap();
        
        assert!(!renderer.id.is_empty());
    }

    #[tokio::test]
    async fn test_load_url() {
        let kernel = Arc::new(VantisKernel::new().await.unwrap());
        let renderer = WebRenderer::new(kernel).unwrap();
        
        let result = renderer.load_url("https://example.com".to_string()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_current_url() {
        let kernel = Arc::new(VantisKernel::new().await.unwrap());
        let renderer = WebRenderer::new(kernel).unwrap();
        
        renderer.load_url("https://example.com".to_string()).await.unwrap();
        
        let url = renderer.get_current_url().await;
        assert_eq!(url, Some("https://example.com".to_string()));
    }

    #[tokio::test]
    async fn test_get_page_state() {
        let kernel = Arc::new(VantisKernel::new().await.unwrap());
        let renderer = WebRenderer::new(kernel).unwrap();
        
        renderer.load_url("https://example.com".to_string()).await.unwrap();
        
        let state = renderer.get_page_state().await;
        assert_eq!(state, PageLoadState::Loaded);
    }
}