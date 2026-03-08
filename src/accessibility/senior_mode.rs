//! Senior Mode Module
//! 
//! Provides a simplified, accessible interface optimized for elderly users
//! with large icons, high contrast, and voice assistance.

use std::collections::HashMap;
use serde::{Serialize, Deserialize];
use crate::accessibility::{AccessibilityError, AccessibilityResult};

/// Senior mode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeniorConfig {
    /// Enable senior mode
    pub enabled: bool,
    /// Font size
    pub font_size: FontSize,
    /// UI layout style
    pub layout: UILayout,
    /// High contrast mode
    pub high_contrast: bool,
    /// Enable voice assistance
    pub voice_assistance: bool,
    /// Simplified navigation
    pub simplified_nav: bool,
    /// Large icons
    pub large_icons: bool,
    /// Show labels on icons
    pub show_labels: bool,
    /// Reduce animations
    pub reduce_animations: bool,
    /// Auto-read page content
    pub auto_read: bool,
    /// Confirm before actions
    pub confirm_actions: bool,
    /// Highlight focused elements
    pub highlight_focus: bool,
    /// Custom color scheme
    pub color_scheme: ColorScheme,
}

impl Default for SeniorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            font_size: FontSize::ExtraLarge,
            layout: UILayout::Simplified,
            high_contrast: true,
            voice_assistance: true,
            simplified_nav: true,
            large_icons: true,
            show_labels: true,
            reduce_animations: true,
            auto_read: false,
            confirm_actions: true,
            highlight_focus: true,
            color_scheme: ColorScheme::HighContrast,
        }
    }
}

/// Font size options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontSize {
    /// Normal size (16px base)
    Normal,
    /// Large (20px base)
    Large,
    /// Extra Large (24px base)
    ExtraLarge,
    /// Huge (32px base)
    Huge,
}

impl FontSize {
    /// Get the base font size in pixels
    pub fn base_size(&self) -> u32 {
        match self {
            FontSize::Normal => 16,
            FontSize::Large => 20,
            FontSize::ExtraLarge => 24,
            FontSize::Huge => 32,
        }
    }
    
    /// Get scaled size for a specific element type
    pub fn scaled_size(&self, element: ElementSize) -> u32 {
        let base = self.base_size();
        match element {
            ElementSize::Small => base,
            ElementSize::Medium => (base as f32 * 1.25) as u32,
            ElementSize::Large => (base as f32 * 1.5) as u32,
            ElementSize::Title => (base as f32 * 2.0) as u32,
        }
    }
}

/// Element size categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementSize {
    Small,
    Medium,
    Large,
    Title,
}

/// UI Layout style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UILayout {
    /// Standard layout
    Standard,
    /// Simplified with fewer elements
    Simplified,
    /// Single column layout
    SingleColumn,
    /// Large tiles layout
    Tiles,
}

/// Color scheme options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorScheme {
    /// Default browser colors
    Default,
    /// High contrast black and white
    HighContrast,
    /// Dark mode with yellow accent
    DarkYellow,
    /// Light mode with blue accent
    LightBlue,
    /// Inverted colors
    Inverted,
    /// Custom colors
    Custom,
}

