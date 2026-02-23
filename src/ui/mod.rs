//! VantisUI - User Interface Module
//! 
//! Next-generation UI system:
//! - WebGPU rendering
//! - Hybrid-GPU acceleration
//! - Drag & Drop customization
//! - Ambient Chameleon theming
//! - 144Hz+ smoothness

pub mod app;
pub mod browser;
pub mod components;
pub mod theming;
pub mod renderer;

pub use app::VantisUI;
pub use browser::BrowserWindow;
pub use components::UIComponent;
pub use theming::ThemeManager;
pub use renderer::GPURenderer;