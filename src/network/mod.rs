//! Network Module
//! 
//! Network protocols and communication:
//! - Onion Protocol (Tor)
//! - Magnet Core (BitTorrent)
//! - Nano-Sharding VPN
//! - Mesh networking

pub mod onion;
pub mod magnet;
pub mod vpn;
pub mod mesh;

pub use onion::*;
pub use magnet::*;
pub use vpn::*;
pub use mesh::*;

use anyhow::Result;
use std::net::SocketAddr;

/// Network manager coordinating all protocols
pub struct NetworkManager {
    /// Onion protocol handler
    onion: Option<OnionProtocol>,
    /// Magnet protocol handler
    magnet: Option<MagnetCore>,
    /// VPN handler
    vpn: Option<NanoVpn>,
    /// Mesh network handler
    mesh: Option<MeshNetwork>,
    /// Network configuration
    config: NetworkConfig,
}

/// Network configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Enable Tor/Onion routing
    pub enable_onion: bool,
    /// Enable BitTorrent/Magnet
    pub enable_magnet: bool,
    /// Enable VPN
    pub enable_vpn: bool,
    /// Enable mesh networking
    pub enable_mesh: bool,
    /// Default bind address
    pub bind_address: SocketAddr,
    /// Maximum connections per protocol
    pub max_connections: usize,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            enable_onion: false,
            enable_magnet: false,
            enable_vpn: false,
            enable_mesh: false,
            bind_address: "0.0.0.0:0".parse().unwrap(),
            max_connections: 100,
        }
    }
}

impl NetworkManager {
    /// Creates a new network manager
    pub fn new(config: NetworkConfig) -> Self {
        Self {
            onion: None,
            magnet: None,
            vpn: None,
            mesh: None,
            config,
        }
    }
    
    /// Initializes enabled protocols
    pub fn initialize(&mut self) -> Result<()> {
        if self.config.enable_onion {
            self.onion = Some(OnionProtocol::new()?);
            log::info!("Onion protocol initialized");
        }
        
        if self.config.enable_magnet {
            self.magnet = Some(MagnetCore::new()?);
            log::info!("Magnet core initialized");
        }
        
        if self.config.enable_vpn {
            self.vpn = Some(NanoVpn::new()?);
            log::info!("Nano VPN initialized");
        }
        
        if self.config.enable_mesh {
            self.mesh = Some(MeshNetwork::new()?);
            log::info!("Mesh network initialized");
        }
        
        Ok(())
    }
    
    /// Gets the onion protocol handler
    pub fn onion(&self) -> Option<&OnionProtocol> {
        self.onion.as_ref()
    }
    
    /// Gets the magnet protocol handler
    pub fn magnet(&self) -> Option<&MagnetCore> {
        self.magnet.as_ref()
    }
    
    /// Gets the VPN handler
    pub fn vpn(&self) -> Option<&NanoVpn> {
        self.vpn.as_ref()
    }
    
    /// Gets the mesh network handler
    pub fn mesh(&self) -> Option<&MeshNetwork> {
        self.mesh.as_ref()
    }
    
    /// Shuts down all protocols
    pub fn shutdown(&mut self) -> Result<()> {
        if let Some(onion) = self.onion.take() {
            onion.shutdown()?;
            log::info!("Onion protocol shut down");
        }
        
        if let Some(magnet) = self.magnet.take() {
            magnet.shutdown()?;
            log::info!("Magnet core shut down");
        }
        
        if let Some(vpn) = self.vpn.take() {
            vpn.shutdown()?;
            log::info!("VPN shut down");
        }
        
        if let Some(mesh) = self.mesh.take() {
            mesh.shutdown()?;
            log::info!("Mesh network shut down");
        }
        
        Ok(())
    }
}