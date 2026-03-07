/// # WebRTC ICE (Interactive Connectivity Establishment) Module
/// 
/// This module implements ICE candidate handling and server configuration
/// for NAT traversal in WebRTC connections.
/// 
/// ## Features
/// - STUN/TURN server configuration
/// - ICE candidate gathering
/// - ICE candidate filtering
/// - Connection establishment
/// - IPv4/IPv6 support

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur in ICE operations
#[derive(Error, Debug)]
pub enum ICEError {
    #[error("Candidate gathering failed: {0}")]
    GatheringFailed(String),
    #[error("Invalid candidate format: {0}")]
    InvalidCandidate(String),
    #[error("No ICE candidates available")]
    NoCandidates,
    #[error("ICE server configuration error: {0}")]
    ServerConfigError(String),
    #[error("NAT traversal failed: {0}")]
    NATTraversalFailed(String),
}

/// ICE candidate component type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RTCIceComponent {
    /// RTP component (media)
    Rtp = 1,
    /// RTCP component (control)
    Rtcp = 2,
}

/// ICE candidate type (based on how it was gathered)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RTCIceCandidateType {
    /// Host candidate (local IP)
    Host,
    /// Server reflexive candidate (public IP via STUN)
    Srflx,
    /// Peer reflexive candidate (discovered during connectivity checks)
    Prflx,
    /// Relay candidate (TURN server)
    Relay,
}

/// ICE candidate protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RTCIceProtocol {
    /// UDP protocol (default for WebRTC)
    Udp,
    /// TCP protocol
    Tcp,
}

/// ICE candidate TCP type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RTCIceTcpCandidateType {
    /// Active TCP connection
    Active,
    /// Passive TCP connection
    Passive,
    /// Simultaneous open
    So,
}

/// ICE connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RTCIceConnectionState {
    /// No ICE connection
    New,
    /// Checking candidates
    Checking,
    /// Connection established
    Connected,
    /// Connection completed (all candidates checked)
    Completed,
    /// Connection failed
    Failed,
    /// Connection disconnected
    Disconnected,
    /// Connection closed
    Closed,
}

/// ICE gathering state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RTCIceGatheringState {
    /// Not gathering
    New,
    /// Gathering candidates
    Gathering,
    /// Gathering complete
    Complete,
}

/// ICE transport policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RTCIceTransportPolicy {
    /// Use all candidates
    All,
    /// Use only relay candidates (for privacy)
    Relay,
}

/// ICE server configuration (STUN/TURN)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RTCIceServer {
    /// Server URLs (stun: or turn: or turns:)
    pub urls: Vec<String>,
    /// Username for TURN servers
    pub username: Option<String>,
    /// Credential for TURN servers
    pub credential: Option<String>,
    /// Credential type
    pub credential_type: Option<String>,
}

impl RTCIceServer {
    /// Create a new ICE server from a URL
    pub fn new(url: String) -> Self {
        Self {
            urls: vec![url],
            username: None,
            credential: None,
            credential_type: None,
        }
    }

    /// Create a STUN server configuration
    pub fn stun_server() -> Self {
        Self::new("stun:stun.l.google.com:19302".to_string())
    }

    /// Create a TURN server configuration
    pub fn turn_server(url: String, username: String, credential: String) -> Self {
        Self {
            urls: vec![url],
            username: Some(username),
            credential: Some(credential),
            credential_type: Some("password".to_string()),
        }
    }
}

/// Represents an ICE candidate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RTCIceCandidate {
    /// Candidate foundation (grouping)
    pub foundation: String,
    /// Component ID (RTP=1, RTCP=2)
    pub component: u16,
    /// Protocol
    pub protocol: RTCIceProtocol,
    /// Priority (higher = better)
    pub priority: u32,
    /// IP address
    pub ip: String,
    /// Port number
    pub port: u16,
    /// Candidate type
    pub candidate_type: RTCIceCandidateType,
    /// Related address (for srflx/relay)
    pub related_address: Option<String>,
    /// Related port (for srflx/relay)
    pub related_port: Option<u16>,
    /// TCP type (for TCP candidates)
    pub tcp_type: Option<RTCIceTcpCandidateType>,
    /// Media stream identification tag
    pub sdp_mid: Option<String>,
    /// Media stream index
    pub sdp_mline_index: Option<u16>,
    /// Username fragment
    pub username_fragment: Option<String>,
}

