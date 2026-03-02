//! UI Components (Optimized)
//! 
//! Reusable UI components:
//! - Buttons
//! - Inputs
//! - Panels
//! - Modals
//! - Dropdowns
//!
//! Optimizations:
//! - Pre-allocated string capacities
//! - Reduced clones
//! - Optimized string operations

use log::debug;
use serde::{Deserialize, Serialize};

/// UI Component base trait
pub trait UIComponent {
    fn render(&self) -> String;
    fn update(&mut self, event: UIEvent);
}

/// UI Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UIEvent {
    Click,
    Hover,
    Input(String),
    Focus,
    Blur,
    KeyPress(char),
}

/// Button component (Optimized)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Button {
    id: String,
    text: String,
    enabled: bool,
    style: ButtonStyle,
}

/// Button styles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ButtonStyle {
    Primary,
    Secondary,
    Success,
    Danger,
    Warning,
}

impl Button {
    /// Create a new button (Optimized)
    pub fn new(id: String, text: String) -> Self {
        Self {
            id,
            text,
            enabled: true,
            style: ButtonStyle::Primary,
        }
    }
    
    /// Set button style
    pub fn with_style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }
    
    /// Enable/disable button
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl UIComponent for Button {
    fn render(&self) -> String {
        let mut html = String::with_capacity(100);
        
        html.push_str(r#"<button id=""#);
        html.push_str(&self.id);
        html.push_str(r#"" class="button button-"#);
        html.push_str(&format!("{:?}", self.style).to_lowercase());
        html.push_str(r#"" "#);
        if !self.enabled {
            html.push_str(r#"disabled"#);
        }
        html.push_str(r#">"#);
        html.push_str(&self.text);
        html.push_str(r#"</button>"#);
        
        html
    }
    
    fn update(&mut self, event: UIEvent) {
        if !self.enabled {
            return;
        }
        
        match event {
            UIEvent::Click => {
                debug!("Button clicked: {}", self.id);
            }
            _ => {}
        }
    }
}

/// Input component (Optimized)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    id: String,
    placeholder: String,
    value: String,
    input_type: InputType,
    readonly: bool,
}

/// Input types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputType {
    Text,
    Password,
    Email,
    Url,
    Number,
}

impl Input {
    /// Create a new input (Optimized)
    pub fn new(id: String, placeholder: String) -> Self {
        Self {
            id,
            placeholder,
            value: String::with_capacity(100),
            input_type: InputType::Text,
            readonly: false,
        }
    }
    
    /// Set input type
    pub fn with_type(mut self, input_type: InputType) -> Self {
        self.input_type = input_type;
        self
    }
    
    /// Get input value (Optimized - returns reference)
    pub fn value(&self) -> &str {
        &self.value
    }
    
    /// Set input value
    pub fn set_value(&mut self, value: String) {
        self.value = value;
    }
}

impl UIComponent for Input {
    fn render(&self) -> String {
        let mut html = String::with_capacity(150);
        
        html.push_str(r#"<input id=""#);
        html.push_str(&self.id);
        html.push_str(r#"" type=""#);
        html.push_str(&format!("{:?}", self.input_type).to_lowercase());
        html.push_str(r#"" placeholder=""#);
        html.push_str(&self.placeholder);
        html.push_str(r#"" value=""#);
        html.push_str(&self.value);
        html.push_str(r#""" "#);
        if self.readonly {
            html.push_str(r#"readonly"#);
        }
        html.push_str(r#">"#);
        
        html
    }
    
    fn update(&mut self, event: UIEvent) {
        match event {
            UIEvent::Input(value) => {
                self.value = value;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_creation() {
        let button = Button::new("test-btn".to_string(), "Click Me".to_string());
        
        assert_eq!(button.id, "test-btn");
        assert_eq!(button.text, "Click Me");
        assert!(button.enabled);
    }

    #[test]
    fn test_button_render() {
        let button = Button::new("test-btn".to_string(), "Click Me".to_string());
        
        let html = button.render();
        
        assert!(html.contains(r#"id="test-btn""#));
        assert!(html.contains("Click Me"));
    }

    #[test]
    fn test_button_with_style() {
        let button = Button::new("test-btn".to_string(), "Click Me".to_string())
            .with_style(ButtonStyle::Danger);
        
        assert_eq!(format!("{:?}", button.style), "Danger");
    }

    #[test]
    fn test_input_creation() {
        let input = Input::new("test-input".to_string(), "Enter text".to_string());
        
        assert_eq!(input.id, "test-input");
        assert_eq!(input.placeholder, "Enter text");
        assert_eq!(input.value(), "");
    }

    #[test]
    fn test_input_render() {
        let input = Input::new("test-input".to_string(), "Enter text".to_string());
        
        let html = input.render();
        
        assert!(html.contains(r#"id="test-input""#));
        assert!(html.contains("Enter text"));
    }

    #[test]
    fn test_input_set_value() {
        let mut input = Input::new("test-input".to_string(), "Enter text".to_string());
        
        input.set_value("Hello World".to_string());
        
        assert_eq!(input.value(), "Hello World");
    }
}