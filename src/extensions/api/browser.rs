//! Browser API
//!
//! Provides browser-level APIs for extensions.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Global browser state
static mut BROWSER_STATE: Option<Arc<RwLock<BrowserState>>> = None;

fn get_browser_state() -> Arc<RwLock<BrowserState>> {
    unsafe {
        if BROWSER_STATE.is_none() {
            BROWSER_STATE = Some(Arc::new(RwLock::new(BrowserState::new())));
        }
        BROWSER_STATE.clone().unwrap()
    }
}

/// Browser state
struct BrowserState {
    tabs: HashMap<String, TabInfo>,
    active_tab_id: Option<String>,
    extension_badges: HashMap<String, BadgeState>,
    version: String,
}

impl BrowserState {
    fn new() -> Self {
        Self {
            tabs: HashMap::new(),
            active_tab_id: None,
            extension_badges: HashMap::new(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Badge state for an extension
#[derive(Debug, Clone)]
struct BadgeState {
    text: Option<String>,
    color: BadgeColor,
}

/// Browser API
pub struct BrowserAPI {
    /// Extension ID
    extension_id: String,
}

impl BrowserAPI {
    /// Creates a new browser API
    pub fn new(extension_id: String) -> Self {
        // Initialize badge state for this extension
        let state = get_browser_state();
        let mut guard = state.write().unwrap();
        guard.extension_badges.entry(extension_id.clone()).or_insert_with(|| BadgeState {
            text: None,
            color: BadgeColor { r: 0, g: 0, b: 0, a: 255 },
        });
        
        Self { extension_id }
    }

    /// Gets browser runtime info
    pub fn get_runtime(&self) -> Result<RuntimeInfo> {
        let state = get_browser_state();
        let guard = state.read().unwrap();
        
        Ok(RuntimeInfo {
            id: self.extension_id.clone(),
            name: "VantisWeb".to_string(),
            version: guard.version.clone(),
        })
    }

    /// Opens a new tab
    pub fn open_tab(&self, url: &str) -> Result<String> {
        let state = get_browser_state();
        let mut guard = state.write().unwrap();
        
        let tab_id = uuid::Uuid::new_v4().to_string();
        let tab_info = TabInfo {
            id: tab_id.clone(),
            title: url.to_string(),
            url: url.to_string(),
            active: true,
            window_id: "main".to_string(),
            index: guard.tabs.len() as u32,
            status: TabStatus::Loading,
        };
        
        // Deactivate other tabs
        for tab in guard.tabs.values_mut() {
            tab.active = false;
        }
        
        guard.tabs.insert(tab_id.clone(), tab_info);
        guard.active_tab_id = Some(tab_id.clone());
        
        Ok(tab_id)
    }

    /// Closes a tab
    pub fn close_tab(&self, tab_id: &str) -> Result<()> {
        let state = get_browser_state();
        let mut guard = state.write().unwrap();
        
        if guard.tabs.remove(tab_id).is_some() {
            // If we closed the active tab, activate another
            if guard.active_tab_id.as_deref() == Some(tab_id) {
                guard.active_tab_id = guard.tabs.keys().next().cloned();
                if let Some(new_active_id) = &guard.active_tab_id {
                    if let Some(tab) = guard.tabs.get_mut(new_active_id) {
                        tab.active = true;
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Gets all tabs
    pub fn get_tabs(&self) -> Result<Vec<TabInfo>> {
        let state = get_browser_state();
        let guard = state.read().unwrap();
        
        Ok(guard.tabs.values().cloned().collect())
    }

    /// Gets the active tab
    pub fn get_active_tab(&self) -> Result<Option<TabInfo>> {
        let state = get_browser_state();
        let guard = state.read().unwrap();
        
        if let Some(active_id) = &guard.active_tab_id {
            Ok(guard.tabs.get(active_id).cloned())
        } else {
            Ok(None)
        }
    }

    /// Sets the badge text
    pub fn set_badge_text(&self, text: &str) -> Result<()> {
        let state = get_browser_state();
        let mut guard = state.write().unwrap();
        
        if let Some(badge) = guard.extension_badges.get_mut(&self.extension_id) {
            badge.text = Some(text.to_string());
        }
        
        Ok(())
    }

    /// Gets the badge text
    pub fn get_badge_text(&self) -> Result<Option<String>> {
        let state = get_browser_state();
        let guard = state.read().unwrap();
        
        Ok(guard.extension_badges.get(&self.extension_id).and_then(|b| b.text.clone()))
    }

    /// Sets the badge color
    pub fn set_badge_color(&self, color: BadgeColor) -> Result<()> {
        let state = get_browser_state();
        let mut guard = state.write().unwrap();
        
        if let Some(badge) = guard.extension_badges.get_mut(&self.extension_id) {
            badge.color = color;
        }
        
        Ok(())
    }
    
    /// Gets the badge color
    pub fn get_badge_color(&self) -> Result<BadgeColor> {
        let state = get_browser_state();
        let guard = state.read().unwrap();
        
        Ok(guard.extension_badges.get(&self.extension_id)
            .map(|b| b.color.clone())
            .unwrap_or(BadgeColor { r: 0, g: 0, b: 0, a: 255 }))
    }

    /// Shows a notification
    pub fn show_notification(&self, notification: Notification) -> Result<String> {
        // In a real implementation, this would integrate with the system notification API
        log::info!(
            "Extension {} showing notification: {} - {}",
            self.extension_id,
            notification.title,
            notification.message
        );
        
        let notification_id = uuid::Uuid::new_v4().to_string();
        Ok(notification_id)
    }
    
    /// Updates a tab
    pub fn update_tab(&self, tab_id: &str, update: TabUpdate) -> Result<()> {
        let state = get_browser_state();
        let mut guard = state.write().unwrap();
        
        if let Some(tab) = guard.tabs.get_mut(tab_id) {
            if let Some(url) = update.url {
                tab.url = url;
            }
            if let Some(active) = update.active {
                if active {
                    // Deactivate other tabs
                    for t in guard.tabs.values_mut() {
                        t.active = false;
                    }
                    guard.active_tab_id = Some(tab_id.to_string());
                }
                tab.active = active;
            }
        }
        
        Ok(())
    }
    
    /// Get a specific tab by ID
    pub fn get_tab(&self, tab_id: &str) -> Result<Option<TabInfo>> {
        let state = get_browser_state();
        let guard = state.read().unwrap();
        
        Ok(guard.tabs.get(tab_id).cloned())
    }
}

/// Runtime information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeInfo {
    /// Extension ID
    pub id: String,
    /// Browser name
    pub name: String,
    /// Browser version
    pub version: String,
}

/// Tab information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabInfo {
    /// Tab ID
    pub id: String,
    /// Tab title
    pub title: String,
    /// Tab URL
    pub url: String,
    /// Whether the tab is active
    pub active: bool,
    /// Window ID
    pub window_id: String,
    /// Tab index
    pub index: u32,
    /// Tab status
    pub status: TabStatus,
}

/// Tab status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TabStatus {
    Loading,
    Complete,
    Error,
}

/// Tab update options
#[derive(Debug, Clone, Default)]
pub struct TabUpdate {
    pub url: Option<String>,
    pub active: Option<bool>,
}

/// Badge color
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadgeColor {
    /// Red component (0-255)
    pub r: u8,
    /// Green component (0-255)
    pub g: u8,
    /// Blue component (0-255)
    pub b: u8,
    /// Alpha component (0-255)
    pub a: u8,
}

impl Default for BadgeColor {
    fn default() -> Self {
        Self { r: 0, g: 0, b: 0, a: 255 }
    }
}

impl BadgeColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
    
    pub fn red() -> Self {
        Self { r: 255, g: 0, b: 0, a: 255 }
    }
    
    pub fn green() -> Self {
        Self { r: 0, g: 255, b: 0, a: 255 }
    }
    
    pub fn blue() -> Self {
        Self { r: 0, g: 0, b: 255, a: 255 }
    }
}

/// Notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Notification title
    pub title: String,
    /// Notification message
    pub message: String,
    /// Notification icon URL
    pub icon_url: Option<String>,
    /// Notification type
    #[serde(default)]
    pub notification_type: NotificationType,
}

impl Notification {
    pub fn new(title: &str, message: &str) -> Self {
        Self {
            title: title.to_string(),
            message: message.to_string(),
            icon_url: None,
            notification_type: NotificationType::Basic,
        }
    }
    
    pub fn with_icon(mut self, icon_url: &str) -> Self {
        self.icon_url = Some(icon_url.to_string());
        self
    }
}

/// Notification type
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum NotificationType {
    #[default]
    Basic,
    Image,
    List,
    Progress,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_api_creation() {
        let api = BrowserAPI::new("test-extension".to_string());
        assert_eq!(api.extension_id, "test-extension");
    }

    #[test]
    fn test_get_runtime() {
        let api = BrowserAPI::new("test-extension".to_string());
        let runtime = api.get_runtime().unwrap();

        assert_eq!(runtime.id, "test-extension");
        assert_eq!(runtime.name, "VantisWeb");
    }

    #[test]
    fn test_open_and_close_tab() {
        let api = BrowserAPI::new("test-tabs".to_string());
        
        let tab_id = api.open_tab("https://example.com").unwrap();
        assert!(!tab_id.is_empty());
        
        let tabs = api.get_tabs().unwrap();
        assert_eq!(tabs.len(), 1);
        
        api.close_tab(&tab_id).unwrap();
        let tabs = api.get_tabs().unwrap();
        assert!(tabs.is_empty());
    }

    #[test]
    fn test_active_tab() {
        let api = BrowserAPI::new("test-active".to_string());
        
        let tab_id = api.open_tab("https://example.com").unwrap();
        let active = api.get_active_tab().unwrap();
        
        assert!(active.is_some());
        assert_eq!(active.unwrap().id, tab_id);
    }

    #[test]
    fn test_badge() {
        let api = BrowserAPI::new("test-badge".to_string());
        
        api.set_badge_text("5").unwrap();
        let text = api.get_badge_text().unwrap();
        assert_eq!(text, Some("5".to_string()));
        
        api.set_badge_color(BadgeColor::red()).unwrap();
        let color = api.get_badge_color().unwrap();
        assert_eq!(color.r, 255);
    }

    #[test]
    fn test_notification() {
        let api = BrowserAPI::new("test-notification".to_string());
        
        let notification = Notification::new("Test", "This is a test notification");
        let result = api.show_notification(notification);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_badge_color_presets() {
        assert_eq!(BadgeColor::red().r, 255);
        assert_eq!(BadgeColor::green().g, 255);
        assert_eq!(BadgeColor::blue().b, 255);
    }
}