//! Setup Wizard Module
//! 
//! Provides first-run setup wizard functionality for VantisWeb installation.

use serde::{Serialize, Deserialize};
use std::path::PathBuf;

/// Setup wizard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupConfig {
    /// Enable setup wizard
    pub enabled: bool,
    /// Skip welcome page
    pub skip_welcome: bool,
    /// Default install path
    pub default_path: PathBuf,
    /// Allow custom path
    pub allow_custom_path: bool,
    /// Show license agreement
    pub show_license: bool,
    /// Show component selection
    pub show_components: bool,
    /// Default components
    pub default_components: Vec<Component>,
}

impl Default for SetupConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            skip_welcome: false,
            default_path: PathBuf::from("C:\\Program Files\\VantisWeb"),
            allow_custom_path: true,
            show_license: true,
            show_components: true,
            default_components: vec![
                Component::Core,
                Component::Shortcuts,
                Component::DefaultBrowser,
            ],
        }
    }
}

/// Setup step enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SetupStep {
    Welcome,
    License,
    Path,
    Components,
    Shortcuts,
    DefaultBrowser,
    Installing,
    Complete,
    Error,
}

/// Installable components
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Component {
    /// Core browser files
    Core,
    /// Desktop and start menu shortcuts
    Shortcuts,
    /// Set as default browser
    DefaultBrowser,
    /// PDF viewer component
    PdfViewer,
    /// Ad blocker
    AdBlocker,
    /// Developer tools
    DeveloperTools,
    /// Accessibility features
    Accessibility,
    /// Language packs
    LanguagePacks,
}

impl Component {
    pub fn display_name(&self) -> &'static str {
        match self {
            Component::Core => "VantisWeb Core (Required)",
            Component::Shortcuts => "Desktop & Start Menu Shortcuts",
            Component::DefaultBrowser => "Set as Default Browser",
            Component::PdfViewer => "Built-in PDF Viewer",
            Component::AdBlocker => "Ad Blocker",
            Component::DeveloperTools => "Developer Tools",
            Component::Accessibility => "Accessibility Features",
            Component::LanguagePacks => "Additional Language Packs",
        }
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            Component::Core => "Essential browser files and components",
            Component::Shortcuts => "Create shortcuts for easy access to VantisWeb",
            Component::DefaultBrowser => "Set VantisWeb as your default web browser",
            Component::PdfViewer => "View PDF files directly in the browser",
            Component::AdBlocker => "Block advertisements and tracking scripts",
            Component::DeveloperTools => "Web development and debugging tools",
            Component::Accessibility => "Screen reader support, high contrast mode, and more",
            Component::LanguagePacks => "Install additional interface languages",
        }
    }
    
    pub fn required(&self) -> bool {
        matches!(self, Component::Core)
    }
    
    pub fn size_mb(&self) -> u32 {
        match self {
            Component::Core => 85,
            Component::Shortcuts => 1,
            Component::DefaultBrowser => 0,
            Component::PdfViewer => 15,
            Component::AdBlocker => 5,
            Component::DeveloperTools => 10,
            Component::Accessibility => 8,
            Component::LanguagePacks => 25,
        }
    }
}

/// Setup wizard state
#[derive(Debug, Clone)]
pub struct SetupState {
    /// Current step
    pub current_step: SetupStep,
    /// Selected components
    pub selected_components: Vec<Component>,
    /// Install path
    pub install_path: PathBuf,
    /// License accepted
    pub license_accepted: bool,
    /// Progress percentage (0-100)
    pub progress: u8,
    /// Current file being installed
    pub current_file: String,
    /// Error message if any
    pub error: Option<String>,
}

impl Default for SetupState {
    fn default() -> Self {
        Self {
            current_step: SetupStep::Welcome,
            selected_components: vec![Component::Core],
            install_path: PathBuf::from("C:\\Program Files\\VantisWeb"),
            license_accepted: false,
            progress: 0,
            current_file: String::new(),
            error: None,
        }
    }
}

/// Setup wizard
pub struct SetupWizard {
    /// Configuration
    config: SetupConfig,
    /// Current state
    state: SetupState,
}

impl SetupWizard {
    /// Create a new setup wizard
    pub fn new(config: SetupConfig) -> Self {
        let mut state = SetupState::default();
        state.selected_components = config.default_components.clone();
        state.install_path = config.default_path.clone();
        
        Self { config, state }
    }
    
    /// Get current step
    pub fn current_step(&self) -> SetupStep {
        self.state.current_step
    }
    
    /// Get current state
    pub fn state(&self) -> &SetupState {
        &self.state
    }
    