impl ColorScheme {
    /// Get CSS variables for this color scheme
    pub fn css_variables(&self) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        
        match self {
            ColorScheme::Default => {
                vars.insert("--bg-primary".into(), "#ffffff".into());
                vars.insert("--bg-secondary".into(), "#f5f5f5".into());
                vars.insert("--text-primary".into(), "#000000".into());
                vars.insert("--text-secondary".into(), "#666666".into());
                vars.insert("--accent".into(), "#0066cc".into());
                vars.insert("--focus-ring".into(), "#0066cc".into());
            }
            ColorScheme::HighContrast => {
                vars.insert("--bg-primary".into(), "#ffffff".into());
                vars.insert("--bg-secondary".into(), "#000000".into());
                vars.insert("--text-primary".into(), "#000000".into());
                vars.insert("--text-secondary".into(), "#333333".into());
                vars.insert("--accent".into(), "#0000ff".into());
                vars.insert("--focus-ring".into(), "#ff0000".into());
            }
            ColorScheme::DarkYellow => {
                vars.insert("--bg-primary".into(), "#1a1a1a".into());
                vars.insert("--bg-secondary".into(), "#2a2a2a".into());
                vars.insert("--text-primary".into(), "#ffff00".into());
                vars.insert("--text-secondary".into(), "#cccc00".into());
                vars.insert("--accent".into(), "#ffff00".into());
                vars.insert("--focus-ring".into(), "#ffff00".into());
            }
            ColorScheme::LightBlue => {
                vars.insert("--bg-primary".into(), "#e6f3ff".into());
                vars.insert("--bg-secondary".into(), "#cce6ff".into());
                vars.insert("--text-primary".into(), "#003366".into());
                vars.insert("--text-secondary".into(), "#336699".into());
                vars.insert("--accent".into(), "#0066cc".into());
                vars.insert("--focus-ring".into(), "#0066cc".into());
            }
            ColorScheme::Inverted => {
                vars.insert("--bg-primary".into(), "#000000".into());
                vars.insert("--bg-secondary".into(), "#1a1a1a".into());
                vars.insert("--text-primary".into(), "#ffffff".into());
                vars.insert("--text-secondary".into(), "#cccccc".into());
                vars.insert("--accent".into(), "#00ccff".into());
                vars.insert("--focus-ring".into(), "#00ccff".into());
            }
            ColorScheme::Custom => {
                // Custom colors would be set separately
            }
        }
        
        vars
    }
}

/// Simplified menu item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplifiedMenuItem {
    /// Item ID
    pub id: String,
    /// Display label
    pub label: String,
    /// Icon name
    pub icon: String,
    /// Description for voice assistance
    pub description: String,
    /// Keyboard shortcut
    pub shortcut: Option<String>,
    /// Is enabled
    pub enabled: bool,
}

impl SimplifiedMenuItem {
    pub fn new(id: &str, label: &str, icon: &str, description: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            icon: icon.to_string(),
            description: description.to_string(),
            shortcut: None,
            enabled: true,
        }
    }
    
    pub fn with_shortcut(mut self, shortcut: &str) -> Self {
        self.shortcut = Some(shortcut.to_string());
        self
    }
}

/// Senior mode statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SeniorModeStats {
    /// Time active in seconds
    pub active_time: f64,
    /// Actions performed
    pub actions_performed: u64,
    /// Voice commands used
    pub voice_commands_used: u64,
    /// Errors encountered
    pub errors_encountered: u64,
    /// Help requested count
    pub help_requested: u64,
}

/// Main senior mode struct
pub struct SeniorMode {
    /// Configuration
    config: SeniorConfig,
    /// Simplified menu items
    menu_items: Vec<SimplifiedMenuItem>,
    /// Quick access items
    quick_access: Vec<SimplifiedMenuItem>,
    /// Statistics
    stats: SeniorModeStats,
    /// Is currently active
    is_active: bool,
    /// Confirmation dialog state
    pending_confirmation: Option<PendingAction>,
}

/// Pending action awaiting confirmation
#[derive(Debug, Clone)]
pub struct PendingAction {
    /// Action ID
    pub id: String,
    /// Action description
    pub description: String,
    /// Callback action type
    pub action_type: ActionType,
    /// Created timestamp
    pub created: std::time::Instant,
}

/// Types of actions that can be confirmed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    Navigate,
    Close,
    Delete,
    Download,
    Submit,
    Clear,
}

impl SeniorMode {
    /// Create a new senior mode
    pub fn new(config: SeniorConfig) -> Self {
        let mut mode = Self {
            config,
            menu_items: Vec::new(),
            quick_access: Vec::new(),
            stats: SeniorModeStats::default(),
            is_active: false,
            pending_confirmation: None,
        };
        
        mode.initialize_menu_items();
        mode
    }
    
