//! Theme Module
//! 
//! This module provides theme customization for the reading mode,
//! including colors, fonts, and layout options.
//! 
//! # Features
//! - Predefined themes (Light, Dark, Sepia, High Contrast)
//! - Custom theme support
//! - Font selection
//! - Color customization

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::ReadingTheme;

/// Theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub name: String,
    pub preset: ReadingTheme,
    pub colors: ThemeColors,
    pub typography: TypographySettings,
    pub layout: LayoutSettings,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: "Default Light".to_string(),
            preset: ReadingTheme::Light,
            colors: ThemeColors::light(),
            typography: TypographySettings::default(),
            layout: LayoutSettings::default(),
        }
    }
}

impl ThemeConfig {
    /// Create a light theme
    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
            preset: ReadingTheme::Light,
            colors: ThemeColors::light(),
            typography: TypographySettings::default(),
            layout: LayoutSettings::default(),
        }
    }

    /// Create a dark theme
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            preset: ReadingTheme::Dark,
            colors: ThemeColors::dark(),
            typography: TypographySettings::default(),
            layout: LayoutSettings::default(),
        }
    }

    /// Create a sepia theme
    pub fn sepia() -> Self {
        Self {
            name: "Sepia".to_string(),
            preset: ReadingTheme::Sepia,
            colors: ThemeColors::sepia(),
            typography: TypographySettings::default(),
            layout: LayoutSettings::default(),
        }
    }

    /// Create a high contrast theme
    pub fn high_contrast() -> Self {
        Self {
            name: "High Contrast".to_string(),
            preset: ReadingTheme::HighContrast,
            colors: ThemeColors::high_contrast(),
            typography: TypographySettings {
                font_family: "Arial, sans-serif".to_string(),
                ..TypographySettings::default()
            },
            layout: LayoutSettings::default(),
        }
    }
}

/// Theme colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub background: String,
    pub text: String,
    pub text_secondary: String,
    pub accent: String,
    pub link: String,
    pub border: String,
    pub highlight: String,
}

impl ThemeColors {
    /// Light theme colors
    pub fn light() -> Self {
        Self {
            background: "#ffffff".to_string(),
            text: "#333333".to_string(),
            text_secondary: "#666666".to_string(),
            accent: "#007bff".to_string(),
            link: "#0066cc".to_string(),
            border: "#eeeeee".to_string(),
            highlight: "#fff3cd".to_string(),
        }
    }

    /// Dark theme colors
    pub fn dark() -> Self {
        Self {
            background: "#1a1a1a".to_string(),
            text: "#e0e0e0".to_string(),
            text_secondary: "#999999".to_string(),
            accent: "#5c9aff".to_string(),
            link: "#5c9aff".to_string(),
            border: "#333333".to_string(),
            highlight: "#3d3d00".to_string(),
        }
    }

    /// Sepia theme colors
    pub fn sepia() -> Self {
        Self {
            background: "#f4ecd8".to_string(),
            text: "#5b4636".to_string(),
            text_secondary: "#8b7666".to_string(),
            accent: "#8b4513".to_string(),
            link: "#8b4513".to_string(),
            border: "#d4c4b0".to_string(),
            highlight: "#fff8dc".to_string(),
        }
    }

    /// High contrast theme colors
    pub fn high_contrast() -> Self {
        Self {
            background: "#ffffff".to_string(),
            text: "#000000".to_string(),
            text_secondary: "#000000".to_string(),
            accent: "#000000".to_string(),
            link: "#000000".to_string(),
            border: "#000000".to_string(),
            highlight: "#ffff00".to_string(),
        }
    }
}

/// Typography settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypographySettings {
    pub font_family: String,
    pub font_size_base: u16,
    pub line_height: f32,
    pub title_size: f32,
    pub subtitle_size: f32,
    pub body_size: f32,
    pub letter_spacing: f32,
}

impl Default for TypographySettings {
    fn default() -> Self {
        Self {
            font_family: "Georgia, serif".to_string(),
            font_size_base: 18,
            line_height: 1.6,
            title_size: 2.5,
            subtitle_size: 1.8,
            body_size: 1.1,
            letter_spacing: 0.0,
        }
    }
}

