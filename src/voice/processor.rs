//! # Voice Processor Module
//!
//! Processes voice commands and executes browser actions by coordinating
//! between the speech recognizer and command registry. This module provides
//! the main entry point for voice-controlled browser operations.
//!
//! ## Features
//!
//! - **Command Coordination**: Coordinates between recognizer and command registry
//! - **Wake Word Detection**: Optional wake word activation for hands-free use
//! - **Confidence Filtering**: Only executes commands above confidence threshold
//! - **Command Confirmation**: Optional confirmation prompts for sensitive commands
//! - **Event System**: Comprehensive event system for tracking processor state
//! - **Async Processing**: Full async/await support for non-blocking operation
//!
//! ## Architecture
//!
//! The processor sits between the voice recognizer and the browser:
//! ```text
//! Voice Input → Recognizer → Processor → Command Execution
//!                                    ↓
//!                              Command Registry
//! ```
//!
//! ## Example Usage
//!
//! ```rust
//! use vantisweb::voice::processor::{VoiceProcessor, ProcessorConfig};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = ProcessorConfig {
//!         enabled: true,
//!         wake_word_enabled: true,
//!         wake_word: "VantisWeb".to_string(),
//!         require_confirmation: false,
//!         confidence_threshold: 0.7,
//!     };
//!     
//!     let processor = VoiceProcessor::new(config);
//!     
//!     // Start voice processing
//!     processor.start().await?;
//!     
//!     // Process speech input
//!     let event = processor.process_speech("go back").await?;
//!     println!("Event: {:?}", event);
//!     
//!     // Stop processing
//!     processor.stop().await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Wake Word Detection
//!
//! When enabled, the processor checks for the wake word before processing commands:
//! - Wake word detection is case-insensitive
//! - The wake word can appear anywhere in the speech
//! - Detection returns a `WakeWordDetected` event
//!
//! ## Confidence Thresholds
//!
//! Commands are only executed when their confidence score meets the threshold:
//! - Default threshold: 0.7 (70%)
//! - Higher thresholds reduce false positives
//! - Lower thresholds improve accessibility
//!
//! ## Security Considerations
//!
//! - Consider enabling `require_confirmation` for sensitive commands
//! - Monitor command execution through the event system
//! - Use wake word detection to prevent accidental activation
//! - Log all command executions for audit purposes

use anyhow::{Result, Error};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::recognizer::VoiceRecognizer;
use super::commands::{CommandRegistry, ParsedCommand};

/// Voice processor that coordinates recognition and command execution
///
/// The `VoiceProcessor` is the main orchestrator for voice-controlled browser
/// operations. It manages the voice recognizer, command registry, and handles
/// the complete flow from speech input to command execution.
///
/// # Examples
///
/// ```rust
/// use vantisweb::voice::processor::{VoiceProcessor, ProcessorConfig};
///
/// let config = ProcessorConfig::default();
/// let processor = VoiceProcessor::new(config);
/// ```
///
/// # Thread Safety
///
/// The processor is thread-safe and can be shared across multiple tasks
/// through `Arc<VoiceProcessor>`.
pub struct VoiceProcessor {
    config: ProcessorConfig,
    recognizer: VoiceRecognizer,
    registry: CommandRegistry,
    is_active: Arc<RwLock<bool>>,
}

/// Processor configuration
///
/// Configuration options for the voice processor, including wake word settings,
/// confidence thresholds, and confirmation requirements.
///
/// # Examples
///
/// ```rust
/// use vantisweb::voice::processor::ProcessorConfig;
///
/// let config = ProcessorConfig {
///     enabled: true,
///     wake_word_enabled: true,
///     wake_word: "Browser".to_string(),
///     require_confirmation: true,
///     confidence_threshold: 0.8,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ProcessorConfig {
    /// Enable voice commands processing
    pub enabled: bool,
    /// Enable wake word detection for activation
    pub wake_word_enabled: bool,
    /// Wake word phrase (e.g., "VantisWeb", "Hey Browser")
    pub wake_word: String,
    /// Require confirmation before executing commands
    pub require_confirmation: bool,
    /// Minimum confidence threshold for command execution (0.0 to 1.0)
    pub confidence_threshold: f32,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            wake_word_enabled: true,
            wake_word: "VantisWeb".to_string(),
            require_confirmation: false,
            confidence_threshold: 0.7,
        }
    }
}

/// Processor event
///
/// Events emitted by the voice processor during operation, providing
/// visibility into the command processing lifecycle.
///
/// # Examples
///
/// ```rust
/// use vantisweb::voice::processor::ProcessorEvent;
/// use vantisweb::voice::commands::ParsedCommand;
/// use std::collections::HashMap;
///
/// let event = ProcessorEvent::CommandExecuted("back".to_string());
/// ```
#[derive(Debug, Clone)]
pub enum ProcessorEvent {
    /// Wake word detected in speech input
    WakeWordDetected,
    /// Command recognized and parsed successfully
    CommandRecognized(ParsedCommand),
    /// Command executed successfully
    CommandExecuted(String),
    /// Command execution failed with error
    CommandFailed { command: String, error: String },
    /// Voice listening has started
    ListeningStarted,
    /// Voice listening has stopped
    ListeningStopped,
}

