//! # Voice Feedback Module
//!
//! Provides visual and auditory feedback for voice interactions, enhancing
//! the user experience by confirming voice recognition events and command
//! execution status.
//!
//! ## Features
//!
//! - **Dual Feedback Modes**: Visual and auditory feedback options
//! - **Configurable Output**: Choose between visual, audio, both, or none
//! - **Event-Driven**: Responds to all voice interaction events
//! - **Toggle Control**: Enable/disable feedback at runtime
//! - **Customizable**: Easy to extend with custom feedback behaviors
//!
//! ## Feedback Types
//!
//! The module supports four feedback modes:
//! - `None`: No feedback (silent mode)
//! - `Visual`: Console/UI indicators only
//! - `Audio`: Sound effects only
//! - `Both`: Combined visual and audio feedback
//!
//! ## Supported Events
//!
//! Feedback is provided for the following events:
//! - `ListeningStarted`: Microphone activated
//! - `ListeningStopped`: Microphone deactivated
//! - `SpeechDetected`: Voice input detected
//! - `CommandRecognized`: Command parsed successfully
//! - `CommandSuccess`: Command executed successfully
//! - `CommandFailed`: Command execution failed
//! - `WakeWordDetected`: Wake word recognized
//!
//! ## Example Usage
//!
//! ```rust
//! use vantisweb::voice::feedback::{VoiceFeedback, FeedbackType, FeedbackEvent};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create feedback system with both visual and audio
//!     let feedback = VoiceFeedback::new(FeedbackType::Both);
//!
//!     // Provide feedback for events
//!     feedback.provide_feedback(FeedbackEvent::ListeningStarted).await;
//!     feedback.provide_feedback(FeedbackEvent::CommandRecognized {
//!         command: "back".to_string(),
//!     }).await;
//!     feedback.provide_feedback(FeedbackEvent::CommandSuccess).await;
//!
//!     // Toggle feedback
//!     feedback.disable().await;
//!     feedback.enable().await;
//!
//!     // Change feedback type
//!     feedback.set_feedback_type(FeedbackType::Visual).await;
//! }
//! ```
//!
//! ## Visual Feedback
//!
//! Visual feedback uses emoji indicators in console output:
//! - 🎤 Listening (microphone active)
//! - 🔇 Not listening (microphone inactive)
//! - 👂 Speech detected (audio input received)
//! - ✓ Command: [name] (command parsed)
//! - ✅ Command executed (success)
//! - ❌ Command failed (error)
//! - 🎯 Wake word detected
//!
//! ## Audio Feedback
//!
//! Audio feedback plays sounds for key events:
//! - Listening start/end sounds
//! - Success/error confirmation tones
//! - Wake word activation chime
//!
//! ## Accessibility
//!
//! The dual feedback system ensures accessibility:
//! - Visual feedback for hearing-impaired users
//! - Audio feedback for visually-impaired users
//! - Both modes for comprehensive feedback
//! - Silent mode for noise-sensitive environments

use std::sync::Arc;
use tokio::sync::RwLock;

/// Voice feedback system
///
/// The `VoiceFeedback` struct manages feedback output for voice interactions.
/// It supports multiple feedback types and can be enabled/disabled at runtime.
///
/// # Examples
///
/// ```rust
/// use vantisweb::voice::feedback::{VoiceFeedback, FeedbackType};
///
/// let feedback = VoiceFeedback::new(FeedbackType::Both);
/// ```
///
/// # Thread Safety
///
/// The feedback system is thread-safe and can be shared across multiple tasks
/// through `Arc<VoiceFeedback>`.
pub struct VoiceFeedback {
    enabled: Arc<RwLock<bool>>,
    feedback_type: Arc<RwLock<FeedbackType>>,
}

/// Feedback type
///
/// Defines the type of feedback to provide for voice interaction events.
///
/// # Examples
///
/// ```rust
/// use vantisweb::voice::feedback::FeedbackType;
///
/// let feedback_type = FeedbackType::Both;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackType {
    /// No feedback (silent mode)
    None,
    /// Visual feedback only (console/UI indicators)
    Visual,
    /// Audio feedback only (sound effects)
    Audio,
    /// Both visual and audio feedback
    Both,
}

