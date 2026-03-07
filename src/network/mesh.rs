//! Mesh Networking Implementation
//! 
//! Provides decentralized peer-to-peer mesh networking

use anyhow::{anyhow, Result};
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Mesh node
#[derive(Debug, Clone)]
pub struct MeshNode {
    /// Node ID
    pub id: String,
    /// Node address
    pub address: SocketAddr,
    /// Node public key
    pub public_key: Vec<u8>,
    /// Node capabilities
    pub capabilities: NodeCapabilities,
    /// Connection quality (0-100)
    pub quality: u8,
    /// Last seen
    pub last_seen: Instant,
    /// Hop count to this node
    pub hops: u8,
}

/// Node capabilities
#[derive(Debug, Clone, Default)]
pub struct NodeCapabilities {
    /// Can relay traffic
    pub relay: bool,
    /// Can store data
    pub storage: bool,
    /// Available bandwidth (bytes/sec)
    pub bandwidth: u64,
    /// Available storage (bytes)
    pub storage_capacity: u64,
}

/// Mesh connection
#[derive(Debug, Clone)]
pub struct MeshConnection {
    /// Connection ID
    pub id: String,
    /// Local node ID
    pub local_id: String,
    /// Remote node ID
    pub remote_id: String,
    /// Connection state
    pub state: ConnectionState,
    /// Bytes sent
    pub bytes_sent: u64,
    /// Bytes received
    pub bytes_received: u64,
    /// Created at
    pub created_at: Instant,
}

/// Connection state
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    /// Pending connection
    Pending,
    /// Handshaking
    Handshaking,
    /// Connected
    Connected,
    /// Disconnected
    Disconnected,
    /// Failed
    Failed(String),
}

/// Mesh route
#[derive(Debug, Clone)]
pub struct MeshRoute {
    /// Destination node ID
    pub destination: String,
    /// Path to destination (node IDs)
    pub path: Vec<String>,
    /// Route metric (lower is better)
    pub metric: u32,
    /// Route last updated
    pub updated: Instant,
}

/// Mesh message
#[derive(Debug, Clone)]
pub struct MeshMessage {
    /// Message ID
    pub id: String,
    /// Source node ID
    pub source: String,
    /// Destination node ID (or broadcast)
    pub destination: Option<String>,
    /// Message type
    pub message_type: MeshMessageType,
    /// Payload
    pub payload: Vec<u8>,
    /// Timestamp
    pub timestamp: Instant,
    /// TTL (hops remaining)
    pub ttl: u8,
}

/// Mesh message type
#[derive(Debug, Clone, PartialEq)]
pub enum MeshMessageType {
    /// Node discovery
    Discovery,
    /// Node announcement
    Announcement,
    /// Data message
    Data,
    /// Route request
    RouteRequest,
    /// Route reply
    RouteReply,
    /// Ping
    Ping,
    /// Pong
    Pong,
}

/// Mesh network handler
pub struct MeshNetwork {
    /// Local node ID
    local_id: String,
    /// Known nodes
    nodes: Arc<Mutex<HashMap<String, MeshNode>>>,
    /// Active connections
    connections: Arc<Mutex<HashMap<String, MeshConnection>>>,
    /// Routing table
    routes: Arc<Mutex<HashMap<String, MeshRoute>>>,
    /// Configuration
    config: MeshConfig,
    /// Running state
    running: Arc<Mutex<bool>>,
}

/// Mesh network configuration
#[derive(Debug, Clone)]
pub struct MeshConfig {
    /// Listen port
    pub listen_port: u16,
    /// Maximum connections
    pub max_connections: usize,
    /// Discovery interval
    pub discovery_interval: Duration,
    /// Route refresh interval
    pub route_refresh_interval: Duration,
    /// Maximum hops
    pub max_hops: u8,
    /// Enable relaying
    pub enable_relay: bool,
    /// Bootstrap nodes
    pub bootstrap_nodes: Vec<SocketAddr>,
}

impl Default for MeshConfig {
    fn default() -> Self {
        Self {
            listen_port: 7654,
            max_connections: 20,
            discovery_interval: Duration::from_secs(60),
            route_refresh_interval: Duration::from_secs(30),
            max_hops: 10,
            enable_relay: true,
            bootstrap_nodes: Vec::new(),
        }
    }
}

