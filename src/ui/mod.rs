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
pub mod profile_ui;

pub use app::VantisUI;
pub use profile_ui::{
    ProfileManagerUI, ProfileTemplateUI, ProfileSyncUI,
    ProfileAnalyticsUI, ProfileSecurityUI
};