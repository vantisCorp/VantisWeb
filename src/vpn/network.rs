//! Network Management
//!
//! Network interface management, routing, kill switch, and connection testing.

use std::collections::HashSet;
use std::net::IpAddr;
use crate::vpn::{VPNServer, VPNError, SplitTunnelRule};

/// Network manager
pub struct NetworkManager {
    kill_switch_enabled: bool,
    default_gateway: Option<String>,
    routing_rules: HashSet<RoutingRule>,
    split_tunnel_rules: Vec<SplitTunnelRule>,
}

/// Routing rule
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RoutingRule {
    destination: String,
    gateway: String,
    interface: String,
}

/// Network interface
#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub ip: String,
    pub netmask: String,
    pub gateway: Option<String>,
    pub is_up: bool,
}

impl NetworkManager {
    /// Create a new network manager
    pub fn new() -> Self {
        Self {
            kill_switch_enabled: false,
            default_gateway: None,
            routing_rules: HashSet::new(),
            split_tunnel_rules: vec![],
        }
    }

    /// Enable kill switch
    pub async fn enable_kill_switch(&mut self) -> Result<(), VPNError> {
        // In a real implementation, this would:
        // 1. Block all non-VPN traffic
        // 2. Allow VPN traffic only
        // 3. Configure firewall rules

        self.kill_switch_enabled = true;
        
        log::info!("Kill switch enabled");
        Ok(())
    }

    /// Disable kill switch
    pub async fn disable_kill_switch(&mut self) -> Result<(), VPNError> {
        // In a real implementation, this would:
        // 1. Restore normal routing
        // 2. Remove firewall blocks

        self.kill_switch_enabled = false;
        
        log::info!("Kill switch disabled");
        Ok(())
    }

    /// Check if kill switch is enabled
    pub fn is_kill_switch_enabled(&self) -> bool {
        self.kill_switch_enabled
    }

    /// Add routing rule
    pub fn add_routing_rule(&mut self, rule: RoutingRule) -> Result<(), VPNError> {
        // In a real implementation, this would add a route table entry
        self.routing_rules.insert(rule);
        Ok(())
    }

    /// Remove routing rule
    pub fn remove_routing_rule(&mut self, destination: &str) -> Result<(), VPNError> {
        self.routing_rules.retain(|r| r.destination != destination);
        Ok(())
    }

    /// Get default gateway
    pub async fn get_default_gateway(&self) -> Result<String, VPNError> {
        // In a real implementation, this would query the routing table
        Ok(self.default_gateway.clone().unwrap_or_default())
    }

    /// Set default gateway
    pub async fn set_default_gateway(&mut self, gateway: String) -> Result<(), VPNError> {
        self.default_gateway = Some(gateway);
        Ok(())
    }

    /// Ping server to measure latency
    pub async fn ping_server(&self, server: &VPNServer) -> Result<u64, VPNError> {
        // In a real implementation, this would:
        // 1. Send ICMP echo requests
        // 2. Measure round-trip time
        // 3. Return average latency

        // Simulate latency measurement
        let simulated_latency = match server.country.as_str() {
            "US" | "CA" | "MX" => 20 + (rand::random::<u64>() % 30),
            "GB" | "DE" | "FR" => 80 + (rand::random::<u64>() % 40),
            "JP" | "KR" | "SG" => 120 + (rand::random::<u64>() % 50),
            "AU" | "NZ" => 180 + (rand::random::<u64>() % 60),
            _ => 60 + (rand::random::<u64>() % 80),
        };

        Ok(simulated_latency)
    }

    /// Test VPN connection
    pub async fn test_vpn_connection(&self) -> Result<bool, VPNError> {
        // In a real implementation, this would:
        // 1. Check VPN interface status
        // 2. Test connectivity through VPN
        // 3. Verify routing table
        // 4. Check for IP leaks

        Ok(true)
    }

    /// Get network interfaces
    pub async fn get_interfaces(&self) -> Result<Vec<NetworkInterface>, VPNError> {
        // In a real implementation, this would query the OS for network interfaces
        
        Ok(vec![
            NetworkInterface {
                name: "eth0".to_string(),
                ip: "192.168.1.100".to_string(),
                netmask: "255.255.255.0".to_string(),
                gateway: Some("192.168.1.1".to_string()),
                is_up: true,
            }
        ])
    }

