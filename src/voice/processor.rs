//! Voice Processor Module
//!
//! Processes voice commands and executes browser actions.

use anyhow::{Result, Error};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::recognizer::VoiceRecognizer;
use super::commands::{CommandRegistry, ParsedCommand};

/// Voice processor that coordinates recognition and command execution
pub struct VoiceProcessor {
    config: ProcessorConfig,
    recognizer: VoiceRecognizer,
    registry: CommandRegistry,
    is_active: Arc<RwLock<bool>>,
}

/// Processor configuration
#[derive(Debug, Clone)]
pub struct ProcessorConfig {
    /// Enable voice commands
    pub enabled: bool,
    /// Enable wake word detection
    pub wake_word_enabled: bool,
    /// Wake word phrase
    pub wake_word: String,
    /// Enable command confirmation
    pub require_confirmation: bool,
    /// Confidence threshold
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
#[derive(Debug, Clone)]
pub enum ProcessorEvent {
    /// Wake word detected
    WakeWordDetected,
    /// Command recognized
    CommandRecognized(ParsedCommand),
    /// Command executed successfully
    CommandExecuted(String),
    /// Command execution failed
    CommandFailed { command: String, error: String },
    /// Listening started
    ListeningStarted,
    /// Listening stopped
    ListeningStopped,
}

impl VoiceProcessor {
    /// Create a new voice processor
    pub fn new(config: ProcessorConfig) -> Self {
        Self {
            recognizer: VoiceRecognizer::new(super::recognizer::RecognizerConfig::default()),
            registry: CommandRegistry::new(),
            config,
            is_active: Arc::new(RwLock::new(false)),
        }
    }

    /// Start voice processing
    pub async fn start(&self) -> Result<()> {
        let mut active = self.is_active.write().await;
        *active = true;
        self.recognizer.start().await?;
        Ok(())
    }

    /// Stop voice processing
    pub async fn stop(&self) -> Result<()> {
        let mut active = self.is_active.write().await;
        *active = false;
        self.recognizer.stop().await?;
        Ok(())
    }

    /// Check if processor is active
    pub async fn is_active(&self) -> bool {
        *self.is_active.read().await
    }

    /// Process speech input
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

    async fn execute_back(&self) -> Result<()> {
        // In a real implementation, this would navigate back
        Ok(())
    }

    async fn execute_forward(&self) -> Result<()> {
        // In a real implementation, this would navigate forward
        Ok(())
    }

    async fn execute_refresh(&self) -> Result<()> {
        // In a real implementation, this would refresh the page
        Ok(())
    }

    async fn execute_search(&self, params: &std::collections::HashMap<String, String>) -> Result<()> {
        if let Some(query) = params.get("query") {
            // In a real implementation, this would perform a search
            println!("Searching for: {}", query);
        }
        Ok(())
    }

    async fn execute_new_tab(&self) -> Result<()> {
        // In a real implementation, this would open a new tab
        Ok(())
    }

    async fn execute_close_tab(&self) -> Result<()> {
        // In a real implementation, this would close the current tab
        Ok(())
    }

    async fn execute_next_tab(&self) -> Result<()> {
        // In a real implementation, this would switch to the next tab
        Ok(())
    }

    async fn execute_previous_tab(&self) -> Result<()> {
        // In a real implementation, this would switch to the previous tab
        Ok(())
    }

    async fn execute_bookmark(&self) -> Result<()> {
        // In a real implementation, this would bookmark the current page
        Ok(())
    }

    async fn execute_scroll_down(&self) -> Result<()> {
        // In a real implementation, this would scroll down
        Ok(())
    }

    async fn execute_scroll_up(&self) -> Result<()> {
        // In a real implementation, this would scroll up
        Ok(())
    }

    async fn execute_go_home(&self) -> Result<()> {
        // In a real implementation, this would navigate to home
        Ok(())
    }

    /// Get the command registry
    pub fn registry(&self) -> &CommandRegistry {
        &self.registry
    }

    /// Get the recognizer
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
            }
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