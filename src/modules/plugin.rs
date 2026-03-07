//! Plugin Manager
//! 
//! Manages plugins for extending browser functionality

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Plugin manager
pub struct PluginManager {
    /// Loaded plugins
    plugins: Arc<Mutex<HashMap<String, Plugin>>>,
    /// Plugin hooks
    hooks: Arc<Mutex<HashMap<PluginHook, Vec<String>>>>,
}

/// Plugin instance
#[derive(Debug, Clone)]
pub struct Plugin {
    /// Plugin ID
    pub id: String,
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin path
    pub path: PathBuf,
    /// Plugin state
    pub state: PluginState,
    /// Plugin permissions
    pub permissions: Vec<PluginPermission>,
    /// Registered hooks
    pub registered_hooks: Vec<PluginHook>,
}

/// Plugin state
#[derive(Debug, Clone, PartialEq)]
pub enum PluginState {
    /// Loaded but not active
    Inactive,
    /// Active and running
    Active,
    /// Disabled
    Disabled,
    /// Error state
    Error(String),
}

/// Plugin permission
#[derive(Debug, Clone, PartialEq)]
pub enum PluginPermission {
    /// Access to web requests
    WebRequest,
    /// Access to tabs
    Tabs,
    /// Access to bookmarks
    Bookmarks,
    /// Access to history
    History,
    /// Access to cookies
    Cookies,
    /// Access to storage
    Storage,
    /// Access to notifications
    Notifications,
    /// Access to clipboard
    Clipboard,
    /// Native messaging
    NativeMessaging,
    /// Custom permission
    Custom(String),
}

/// Plugin hook points
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum PluginHook {
    /// Before navigation
    BeforeNavigate,
    /// After navigation
    AfterNavigate,
    /// Before request
    BeforeRequest,
    /// After response
    AfterResponse,
    /// On page load
    PageLoad,
    /// On page unload
    PageUnload,
    /// On tab created
    TabCreated,
    /// On tab removed
    TabRemoved,
    /// On tab activated
    TabActivated,
    /// On download started
    DownloadStarted,
    /// On download complete
    DownloadComplete,
    /// On message received
    MessageReceived,
    /// Custom hook
    Custom(String),
}

/// Plugin API context
pub struct PluginContext {
    /// Plugin ID
    pub plugin_id: String,
    /// Current tab ID (if applicable)
    pub tab_id: Option<String>,
    /// Current URL (if applicable)
    pub url: Option<String>,
    /// Additional data
    pub data: HashMap<String, serde_json::Value>,
}

impl PluginManager {
    /// Creates a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(Mutex::new(HashMap::new())),
            hooks: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Loads a plugin from path
    pub fn load_plugin(&self, path: &PathBuf) -> Result<Plugin> {
        log::info!("Loading plugin from: {:?}", path);
        
        // Read manifest
        let manifest_path = path.join("plugin.json");
        if !manifest_path.exists() {
            return Err(anyhow!("Plugin manifest not found"));
        }
        
        let manifest_content = std::fs::read_to_string(&manifest_path)?;
        let manifest: PluginManifest = serde_json::from_str(&manifest_content)?;
        
        let plugin = Plugin {
            id: manifest.id.clone(),
            name: manifest.name,
            version: manifest.version,
            path: path.clone(),
            state: PluginState::Active,
            permissions: manifest.permissions,
            registered_hooks: vec![],
        };
        
        // Register hooks
        for hook in &manifest.hooks {
            self.register_hook(&plugin.id, hook.clone());
        }
        
        // Store plugin
        self.plugins.lock().unwrap().insert(plugin.id.clone(), plugin.clone());
        
        log::info!("Plugin loaded: {} v{}", plugin.name, plugin.version);
        Ok(plugin)
    }
    
    /// Unloads a plugin
    pub fn unload_plugin(&self, plugin_id: &str) -> Result<()> {
        let mut plugins = self.plugins.lock().unwrap();
        
        if let Some(plugin) = plugins.remove(plugin_id) {
            // Unregister hooks
            let mut hooks = self.hooks.lock().unwrap();
            for hook in &plugin.registered_hooks {
                if let Some(plugin_list) = hooks.get_mut(hook) {
                    plugin_list.retain(|id| id != plugin_id);
                }
            }
            
            log::info!("Plugin unloaded: {}", plugin_id);
        }
        
        Ok(())
    }
    
    /// Enables a plugin
    pub fn enable_plugin(&self, plugin_id: &str) -> Result<()> {
        let mut plugins = self.plugins.lock().unwrap();
        
        if let Some(plugin) = plugins.get_mut(plugin_id) {
            plugin.state = PluginState::Active;
            log::info!("Plugin enabled: {}", plugin_id);
        }
        
        Ok(())
    }
    
