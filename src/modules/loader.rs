//! Module Loader
//! 
//! Handles loading and unloading of modules

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;

use super::{LoadedModule, ModuleMetadata, ModuleState};

/// Module loader
pub struct ModuleLoader {
    /// Module directory
    module_dir: PathBuf,
    /// Loaded module handles
    handles: HashMap<String, ModuleHandle>,
}

/// Handle to a loaded module
#[derive(Debug)]
struct ModuleHandle {
    /// Module path
    path: PathBuf,
    /// Library handle (in real implementation, would be libloading::Library)
    _loaded: bool,
}

impl ModuleLoader {
    /// Creates a new module loader
    pub fn new(module_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&module_dir)?;
        
        Ok(Self {
            module_dir,
            handles: HashMap::new(),
        })
    }
    
    /// Loads a module
    pub fn load(&self, module_id: &str) -> Result<LoadedModule> {
        let module_path = self.find_module(module_id)?;
        
        log::info!("Loading module from: {:?}", module_path);
        
        // Read module manifest
        let manifest_path = module_path.join("module.json");
        let metadata = self.read_manifest(&manifest_path)?;
        
        // Load module code
        let main_file = module_path.join("main.js");
        if main_file.exists() {
            log::debug!("Found main.js for module {}", module_id);
        }
        
        let module = LoadedModule {
            id: module_id.to_string(),
            metadata: metadata.clone(),
            state: ModuleState::Loaded,
            enabled: true,
            path: module_path.clone(),
            exports: HashMap::new(),
        };
        
        log::info!("Module {} v{} loaded successfully", metadata.name, metadata.version);
        Ok(module)
    }
    
    /// Unloads a module
    pub fn unload(&self, module: &LoadedModule) -> Result<()> {
        log::info!("Unloading module: {}", module.id);
        
        // In real implementation, would call module cleanup function
        // and unload dynamic library
        
        Ok(())
    }
    
    /// Finds a module by ID
    fn find_module(&self, module_id: &str) -> Result<PathBuf> {
        // Check built-in modules first
        let builtin_path = self.module_dir.join("builtin").join(module_id);
        if builtin_path.exists() {
            return Ok(builtin_path);
        }
        
        // Check installed modules
        let installed_path = self.module_dir.join("installed").join(module_id);
        if installed_path.exists() {
            return Ok(installed_path);
        }
        
        // Check by direct path
        let direct_path = self.module_dir.join(module_id);
        if direct_path.exists() {
            return Ok(direct_path);
        }
        
        Err(anyhow!("Module not found: {}", module_id))
    }
    
    /// Reads a module manifest
    fn read_manifest(&self, path: &PathBuf) -> Result<ModuleMetadata> {
        let content = fs::read_to_string(path)
            .map_err(|e| anyhow!("Failed to read module manifest: {}", e))?;
        
        let metadata: ModuleMetadata = serde_json::from_str(&content)
            .map_err(|e| anyhow!("Failed to parse module manifest: {}", e))?;
        
        Ok(metadata)
    }
    
    /// Lists available modules
    pub fn list_available(&self) -> Result<Vec<ModuleMetadata>> {
        let mut modules = Vec::new();
        
        // Scan builtin modules
        let builtin_dir = self.module_dir.join("builtin");
        if builtin_dir.exists() {
            for entry in fs::read_dir(&builtin_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    let manifest = path.join("module.json");
                    if manifest.exists() {
                        if let Ok(metadata) = self.read_manifest(&manifest) {
                            modules.push(metadata);
                        }
                    }
                }
            }
        }
        
        // Scan installed modules
        let installed_dir = self.module_dir.join("installed");
        if installed_dir.exists() {
            for entry in fs::read_dir(&installed_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    let manifest = path.join("module.json");
                    if manifest.exists() {
                        if let Ok(metadata) = self.read_manifest(&manifest) {
                            modules.push(metadata);
                        }
                    }
                }
            }
        }
        
        Ok(modules)
    }
    
    /// Validates a module
    pub fn validate(&self, module_path: &PathBuf) -> Result<Vec<String>> {
        let mut warnings = Vec::new();
        
        // Check manifest exists
        let manifest = module_path.join("module.json");
        if !manifest.exists() {
            return Err(anyhow!("Missing module.json manifest"));
        }
        
        // Read and validate manifest
        let metadata = self.read_manifest(&manifest)?;
        
        // Check required fields
        if metadata.id.is_empty() {
            warnings.push("Module ID is empty".to_string());
        }
        if metadata.name.is_empty() {
            warnings.push("Module name is empty".to_string());
        }
        if metadata.version.is_empty() {
            warnings.push("Module version is empty".to_string());
        }
        
        // Check main file
        let main_js = module_path.join("main.js");
        let main_wasm = module_path.join("main.wasm");
        
        if !main_js.exists() && !main_wasm.exists() {
            warnings.push("No main.js or main.wasm found".to_string());
        }
        
        // Check permissions
        for perm in &metadata.permissions {
            if !Self::is_valid_permission(perm) {
                warnings.push(format!("Unknown permission: {}", perm));
            }
        }
        
        Ok(warnings)
    }
    
    /// Checks if a permission is valid
    fn is_valid_permission(perm: &str) -> bool {
        let valid_permissions = [
            "storage",
            "tabs",
            "bookmarks",
            "history",
            "cookies",
            "downloads",
            "network",
            "clipboard",
            "notifications",
            "geolocation",
            "camera",
            "microphone",
            "system",
            "native",
        ];
        
        valid_permissions.contains(&perm) || perm.starts_with("custom.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_loader_creation() {
        let temp = TempDir::new().unwrap();
        let loader = ModuleLoader::new(temp.path().to_path_buf()).unwrap();
        assert!(loader.list_available().unwrap().is_empty());
    }
}