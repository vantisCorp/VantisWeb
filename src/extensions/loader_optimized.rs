//! Extension Loader (Optimized Version)
//!
//! Handles loading extensions from the filesystem.
//!
//! Optimizations Applied:
//! - Pre-allocated collection capacity
//! - Reduced string allocations
//! - Optimized file I/O operations
//! - Better memory management

use anyhow::{anyhow, Result};
use log::{debug, info, warn};
use std::fs;
use std::path::{Path, PathBuf};
use super::extension::{Extension, ExtensionInfo, ExtensionState, ExtensionType};
use super::manifest::{Manifest, ManifestParser};

/// Extension loader (optimized)
pub struct ExtensionLoader {
    /// Extensions directory
    extensions_dir: PathBuf,
}

impl ExtensionLoader {
    /// Creates a new extension loader
    pub fn new<P: AsRef<Path>>(extensions_dir: P) -> Self {
        Self {
            extensions_dir: extensions_dir.as_ref().to_path_buf(),
        }
    }

    /// Loads all extensions from the extensions directory (optimized: pre-allocated capacity)
    pub fn load_all(&self) -> Result<Vec<Box<dyn Extension>>> {
        info!("Loading extensions from: {:?}", self.extensions_dir);

        if !self.extensions_dir.exists() {
            warn!("Extensions directory does not exist: {:?}", self.extensions_dir);
            return Ok(Vec::with_capacity(0));
        }

        // Pre-allocate capacity based on directory entries
        let entries: Vec<_> = fs::read_dir(&self.extensions_dir)?
            .filter_map(|e| e.ok())
            .collect();
        
        let mut extensions = Vec::with_capacity(entries.len());

        for entry in entries {
            let path = entry.path();

            if path.is_dir() {
                debug!("Found extension directory: {:?}", path);

                match self.load_extension(&path) {
                    Ok(extension) => {
                        info!("Loaded extension: {}", extension.info().name);
                        extensions.push(extension);
                    }
                    Err(e) => {
                        warn!("Failed to load extension from {:?}: {}", path, e);
                    }
                }
            }
        }

        info!("Loaded {} extensions", extensions.len());
        Ok(extensions)
    }

    /// Loads a single extension from a directory (optimized: reduced allocations)
    pub fn load_extension<P: AsRef<Path>>(&self, extension_dir: P) -> Result<Box<dyn Extension>> {
        let extension_dir = extension_dir.as_ref();
        let manifest_path = extension_dir.join("manifest.json");

        if !manifest_path.exists() {
            return Err(anyhow!("manifest.json not found in {:?}", extension_dir));
        }

        // Parse manifest
        let manifest = ManifestParser::parse(&manifest_path)?;

        // Create extension info
        let info = ExtensionInfo {
            id: self.generate_extension_id(&manifest.name, &manifest.version),
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            description: manifest.description.clone(),
            author: manifest.author.clone(),
            extension_type: self.determine_extension_type(&manifest),
            state: ExtensionState::Installed,
            permissions: manifest.permissions.clone(),
            icons: self.extract_icons(&manifest, extension_dir),
            homepage_url: manifest.homepage_url.clone(),
        };

        // Load extension files (optimized: use with_capacity)
        let background_script = manifest.background.as_ref().map(|bg| {
            fs::read_to_string(extension_dir.join(&bg.script))
                .unwrap_or_else(|_| String::new())
        });

        let content_scripts: Vec<String> = manifest.content_scripts
            .iter()
            .flat_map(|cs| cs.js.iter())
            .filter_map(|js| {
                fs::read_to_string(extension_dir.join(js)).ok()
            })
            .collect();

        let popup_html = manifest.popup.as_ref().map(|popup| {
            fs::read_to_string(extension_dir.join(&popup.html))
                .unwrap_or_else(|_| String::new())
        });

        // Create extension instance
        let extension = Box::new(LoadedExtension {
            info,
            background_script,
            content_scripts,
            popup_html,
            manifest,
        });

        Ok(extension)
    }

    /// Generates a unique extension ID (optimized: reduced allocations)
    fn generate_extension_id(&self, name: &str, version: &str) -> String {
        // Pre-allocate capacity for typical extension ID format
        let mut result = String::with_capacity(name.len() + version.len() + 1);
        
        // Convert to lowercase and replace spaces with dashes
        for c in name.chars() {
            if c == ' ' {
                result.push('-');
            } else {
                result.extend(c.to_lowercase());
            }
        }
        
        result.push('@');
        result.push_str(version);
        
        result
    }

    /// Determines the extension type from the manifest
    fn determine_extension_type(&manifest: &Manifest) -> ExtensionType {
        if manifest.popup.is_some() {
            ExtensionType::Popup
        } else if manifest.background.is_some() {
            ExtensionType::BackgroundScript
        } else if !manifest.content_scripts.is_empty() {
            ExtensionType::ContentScript
        } else {
            ExtensionType::Custom("unknown".to_string())
        }
    }