    /// Disables a plugin
    pub fn disable_plugin(&self, plugin_id: &str) -> Result<()> {
        let mut plugins = self.plugins.lock().unwrap();
        
        if let Some(plugin) = plugins.get_mut(plugin_id) {
            plugin.state = PluginState::Disabled;
            log::info!("Plugin disabled: {}", plugin_id);
        }
        
        Ok(())
    }
    
    /// Gets a plugin
    pub fn get_plugin(&self, plugin_id: &str) -> Option<Plugin> {
        self.plugins.lock().unwrap().get(plugin_id).cloned()
    }
    
    /// Gets all plugins
    pub fn get_plugins(&self) -> Vec<Plugin> {
        self.plugins.lock().unwrap().values().cloned().collect()
    }
    
    /// Gets active plugins
    pub fn get_active_plugins(&self) -> Vec<Plugin> {
        self.plugins.lock().unwrap()
            .values()
            .filter(|p| p.state == PluginState::Active)
            .cloned()
            .collect()
    }
    
    /// Registers a hook for a plugin
    fn register_hook(&self, plugin_id: &str, hook: PluginHook) {
        let mut hooks = self.hooks.lock().unwrap();
        hooks.entry(hook).or_insert_with(Vec::new).push(plugin_id.to_string());
        
        if let Some(plugin) = self.plugins.lock().unwrap().get_mut(plugin_id) {
            plugin.registered_hooks.push(hook);
        }
    }
    
    /// Triggers a hook
    pub fn trigger_hook(&self, hook: &PluginHook, context: &PluginContext) -> Result<Vec<HookResult>> {
        let hooks = self.hooks.lock().unwrap();
        let plugins = self.plugins.lock().unwrap();
        
        let mut results = Vec::new();
        
        if let Some(plugin_ids) = hooks.get(hook) {
            for plugin_id in plugin_ids {
                if let Some(plugin) = plugins.get(plugin_id) {
                    if plugin.state != PluginState::Active {
                        continue;
                    }
                    
                    // Execute hook (in real implementation, would call into plugin)
                    let result = self.execute_hook(plugin, hook, context);
                    results.push(result);
                }
            }
        }
        
        Ok(results)
    }
    
    /// Executes a hook in a plugin
    fn execute_hook(&self, plugin: &Plugin, hook: &PluginHook, context: &PluginContext) -> HookResult {
        log::debug!("Executing hook {:?} for plugin {}", hook, plugin.id);
        
        // In real implementation, would call into plugin's JavaScript/WASM code
        // For now, return default result
        HookResult {
            plugin_id: plugin.id.clone(),
            action: HookAction::Continue,
            data: None,
        }
    }
    
    /// Sends a message to a plugin
    pub fn send_message(&self, plugin_id: &str, message: &str) -> Result<String> {
        let plugins = self.plugins.lock().unwrap();
        
        let plugin = plugins.get(plugin_id)
            .ok_or_else(|| anyhow!("Plugin not found: {}", plugin_id))?;
        
        if plugin.state != PluginState::Active {
            return Err(anyhow!("Plugin is not active: {}", plugin_id));
        }
        
        log::debug!("Sending message to plugin {}: {}", plugin_id, message);
        
        // In real implementation, would call into plugin's message handler
        Ok("ack".to_string())
    }
    
    /// Checks if a plugin has a permission
    pub fn has_permission(&self, plugin_id: &str, permission: &PluginPermission) -> bool {
        let plugins = self.plugins.lock().unwrap();
        
        if let Some(plugin) = plugins.get(plugin_id) {
            plugin.permissions.contains(permission)
        } else {
            false
        }
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a hook execution
#[derive(Debug, Clone)]
pub struct HookResult {
    /// Plugin ID
    pub plugin_id: String,
    /// Action to take
    pub action: HookAction,
    /// Additional data
    pub data: Option<serde_json::Value>,
}

/// Action for hook result
#[derive(Debug, Clone, PartialEq)]
pub enum HookAction {
    /// Continue normally
    Continue,
    /// Cancel the action
    Cancel,
    /// Redirect to URL
    Redirect(String),
    /// Modify request/response
    Modify(serde_json::Value),
}

/// Plugin manifest structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PluginManifest {
    id: String,
    name: String,
    version: String,
    #[serde(default)]
    permissions: Vec<PluginPermission>,
    #[serde(default)]
    hooks: Vec<PluginHook>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_plugin_manager_creation() {
        let manager = PluginManager::new();
        assert!(manager.get_plugins().is_empty());
    }
    
    #[test]
    fn test_trigger_hook() {
        let manager = PluginManager::new();
        
        let context = PluginContext {
            plugin_id: "test".to_string(),
            tab_id: None,
            url: Some("https://example.com".to_string()),
            data: HashMap::new(),
        };
        
        let results = manager.trigger_hook(&PluginHook::PageLoad, &context).unwrap();
        assert!(results.is_empty());
    }
    
    #[test]
    fn test_has_permission() {
        let manager = PluginManager::new();
        
        // No plugin loaded, so no permission
        assert!(!manager.has_permission("nonexistent", &PluginPermission::Tabs));
    }
}