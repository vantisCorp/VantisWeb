//! Browser Window
//! 
//! Main browser window implementation:
//! - Tab management
//! - Navigation
//! - Address bar
//! - Bookmarks
//! - WebRenderer integration

use anyhow::Result;
use log::{debug, info};
use std::sync::Arc;

use crate::core::kernel::VantisKernel;
use crate::engine::web_renderer::WebRenderer;

/// Browser window
pub struct BrowserWindow {
    id: String,
    title: String,
    width: u32,
    height: u32,
    kernel: Arc<VantisKernel>,
    tabs: Vec<Tab>,
    active_tab: Option<usize>,
}

/// Browser tab
#[derive(Clone)]
pub struct Tab {
    id: String,
    title: String,
    url: String,
    is_loading: bool,
    web_renderer: Option<Arc<WebRenderer>>,
}

impl BrowserWindow {
    /// Create a new browser window
    pub async fn new(title: String, width: u32, height: u32, kernel: Arc<VantisKernel>) -> Result<Self> {
        info!("Creating browser window: {} ({}x{})", title, width, height);
        
        let mut window = Self {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            width,
            height,
            kernel,
            tabs: Vec::new(),
            active_tab: None,
        };
        
        // Create initial tab with home page
        window.create_tab("https://vantis.ai/home".to_string()).await?;
        
        Ok(window)
    }
    
    /// Create a new tab
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
    
    /// Close current tab
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
    
    /// Navigate to URL
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
    
    /// Go back in history
    pub async fn go_back(&mut self) -> Result<()> {
        info!("Going back");
        
        if let Some(active) = self.active_tab {
            if let Some(renderer) = &self.tabs[active].web_renderer {
                renderer.go_back().await?;
            }
            debug!("Navigating back in tab: {}", self.tabs[active].title);
        }
        
        Ok(())
    }
    
    /// Go forward in history
    pub async fn go_forward(&mut self) -> Result<()> {
        info!("Going forward");
        
        if let Some(active) = self.active_tab {
            if let Some(renderer) = &self.tabs[active].web_renderer {
                renderer.go_forward().await?;
            }
            debug!("Navigating forward in tab: {}", self.tabs[active].title);
        }
        
        Ok(())
    }
    
    /// Refresh current page
    pub async fn refresh(&mut self) -> Result<()> {
        info!("Refreshing page");
        
        if let Some(active) = self.active_tab {
            self.tabs[active].is_loading = true;
            
            // Reload page in WebRenderer
            if let Some(renderer) = &self.tabs[active].web_renderer {
                renderer.reload().await?;
            }
            
            self.tabs[active].is_loading = false;
        }
        
        Ok(())
    }
    
    /// Get window ID
    pub fn id(&self) -> &str {
        &self.id
    }
    
    /// Get window title
    pub fn title(&self) -> &str {
        &self.title
    }
    
    /// Get tabs
    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }
    
    /// Get active tab
    pub fn active_tab(&self) -> Option<&Tab> {
        self.active_tab.and_then(|idx| self.tabs.get(idx))
    }
    
    /// Get WebRenderer for active tab
    pub fn active_web_renderer(&self) -> Option<Arc<WebRenderer>> {
        self.active_tab().and_then(|tab| tab.web_renderer.clone())
    }
    
    /// Close window
    pub async fn close(&mut self) -> Result<()> {
        info!("Closing browser window: {}", self.title);
        
        // Cleanup
        self.tabs.clear();
        self.active_tab = None;
        
        info!("✓ Window closed");
        
        Ok(())
    }
}