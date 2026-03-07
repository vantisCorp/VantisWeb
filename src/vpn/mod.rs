//! VPN Module
//!
//! Built-in VPN client with support for multiple protocols:
//! - WireGuard, OpenVPN, IKEv2
//! - Server selection with latency
//! - Kill switch, split tunneling
//! - DNS leak protection

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::collections::HashMap;

pub mod wireguard;
pub mod openvpn;
pub mod ikev2;
pub mod network;
pub mod dns;

use wireguard::WireGuardClient;
use openvpn::OpenVPNClient;
use ikev2::IKEv2Client;
use network::NetworkManager;
use dns::DNSProtection;

/// VPN connection states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VPNState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Disconnecting,
    Error(String),
}

/// VPN protocol types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VPNProtocol {
    WireGuard,
    OpenVPN,
    IKEv2,
}

/// VPN server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VPNServer {
    pub id: String,
    pub name: String,
    pub country: String,
    pub city: String,
    pub hostname: String,
    pub port: u16,
    pub protocol: VPNProtocol,
    pub load: f64, // 0.0 to 1.0
    pub latency_ms: Option<u64>,
    pub features: ServerFeatures,
}

/// Server features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerFeatures {
    pub p2p: bool,
    pub streaming: bool,
    pub double_vpn: bool,
    pub obfuscated: bool,
}

/// VPN configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VPNConfig {
    pub protocol: VPNProtocol,
    pub auto_connect: bool,
    pub kill_switch: bool,
    pub split_tunneling: bool,
    pub split_tunneling_rules: Vec<SplitTunnelRule>,
    pub dns_protection: bool,
    pub custom_dns: Option<String>,
    pub obfuscation: bool,
    pub mtu: Option<u16>,
    pub reconnect_on_drop: bool,
    pub unsafe_network_detection: bool,
}

impl Default for VPNConfig {
    fn default() -> Self {
        Self {
            protocol: VPNProtocol::WireGuard,
            auto_connect: false,
            kill_switch: true,
            split_tunneling: false,
            split_tunneling_rules: vec![],
            dns_protection: true,
            custom_dns: None,
            obfuscation: false,
            mtu: None,
            reconnect_on_drop: true,
            unsafe_network_detection: false,
        }
    }
}

/// Split tunneling rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitTunnelRule {
    pub hostname: String,
    pub bypass_vpn: bool,
    pub description: String,
}

/// VPN statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VPNStats {
    pub connected_since: Option<chrono::DateTime<chrono::Utc>>,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connection_uptime: Duration,
    pub current_server: Option<VPNServer>,
}

/// Main VPN manager
pub struct VPNManager {
    config: Arc<RwLock<VPNConfig>>,
    state: Arc<RwLock<VPNState>>,
    wireguard: Arc<WireGuardClient>,
    openvpn: Arc<OpenVPNClient>,
    ikev2: Arc<IKEv2Client>,
    network: Arc<NetworkManager>,
    dns: Arc<DNSProtection>,
    servers: Arc<RwLock<Vec<VPNServer>>>,
    stats: Arc<RwLock<VPNStats>>,
    current_connection: Arc<RwLock<Option<VPNConnection>>>,
}

/// VPN connection handle
#[derive(Clone)]
struct VPNConnection {
    protocol: VPNProtocol,
    server_id: String,
    interface_name: String,
}

impl VPNManager {
    /// Create a new VPN manager
    pub fn new(config: VPNConfig) -> Self {
        let wireguard = Arc::new(WireGuardClient::new());
        let openvpn = Arc::new(OpenVPNClient::new());
        let ikev2 = Arc::new(IKEv2Client::new());
        let network = Arc::new(NetworkManager::new());
        let dns = Arc::new(DNSProtection::new());

        Self {
            config: Arc::new(RwLock::new(config)),
            state: Arc::new(RwLock::new(VPNState::Disconnected)),
            wireguard,
            openvpn,
            ikev2,
            network,
            dns,
            servers: Arc::new(RwLock::new(vec![])),
            stats: Arc::new(RwLock::new(VPNStats {
                connected_since: None,
                bytes_sent: 0,
                bytes_received: 0,
                connection_uptime: Duration::ZERO,
                current_server: None,
            })),
            current_connection: Arc::new(RwLock::new(None)),
        }
    }

