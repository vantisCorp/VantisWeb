//! # WebRTC Module
//!
//! Provides WebRTC (Web Real-Time Communication) support for peer-to-peer
//! audio, video, and data communication in the browser. This module implements
//! the W3C WebRTC specification with full support for media streams, data
//! channels, and screen sharing.
//!
//! ## Features
//!
//! - **Peer Connections**: Full RTCPeerConnection implementation
//! - **Media Streams**: Support for audio and video capture
//! - **Data Channels**: Reliable and unreliable data transport
//! - **Screen Sharing**: GetDisplayMedia support
//! - **STUN/TURN**: ICE server configuration
//! - **Signaling**: Flexible signaling channel interface
//! - **Statistics**: Connection quality monitoring
//!
//! ## Example Usage
//!
//! ```rust
//! use vantisweb::webrtc::{WebRTCManager, RTCConfiguration};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = RTCConfiguration::default();
//!     let manager = WebRTCManager::new(config);
//!
//!     // Create peer connection
//!     let peer_id = "remote-peer".to_string();
//!     let connection = manager.create_peer_connection(&peer_id).await?;
//!
//!     // Add media tracks
//!     connection.add_audio_track().await?;
//!     connection.add_video_track().await?;
//!
//!     // Create offer
//!     let offer = connection.create_offer().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Security Considerations
//!
//! - Always validate signaling data before processing
//! - Use TURN servers with authentication for production
//! - Implement proper permission handling for media access
//! - Monitor connection quality for security anomalies
//! - Encrypt all data channels using DTLS-SRTP

pub mod connection;
pub mod data_channel;
pub mod ice;
pub mod media;
pub mod signaling;
pub mod stats;

#[cfg(test)]
mod tests;

pub use connection::{RTCPeerConnection, PeerConnectionState};
pub use data_channel::{RTCDataChannel, DataChannelState};
pub use ice::{ICEServer, RTCIceCandidate};
pub use media::{MediaStream, MediaTrack, MediaConstraints};
pub use signaling::{SignalingChannel, SignalingMessage};
pub use stats::{RTCStats, ConnectionQuality};

use anyhow::{Result, Error};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// WebRTC manager for coordinating peer connections
///
/// The `WebRTCManager` provides a high-level interface for managing
/// multiple peer connections, media streams, and signaling.
///
/// # Examples
///
/// ```rust
/// use vantisweb::webrtc::{WebRTCManager, RTCConfiguration};
///
/// let config = RTCConfiguration::default();
/// let manager = WebRTCManager::new(config);
/// ```
///
/// # Thread Safety
///
/// The manager is thread-safe and can be shared across multiple tasks
/// through `Arc<WebRTCManager>`.
pub struct WebRTCManager {
    config: RTCConfiguration,
    peer_connections: Arc<RwLock<HashMap<String, Arc<RTCPeerConnection>>>>,
}

/// WebRTC configuration
///
/// Configuration options for WebRTC connections including ICE servers
/// and media constraints.
///
/// # Examples
///
/// ```rust
/// use vantisweb::webrtc::{RTCConfiguration, ICEServer};
///
/// let config = RTCConfiguration {
///     ice_servers: vec![
//!         ICEServer {
//!             urls: vec!["stun:stun.l.google.com:19302".to_string()],
//!             username: None,
//!             credential: None,
//!         }
//!     ],
//!     ..Default::default()
//! };
/// ```
#[derive(Debug, Clone)]
pub struct RTCConfiguration {
    /// List of ICE servers (STUN/TURN)
    pub ice_servers: Vec<ICEServer>,
    /// ICE transport policy
    pub ice_transport_policy: ICETransportPolicy,
    /// Bundle policy
    pub bundle_policy: BundlePolicy,
    /// RTCP mux policy
    pub rtcp_mux_policy: RTCPMuxPolicy,
    /// Enable IPv6
    pub enable_ipv6: bool,
}

impl Default for RTCConfiguration {
    fn default() -> Self {
        Self {
            ice_servers: vec![
                ICEServer {
                    urls: vec!["stun:stun.l.google.com:19302".to_string()],
                    username: None,
                    credential: None,
                }
            ],
            ice_transport_policy: ICETransportPolicy::All,
            bundle_policy: BundlePolicy::Balanced,
            rtcp_mux_policy: RTCPMuxPolicy::Require,
            enable_ipv6: true,
        }
    }
}

