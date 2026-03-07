/// # Cross-Platform Installer and Update System Module
/// 
/// This module provides comprehensive installer creation and update functionality
/// for the VantisWeb browser across Windows, macOS, and Linux platforms.
//!
//! ## Features
//!
//! - **Installer Creation**: Generate installers for all major platforms
//! - **Auto-Update**: Automatic checking and downloading of updates
//! - **Delta Updates**: Efficient differential updates
//! - **Version Management**: Semantic versioning and rollback
//! - **Platform-Specific**: Optimized for each OS
//! - **Code Signing**: Digital signature support
//! - **Silent Install**: Unattended installation options

pub mod installer;
pub mod updater;
pub mod platform;
pub mod signature;
pub mod config;

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur in installer/update operations
#[derive(Error, Debug)]
pub enum InstallerError {
    #[error("Build error: {0}")]
    BuildError(String),
    #[error("Package error: {0}")]
    PackageError(String),
    #[error("Update check failed: {0}")]
    UpdateCheckFailed(String),
    #[error("Download failed: {0}")]
    DownloadFailed(String),
    #[error("Signature verification failed: {0}")]
    SignatureError(String),
    #[error("Platform not supported: {0}")]
    PlatformNotSupported(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Result type for installer operations
pub type Result<T> = std::result::Result<T, InstallerError>;

/// Platform identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Platform {
    Windows,
    MacOS,
    Linux,
}

/// Architecture identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Architecture {
    X86_64,
    Aarch64,
    Arm,
}

/// Package format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageFormat {
    /// Windows Installer
    Msi,
    /// Nullsoft Scriptable Install System (Windows)
    Nsis,
    /// macOS Disk Image
    Dmg,
    /// macOS Package
    Pkg,
    /// Linux Debian Package
    Deb,
    /// Linux RPM Package
    Rpm,
    /// Linux AppImage
    AppImage,
}

/// Version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    /// Major version
    pub major: u32,
    /// Minor version
    pub minor: u32,
    /// Patch version
    pub patch: u32,
    /// Pre-release identifier
    pub pre: Option<String>,
    /// Build metadata
    pub build: Option<String>,
}

impl Version {
    /// Create a new version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            pre: None,
            build: None,
        }
    }

    /// Parse version from string
    pub fn parse(version_str: &str) -> Result<Self> {
        let parts: Vec<&str> = version_str.split('-').collect();
        let main = parts[0];
        
        let version_parts: Vec<&str> = main.split('.').collect();
        if version_parts.len() != 3 {
            return Err(InstallerError::ConfigError(
                "Invalid version format".to_string()
            ));
        }

        let major = version_parts[0].parse()
            .map_err(|_| InstallerError::ConfigError("Invalid major version".to_string()))?;
        let minor = version_parts[1].parse()
            .map_err(|_| InstallerError::ConfigError("Invalid minor version".to_string()))?;
        let patch = version_parts[2].parse()
            .map_err(|_| InstallerError::ConfigError("Invalid patch version".to_string()))?;

        let pre = if parts.len() > 1 {
            Some(parts[1].to_string())
        } else {
            None
        };

        Ok(Self {
            major,
            minor,
            patch,
            pre,
            build: None,
        })
    }

    /// Convert to string
    pub fn to_string(&self) -> String {
        let mut s = format!("{}.{}.{}", self.major, self.minor, self.patch);
        if let Some(ref pre) = self.pre {
            s.push_str("-");
            s.push_str(pre);
        }
        s
    }

    /// Compare versions
    pub fn compare(&self, other: &Version) -> std::cmp::Ordering {
        match self.major.cmp(&other.major) {
            std::cmp::Ordering::Equal => {
                match self.minor.cmp(&other.minor) {
                    std::cmp::Ordering::Equal => {
                        self.patch.cmp(&other.patch)
                    }
                    other => other,
                }
            }
            other => other,
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    /// New version
    pub version: Version,
    /// Release date
    pub release_date: chrono::DateTime<chrono::Utc>,
    /// Download URL
    pub download_url: String,
    /// File size
    pub file_size: u64,
    /// Checksum (SHA256)
    pub checksum: String,
    /// Release notes
    pub release_notes: String,
    /// Signature URL
    pub signature_url: Option<String>,
    /// Is this update mandatory
    pub mandatory: bool,
    /// Minimum compatible version
    pub min_compatible_version: Option<Version>,
}

/// Build configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    /// Application name
    pub app_name: String,
    /// Application version
    pub app_version: Version,
    /// Target platform
    pub platform: Platform,
    /// Target architecture
    pub architecture: Architecture,
    /// Package format
    pub package_format: PackageFormat,
    /// Output directory
    pub output_dir: String,
    /// Source directory
    pub source_dir: String,
    /// Icon path
    pub icon_path: Option<String>,
    /// Enable code signing
    pub sign_package: bool,
    /// Enable compression
    pub compress: bool,
    /// Silent installation support
    pub silent_install: bool,
}