impl Default for VoiceFeedback {
    fn default() -> Self {
        Self {
            enabled: Arc::new(RwLock::new(true)),
            feedback_type: Arc::new(RwLock::new(FeedbackType::Both)),
        }
    }
}

/// Feedback event
///
/// Events that trigger feedback responses. Each event corresponds to a
/// specific state or action in the voice interaction lifecycle.
///
/// # Examples
///
/// ```rust
/// use vantisweb::voice::feedback::FeedbackEvent;
///
/// let event = FeedbackEvent::CommandSuccess;
/// ```
#[derive(Debug, Clone)]
pub enum FeedbackEvent {
    /// Microphone activated, listening for input
    ListeningStarted,
    /// Microphone deactivated, stopped listening
    ListeningStopped,
    /// Voice input detected in audio stream
    SpeechDetected,
    /// Command parsed successfully from speech
    CommandRecognized { command: String },
    /// Command executed successfully
    CommandSuccess,
    /// Command execution failed
    CommandFailed,
    /// Wake word detected in speech
    WakeWordDetected,
}

impl VoiceFeedback {
    /// Create a new feedback system
    ///
    /// Initializes a new `VoiceFeedback` with the specified feedback type.
    /// The system starts in enabled state.
    ///
    /// # Arguments
    ///
    /// * `feedback_type` - The type of feedback to provide
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantisweb::voice::feedback::{VoiceFeedback, FeedbackType};
    ///
    /// let feedback = VoiceFeedback::new(FeedbackType::Visual);
    /// ```
    pub fn new(feedback_type: FeedbackType) -> Self {
        Self {
            enabled: Arc::new(RwLock::new(true)),
            feedback_type: Arc::new(RwLock::new(feedback_type)),
        }
    }

    /// Provide feedback for an event
    ///
    /// Delivers the appropriate feedback for the given event based on
    /// the current feedback type and enabled state.
    ///
    /// # Arguments
    ///
    /// * `event` - The feedback event to respond to
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantisweb::voice::feedback::{VoiceFeedback, FeedbackType, FeedbackEvent};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let feedback = VoiceFeedback::new(FeedbackType::Visual);
    ///     feedback.provide_feedback(FeedbackEvent::ListeningStarted).await;
    /// }
    /// ```
    pub async fn provide_feedback(&self, event: FeedbackEvent) {
        if !*self.enabled.read().await {
            return;
        }

        let feedback_type = *self.feedback_type.read().await;

        match feedback_type {
            FeedbackType::None => {}
            FeedbackType::Visual => self.visual_feedback(&event),
            FeedbackType::Audio => self.audio_feedback(&event),
            FeedbackType::Both => {
                self.visual_feedback(&event);
                self.audio_feedback(&event);
            }
        }
    }

    /// Visual feedback
    ///
    /// Provides visual feedback for events using console output with
    /// emoji indicators for clear visual identification.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to provide visual feedback for
    fn visual_feedback(&self, event: &FeedbackEvent) {
        match event {
            FeedbackEvent::ListeningStarted => {
                // Show microphone indicator
                println!("🎤 Listening...");
            }
            FeedbackEvent::ListeningStopped => {
                // Hide microphone indicator
                println!("🔇 Not listening");
            }
            FeedbackEvent::SpeechDetected => {
                // Show speech detection animation
                println!("👂 Speech detected");
            }
            FeedbackEvent::CommandRecognized { command } => {
                // Show recognized command
                println!("✓ Command: {}", command);
            }
            FeedbackEvent::CommandSuccess => {
                // Show success indicator
                println!("✅ Command executed");
            }
            FeedbackEvent::CommandFailed => {
                // Show error indicator
                println!("❌ Command failed");
            }
            FeedbackEvent::WakeWordDetected => {
                // Show wake word detection
                println!("🎯 Wake word detected");
            }
        }
    }

