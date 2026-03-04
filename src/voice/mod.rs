//! Voice Commands Module
//!
//! Provides voice command support allowing users to navigate, search,
//! and control the browser using voice commands.

pub mod recognizer;
pub mod commands;
pub mod processor;
pub mod feedback;

pub use recognizer::{VoiceRecognizer, RecognizerConfig};
pub use commands::{Command, CommandType, CommandRegistry};
pub use processor::{VoiceProcessor, ProcessorConfig};
pub use feedback::{VoiceFeedback, FeedbackType};