impl TypographySettings {
    /// Create typography settings for dyslexia-friendly reading
    pub fn dyslexia_friendly() -> Self {
        Self {
            font_family: "OpenDyslexic, Comic Sans MS, sans-serif".to_string(),
            font_size_base: 20,
            line_height: 1.8,
            title_size: 2.5,
            subtitle_size: 1.8,
            body_size: 1.1,
            letter_spacing: 0.1,
        }
    }

    /// Create typography settings for larger text
    pub fn large_print() -> Self {
        Self {
            font_family: "Georgia, serif".to_string(),
            font_size_base: 22,
            line_height: 1.8,
            title_size: 3.0,
            subtitle_size: 2.0,
            body_size: 1.2,
            letter_spacing: 0.0,
        }
    }

    /// Create typography settings for compact reading
    pub fn compact() -> Self {
        Self {
            font_family: "Arial, sans-serif".to_string(),
            font_size_base: 15,
            line_height: 1.4,
            title_size: 2.0,
            subtitle_size: 1.5,
            body_size: 1.0,
            letter_spacing: 0.0,
        }
    }
}

/// Layout settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutSettings {
    pub max_width: u16,
    pub margin_width: u16,
    pub padding_vertical: u16,
    pub paragraph_spacing: u16,
    pub indent_first_line: bool,
    pub center_text: bool,
}

impl Default for LayoutSettings {
    fn default() -> Self {
        Self {
            max_width: 800,
            margin_width: 60,
            padding_vertical: 40,
            paragraph_spacing: 24,
            indent_first_line: false,
            center_text: false,
        }
    }
}

impl LayoutSettings {
    /// Create wide layout
    pub fn wide() -> Self {
        Self {
            max_width: 1000,
            margin_width: 40,
            padding_vertical: 40,
            paragraph_spacing: 24,
            indent_first_line: false,
            center_text: false,
        }
    }

    /// Create narrow layout (for mobile)
    pub fn narrow() -> Self {
        Self {
            max_width: 600,
            margin_width: 20,
            padding_vertical: 20,
            paragraph_spacing: 20,
            indent_first_line: false,
            center_text: false,
        }
    }

    /// Create centered layout
    pub fn centered() -> Self {
        Self {
            max_width: 700,
            margin_width: 60,
            padding_vertical: 40,
            paragraph_spacing: 24,
            indent_first_line: false,
            center_text: true,
        }
    }
}

/// Font options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontOption {
    pub name: String,
    pub family: String,
    pub category: FontCategory,
}

/// Font category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontCategory {
    Serif,
    SansSerif,
    Monospace,
    Display,
    Handwriting,
}

/// Available fonts
pub fn get_available_fonts() -> Vec<FontOption> {
    vec![
        // Serif fonts
        FontOption {
            name: "Georgia".to_string(),
            family: "Georgia, serif".to_string(),
            category: FontCategory::Serif,
        },
        FontOption {
            name: "Times New Roman".to_string(),
            family: "&quot;Times New Roman&quot;, serif".to_string(),
            category: FontCategory::Serif,
        },
        FontOption {
            name: "Merriweather".to_string(),
            family: "Merriweather, Georgia, serif".to_string(),
            category: FontCategory::Serif,
        },
        FontOption {
            name: "Palatino".to_string(),
            family: "Palatino, &quot;Palatino Linotype&quot;, serif".to_string(),
            category: FontCategory::Serif,
        },
        // Sans-serif fonts
        FontOption {
            name: "Arial".to_string(),
            family: "Arial, Helvetica, sans-serif".to_string(),
            category: FontCategory::SansSerif,
        },
        FontOption {
            name: "Helvetica".to_string(),
            family: "Helvetica, Arial, sans-serif".to_string(),
            category: FontCategory::SansSerif,
        },
        FontOption {
            name: "Open Sans".to_string(),
            family: "&quot;Open Sans&quot;, Arial, sans-serif".to_string(),
            category: FontCategory::SansSerif,
        },
        FontOption {
            name: "Verdana".to_string(),
            family: "Verdana, Geneva, sans-serif".to_string(),
            category: FontCategory::SansSerif,
        },
        FontOption {
            name: "Roboto".to_string(),
            family: "Roboto, Arial, sans-serif".to_string(),
            category: FontCategory::SansSerif,
        },
        // Monospace fonts
        FontOption {
            name: "Courier New".to_string(),
            family: "&quot;Courier New&quot;, Courier, monospace".to_string(),
            category: FontCategory::Monospace,
        },
        FontOption {
            name: "Consolas".to_string(),
            family: "Consolas, &quot;Courier New&quot;, monospace".to_string(),
            category: FontCategory::Monospace,
        },
    ]
}