    /// Connect to VPN server
    pub async fn connect(&self, server_id: &str) -> Result<(), VPNError> {
        let mut state = self.state.write().await;
        *state = VPNState::Connecting;
        drop(state);

        let servers = self.servers.read().await;
        let server = servers.iter()
            .find(|s| s.id == server_id)
            .ok_or_else(|| VPNError::ServerNotFound(server_id.to_string()))?;
        drop(servers);

        let config = self.config.read().await;
        let protocol = config.protocol;

        let result = match protocol {
            VPNProtocol::WireGuard => {
                self.wireguard.connect(server).await
            }
            VPNProtocol::OpenVPN => {
                self.openvpn.connect(server).await
            }
            VPNProtocol::IKEv2 => {
                self.ikev2.connect(server).await
            }
        };

        match result {
            Ok(connection) => {
                // Update DNS if enabled
                if config.dns_protection {
                    if let Err(e) = self.dns.protect(config.custom_dns.as_deref()).await {
                        log::warn!("Failed to protect DNS: {:?}", e);
                    }
                }

                // Update network routing
                if config.kill_switch {
                    if let Err(e) = self.network.enable_kill_switch().await {
                        log::warn!("Failed to enable kill switch: {:?}", e);
                    }
                }

                // Update state and stats
                let mut state = self.state.write().await;
                *state = VPNState::Connected;
                drop(state);

                let mut stats = self.stats.write().await;
                stats.connected_since = Some(chrono::Utc::now());
                stats.current_server = Some(server.clone());
                drop(stats);

                let mut conn = self.current_connection.write().await;
                *conn = Some(VPNConnection {
                    protocol,
                    server_id: server_id.to_string(),
                    interface_name: connection.interface_name,
                });

                Ok(())
            }
            Err(e) => {
                let mut state = self.state.write().await;
                *state = VPNState::Error(e.to_string());
                Err(e)
            }
        }
    }

    /// Disconnect from VPN
    pub async fn disconnect(&self) -> Result<(), VPNError> {
        let mut state = self.state.write().await;
        *state = VPNState::Disconnecting;
        drop(state);

        let connection = self.current_connection.write().await.take();
        if let Some(conn) = connection {
            match conn.protocol {
                VPNProtocol::WireGuard => {
                    self.wireguard.disconnect().await?;
                }
                VPNProtocol::OpenVPN => {
                    self.openvpn.disconnect().await?;
                }
                VPNProtocol::IKEv2 => {
                    self.ikev2.disconnect().await?;
                }
            }
        }

        // Disable kill switch
        if let Err(e) = self.network.disable_kill_switch().await {
            log::warn!("Failed to disable kill switch: {:?}", e);
        }

        // Restore DNS
        if let Err(e) = self.dns.restore().await {
            log::warn!("Failed to restore DNS: {:?}", e);
        }

        // Update state and stats
        let mut state = self.state.write().await;
        *state = VPNState::Disconnected;
        drop(state);

        let mut stats = self.stats.write().await;
        stats.connected_since = None;
        stats.current_server = None;

        Ok(())
    }

    /// Get current connection state
    pub async fn state(&self) -> VPNState {
        *self.state.read().await
    }

    /// Get connection statistics
    pub async fn stats(&self) -> VPNStats {
        self.stats.read().await.clone()
    }

    /// Update configuration
    pub async fn update_config(&self, config: VPNConfig) {
        let mut cfg = self.config.write().await;
        *cfg = config;
    }

