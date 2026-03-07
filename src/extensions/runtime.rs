// VantisWeb Browser - Advanced Extension System
// Runtime API Implementation (chrome.runtime equivalent)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::extensions::Extension;

/// Runtime API for extensions - implements chrome.runtime API
#[derive(Clone, Debug)]
pub struct RuntimeAPI {
    extensions: Arc<RwLock<HashMap<Uuid, Arc<Extension>>>>,
    event_listeners: Arc<RwLock<HashMap<Uuid, Vec<RuntimeEventListener>>>>,
    message_handlers: Arc<RwLock<HashMap<Uuid, MessageHandler>>>,
    installation_state: Arc<RwLock<HashMap<Uuid, InstallationState>>>,
    platform_info: Arc<RwLock<PlatformInfo>>,
}

impl RuntimeAPI {
    /// Create a new Runtime API instance
    pub fn new() -> Self {
        Self {
            extensions: Arc::new(RwLock::new(HashMap::new())),
            event_listeners: Arc::new(RwLock::new(HashMap::new())),
            message_handlers: Arc::new(RwLock::new(HashMap::new())),
            installation_state: Arc::new(RwLock::new(HashMap::new())),
            platform_info: Arc::new(RwLock::new(PlatformInfo::default())),
        }
    }

    /// Register an extension with the runtime
    pub async fn register_extension(&self, extension: Arc<Extension>) -> Result<(), RuntimeError> {
        let extension_id = extension.id;
        let mut extensions = self.extensions.write().await;
        
        // Initialize listener list for this extension
        let mut listeners = self.event_listeners.write().await;
        listeners.entry(extension_id).or_insert_with(Vec::new);
        
        // Initialize message handler
        let mut handlers = self.message_handlers.write().await;
        handlers.entry(extension_id).or_insert_with(MessageHandler::new);
        
        // Set installation state
        let mut state = self.installation_state.write().await;
        state.insert(extension_id, InstallationState::Installed);
        
        extensions.insert(extension_id, extension);
        
        // Emit onInstalled event
        self.emit_on_installed(extension_id, None).await;
        
        Ok(())
    }

    /// Unregister an extension from runtime
    pub async fn unregister_extension(&self, extension_id: Uuid) -> Result<(), RuntimeError> {
        // Emit onSuspend event
        self.emit_on_suspend(extension_id).await;
        
        let mut extensions = self.extensions.write().await;
        extensions.remove(&extension_id);
        
        let mut listeners = self.event_listeners.write().await;
        listeners.remove(&extension_id);
        
        let mut handlers = self.message_handlers.write().await;
        handlers.remove(&extension_id);
        
        let mut state = self.installation_state.write().await;
        state.remove(&extension_id);
        
        Ok(())
    }

    /// Get extension ID by UUID or internal ID
    pub async fn get_extension_id(&self, extension_id: Uuid) -> Result<String, RuntimeError> {
        let extensions = self.extensions.read().await;
        extensions.get(&extension_id)
            .ok_or(RuntimeError::ExtensionNotFound)
            .map(|ext| ext.id.to_string())
    }

    /// Get extension manifest
    pub async fn get_manifest(&self, extension_id: Uuid) -> Result<serde_json::Value, RuntimeError> {
        let extensions = self.extensions.read().await;
        let extension = extensions.get(&extension_id)
            .ok_or(RuntimeError::ExtensionNotFound)?;
        
        Ok(serde_json::to_value(&extension.manifest)?)
    }

    /// Get URL for a resource in the extension
    pub async fn get_url(&self, extension_id: Uuid, path: &str) -> Result<String, RuntimeError> {
        let extensions = self.extensions.read().await;
        let extension = extensions.get(&extension_id)
            .ok_or(RuntimeError::ExtensionNotFound)?;
        
        Ok(format!("vantis-extension://{}/{}", extension_id, path.trim_start_matches('/')))
    }

