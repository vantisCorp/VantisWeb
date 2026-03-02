//! Theme Manager (Optimized)
//! 
//! Dynamic theming system:
//! - Ambient Chameleon (automatic color adaptation)
//! - Light/Dark modes
//! - Custom themes
//! - User preferences
//!
//! Optimizations:
//! - Pre-allocated string capacities
//! - Reduced clones
//! - Optimized string operations

use anyhow::Result;
use log::info;
use serde::{Deserialize, Serialize};

/// Theme Manager (Optimized)
pub struct ThemeManager {
    current_theme: Theme,
    ambient_chameleon: bool,
}

/// Theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub mode: ThemeMode,
    pub colors: ColorScheme,
    pub fonts: FontScheme,
}

/// Theme mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThemeMode {
    Light,
    Dark,
    Auto,
}

/// Color scheme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub background: String,
    pub foreground: String,
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub surface: String,
    pub error: String,
    pub warning: String,
    pub success: String,
}

/// Font scheme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontScheme {
    pub primary: String,
    pub monospace: String,
    pub size_base: u8,
    pub size_heading: u8,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "Vantis Dark".to_string(),
            mode: ThemeMode::Dark,
            colors: ColorScheme {
                background: "#1a1a2e".to_string(),
                foreground: "#eaeaea".to_string(),
                primary: "#16213e".to_string(),
                secondary: "#0f3460".to_string(),
                accent: "#e94560".to_string(),
                surface: "#262642".to_string(),
                error: "#ff4757".to_string(),
                warning: "#ffa502".to_string(),
                success: "#2ed573".to_string(),
            },
            fonts: FontScheme {
                primary: "Inter".to_string(),
                monospace: "JetBrains Mono".to_string(),
                size_base: 16,
                size_heading: 24,
            },
        }
    }
}

impl ThemeManager {
    /// Create a new theme manager (Optimized)
    pub fn new() -> Result<Self> {
        info!("Initializing Theme Manager...");
        
        let current_theme = Theme::default();
        
        Ok(Self {
            current_theme,
            ambient_chameleon: true,
        })
    }
    
    /// Get current theme (Optimized - returns reference)
    pub fn current_theme(&self) -> &Theme {
        &self.current_theme
    }
    
    /// Set theme
    pub fn set_theme(&mut self, theme: Theme) {
        info!("Switching theme: {}", theme.name);
        self.current_theme = theme;
    }
    
    /// Enable/disable Ambient Chameleon
    pub fn set_ambient_chameleon(&mut self, enabled: bool) {
        info!("Ambient Chameleon: {}", if enabled { "enabled" } else { "disabled" });
        self.ambient_chameleon = enabled;
    }
    
    /// Get theme as CSS variables (Optimized)
    pub fn to_css_variables(&self) -> String {
        let mut css = String::with_capacity(500);
        
        css.push_str(r#"
:root {
    --color-background: "#);
        css.push_str(&self.current_theme.colors.background);
        css.push_str(r#";
    --color-foreground: "#);
        css.push_str(&self.current_theme.colors.foreground);
        css.push_str(r#";
    --color-primary: "#);
        css.push_str(&self.current_theme.colors.primary);
        css.push_str(r#";
    --color-secondary: "#);
        css.push_str(&self.current_theme.colors.secondary);
        css.push_str(r#";
    --color-accent: "#);
        css.push_str(&self.current_theme.colors.accent);
        css.push_str(r#";
    --color-surface: "#);
        css.push_str(&self.current_theme.colors.surface);
        css.push_str(r#";
    --color-error: "#);
        css.push_str(&self.current_theme.colors.error);
        css.push_str(r#";
    --color-warning: "#);
        css.push_str(&self.current_theme.colors.warning);
        css.push_str(r#";
    --color-success: "#);
        css.push_str(&self.current_theme.colors.success);
        css.push_str(r#";
    --font-primary: "#);
        css.push_str(&self.current_theme.fonts.primary);
        css.push_str(r#";
    --font-monospace: "#);
        css.push_str(&self.current_theme.fonts.monospace);
        css.push_str(r#";
    --font-size-base: "#);
        css.push_str(&self.current_theme.fonts.size_base.to_string());
        css.push_str(r#"px;
    --font-size-heading: "#);
        css.push_str(&self.current_theme.fonts.size_heading.to_string());
        css.push_str(r#"px;
}
"#);
        
        css
    }
    
    /// Is Ambient Chameleon enabled
    pub fn is_ambient_chameleon_enabled(&self) -> bool {
        self.ambient_chameleon
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_manager_creation() {
        let manager = ThemeManager::new().unwrap();
        
        assert_eq!(manager.current_theme().name, "Vantis Dark");
        assert!(manager.is_ambient_chameleon_enabled());
    }

    #[test]
    fn test_set_theme() {
        let mut manager = ThemeManager::new().unwrap();
        
        let theme = Theme {
            name: "Custom Theme".to_string(),
            mode: ThemeMode::Light,
            colors: ColorScheme {
                background: "#ffffff".to_string(),
                foreground: "#000000".to_string(),
                primary: "#0000ff".to_string(),
                secondary: "#00ff00".to_string(),
                accent: "#ff0000".to_string(),
                surface: "#f0f0f0".to_string(),
                error: "#ff0000".to_string(),
                warning: "#ffaa00".to_string(),
                success: "#00ff00".to_string(),
            },
            fonts: FontScheme {
                primary: "Arial".to_string(),
                monospace: "Courier".to_string(),
                size_base: 14,
                size_heading: 20,
            },
        };
        
        manager.set_theme(theme);
        
        assert_eq!(manager.current_theme().name, "Custom Theme");
    }

    #[test]
    fn test_to_css_variables() {
        let manager = ThemeManager::new().unwrap();
        
        let css = manager.to_css_variables();
        
        assert!(css.contains("--color-background"));
        assert!(css.contains("--font-primary"));
    }
}