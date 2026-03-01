//! Vantis Configuration Manager
//! 
//! Centralized configuration management:
//! - User preferences
//! - System settings
//! - Module configurations
//! - Profile management

use anyhow::{Context, Result};
use directories::ProjectDirs;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Vantis Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VantisConfig {
    pub version: String,
    pub general: GeneralConfig,
    pub ui: UIConfig,
    pub security: SecurityConfig,
    pub privacy: PrivacyConfig,
    pub performance: PerformanceConfig,
}

impl Default for VantisConfig {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            panic!("Failed to create default configuration")
        })
    }
}

impl VantisConfig {
    /// Create a new default configuration
    pub fn new() -> Result<Self> {
        info!("Creating Vantis configuration...");
        
        let _project_dirs = ProjectDirs::from("com", "vantis", "vantisweb")
            .context("Failed to get project directories")?;
        
        let general = GeneralConfig {
            language: "en".to_string(),
            theme: "dark".to_string(),
            auto_update: true,
            check_updates: true,
        };
        
        let ui = UIConfig {
            zoom_level: 100,
            font_size: 16,
            animations: true,
            hardware_acceleration: true,
        };
        
        let security = SecurityConfig {
            sandbox_enabled: true,
            tracker_blocking: true,
            phishing_protection: true,
            secure_dns: "auto".to_string(),
        };
        
        let privacy = PrivacyConfig {
            do_not_track: true,
            block_third_party_cookies: true,
            clear_browsing_data_on_exit: false,
            telemetry: false,
        };
        
        let performance = PerformanceConfig {
            hardware_acceleration: true,
            background_tabs_limit: 4,
            preloading_enabled: true,
        };
        
        Ok(Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            general,
            ui,
            security,
            privacy,
            performance,
        })
    }
    
    /// Get data directory
    pub fn get_data_dir(&self) -> PathBuf {
        let project_dirs = ProjectDirs::from("com", "vantis", "vantisweb")
            .expect("Failed to get project directories");
        
        project_dirs.data_local_dir().to_path_buf()
    }
    
    /// Get cache directory
    pub fn get_cache_dir(&self) -> PathBuf {
        let project_dirs = ProjectDirs::from("com", "vantis", "vantisweb")
            .expect("Failed to get project directories");
        
        project_dirs.cache_dir().to_path_buf()
    }
    
    /// Get config directory
    pub fn get_config_dir(&self) -> PathBuf {
        let project_dirs = ProjectDirs::from("com", "vantis", "vantisweb")
            .expect("Failed to get project directories");
        
        project_dirs.config_dir().to_path_buf()
    }
    
    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_dir = self.get_config_dir();
        std::fs::create_dir_all(&config_dir)?;
        
        let config_file = config_dir.join("config.toml");
        let config_toml = toml::to_string_pretty(self)
            .context("Failed to serialize configuration")?;
        
        std::fs::write(&config_file, config_toml)
            .context("Failed to write configuration file")?;
        
        debug!("Configuration saved to: {:?}", config_file);
        
        Ok(())
    }
    
    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let config_dir = ProjectDirs::from("com", "vantis", "vantisweb")
            .expect("Failed to get project directories")
            .config_dir()
            .to_path_buf();
        
        let config_file = config_dir.join("config.toml");
        
        if config_file.exists() {
            info!("Loading configuration from: {:?}", config_file);
            
            let config_toml = std::fs::read_to_string(&config_file)
                .context("Failed to read configuration file")?;
            
            let config: VantisConfig = toml::from_str(&config_toml)
                .context("Failed to parse configuration")?;
            
            Ok(config)
        } else {
            info!("Configuration file not found, using defaults");
            Ok(Self::default())
        }
    }
}

/// General configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub language: String,
    pub theme: String,
    pub auto_update: bool,
    pub check_updates: bool,
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    pub zoom_level: u8,
    pub font_size: u8,
    pub animations: bool,
    pub hardware_acceleration: bool,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub sandbox_enabled: bool,
    pub tracker_blocking: bool,
    pub phishing_protection: bool,
    pub secure_dns: String,
}

/// Privacy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    pub do_not_track: bool,
    pub block_third_party_cookies: bool,
    pub clear_browsing_data_on_exit: bool,
    pub telemetry: bool,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub hardware_acceleration: bool,
    pub background_tabs_limit: u8,
    pub preloading_enabled: bool,
}