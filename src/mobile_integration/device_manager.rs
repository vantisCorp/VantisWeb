//! Mobile Device Manager
//! 
//! Manages paired mobile devices, discovery, and device lifecycle.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{
    MobileDevice, DeviceType, DeviceCapabilities, SyncSettings,
    DevicePairingInfo, MobileEvent, MobileConfig,
};

/// Device manager for mobile devices
pub struct MobileDeviceManager {
    /// Paired devices
    devices: RwLock<HashMap<String, MobileDevice>>,
    /// Device tokens for authentication
    device_tokens: RwLock<HashMap<String, String>>,
    /// Event sender
    event_sender: broadcast::Sender<MobileEvent>,
    /// Configuration
    config: RwLock<MobileConfig>,
    /// Discovery active flag
    discovery_active: RwLock<bool>,
}

impl MobileDeviceManager {
    /// Create a new device manager
    pub async fn new(
        config: MobileConfig,
        event_sender: broadcast::Sender<MobileEvent>,
    ) -> Result<Self> {
        Ok(Self {
            devices: RwLock::new(HashMap::new()),
            device_tokens: RwLock::new(HashMap::new()),
            event_sender,
            config: RwLock::new(config),
            discovery_active: RwLock::new(false),
        })
    }
    
    /// Start device discovery
    pub async fn start_discovery(&self) -> Result<()> {
        let mut active = self.discovery_active.write().await;
        *active = true;
        
        tracing::info!("Mobile device discovery started");
        Ok(())
    }
    
    /// Stop device discovery
    pub async fn stop_discovery(&self) -> Result<()> {
        let mut active = self.discovery_active.write().await;
        *active = false;
        
        tracing::info!("Mobile device discovery stopped");
        Ok(())
    }
    
    /// Check if discovery is active
    pub async fn is_discovery_active(&self) -> bool {
        *self.discovery_active.read().await
    }
    
    /// Pair a new device
    pub async fn pair_device(&self, info: DevicePairingInfo) -> Result<MobileDevice> {
        let config = self.config.read().await;
        
        // Check device limit
        let devices = self.devices.read().await;
        if devices.len() >= config.max_devices {
            return Err(anyhow::anyhow!("Maximum number of devices reached"));
        }
        drop(devices);
        
        // Generate device ID and token
        let device_id = Uuid::new_v4().to_string();
        let token = self.generate_device_token();
        
        // Create device
        let device = MobileDevice {
            id: device_id.clone(),
            name: info.name,
            device_type: info.device_type,
            model: info.model,
            os_version: info.os_version,
            app_version: info.app_version,
            is_connected: true,
            last_seen: Utc::now(),
            paired_at: Utc::now(),
            capabilities: info.capabilities,
            push_token: info.push_token,
            sync_settings: SyncSettings::default(),
        };
        
        // Store device
        let mut devices = self.devices.write().await;
        devices.insert(device_id.clone(), device.clone());
        
        // Store token
        let mut tokens = self.device_tokens.write().await;
        tokens.insert(device_id.clone(), token);
        
        // Emit event
        let _ = self.event_sender.send(MobileEvent::DevicePaired(device_id.clone()));
        
        tracing::info!("Device paired: {} ({})", device.name, device.id);
        Ok(device)
    }
    
    /// Unpair a device
    pub async fn unpair_device(&self, device_id: &str) -> Result<()> {
        let mut devices = self.devices.write().await;
        
        if let Some(device) = devices.remove(device_id) {
            // Remove token
            let mut tokens = self.device_tokens.write().await;
            tokens.remove(device_id);
            
            // Emit event
            let _ = self.event_sender.send(MobileEvent::DeviceUnpaired(device_id.to_string()));
            
            tracing::info!("Device unpaired: {} ({})", device.name, device.id);
        }
        
        Ok(())
    }
    
    /// Get paired devices
    pub async fn get_paired_devices(&self) -> Result<Vec<MobileDevice>> {
        let devices = self.devices.read().await;
        Ok(devices.values().cloned().collect())
    }
    
    /// Get device by ID
    pub async fn get_device(&self, device_id: &str) -> Result<Option<MobileDevice>> {
        let devices = self.devices.read().await;
        Ok(devices.get(device_id).cloned())
    }
    
    /// Check if device exists
    pub async fn device_exists(&self, device_id: &str) -> bool {
        let devices = self.devices.read().await;
        devices.contains_key(device_id)
    }
    
    /// Validate device token
    pub async fn validate_token(&self, device_id: &str, token: &str) -> bool {
        let tokens = self.device_tokens.read().await;
        tokens.get(device_id).map(|t| t == token).unwrap_or(false)
    }
    
