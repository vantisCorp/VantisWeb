//! UI Components
//! 
//! Reusable UI components:
//! - Buttons
//! - Inputs
//! - Panels
//! - Modals
//! - Dropdowns

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

/// Button component
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
    /// Create a new button
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
        format!(
            r#"<button id="{}" class="button button-{:?}" {}>{}</button>"#,
            self.id,
            self.style,
            if self.enabled { "" } else { "disabled" },
            self.text
        )
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

/// Input component
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
    /// Create a new input
    pub fn new(id: String, placeholder: String) -> Self {
        Self {
            id,
            placeholder,
            value: String::new(),
            input_type: InputType::Text,
            readonly: false,
        }
    }
    
    /// Set input type
    pub fn with_type(mut self, input_type: InputType) -> Self {
        self.input_type = input_type;
        self
    }
    
    /// Get input value
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
        format!(
            r#"<input id="{}" type="{:?}" placeholder="{}" value="{}" {}>"#,
            self.id,
            self.input_type,
            self.placeholder,
            self.value,
            if self.readonly { "readonly" } else { "" }
        )
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