//! Module Registry
//! 
//! Tracks installed modules and their states

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::ModuleMetadata;

/// Module registry entry
#[derive(Debug, Clone)]
pub struct RegistryEntry {
    /// Module metadata
    pub metadata: ModuleMetadata,
    /// Installation timestamp
    pub installed_at: chrono::DateTime<chrono::Utc>,
    /// Last updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Installation path
    pub path: String,
    /// Whether module is enabled
    pub enabled: bool,
    /// Usage count
    pub usage_count: u64,
}

/// Module registry
pub struct ModuleRegistry {
    /// Registered modules
    entries: Arc<Mutex<HashMap<String, RegistryEntry>>>,
}

impl ModuleRegistry {
    /// Creates a new module registry
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Registers a module
    pub fn register(&self, metadata: &ModuleMetadata) -> Result<()> {
        let mut entries = self.entries.lock().unwrap();
        
        if entries.contains_key(&metadata.id) {
            return Err(anyhow!("Module already registered: {}", metadata.id));
        }
        
        let entry = RegistryEntry {
            metadata: metadata.clone(),
            installed_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            path: format!("./modules/{}", metadata.id),
            enabled: true,
            usage_count: 0,
        };
        
        entries.insert(metadata.id.clone(), entry);
        
        log::info!("Registered module: {}", metadata.id);
        Ok(())
    }
    
    /// Unregisters a module
    pub fn unregister(&self, module_id: &str) -> Result<()> {
        let mut entries = self.entries.lock().unwrap();
        
        if entries.remove(module_id).is_some() {
            log::info!("Unregistered module: {}", module_id);
        }
        
        Ok(())
    }
    
    /// Gets a module entry
    pub fn get(&self, module_id: &str) -> Option<RegistryEntry> {
        self.entries.lock().unwrap().get(module_id).cloned()
    }
    
    /// Gets all registered modules
    pub fn get_all(&self) -> Vec<RegistryEntry> {
        self.entries.lock().unwrap().values().cloned().collect()
    }
    
    /// Checks if a module is registered
    pub fn contains(&self, module_id: &str) -> bool {
        self.entries.lock().unwrap().contains_key(module_id)
    }
    
    /// Updates module usage count
    pub fn increment_usage(&self, module_id: &str) -> Result<()> {
        let mut entries = self.entries.lock().unwrap();
        
        if let Some(entry) = entries.get_mut(module_id) {
            entry.usage_count += 1;
        }
        
        Ok(())
    }
    
    /// Sets module enabled state
    pub fn set_enabled(&self, module_id: &str, enabled: bool) -> Result<()> {
        let mut entries = self.entries.lock().unwrap();
        
        if let Some(entry) = entries.get_mut(module_id) {
            entry.enabled = enabled;
        }
        
        Ok(())
    }
    
    /// Finds modules by category
    pub fn find_by_category(&self, category: &str) -> Vec<RegistryEntry> {
        self.entries.lock().unwrap()
            .values()
            .filter(|e| format!("{:?}", e.metadata.category).to_lowercase() == category.to_lowercase())
            .cloned()
            .collect()
    }
    
    /// Finds modules by author
    pub fn find_by_author(&self, author: &str) -> Vec<RegistryEntry> {
        self.entries.lock().unwrap()
            .values()
            .filter(|e| e.metadata.author.to_lowercase() == author.to_lowercase())
            .cloned()
            .collect()
    }
    
    /// Checks for updates (returns modules with available updates)
    pub fn check_updates(&self) -> Result<Vec<String>> {
        // In real implementation, would check against marketplace
        Ok(Vec::new())
    }
    
    /// Gets dependency tree for a module
    pub fn get_dependency_tree(&self, module_id: &str) -> Result<Vec<String>> {
        let entries = self.entries.lock().unwrap();
        
        let entry = entries.get(module_id)
            .ok_or_else(|| anyhow!("Module not found: {}", module_id))?;
        
        let mut deps = Vec::new();
        self.collect_dependencies(&entries, &entry.metadata, &mut deps)?;
        
        Ok(deps)
    }
    
    /// Recursively collects dependencies
    fn collect_dependencies(
        &self,
        entries: &HashMap<String, RegistryEntry>,
        metadata: &ModuleMetadata,
        deps: &mut Vec<String>,
    ) -> Result<()> {
        for dep in &metadata.dependencies {
            if !deps.contains(&dep.id) {
                deps.push(dep.id.clone());
                
                if let Some(entry) = entries.get(&dep.id) {
                    self.collect_dependencies(entries, &entry.metadata, deps)?;
                }
            }
        }
        
        Ok(())
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::ModuleCategory;
    
    fn create_test_metadata(id: &str) -> ModuleMetadata {
        ModuleMetadata {
            id: id.to_string(),
            name: format!("Test Module {}", id),
            version: "1.0.0".to_string(),
            description: "A test module".to_string(),
            author: "Test Author".to_string(),
            homepage: None,
            license: "MIT".to_string(),
            dependencies: vec![],
            permissions: vec![],
            icon: None,
            category: ModuleCategory::Tools,
        }
    }
    
    #[test]
    fn test_registry_creation() {
        let registry = ModuleRegistry::new();
        assert!(registry.get_all().is_empty());
    }
    
    #[test]
    fn test_register_module() {
        let registry = ModuleRegistry::new();
        let metadata = create_test_metadata("test-module");
        
        registry.register(&metadata).unwrap();
        assert!(registry.contains("test-module"));
    }
    
    #[test]
    fn test_unregister_module() {
        let registry = ModuleRegistry::new();
        let metadata = create_test_metadata("test-module");
        
        registry.register(&metadata).unwrap();
        registry.unregister("test-module").unwrap();
        
        assert!(!registry.contains("test-module"));
    }
    
    #[test]
    fn test_increment_usage() {
        let registry = ModuleRegistry::new();
        let metadata = create_test_metadata("test-module");
        
        registry.register(&metadata).unwrap();
        registry.increment_usage("test-module").unwrap();
        
        let entry = registry.get("test-module").unwrap();
        assert_eq!(entry.usage_count, 1);
    }
}