//! Messaging API
//!
//! Provides messaging APIs for extensions to communicate with each other and the browser.

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Messaging API
pub struct MessagingAPI {
    /// Extension ID
    extension_id: String,
}

impl MessagingAPI {
    /// Creates a new messaging API
    pub fn new(extension_id: String) -> Self {
        Self { extension_id }
    }

    /// Sends a message to another extension
    pub fn send_message(&self, target_extension_id: &str, message: Message) -> Result<MessageResponse> {
        // TODO: Implement message sending
        Ok(MessageResponse {
            success: true,
            data: None,
        })
    }

    /// Broadcasts a message to all extensions
    pub fn broadcast(&self, message: Message) -> Result<()> {
        // TODO: Implement message broadcasting
        Ok(())
    }

    /// Listens for messages from other extensions
    pub fn on_message<F>(&self, callback: F)
    where
        F: Fn(String, Message) + Send + Sync + 'static,
    {
        // TODO: Implement message listening
    }

    /// Sends a message to the browser
    pub fn send_to_browser(&self, message: Message) -> Result<MessageResponse> {
        // TODO: Implement sending to browser
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
        // TODO: Implement browser message listening
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
}

impl Message {
    /// Creates a new message
    pub fn new(message_type: &str) -> Self {
        Self {
            message_type: message_type.to_string(),
            data: None,
            id: None,
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
}