    /// Send a message to another extension or context
    pub async fn send_message(
        &self,
        sender_id: Uuid,
        target_id: Uuid,
        message: RuntimeMessage,
    ) -> Result<MessageResponse, RuntimeError> {
        let handlers = self.message_handlers.read().await;
        let handler = handlers.get(&target_id)
            .ok_or(RuntimeError::ExtensionNotFound)?;
        
        handler.handle_message(sender_id, message).await
    }

    /// Send a message to all extensions
    pub async fn broadcast_message(
        &self,
        sender_id: Uuid,
        message: RuntimeMessage,
    ) -> Vec<Result<MessageResponse, RuntimeError>> {
        let extensions = self.extensions.read().await;
        let mut results = Vec::new();
        
        for (ext_id, _) in extensions.iter() {
            if *ext_id != sender_id {
                results.push(self.send_message(sender_id, *ext_id, message.clone()).await);
            }
        }
        
        results
    }

    /// Register an event listener for runtime events
    pub async fn add_listener(&self, extension_id: Uuid, listener: RuntimeEventListener) {
        let mut listeners = self.event_listeners.write().await;
        listeners.entry(extension_id)
            .or_insert_with(Vec::new)
            .push(listener);
    }

    /// Remove an event listener
    pub async fn remove_listener(&self, extension_id: Uuid, listener_id: Uuid) {
        let mut listeners = self.event_listeners.write().await;
        if let Some(ext_listeners) = listeners.get_mut(&extension_id) {
            ext_listeners.retain(|l| l.id != listener_id);
        }
    }

    /// Emit onInstalled event
    pub async fn emit_on_installed(&self, extension_id: Uuid, reason: Option<InstallReason>) {
        let listeners = self.event_listeners.read().await;
        if let Some(ext_listeners) = listeners.get(&extension_id) {
            for listener in ext_listeners {
                if listener.event_type == RuntimeEventType::OnInstalled {
                    let _ = listener.callback(ExtensionEvent::Installed(reason.unwrap_or(InstallReason::Install)));
                }
            }
        }
    }

    /// Emit onSuspend event
    async fn emit_on_suspend(&self, extension_id: Uuid) {
        let listeners = self.event_listeners.read().await;
        if let Some(ext_listeners) = listeners.get(&extension_id) {
            for listener in ext_listeners {
                if listener.event_type == RuntimeEventType::OnSuspend {
                    let _ = listener.callback(ExtensionEvent::Suspend);
                }
            }
        }
    }

    /// Emit onUpdateAvailable event
    pub async fn emit_on_update_available(&self, extension_id: Uuid, version: String) {
        let listeners = self.event_listeners.read().await;
        if let Some(ext_listeners) = listeners.get(&extension_id) {
            for listener in ext_listeners {
                if listener.event_type == RuntimeEventType::OnUpdateAvailable {
                    let _ = listener.callback(ExtensionEvent::UpdateAvailable(version));
                }
            }
        }
    }

    /// Emit onConnect event
    pub async fn emit_on_connect(&self, extension_id: Uuid, port: Port) {
        let listeners = self.event_listeners.read().await;
        if let Some(ext_listeners) = listeners.get(&extension_id) {
            for listener in ext_listeners {
                if listener.event_type == RuntimeEventType::OnConnect {
                    let _ = listener.callback(ExtensionEvent::Connect(port));
                }
            }
        }
    }

    /// Get platform information
    pub async fn get_platform_info(&self) -> PlatformInfo {
        self.platform_info.read().await.clone()
    }

    /// Get platform OS
    pub async fn get_platform_os(&self) -> String {
        self.platform_info.read().await.os.clone()
    }

    /// Get platform architecture
    pub async fn get_platform_arch(&self) -> String {
        self.platform_info.read().await.arch.clone()
    }

    /// Get platform nacl_arch
    pub async fn get_platform_nacl_arch(&self) -> String {
        self.platform_info.read().await.nacl_arch.clone()
    }

