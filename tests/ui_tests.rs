//! Unit tests for UI modules

#[cfg(test)]
mod tests {
    #[test]
    fn test_theme_manager() {
        // Test theme manager
        let theme_manager = vanisweb::ui::theming::ThemeManager::new().unwrap();
        
        let theme = theme_manager.current_theme();
        assert_eq!(theme.mode, vanisweb::ui::theming::ThemeMode::Dark);
    }

    #[test]
    fn test_button_component() {
        // Test button component
        let button = vanisweb::ui::components::Button::new(
            "test_button".to_string(),
            "Click Me".to_string()
        );
        
        assert_eq!(button.text, "Click Me");
        assert!(button.enabled);
    }

    #[test]
    fn test_button_render() {
        // Test button rendering
        let button = vanisweb::ui::components::Button::new(
            "test_button".to_string(),
            "Click Me".to_string()
        );
        
        let rendered = button.render();
        assert!(rendered.contains("Click Me"));
        assert!(rendered.contains("button"));
    }

    #[test]
    fn test_input_component() {
        // Test input component
        let input = vanisweb::ui::components::Input::new(
            "test_input".to_string(),
            "Enter text...".to_string()
        );
        
        assert_eq!(input.placeholder, "Enter text...");
        assert_eq!(input.value(), "");
    }

    #[test]
    fn test_input_value() {
        // Test input value setting
        let mut input = vanisweb::ui::components::Input::new(
            "test_input".to_string(),
            "Enter text...".to_string()
        );
        
        input.set_value("Test Value".to_string());
        assert_eq!(input.value(), "Test Value");
    }
}