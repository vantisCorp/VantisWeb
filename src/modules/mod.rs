//! Modules Module (Atom Switch)
//! 
//! Atomic module system:
//! - Dynamic module loading/unloading
//! - Atom Switch architecture
//! - Module marketplace
//! - Plugin system

pub mod loader;
pub mod registry;
pub mod marketplace;
pub mod plugin;

pub use loader::*;
pub use registry::*;
pub use marketplace::*;
pub use plugin::*;

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Atom Switch module manager
pub struct AtomSwitch {
    /// Module loader
    loader: ModuleLoader,
    /// Module registry
    registry: ModuleRegistry,
    /// Marketplace client
    marketplace: ModuleMarketplace,
    /// Plugin manager
    plugins: PluginManager,
    /// Active modules
    modules: Arc<Mutex<HashMap<String, LoadedModule>>>,
    /// Configuration
    config: AtomConfig,
}

/// Atom Switch configuration
#[derive(Debug, Clone)]
pub struct AtomConfig {
    /// Module directory
    pub module_dir: PathBuf,
    /// Enable marketplace
    pub enable_marketplace: bool,
    /// Auto-update modules
    pub auto_update: bool,
    /// Maximum modules
    pub max_modules: usize,
    /// Enable hot reload
    pub hot_reload: bool,
    /// Sandbox modules
    pub sandbox: bool,
}

impl Default for AtomConfig {
    fn default() -> Self {
        Self {
            module_dir: PathBuf::from("./modules"),
            enable_marketplace: true,
            auto_update: false,
            max_modules: 100,
            hot_reload: true,
            sandbox: true,
        }
    }
}

impl AtomSwitch {
    /// Creates a new Atom Switch manager
    pub fn new() -> Result<Self> {
        Self::with_config(AtomConfig::default())
    }
    
    /// Creates with custom configuration
    pub fn with_config(config: AtomConfig) -> Result<Self> {
        std::fs::create_dir_all(&config.module_dir)?;
        
        let loader = ModuleLoader::new(config.module_dir.clone())?;
        let registry = ModuleRegistry::new();
        let marketplace = ModuleMarketplace::new();
        let plugins = PluginManager::new();
        
        Ok(Self {
            loader,
            registry,
            marketplace,
            plugins,
            modules: Arc::new(Mutex::new(HashMap::new())),
            config,
        })
    }
    
    /// Installs a module
    pub fn install_module(&self, module_id: &str) -> Result<LoadedModule> {
        // Check if already installed
        {
            let modules = self.modules.lock().unwrap();
            if modules.contains_key(module_id) {
                return Err(anyhow!("Module already installed: {}", module_id));
            }
        }
        
        // Load the module
        let module = self.loader.load(module_id)?;
        
        // Register in registry
        self.registry.register(&module.metadata)?;
        
        // Store in active modules
        self.modules.lock().unwrap().insert(module_id.to_string(), module.clone());
        
        log::info!("Installed module: {} v{}", module.metadata.name, module.metadata.version);
        Ok(module)
    }
    
    /// Uninstalls a module
    pub fn uninstall_module(&self, module_id: &str) -> Result<()> {
        let mut modules = self.modules.lock().unwrap();
        
        if let Some(module) = modules.remove(module_id) {
            // Unregister from registry
            self.registry.unregister(module_id)?;
            
            // Unload
            self.loader.unload(&module)?;
            
            log::info!("Uninstalled module: {}", module_id);
        }
        
        Ok(())
    }
    
    /// Enables a module
    pub fn enable_module(&self, module_id: &str) -> Result<()> {
        let mut modules = self.modules.lock().unwrap();
        if let Some(module) = modules.get_mut(module_id) {
            module.enabled = true;
            log::info!("Enabled module: {}", module_id);
        }
        Ok(())
    }
    
    /// Disables a module
    pub fn disable_module(&self, module_id: &str) -> Result<()> {
        let mut modules = self.modules.lock().unwrap();
        if let Some(module) = modules.get_mut(module_id) {
            module.enabled = false;
            log::info!("Disabled module: {}", module_id);
        }
        Ok(())
    }
    
    /// Gets an installed module
    pub fn get_module(&self, module_id: &str) -> Option<LoadedModule> {
        self.modules.lock().unwrap().get(module_id).cloned()
    }
    
    /// Gets all installed modules
    pub fn get_modules(&self) -> Vec<LoadedModule> {
        self.modules.lock().unwrap().values().cloned().collect()
    }
    
    /// Searches marketplace for modules
    pub fn search_marketplace(&self, query: &str) -> Result<Vec<ModuleMetadata>> {
        self.marketplace.search(query)
    }
    
    /// Installs a plugin
    pub fn install_plugin(&self, plugin_path: &PathBuf) -> Result<Plugin> {
        self.plugins.load_plugin(plugin_path)
    }
    
    /// Gets the module loader
    pub fn loader(&self) -> &ModuleLoader {
        &self.loader
    }
    
    /// Gets the module registry
    pub fn registry(&self) -> &ModuleRegistry {
        &self.registry
    }
    
    /// Shuts down all modules
    pub fn shutdown(&self) -> Result<()> {
        let module_ids: Vec<String> = self.modules.lock().unwrap().keys().cloned().collect();
        
        for id in module_ids {
            self.uninstall_module(&id)?;
        }
        
        log::info!("Atom Switch shut down");
        Ok(())
    }
}

/// Loaded module instance
#[derive(Debug, Clone)]
pub struct LoadedModule {
    /// Module ID
    pub id: String,
    /// Module metadata
    pub metadata: ModuleMetadata,
    /// Module state
    pub state: ModuleState,
    /// Whether module is enabled
    pub enabled: bool,
    /// Module path
    pub path: PathBuf,
    /// Module exports
    pub exports: HashMap<String, serde_json::Value>,
}

/// Module state
#[derive(Debug, Clone, PartialEq)]
pub enum ModuleState {
    /// Loading
    Loading,
    /// Loaded
    Loaded,
    /// Initializing
    Initializing,
    /// Active
    Active,
    /// Error
    Error(String),
    /// Unloaded
    Unloaded,
}

/// Module metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModuleMetadata {
    /// Module ID
    pub id: String,
    /// Module name
    pub name: String,
    /// Module version
    pub version: String,
    /// Module description
    pub description: String,
    /// Module author
    pub author: String,
    /// Module homepage
    pub homepage: Option<String>,
    /// Module license
    pub license: String,
    /// Module dependencies
    pub dependencies: Vec<ModuleDependency>,
    /// Module permissions
    pub permissions: Vec<String>,
    /// Module icon
    pub icon: Option<String>,
    /// Module category
    pub category: ModuleCategory,
}

/// Module dependency
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModuleDependency {
    /// Dependency module ID
    pub id: String,
    /// Version constraint (semver)
    pub version: String,
    /// Whether dependency is optional
    pub optional: bool,
}

/// Module category
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum ModuleCategory {
    /// Productivity
    Productivity,
    /// Development
    Development,
    /// Security
    Security,
    /// Privacy
    Privacy,
    /// Media
    Media,
    /// Social
    Social,
    /// Tools
    Tools,
    /// Theme
    Theme,
    /// Other,
    Other,
}