/// Installer and update manager
pub struct InstallerManager {
    /// Build configuration
    config: Arc<RwLock<BuildConfig>>,
    /// Update checker
    updater: Arc<RwLock<updater::Updater>>,
    /// Platform-specific installer
    platform_installer: Arc<RwLock<platform::PlatformInstaller>>,
}

impl InstallerManager {
    /// Create a new installer manager
    pub async fn new(config: BuildConfig) -> Result<Self> {
        let updater = Arc::new(RwLock::new(updater::Updater::new(
            config.app_version.clone(),
            "https://updates.vantisweb.com".to_string(),
        ).await?));

        let platform_installer = Arc::new(RwLock::new(
            platform::PlatformInstaller::new(config.platform, config.architecture)
        ));

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            updater,
            platform_installer,
        })
    }

    /// Build installer package
    pub async fn build_installer(&self) -> Result<String> {
        let config = self.config.read().await;
        let mut installer = self.platform_installer.write().await;
        
        installer.build(&config).await
    }

    /// Check for updates
    pub async fn check_for_updates(&self) -> Result<Option<UpdateInfo>> {
        let updater = self.updater.read().await;
        updater.check_for_updates().await
    }

    /// Download update
    pub async fn download_update(&self, update_info: &UpdateInfo, output_path: &str) -> Result<()> {
        let mut updater = self.updater.write().await;
        updater.download_update(update_info, output_path).await
    }

    /// Install update
    pub async fn install_update(&self, update_path: &str) -> Result<()> {
        let mut updater = self.updater.write().await;
        updater.install_update(update_path).await
    }

    /// Verify package signature
    pub async fn verify_signature(&self, package_path: &str, signature_path: &str) -> Result<bool> {
        let config = self.config.read().await;
        let verifier = signature::SignatureVerifier::new(&config);
        verifier.verify(package_path, signature_path).await
    }

    /// Sign package
    pub async fn sign_package(&self, package_path: &str) -> Result<String> {
        let config = self.config.read().await;
        let signer = signature::PackageSigner::new(&config);
        signer.sign(package_path).await
    }

    /// Create delta update
    pub async fn create_delta(&self, old_version: &Version, new_version: &Version) -> Result<Vec<u8>> {
        let mut updater = self.updater.write().await;
        updater.create_delta(old_version, new_version).await
    }

    /// Apply delta update
    pub async fn apply_delta(&self, delta_data: Vec<u8>, old_version: &Version) -> Result<()> {
        let mut updater = self.updater.write().await;
        updater.apply_delta(delta_data, old_version).await
    }

    /// Rollback to previous version
    pub async fn rollback(&self) -> Result<()> {
        let mut updater = self.updater.write().await;
        updater.rollback().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_creation() {
        let v = Version::new(1, 2, 3);
        assert_eq!(v.major, 1);
        assert_eq!(v.to_string(), "1.2.3");
    }

    #[test]
    fn test_version_parse() {
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_version_compare() {
        let v1 = Version::new(1, 2, 3);
        let v2 = Version::new(1, 2, 4);
        
        assert_eq!(v1.compare(&v2), std::cmp::Ordering::Less);
    }
}