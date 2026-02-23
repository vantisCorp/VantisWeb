//! Settings Module
//! 
//! Application settings management:
//! - User preferences
//! - Security settings
//! - Privacy settings
//! - Performance settings

use anyhow::{Context, Result};
use log::{debug, info};
use serde::{Deserialize, Serialize};

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub general: GeneralSettings,
    pub security: SecuritySettings,
    pub privacy: PrivacySettings,
    pub performance: PerformanceSettings,
    pub ui: UISettings,
}

/// General settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    pub startup_page: String,
    pub search_engine: String,
    pub language: String,
    pub download_folder: String,
    pub check_updates: bool,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            startup_page: "https://vantis.ai/home".to_string(),
            search_engine: "https://google.com/search?q=".to_string(),
            language: "en".to_string(),
            download_folder: "downloads".to_string(),
            check_updates: true,
        }
    }
}

/// Security settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    pub sandbox_enabled: bool,
    pub tracker_blocking: bool,
    pub phishing_protection: bool,
    pub malware_scanning: bool,
    pub secure_dns: String,
    pub https_only: bool,
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            sandbox_enabled: true,
            tracker_blocking: true,
            phishing_protection: true,
            malware_scanning: true,
            secure_dns: "auto".to_string(),
            https_only: true,
        }
    }
}

/// Privacy settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    pub do_not_track: bool,
    pub block_third_party_cookies: bool,
    pub clear_history_on_exit: bool,
    pub clear_cookies_on_exit: bool,
    pub clear_cache_on_exit: bool,
    pub telemetry: bool,
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            do_not_track: true,
            block_third_party_cookies: true,
            clear_history_on_exit: false,
            clear_cookies_on_exit: false,
            clear_cache_on_exit: false,
            telemetry: false,
        }
    }
}

/// Performance settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    pub hardware_acceleration: bool,
    pub background_tabs_limit: u8,
    pub preloading_enabled: bool,
    pub memory_saver: bool,
    pub cpu_saver: bool,
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            hardware_acceleration: true,
            background_tabs_limit: 4,
            preloading_enabled: true,
            memory_saver: false,
            cpu_saver: false,
        }
    }
}

/// UI settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UISettings {
    pub theme: String,
    pub font_size: u8,
    pub zoom_level: u8,
    pub animations: bool,
    pub compact_mode: bool,
    pub show_bookmarks_bar: bool,
}

impl Default for UISettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            font_size: 16,
            zoom_level: 100,
            animations: true,
            compact_mode: false,
            show_bookmarks_bar: true,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            general: GeneralSettings::default(),
            security: SecuritySettings::default(),
            privacy: PrivacySettings::default(),
            performance: PerformanceSettings::default(),
            ui: UISettings::default(),
        }
    }
}

/// Settings manager
pub struct SettingsManager {
    settings: Settings,
    settings_file: String,
}

impl SettingsManager {
    /// Create a new settings manager
    pub fn new(settings_file: String) -> Result<Self> {
        info!("Initializing Settings Manager...");
        
        let settings = Self::load_settings(&settings_file)?;
        
        Ok(Self {
            settings,
            settings_file,
        })
    }
    
    /// Load settings from file
    fn load_settings(settings_file: &str) -> Result<Settings> {
        if std::path::Path::new(settings_file).exists() {
            info!("Loading settings from: {}", settings_file);
            
            let settings_toml = std::fs::read_to_string(settings_file)
                .context("Failed to read settings file")?;
            
            let settings: Settings = toml::from_str(&settings_toml)
                .context("Failed to parse settings")?;
            
            Ok(settings)
        } else {
            info!("Settings file not found, using defaults");
            Ok(Settings::default())
        }
    }
    
    /// Save settings to file
    pub fn save(&self) -> Result<()> {
        info!("Saving settings to: {}", self.settings_file);
        
        let settings_toml = toml::to_string_pretty(&self.settings)
            .context("Failed to serialize settings")?;
        
        std::fs::write(&self.settings_file, settings_toml)
            .context("Failed to write settings file")?;
        
        debug!("Settings saved successfully");
        
        Ok(())
    }
    
    /// Get settings
    pub fn get_settings(&self) -> &Settings {
        &self.settings
    }
    
    /// Update settings
    pub fn update_settings(&mut self, settings: Settings) {
        self.settings = settings;
    }
    
    /// Reset to defaults
    pub fn reset_to_defaults(&mut self) {
        info!("Resetting settings to defaults");
        self.settings = Settings::default();
    }
    
    /// Get general settings
    pub fn general(&self) -> &GeneralSettings {
        &self.settings.general
    }
    
    /// Get security settings
    pub fn security(&self) -> &SecuritySettings {
        &self.settings.security
    }
    
    /// Get privacy settings
    pub fn privacy(&self) -> &PrivacySettings {
        &self.settings.privacy
    }
    
    /// Get performance settings
    pub fn performance(&self) -> &PerformanceSettings {
        &self.settings.performance
    }
    
    /// Get UI settings
    pub fn ui(&self) -> &UISettings {
        &self.settings.ui
    }
}