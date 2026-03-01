//! Profile Templates
//!
//! Predefined profile templates for different use cases.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Profile template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileTemplate {
    /// Template name
    pub name: String,
    /// Template description
    pub description: String,
    /// Template category
    pub category: TemplateCategory,
    /// Template settings
    pub settings: TemplateSettings,
    /// Template extensions
    pub extensions: Vec<String>,
    /// Template theme
    pub theme: Option<String>,
}

/// Template category
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TemplateCategory {
    /// Work profile
    Work,
    /// Gaming profile
    Gaming,
    /// Privacy profile
    Privacy,
    /// Developer profile
    Developer,
    /// Custom profile
    Custom(String),
}

/// Template settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSettings {
    /// Search engine
    pub search_engine: Option<String>,
    /// Homepage URL
    pub homepage: Option<String>,
    /// Custom start pages
    pub start_pages: Vec<String>,
    /// Keyboard shortcuts
    pub keyboard_shortcuts: HashMap<String, String>,
    /// Privacy settings
    pub privacy: PrivacySettings,
    /// Performance settings
    pub performance: PerformanceSettings,
}

/// Privacy settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    /// Block trackers
    pub block_trackers: bool,
    /// Block ads
    pub block_ads: bool,
    /// Clear cookies on exit
    pub clear_cookies_on_exit: bool,
    /// Clear history on exit
    pub clear_history_on_exit: bool,
    /// Enable private mode by default
    pub private_mode_by_default: bool,
    /// Disable JavaScript
    pub disable_javascript: bool,
}

/// Performance settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    /// Hardware acceleration
    pub hardware_acceleration: bool,
    /// Memory limit (MB)
    pub memory_limit: Option<u64>,
    /// CPU priority
    pub cpu_priority: CPUPriority,
    /// Cache size (MB)
    pub cache_size: u64,
}

/// CPU priority
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CPUPriority {
    Low,
    Normal,
    High,
}

/// Profile templates manager
pub struct TemplateManager {
    templates: HashMap<String, ProfileTemplate>,
}

