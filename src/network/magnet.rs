//! Magnet Core (BitTorrent) Implementation
//! 
//! Provides peer-to-peer file sharing via BitTorrent protocol

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Magnet link
#[derive(Debug, Clone)]
pub struct MagnetLink {
    /// Info hash (BTIH)
    pub info_hash: String,
    /// Display name
    pub name: String,
    /// Trackers
    pub trackers: Vec<String>,
    /// Web seeds
    pub web_seeds: Vec<String>,
    /// File size in bytes
    pub length: u64,
}

/// Torrent peer
#[derive(Debug, Clone)]
pub struct TorrentPeer {
    /// Peer ID
    pub id: String,
    /// Peer address
    pub address: SocketAddr,
    /// Peer state
    pub state: PeerState,
    /// Downloaded bytes
    pub downloaded: u64,
    /// Uploaded bytes
    pub uploaded: u64,
}

/// Peer state
#[derive(Debug, Clone, PartialEq)]
pub enum PeerState {
    /// Disconnected
    Disconnected,
    /// Connecting
    Connecting,
    /// Handshaking
    Handshaking,
    /// Connected and choking
    Choked,
    /// Connected and interested
    Interested,
    /// Connected and downloading
    Downloading,
    /// Connected and seeding
    Seeding,
}

/// Download task
#[derive(Debug, Clone)]
pub struct DownloadTask {
    /// Task ID
    pub id: String,
    /// Magnet link
    pub magnet: MagnetLink,
    /// Download directory
    pub download_dir: PathBuf,
    /// Download status
    pub status: DownloadStatus,
    /// Progress (0.0 - 1.0)
    pub progress: f64,
    /// Download speed (bytes/sec)
    pub download_speed: u64,
    /// Upload speed (bytes/sec)
    pub upload_speed: u64,
    /// Connected peers
    pub peers: Vec<String>,
}

/// Download status
#[derive(Debug, Clone, PartialEq)]
pub enum DownloadStatus {
    /// Queued
    Queued,
    /// Finding peers
    FindingPeers,
    /// Downloading metadata
    Metadata,
    /// Downloading
    Downloading,
    /// Seeding
    Seeding,
    /// Completed
    Completed,
    /// Paused
    Paused,
    /// Error
    Error(String),
}

/// Magnet core handler
pub struct MagnetCore {
    /// Active downloads
    downloads: Arc<Mutex<HashMap<String, DownloadTask>>>,
    /// Known peers
    peers: Arc<Mutex<HashMap<String, TorrentPeer>>>,
    /// Configuration
    config: MagnetConfig,
    /// Running state
    running: Arc<Mutex<bool>>,
}

/// Magnet configuration
#[derive(Debug, Clone)]
pub struct MagnetConfig {
    /// Listen port
    pub listen_port: u16,
    /// Maximum connections
    pub max_connections: usize,
    /// Download directory
    pub download_dir: PathBuf,
    /// Maximum upload speed (0 = unlimited)
    pub max_upload_speed: u64,
    /// Maximum download speed (0 = unlimited)
    pub max_download_speed: u64,
    /// Enable DHT
    pub enable_dht: bool,
    /// Enable PEX
    pub enable_pex: bool,
}

impl Default for MagnetConfig {
    fn default() -> Self {
        Self {
            listen_port: 6881,
            max_connections: 50,
            download_dir: PathBuf::from("./downloads"),
            max_upload_speed: 0,
            max_download_speed: 0,
            enable_dht: true,
            enable_pex: true,
        }
    }
}

impl MagnetCore {
    /// Creates a new magnet core handler
    pub fn new() -> Result<Self> {
        Self::with_config(MagnetConfig::default())
    }
    
    /// Creates with custom configuration
    pub fn with_config(config: MagnetConfig) -> Result<Self> {
        log::info!("Initializing Magnet core on port {}", config.listen_port);
        
        // Create download directory if needed
        std::fs::create_dir_all(&config.download_dir)?;
        
        Ok(Self {
            downloads: Arc::new(Mutex::new(HashMap::new())),
            peers: Arc::new(Mutex::new(HashMap::new())),
            config,
            running: Arc::new(Mutex::new(true)),
        })
    }
    
    /// Parses a magnet link
    pub fn parse_magnet(&self, uri: &str) -> Result<MagnetLink> {
        if !uri.starts_with("magnet:?") {
            return Err(anyhow!("Invalid magnet link"));
        }
        
        let mut info_hash = String::new();
        let mut name = String::new();
        let mut trackers = Vec::new();
        let mut length = 0u64;
        
        // Parse query parameters
        let query = uri.strip_prefix("magnet:?").unwrap_or("");
        for pair in query.split('&') {
            let parts: Vec<&str> = pair.splitn(2, '=').collect();
            if parts.len() != 2 {
                continue;
            }
            
            match parts[0] {
                "xt" => {
                    // Exact topic (info hash)
                    let xt = urlencoding_decode(parts[1])?;
                    if let Some(hash) = xt.strip_prefix("urn:btih:") {
                        info_hash = hash.to_uppercase();
                    }
                }
                "dn" => {
                    // Display name
                    name = urlencoding_decode(parts[1])?;
                }
                "tr" => {
                    // Tracker
                    trackers.push(urlencoding_decode(parts[1])?);
                }
                "xl" => {
                    // Exact length
                    length = parts[1].parse().unwrap_or(0);
                }
                "ws" => {
                    // Web seed - not included in trackers
                }
                _ => {}
            }
        }
        
        if info_hash.is_empty() {
            return Err(anyhow!("No info hash in magnet link"));
        }
        
        Ok(MagnetLink {
            info_hash,
            name,
            trackers,
            web_seeds: Vec::new(),
            length,
        })
    }
    
