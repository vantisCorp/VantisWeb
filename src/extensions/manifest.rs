//! Extension Manifest
//!
//! Handles parsing and validation of extension manifest files.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Extension manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Extension name
    pub name: String,
    /// Extension version
    pub version: String,
    /// Extension description
    pub description: String,
    /// Extension author
    pub author: String,
    /// Extension manifest version
    pub manifest_version: u32,
    /// Extension permissions
    #[serde(default)]
    pub permissions: Vec<String>,
    /// Background script
    pub background: Option<BackgroundScript>,
    /// Content scripts
    #[serde(default)]
    pub content_scripts: Vec<ContentScript>,
    /// Popup
    pub popup: Option<Popup>,
    /// Icons
    #[serde(default)]
    pub icons: Icons,
    /// Homepage URL
    pub homepage_url: Option<String>,
    /// Minimum browser version
    pub minimum_browser_version: Option<String>,
}

/// Background script configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundScript {
    /// Script file path
    pub script: String,
    /// Whether the script is persistent
    #[serde(default)]
    pub persistent: bool,
}

/// Content script configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentScript {
    /// Script file paths
    pub js: Vec<String>,
    /// CSS file paths
    #[serde(default)]
    pub css: Vec<String>,
    /// Match patterns
    pub matches: Vec<String>,
    /// Whether to run at document start
    #[serde(default)]
    pub run_at: String,
}

/// Popup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Popup {
    /// Popup HTML file path
    pub html: String,
    /// Default popup title
    pub default_title: Option<String>,
    /// Popup icon
    pub default_icon: Option<String>,
}

/// Icons configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Icons {
    /// 16x16 icon
    #[serde(rename = "16")]
    pub icon_16: Option<String>,
    /// 32x32 icon
    #[serde(rename = "32")]
    pub icon_32: Option<String>,
    /// 48x48 icon
    #[serde(rename = "48")]
    pub icon_48: Option<String>,
    /// 128x128 icon
    #[serde(rename = "128")]
    pub icon_128: Option<String>,
}

/// Manifest parser
pub struct ManifestParser;

impl ManifestParser {
    /// Parses a manifest file
    pub fn parse<P: AsRef<Path>>(path: P) -> Result<Manifest> {
        let path = path.as_ref();
        
        // Read manifest file
        let content = fs::read_to_string(path)
            .map_err(|e| anyhow!("Failed to read manifest file: {}", e))?;
        
        // Parse JSON
        let manifest: Manifest = serde_json::from_str(&content)
            .map_err(|e| anyhow!("Failed to parse manifest JSON: {}", e))?;
        
        // Validate manifest
        Self::validate(&manifest)?;
        
        Ok(manifest)
    }
    
    /// Validates a manifest
    fn validate(manifest: &Manifest) -> Result<()> {
        // Check required fields
        if manifest.name.is_empty() {
            return Err(anyhow!("Manifest name cannot be empty"));
        }
        
        if manifest.version.is_empty() {
            return Err(anyhow!("Manifest version cannot be empty"));
        }
        
        if manifest.author.is_empty() {
            return Err(anyhow!("Manifest author cannot be empty"));
        }
        
        // Validate manifest version
        if manifest.manifest_version != 2 {
            return Err(anyhow!("Only manifest version 2 is supported"));
        }
        
        // Validate version format (semver)
        Self::validate_version(&manifest.version)?;
        
        // Validate permissions
        for permission in &manifest.permissions {
            Self::validate_permission(permission)?;
        }
        
        // Validate content scripts
        for content_script in &manifest.content_scripts {
            if content_script.js.is_empty() && content_script.css.is_empty() {
                return Err(anyhow!("Content script must have at least one js or css file"));
            }
            
            if content_script.matches.is_empty() {
                return Err(anyhow!("Content script must have at least one match pattern"));
            }
        }
        
        Ok(())
    }
    
    /// Validates a version string (semver)
    fn validate_version(version: &str) -> Result<()> {
        let parts: Vec<&str> = version.split('.').collect();
        
        if parts.len() < 2 || parts.len() > 3 {
            return Err(anyhow!("Version must be in semver format (e.g., 1.0.0)"));
        }
        
        for part in parts {
            part.parse::<u32>()
                .map_err(|_| anyhow!("Version part '{}' is not a valid number", part))?;
        }
        
        Ok(())
    }
    
    /// Validates a permission
    fn validate_permission(permission: &str) -> Result<()> {
        let valid_permissions = vec![
            "storage",
            "tabs",
            "activeTab",
            "cookies",
            "webRequest",
            "webNavigation",
            "notifications",
            "alarms",
            "bookmarks",
            "history",
            "downloads",
            "clipboardRead",
            "clipboardWrite",
            "geolocation",
            "idle",
        ];
        
        if !valid_permissions.contains(&permission) {
            return Err(anyhow!("Invalid permission: {}", permission));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_manifest() {
        let manifest_json = r#"{
            "name": "Test Extension",
            "version": "1.0.0",
            "description": "A test extension",
            "author": "Test Author",
            "manifest_version": 2,
            "permissions": ["storage"],
            "background": {
                "script": "background.js",
                "persistent": false
            },
            "content_scripts": [
                {
                    "js": ["content.js"],
                    "matches": ["<all_urls>"],
                    "run_at": "document_end"
                }
            ],
            "popup": {
                "html": "popup.html",
                "default_title": "Test Popup"
            },
            "icons": {
                "16": "icon16.png",
                "48": "icon48.png",
                "128": "icon128.png"
            }
        }"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(manifest_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let manifest = ManifestParser::parse(temp_file.path()).unwrap();
        
        assert_eq!(manifest.name, "Test Extension");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.manifest_version, 2);
        assert_eq!(manifest.permissions.len(), 1);
        assert!(manifest.background.is_some());
        assert_eq!(manifest.content_scripts.len(), 1);
        assert!(manifest.popup.is_some());
    }

    #[test]
    fn test_validate_version() {
        assert!(ManifestParser::validate_version("1.0.0").is_ok());
        assert!(ManifestParser::validate_version("1.0").is_ok());
        assert!(ManifestParser::validate_version("1").is_err());
        assert!(ManifestParser::validate_version("1.0.0.0").is_err());
        assert!(ManifestParser::validate_version("a.b.c").is_err());
    }

    #[test]
    fn test_validate_permission() {
        assert!(ManifestParser::validate_permission("storage").is_ok());
        assert!(ManifestParser::validate_permission("tabs").is_ok());
        assert!(ManifestParser::validate_permission("invalid").is_err());
    }

    #[test]
    fn test_parse_invalid_manifest() {
        let manifest_json = r#"{
            "name": "",
            "version": "1.0.0",
            "description": "A test extension",
            "author": "Test Author",
            "manifest_version": 2
        }"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(manifest_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let result = ManifestParser::parse(temp_file.path());
        assert!(result.is_err());
    }
}