impl MeshNetwork {
    /// Creates a new mesh network handler
    pub fn new() -> Result<Self> {
        Self::with_config(MeshConfig::default())
    }
    
    /// Creates with custom configuration
    pub fn with_config(config: MeshConfig) -> Result<Self> {
        let local_id = uuid::Uuid::new_v4().to_string();
        
        log::info!("Initializing mesh network with ID: {}", local_id);
        
        Ok(Self {
            local_id,
            nodes: Arc::new(Mutex::new(HashMap::new())),
            connections: Arc::new(Mutex::new(HashMap::new())),
            routes: Arc::new(Mutex::new(HashMap::new())),
            config,
            running: Arc::new(Mutex::new(true)),
        })
    }
    
    /// Gets the local node ID
    pub fn local_id(&self) -> &str {
        &self.local_id
    }
    
    /// Connects to a bootstrap node
    pub fn bootstrap(&self) -> Result<()> {
        for addr in &self.config.bootstrap_nodes {
            self.connect_to_node(addr)?;
        }
        Ok(())
    }
    
    /// Connects to a node
    pub fn connect_to_node(&self, address: &SocketAddr) -> Result<MeshConnection> {
        log::info!("Connecting to mesh node at {}", address);
        
        // Generate remote ID (in real implementation, obtained through handshake)
        let remote_id = uuid::Uuid::new_v4().to_string();
        
        let conn = MeshConnection {
            id: uuid::Uuid::new_v4().to_string(),
            local_id: self.local_id.clone(),
            remote_id: remote_id.clone(),
            state: ConnectionState::Connected,
            bytes_sent: 0,
            bytes_received: 0,
            created_at: Instant::now(),
        };
        
        // Add to connections
        self.connections.lock().unwrap().insert(conn.id.clone(), conn.clone());
        
        // Add node to known nodes
        let node = MeshNode {
            id: remote_id.clone(),
            address: *address,
            public_key: vec![0u8; 32],
            capabilities: NodeCapabilities::default(),
            quality: 100,
            last_seen: Instant::now(),
            hops: 1,
        };
        
        self.nodes.lock().unwrap().insert(remote_id, node);
        
        log::info!("Connected to mesh node");
        Ok(conn)
    }
    
    /// Disconnects from a node
    pub fn disconnect(&self, connection_id: &str) -> Result<()> {
        let mut connections = self.connections.lock().unwrap();
        if let Some(conn) = connections.remove(connection_id) {
            log::info!("Disconnected from node: {}", conn.remote_id);
            
            // Remove from routes
            let mut routes = self.routes.lock().unwrap();
            routes.retain(|_, r| !r.path.contains(&conn.remote_id));
        }
        Ok(())
    }
    
    /// Sends a message to a node
    pub fn send_message(&self, destination: &str, payload: Vec<u8>) -> Result<String> {
        let message = MeshMessage {
            id: uuid::Uuid::new_v4().to_string(),
            source: self.local_id.clone(),
            destination: Some(destination.to_string()),
            message_type: MeshMessageType::Data,
            payload,
            timestamp: Instant::now(),
            ttl: self.config.max_hops,
        };
        
        // Find route to destination
        let route = self.find_route(destination)?;
        
        log::info!("Sending message {} to {} via {:?}", message.id, destination, route.path);
        
        // Update connection stats
        self.connections.lock().unwrap()
            .values_mut()
            .for_each(|c| c.bytes_sent += message.payload.len() as u64);
        
        Ok(message.id)
    }
    
    /// Broadcasts a message to all nodes
    pub fn broadcast(&self, payload: Vec<u8>) -> Result<String> {
        let message = MeshMessage {
            id: uuid::Uuid::new_v4().to_string(),
            source: self.local_id.clone(),
            destination: None,
            message_type: MeshMessageType::Data,
            payload,
            timestamp: Instant::now(),
            ttl: self.config.max_hops,
        };
        
        log::info!("Broadcasting message {} to all nodes", message.id);
        
        // Update all connections
        let nodes_count = self.nodes.lock().unwrap().len();
        self.connections.lock().unwrap()
            .values_mut()
            .for_each(|c| c.bytes_sent += message.payload.len() as u64);
        
        Ok(message.id)
    }
    
