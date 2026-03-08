//! QR Code Bridge for Mobile Pairing
//! 
//! Handles QR code generation and scanning for device pairing.

use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

use super::{QRPairingData, MobileEvent};

/// QR Code bridge for pairing
pub struct QRCodeBridge {
    /// Active pairing codes
    pairing_codes: RwLock<Vec<PairingCode>>,
    /// Event sender
    event_sender: broadcast::Sender<MobileEvent>,
    /// Configuration
    config: RwLock<QRBridgeConfig>,
}

/// Pairing code entry
#[derive(Debug, Clone)]
pub struct PairingCode {
    /// Code ID
    pub id: String,
    /// Pairing code
    pub code: String,
    /// QR data URL
    pub qr_data_url: String,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Expires at
    pub expires_at: DateTime<Utc>,
    /// Scanned by device ID
    pub scanned_by: Option<String>,
    /// Used flag
    pub used: bool,
}

/// QR Bridge configuration
#[derive(Debug, Clone)]
pub struct QRBridgeConfig {
    /// Code length
    pub code_length: usize,
    /// Expiration time in seconds
    pub expiration_seconds: i64,
    /// QR code size
    pub qr_size: usize,
    /// Enable deep link
    pub deep_link_enabled: bool,
    /// Deep link prefix
    pub deep_link_prefix: String,
}

impl Default for QRBridgeConfig {
    fn default() -> Self {
        Self {
            code_length: 6,
            expiration_seconds: 300,
            qr_size: 256,
            deep_link_enabled: true,
            deep_link_prefix: "vantisweb://pair".to_string(),
        }
    }
}

/// QR code data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRCodeData {
    /// Pairing code
    pub code: String,
    /// Browser ID
    pub browser_id: String,
    /// Browser name
    pub browser_name: String,
    /// Timestamp
    pub timestamp: i64,
    /// Deep link URL
    pub deep_link: String,
    /// Version
    pub version: u32,
}

impl QRCodeBridge {
    /// Create a new QR bridge
    pub async fn new(event_sender: broadcast::Sender<MobileEvent>) -> Result<Self> {
        Ok(Self {
            pairing_codes: RwLock::new(Vec::new()),
            event_sender,
            config: RwLock::new(QRBridgeConfig::default()),
        })
    }
    
    /// Generate pairing QR code
    pub async fn generate_pairing_qr(&self) -> Result<QRPairingData> {
        let config = self.config.read().await;
        
        // Generate unique pairing code
        let code = self.generate_code(config.code_length);
        
        // Create QR data
        let qr_data = QRCodeData {
            code: code.clone(),
            browser_id: Uuid::new_v4().to_string(),
            browser_name: "VantisWeb Browser".to_string(),
            timestamp: Utc::now().timestamp(),
            deep_link: format!("{}?code={}", config.deep_link_prefix, code),
            version: 1,
        };
        
        // Generate QR data URL
        let qr_data_url = self.generate_qr_data_url(&qr_data).await?;
        
        // Create pairing code entry
        let pairing_code = PairingCode {
            id: Uuid::new_v4().to_string(),
            code: code.clone(),
            qr_data_url: qr_data_url.clone(),
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::seconds(config.expiration_seconds),
            scanned_by: None,
            used: false,
        };
        
        // Store pairing code
        let mut codes = self.pairing_codes.write().await;
        codes.push(pairing_code.clone());
        
        // Emit pairing event
        let _ = self.event_sender.send(MobileEvent::PairingRequested(code.clone()));
        
        Ok(QRPairingData {
            code: code.clone(),
            qr_data_url,
            expires_at: pairing_code.expires_at,
            scanned_by: None,
        })
    }
    
    /// Validate pairing code
    pub async fn validate_pairing_code(&self, code: &str) -> Result<Option<PairingCode>> {
        let mut codes = self.pairing_codes.write().await;
        
        // Find matching code
        if let Some(pairing_code) = codes.iter_mut().find(|c| c.code == code) {
            // Check if not expired and not used
            if pairing_code.expires_at > Utc::now() && !pairing_code.used {
                return Ok(Some(pairing_code.clone()));
            }
        }
        
        Ok(None)
    }
    