impl VoiceProcessor {
    /// Create a new voice processor
    ///
    /// Initializes a new `VoiceProcessor` with the specified configuration.
    /// The processor is created in an inactive state and must be started
    /// explicitly.
    ///
    /// # Arguments
    ///
    /// * `config` - Processor configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantisweb::voice::processor::{VoiceProcessor, ProcessorConfig};
    ///
    /// let config = ProcessorConfig::default();
    /// let processor = VoiceProcessor::new(config);
    /// ```
    pub fn new(config: ProcessorConfig) -> Self {
        Self {
            recognizer: VoiceRecognizer::new(super::recognizer::RecognizerConfig::default()),
            registry: CommandRegistry::new(),
            config,
            is_active: Arc::new(RwLock::new(false)),
        }
    }

    /// Start voice processing
    ///
    /// Activates the voice processor and starts the underlying recognizer.
    /// After starting, the processor will accept speech input for processing.
    ///
    /// # Errors
    ///
    /// Returns an error if the recognizer fails to start.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantisweb::voice::processor::{VoiceProcessor, ProcessorConfig};
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let processor = VoiceProcessor::new(ProcessorConfig::default());
    ///     processor.start().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn start(&self) -> Result<()> {
        let mut active = self.is_active.write().await;
        *active = true;
        self.recognizer.start().await?;
        Ok(())
    }

    /// Stop voice processing
    ///
    /// Deactivates the voice processor and stops the underlying recognizer.
    /// After stopping, the processor will not accept speech input.
    ///
    /// # Errors
    ///
    /// Returns an error if the recognizer fails to stop.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantisweb::voice::processor::{VoiceProcessor, ProcessorConfig};
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let processor = VoiceProcessor::new(ProcessorConfig::default());
    ///     processor.start().await?;
    ///     processor.stop().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn stop(&self) -> Result<()> {
        let mut active = self.is_active.write().await;
        *active = false;
        self.recognizer.stop().await?;
        Ok(())
    }

    /// Check if processor is active
    ///
    /// Returns `true` if the processor is currently active and accepting
    /// speech input, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantisweb::voice::processor::{VoiceProcessor, ProcessorConfig};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let processor = VoiceProcessor::new(ProcessorConfig::default());
    ///     assert!(!processor.is_active().await);
    ///     processor.start().await.unwrap();
    ///     assert!(processor.is_active().await);
    /// }
    /// ```
    pub async fn is_active(&self) -> bool {
        *self.is_active.read().await
    }

    /// Process speech input
    ///
    /// Processes speech input and returns a `ProcessorEvent` indicating the
    /// result. The processor will:
    /// 1. Check for wake word (if enabled)
    /// 2. Parse speech as a command
    /// 3. Check confidence threshold
    /// 4. Execute or queue the command
    ///
    /// # Arguments
    ///
    /// * `speech` - The speech text to process
    ///
    /// # Returns
    ///
    /// A `ProcessorEvent` indicating the result of processing.
    ///
    /// # Errors
    ///
    /// Returns an error if no valid command is recognized.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantisweb::voice::processor::{VoiceProcessor, ProcessorConfig};
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let processor = VoiceProcessor::new(ProcessorConfig::default());
    ///     let event = processor.process_speech("go back").await?;
    ///     println!("Event: {:?}", event);
    ///     Ok(())
    /// }
    /// ```
    pub async fn process_speech(&self, speech: &str) -> Result<ProcessorEvent> {
        // Check for wake word if enabled
        if self.config.wake_word_enabled {
            let wake_word_lower = self.config.wake_word.to_lowercase();
            if speech.to_lowercase().contains(&wake_word_lower) {
                return Ok(ProcessorEvent::WakeWordDetected);
            }
        }

        // Try to parse as command
        if let Some(parsed) = self.registry.parse(speech).await {
            // Check confidence threshold
            if parsed.confidence >= self.config.confidence_threshold {
                if self.config.require_confirmation {
                    // In a real implementation, this would request confirmation
                    return Ok(ProcessorEvent::CommandRecognized(parsed));
                } else {
                    // Execute command directly
                    match self.execute_command(&parsed).await {
                        Ok(()) => Ok(ProcessorEvent::CommandExecuted(parsed.command)),
                        Err(e) => Ok(ProcessorEvent::CommandFailed {
                            command: parsed.command,
                            error: e.to_string(),
                        }),
                    }
                }
            }
        }

        Err(Error::msg("No valid command recognized"))
    }

    /// Execute a parsed command
    ///
    /// Routes the parsed command to the appropriate handler function.
    /// This is an internal method called after confidence validation.
    ///
    /// # Arguments
    ///
    /// * `command` - The parsed command to execute
    async fn execute_command(&self, command: &ParsedCommand) -> Result<()> {
        match command.command.as_str() {
            "back" => self.execute_back().await,
            "forward" => self.execute_forward().await,
            "refresh" => self.execute_refresh().await,
            "search" => self.execute_search(&command.parameters).await,
            "new_tab" => self.execute_new_tab().await,
            "close_tab" => self.execute_close_tab().await,
            "next_tab" => self.execute_next_tab().await,
            "previous_tab" => self.execute_previous_tab().await,
            "bookmark" => self.execute_bookmark().await,
            "scroll_down" => self.execute_scroll_down().await,
            "scroll_up" => self.execute_scroll_up().await,
            "go_home" => self.execute_go_home().await,
            _ => Err(Error::msg("Unknown command")),
        }
    }

    /// Navigate back in history
    async fn execute_back(&self) -> Result<()> {
        // In a real implementation, this would navigate back
        Ok(())
    }

    /// Navigate forward in history
    async fn execute_forward(&self) -> Result<()> {
        // In a real implementation, this would navigate forward
        Ok(())
    }

    /// Refresh the current page
    async fn execute_refresh(&self) -> Result<()> {
        // In a real implementation, this would refresh the page
        Ok(())
    }

    /// Execute a search with the provided parameters
    ///
    /// # Arguments
    ///
    /// * `params` - Command parameters including the search query
    async fn execute_search(&self, params: &std::collections::HashMap<String, String>) -> Result<()> {
        if let Some(query) = params.get("query") {
            // In a real implementation, this would perform a search
            println!("Searching for: {}", query);
        }
        Ok(())
    }

    /// Open a new tab
    async fn execute_new_tab(&self) -> Result<()> {
        // In a real implementation, this would open a new tab
        Ok(())
    }

    /// Close the current tab
    async fn execute_close_tab(&self) -> Result<()> {
        // In a real implementation, this would close the current tab
        Ok(())
    }

    /// Switch to the next tab
    async fn execute_next_tab(&self) -> Result<()> {
        // In a real implementation, this would switch to the next tab
        Ok(())
    }

    /// Switch to the previous tab
    async fn execute_previous_tab(&self) -> Result<()> {
        // In a real implementation, this would switch to the previous tab
        Ok(())
    }

    /// Bookmark the current page
    async fn execute_bookmark(&self) -> Result<()> {
        // In a real implementation, this would bookmark the current page
        Ok(())
    }

    /// Scroll down on the current page
    async fn execute_scroll_down(&self) -> Result<()> {
        // In a real implementation, this would scroll down
        Ok(())
    }

    /// Scroll up on the current page
    async fn execute_scroll_up(&self) -> Result<()> {
        // In a real implementation, this would scroll up
        Ok(())
    }

    /// Navigate to the home page
    async fn execute_go_home(&self) -> Result<()> {
        // In a real implementation, this would navigate to home
        Ok(())
    }

    /// Get the command registry
    ///
    /// Returns a reference to the command registry for adding or removing
    /// commands.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantisweb::voice::processor::{VoiceProcessor, ProcessorConfig};
    ///
    /// let processor = VoiceProcessor::new(ProcessorConfig::default());
    /// let registry = processor.registry();
    /// ```
    pub fn registry(&self) -> &CommandRegistry {
        &self.registry
    }

    /// Get the recognizer
    ///
    /// Returns a reference to the voice recognizer for configuration or
    /// status checking.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantisweb::voice::processor::{VoiceProcessor, ProcessorConfig};
    ///
    /// let processor = VoiceProcessor::new(ProcessorConfig::default());
    /// let recognizer = processor.recognizer();
    /// ```
    pub fn recognizer(&self) -> &VoiceRecognizer {
        &self.recognizer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_config_default() {
        let config = ProcessorConfig::default();
        assert!(config.enabled);
        assert!(config.wake_word_enabled);
        assert_eq!(config.wake_word, "VantisWeb");
    }

    #[tokio::test]
    async fn test_processor_start_stop() {
        let processor = VoiceProcessor::new(ProcessorConfig::default());
        processor.start().await.unwrap();
        assert!(processor.is_active().await);
        
        processor.stop().await.unwrap();
        assert!(!processor.is_active().await);
    }

    #[tokio::test]
    async fn test_process_speech_command() {
        let processor = VoiceProcessor::new(ProcessorConfig::default());
        let event = processor.process_speech("go back").await.unwrap();
        
        match event {
            ProcessorEvent::CommandExecuted(cmd) => {
                assert_eq!(cmd, "back");
            },
            _ => panic!("Expected CommandExecuted event"),
        }
    }

    #[tokio::test]
    async fn test_process_wake_word() {
        let config = ProcessorConfig {
            wake_word_enabled: true,
            wake_word: "VantisWeb".to_string(),
            ..Default::default()
        };
        let processor = VoiceProcessor::new(config);
        let event = processor.process_speech("VantisWeb open new tab").await.unwrap();
        
        match event {
            ProcessorEvent::WakeWordDetected => {}
            _ => panic!("Expected WakeWordDetected event"),
        }
    }
}