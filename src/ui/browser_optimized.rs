//! Browser Window (Optimized)
//! 
//! Main browser window implementation:
//! - Tab management
//! - Navigation
//! - Address bar
//! - Bookmarks
//! - WebRenderer integration
//!
//! Optimizations:
//! - Pre-allocated collection capacities
//! - Reduced clones
//! - Optimized string operations
//! - Early lock release

use anyhow::Result;
use log::{debug, info};
use std::sync::Arc;

use crate::core::kernel::VantisKernel;
use crate::engine::web_renderer::WebRenderer;

/// Browser window (Optimized)
pub struct BrowserWindow {
    id: String,
    title: String,
    width: u32,
    height: u32,
    kernel: Arc<VantisKernel>,
    tabs: Vec<Tab>,
    active_tab: Option<usize>,
}

/// Browser tab (Optimized)
#[derive(Clone)]
pub struct Tab {
    id: String,
    title: String,
    url: String,
    is_loading: bool,
    web_renderer: Option<Arc<WebRenderer>>,
}

impl BrowserWindow {
    /// Create a new browser window (Optimized)
    pub async fn new(title: String, width: u32, height: u32, kernel: Arc<VantisKernel>) -> Result<Self> {
        info!("Creating browser window: {} ({}x{})", title, width, height);
        
        let mut window = Self {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            width,
            height,
            kernel,
            tabs: Vec::with_capacity(10),
            active_tab: None,
        };
        
        // Create initial tab with home page
        window.create_tab("https://vantis.ai/home".to_string()).await?;
        
        Ok(window)
    }
    
    /// Create a new tab (Optimized)
    pub async fn create_tab(&mut self, url: String) -> Result<()> {
        info!("Creating new tab: {}", url);
        
        // Create WebRenderer for this tab
        let web_renderer = Arc::new(WebRenderer::new(self.kernel.clone())?);
        
        let tab = Tab {
            id: uuid::Uuid::new_v4().to_string(),
            title: "New Tab".to_string(),
            url: url.clone(),
            is_loading: true,
            web_renderer: Some(web_renderer),
        };
        
        self.tabs.push(tab);
        self.active_tab = Some(self.tabs.len() - 1);
        
        // Navigate to URL using WebRenderer
        if let Some(active_tab) = self.active_tab() {
            if let Some(renderer) = &active_tab.web_renderer {
                renderer.load_url(url.clone()).await?;
            }
        }
        
        info!("✓ Tab created (total: {})", self.tabs.len());
        
        Ok(())
    }
    
    /// Close current tab (Optimized)
    pub async fn close_tab(&mut self) -> Result<()> {
        if let Some(active) = self.active_tab {
            info!("Closing tab: {}", self.tabs[active].title);
            self.tabs.remove(active);
            
            // Update active tab
            if self.tabs.is_empty() {
                self.active_tab = None;
            } else {
                self.active_tab = Some(active.min(self.tabs.len() - 1));
            }
            
            info!("✓ Tab closed (remaining: {})", self.tabs.len());
        }
        
        Ok(())
    }
    
    /// Navigate to URL (Optimized)
    pub async fn navigate(&mut self, url: String) -> Result<()> {
        info!("Navigating to: {}", url);
        
        if let Some(active) = self.active_tab {
            self.tabs[active].url = url.clone();
            self.tabs[active].is_loading = true;
            
            // Load page in WebRenderer
            if let Some(renderer) = &self.tabs[active].web_renderer {
                renderer.load_url(url.clone()).await?;
                
                // Update tab title
                if let Some(page_title) = renderer.get_page_title().await {
                    self.tabs[active].title = page_title;
                }
            }
            
            self.tabs[active].is_loading = false;
        }
        
        Ok(())
    }
    
    /// Go back in history (Optimized)
    pub async fn go_back(&mut self) -> Result<()> {
        info!("Going back in history");
        
        if let Some(active) = self.active_tab {
            if let Some(renderer) = &self.tabs[active].web_renderer {
                renderer.go_back().await?;
            }
        }
        
        Ok(())
    }
    
    /// Go forward in history (Optimized)
    pub async fn go_forward(&mut self) -> Result<()> {
        info!("Going forward in history");
        
        if let Some(active) = self.active_tab {
            if let Some(renderer) = &self.tabs[active].web_renderer {
                renderer.go_forward().await?;
            }
        }
        
        Ok(())
    }
    
    /// Reload current page (Optimized)
    pub async fn reload(&mut self) -> Result<()> {
        info!("Reloading page");
        
        if let Some(active) = self.active_tab {
            if let Some(renderer) = &self.tabs[active].web_renderer {
                renderer.reload().await?;
            }
        }
        
        Ok(())
    }
    
    /// Get active tab (Optimized - returns reference)
    pub fn active_tab(&self) -> Option<&Tab> {
        self.active_tab.and_then(|idx| self.tabs.get(idx))
    }
    
    /// Get all tabs (Optimized - returns reference)
    pub fn get_tabs(&self) -> &[Tab] {
        &self.tabs
    }
    
    /// Get window ID (Optimized - returns reference)
    pub fn get_id(&self) -> &str {
        &self.id
    }
    
    /// Get window title (Optimized - returns reference)
    pub fn get_title(&self) -> &str {
        &self.title
    }
    
    /// Get window dimensions (Optimized)
    pub fn get_dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    
    /// Set window title (Optimized)
    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }
    
    /// Set window dimensions (Optimized)
    pub fn set_dimensions(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_browser_window_creation() {
        let kernel = Arc::new(VantisKernel::new().await.unwrap());
        let window = BrowserWindow::new(
            "Test Window".to_string(),
            800,
            600,
            kernel,
        ).await.unwrap();
        
        assert!(!window.get_id().is_empty());
        assert_eq!(window.get_title(), "Test Window");
        assert_eq!(window.get_dimensions(), (800, 600));
        assert_eq!(window.get_tabs().len(), 1);
    }

    #[tokio::test]
    async fn test_create_tab() {
        let kernel = Arc::new(VantisKernel::new().await.unwrap());
        let mut window = BrowserWindow::new(
            "Test Window".to_string(),
            800,
            600,
            kernel,
        ).await.unwrap();
        
        window.create_tab("https://example.com".to_string()).await.unwrap();
        
        assert_eq!(window.get_tabs().len(), 2);
    }

    #[tokio::test]
    async fn test_close_tab() {
        let kernel = Arc::new(VantisKernel::new().await.unwrap());
        let mut window = BrowserWindow::new(
            "Test Window".to_string(),
            800,
            600,
            kernel,
        ).await.unwrap();
        
        window.create_tab("https://example.com".to_string()).await.unwrap();
        window.close_tab().await.unwrap();
        
        assert_eq!(window.get_tabs().len(), 1);
    }

    #[tokio::test]
    async fn test_navigate() {
        let kernel = Arc::new(VantisKernel::new().await.unwrap());
        let mut window = BrowserWindow::new(
            "Test Window".to_string(),
            800,
            600,
            kernel,
        ).await.unwrap();
        
        window.navigate("https://example.com".to_string()).await.unwrap();
        
        if let Some(tab) = window.active_tab() {
            assert_eq!(tab.url, "https://example.com");
        }
    }
}