    /// Mark code as scanned
    pub async fn mark_scanned(&self, code: &str, device_id: &str) -> Result<()> {
        let mut codes = self.pairing_codes.write().await;
        
        if let Some(pairing_code) = codes.iter_mut().find(|c| c.code == code) {
            pairing_code.scanned_by = Some(device_id.to_string());
            tracing::info!("Pairing code scanned: {} by device: {}", code, device_id);
        }
        
        Ok(())
    }
    
    /// Use pairing code
    pub async fn use_code(&self, code: &str) -> Result<bool> {
        let mut codes = self.pairing_codes.write().await;
        
        if let Some(pairing_code) = codes.iter_mut().find(|c| c.code == code) {
            if pairing_code.expires_at > Utc::now() && !pairing_code.used {
                pairing_code.used = true;
                tracing::info!("Pairing code used: {}", code);
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// Get active pairing codes
    pub async fn get_active_codes(&self) -> Result<Vec<PairingCode>> {
        let codes = self.pairing_codes.write().await;
        
        Ok(codes.iter()
            .filter(|c| c.expires_at > Utc::now() && !c.used)
            .cloned()
            .collect())
    }
    
    /// Cancel pairing code
    pub async fn cancel_code(&self, code: &str) -> Result<()> {
        let mut codes = self.pairing_codes.write().await;
        codes.retain(|c| c.code != code);
        
        tracing::info!("Pairing code cancelled: {}", code);
        Ok(())
    }
    
    /// Clean expired codes
    pub async fn clean_expired_codes(&self) -> Result<u32> {
        let mut codes = self.pairing_codes.write().await;
        let initial_count = codes.len();
        
        codes.retain(|c| c.expires_at > Utc::now());
        
        Ok((initial_count - codes.len()) as u32)
    }
    
    /// Generate pairing code
    fn generate_code(&self, length: usize) -> String {
        // Generate alphanumeric code
        const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
        
        use std::time::{SystemTime, UNIX_EPOCH};
        let mut result = String::new();
        let mut ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        
        for _ in 0..length {
            let idx = (ts % CHARSET.len() as u128) as usize;
            result.push(CHARSET[idx] as char);
            ts /= 37;
        }
        
        result
    }
    
    /// Generate QR data URL
    async fn generate_qr_data_url(&self, data: &QRCodeData) -> Result<String> {
        // Serialize data
        let json = serde_json::to_string(data)?;
        
        // In a real implementation, this would generate an actual QR code image
        // For now, we return a data URL with the JSON
        let encoded = urlencoding::encode(&json);
        
        // Return as a data URL (placeholder - would be actual QR image)
        // Using SVG QR code format
        let svg = self.generate_qr_svg(&json)?;
        let encoded_svg = urlencoding::encode(&svg);
        
        Ok(format!("data:image/svg+xml,{}", encoded_svg))
    }
    
    /// Generate QR code as SVG
    fn generate_qr_svg(&self, data: &str) -> Result<String> {
        // Simplified QR code representation
        // In production, use a proper QR code library like `qrcode`
        let size = 200;
        let module_count = 21; // Version 1 QR code
        
        // Create a simple placeholder pattern
        let mut modules = vec![vec![false; module_count]; module_count];
        
        // Add finder patterns (corners)
        self.add_finder_pattern(&mut modules, 0, 0);
        self.add_finder_pattern(&mut modules, module_count - 7, 0);
        self.add_finder_pattern(&mut modules, 0, module_count - 7);
        
        // Generate SVG
        let module_size = size as f64 / module_count as f64;
        let mut rects = String::new();
        
        for (y, row) in modules.iter().enumerate() {
            for (x, &filled) in row.iter().enumerate() {
                if filled {
                    let px = x as f64 * module_size;
                    let py = y as f64 * module_size;
                    rects.push_str(&format!(
                        r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#000"/>"##,
                        px, py, module_size, module_size
                    ));
                }
            }
        }
        
        // Add timing patterns
        for i in 0..module_count {
            let x = i as f64 * module_size;
            let y = 6.0 * module_size;
            if i % 2 == 0 && i != 6 {
                rects.push_str(&format!(
                    r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#000"/>"##,
                    x, y, module_size, module_size
                ));
                rects.push_str(&format!(
                    r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#000"/>"##,
                    y, x, module_size, module_size
                ));
            }
        }
        
        Ok(format!(
            r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">
    <rect width="100%" height="100%" fill="#fff"/>
    {}
</svg>"##,
            size, size, size, size, rects
        ))
    }
    
    /// Add QR finder pattern
    fn add_finder_pattern(&self, modules: &mut Vec<Vec<bool>>, x: usize, y: usize) {
        // Outer border
        for i in 0..7 {
            if x + i < modules.len() {
                modules[y][x + i] = true;
                modules[y + 6][x + i] = true;
            }
            if y + i < modules.len() {
                modules[y + i][x] = true;
                modules[y + i][x + 6] = true;
            }
        }
        
        // Inner square
        for i in 0..3 {
            for j in 0..3 {
                modules[y + 2 + i][x + 2 + j] = true;
            }
        }
    }
    
    /// Create deep link URL
    pub async fn create_deep_link(&self, code: &str) -> Result<String> {
        let config = self.config.read().await;
        Ok(format!("{}?code={}", config.deep_link_prefix, code))
    }
    
    /// Parse deep link
    pub fn parse_deep_link(&self, url: &str) -> Option<String> {
        // Extract code from deep link
        if url.starts_with("vantisweb://pair?code=") {
            Some(url[23..].to_string())
        } else {
            None
        }
    }
    
    /// Update configuration
    pub async fn update_config(&self, config: QRBridgeConfig) -> Result<()> {
        let mut current = self.config.write().await;
        *current = config;
        Ok(())
    }
    
    /// Get statistics
    pub async fn get_statistics(&self) -> Result<QRBridgeStatistics> {
        let codes = self.pairing_codes.write().await;
        
        let total = codes.len();
        let active = codes.iter().filter(|c| c.expires_at > Utc::now() && !c.used).count();
        let used = codes.iter().filter(|c| c.used).count();
        let expired = total - active - used;
        
        Ok(QRBridgeStatistics {
            total_codes: total as u64,
            active_codes: active as u64,
            used_codes: used as u64,
            expired_codes: expired as u64,
        })
    }
}

/// QR Bridge statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRBridgeStatistics {
    pub total_codes: u64,
    pub active_codes: u64,
    pub used_codes: u64,
    pub expired_codes: u64,
}

/// URL encoding module (simplified)
mod urlencoding {
    pub fn encode(s: &str) -> String {
        s.chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || "-_.~".contains(c) {
                    c.to_string()
                } else {
                    format!("%{:02X}", c as u8)
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_generate_pairing_qr() {
        let (tx, _rx) = broadcast::channel(16);
        let bridge = QRCodeBridge::new(tx).await.unwrap();
        
        let qr_data = bridge.generate_pairing_qr().await.unwrap();
        
        assert_eq!(qr_data.code.len(), 6);
        assert!(qr_data.qr_data_url.starts_with("data:image/svg+xml"));
    }
    
    #[tokio::test]
    async fn test_validate_pairing_code() {
        let (tx, _rx) = broadcast::channel(16);
        let bridge = QRCodeBridge::new(tx).await.unwrap();
        
        let qr_data = bridge.generate_pairing_qr().await.unwrap();
        
        let valid = bridge.validate_pairing_code(&qr_data.code).await.unwrap();
        assert!(valid.is_some());
        
        // Use the code
        let used = bridge.use_code(&qr_data.code).await.unwrap();
        assert!(used);
        
        // Should no longer be valid after use
        let valid = bridge.validate_pairing_code(&qr_data.code).await.unwrap();
        assert!(valid.is_none());
    }
    
    #[tokio::test]
    async fn test_deep_link() {
        let (tx, _rx) = broadcast::channel(16);
        let bridge = QRCodeBridge::new(tx).await.unwrap();
        
        let deep_link = bridge.create_deep_link("ABC123").await.unwrap();
        assert_eq!(deep_link, "vantisweb://pair?code=ABC123");
        
        let parsed = bridge.parse_deep_link(&deep_link);
        assert_eq!(parsed, Some("ABC123".to_string()));
    }
    
    #[test]
    fn test_qr_bridge_config_default() {
        let config = QRBridgeConfig::default();
        assert_eq!(config.code_length, 6);
        assert_eq!(config.expiration_seconds, 300);
        assert!(config.deep_link_enabled);
    }
}