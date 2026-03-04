//! Voice Feedback Module
//!
//! Provides visual and auditory feedback for voice interactions.

use std::sync::Arc;
use tokio::sync::RwLock;

/// Voice feedback system
pub struct VoiceFeedback {
    enabled: Arc<RwLock<bool>>,
    feedback_type: Arc<RwLock<FeedbackType>>,
}

/// Feedback type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackType {
    /// No feedback
    None,
    /// Visual only
    Visual,
    /// Audio only
    Audio,
    /// Both visual and audio
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
#[derive(Debug, Clone)]
pub enum FeedbackEvent {
    /// Listening started
    ListeningStarted,
    /// Listening stopped
    ListeningStopped,
    /// Speech detected
    SpeechDetected,
    /// Command recognized
    CommandRecognized { command: String },
    /// Command executed successfully
    CommandSuccess,
    /// Command failed
    CommandFailed,
    /// Wake word detected
    WakeWordDetected,
}

impl VoiceFeedback {
    /// Create a new feedback system
    pub fn new(feedback_type: FeedbackType) -> Self {
        Self {
            enabled: Arc::new(RwLock::new(true)),
            feedback_type: Arc::new(RwLock::new(feedback_type)),
        }
    }

    /// Provide feedback for an event
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
    pub async fn enable(&self) {
        let mut enabled = self.enabled.write().await;
        *enabled = true;
    }

    /// Disable feedback
    pub async fn disable(&self) {
        let mut enabled = self.enabled.write().await;
        *enabled = false;
    }

    /// Set feedback type
    pub async fn set_feedback_type(&self, feedback_type: FeedbackType) {
        let mut ft = self.feedback_type.write().await;
        *ft = feedback_type;
    }

    /// Check if feedback is enabled
    pub async fn is_enabled(&self) -> bool {
        *self.enabled.read().await
    }

    /// Get current feedback type
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