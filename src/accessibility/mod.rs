//! Accessibility Module - Universal Access Features
//! 
//! This module provides comprehensive accessibility features for VantisWeb Browser,
//! ensuring universal access for all users regardless of their abilities.
//!
//! ## Features
//! - Eye & Head Tracking for hands-free navigation
//! - Voice God Mode for complete voice control
//! - AI Vision Describer for screen reading and image descriptions
//! - Senior Mode for simplified, accessible interface
//! - Tremor Guard for cursor stabilization

pub mod eye_tracking;
pub mod voice_control;
pub mod vision_describer;
pub mod senior_mode;
pub mod tremor_guard;
pub mod accessibility_manager;

// Re-export main types
pub use eye_tracking::{EyeTracker, EyeTrackingConfig, GazePoint, HeadGesture};
pub use voice_control::{VoiceController, VoiceCommand, VoiceConfig, VoiceResult};
pub use vision_describer::{VisionDescriber, VisionConfig, SceneDescription, ElementType};
pub use senior_mode::{SeniorMode, SeniorConfig, UILayout, FontSize};
pub use tremor_guard::{TremorGuard, TremorConfig, CursorState, StabilizationMode};
pub use accessibility_manager::{AccessibilityManager, AccessibilityConfig, AccessibilityProfile};

/// Accessibility error types
#[derive(Debug, thiserror::Error)]
pub enum AccessibilityError {
    #[error("Eye tracking initialization failed: {0}")]
    EyeTrackingInit(String),
    
    #[error("Voice recognition error: {0}")]
    VoiceRecognition(String),
    
    #[error("Vision describer error: {0}")]
    VisionDescriber(String),
    
    #[error("Camera access denied: {0}")]
    CameraAccess(String),
    
    #[error("Microphone access denied: {0}")]
    MicrophoneAccess(String),
    
    #[error("Accessibility profile not found: {0}")]
    ProfileNotFound(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Hardware not supported: {0}")]
    HardwareNotSupported(String),
}

/// Result type for accessibility operations
pub type AccessibilityResult<T> = Result<T, AccessibilityError>;

/// Accessibility level for features
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessibilityLevel {
    /// Full accessibility support
    Full,
    /// Partial support with limitations
    Partial,
    /// Minimal support
    Minimal,
    /// Feature disabled
    Disabled,
}

use serde::{Serialize, Deserialize};