    /// Regenerate device token
    pub async fn regenerate_token(&self, device_id: &str) -> Result<String> {
        let token = self.generate_device_token();
        
        let mut tokens = self.device_tokens.write().await;
        tokens.insert(device_id.to_string(), token.clone());
        
        tracing::info!("Token regenerated for device: {}", device_id);
        Ok(token)
    }
    
    /// Update device connection status
    pub async fn update_connection_status(&self, device_id: &str, connected: bool) -> Result<()> {
        let mut devices = self.devices.write().await;
        
        if let Some(device) = devices.get_mut(device_id) {
            let was_connected = device.is_connected;
            device.is_connected = connected;
            device.last_seen = Utc::now();
            
            if was_connected != connected {
                if connected {
                    let _ = self.event_sender.send(MobileEvent::DeviceConnected(device_id.to_string()));
                } else {
                    let _ = self.event_sender.send(MobileEvent::DeviceDisconnected(device_id.to_string()));
                }
            }
        }
        
        Ok(())
    }
    
    /// Update device last seen
    pub async fn update_last_seen(&self, device_id: &str) -> Result<()> {
        let mut devices = self.devices.write().await;
        
        if let Some(device) = devices.get_mut(device_id) {
            device.last_seen = Utc::now();
        }
        
        Ok(())
    }
    
    /// Update device push token
    pub async fn update_push_token(&self, device_id: &str, push_token: String) -> Result<()> {
        let mut devices = self.devices.write().await;
        
        if let Some(device) = devices.get_mut(device_id) {
            device.push_token = Some(push_token);
        }
        
        Ok(())
    }
    
    /// Update device sync settings
    pub async fn update_sync_settings(
        &self,
        device_id: &str,
        settings: SyncSettings,
    ) -> Result<()> {
        let mut devices = self.devices.write().await;
        
        if let Some(device) = devices.get_mut(device_id) {
            device.sync_settings = settings;
        }
        
        Ok(())
    }
    
    /// Get device sync settings
    pub async fn get_sync_settings(&self, device_id: &str) -> Result<Option<SyncSettings>> {
        let devices = self.devices.read().await;
        Ok(devices.get(device_id).map(|d| d.sync_settings.clone()))
    }
    
    /// Update device info
    pub async fn update_device_info(
        &self,
        device_id: &str,
        name: Option<String>,
        os_version: Option<String>,
        app_version: Option<String>,
        capabilities: Option<DeviceCapabilities>,
    ) -> Result<()> {
        let mut devices = self.devices.write().await;
        
        if let Some(device) = devices.get_mut(device_id) {
            if let Some(n) = name {
                device.name = n;
            }
            if let Some(v) = os_version {
                device.os_version = v;
            }
            if let Some(v) = app_version {
                device.app_version = v;
            }
            if let Some(c) = capabilities {
                device.capabilities = c;
            }
            device.last_seen = Utc::now();
        }
        
        Ok(())
    }
    
    /// Get connected devices
    pub async fn get_connected_devices(&self) -> Result<Vec<MobileDevice>> {
        let devices = self.devices.read().await;
        Ok(devices.values().filter(|d| d.is_connected).cloned().collect())
    }
    
    /// Get devices by type
    pub async fn get_devices_by_type(&self, device_type: DeviceType) -> Result<Vec<MobileDevice>> {
        let devices = self.devices.read().await;
        Ok(devices.values().filter(|d| d.device_type == device_type).cloned().collect())
    }
    
    /// Get devices with push capability
    pub async fn get_push_capable_devices(&self) -> Result<Vec<MobileDevice>> {
        let devices = self.devices.read().await;
        Ok(devices.values()
            .filter(|d| d.capabilities.push_notifications && d.push_token.is_some())
            .cloned()
            .collect())
    }
    
    /// Get devices with remote control capability
    pub async fn get_remote_capable_devices(&self) -> Result<Vec<MobileDevice>> {
        let devices = self.devices.read().await;
        Ok(devices.values()
            .filter(|d| d.capabilities.remote_control)
            .cloned()
            .collect())
    }
    
    /// Clean up inactive devices
    pub async fn cleanup_inactive_devices(&self, max_inactive_days: u64) -> Result<Vec<String>> {
        let mut devices = self.devices.write().await;
        let mut tokens = self.device_tokens.write().await;
        let mut removed = Vec::new();
        
        let cutoff = Utc::now() - chrono::Duration::days(max_inactive_days as i64);
        
        devices.retain(|id, device| {
            let should_remove = device.last_seen < cutoff;
            if should_remove {
                tokens.remove(id);
                removed.push(id.clone());
                tracing::info!("Removed inactive device: {}", id);
            }
            !should_remove
        });
        
        Ok(removed)
    }
    