    /// Request an extension update check
    pub async fn request_update_check(&self, extension_id: Uuid) -> Result<UpdateCheckResult, RuntimeError> {
        let extensions = self.extensions.read().await;
        extensions.get(&extension_id)
            .ok_or(RuntimeError::ExtensionNotFound)?;
        
        // In a real implementation, this would check for updates from a marketplace
        Ok(UpdateCheckResult {
            status: UpdateStatus::NoUpdate,
            version: None,
        })
    }

    /// Reload an extension
    pub async fn reload(&self, extension_id: Uuid) -> Result<(), RuntimeError> {
        let mut state = self.installation_state.write().await;
        state.entry(extension_id)
            .and_modify(|s| *s = InstallationState::Reloading);
        
        // Emit onSuspend event
        self.emit_on_suspend(extension_id).await;
        
        // Reload logic would go here
        
        // Set state back to installed
        state.entry(extension_id)
            .and_modify(|s| *s = InstallationState::Installed);
        
        Ok(())
    }

    /// Set uninstall URL
    pub async fn set_uninstall_url(&self, extension_id: Uuid, url: String) -> Result<(), RuntimeError> {
        let extensions = self.extensions.read().await;
        extensions.get(&extension_id)
            .ok_or(RuntimeError::ExtensionNotFound)?;
        
        // Store uninstall URL - would need to add to extension state
        Ok(())
    }

    /// Restart extension after delay
    pub async fn restart_after_delay(&self, extension_id: Uuid, seconds: u32) -> Result<(), RuntimeError> {
        let extensions = self.extensions.read().await;
        extensions.get(&extension_id)
            .ok_or(RuntimeError::ExtensionNotFound)?;
        
        tokio::time::sleep(tokio::time::Duration::from_secs(seconds as u64)).await;
        self.reload(extension_id).await
    }

    /// Open extension options page
    pub async fn open_options_page(&self, extension_id: Uuid) -> Result<String, RuntimeError> {
        let extensions = self.extensions.read().await;
        let extension = extensions.get(&extension_id)
            .ok_or(RuntimeError::ExtensionNotFound)?;
        
        if let Some(options_page) = extension.manifest.get("options_page").and_then(|v| v.as_str()) {
            Ok(self.get_url(extension_id, options_page).await?)
        } else if let Some(options_ui) = extension.manifest.get("options_ui") {
            if let Some(page) = options_ui.get("page").and_then(|v| v.as_str()) {
                Ok(self.get_url(extension_id, page).await?)
            } else {
                Err(RuntimeError::OptionsPageNotFound)
            }
        } else {
            Err(RuntimeError::OptionsPageNotFound)
        }
    }

    /// Connect to another extension
    pub async fn connect(&self, sender_id: Uuid, target_id: Uuid, name: Option<String>) -> Result<Port, RuntimeError> {
        let extensions = self.extensions.read().await;
        extensions.get(&target_id)
            .ok_or(RuntimeError::ExtensionNotFound)?;
        
        let port_id = Uuid::new_v4();
        let port = Port {
            id: port_id,
            sender_id,
            target_id,
            name: name.unwrap_or_default(),
            disconnected: false,
        };
        
        // Emit onConnect event
        self.emit_on_connect(target_id, port.clone()).await;
        
        Ok(port)
    }

    /// Get last error for an extension
    pub async fn get_last_error(&self, extension_id: Uuid) -> Option<String> {
        // In a real implementation, this would return the last error
        None
    }