impl RTCIceCandidate {
    /// Create a new host candidate
    pub fn new_host(ip: String, port: u16, component: u16) -> Self {
        Self {
            foundation: Self::generate_foundation(),
            component,
            protocol: RTCIceProtocol::Udp,
            priority: Self::calculate_priority(RTCIceCandidateType::Host, 0),
            ip,
            port,
            candidate_type: RTCIceCandidateType::Host,
            related_address: None,
            related_port: None,
            tcp_type: None,
            sdp_mid: None,
            sdp_mline_index: None,
            username_fragment: None,
        }
    }

    /// Create a server reflexive candidate
    pub fn new_srflx(ip: String, port: u16, base_ip: String, base_port: u16, component: u16) -> Self {
        Self {
            foundation: Self::generate_foundation(),
            component,
            protocol: RTCIceProtocol::Udp,
            priority: Self::calculate_priority(RTCIceCandidateType::Srflx, 0),
            ip,
            port,
            candidate_type: RTCIceCandidateType::Srflx,
            related_address: Some(base_ip),
            related_port: Some(base_port),
            tcp_type: None,
            sdp_mid: None,
            sdp_mline_index: None,
            username_fragment: None,
        }
    }

    /// Create a relay candidate (TURN)
    pub fn new_relay(ip: String, port: u16, component: u16) -> Self {
        Self {
            foundation: Self::generate_foundation(),
            component,
            protocol: RTCIceProtocol::Udp,
            priority: Self::calculate_priority(RTCIceCandidateType::Relay, 0),
            ip,
            port,
            candidate_type: RTCIceCandidateType::Relay,
            related_address: None,
            related_port: None,
            tcp_type: None,
            sdp_mid: None,
            sdp_mline_index: None,
            username_fragment: None,
        }
    }

    /// Generate a random foundation string
    fn generate_foundation() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("f{}", timestamp % 1_000_000_000)
    }

    /// Calculate candidate priority according to RFC 5245
    fn calculate_priority(candidate_type: RTCIceCandidateType, local_preference: u16) -> u32 {
        let type_preference = match candidate_type {
            RTCIceCandidateType::Host => 126,
            RTCIceCandidateType::Srflx => 100,
            RTCIceCandidateType::Prflx => 110,
            RTCIceCandidateType::Relay => 0,
        };
        
        (type_preference << 24) as u32
            | ((local_preference as u32) << 8)
            | ((256 - 1) as u32)
    }

    /// Convert candidate to SDP format
    pub fn to_sdp(&self) -> String {
        let type_str = match self.candidate_type {
            RTCIceCandidateType::Host => "host",
            RTCIceCandidateType::Srflx => "srflx",
            RTCIceCandidateType::Prflx => "prflx",
            RTCIceCandidateType::Relay => "relay",
        };

        let protocol_str = match self.protocol {
            RTCIceProtocol::Udp => "UDP",
            RTCIceProtocol::Tcp => "TCP",
        };

        let mut candidate = format!(
            "candidate:{} {} {} {} {} {} typ {}",
            self.foundation,
            self.component,
            protocol_str,
            self.priority,
            self.ip,
            self.port,
            type_str
        );

        if let (Some(rel_addr), Some(rel_port)) = (&self.related_address, self.related_port) {
            candidate.push_str(&format!(" raddr {} rport {}", rel_addr, rel_port));
        }

        if let Some(tcp_type) = &self.tcp_type {
            let tcp_str = match tcp_type {
                RTCIceTcpCandidateType::Active => "active",
                RTCIceTcpCandidateType::Passive => "passive",
                RTCIceTcpCandidateType::So => "so",
            };
            candidate.push_str(&format!(" tcptype {}", tcp_str));
        }

        candidate
    }
}

/// ICE candidate pair for connectivity checks
#[derive(Debug, Clone)]
pub struct RTCIceCandidatePair {
    /// Local candidate
    pub local: RTCIceCandidate,
    /// Remote candidate
    pub remote: RTCIceCandidate,
    /// Pair priority
    pub priority: u64,
    /// Pair state
    pub state: IceCandidatePairState,
}

/// ICE candidate pair state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IceCandidatePairState {
    /// Waiting for connectivity check
    Waiting,
    /// In progress
    InProgress,
    /// Successfully connected
    Succeeded,
    /// Failed connectivity check
    Failed,
}

/// ICE agent for candidate gathering and connection management
pub struct IceAgent {
    /// ICE configuration
    config: IceConfig,
    /// Local candidates
    local_candidates: Arc<RwLock<Vec<RTCIceCandidate>>>,
    /// Remote candidates
    remote_candidates: Arc<RwLock<Vec<RTCIceCandidate>>>,
    /// ICE connection state
    connection_state: Arc<RwLock<RTCIceConnectionState>>,
    /// ICE gathering state
    gathering_state: Arc<RwLock<RTCIceGatheringState>>,
}