    /// Initialize default menu items
    fn initialize_menu_items(&mut self) {
        // Main navigation items
        self.menu_items = vec![
            SimplifiedMenuItem::new("home", "Home", "home", "Go to your home page")
                .with_shortcut("Alt+H"),
            SimplifiedMenuItem::new("back", "Go Back", "arrow-left", "Go back to previous page")
                .with_shortcut("Alt+Left"),
            SimplifiedMenuItem::new("forward", "Go Forward", "arrow-right", "Go forward to next page")
                .with_shortcut("Alt+Right"),
            SimplifiedMenuItem::new("search", "Search", "search", "Search the web")
                .with_shortcut("Ctrl+K"),
            SimplifiedMenuItem::new("bookmarks", "Bookmarks", "star", "View your saved pages")
                .with_shortcut("Ctrl+B"),
            SimplifiedMenuItem::new("history", "History", "clock", "View pages you visited")
                .with_shortcut("Ctrl+H"),
            SimplifiedMenuItem::new("zoom-in", "Zoom In", "zoom-in", "Make text and images bigger")
                .with_shortcut("Ctrl++"),
            SimplifiedMenuItem::new("zoom-out", "Zoom Out", "zoom-out", "Make text and images smaller")
                .with_shortcut("Ctrl+-"),
            SimplifiedMenuItem::new("read", "Read Aloud", "volume-up", "Have the page read to you")
                .with_shortcut("Ctrl+R"),
            SimplifiedMenuItem::new("help", "Help", "help", "Get help using the browser")
                .with_shortcut("F1"),
        ];
        
        // Quick access items (shown prominently)
        self.quick_access = vec![
            SimplifiedMenuItem::new("google", "Google", "search", "Search with Google"),
            SimplifiedMenuItem::new("youtube", "YouTube", "video", "Watch videos on YouTube"),
            SimplifiedMenuItem::new("news", "News", "newspaper", "Read the latest news"),
            SimplifiedMenuItem::new("email", "Email", "email", "Check your email"),
            SimplifiedMenuItem::new("weather", "Weather", "cloud", "Check the weather"),
        ];
    }
    
    /// Activate senior mode
    pub fn activate(&mut self) -> AccessibilityResult<()> {
        self.is_active = true;
        Ok(())
    }
    
    /// Deactivate senior mode
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }
    
    /// Check if active
    pub fn is_active(&self) -> bool {
        self.is_active && self.config.enabled
    }
    
    /// Get configuration
    pub fn get_config(&self) -> &SeniorConfig {
        &self.config
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: SeniorConfig) {
        self.config = config;
    }
    
    /// Get menu items
    pub fn get_menu_items(&self) -> &[SimplifiedMenuItem] {
        &self.menu_items
    }
    
    /// Get quick access items
    pub fn get_quick_access(&self) -> &[SimplifiedMenuItem] {
        &self.quick_access
    }
    
    /// Add quick access item
    pub fn add_quick_access(&mut self, item: SimplifiedMenuItem) {
        self.quick_access.push(item);
    }
    
    /// Remove quick access item
    pub fn remove_quick_access(&mut self, id: &str) {
        self.quick_access.retain(|item| item.id != id);
    }
    
    /// Request confirmation for an action
    pub fn request_confirmation(&mut self, action_type: ActionType, description: &str) -> String {
        if !self.config.confirm_actions {
            // If confirmations disabled, return empty string (auto-confirmed)
            return String::new();
        }
        
        let pending = PendingAction {
            id: uuid_string(),
            description: description.to_string(),
            action_type,
            created: std::time::Instant::now(),
        };
        
        let id = pending.id.clone();
        self.pending_confirmation = Some(pending);
        
        id
    }
    
    /// Confirm pending action
    pub fn confirm_action(&mut self, id: &str) -> AccessibilityResult<bool> {
        if let Some(ref pending) = self.pending_confirmation {
            if pending.id == id {
                self.pending_confirmation = None;
                self.stats.actions_performed += 1;
                return Ok(true);
            }
        }
        Ok(false)
    }
    
    /// Cancel pending action
    pub fn cancel_action(&mut self) {
        self.pending_confirmation = None;
    }
    
    /// Get pending confirmation
    pub fn get_pending_confirmation(&self) -> Option<&PendingAction> {
        self.pending_confirmation.as_ref()
    }
    
    /// Get CSS for current settings
    pub fn get_css(&self) -> String {
        let font_size = self.config.font_size.base_size();
        let vars = self.config.color_scheme.css_variables();
        
        let mut css = format!(
            ":root {{\n\
             --font-size-base: {}px;\n\
             --icon-size: {}px;\n",
            font_size,
            if self.config.large_icons { 48 } else { 32 }
        );
        
        for (key, value) in vars {
            css.push_str(&format!("  {}: {};\n", key, value));
        }
        
        css.push_str("}\n\n");
        
        // Additional senior mode CSS
        if self.config.reduce_animations {
            css.push_str("*, *::before, *::after {\n");
            css.push_str("  animation-duration: 0.01ms !important;\n");
            css.push_str("  transition-duration: 0.01ms !important;\n");
            css.push_str("}\n\n");
        }
        
        if self.config.highlight_focus {
            css.push_str(":focus {\n");
            css.push_str("  outline: 3px solid var(--focus-ring) !important;\n");
            css.push_str("  outline-offset: 2px !important;\n");
            css.push_str("}\n\n");
        }
        
        css
    }
    
    /// Record voice command usage
    pub fn record_voice_command(&mut self) {
        self.stats.voice_commands_used += 1;
    }
    
    /// Record error
    pub fn record_error(&mut self) {
        self.stats.errors_encountered += 1;
    }
    
    /// Record help request
    pub fn record_help_request(&mut self) {
        self.stats.help_requested += 1;
    }
    
    /// Get statistics
    pub fn get_stats(&self) -> &SeniorModeStats {
        &self.stats
    }
}