    /// Advance to next step
    pub fn next_step(&mut self) -> bool {
        self.state.current_step = match self.state.current_step {
            SetupStep::Welcome => {
                if self.config.skip_welcome {
                    SetupStep::License
                } else {
                    SetupStep::License
                }
            }
            SetupStep::License => {
                if self.state.license_accepted {
                    SetupStep::Path
                } else {
                    return false;
                }
            }
            SetupStep::Path => SetupStep::Components,
            SetupStep::Components => SetupStep::Shortcuts,
            SetupStep::Shortcuts => SetupStep::DefaultBrowser,
            SetupStep::DefaultBrowser => SetupStep::Installing,
            SetupStep::Installing => SetupStep::Complete,
            SetupStep::Complete => return false,
            SetupStep::Error => return false,
        };
        true
    }
    
    /// Go to previous step
    pub fn previous_step(&mut self) {
        self.state.current_step = match self.state.current_step {
            SetupStep::Welcome => SetupStep::Welcome,
            SetupStep::License => SetupStep::Welcome,
            SetupStep::Path => SetupStep::License,
            SetupStep::Components => SetupStep::Path,
            SetupStep::Shortcuts => SetupStep::Components,
            SetupStep::DefaultBrowser => SetupStep::Shortcuts,
            SetupStep::Installing => SetupStep::Installing,
            SetupStep::Complete => SetupStep::Complete,
            SetupStep::Error => SetupStep::Error,
        };
    }
    
    /// Set install path
    pub fn set_install_path(&mut self, path: PathBuf) {
        self.state.install_path = path;
    }
    
    /// Accept license
    pub fn accept_license(&mut self) {
        self.state.license_accepted = true;
    }
    
    /// Toggle component selection
    pub fn toggle_component(&mut self, component: Component) {
        if component.required() {
            return; // Cannot deselect required components
        }
        
        if self.state.selected_components.contains(&component) {
            self.state.selected_components.retain(|c| c != &component);
        } else {
            self.state.selected_components.push(component);
        }
    }
    
    /// Check if component is selected
    pub fn is_component_selected(&self, component: &Component) -> bool {
        self.state.selected_components.contains(component)
    }
    
    /// Calculate total install size
    pub fn total_size_mb(&self) -> u32 {
        self.state.selected_components.iter()
            .map(|c| c.size_mb())
            .sum()
    }
    
    /// Update installation progress
    pub fn update_progress(&mut self, progress: u8, current_file: &str) {
        self.state.progress = progress.min(100);
        self.state.current_file = current_file.to_string();
    }
    
    /// Set error
    pub fn set_error(&mut self, error: &str) {
        self.state.current_step = SetupStep::Error;
        self.state.error = Some(error.to_string());
    }
    
    /// Check if can proceed
    pub fn can_proceed(&self) -> bool {
        match self.state.current_step {
            SetupStep::License => self.state.license_accepted,
            SetupStep::Path => {
                // Check if path is valid
                !self.state.install_path.as_os_str().is_empty()
            }
            SetupStep::Components => {
                // Must have at least core
                self.state.selected_components.contains(&Component::Core)
            }
            _ => true,
        }
    }
    
    /// Run the wizard (UI would call this)
    pub fn run(&mut self) -> Result<(), String> {
        // This would be implemented with actual UI
        // For now, just validate state
        if !self.state.license_accepted && self.config.show_license {
            return Err("License must be accepted".to_string());
        }
        
        if !self.state.selected_components.contains(&Component::Core) {
            return Err("Core component is required".to_string());
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_setup_config_default() {
        let config = SetupConfig::default();
        assert!(config.enabled);
        assert!(config.show_license);
    }
    
    #[test]
    fn test_component_info() {
        assert!(Component::Core.required());
        assert!(!Component::Shortcuts.required());
        assert!(Component::Core.size_mb() > 0);
    }
    
    #[test]
    fn test_setup_wizard_steps() {
        let config = SetupConfig::default();
        let mut wizard = SetupWizard::new(config);
        
        assert_eq!(wizard.current_step(), SetupStep::Welcome);
        
        wizard.accept_license();
        assert!(wizard.can_proceed());
    }
    
    #[test]
    fn test_component_selection() {
        let config = SetupConfig::default();
        let mut wizard = SetupWizard::new(config);
        
        // Cannot deselect core
        wizard.toggle_component(Component::Core);
        assert!(wizard.is_component_selected(&Component::Core));
        
        // Can toggle other components
        wizard.toggle_component(Component::PdfViewer);
        assert!(wizard.is_component_selected(&Component::PdfViewer));
        
        wizard.toggle_component(Component::PdfViewer);
        assert!(!wizard.is_component_selected(&Component::PdfViewer));
    }
    
    #[test]
    fn test_total_size() {
        let config = SetupConfig::default();
        let wizard = SetupWizard::new(config);
        
        let size = wizard.total_size_mb();
        assert!(size > 0);
    }
}