    /// Extracts icon paths from the manifest (optimized: pre-allocated capacity)
    fn extract_icons(&manifest: &Manifest, extension_dir: &Path) -> std::collections::HashMap<String, String> {
        let mut icons = std::collections::HashMap::with_capacity(4);

        if let Some(icon_16) = &manifest.icons.icon_16 {
            icons.insert("16".to_string(), extension_dir.join(icon_16).to_string_lossy().to_string());
        }
        if let Some(icon_32) = &manifest.icons.icon_32 {
            icons.insert("32".to_string(), extension_dir.join(icon_32).to_string_lossy().to_string());
        }
        if let Some(icon_48) = &manifest.icons.icon_48 {
            icons.insert("48".to_string(), extension_dir.join(icon_48).to_string_lossy().to_string());
        }
        if let Some(icon_128) = &manifest.icons.icon_128 {
            icons.insert("128".to_string(), extension_dir.join(icon_128).to_string_lossy().to_string());
        }

        icons
    }
}

/// Loaded extension implementation
struct LoadedExtension {
    info: ExtensionInfo,
    background_script: Option<String>,
    content_scripts: Vec<String>,
    popup_html: Option<String>,
    manifest: Manifest,
}

impl Extension for LoadedExtension {
    fn info(&self) -> &ExtensionInfo {
        &self.info
    }

    fn on_load(&mut self) -> Result<()> {
        debug!("Extension loaded: {}", self.info.name);
        Ok(())
    }

    fn on_enable(&mut self) -> Result<()> {
        debug!("Extension enabled: {}", self.info.name);
        self.info.state = ExtensionState::Enabled;
        Ok(())
    }

    fn on_disable(&mut self) -> Result<()> {
        debug!("Extension disabled: {}", self.info.name);
        self.info.state = ExtensionState::Disabled;
        Ok(())
    }

    fn on_unload(&mut self) -> Result<()> {
        debug!("Extension unloaded: {}", self.info.name);
        Ok(())
    }

    fn on_message(&mut self, message: &str) -> Result<String> {
        debug!("Extension received message: {}", message);
        Ok("Message received".to_string())
    }

    fn background_script(&self) -> Option<&str> {
        self.background_script.as_deref()
    }

    fn content_scripts(&self) -> Vec<&str> {
        self.content_scripts.iter().map(|s| s.as_str()).collect()
    }

    fn popup_html(&self) -> Option<&str> {
        self.popup_html.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_generate_extension_id() {
        let loader = ExtensionLoader::new("/tmp/extensions");
        let id = loader.generate_extension_id("Test Extension", "1.0.0");
        assert_eq!(id, "test-extension@1.0.0");
    }

    #[test]
    fn test_determine_extension_type() {
        let loader = ExtensionLoader::new("/tmp/extensions");

        let mut manifest = Manifest {
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            author: "Test".to_string(),
            manifest_version: 2,
            permissions: vec![],
            background: None,
            content_scripts: vec![],
            popup: None,
            icons: Icons::default(),
            homepage_url: None,
            minimum_browser_version: None,
        };

        // Test popup type
        manifest.popup = Some(Popup {
            html: "popup.html".to_string(),
            default_title: None,
            default_icon: None,
        });
        assert_eq!(loader.determine_extension_type(&manifest), ExtensionType::Popup);

        // Test background script type
        manifest.popup = None;
        manifest.background = Some(BackgroundScript {
            script: "background.js".to_string(),
            persistent: false,
        });
        assert_eq!(loader.determine_extension_type(&manifest), ExtensionType::BackgroundScript);

        // Test content script type
        manifest.background = None;
        manifest.content_scripts.push(ContentScript {
            js: vec!["content.js".to_string()],
            css: vec![],
            matches: vec!["<all_urls>".to_string()],
            run_at: "document_end".to_string(),
        });
        assert_eq!(loader.determine_extension_type(&manifest), ExtensionType::ContentScript);
    }

    #[test]
    fn test_load_extension() {
        let temp_dir = TempDir::new().unwrap();
        let extension_dir = temp_dir.path().join("test-extension");
        fs::create_dir(&extension_dir).unwrap();

        let manifest_json = r#"{
                "name": "Test Extension",
                "version": "1.0.0",
                "description": "A test extension",
                "author": "Test Author",
                "manifest_version": 2,
                "permissions": ["storage"]
            }"#;

        let manifest_path = extension_dir.join("manifest.json");
        let mut file = fs::File::create(&manifest_path).unwrap();
        file.write_all(manifest_json.as_bytes()).unwrap();

        let loader = ExtensionLoader::new(temp_dir.path());
        let result = loader.load_extension(&extension_dir);

        assert!(result.is_ok());
        let extension = result.unwrap();
        assert_eq!(extension.info().name, "Test Extension");
        assert_eq!(extension.info().version, "1.0.0");
    }
}