/// Simple UUID generation
fn uuid_string() -> String {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", nanos)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_senior_config_default() {
        let config = SeniorConfig::default();
        assert!(config.high_contrast);
        assert!(config.large_icons);
        assert_eq!(config.font_size, FontSize::ExtraLarge);
    }
    
    #[test]
    fn test_font_size_scaling() {
        assert_eq!(FontSize::Normal.base_size(), 16);
        assert_eq!(FontSize::ExtraLarge.base_size(), 24);
        assert_eq!(FontSize::Huge.base_size(), 32);
    }
    
    #[test]
    fn test_senior_mode_creation() {
        let config = SeniorConfig::default();
        let mode = SeniorMode::new(config);
        assert!(!mode.is_active());
    }
    
    #[test]
    fn test_senior_mode_activation() {
        let config = SeniorConfig { enabled: true, ..Default::default() };
        let mut mode = SeniorMode::new(config);
        mode.activate().unwrap();
        assert!(mode.is_active());
    }
    
    #[test]
    fn test_menu_items() {
        let config = SeniorConfig::default();
        let mode = SeniorMode::new(config);
        assert!(!mode.get_menu_items().is_empty());
        assert!(!mode.get_quick_access().is_empty());
    }
    
    #[test]
    fn test_confirmation_flow() {
        let config = SeniorConfig { confirm_actions: true, ..Default::default() };
        let mut mode = SeniorMode::new(config);
        
        let id = mode.request_confirmation(ActionType::Navigate, "Go to example.com");
        assert!(!id.is_empty());
        assert!(mode.get_pending_confirmation().is_some());
        
        mode.cancel_action();
        assert!(mode.get_pending_confirmation().is_none());
    }
    
    #[test]
    fn test_css_generation() {
        let config = SeniorConfig::default();
        let mode = SeniorMode::new(config);
        
        let css = mode.get_css();
        assert!(css.contains("--font-size-base"));
        assert!(css.contains("--icon-size"));
    }
    
    #[test]
    fn test_color_scheme_css() {
        let vars = ColorScheme::HighContrast.css_variables();
        assert_eq!(vars.get("--bg-primary").unwrap(), "#ffffff");
        assert_eq!(vars.get("--text-primary").unwrap(), "#000000");
    }
}