/// ICE transport policy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ICETransportPolicy {
    /// Use all available transports
    All,
    /// Only use relay
    Relay,
}

/// Bundle policy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BundlePolicy {
    /// Bundle if possible
    Balanced,
    /// Always bundle
    MaxBundle,
    /// Never bundle
    MaxCompat,
}

/// RTCP mux policy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RTCPMuxPolicy {
    /// Require RTCP muxing
    Require,
    /// Negotiate RTCP muxing
    Negotiate,
}

impl WebRTCManager {
    /// Create a new WebRTC manager
    ///
    /// # Arguments
    ///
    /// * `config` - WebRTC configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantisweb::webrtc::{WebRTCManager, RTCConfiguration};
    ///
    /// let config = RTCConfiguration::default();
    /// let manager = WebRTCManager::new(config);
    /// ```
    pub fn new(config: RTCConfiguration) -> Self {
        Self {
            config,
            peer_connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new peer connection
    ///
    /// Creates and initializes a new peer connection with the specified peer ID.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - Unique identifier for the remote peer
    ///
    /// # Returns
    ///
    /// Arc to the created peer connection
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantisweb::webrtc::{WebRTCManager, RTCConfiguration};
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let manager = WebRTCManager::new(RTCConfiguration::default());
    ///     let connection = manager.create_peer_connection("peer-1").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn create_peer_connection(&self, peer_id: &str) -> Result<Arc<RTCPeerConnection>> {
        let connection = Arc::new(RTCPeerConnection::new(peer_id.to_string(), self.config.clone())?);
        
        let mut connections = self.peer_connections.write().await;
        connections.insert(peer_id.to_string(), connection.clone());
        
        Ok(connection)
    }

    /// Get a peer connection by ID
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The peer ID to look up
    ///
    /// # Returns
    ///
    /// Option with the peer connection if found
    pub async fn get_peer_connection(&self, peer_id: &str) -> Option<Arc<RTCPeerConnection>> {
        let connections = self.peer_connections.read().await;
        connections.get(peer_id).cloned()
    }

    /// Close a peer connection
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The peer ID to close
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantisweb::webrtc::{WebRTCManager, RTCConfiguration};
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let manager = WebRTCManager::new(RTCConfiguration::default());
    ///     manager.create_peer_connection("peer-1").await?;
    ///     manager.close_peer_connection("peer-1").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn close_peer_connection(&self, peer_id: &str) -> Result<()> {
        let mut connections = self.peer_connections.write().await;
        if let Some(connection) = connections.remove(peer_id) {
            connection.close().await?;
        }
        Ok(())
    }

    /// Get all active peer connections
    ///
    /// # Returns
    ///
    /// Vector of all active peer connections
    pub async fn get_all_connections(&self) -> Vec<Arc<RTCPeerConnection>> {
        let connections = self.peer_connections.read().await;
        connections.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = RTCConfiguration::default();
        assert!(!config.ice_servers.is_empty());
    }

    #[tokio::test]
    async fn test_manager_create_connection() {
        let manager = WebRTCManager::new(RTCConfiguration::default());
        let connection = manager.create_peer_connection("test-peer").await;
        assert!(connection.is_ok());
    }

    #[tokio::test]
    async fn test_manager_get_connection() {
        let manager = WebRTCManager::new(RTCConfiguration::default());
        let peer_id = "test-peer";
        
        manager.create_peer_connection(peer_id).await.unwrap();
        let connection = manager.get_peer_connection(peer_id).await;
        
        assert!(connection.is_some());
    }

    #[tokio::test]
    async fn test_manager_close_connection() {
        let manager = WebRTCManager::new(RTCConfiguration::default());
        let peer_id = "test-peer";
        
        manager.create_peer_connection(peer_id).await.unwrap();
        let result = manager.close_peer_connection(peer_id).await;
        
        assert!(result.is_ok());
        assert!(manager.get_peer_connection(peer_id).await.is_none());
    }
}