/// Theme manager
pub struct ThemeManager {
    themes: HashMap<String, ThemeConfig>,
    current_theme: String,
}

impl ThemeManager {
    /// Create a new theme manager
    pub fn new() -> Self {
        let mut themes = HashMap::new();

        // Add default themes
        let light = ThemeConfig::light();
        themes.insert(light.name.clone(), light);

        let dark = ThemeConfig::dark();
        themes.insert(dark.name.clone(), dark);

        let sepia = ThemeConfig::sepia();
        themes.insert(sepia.name.clone(), sepia);

        let high_contrast = ThemeConfig::high_contrast();
        themes.insert(high_contrast.name.clone(), high_contrast);

        Self {
            themes,
            current_theme: "Light".to_string(),
        }
    }

    /// Get current theme
    pub fn get_current_theme(&self) -> Option<&ThemeConfig> {
        self.themes.get(&self.current_theme)
    }

    /// Set current theme
    pub fn set_theme(&mut self, name: &str) -> bool {
        if self.themes.contains_key(name) {
            self.current_theme = name.to_string();
            true
        } else {
            false
        }
    }

    /// Add custom theme
    pub fn add_theme(&mut self, theme: ThemeConfig) {
        self.themes.insert(theme.name.clone(), theme);
    }

    /// Remove custom theme
    pub fn remove_theme(&mut self, name: &str) -> bool {
        if name != "Light" && name != "Dark" && name != "Sepia" && name != "High Contrast" {
            self.themes.remove(name).is_some()
        } else {
            false
        }
    }

    /// Get all themes
    pub fn get_all_themes(&self) -> Vec<&ThemeConfig> {
        self.themes.values().collect()
    }

    /// Get theme by name
    pub fn get_theme(&self, name: &str) -> Option<&ThemeConfig> {
        self.themes.get(name)
    }

    /// Generate CSS for a theme
    pub fn generate_css(&self, theme: &ThemeConfig) -> String {
        format!(
            r##"
:root {{
    --bg-color: {};
    --text-color: {};
    --text-secondary: {};
    --accent-color: {};
    --link-color: {};
    --border-color: {};
    --highlight-color: {};
    
    --font-family: {};
    --font-size: {}px;
    --line-height: {};
    
    --max-width: {}px;
    --margin-width: {}px;
    --paragraph-spacing: {}px;
}}
"##,
            theme.colors.background,
            theme.colors.text,
            theme.colors.text_secondary,
            theme.colors.accent,
            theme.colors.link,
            theme.colors.border,
            theme.colors.highlight,
            theme.typography.font_family,
            theme.typography.font_size_base,
            theme.typography.line_height,
            theme.layout.max_width,
            theme.layout.margin_width,
            theme.layout.paragraph_spacing,
        )
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme() {
        let config = ThemeConfig::default();
        assert_eq!(config.preset, ReadingTheme::Light);
        assert!(config.name.contains("Light"));
    }

    #[test]
    fn test_theme_manager() {
        let mut manager = ThemeManager::new();
        
        assert!(manager.set_theme("Dark"));
        let theme = manager.get_current_theme();
        assert!(theme.is_some());
        assert_eq!(theme.unwrap().preset, ReadingTheme::Dark);
    }

    #[test]
    fn test_typography_presets() {
        let dyslexia = TypographySettings::dyslexia_friendly();
        assert!(dyslexia.font_family.contains("OpenDyslexic"));
        assert!(dyslexia.line_height > 1.6);

        let large = TypographySettings::large_print();
        assert!(large.font_size_base >= 20);
    }
}