    /// Audio feedback
    ///
    /// Provides audio feedback for events using sound effects.
    /// In a production implementation, this would play actual audio files.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to provide audio feedback for
    fn audio_feedback(&self, event: &FeedbackEvent) {
        match event {
            FeedbackEvent::ListeningStarted => {
                // Play listening sound
                println!("🔊 Playing listening sound");
            }
            FeedbackEvent::CommandSuccess => {
                // Play success sound
                println!("🔊 Playing success sound");
            }
            FeedbackEvent::CommandFailed => {
                // Play error sound
                println!("🔊 Playing error sound");
            }
            FeedbackEvent::WakeWordDetected => {
                // Play wake word sound
                println!("🔊 Playing wake word sound");
            }
            _ => {}
        }
    }

    /// Enable feedback
    ///
    /// Enables the feedback system if it was previously disabled.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantisweb::voice::feedback::{VoiceFeedback, FeedbackType};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let feedback = VoiceFeedback::new(FeedbackType::Both);
    ///     feedback.disable().await;
    ///     feedback.enable().await;
    ///     assert!(feedback.is_enabled().await);
    /// }
    /// ```
    pub async fn enable(&self) {
        let mut enabled = self.enabled.write().await;
        *enabled = true;
    }

    /// Disable feedback
    ///
    /// Disables the feedback system. No feedback will be provided
    /// until `enable()` is called.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantisweb::voice::feedback::{VoiceFeedback, FeedbackType};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let feedback = VoiceFeedback::new(FeedbackType::Both);
    ///     feedback.disable().await;
    ///     assert!(!feedback.is_enabled().await);
    /// }
    /// ```
    pub async fn disable(&self) {
        let mut enabled = self.enabled.write().await;
        *enabled = false;
    }

    /// Set feedback type
    ///
    /// Changes the feedback type at runtime.
    ///
    /// # Arguments
    ///
    /// * `feedback_type` - The new feedback type
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantisweb::voice::feedback::{VoiceFeedback, FeedbackType};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let feedback = VoiceFeedback::new(FeedbackType::Visual);
    ///     feedback.set_feedback_type(FeedbackType::Audio).await;
    /// }
    /// ```
    pub async fn set_feedback_type(&self, feedback_type: FeedbackType) {
        let mut ft = self.feedback_type.write().await;
        *ft = feedback_type;
    }

    /// Check if feedback is enabled
    ///
    /// Returns `true` if feedback is currently enabled.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantisweb::voice::feedback::{VoiceFeedback, FeedbackType};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let feedback = VoiceFeedback::new(FeedbackType::Both);
    ///     assert!(feedback.is_enabled().await);
    /// }
    /// ```
    pub async fn is_enabled(&self) -> bool {
        *self.enabled.read().await
    }

    /// Get current feedback type
    ///
    /// Returns the currently configured feedback type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantisweb::voice::feedback::{VoiceFeedback, FeedbackType};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let feedback = VoiceFeedback::new(FeedbackType::Visual);
    ///     assert_eq!(feedback.get_feedback_type().await, FeedbackType::Visual);
    /// }
    /// ```
    pub async fn get_feedback_type(&self) -> FeedbackType {
        *self.feedback_type.read().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_default() {
        let feedback = VoiceFeedback::default();
        assert!(feedback.is_enabled());
    }

    #[tokio::test]
    async fn test_feedback_enable_disable() {
        let feedback = VoiceFeedback::new(FeedbackType::Visual);
        
        assert!(feedback.is_enabled().await);
        feedback.disable().await;
        assert!(!feedback.is_enabled().await);
        feedback.enable().await;
        assert!(feedback.is_enabled().await);
    }

    #[tokio::test]
    async fn test_feedback_type() {
        let feedback = VoiceFeedback::new(FeedbackType::Audio);
        assert_eq!(feedback.get_feedback_type().await, FeedbackType::Audio);
        
        feedback.set_feedback_type(FeedbackType::Both).await;
        assert_eq!(feedback.get_feedback_type().await, FeedbackType::Both);
    }

    #[tokio::test]
    async fn test_provide_feedback() {
        let feedback = VoiceFeedback::new(FeedbackType::Visual);
        feedback.provide_feedback(FeedbackEvent::ListeningStarted).await;
        feedback.provide_feedback(FeedbackEvent::CommandSuccess).await;
        feedback.provide_feedback(FeedbackEvent::CommandFailed).await;
    }
}