    /// Finds a route to a destination
    pub fn find_route(&self, destination: &str) -> Result<MeshRoute> {
        // Check routing table first
        {
            let routes = self.routes.lock().unwrap();
            if let Some(route) = routes.get(destination) {
                return Ok(route.clone());
            }
        }
        
        // Find direct connection
        let nodes = self.nodes.lock().unwrap();
        if let Some(node) = nodes.get(destination) {
            return Ok(MeshRoute {
                destination: destination.to_string(),
                path: vec![destination.to_string()],
                metric: node.hops as u32,
                updated: Instant::now(),
            });
        }
        
        // No route found - would initiate route discovery in real implementation
        Err(anyhow!("No route to node: {}", destination))
    }
    
    /// Discovers nearby nodes
    pub fn discover_nodes(&self) -> Result<Vec<MeshNode>> {
        log::info!("Discovering mesh nodes...");
        
        // Send discovery message
        let _message = MeshMessage {
            id: uuid::Uuid::new_v4().to_string(),
            source: self.local_id.clone(),
            destination: None,
            message_type: MeshMessageType::Discovery,
            payload: vec![],
            timestamp: Instant::now(),
            ttl: 3,
        };
        
        // Return known nodes
        Ok(self.nodes.lock().unwrap().values().cloned().collect())
    }
    
    /// Gets all known nodes
    pub fn get_nodes(&self) -> Vec<MeshNode> {
        self.nodes.lock().unwrap().values().cloned().collect()
    }
    
    /// Gets active connections
    pub fn get_connections(&self) -> Vec<MeshConnection> {
        self.connections.lock().unwrap().values().cloned().collect()
    }
    
    /// Gets the routing table
    pub fn get_routes(&self) -> Vec<MeshRoute> {
        self.routes.lock().unwrap().values().cloned().collect()
    }
    
    /// Updates the routing table
    pub fn update_routes(&self) -> Result<()> {
        log::debug!("Updating routing table");
        
        let nodes = self.nodes.lock().unwrap();
        let mut routes = self.routes.lock().unwrap();
        
        // Create direct routes to known nodes
        for (id, node) in nodes.iter() {
            let route = MeshRoute {
                destination: id.clone(),
                path: vec![id.clone()],
                metric: node.hops as u32,
                updated: Instant::now(),
            };
            routes.insert(id.clone(), route);
        }
        
        Ok(())
    }
    
    /// Shuts down the mesh network
    pub fn shutdown(self) -> Result<()> {
        *self.running.lock().unwrap() = false;
        
        // Clear all state
        self.nodes.lock().unwrap().clear();
        self.connections.lock().unwrap().clear();
        self.routes.lock().unwrap().clear();
        
        log::info!("Mesh network shut down");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mesh_creation() {
        let mesh = MeshNetwork::new().unwrap();
        assert!(!mesh.local_id().is_empty());
        assert!(mesh.get_nodes().is_empty());
    }
    
    #[test]
    fn test_connect_to_node() {
        let mesh = MeshNetwork::new().unwrap();
        let addr: SocketAddr = "127.0.0.1:7654".parse().unwrap();
        
        let conn = mesh.connect_to_node(&addr).unwrap();
        assert_eq!(conn.state, ConnectionState::Connected);
        
        let nodes = mesh.get_nodes();
        assert_eq!(nodes.len(), 1);
    }
    
    #[test]
    fn test_send_message() {
        let mesh = MeshNetwork::new().unwrap();
        let addr: SocketAddr = "127.0.0.1:7654".parse().unwrap();
        
        let conn = mesh.connect_to_node(&addr).unwrap();
        let msg_id = mesh.send_message(&conn.remote_id, vec![1, 2, 3]).unwrap();
        
        assert!(!msg_id.is_empty());
    }
    
    #[test]
    fn test_broadcast() {
        let mesh = MeshNetwork::new().unwrap();
        
        // Add some nodes
        let addr1: SocketAddr = "127.0.0.1:7654".parse().unwrap();
        let addr2: SocketAddr = "127.0.0.1:7655".parse().unwrap();
        
        mesh.connect_to_node(&addr1).unwrap();
        mesh.connect_to_node(&addr2).unwrap();
        
        let msg_id = mesh.broadcast(vec![1, 2, 3]).unwrap();
        assert!(!msg_id.is_empty());
    }
}