    /// Get current configuration
    pub async fn config(&self) -> VPNConfig {
        self.config.read().await.clone()
    }

    /// Add server to list
    pub async fn add_server(&self, server: VPNServer) {
        let mut servers = self.servers.write().await;
        servers.push(server);
    }

    /// Get all servers
    pub async fn servers(&self) -> Vec<VPNServer> {
        self.servers.read().await.clone()
    }

    /// Get servers by protocol
    pub async fn servers_by_protocol(&self, protocol: VPNProtocol) -> Vec<VPNServer> {
        let servers = self.servers.read().await;
        servers.iter()
            .filter(|s| s.protocol == protocol)
            .cloned()
            .collect()
    }

    /// Measure latency to servers
    pub async fn measure_latency(&self) -> Result<(), VPNError> {
        let servers = self.servers.read().await;
        for server in servers.iter() {
            let latency = self.network.ping_server(server).await?;
            let mut servers = self.servers.write().await;
            if let Some(s) = servers.iter_mut().find(|s| s.id == server.id) {
                s.latency_ms = Some(latency);
            }
        }
        Ok(())
    }

    /// Get best server (lowest latency)
    pub async fn best_server(&self) -> Option<VPNServer> {
        let servers = self.servers.read().await;
        servers.iter()
            .filter(|s| s.latency_ms.is_some())
            .min_by_key(|s| s.latency_ms.unwrap())
            .cloned()
    }

    /// Add split tunneling rule
    pub async fn add_split_tunnel_rule(&self, rule: SplitTunnelRule) {
        let mut config = self.config.write().await;
        config.split_tunneling_rules.push(rule);
    }

    /// Remove split tunneling rule
    pub async fn remove_split_tunnel_rule(&self, hostname: &str) -> bool {
        let mut config = self.config.write().await;
        let initial_len = config.split_tunneling_rules.len();
        config.split_tunneling_rules.retain(|r| r.hostname != hostname);
        config.split_tunneling_rules.len() < initial_len
    }

    /// Check for DNS leaks
    pub async fn check_dns_leaks(&self) -> Result<bool, VPNError> {
        self.dns.check_leak().await
    }

    /// Test connection
    pub async fn test_connection(&self) -> Result<bool, VPNError> {
        if *self.state.read().await != VPNState::Connected {
            return Ok(false);
        }
        self.network.test_vpn_connection().await
    }
}

/// VPN errors
#[derive(Debug, thiserror::Error)]
pub enum VPNError {
    #[error("Server not found: {0}")]
    ServerNotFound(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Timeout")]
    Timeout,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_vpn_manager_creation() {
        let config = VPNConfig::default();
        let manager = VPNManager::new(config);
        assert_eq!(manager.state().await, VPNState::Disconnected);
    }

    #[tokio::test]
    async fn test_server_addition() {
        let manager = VPNManager::new(VPNConfig::default());
        let server = VPNServer {
            id: "test-server".to_string(),
            name: "Test Server".to_string(),
            country: "US".to_string(),
            city: "New York".to_string(),
            hostname: "test.vpn.com".to_string(),
            port: 443,
            protocol: VPNProtocol::WireGuard,
            load: 0.5,
            latency_ms: None,
            features: ServerFeatures {
                p2p: false,
                streaming: false,
                double_vpn: false,
                obfuscated: false,
            },
        };
        manager.add_server(server).await;
        let servers = manager.servers().await;
        assert_eq!(servers.len(), 1);
    }

    #[tokio::test]
    async fn test_split_tunnel_rules() {
        let manager = VPNManager::new(VPNConfig::default());
        let rule = SplitTunnelRule {
            hostname: "example.com".to_string(),
            bypass_vpn: true,
            description: "Local network".to_string(),
        };
        manager.add_split_tunnel_rule(rule).await;
        let config = manager.config().await;
        assert_eq!(config.split_tunneling_rules.len(), 1);
    }
}