/// ICE agent configuration
#[derive(Debug, Clone)]
pub struct IceConfig {
    /// ICE servers
    pub ice_servers: Vec<RTCIceServer>,
    /// ICE transport policy
    pub ice_transport_policy: RTCIceTransportPolicy,
    /// Enable IPv6
    pub enable_ipv6: bool,
    /// Enable UDP
    pub enable_udp: bool,
    /// Enable TCP
    pub enable_tcp: bool,
}

impl Default for IceConfig {
    fn default() -> Self {
        Self {
            ice_servers: vec![RTCIceServer::stun_server()],
            ice_transport_policy: RTCIceTransportPolicy::All,
            enable_ipv6: true,
            enable_udp: true,
            enable_tcp: true,
        }
    }
}

impl IceAgent {
    /// Create a new ICE agent
    pub fn new(config: IceConfig) -> Self {
        Self {
            config,
            local_candidates: Arc::new(RwLock::new(Vec::new())),
            remote_candidates: Arc::new(RwLock::new(Vec::new())),
            connection_state: Arc::new(RwLock::new(RTCIceConnectionState::New)),
            gathering_state: Arc::new(RwLock::new(RTCIceGatheringState::New)),
        }
    }

    /// Start gathering local candidates
    pub async fn gather_candidates(&self) -> Result<(), ICEError> {
        *self.gathering_state.write().await = RTCIceGatheringState::Gathering;
        
        // In a real implementation, this would:
        // 1. Get local IP addresses (host candidates)
        // 2. Query STUN servers (srflx candidates)
        // 3. Allocate TURN relays (relay candidates)
        
        // For demonstration, add a mock host candidate
        let host_candidate = RTCIceCandidate::new_host("192.168.1.1".to_string(), 50000, 1);
        self.local_candidates.write().await.push(host_candidate);
        
        *self.gathering_state.write().await = RTCIceGatheringState::Complete;
        
        Ok(())
    }

    /// Add a remote candidate
    pub async fn add_remote_candidate(&self, candidate: RTCIceCandidate) {
        self.remote_candidates.write().await.push(candidate);
    }

    /// Get all local candidates
    pub async fn get_local_candidates(&self) -> Vec<RTCIceCandidate> {
        self.local_candidates.read().await.clone()
    }

    /// Get all remote candidates
    pub async fn get_remote_candidates(&self) -> Vec<RTCIceCandidate> {
        self.remote_candidates.read().await.clone()
    }

    /// Get ICE connection state
    pub async fn connection_state(&self) -> RTCIceConnectionState {
        *self.connection_state.read().await
    }

    /// Get ICE gathering state
    pub async fn gathering_state(&self) -> RTCIceGatheringState {
        *self.gathering_state.read().await
    }

    /// Start connectivity checks
    pub async fn start_connectivity_checks(&self) {
        *self.connection_state.write().await = RTCIceConnectionState::Checking;
        
        // In a real implementation, this would:
        // 1. Form candidate pairs
        // 2. Sort by priority
        // 3. Send STUN binding requests
        // 4. Update state based on responses
        
        // For demonstration, simulate successful connection
        *self.connection_state.write().await = RTCIceConnectionState::Connected;
    }

    /// Close the ICE agent
    pub async fn close(&self) {
        *self.connection_state.write().await = RTCIceConnectionState::Closed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ice_server_creation() {
        let stun = RTCIceServer::stun_server();
        assert!(stun.urls[0].starts_with("stun:"));
        assert!(stun.username.is_none());
    }

    #[test]
    fn test_candidate_creation() {
        let candidate = RTCIceCandidate::new_host("192.168.1.1".to_string(), 50000, 1);
        assert_eq!(candidate.ip, "192.168.1.1");
        assert_eq!(candidate.port, 50000);
        assert_eq!(candidate.candidate_type, RTCIceCandidateType::Host);
    }

    #[test]
    fn test_candidate_to_sdp() {
        let candidate = RTCIceCandidate::new_host("192.168.1.1".to_string(), 50000, 1);
        let sdp = candidate.to_sdp();
        assert!(sdp.contains("host"));
        assert!(sdp.contains("192.168.1.1"));
    }

    #[tokio::test]
    async fn test_ice_agent_gathering() {
        let config = IceConfig::default();
        let agent = IceAgent::new(config);
        
        let result = agent.gather_candidates().await;
        assert!(result.is_ok());
        assert_eq!(agent.gathering_state().await, RTCIceGatheringState::Complete);
    }
}