    /// Check for IP leaks
    pub async fn check_ip_leaks(&self) -> Result<IPLeakResult, VPNError> {
        // In a real implementation, this would:
        // 1. Query external IP check services
        // 2. Compare with expected VPN IP
        // 3. Check for WebRTC leaks
        // 4. Check for DNS leaks

        Ok(IPLeakResult {
            ip_leaked: false,
            detected_ip: "203.0.113.1".to_string(),
            expected_ip: Some("203.0.113.1".to_string()),
            webrtc_leaked: false,
            dns_leaked: false,
        })
    }

    /// Get current public IP
    pub async fn get_public_ip(&self) -> Result<String, VPNError> {
        // In a real implementation, this would query an IP check service
        Ok("203.0.113.1".to_string())
    }

    /// Configure split tunneling
    pub async fn configure_split_tunnel(&mut self, rules: Vec<SplitTunnelRule>) -> Result<(), VPNError> {
        self.split_tunnel_rules = rules;
        
        // In a real implementation, this would:
        // 1. Add routes for bypass hosts
        // 2. Configure policy routing
        // 3. Update firewall rules

        log::info!("Configured {} split tunneling rules", self.split_tunnel_rules.len());
        Ok(())
    }

    /// Get split tunneling rules
    pub fn split_tunnel_rules(&self) -> &[SplitTunnelRule] {
        &self.split_tunnel_rules
    }

    /// Check if URL should bypass VPN
    pub fn should_bypass_vpn(&self, url: &str) -> bool {
        for rule in &self.split_tunnel_rules {
            if url.contains(&rule.hostname) {
                return rule.bypass_vpn;
            }
        }
        false
    }

    /// Flush routing cache
    pub async fn flush_routing_cache(&self) -> Result<(), VPNError> {
        // In a real implementation, this would flush the OS routing cache
        Ok(())
    }

    /// Get active connections
    pub async fn get_active_connections(&self) -> Result<Vec<ActiveConnection>, VPNError> {
        // In a real implementation, this would query the connection table
        
        Ok(vec![
            ActiveConnection {
                protocol: "TCP".to_string(),
                local_address: "192.168.1.100:54321".to_string(),
                remote_address: "203.0.113.1:443".to_string(),
                state: "ESTABLISHED".to_string(),
                pid: None,
            }
        ])
    }
}

/// IP leak check result
#[derive(Debug, Clone)]
pub struct IPLeakResult {
    pub ip_leaked: bool,
    pub detected_ip: String,
    pub expected_ip: Option<String>,
    pub webrtc_leaked: bool,
    pub dns_leaked: bool,
}

/// Active network connection
#[derive(Debug, Clone)]
pub struct ActiveConnection {
    pub protocol: String,
    pub local_address: String,
    pub remote_address: String,
    pub state: String,
    pub pid: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vpn::{ServerFeatures, VPNProtocol};

    #[tokio::test]
    async fn test_kill_switch() {
        let mut manager = NetworkManager::new();
        assert!(!manager.is_kill_switch_enabled());
        
        manager.enable_kill_switch().await.unwrap();
        assert!(manager.is_kill_switch_enabled());
        
        manager.disable_kill_switch().await.unwrap();
        assert!(!manager.is_kill_switch_enabled());
    }

    #[tokio::test]
    async fn test_ping_server() {
        let manager = NetworkManager::new();
        let server = VPNServer {
            id: "us-test".to_string(),
            name: "US Test".to_string(),
            country: "US".to_string(),
            city: "New York".to_string(),
            hostname: "us.vpn.com".to_string(),
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
        
        let latency = manager.ping_server(&server).await.unwrap();
        assert!(latency >= 20);
        assert!(latency < 100);
    }

    #[tokio::test]
    async fn test_split_tunneling() {
        let mut manager = NetworkManager::new();
        let rule = SplitTunnelRule {
            hostname: "example.com".to_string(),
            bypass_vpn: true,
            description: "Bypass for local site".to_string(),
        };
        
        manager.configure_split_tunnel(vec![rule]).await.unwrap();
        assert!(manager.should_bypass_vpn("https://example.com/path"));
        assert!(!manager.should_bypass_vpn("https://other.com"));
    }

    #[tokio::test]
    async fn test_routing_rules() {
        let mut manager = NetworkManager::new();
        let rule = RoutingRule {
            destination: "10.0.0.0/24".to_string(),
            gateway: "10.0.0.1".to_string(),
            interface: "wg0".to_string(),
        };
        
        manager.add_routing_rule(rule).unwrap();
        assert_eq!(manager.routing_rules.len(), 1);
        
        manager.remove_routing_rule("10.0.0.0/24").unwrap();
        assert_eq!(manager.routing_rules.len(), 0);
    }
}