/// # Configuration Module
/// 
/// Provides configuration management for the installer and update system.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{Platform, Architecture, PackageFormat, Version, InstallerError};

/// Installer configuration file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallerConfig {
    /// Application information
    pub app: AppInfo,
    /// Build configuration
    pub build: BuildConfigOptions,
    /// Update configuration
    pub updates: UpdateConfig,
    /// Code signing configuration
    pub signing: SigningConfig,
}

/// Application information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    /// Application name
    pub name: String,
    /// Application display name
    pub display_name: String,
    /// Application version
    pub version: Version,
    /// Application publisher
    pub publisher: String,
    /// Application description
    pub description: String,
    /// Application website
    pub website: String,
    /// Application icon path
    pub icon_path: String,
}

/// Build configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfigOptions {
    /// Source directory
    pub source_dir: String,
    /// Output directory
    pub output_dir: String,
    /// Build targets (platform + arch combinations)
    pub targets: Vec<BuildTarget>,
    /// Package formats to generate
    pub package_formats: Vec<PackageFormat>,
    /// Enable compression
    pub compress: bool,
    /// Compression level
    pub compression_level: u8,
    /// Enable silent installation
    pub silent_install: bool,
    /// Include debug symbols
    pub include_debug: bool,
}

/// Build target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildTarget {
    /// Platform
    pub platform: Platform,
    /// Architecture
    pub architecture: Architecture,
}

/// Update configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    /// Update server URL
    pub update_server: String,
    /// Check interval in hours
    pub check_interval_hours: u32,
    /// Auto-update enabled
    pub auto_update_enabled: bool,
    /// Beta channel enabled
    pub beta_channel_enabled: bool,
    /// Nightly channel enabled
    pub nightly_channel_enabled: bool,
    /// Current update channel
    pub current_channel: String,
    /// Delta updates enabled
    pub delta_updates_enabled: bool,
}

/// Code signing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningConfig {
    /// Enable code signing
    pub enabled: bool,
    /// Signing certificate path
    pub certificate_path: Option<String>,
    /// Signing key path
    pub key_path: Option<String>,
    /// Signature algorithm
    pub algorithm: String,
}

impl InstallerConfig {
    /// Load configuration from file
    pub fn load_from_file(path: &str) -> Result<Self, InstallerError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| InstallerError::ConfigError(format!("Failed to read config: {}", e)))?;

        let config: InstallerConfig = serde_yaml::from_str(&content)
            .map_err(|e| InstallerError::ConfigError(format!("Failed to parse config: {}", e)))?;

        Ok(config)
    }

    /// Save configuration to file
    pub fn save_to_file(&self, path: &str) -> Result<(), InstallerError> {
        let content = serde_yaml::to_string(self)
            .map_err(|e| InstallerError::ConfigError(format!("Failed to serialize config: {}", e)))?;

        std::fs::write(path, content)
            .map_err(|e| InstallerError::ConfigError(format!("Failed to write config: {}", e)))?;

        Ok(())
    }

    /// Get default configuration
    pub fn default() -> Self {
        Self {
            app: AppInfo {
                name: "vantisweb".to_string(),
                display_name: "VantisWeb".to_string(),
                version: Version::new(1, 0, 0),
                publisher: "VantisWeb Team".to_string(),
                description: "Next-generation web browser with Liquid Core Architecture".to_string(),
                website: "https://vantisweb.com".to_string(),
                icon_path: "resources/icon.png".to_string(),
            },
            build: BuildConfigOptions {
                source_dir: "./dist".to_string(),
                output_dir: "./build".to_string(),
                targets: vec![
                    BuildTarget {
                        platform: Platform::Windows,
                        architecture: Architecture::X86_64,
                    },
                    BuildTarget {
                        platform: Platform::MacOS,
                        architecture: Architecture::X86_64,
                    },
                    BuildTarget {
                        platform: Platform::Linux,
                        architecture: Architecture::X86_64,
                    },
                ],
                package_formats: vec![
                    PackageFormat::Nsis,
                    PackageFormat::Dmg,
                    PackageFormat::Deb,
                ],
                compress: true,
                compression_level: 6,
                silent_install: true,
                include_debug: false,
            },
            updates: UpdateConfig {
                update_server: "https://updates.vantisweb.com".to_string(),
                check_interval_hours: 24,
                auto_update_enabled: true,
                beta_channel_enabled: false,
                nightly_channel_enabled: false,
                current_channel: "stable".to_string(),
                delta_updates_enabled: true,
            },
            signing: SigningConfig {
                enabled: false,
                certificate_path: None,
                key_path: None,
                algorithm: "RSA4096".to_string(),
            },
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), InstallerError> {
        // Validate app info
        if self.app.name.is_empty() {
            return Err(InstallerError::ConfigError("App name is required".to_string()));
        }

        if self.app.display_name.is_empty() {
            return Err(InstallerError::ConfigError("App display name is required".to_string()));
        }

        // Validate build config
        if self.build.source_dir.is_empty() {
            return Err(InstallerError::ConfigError("Source directory is required".to_string()));
        }

        if self.build.output_dir.is_empty() {
            return Err(InstallerError::ConfigError("Output directory is required".to_string()));
        }

        if self.build.targets.is_empty() {
            return Err(InstallerError::ConfigError("At least one build target is required".to_string()));
        }

        if self.build.package_formats.is_empty() {
            return Err(InstallerError::ConfigError("At least one package format is required".to_string()));
        }

        // Validate update config
        if self.updates.update_server.is_empty() {
            return Err(InstallerError::ConfigError("Update server URL is required".to_string()));
        }

        // Validate signing config
        if self.signing.enabled {
            if self.signing.certificate_path.is_none() {
                return Err(InstallerError::ConfigError("Certificate path is required when signing is enabled".to_string()));
            }
        }

        Ok(())
    }
}

impl Default for InstallerConfig {
    fn default() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = InstallerConfig::default();
        assert_eq!(config.app.name, "vantisweb");
        assert_eq!(config.build.targets.len(), 3);
    }

    #[test]
    fn test_config_validation() {
        let config = InstallerConfig::default();
        let result = config.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_validation_empty_name() {
        let mut config = InstallerConfig::default();
        config.app.name = String::new();
        let result = config.validate();
        assert!(result.is_err());
    }
}