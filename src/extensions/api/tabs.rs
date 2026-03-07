//! Tabs API
//!
//! Provides APIs for extensions to interact with browser tabs.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Global tabs state
static mut TABS_STATE: Option<Arc<RwLock<TabsState>>> = None;

fn get_tabs_state() -> Arc<RwLock<TabsState>> {
    unsafe {
        if TABS_STATE.is_none() {
            TABS_STATE = Some(Arc::new(RwLock::new(TabsState::new())));
        }
        TABS_STATE.clone().unwrap()
    }
}

/// Tabs state
struct TabsState {
    tabs: HashMap<String, Tab>,
    active_tab_id: Option<String>,
    tab_listeners: Vec<Box<dyn Fn(Tab) + Send + Sync>>,
    update_listeners: Vec<Box<dyn Fn(String, Tab, Tab) + Send + Sync>>,
    remove_listeners: Vec<Box<dyn Fn(String, RemoveInfo) + Send + Sync>>,
}

impl TabsState {
    fn new() -> Self {
        Self {
            tabs: HashMap::new(),
            active_tab_id: None,
            tab_listeners: Vec::new(),
            update_listeners: Vec::new(),
            remove_listeners: Vec::new(),
        }
    }
}

/// Tabs API
pub struct TabsAPI {
    /// Extension ID
    extension_id: String,
}

impl TabsAPI {
    /// Creates a new tabs API
    pub fn new(extension_id: String) -> Self {
        Self { extension_id }
    }

    /// Gets all tabs
    pub fn get_all(&self) -> Result<Vec<Tab>> {
        let state = get_tabs_state();
        let guard = state.read().unwrap();
        
        Ok(guard.tabs.values().cloned().collect())
    }

    /// Gets a tab by ID
    pub fn get(&self, tab_id: &str) -> Result<Option<Tab>> {
        let state = get_tabs_state();
        let guard = state.read().unwrap();
        
        Ok(guard.tabs.get(tab_id).cloned())
    }

    /// Gets the active tab in the current window
    pub fn get_active(&self) -> Result<Option<Tab>> {
        let state = get_tabs_state();
        let guard = state.read().unwrap();
        
        if let Some(active_id) = &guard.active_tab_id {
            Ok(guard.tabs.get(active_id).cloned())
        } else {
            Ok(None)
        }
    }

    /// Creates a new tab
    pub fn create(&self, options: CreateTabOptions) -> Result<Tab> {
        let state = get_tabs_state();
        let mut guard = state.write().unwrap();
        
        let tab_id = uuid::Uuid::new_v4().to_string();
        let url = options.url.unwrap_or_else(|| "about:blank".to_string());
        let active = options.active.unwrap_or(true);
        let pinned = options.pinned.unwrap_or(false);
        
        // Deactivate other tabs if this one is active
        if active {
            for tab in guard.tabs.values_mut() {
                tab.active = false;
            }
            guard.active_tab_id = Some(tab_id.clone());
        }
        
        let tab = Tab {
            id: tab_id.clone(),
            url,
            title: "New Tab".to_string(),
            active,
            window_id: options.window_id.unwrap_or(0),
            index: options.index.unwrap_or_else(|| guard.tabs.len() as u32),
            pinned,
            status: TabStatus::Loading,
            fav_icon_url: None,
        };
        
        guard.tabs.insert(tab_id, tab.clone());
        
        // Notify listeners
        let listeners: Vec<Box<dyn Fn(Tab) + Send + Sync>> = guard.tab_listeners.iter()
            .map(|l| unsafe { std::ptr::read(l) })
            .collect();
        drop(guard);
        
        for listener in listeners {
            listener(tab.clone());
        }
        
        Ok(tab)
    }

    /// Updates a tab
    pub fn update(&self, tab_id: &str, options: UpdateTabOptions) -> Result<Tab> {
        let state = get_tabs_state();
        let mut guard = state.write().unwrap();
        
        if let Some(tab) = guard.tabs.get_mut(tab_id) {
            let old_tab = tab.clone();
            
            if let Some(url) = &options.url {
                tab.url = url.clone();
                tab.status = TabStatus::Loading;
            }
            if let Some(active) = options.active {
                if active {
                    // Deactivate other tabs
                    for t in guard.tabs.values_mut() {
                        t.active = false;
                    }
                    guard.active_tab_id = Some(tab_id.to_string());
                }
                tab.active = active;
            }
            if let Some(pinned) = options.pinned {
                tab.pinned = pinned;
            }
            if let Some(title) = &options.title {
                tab.title = title.clone();
            }
            
            let updated_tab = tab.clone();
            
            // Notify update listeners
            let listeners: Vec<_> = guard.update_listeners.iter()
                .map(|l| unsafe { std::ptr::read(l) })
                .collect();
            drop(guard);
            
            for listener in listeners {
                listener(tab_id.to_string(), old_tab, updated_tab.clone());
            }
            
            Ok(updated_tab)
        } else {
            Err(anyhow::anyhow!("Tab not found: {}", tab_id))
        }
    }