    /// Check if extension can access URL
    pub async fn can_access_url(&self, extension_id: Uuid, url: &str) -> bool {
        let extensions = self.extensions.read().await;
        if let Some(extension) = extensions.get(&extension_id) {
            // Check permissions
            if let Some(permissions) = extension.manifest.get("permissions").and_then(|v| v.as_array()) {
                for perm in permissions {
                    if let Some(perm_str) = perm.as_str() {
                        if perm_str == "<all_urls>" || url.contains(perm_str) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

/// Message handler for runtime messages
struct MessageHandler {
    listeners: Vec<MessageListener>,
}

impl MessageHandler {
    fn new() -> Self {
        Self {
            listeners: Vec::new(),
        }
    }

    async fn handle_message(&self, sender_id: Uuid, message: RuntimeMessage) -> Result<MessageResponse, RuntimeError> {
        for listener in &self.listeners {
            if listener.filter.as_ref().map_or(true, |f| f(&message)) {
                if let Some(response) = (listener.callback)(sender_id, message.clone()).await {
                    return Ok(response);
                }
            }
        }
        Ok(MessageResponse::None)
    }

    fn add_listener(&mut self, filter: Option<MessageFilter>, callback: MessageCallback) {
        self.listeners.push(MessageListener { filter, callback });
    }
}

/// Runtime error types
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("Extension not found")]
    ExtensionNotFound,
    #[error("Invalid message format")]
    InvalidMessageFormat,
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Options page not found")]
    OptionsPageNotFound,
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Extension event types
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeEventType {
    OnInstalled,
    OnSuspended,
    OnSuspend,
    OnUpdateAvailable,
    OnStartup,
    OnConnect,
    OnMessage,
}

/// Extension events
#[derive(Clone, Debug)]
pub enum ExtensionEvent {
    Installed(InstallReason),
    Suspended,
    Suspend,
    UpdateAvailable(String),
    Startup,
    Connect(Port),
    Message(Uuid, RuntimeMessage),
}

/// Installation reason
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallReason {
    Install,
    Update,
    BrowserUpdate,
    SharedModuleUpdate,
}

/// Extension installation state
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstallationState {
    Installing,
    Installed,
    Updating,
    Reloading,
    Uninstalling,
}

/// Platform information
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PlatformInfo {
    pub os: String,
    pub arch: String,
    pub nacl_arch: String,
}

/// Runtime message
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RuntimeMessage {
    String(String),
    Number(i64),
    Boolean(bool),
    Object(serde_json::Value),
    Array(Vec<serde_json::Value>),
}

/// Message response
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageResponse {
    String(String),
    Number(i64),
    Boolean(bool),
    Object(serde_json::Value),
    Array(Vec<serde_json::Value>),
    None,
}

/// Port for extension communication
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Port {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub target_id: Uuid,
    pub name: String,
    pub disconnected: bool,
}

impl Port {
    pub fn disconnect(&mut self) {
        self.disconnected = true;
    }

    pub fn post_message(&self, message: RuntimeMessage) -> Result<(), RuntimeError> {
        if self.disconnected {
            return Err(RuntimeError::ExtensionNotFound);
        }
        // Send message logic
        Ok(())
    }
}

/// Update check result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateCheckResult {
    pub status: UpdateStatus,
    pub version: Option<String>,
}

/// Update status
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateStatus {
    NoUpdate,
    UpdateAvailable,
    Throttled,
    Error,
}

/// Runtime event listener
#[derive(Clone)]
pub struct RuntimeEventListener {
    pub id: Uuid,
    pub event_type: RuntimeEventType,
    pub callback: Arc<dyn Fn(ExtensionEvent) -> Result<(), RuntimeError> + Send + Sync>,
}

/// Message listener
struct MessageListener {
    filter: Option<MessageFilter>,
    callback: MessageCallback,
}

/// Message filter
type MessageFilter = dyn Fn(&RuntimeMessage) -> bool + Send + Sync;

/// Message callback
type MessageCallback = Arc<dyn Fn(Uuid, RuntimeMessage) -> Option<MessageResponse> + Send + Sync>;

/// Create a message filter
pub fn create_message_filter<F>(filter: F) -> Box<MessageFilter>
where
    F: Fn(&RuntimeMessage) -> bool + Send + Sync + 'static,
{
    Box::new(filter)
}

/// Create a message callback
pub fn create_message_callback<F>(callback: F) -> MessageCallback
where
    F: Fn(Uuid, RuntimeMessage) -> Option<MessageResponse> + Send + Sync + 'static,
{
    Arc::new(callback)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extensions::Manifest;

    #[tokio::test]
    async fn test_runtime_api_creation() {
        let runtime = RuntimeAPI::new();
        assert_eq!(runtime.extensions.read().await.len(), 0);
    }

    #[tokio::test]
    async fn test_extension_registration() {
        let runtime = RuntimeAPI::new();
        let extension = Arc::new(Extension {
            id: Uuid::new_v4(),
            name: "Test Extension".to_string(),
            version: "1.0.0".to_string(),
            manifest: serde_json::json!({
                "name": "Test Extension",
                "version": "1.0.0",
                "manifest_version": 3
            }),
            enabled: true,
            path: "/test".to_string(),
        });

        let result = runtime.register_extension(extension.clone()).await;
        assert!(result.is_ok());
        
        let manifest = runtime.get_manifest(extension.id).await;
        assert!(manifest.is_ok());
    }

    #[tokio::test]
    async fn test_get_url() {
        let runtime = RuntimeAPI::new();
        let extension_id = Uuid::new_v4();
        let extension = Arc::new(Extension {
            id: extension_id,
            name: "Test Extension".to_string(),
            version: "1.0.0".to_string(),
            manifest: serde_json::json!({
                "name": "Test Extension",
                "version": "1.0.0",
                "manifest_version": 3
            }),
            enabled: true,
            path: "/test".to_string(),
        });

        runtime.register_extension(extension).await.unwrap();
        
        let url = runtime.get_url(extension_id, "/test.html").await;
        assert!(url.is_ok());
        assert!(url.unwrap().starts_with("vantis-extension://"));
    }

    #[tokio::test]
    async fn test_send_message() {
        let runtime = RuntimeAPI::new();
        let sender_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();
        
        let sender = Arc::new(Extension {
            id: sender_id,
            name: "Sender".to_string(),
            version: "1.0.0".to_string(),
            manifest: serde_json::json!({"name": "Sender", "version": "1.0.0", "manifest_version": 3}),
            enabled: true,
            path: "/sender".to_string(),
        });
        
        let target = Arc::new(Extension {
            id: target_id,
            name: "Target".to_string(),
            version: "1.0.0".to_string(),
            manifest: serde_json::json!({"name": "Target", "version": "1.0.0", "manifest_version": 3}),
            enabled: true,
            path: "/target".to_string(),
        });

        runtime.register_extension(sender).await.unwrap();
        runtime.register_extension(target).await.unwrap();
        
        let message = RuntimeMessage::String("Hello".to_string());
        let result = runtime.send_message(sender_id, target_id, message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_event_listeners() {
        let runtime = RuntimeAPI::new();
        let extension_id = Uuid::new_v4();
        let extension = Arc::new(Extension {
            id: extension_id,
            name: "Test Extension".to_string(),
            version: "1.0.0".to_string(),
            manifest: serde_json::json!({"name": "Test", "version": "1.0.0", "manifest_version": 3}),
            enabled: true,
            path: "/test".to_string(),
        });

        runtime.register_extension(extension).await.unwrap();
        
        let listener = RuntimeEventListener {
            id: Uuid::new_v4(),
            event_type: RuntimeEventType::OnInstalled,
            callback: Arc::new(|_| Ok(())),
        };
        
        runtime.add_listener(extension_id, listener).await;
        
        let listeners = runtime.event_listeners.read().await;
        assert!(listeners.get(&extension_id).is_some());
        assert_eq!(listeners.get(&extension_id).unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_platform_info() {
        let runtime = RuntimeAPI::new();
        let platform = runtime.get_platform_info().await;
        assert!(!platform.os.is_empty());
        assert!(!platform.arch.is_empty());
    }

    #[tokio::test]
    async fn test_port_communication() {
        let sender_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();
        
        let mut port = Port {
            id: Uuid::new_v4(),
            sender_id,
            target_id,
            name: "test_port".to_string(),
            disconnected: false,
        };
        
        let message = RuntimeMessage::String("Test".to_string());
        let result = port.post_message(message);
        assert!(result.is_ok());
        
        port.disconnect();
        let result = port.post_message(message);
        assert!(result.is_err());
    }
}