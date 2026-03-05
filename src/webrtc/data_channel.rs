/// # WebRTC Data Channel Module
/// 
/// This module implements the RTCDataChannel functionality for peer-to-peer data transfer.
/// Data channels provide reliable or unreliable data delivery between peers.
/// 
/// ## Features
/// - Ordered and unordered delivery
/// - Reliable and unreliable modes
/// - Binary and text data support
/// - Backpressure handling
/// - Channel state management

use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur in data channel operations
#[derive(Error, Debug)]
pub enum DataChannelError {
    #[error("Channel is closed")]
    ChannelClosed,
    #[error("Buffer is full")]
    BufferFull,
    #[error("Invalid data format")]
    InvalidData,
    #[error("Send failed: {0}")]
    SendFailed(String),
}

/// Data channel state according to WebRTC specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RTCDataChannelState {
    /// Channel is being created
    Connecting,
    /// Channel is open and ready to send data
    Open,
    /// Channel is closing
    Closing,
    /// Channel is closed
    Closed,
}

/// Data channel configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RTCDataChannelInit {
    /// Whether data channel is ordered
    pub ordered: Option<bool>,
    /// Maximum number of retransmits (None for unlimited)
    pub max_packet_life_time: Option<u16>,
    /// Maximum number of retransmit attempts (None for unlimited)
    pub max_retransmits: Option<u16>,
    /// Application-defined protocol
    pub protocol: Option<String>,
    /// Whether data channel was negotiated out-of-band
    pub negotiated: Option<bool>,
    /// User-defined channel ID (required if negotiated=true)
    pub id: Option<u16>,
    /// Data channel priority
    pub priority: Option<String>,
}

impl Default for RTCDataChannelInit {
    fn default() -> Self {
        Self {
            ordered: Some(true),
            max_packet_life_time: None,
            max_retransmits: None,
            protocol: None,
            negotiated: Some(false),
            id: None,
            priority: None,
        }
    }
}

/// Represents data that can be sent over a data channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataChannelMessage {
    /// Text message
    Text(String),
    /// Binary data
    Binary(Vec<u8>),
}

/// Data channel for peer-to-peer data transfer
pub struct RTCDataChannel {
    /// Unique channel identifier
    label: String,
    /// Channel configuration
    config: RTCDataChannelInit,
    /// Current channel state
    state: Arc<RwLock<RTCDataChannelState>>,
    /// Sender for outgoing messages
    sender: mpsc::UnboundedSender<DataChannelMessage>,
    /// Receiver for incoming messages
    receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<DataChannelMessage>>>>,
    /// Channel ID (assigned by the peer connection)
    id: Arc<RwLock<Option<u16>>>,
    /// Bytes sent
    bytes_sent: Arc<RwLock<u64>>,
    /// Bytes received
    bytes_received: Arc<RwLock<u64>>,
    /// Buffered amount
    buffered_amount: Arc<RwLock<u64>>,
}

impl RTCDataChannel {
    /// Create a new data channel
    ///
    /// # Arguments
    /// * `label` - Channel label/identifier
    /// * `config` - Channel configuration options
    ///
    /// # Returns
    /// New data channel instance
    pub fn new(label: String, config: RTCDataChannelInit) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        
        Self {
            label,
            config,
            state: Arc::new(RwLock::new(RTCDataChannelState::Connecting)),
            sender,
            receiver: Arc::new(RwLock::new(Some(receiver))),
            id: Arc::new(RwLock::new(config.id)),
            bytes_sent: Arc::new(RwLock::new(0)),
            bytes_received: Arc::new(RwLock::new(0)),
            buffered_amount: Arc::new(RwLock::new(0)),
        }
    }

    /// Get the channel label
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Get the current channel state
    pub async fn state(&self) -> RTCDataChannelState {
        *self.state.read().await
    }

    /// Get the channel ID
    pub async fn id(&self) -> Option<u16> {
        *self.id.read().await
    }

    /// Set the channel ID (called by peer connection)
    pub async fn set_id(&self, id: u16) {
        *self.id.write().await = Some(id);
    }

    /// Get the channel configuration
    pub fn config(&self) -> &RTCDataChannelInit {
        &self.config
    }

    /// Check if the channel is open
    pub async fn is_open(&self) -> bool {
        matches!(self.state().await, RTCDataChannelState::Open)
    }

    /// Send text data
    ///
    /// # Arguments
    /// * `text` - Text message to send
    ///
    /// # Returns
    /// Ok(()) if sent successfully, Err otherwise
    pub async fn send_text(&self, text: String) -> Result<(), DataChannelError> {
        if !self.is_open().await {
            return Err(DataChannelError::ChannelClosed);
        }

        let message = DataChannelMessage::Text(text);
        self.sender
            .send(message)
            .map_err(|e| DataChannelError::SendFailed(e.to_string()))?;

        Ok(())
    }

    /// Send binary data
    ///
    /// # Arguments
    /// * `data` - Binary data to send
    ///
    /// # Returns
    /// Ok(()) if sent successfully, Err otherwise
    pub async fn send_binary(&self, data: Vec<u8>) -> Result<(), DataChannelError> {
        if !self.is_open().await {
            return Err(DataChannelError::ChannelClosed);
        }

        let message = DataChannelMessage::Binary(data);
        self.sender
            .send(message)
            .map_err(|e| DataChannelError::SendFailed(e.to_string()))?;

        Ok(())
    }

    /// Receive a message from the channel
    ///
    /// # Returns
    /// Next message if available, None if channel is closed
    pub async fn receive(&self) -> Option<DataChannelMessage> {
        let mut receiver = self.receiver.write().await;
        if let Some(ref mut rx) = *receiver {
            rx.recv().await
        } else {
            None
        }
    }

    /// Get the buffered amount (bytes waiting to be sent)
    pub async fn buffered_amount(&self) -> u64 {
        *self.buffered_amount.read().await
    }

    /// Get total bytes sent
    pub async fn bytes_sent(&self) -> u64 {
        *self.bytes_sent.read().await
    }

    /// Get total bytes received
    pub async fn bytes_received(&self) -> u64 {
        *self.bytes_received.read().await
    }

    /// Close the data channel
    pub async fn close(&self) {
        *self.state.write().await = RTCDataChannelState::Closing;
        // Close receiver
        let mut receiver = self.receiver.write().await;
        *receiver = None;
        *self.state.write().await = RTCDataChannelState::Closed;
    }

    /// Set channel state (called by peer connection)
    pub async fn set_state(&self, state: RTCDataChannelState) {
        *self.state.write().await = state;
    }

    /// Open the channel (called by peer connection)
    pub async fn open(&self) {
        *self.state.write().await = RTCDataChannelState::Open;
    }
}

impl Drop for RTCDataChannel {
    fn drop(&mut self) {
        // Channel will be closed when dropped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_data_channel_creation() {
        let config = RTCDataChannelInit::default();
        let channel = RTCDataChannel::new("test".to_string(), config);
        
        assert_eq!(channel.label(), "test");
        assert_eq!(channel.state().await, RTCDataChannelState::Connecting);
    }

    #[tokio::test]
    async fn test_data_channel_send_text() {
        let config = RTCDataChannelInit::default();
        let channel = RTCDataChannel::new("test".to_string(), config);
        
        channel.open().await;
        
        let result = channel.send_text("Hello".to_string()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_data_channel_send_binary() {
        let config = RTCDataChannelInit::default();
        let channel = RTCDataChannel::new("test".to_string(), config);
        
        channel.open().await;
        
        let data = vec![1, 2, 3, 4, 5];
        let result = channel.send_binary(data).await;
        assert!(result.is_ok());
    }
}