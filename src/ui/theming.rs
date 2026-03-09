//! Theme Manager
//! 
//! Dynamic theming system:
//! - Ambient Chameleon (automatic color adaptation)
//! - Light/Dark modes
//! - Custom themes
//! - User preferences

use anyhow::Result;
use log::info;
use serde::{Deserialize, Serialize};

/// Theme Manager
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
    /// Create a new theme manager
    pub fn new() -> Result<Self> {
        info!("Initializing Theme Manager...");
        
        let current_theme = Theme::default();
        
        Ok(Self {
            current_theme,
            ambient_chameleon: true,
        })
    }
    
    /// Get current theme
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
    
    /// Get theme as CSS variables
    pub fn to_css_variables(&self) -> String {
        format!(
            r##"
:root {{
    --color-background: {};
    --color-foreground: {};
    --color-primary: {};
    --color-secondary: {};
    --color-accent: {};
    --color-surface: {};
    --color-error: {};
    --color-warning: {};
    --color-success: {};
    --font-primary: {};
    --font-monospace: {};
    --font-size-base: {}px;
    --font-size-heading: {}px;
}}
"##,
            self.current_theme.colors.background,
            self.current_theme.colors.foreground,
            self.current_theme.colors.primary,
            self.current_theme.colors.secondary,
            self.current_theme.colors.accent,
            self.current_theme.colors.surface,
            self.current_theme.colors.error,
            self.current_theme.colors.warning,
            self.current_theme.colors.success,
            self.current_theme.fonts.primary,
            self.current_theme.fonts.monospace,
            self.current_theme.fonts.size_base,
            self.current_theme.fonts.size_heading,
        )
    }
}