    /// Removes a tab
    pub fn remove(&self, tab_id: &str) -> Result<()> {
        let state = get_tabs_state();
        let mut guard = state.write().unwrap();
        
        if let Some(tab) = guard.tabs.remove(tab_id) {
            let window_id = tab.window_id;
            
            // If we removed the active tab, activate another
            if guard.active_tab_id.as_deref() == Some(tab_id) {
                guard.active_tab_id = guard.tabs.keys().next().cloned();
                if let Some(new_active_id) = &guard.active_tab_id {
                    if let Some(new_tab) = guard.tabs.get_mut(new_active_id) {
                        new_tab.active = true;
                    }
                }
            }
            
            let remove_info = RemoveInfo {
                window_id,
                is_window_closing: false,
            };
            
            // Notify remove listeners
            let listeners: Vec<_> = guard.remove_listeners.iter()
                .map(|l| unsafe { std::ptr::read(l) })
                .collect();
            drop(guard);
            
            for listener in listeners {
                listener(tab_id.to_string(), remove_info);
            }
        }
        
        Ok(())
    }

    /// Reloads a tab
    pub fn reload(&self, tab_id: &str) -> Result<()> {
        let state = get_tabs_state();
        let mut guard = state.write().unwrap();
        
        if let Some(tab) = guard.tabs.get_mut(tab_id) {
            tab.status = TabStatus::Loading;
            // In a real implementation, this would trigger a page reload
            log::info!("Reloading tab {}: {}", tab_id, tab.url);
        }
        
        Ok(())
    }

    /// Executes JavaScript in a tab
    pub fn execute_script(&self, tab_id: &str, code: &str) -> Result<ScriptResult> {
        let state = get_tabs_state();
        let guard = state.read().unwrap();
        
        if !guard.tabs.contains_key(tab_id) {
            return Err(anyhow::anyhow!("Tab not found: {}", tab_id));
        }
        
        // In a real implementation, this would execute JS in the webview
        log::info!("Executing script in tab {}: {} bytes", tab_id, code.len());
        
        Ok(ScriptResult {
            result: Some(serde_json::json!({ "executed": true })),
            error: None,
        })
    }

    /// Sends a message to a tab
    pub fn send_message(&self, tab_id: &str, message: serde_json::Value) -> Result<serde_json::Value> {
        let state = get_tabs_state();
        let guard = state.read().unwrap();
        
        if !guard.tabs.contains_key(tab_id) {
            return Err(anyhow::anyhow!("Tab not found: {}", tab_id));
        }
        
        // In a real implementation, this would send a message to the tab's content script
        log::debug!("Sending message to tab {}: {:?}", tab_id, message);
        
        Ok(serde_json::json!({ "sent": true }))
    }

    /// Listens for tab created events
    pub fn on_created<F>(&self, callback: F)
    where
        F: Fn(Tab) + Send + Sync + 'static,
    {
        let state = get_tabs_state();
        let mut guard = state.write().unwrap();
        guard.tab_listeners.push(Box::new(callback));
    }

    /// Listens for tab updated events
    pub fn on_updated<F>(&self, callback: F)
    where
        F: Fn(String, Tab, Tab) + Send + Sync + 'static,
    {
        let state = get_tabs_state();
        let mut guard = state.write().unwrap();
        guard.update_listeners.push(Box::new(callback));
    }

    /// Listens for tab removed events
    pub fn on_removed<F>(&self, callback: F)
    where
        F: Fn(String, RemoveInfo) + Send + Sync + 'static,
    {
        let state = get_tabs_state();
        let mut guard = state.write().unwrap();
        guard.remove_listeners.push(Box::new(callback));
    }
    
