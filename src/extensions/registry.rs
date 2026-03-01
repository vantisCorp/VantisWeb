//! Extension Registry
//!
//! Manages the collection of loaded extensions.

use anyhow::{anyhow, Result};
use log::{debug, info};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use super::extension::{Extension, ExtensionInfo, ExtensionState};

/// Extension registry
pub struct ExtensionRegistry {
    /// Registered extensions
    extensions: Arc<Mutex<HashMap<String, Box<dyn Extension>>>>,
}

impl ExtensionRegistry {
    /// Creates a new extension registry
    pub fn new() -> Self {
        Self {
            extensions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Registers an extension
    pub fn register(&self, extension: Box<dyn Extension>) -> Result<()> {
        let info = extension.info();
        let id = info.id.clone();

        debug!("Registering extension: {} ({})", info.name, id);

        let mut extensions = self.extensions.lock()
            .map_err(|e| anyhow!("Failed to lock registry: {}", e))?;

        if extensions.contains_key(&id) {
            return Err(anyhow!("Extension already registered: {}", id));
        }

        extensions.insert(id, extension);
        info!("Extension registered: {}", info.name);

        Ok(())
    }

    /// Unregisters an extension
    pub fn unregister(&self, extension_id: &str) -> Result<()> {
        debug!("Unregistering extension: {}", extension_id);

        let mut extensions = self.extensions.lock()
            .map_err(|e| anyhow!("Failed to lock registry: {}", e))?;

        if !extensions.contains_key(extension_id) {
            return Err(anyhow!("Extension not found: {}", extension_id));
        }

        extensions.remove(extension_id);
        info!("Extension unregistered: {}", extension_id);

        Ok(())
    }

    /// Gets an extension by ID
    pub fn get(&self, extension_id: &str) -> Result<Arc<Mutex<Box<dyn Extension>>>> {
        let extensions = self.extensions.lock()
            .map_err(|e| anyhow!("Failed to lock registry: {}", e))?;

        extensions.get(extension_id)
            .map(|ext| Arc::new(Mutex::new(ext.clone())))
            .ok_or_else(|| anyhow!("Extension not found: {}", extension_id))
    }

    /// Gets all extensions
    pub fn get_all(&self) -> Result<Vec<ExtensionInfo>> {
        let extensions = self.extensions.lock()
            .map_err(|e| anyhow!("Failed to lock registry: {}", e))?;

        Ok(extensions.values()
            .map(|ext| ext.info().clone())
            .collect())
    }

    /// Gets extensions by state
    pub fn get_by_state(&self, state: ExtensionState) -> Result<Vec<ExtensionInfo>> {
        let extensions = self.extensions.lock()
            .map_err(|e| anyhow!("Failed to lock registry: {}", e))?;

        Ok(extensions.values()
            .filter(|ext| ext.info().state == state)
            .map(|ext| ext.info().clone())
            .collect())
    }

    /// Gets enabled extensions
    pub fn get_enabled(&self) -> Result<Vec<ExtensionInfo>> {
        self.get_by_state(ExtensionState::Enabled)
    }

    /// Gets disabled extensions
    pub fn get_disabled(&self) -> Result<Vec<ExtensionInfo>> {
        self.get_by_state(ExtensionState::Disabled)
    }

    /// Gets the count of registered extensions
    pub fn count(&self) -> Result<usize> {
        let extensions = self.extensions.lock()
            .map_err(|e| anyhow!("Failed to lock registry: {}", e))?;

        Ok(extensions.len())
    }

    /// Checks if an extension is registered
    pub fn contains(&self, extension_id: &str) -> Result<bool> {
        let extensions = self.extensions.lock()
            .map_err(|e| anyhow!("Failed to lock registry: {}", e))?;

        Ok(extensions.contains_key(extension_id))
    }

    /// Clears all extensions
    pub fn clear(&self) -> Result<()> {
        debug!("Clearing extension registry");

        let mut extensions = self.extensions.lock()
            .map_err(|e| anyhow!("Failed to lock registry: {}", e))?;

        extensions.clear();
        info!("Extension registry cleared");

        Ok(())
    }
}

impl Default for ExtensionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extensions::extension::{ExtensionType, LoadedExtension};

    struct MockExtension {
        info: ExtensionInfo,
    }

    impl MockExtension {
        fn new(name: &str, version: &str) -> Self {
            Self {
                info: ExtensionInfo {
                    id: format!("{}@{}", name, version),
                    name: name.to_string(),
                    version: version.to_string(),
                    description: "Mock extension".to_string(),
                    author: "Test".to_string(),
                    extension_type: ExtensionType::ContentScript,
                    state: ExtensionState::Installed,
                    permissions: vec![],
                    icons: HashMap::new(),
                    homepage_url: None,
                },
            }
        }
    }

    impl Extension for MockExtension {
        fn info(&self) -> &ExtensionInfo {
            &self.info
        }

        fn on_load(&mut self) -> Result<()> {
            Ok(())
        }

        fn on_enable(&mut self) -> Result<()> {
            self.info.state = ExtensionState::Enabled;
            Ok(())
        }

        fn on_disable(&mut self) -> Result<()> {
            self.info.state = ExtensionState::Disabled;
            Ok(())
        }

        fn on_unload(&mut self) -> Result<()> {
            Ok(())
        }

        fn on_message(&mut self, _message: &str) -> Result<String> {
            Ok("OK".to_string())
        }
    }

    #[test]
    fn test_register_extension() {
        let registry = ExtensionRegistry::new();
        let extension = Box::new(MockExtension::new("Test", "1.0.0"));

        let result = registry.register(extension);
        assert!(result.is_ok());

        let count = registry.count().unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_register_duplicate_extension() {
        let registry = ExtensionRegistry::new();
        let extension1 = Box::new(MockExtension::new("Test", "1.0.0"));
        let extension2 = Box::new(MockExtension::new("Test", "1.0.0"));

        registry.register(extension1).unwrap();
        let result = registry.register(extension2);

        assert!(result.is_err());
    }

    #[test]
    fn test_unregister_extension() {
        let registry = ExtensionRegistry::new();
        let extension = Box::new(MockExtension::new("Test", "1.0.0"));

        registry.register(extension).unwrap();
        let result = registry.unregister("Test@1.0.0");

        assert!(result.is_ok());

        let count = registry.count().unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_get_extension() {
        let registry = ExtensionRegistry::new();
        let extension = Box::new(MockExtension::new("Test", "1.0.0"));

        registry.register(extension).unwrap();
        let result = registry.get("Test@1.0.0");

        assert!(result.is_ok());
    }

    #[test]
    fn test_get_all_extensions() {
        let registry = ExtensionRegistry::new();

        registry.register(Box::new(MockExtension::new("Test1", "1.0.0"))).unwrap();
        registry.register(Box::new(MockExtension::new("Test2", "1.0.0"))).unwrap();

        let extensions = registry.get_all().unwrap();
        assert_eq!(extensions.len(), 2);
    }

    #[test]
    fn test_get_enabled_extensions() {
        let registry = ExtensionRegistry::new();

        let mut ext1 = Box::new(MockExtension::new("Test1", "1.0.0"));
        ext1.on_enable().unwrap();
        registry.register(ext1).unwrap();

        let ext2 = Box::new(MockExtension::new("Test2", "1.0.0"));
        registry.register(ext2).unwrap();

        let enabled = registry.get_enabled().unwrap();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].name, "Test1");
    }

    #[test]
    fn test_contains_extension() {
        let registry = ExtensionRegistry::new();
        let extension = Box::new(MockExtension::new("Test", "1.0.0"));

        registry.register(extension).unwrap();

        assert!(registry.contains("Test@1.0.0").unwrap());
        assert!(!registry.contains("NonExistent").unwrap());
    }

    #[test]
    fn test_clear_registry() {
        let registry = ExtensionRegistry::new();

        registry.register(Box::new(MockExtension::new("Test1", "1.0.0"))).unwrap();
        registry.register(Box::new(MockExtension::new("Test2", "1.0.0"))).unwrap();

        registry.clear().unwrap();

        let count = registry.count().unwrap();
        assert_eq!(count, 0);
    }
}