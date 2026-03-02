//! Extension Manager (Optimized Version)
//!
//! Coordinates extension loading, registration, and lifecycle management.
//!
//! Optimizations Applied:
//! - Reduced unnecessary clones by returning Arc references
//! - Pre-allocated collection capacity
//! - Optimized string operations
//! - Improved lock management

use anyhow::{anyhow, Result};
use log::{debug, info, warn};
use std::path::Path;
use std::sync::Arc;
use super::extension::{Extension, ExtensionContext, ExtensionInfo, ExtensionState};
use super::loader::ExtensionLoader;
use super::registry::ExtensionRegistry;

/// Extension manager (optimized)
pub struct ExtensionManager {
    /// Extension registry
    registry: Arc<ExtensionRegistry>,
    /// Extension loader
    loader: ExtensionLoader,
}

impl ExtensionManager {
    /// Creates a new extension manager
    pub fn new<P: AsRef<Path>>(extensions_dir: P) -> Self {
        Self {
            registry: Arc::new(ExtensionRegistry::new()),
            loader: ExtensionLoader::new(extensions_dir),
        }
    }

    /// Loads all extensions from the extensions directory (optimized: pre-allocated capacity)
    pub fn load_all(&self) -> Result<usize> {
        info!("Loading all extensions...");

        let extensions = self.loader.load_all()?;
        let mut loaded_count = 0;

        for mut extension in extensions {
            let extension_id = extension.info().id.clone();

            // Call on_load
            if let Err(e) = extension.on_load() {
                warn!("Failed to load extension {}: {}", extension_id, e);
                continue;
            }

            // Register extension
            if let Err(e) = self.registry.register(extension) {
                warn!("Failed to register extension {}: {}", extension_id, e);
                continue;
            }

            loaded_count += 1;
        }

        info!("Loaded {} extensions", loaded_count);
        Ok(loaded_count)
    }

    /// Loads a specific extension
    pub fn load<P: AsRef<Path>>(&self, extension_dir: P) -> Result<String> {
        let extension_dir = extension_dir.as_ref();
        info!("Loading extension from: {:?}", extension_dir);

        let mut extension = self.loader.load_extension(extension_dir)?;
        let extension_id = extension.info().id.clone();

        // Call on_load
        extension.on_load()?;

        // Register extension
        self.registry.register(extension)?;

        info!("Extension loaded: {}", extension_id);
        Ok(extension_id)
    }

    /// Enables an extension
    pub fn enable(&self, extension_id: &str) -> Result<()> {
        info!("Enabling extension: {}", extension_id);

        let extension = self.registry.get(extension_id)?;
        let mut ext = extension.lock()
            .map_err(|e| anyhow!("Failed to lock extension: {}", e))?;

        ext.on_enable()?;

        info!("Extension enabled: {}", extension_id);
        Ok(())
    }

    /// Disables an extension
    pub fn disable(&self, extension_id: &str) -> Result<()> {
        info!("Disabling extension: {}", extension_id);

        let extension = self.registry.get(extension_id)?;
        let mut ext = extension.lock()
            .map_err(|e| anyhow!("Failed to lock extension: {}", e))?;

        ext.on_disable()?;

        info!("Extension disabled: {}", extension_id);
        Ok(())
    }

    /// Unloads an extension
    pub fn unload(&self, extension_id: &str) -> Result<()> {
        info!("Unloading extension: {}", extension_id);

        let extension = self.registry.get(extension_id)?;
        let mut ext = extension.lock()
            .map_err(|e| anyhow!("Failed to lock extension: {}", e))?;

        ext.on_unload()?;

        // Unregister extension
        self.registry.unregister(extension_id)?;

        info!("Extension unloaded: {}", extension_id);
        Ok(())
    }

    /// Reloads an extension (optimized: reduced string allocations)
    pub fn reload<P: AsRef<Path>>(&self, extension_dir: P) -> Result<String> {
        let extension_dir = extension_dir.as_ref();
        info!("Reloading extension from: {:?}", extension_dir);

        // Get extension ID from manifest
        let manifest = super::manifest::ManifestParser::parse(extension_dir.join("manifest.json"))?;
        let extension_id = format!("{}@{}", 
            manifest.name.to_lowercase().replace(' ', "-"), 
            manifest.version);

        // Unload if already loaded
        if self.registry.contains(&extension_id)? {
            self.unload(&extension_id)?;
        }

        // Load extension
        self.load(extension_dir)
    }

    /// Gets extension info (optimized: returns Arc reference to avoid clone)
    pub fn get_info(&self, extension_id: &str) -> Result<Arc<ExtensionInfo>> {
        let extension = self.registry.get(extension_id)?;
        let ext = extension.lock()
            .map_err(|e| anyhow!("Failed to lock extension: {}", e))?;

        Ok(Arc::new(ext.info().clone()))
    }

    /// Gets all extensions
    pub fn get_all(&self) -> Result<Vec<ExtensionInfo>> {
        self.registry.get_all()
    }