    /// Query tabs by criteria
    pub fn query(&self, query_info: QueryInfo) -> Result<Vec<Tab>> {
        let state = get_tabs_state();
        let guard = state.read().unwrap();
        
        let tabs: Vec<Tab> = guard.tabs.values()
            .filter(|tab| {
                if let Some(active) = query_info.active {
                    if tab.active != active {
                        return false;
                    }
                }
                if let Some(pinned) = query_info.pinned {
                    if tab.pinned != pinned {
                        return false;
                    }
                }
                if let Some(url) = &query_info.url {
                    if &tab.url != url {
                        return false;
                    }
                }
                if let Some(status) = &query_info.status {
                    if &tab.status != status {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();
        
        Ok(tabs)
    }
    
    /// Duplicate a tab
    pub fn duplicate(&self, tab_id: &str) -> Result<Tab> {
        let state = get_tabs_state();
        let guard = state.read().unwrap();
        
        if let Some(original) = guard.tabs.get(tab_id) {
            let options = CreateTabOptions {
                url: Some(original.url.clone()),
                active: Some(false),
                window_id: Some(original.window_id),
                index: Some(original.index + 1),
                pinned: Some(original.pinned),
            };
            
            drop(guard);
            return self.create(options);
        }
        
        Err(anyhow::anyhow!("Tab not found: {}", tab_id))
    }
    
    /// Move a tab to a different position
    pub fn move_tab(&self, tab_id: &str, new_index: u32) -> Result<()> {
        let state = get_tabs_state();
        let mut guard = state.write().unwrap();
        
        if let Some(tab) = guard.tabs.get_mut(tab_id) {
            tab.index = new_index;
        }
        
        Ok(())
    }
}

/// Tab information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tab {
    /// Tab ID
    pub id: String,
    /// Tab URL
    pub url: String,
    /// Tab title
    pub title: String,
    /// Whether the tab is active
    pub active: bool,
    /// Window ID
    pub window_id: u32,
    /// Tab index in the window
    pub index: u32,
    /// Whether the tab is pinned
    pub pinned: bool,
    /// Tab status
    pub status: TabStatus,
    /// Favicon URL
    pub fav_icon_url: Option<String>,
}

/// Tab status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TabStatus {
    Loading,
    Complete,
    Error,
}

/// Options for creating a tab
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTabOptions {
    /// URL to load in the tab
    pub url: Option<String>,
    /// Whether the tab should be active
    pub active: Option<bool>,
    /// Window ID to create the tab in
    pub window_id: Option<u32>,
    /// Index to insert the tab at
    pub index: Option<u32>,
    /// Whether the tab should be pinned
    pub pinned: Option<bool>,
}

impl Default for CreateTabOptions {
    fn default() -> Self {
        Self {
            url: Some("about:blank".to_string()),
            active: Some(true),
            window_id: None,
            index: None,
            pinned: Some(false),
        }
    }
}

/// Options for updating a tab
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateTabOptions {
    /// New URL for the tab
    pub url: Option<String>,
    /// Whether the tab should be active
    pub active: Option<bool>,
    /// Whether the tab should be pinned
    pub pinned: Option<bool>,
    /// New title for the tab
    pub title: Option<String>,
}

/// Query options for filtering tabs
#[derive(Debug, Clone, Default)]
pub struct QueryInfo {
    pub active: Option<bool>,
    pub pinned: Option<bool>,
    pub url: Option<String>,
    pub status: Option<TabStatus>,
}

/// Script execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptResult {
    /// Script result
    pub result: Option<serde_json::Value>,
    /// Script error
    pub error: Option<String>,
}

/// Information about a removed tab
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveInfo {
    /// Window ID
    pub window_id: u32,
    /// Whether the window is closing
    pub is_window_closing: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tabs_api_creation() {
        let api = TabsAPI::new("test-extension".to_string());
        assert_eq!(api.extension_id, "test-extension");
    }

    #[test]
    fn test_create_tab() {
        let api = TabsAPI::new("test-create".to_string());
        let options = CreateTabOptions {
            url: Some("https://example.com".to_string()),
            active: Some(true),
            window_id: None,
            index: None,
            pinned: None,
        };

        let tab = api.create(options).unwrap();
        assert_eq!(tab.url, "https://example.com");
        assert!(tab.active);
    }

    #[test]
    fn test_update_tab() {
        let api = TabsAPI::new("test-update".to_string());
        
        let tab = api.create(CreateTabOptions::default()).unwrap();
        
        let options = UpdateTabOptions {
            url: Some("https://updated.com".to_string()),
            active: Some(false),
            ..Default::default()
        };

        let updated = api.update(&tab.id, options).unwrap();
        assert_eq!(updated.url, "https://updated.com");
    }

    #[test]
    fn test_remove_tab() {
        let api = TabsAPI::new("test-remove".to_string());
        
        let tab = api.create(CreateTabOptions::default()).unwrap();
        assert!(api.get(&tab.id).unwrap().is_some());
        
        api.remove(&tab.id).unwrap();
        assert!(api.get(&tab.id).unwrap().is_none());
    }

    #[test]
    fn test_query_tabs() {
        let api = TabsAPI::new("test-query".to_string());
        
        api.create(CreateTabOptions {
            url: Some("https://example.com".to_string()),
            active: Some(true),
            pinned: Some(true),
            ..Default::default()
        }).unwrap();
        
        api.create(CreateTabOptions {
            url: Some("https://other.com".to_string()),
            active: Some(false),
            pinned: Some(false),
            ..Default::default()
        }).unwrap();

        let pinned = api.query(QueryInfo { pinned: Some(true), ..Default::default() }).unwrap();
        assert_eq!(pinned.len(), 1);

        let active = api.query(QueryInfo { active: Some(true), ..Default::default() }).unwrap();
        assert_eq!(active.len(), 1);
    }

    #[test]
    fn test_duplicate_tab() {
        let api = TabsAPI::new("test-duplicate".to_string());
        
        let original = api.create(CreateTabOptions {
            url: Some("https://example.com".to_string()),
            pinned: Some(true),
            ..Default::default()
        }).unwrap();
        
        let duplicate = api.duplicate(&original.id).unwrap();
        assert_eq!(duplicate.url, original.url);
        assert_ne!(duplicate.id, original.id);
    }
}