    /// Update configuration
    pub async fn update_config(&self, config: MobileConfig) -> Result<()> {
        let mut current = self.config.write().await;
        *current = config;
        Ok(())
    }
    
    /// Get device count
    pub async fn device_count(&self) -> usize {
        self.devices.read().await.len()
    }
    
    /// Check if device limit reached
    pub async fn is_device_limit_reached(&self) -> bool {
        let config = self.config.read().await;
        let devices = self.devices.read().await;
        devices.len() >= config.max_devices
    }
    
    /// Generate device token
    fn generate_device_token(&self) -> String {
        // Generate a secure random token
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        
        format!("mobile_{}_{}", Uuid::new_v4(), timestamp)
    }
    
    /// Get device statistics
    pub async fn get_statistics(&self) -> Result<DeviceStatistics> {
        let devices = self.devices.read().await;
        
        let total = devices.len();
        let connected = devices.values().filter(|d| d.is_connected).count();
        let ios_count = devices.values().filter(|d| d.device_type == DeviceType::IOS).count();
        let android_count = devices.values().filter(|d| d.device_type == DeviceType::Android).count();
        
        let push_enabled = devices.values()
            .filter(|d| d.capabilities.push_notifications && d.push_token.is_some())
            .count();
        
        Ok(DeviceStatistics {
            total_devices: total,
            connected_devices: connected,
            ios_devices: ios_count,
            android_devices: android_count,
            push_enabled_devices: push_enabled,
        })
    }
}

/// Device statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeviceStatistics {
    /// Total paired devices
    pub total_devices: usize,
    /// Currently connected
    pub connected_devices: usize,
    /// iOS devices
    pub ios_devices: usize,
    /// Android devices
    pub android_devices: usize,
    /// Push enabled devices
    pub push_enabled_devices: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_config() -> MobileConfig {
        MobileConfig::default()
    }
    
    fn create_test_pairing_info() -> DevicePairingInfo {
        DevicePairingInfo {
            name: "Test iPhone".to_string(),
            device_type: DeviceType::IOS,
            model: "iPhone 15 Pro".to_string(),
            os_version: "iOS 17.0".to_string(),
            app_version: "1.0.0".to_string(),
            pairing_code: "123456".to_string(),
            push_token: Some("test-push-token".to_string()),
            capabilities: DeviceCapabilities::default(),
        }
    }
    
    #[tokio::test]
    async fn test_pair_device() {
        let (tx, _rx) = broadcast::channel(16);
        let manager = MobileDeviceManager::new(create_test_config(), tx).await.unwrap();
        
        let device = manager.pair_device(create_test_pairing_info()).await.unwrap();
        
        assert_eq!(device.name, "Test iPhone");
        assert!(device.is_connected);
    }
    
    #[tokio::test]
    async fn test_get_paired_devices() {
        let (tx, _rx) = broadcast::channel(16);
        let manager = MobileDeviceManager::new(create_test_config(), tx).await.unwrap();
        
        manager.pair_device(create_test_pairing_info()).await.unwrap();
        
        let devices = manager.get_paired_devices().await.unwrap();
        assert_eq!(devices.len(), 1);
    }
    
    #[tokio::test]
    async fn test_unpair_device() {
        let (tx, _rx) = broadcast::channel(16);
        let manager = MobileDeviceManager::new(create_test_config(), tx).await.unwrap();
        
        let device = manager.pair_device(create_test_pairing_info()).await.unwrap();
        manager.unpair_device(&device.id).await.unwrap();
        
        let devices = manager.get_paired_devices().await.unwrap();
        assert!(devices.is_empty());
    }
    
    #[tokio::test]
    async fn test_update_connection_status() {
        let (tx, _rx) = broadcast::channel(16);
        let manager = MobileDeviceManager::new(create_test_config(), tx).await.unwrap();
        
        let device = manager.pair_device(create_test_pairing_info()).await.unwrap();
        
        manager.update_connection_status(&device.id, false).await.unwrap();
        
        let updated = manager.get_device(&device.id).await.unwrap().unwrap();
        assert!(!updated.is_connected);
    }
    
    #[tokio::test]
    async fn test_device_limit() {
        let mut config = MobileConfig::default();
        config.max_devices = 1;
        
        let (tx, _rx) = broadcast::channel(16);
        let manager = MobileDeviceManager::new(config, tx).await.unwrap();
        
        // First device should succeed
        manager.pair_device(create_test_pairing_info()).await.unwrap();
        
        // Second device should fail
        let result = manager.pair_device(create_test_pairing_info()).await;
        assert!(result.is_err());
    }
}