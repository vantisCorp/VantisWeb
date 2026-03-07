//! Messaging API
//!
//! Provides messaging APIs for extensions to communicate with each other and the browser.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Global message bus for all extensions
static mut MESSAGE_BUS: Option<Arc<RwLock<MessageBus>>> = None;

fn get_message_bus() -> Arc<RwLock<MessageBus>> {
    unsafe {
        if MESSAGE_BUS.is_none() {
            MESSAGE_BUS = Some(Arc::new(RwLock::new(MessageBus::new())));
        }
        MESSAGE_BUS.clone().unwrap()
    }
}

/// Message bus for routing messages between extensions
struct MessageBus {
    /// Pending messages per extension
    pending_messages: HashMap<String, Vec<(String, Message)>>,
    /// Message listeners per extension
    listeners: HashMap<String, Vec<Box<dyn Fn(String, Message) + Send + Sync>>>,
    /// Browser message listeners per extension
    browser_listeners: HashMap<String, Vec<Box<dyn Fn(Message) + Send + Sync>>>,
}

impl MessageBus {
    fn new() -> Self {
        Self {
            pending_messages: HashMap::new(),
            listeners: HashMap::new(),
            browser_listeners: HashMap::new(),
        }
    }
}

/// Messaging API
pub struct MessagingAPI {
    /// Extension ID
    extension_id: String,
}

impl MessagingAPI {
    /// Creates a new messaging API
    pub fn new(extension_id: String) -> Self {
        // Initialize message bus for this extension
        let bus = get_message_bus();
        let mut guard = bus.write().unwrap();
        guard.pending_messages.entry(extension_id.clone()).or_insert_with(Vec::new);
        guard.listeners.entry(extension_id.clone()).or_insert_with(Vec::new);
        guard.browser_listeners.entry(extension_id.clone()).or_insert_with(Vec::new);
        
        Self { extension_id }
    }

    /// Sends a message to another extension
    pub fn send_message(&self, target_extension_id: &str, message: Message) -> Result<MessageResponse> {
        let bus = get_message_bus();
        let mut guard = bus.write().unwrap();
        
        // Add message to target's pending messages
        guard.pending_messages
            .entry(target_extension_id.to_string())
            .or_insert_with(Vec::new)
            .push((self.extension_id.clone(), message.clone()));
        
        // Notify any listeners
        if let Some(listeners) = guard.listeners.get(target_extension_id) {
            for listener in listeners {
                listener(self.extension_id.clone(), message.clone());
            }
        }
        
        Ok(MessageResponse {
            success: true,
            data: None,
        })
    }

    /// Broadcasts a message to all extensions
    pub fn broadcast(&self, message: Message) -> Result<()> {
        let bus = get_message_bus();
        let guard = bus.read().unwrap();
        
        // Get all extension IDs (excluding sender)
        let all_extensions: Vec<String> = guard.pending_messages.keys()
            .filter(|id| *id != &self.extension_id)
            .cloned()
            .collect();
        
        drop(guard);
        
        // Send to each extension
        for ext_id in all_extensions {
            self.send_message(&ext_id, message.clone())?;
        }
        
        Ok(())
    }

    /// Listens for messages from other extensions
    pub fn on_message<F>(&self, callback: F)
    where
        F: Fn(String, Message) + Send + Sync + 'static,
    {
        let bus = get_message_bus();
        let mut guard = bus.write().unwrap();
        
        if let Some(listeners) = guard.listeners.get_mut(&self.extension_id) {
            listeners.push(Box::new(callback));
        }
    }

    /// Sends a message to the browser
    pub fn send_to_browser(&self, message: Message) -> Result<MessageResponse> {
        // In a real implementation, this would communicate with the browser process
        // For now, we just log and return success
        log::debug!("Extension {} sent message to browser: {:?}", self.extension_id, message);
        
        Ok(MessageResponse {
            success: true,
            data: None,
        })
    }