impl TemplateManager {
    /// Creates a new template manager with default templates
    pub fn new() -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
        };

        // Load default templates
        manager.load_default_templates();

        manager
    }

    /// Loads default templates
    fn load_default_templates(&mut self) {
        // Work template
        let work_template = ProfileTemplate {
            name: "Work".to_string(),
            description: "Optimized for productivity and work-related tasks".to_string(),
            category: TemplateCategory::Work,
            settings: TemplateSettings {
                search_engine: Some("https://www.google.com/search?q=".to_string()),
                homepage: Some("https://www.google.com".to_string()),
                start_pages: vec![
                    "https://www.google.com".to_string(),
                    "https://github.com".to_string(),
                    "https://stackoverflow.com".to_string(),
                ],
                keyboard_shortcuts: HashMap::new(),
                privacy: PrivacySettings {
                    block_trackers: true,
                    block_ads: true,
                    clear_cookies_on_exit: false,
                    clear_history_on_exit: false,
                    private_mode_by_default: false,
                    disable_javascript: false,
                },
                performance: PerformanceSettings {
                    hardware_acceleration: true,
                    memory_limit: None,
                    cpu_priority: CPUPriority::Normal,
                    cache_size: 512,
                },
            },
            extensions: vec![
                "password-manager".to_string(),
                "productivity-tracker".to_string(),
            ],
            theme: Some("light".to_string()),
        };

        // Gaming template
        let gaming_template = ProfileTemplate {
            name: "Gaming".to_string(),
            description: "Optimized for gaming and streaming".to_string(),
            category: TemplateCategory::Gaming,
            settings: TemplateSettings {
                search_engine: Some("https://www.google.com/search?q=".to_string()),
                homepage: Some("https://www.twitch.tv".to_string()),
                start_pages: vec![
                    "https://www.twitch.tv".to_string(),
                    "https://www.youtube.com".to_string(),
                    "https://discord.com".to_string(),
                ],
                keyboard_shortcuts: HashMap::new(),
                privacy: PrivacySettings {
                    block_trackers: false,
                    block_ads: false,
                    clear_cookies_on_exit: false,
                    clear_history_on_exit: false,
                    private_mode_by_default: false,
                    disable_javascript: false,
                },
                performance: PerformanceSettings {
                    hardware_acceleration: true,
                    memory_limit: Some(4096),
                    cpu_priority: CPUPriority::High,
                    cache_size: 1024,
                },
            },
            extensions: vec![
                "twitch-enhancer".to_string(),
                "discord-rpc".to_string(),
            ],
            theme: Some("dark".to_string()),
        };

        // Privacy template
        let privacy_template = ProfileTemplate {
            name: "Privacy".to_string(),
            description: "Maximum privacy and security".to_string(),
            category: TemplateCategory::Privacy,
            settings: TemplateSettings {
                search_engine: Some("https://duckduckgo.com/?q=".to_string()),
                homepage: Some("https://duckduckgo.com".to_string()),
                start_pages: vec![
                    "https://duckduckgo.com".to_string(),
                ],
                keyboard_shortcuts: HashMap::new(),
                privacy: PrivacySettings {
                    block_trackers: true,
                    block_ads: true,
                    clear_cookies_on_exit: true,
                    clear_history_on_exit: true,
                    private_mode_by_default: true,
                    disable_javascript: false,
                },
                performance: PerformanceSettings {
                    hardware_acceleration: true,
                    memory_limit: None,
                    cpu_priority: CPUPriority::Normal,
                    cache_size: 256,
                },
            },
            extensions: vec![
                "privacy-badger".to_string(),
                "https-everywhere".to_string(),
                "ublock-origin".to_string(),
            ],
            theme: Some("dark".to_string()),
        };

        // Developer template
        let developer_template = ProfileTemplate {
            name: "Developer".to_string(),
            description: "Optimized for web development and debugging".to_string(),
            category: TemplateCategory::Developer,
            settings: TemplateSettings {
                search_engine: Some("https://www.google.com/search?q=".to_string()),
                homepage: Some("https://github.com".to_string()),
                start_pages: vec![
                    "https://github.com".to_string(),
                    "https://stackoverflow.com".to_string(),
                    "https://developer.mozilla.org".to_string(),
                ],
                keyboard_shortcuts: {
                    let mut shortcuts = HashMap::new();
                    shortcuts.insert("F12".to_string(), "toggle-devtools".to_string());
                    shortcuts.insert("Ctrl+Shift+I".to_string(), "toggle-devtools".to_string());
                    shortcuts
                },
                privacy: PrivacySettings {
                    block_trackers: false,
                    block_ads: false,
                    clear_cookies_on_exit: false,
                    clear_history_on_exit: false,
                    private_mode_by_default: false,
                    disable_javascript: false,
                },
                performance: PerformanceSettings {
                    hardware_acceleration: true,
                    memory_limit: Some(8192),
                    cpu_priority: CPUPriority::High,
                    cache_size: 2048,
                },
            },
            extensions: vec![
                "react-devtools".to_string(),
                "vue-devtools".to_string(),
                "redux-devtools".to_string(),
                "lighthouse".to_string(),
            ],
            theme: Some("dark".to_string()),
        };

        self.templates.insert("work".to_string(), work_template);
        self.templates.insert("gaming".to_string(), gaming_template);
        self.templates.insert("privacy".to_string(), privacy_template);
        self.templates.insert("developer".to_string(), developer_template);
    }

    /// Gets a template by name
    pub fn get(&self, name: &str) -> Option<&ProfileTemplate> {
        self.templates.get(name)
    }

    /// Gets all templates
    pub fn get_all(&self) -> Vec<&ProfileTemplate> {
        self.templates.values().collect()
    }

    /// Gets templates by category
    pub fn get_by_category(&self, category: &TemplateCategory) -> Vec<&ProfileTemplate> {
        self.templates
            .values()
            .filter(|t| &t.category == category)
            .collect()
    }

    /// Adds a custom template
    pub fn add(&mut self, template: ProfileTemplate) -> Result<()> {
        let id = template.name.to_lowercase().replace(' ', "-");
        
        if self.templates.contains_key(&id) {
            return Err(anyhow::anyhow!("Template already exists: {}", id));
        }

        self.templates.insert(id, template);
        Ok(())
    }

    /// Removes a template
    pub fn remove(&mut self, name: &str) -> Result<()> {
        if !self.templates.contains_key(name) {
            return Err(anyhow::anyhow!("Template not found: {}", name));
        }

        self.templates.remove(name);
        Ok(())
    }

    /// Exports a template to JSON
    pub fn export(&self, name: &str) -> Result<String> {
        let template = self.get(name)
            .ok_or_else(|| anyhow::anyhow!("Template not found: {}", name))?;

        serde_json::to_string_pretty(template)
            .map_err(|e| anyhow::anyhow!("Failed to export template: {}", e))
    }

    /// Imports a template from JSON
    pub fn import(&mut self, json: &str) -> Result<String> {
        let template: ProfileTemplate = serde_json::from_str(json)
            .map_err(|e| anyhow::anyhow!("Failed to import template: {}", e))?;

        let id = template.name.to_lowercase().replace(' ', "-");
        self.templates.insert(id.clone(), template);

        Ok(id)
    }
}

impl Default for TemplateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_manager_creation() {
        let manager = TemplateManager::new();
        assert_eq!(manager.get_all().len(), 4);
    }

    #[test]
    fn test_get_template() {
        let manager = TemplateManager::new();
        let template = manager.get("work");
        assert!(template.is_some());
        assert_eq!(template.unwrap().name, "Work");
    }

    #[test]
    fn test_get_by_category() {
        let manager = TemplateManager::new();
        let work_templates = manager.get_by_category(&TemplateCategory::Work);
        assert_eq!(work_templates.len(), 1);
    }

    #[test]
    fn test_add_custom_template() {
        let mut manager = TemplateManager::new();
        let template = ProfileTemplate {
            name: "Custom".to_string(),
            description: "Custom template".to_string(),
            category: TemplateCategory::Custom("custom".to_string()),
            settings: TemplateSettings {
                search_engine: None,
                homepage: None,
                start_pages: vec![],
                keyboard_shortcuts: HashMap::new(),
                privacy: PrivacySettings {
                    block_trackers: false,
                    block_ads: false,
                    clear_cookies_on_exit: false,
                    clear_history_on_exit: false,
                    private_mode_by_default: false,
                    disable_javascript: false,
                },
                performance: PerformanceSettings {
                    hardware_acceleration: true,
                    memory_limit: None,
                    cpu_priority: CPUPriority::Normal,
                    cache_size: 512,
                },
            },
            extensions: vec![],
            theme: None,
        };

        let result = manager.add(template);
        assert!(result.is_ok());
        assert_eq!(manager.get_all().len(), 5);
    }

    #[test]
    fn test_export_import_template() {
        let manager = TemplateManager::new();
        let json = manager.export("work").unwrap();

        let mut new_manager = TemplateManager::new();
        let id = new_manager.import(&json).unwrap();

        assert_eq!(id, "work");
    }
}