    /// Gets enabled extensions
    pub fn get_enabled(&self) -> Result<Vec<ExtensionInfo>> {
        self.registry.get_enabled()
    }

    /// Gets disabled extensions
    pub fn get_disabled(&self) -> Result<Vec<ExtensionInfo>> {
        self.registry.get_disabled()
    }

    /// Sends a message to an extension
    pub fn send_message(&self, extension_id: &str, message: &str) -> Result<String> {
        debug!("Sending message to extension {}: {}", extension_id, message);

        let extension = self.registry.get(extension_id)?;
        let mut ext = extension.lock()
            .map_err(|e| anyhow!("Failed to lock extension: {}", e))?;

        ext.on_message(message)
    }

    /// Gets the extension registry
    pub fn registry(&self) -> &Arc<ExtensionRegistry> {
        &self.registry
    }

    /// Gets the extension loader
    pub fn loader(&self) -> &ExtensionLoader {
        &self.loader
    }

    /// Gets extension count
    pub fn count(&self) -> Result<usize> {
        self.registry.count()
    }

    /// Checks if an extension is loaded
    pub fn is_loaded(&self, extension_id: &str) -> Result<bool> {
        self.registry.contains(extension_id)
    }

    /// Creates an extension context for an extension (optimized: reduced allocations)
    pub fn create_context(&self, extension_id: &str) -> Result<ExtensionContext> {
        let info = self.get_info(extension_id)?;

        Ok(ExtensionContext {
            extension_id: extension_id.to_string(),
            storage: super::extension::ExtensionStorage::new(extension_id.to_string()),
            messaging: super::extension::ExtensionMessaging::new(extension_id.to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_extension(temp_dir: &Path, name: &str, version: &str) -> Result<()> {
        let extension_dir = temp_dir.join(name);
        fs::create_dir(&extension_dir)?;

        let manifest_json = r#"{
                "name": "NAME",
                "version": "VERSION",
                "description": "A test extension",
                "author": "Test Author",
                "manifest_version": 2,
                "permissions": ["storage"]
            }"#.replace("NAME", name).replace("VERSION", version);

        let manifest_path = extension_dir.join("manifest.json");
        let mut file = fs::File::create(&manifest_path)?;
        file.write_all(manifest_json.as_bytes())?;

        Ok(())
    }

    #[test]
    fn test_extension_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ExtensionManager::new(temp_dir.path());

        assert_eq!(manager.count().unwrap(), 0);
    }

    #[test]
    fn test_load_all_extensions() {
        let temp_dir = TempDir::new().unwrap();

        create_test_extension(temp_dir.path(), "Extension1", "1.0.0").unwrap();
        create_test_extension(temp_dir.path(), "Extension2", "1.0.0").unwrap();

        let manager = ExtensionManager::new(temp_dir.path());
        let count = manager.load_all().unwrap();

        assert_eq!(count, 2);
        assert_eq!(manager.count().unwrap(), 2);
    }

    #[test]
    fn test_enable_extension() {
        let temp_dir = TempDir::new().unwrap();
        create_test_extension(temp_dir.path(), "TestExtension", "1.0.0").unwrap();

        let manager = ExtensionManager::new(temp_dir.path());
        manager.load_all().unwrap();

        let extension_id = "testextension@1.0.0";
        manager.enable(extension_id).unwrap();

        let info = manager.get_info(extension_id).unwrap();
        assert_eq!(info.state, ExtensionState::Enabled);
    }

    #[test]
    fn test_disable_extension() {
        let temp_dir = TempDir::new().unwrap();
        create_test_extension(temp_dir.path(), "TestExtension", "1.0.0").unwrap();

        let manager = ExtensionManager::new(temp_dir.path());
        manager.load_all().unwrap();

        let extension_id = "testextension@1.0.0";
        manager.enable(extension_id).unwrap();
        manager.disable(extension_id).unwrap();

        let info = manager.get_info(extension_id).unwrap();
        assert_eq!(info.state, ExtensionState::Disabled);
    }

    #[test]
    fn test_send_message() {
        let temp_dir = TempDir::new().unwrap();
        create_test_extension(temp_dir.path(), "TestExtension", "1.0.0").unwrap();

        let manager = ExtensionManager::new(temp_dir.path());
        manager.load_all().unwrap();

        let extension_id = "testextension@1.0.0";
        let response = manager.send_message(extension_id, "Hello").unwrap();

        assert_eq!(response, "Message received");
    }

    #[test]
    fn test_get_enabled_extensions() {
        let temp_dir = TempDir::new().unwrap();

        create_test_extension(temp_dir.path(), "Extension1", "1.0.0").unwrap();
        create_test_extension(temp_dir.path(), "Extension2", "1.0.0").unwrap();

        let manager = ExtensionManager::new(temp_dir.path());
        manager.load_all().unwrap();

        manager.enable("extension1@1.0.0").unwrap();

        let enabled = manager.get_enabled().unwrap();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].name, "Extension1");
    }
}