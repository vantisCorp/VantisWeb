//! Vantis Security Manager
//! 
//! Central security orchestration:
//! - Crypto operations
//! - Immune system coordination
//! - Sandbox management
//! - Threat detection

use anyhow::{Context, Result};
use log::{debug, info};
use serde::{Deserialize, Serialize};

use super::crypto::CryptoEngine;
use super::immune_system::DigitalImmuneSystem;
use super::sandbox::Sandbox;

/// Security Manager - Central security coordinator
pub struct SecurityManager {
    crypto: CryptoEngine,
    immune_system: DigitalImmuneSystem,
    sandbox: Sandbox,
    enabled: bool,
}

/// Security status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStatus {
    pub crypto_enabled: bool,
    pub immune_system_active: bool,
    pub sandbox_active: bool,
    pub threats_detected: u32,
    pub last_scan: Option<chrono::DateTime<chrono::Utc>>,
}

impl SecurityManager {
    /// Create a new security manager
    pub async fn new() -> Result<Self> {
        info!("Initializing Vantis Security Manager...");
        
        let crypto = CryptoEngine::new()?;
        let immune_system = DigitalImmuneSystem::new()?;
        let sandbox = Sandbox::new()?;
        
        info!("✓ Crypto Engine initialized");
        info!("✓ Digital Immune System initialized");
        info!("✓ Sandbox initialized");
        
        Ok(Self {
            crypto,
            immune_system,
            sandbox,
            enabled: true,
        })
    }
    
    /// Encrypt data using post-quantum cryptography
    pub async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        debug!("Encrypting data ({} bytes)", data.len());
        
        let encrypted = self.crypto.encrypt(data)
            .context("Encryption failed")?;
        
        Ok(encrypted)
    }
    
    /// Decrypt data
    pub async fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        debug!("Decrypting data ({} bytes)", data.len());
        
        let decrypted = self.crypto.decrypt(data)
            .context("Decryption failed")?;
        
        Ok(decrypted)
    }
    
    /// Generate cryptographic hash
    pub async fn hash(&self, data: &[u8]) -> Result<String> {
        debug!("Generating hash for {} bytes", data.len());
        
        let hash = self.crypto.hash(data)
            .context("Hash generation failed")?;
        
        Ok(hash)
    }
    
    /// Scan for threats
    pub async fn scan_threats(&self, path: &str) -> Result<ScanResult> {
        info!("Scanning for threats: {}", path);
        
        let threats_found = self.immune_system.scan(path).await?;
        
        Ok(ScanResult {
            path: path.to_string(),
            threats_found,
            safe: threats_found == 0,
        })
    }
    
    /// Initialize sandbox
    pub async fn init_sandbox(&mut self) -> Result<()> {
        info!("Initializing sandbox...");
        
        self.sandbox.initialize()
            .context("Sandbox initialization failed")?;
        
        info!("Sandbox initialized");
        
        Ok(())
    }
    
    /// Get security status
    pub async fn get_status(&self) -> SecurityStatus {
        SecurityStatus {
            crypto_enabled: true,
            immune_system_active: self.immune_system.is_active(),
            sandbox_active: self.sandbox.is_active(),
            threats_detected: self.immune_system.get_threats_detected().await,
            last_scan: self.immune_system.get_last_scan().await,
        }
    }
}

/// Scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub path: String,
    pub threats_found: u32,
    pub safe: bool,
}