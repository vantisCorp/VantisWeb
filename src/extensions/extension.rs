//! Extension Core Types
//!
//! Defines the Extension trait and core types for the extensions system.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Extension type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExtensionType {
    /// Content script that runs in web pages
    ContentScript,
    /// Background script that runs in the background
    BackgroundScript,
    /// Popup UI extension
    Popup,
    /// Theme extension
    Theme,
    /// Custom extension type
    Custom(String),
}

/// Extension state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExtensionState {
    /// Extension is installed but not enabled
    Installed,
    /// Extension is enabled and running
    Enabled,
    /// Extension is disabled
    Disabled,
    /// Extension has an error
    Error(String),
}

/// Extension information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionInfo {
    /// Unique extension ID
    pub id: String,
    /// Extension name
    pub name: String,
    /// Extension version
    pub version: String,
    /// Extension description
    pub description: String,
    /// Extension author
    pub author: String,
    /// Extension type
    pub extension_type: ExtensionType,
    /// Extension state
    pub state: ExtensionState,
    /// Extension permissions
    pub permissions: Vec<String>,
    /// Extension icons
    pub icons: HashMap<String, String>,
    /// Extension homepage URL
    pub homepage_url: Option<String>,
}

/// Extension trait
///
/// All extensions must implement this trait to be loaded and managed.
pub trait Extension: Send + Sync {
    /// Returns the extension information
    fn info(&self) -> &ExtensionInfo;

    /// Called when the extension is loaded
    fn on_load(&mut self) -> Result<()>;

    /// Called when the extension is enabled
    fn on_enable(&mut self) -> Result<()>;

    /// Called when the extension is disabled
    fn on_disable(&mut self) -> Result<()>;

    /// Called when the extension is unloaded
    fn on_unload(&mut self) -> Result<()>;

    /// Called when a message is sent to the extension
    fn on_message(&mut self, message: &str) -> Result<String>;

    /// Returns the extension's background script (if any)
    fn background_script(&self) -> Option<&str> {
        None
    }

    /// Returns the extension's content scripts (if any)
    fn content_scripts(&self) -> Vec<&str> {
        Vec::new()
    }

    /// Returns the extension's popup HTML (if any)
    fn popup_html(&self) -> Option<&str> {
        None
    }
}

/// Extension context
///
/// Provides APIs and utilities for extensions to interact with the browser.
#[derive(Clone)]
pub struct ExtensionContext {
    /// Extension ID
    pub extension_id: String,
    /// Extension storage
    pub storage: ExtensionStorage,
    /// Extension messaging
    pub messaging: ExtensionMessaging,
}

/// Extension storage
///
/// Provides storage API for extensions.
#[derive(Clone)]
pub struct ExtensionStorage {
    /// Extension ID
    pub extension_id: String,
}

impl ExtensionStorage {
    /// Creates a new extension storage
    pub fn new(extension_id: String) -> Self {
        Self { extension_id }
    }

    /// Gets a value from storage
    pub fn get(&self, key: &str) -> Result<Option<String>> {
        // TODO: Implement storage retrieval
        Ok(None)
    }

    /// Sets a value in storage
    pub fn set(&self, key: &str, value: &str) -> Result<()> {
        // TODO: Implement storage setting
        Ok(())
    }

    /// Removes a value from storage
    pub fn remove(&self, key: &str) -> Result<()> {
        // TODO: Implement storage removal
        Ok(())
    }

    /// Clears all storage
    pub fn clear(&self) -> Result<()> {
        // TODO: Implement storage clearing
        Ok(())
    }
}

/// Extension messaging
///
/// Provides messaging API for extensions.
#[derive(Clone)]
pub struct ExtensionMessaging {
    /// Extension ID
    pub extension_id: String,
}

impl ExtensionMessaging {
    /// Creates a new extension messaging
    pub fn new(extension_id: String) -> Self {
        Self { extension_id }
    }

    /// Sends a message to another extension
    pub fn send_message(&self, target_extension_id: &str, message: &str) -> Result<String> {
        // TODO: Implement message sending
        Ok(String::new())
    }

    /// Broadcasts a message to all extensions
    pub fn broadcast(&self, message: &str) -> Result<()> {
        // TODO: Implement message broadcasting
        Ok(())
    }

    /// Listens for messages from other extensions
    pub fn on_message<F>(&self, callback: F)
    where
        F: Fn(String, String) + Send + Sync + 'static,
    {
        // TODO: Implement message listening
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_info_creation() {
        let info = ExtensionInfo {
            id: "test-extension".to_string(),
            name: "Test Extension".to_string(),
            version: "1.0.0".to_string(),
            description: "A test extension".to_string(),
            author: "Test Author".to_string(),
            extension_type: ExtensionType::ContentScript,
            state: ExtensionState::Installed,
            permissions: vec!["storage".to_string()],
            icons: HashMap::new(),
            homepage_url: None,
        };

        assert_eq!(info.id, "test-extension");
        assert_eq!(info.name, "Test Extension");
        assert_eq!(info.version, "1.0.0");
    }

    #[test]
    fn test_extension_state_transitions() {
        let mut state = ExtensionState::Installed;
        assert_eq!(state, ExtensionState::Installed);

        state = ExtensionState::Enabled;
        assert_eq!(state, ExtensionState::Enabled);

        state = ExtensionState::Disabled;
        assert_eq!(state, ExtensionState::Disabled);
    }

    #[test]
    fn test_extension_storage() {
        let storage = ExtensionStorage::new("test-extension".to_string());
        assert_eq!(storage.extension_id, "test-extension");
    }

    #[test]
    fn test_extension_messaging() {
        let messaging = ExtensionMessaging::new("test-extension".to_string());
        assert_eq!(messaging.extension_id, "test-extension");
    }
}