    /// Listens for messages from the browser
    pub fn on_browser_message<F>(&self, callback: F)
    where
        F: Fn(Message) + Send + Sync + 'static,
    {
        let bus = get_message_bus();
        let mut guard = bus.write().unwrap();
        
        if let Some(listeners) = guard.browser_listeners.get_mut(&self.extension_id) {
            listeners.push(Box::new(callback));
        }
    }
    
    /// Get pending messages for this extension
    pub fn get_pending_messages(&self) -> Result<Vec<(String, Message)>> {
        let bus = get_message_bus();
        let mut guard = bus.write().unwrap();
        
        if let Some(messages) = guard.pending_messages.get_mut(&self.extension_id) {
            Ok(std::mem::take(messages))
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Check if there are pending messages
    pub fn has_pending_messages(&self) -> bool {
        let bus = get_message_bus();
        let guard = bus.read().unwrap();
        
        guard.pending_messages
            .get(&self.extension_id)
            .map(|m| !m.is_empty())
            .unwrap_or(false)
    }
}

/// Message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message type
    #[serde(rename = "type")]
    pub message_type: String,
    /// Message data
    pub data: Option<serde_json::Value>,
    /// Message ID
    pub id: Option<String>,
    /// Timestamp
    #[serde(default = "default_timestamp")]
    pub timestamp: i64,
}

fn default_timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}

impl Message {
    /// Creates a new message
    pub fn new(message_type: &str) -> Self {
        Self {
            message_type: message_type.to_string(),
            data: None,
            id: None,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Sets the message data
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    /// Sets the message ID
    pub fn with_id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }
}

/// Message response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResponse {
    /// Whether the message was successful
    pub success: bool,
    /// Response data
    pub data: Option<serde_json::Value>,
}

impl MessageResponse {
    /// Create a successful response
    pub fn success() -> Self {
        Self {
            success: true,
            data: None,
        }
    }
    
    /// Create a successful response with data
    pub fn with_data(data: serde_json::Value) -> Self {
        Self {
            success: true,
            data: Some(data),
        }
    }
    
    /// Create an error response
    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            data: Some(serde_json::json!({ "error": message })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_messaging_api_creation() {
        let api = MessagingAPI::new("test-extension".to_string());
        assert_eq!(api.extension_id, "test-extension");
    }

    #[test]
    fn test_message_creation() {
        let message = Message::new("test-type");
        assert_eq!(message.message_type, "test-type");
        assert!(message.data.is_none());
        assert!(message.id.is_none());
    }

    #[test]
    fn test_message_with_data() {
        let data = serde_json::json!({"key": "value"});
        let message = Message::new("test-type").with_data(data);

        assert!(message.data.is_some());
        assert_eq!(message.data.unwrap(), data);
    }

    #[test]
    fn test_message_with_id() {
        let message = Message::new("test-type").with_id("test-id".to_string());

        assert!(message.id.is_some());
        assert_eq!(message.id.unwrap(), "test-id");
    }
    
    #[test]
    fn test_send_message() {
        let sender = MessagingAPI::new("sender".to_string());
        let receiver = MessagingAPI::new("receiver".to_string());
        
        let message = Message::new("test").with_data(serde_json::json!({"hello": "world"}));
        let response = sender.send_message("receiver", message).unwrap();
        
        assert!(response.success);
        assert!(receiver.has_pending_messages());
        
        let pending = receiver.get_pending_messages().unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].0, "sender");
    }
    
    #[test]
    fn test_broadcast() {
        let sender = MessagingAPI::new("broadcaster".to_string());
        let receiver1 = MessagingAPI::new("receiver1".to_string());
        let receiver2 = MessagingAPI::new("receiver2".to_string());
        
        let message = Message::new("broadcast_test");
        sender.broadcast(message).unwrap();
        
        assert!(receiver1.has_pending_messages());
        assert!(receiver2.has_pending_messages());
    }
}