    /// Starts a download from magnet link
    pub fn start_download(&self, magnet_uri: &str) -> Result<DownloadTask> {
        let magnet = self.parse_magnet(magnet_uri)?;
        
        let task_id = magnet.info_hash.clone();
        
        let task = DownloadTask {
            id: task_id.clone(),
            magnet,
            download_dir: self.config.download_dir.clone(),
            status: DownloadStatus::FindingPeers,
            progress: 0.0,
            download_speed: 0,
            upload_speed: 0,
            peers: Vec::new(),
        };
        
        self.downloads.lock().unwrap().insert(task_id.clone(), task.clone());
        
        log::info!("Started download: {}", task_id);
        Ok(task)
    }
    
    /// Pauses a download
    pub fn pause_download(&self, task_id: &str) -> Result<()> {
        let mut downloads = self.downloads.lock().unwrap();
        if let Some(task) = downloads.get_mut(task_id) {
            task.status = DownloadStatus::Paused;
            log::info!("Paused download: {}", task_id);
        }
        Ok(())
    }
    
    /// Resumes a download
    pub fn resume_download(&self, task_id: &str) -> Result<()> {
        let mut downloads = self.downloads.lock().unwrap();
        if let Some(task) = downloads.get_mut(task_id) {
            if task.status == DownloadStatus::Paused {
                task.status = DownloadStatus::Downloading;
                log::info!("Resumed download: {}", task_id);
            }
        }
        Ok(())
    }
    
    /// Cancels a download
    pub fn cancel_download(&self, task_id: &str) -> Result<()> {
        let mut downloads = self.downloads.lock().unwrap();
        downloads.remove(task_id);
        log::info!("Cancelled download: {}", task_id);
        Ok(())
    }
    
    /// Gets download task
    pub fn get_download(&self, task_id: &str) -> Option<DownloadTask> {
        let downloads = self.downloads.lock().unwrap();
        downloads.get(task_id).cloned()
    }
    
    /// Gets all downloads
    pub fn get_all_downloads(&self) -> Vec<DownloadTask> {
        let downloads = self.downloads.lock().unwrap();
        downloads.values().cloned().collect()
    }
    
    /// Adds a peer
    pub fn add_peer(&self, peer_id: &str, address: SocketAddr) -> Result<()> {
        let peer = TorrentPeer {
            id: peer_id.to_string(),
            address,
            state: PeerState::Disconnected,
            downloaded: 0,
            uploaded: 0,
        };
        
        self.peers.lock().unwrap().insert(peer_id.to_string(), peer);
        log::info!("Added peer: {} at {}", peer_id, address);
        Ok(())
    }
    
    /// Gets connected peers
    pub fn get_peers(&self) -> Vec<TorrentPeer> {
        let peers = self.peers.lock().unwrap();
        peers.values().cloned().collect()
    }
    
    /// Shuts down the protocol
    pub fn shutdown(self) -> Result<()> {
        *self.running.lock().unwrap() = false;
        
        // Clear downloads
        self.downloads.lock().unwrap().clear();
        
        // Clear peers
        self.peers.lock().unwrap().clear();
        
        log::info!("Magnet core shut down");
        Ok(())
    }
}

/// URL decodes a string
fn urlencoding_decode(s: &str) -> Result<String> {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                let byte = u8::from_str_radix(&hex, 16)
                    .map_err(|_| anyhow!("Invalid hex encoding"))?;
                result.push(byte as char);
            }
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_magnet_core_creation() {
        let magnet = MagnetCore::new().unwrap();
        assert!(magnet.get_all_downloads().is_empty());
    }
    
    #[test]
    fn test_parse_magnet() {
        let magnet = MagnetCore::new().unwrap();
        let link = magnet.parse_magnet("magnet:?xt=urn:btih:0123456789ABCDEF0123456789ABCDEF01234567&dn=test+file&tr=udp://tracker.example.com:1337").unwrap();
        
        assert_eq!(link.info_hash, "0123456789ABCDEF0123456789ABCDEF01234567");
        assert_eq!(link.name, "test file");
        assert_eq!(link.trackers, vec!["udp://tracker.example.com:1337"]);
    }
    
    #[test]
    fn test_start_download() {
        let magnet = MagnetCore::new().unwrap();
        let task = magnet.start_download("magnet:?xt=urn:btih:0123456789ABCDEF0123456789ABCDEF01234567&dn=test").unwrap();
        
        assert_eq!(task.status, DownloadStatus::FindingPeers);
    }
}