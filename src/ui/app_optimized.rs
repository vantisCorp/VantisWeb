//! VantisUI - Main Application UI (Optimized)
//! 
//! Central UI orchestration:
//! - Window management
//! - Event handling
//! - Theme management
//! - Rendering coordination
//!
//! Optimizations:
//! - Pre-allocated collection capacities
//! - Reduced clones
//! - Optimized string operations

use anyhow::Result;
use log::{debug, info, warn};
use std::sync::Arc;

use crate::core::kernel::VantisKernel;
use super::browser::BrowserWindow;
use super::theming::ThemeManager;
use super::renderer::GPURenderer;

/// VantisUI - Main UI Application (Optimized)
pub struct VantisUI {
    kernel: Arc<VantisKernel>,
    theme_manager: ThemeManager,
    renderer: GPURenderer,
    windows: Vec<BrowserWindow>,
    is_running: bool,
}

impl VantisUI {
    /// Create a new VantisUI instance (Optimized)
    pub async fn new(kernel: Arc<VantisKernel>) -> Result<Self> {
        info!("Initializing VantisUI...");
        
        let theme_manager = ThemeManager::new()?;
        let renderer = GPURenderer::new()?;
        
        info!("✓ VantisUI initialized");
        
        Ok(Self {
            kernel,
            theme_manager,
            renderer,
            windows: Vec::with_capacity(10),
            is_running: false,
        })
    }
    
    /// Run the UI main loop
    pub async fn run(&mut self) -> Result<()> {
        if self.is_running {
            warn!("UI is already running");
            return Ok(());
        }
        
        info!("Starting VantisUI main loop...");
        self.is_running = true;
        
        // Create main browser window
        self.create_main_window().await?;
        
        // Start render loop
        self.start_render_loop().await?;
        
        // Start event loop
        self.event_loop().await?;
        
        Ok(())
    }
    
    /// Create main browser window (Optimized)
    async fn create_main_window(&mut self) -> Result<()> {
        info!("Creating main browser window...");
        
        let window = BrowserWindow::new(
            "VantisWeb Browser".to_string(),
            1920,
            1080,
            self.kernel.clone(),
        ).await?;
        
        self.windows.push(window);
        
        info!("✓ Main window created");
        
        Ok(())
    }
    
    /// Start GPU render loop
    async fn start_render_loop(&self) -> Result<()> {
        info!("Starting GPU render loop (144Hz target)...");
        
        // In production: Implement WebGPU render loop
        // For MVP: Simulate render loop
        let mut frame_count = 0u64;
        
        let _loop_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(7));
            
            loop {
                interval.tick().await;
                frame_count += 1;
                
                // Render frame
                debug!("Rendering frame: {} (Target: 144Hz)", frame_count);
                
                // In production: WebGPU rendering here
            }
        });
        
        info!("✓ Render loop started");
        
        Ok(())
    }
    
    /// Main event loop
    async fn event_loop(&mut self) -> Result<()> {
        info!("Starting main event loop...");
        
        // In production: Process window events, user input, etc.
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
        
        while self.is_running {
            interval.tick().await;
            
            // Process events
            self.process_events().await?;
            
            // Update UI
            self.update_ui().await?;
        }
        
        info!("Event loop stopped");
        
        Ok(())
    }
    
    /// Process UI events
    async fn process_events(&mut self) -> Result<()> {
        // In production: Handle keyboard, mouse, window events
        debug!("Processing UI events");
        
        Ok(())
    }
    
    /// Update UI
    async fn update_ui(&mut self) -> Result<()> {
        // In production: Update UI state, redraw if needed
        debug!("Updating UI");
        
        Ok(())
    }
    
    /// Stop the UI
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping VantisUI...");
        
        self.is_running = false;
        
        info!("✓ VantisUI stopped");
        
        Ok(())
    }
    
    /// Get theme manager (Optimized - returns reference)
    pub fn get_theme_manager(&self) -> &ThemeManager {
        &self.theme_manager
    }
    
    /// Get renderer (Optimized - returns reference)
    pub fn get_renderer(&self) -> &GPURenderer {
        &self.renderer
    }
    
    /// Get windows (Optimized - returns reference)
    pub fn get_windows(&self) -> &[BrowserWindow] {
        &self.windows
    }
    
    /// Is running
    pub fn is_running(&self) -> bool {
        self.is_running
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_vantis_ui_creation() {
        let kernel = Arc::new(VantisKernel::new().await.unwrap());
        let ui = VantisUI::new(kernel).await.unwrap();
        
        assert!(!ui.is_running());
        assert_eq!(ui.get_windows().len(), 0);
    }

    #[tokio::test]
    async fn test_vantis_ui_stop() {
        let kernel = Arc::new(VantisKernel::new().await.unwrap());
        let mut ui = VantisUI::new(kernel).await.unwrap();
        
        ui.stop().await.unwrap();
        assert!(!ui.is_running());
    }
}