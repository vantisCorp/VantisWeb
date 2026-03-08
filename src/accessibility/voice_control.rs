//! Voice God Mode Module
//! 
//! Provides complete voice control for the browser, enabling hands-free
//! operation through natural language commands with offline processing support.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use crate::accessibility::{AccessibilityError, AccessibilityResult};

/// Voice control configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    /// Enable voice control
    pub enabled: bool,
    /// Language code (e.g., "en-US", "pl-PL")
    pub language: String,
    /// Enable offline mode
    pub offline_mode: bool,
    /// Wake word
    pub wake_word: String,
    /// Voice sensitivity (0.0 - 1.0)
    pub sensitivity: f32,
    /// Command timeout in seconds
    pub command_timeout: u64,
    /// Enable voice feedback
    pub voice_feedback: bool,
    /// Speech rate (0.5 - 2.0)
    pub speech_rate: f32,
    /// Enable continuous listening
    pub continuous_listening: bool,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            language: "en-US".to_string(),
            offline_mode: true,
            wake_word: "Vantis".to_string(),
            sensitivity: 0.5,
            command_timeout: 10,
            voice_feedback: true,
            speech_rate: 1.0,
            continuous_listening: false,
        }
    }
}

/// Voice command types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceCommand {
    // Navigation
    Navigate(String),
    GoBack,
    GoForward,
    Refresh,
    Home,
    
    // Tab management
    NewTab,
    CloseTab,
    SwitchTab(String),
    NextTab,
    PreviousTab,
    
    // Scrolling
    ScrollUp,
    ScrollDown,
    ScrollTop,
    ScrollBottom,
    PageUp,
    PageDown,
    
    // Zoom
    ZoomIn,
    ZoomOut,
    ResetZoom,
    
    // Search
    Search(String),
    Find(String),
    FindNext,
    FindPrevious,
    
    // Bookmarks
    BookmarkPage,
    OpenBookmark(String),
    ShowBookmarks,
    
    // History
    ShowHistory,
    ClearHistory,
    
    // Downloads
    ShowDownloads,
    CancelDownload,
    
    // Media
    PlayPause,
    NextTrack,
    PreviousTrack,
    VolumeUp,
    VolumeDown,
    Mute,
    
    // Reader mode
    ReaderMode,
    
    // Private mode
    PrivateMode,
    NewPrivateTab,
    
    // Settings
    OpenSettings,
    ChangeTheme(String),
    
    // Accessibility
    ReadPage,
    ReadSelection,
    StopReading,
    
    // General
    Help,
    Cancel,
    Confirm,
    Dictate(String),
    
    // Custom command
    Custom(String, Vec<String>),
}

/// Voice command result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceResult {
    /// Recognized text
    pub text: String,
    /// Parsed command
    pub command: Option<VoiceCommand>,
    /// Confidence level (0.0 - 1.0)
    pub confidence: f32,
    /// Processing time in ms
    pub processing_time_ms: u64,
    /// Was wake word detected
    pub wake_word_detected: bool,
    /// Timestamp
    pub timestamp: Instant,
}

impl VoiceResult {
    pub fn new(text: String, command: Option<VoiceCommand>, confidence: f32) -> Self {
        Self {
            text,
            command,
            confidence,
            processing_time_ms: 0,
            wake_word_detected: false,
            timestamp: Instant::now(),
        }
    }
}

/// Voice recognizer state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecognizerState {
    Idle,
    Listening,
    Processing,
    Speaking,
    Error,
}

/// Command pattern for matching
#[derive(Debug, Clone)]
struct CommandPattern {
    patterns: Vec<String>,
    command: VoiceCommand,
    parameters: Vec<String>,
}

/// Main voice controller
pub struct VoiceController {
    /// Configuration
    config: VoiceConfig,
    /// Current state
    state: RecognizerState,
    /// Command patterns
    command_patterns: Vec<CommandPattern>,
    /// Command history
    command_history: VecDeque<VoiceResult>,
    /// Statistics
    stats: VoiceStats,
    /// Last wake word detection
    last_wake_word: Option<Instant>,
    /// Custom commands
    custom_commands: HashMap<String, VoiceCommand>,
}

use std::collections::VecDeque;

/// Voice statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VoiceStats {
    /// Total commands recognized
    pub commands_recognized: u64,
    /// Average recognition confidence
    pub avg_confidence: f32,
    /// Wake word detections
    pub wake_word_detections: u64,
    /// Failed recognitions
    pub failed_recognitions: u64,
    /// Total listening time in seconds
    pub total_listening_time: f64,
    /// Most used commands
    pub command_usage: HashMap<String, u64>,
}

impl VoiceController {
    /// Create a new voice controller
    pub fn new(config: VoiceConfig) -> Self {
        let mut controller = Self {
            config,
            state: RecognizerState::Idle,
            command_patterns: Vec::new(),
            command_history: VecDeque::with_capacity(100),
            stats: VoiceStats::default(),
            last_wake_word: None,
            custom_commands: HashMap::new(),
        };
        
        controller.initialize_command_patterns();
        controller
    }
    
    /// Initialize default command patterns
    fn initialize_command_patterns(&mut self) {
        // Navigation commands
        self.add_pattern(&["go to", "navigate to", "open"], |args| {
            VoiceCommand::Navigate(args.join(" "))
        });
        
        self.add_simple_pattern(&["go back", "back"], VoiceCommand::GoBack);
        self.add_simple_pattern(&["go forward", "forward"], VoiceCommand::GoForward);
        self.add_simple_pattern(&["refresh", "reload", "reload page"], VoiceCommand::Refresh);
        self.add_simple_pattern(&["home", "go home"], VoiceCommand::Home);
        
        // Tab management
        self.add_simple_pattern(&["new tab", "open tab"], VoiceCommand::NewTab);
        self.add_simple_pattern(&["close tab", "close current tab"], VoiceCommand::CloseTab);
        self.add_simple_pattern(&["next tab", "switch to next tab"], VoiceCommand::NextTab);
        self.add_simple_pattern(&["previous tab", "switch to previous tab"], VoiceCommand::PreviousTab);
        
        // Scrolling
        self.add_simple_pattern(&["scroll up", "scroll up a bit"], VoiceCommand::ScrollUp);
        self.add_simple_pattern(&["scroll down", "scroll down a bit"], VoiceCommand::ScrollDown);
        self.add_simple_pattern(&["scroll to top", "go to top"], VoiceCommand::ScrollTop);
        self.add_simple_pattern(&["scroll to bottom", "go to bottom"], VoiceCommand::ScrollBottom);
        
        // Zoom
        self.add_simple_pattern(&["zoom in", "make bigger"], VoiceCommand::ZoomIn);
        self.add_simple_pattern(&["zoom out", "make smaller"], VoiceCommand::ZoomOut);
        self.add_simple_pattern(&["reset zoom", "normal size"], VoiceCommand::ResetZoom);
        
        // Search
        self.add_pattern(&["search for", "search", "find"], |args| {
            VoiceCommand::Search(args.join(" "))
        });
        
        self.add_pattern(&["find", "find text"], |args| {
            VoiceCommand::Find(args.join(" "))
        });
        
        // Bookmarks
        self.add_simple_pattern(&["bookmark this page", "add bookmark"], VoiceCommand::BookmarkPage);
        self.add_simple_pattern(&["show bookmarks", "open bookmarks"], VoiceCommand::ShowBookmarks);
        
        // History
        self.add_simple_pattern(&["show history", "open history"], VoiceCommand::ShowHistory);
        
        // Downloads
        self.add_simple_pattern(&["show downloads", "open downloads"], VoiceCommand::ShowDownloads);
        
        // Media
        self.add_simple_pattern(&["play", "pause", "play pause"], VoiceCommand::PlayPause);
        self.add_simple_pattern(&["next track", "next song"], VoiceCommand::NextTrack);
        self.add_simple_pattern(&["previous track", "previous song"], VoiceCommand::PreviousTrack);
        self.add_simple_pattern(&["volume up", "louder"], VoiceCommand::VolumeUp);
        self.add_simple_pattern(&["volume down", "quieter"], VoiceCommand::VolumeDown);
        self.add_simple_pattern(&["mute", "unmute"], VoiceCommand::Mute);
        
        // Reader mode
        self.add_simple_pattern(&["reader mode", "reading mode"], VoiceCommand::ReaderMode);
        
        // Private mode
        self.add_simple_pattern(&["private mode", "incognito"], VoiceCommand::PrivateMode);
        self.add_simple_pattern(&["new private tab"], VoiceCommand::NewPrivateTab);
        
        // Settings
        self.add_simple_pattern(&["settings", "open settings"], VoiceCommand::OpenSettings);
        self.add_pattern(&["change theme to", "set theme"], |args| {
            VoiceCommand::ChangeTheme(args.join(" "))
        });
        
        // Accessibility
        self.add_simple_pattern(&["read page", "read this page"], VoiceCommand::ReadPage);
        self.add_simple_pattern(&["read selection"], VoiceCommand::ReadSelection);
        self.add_simple_pattern(&["stop reading", "stop"], VoiceCommand::StopReading);
        
        // General
        self.add_simple_pattern(&["help", "show help"], VoiceCommand::Help);
        self.add_simple_pattern(&["cancel", "never mind"], VoiceCommand::Cancel);
        self.add_simple_pattern(&["confirm", "yes", "okay"], VoiceCommand::Confirm);
    }
    
    /// Add a command pattern with arguments
    fn add_pattern<F>(&mut self, triggers: &[&str], handler: F) 
    where
        F: Fn(Vec<String>) -> VoiceCommand + 'static,
    {
        // Store patterns for matching
        for trigger in triggers {
            self.command_patterns.push(CommandPattern {
                patterns: vec![trigger.to_lowercase()],
                command: handler(vec!["text".to_string()]),
                parameters: vec!["text".to_string()],
            });
        }
    }
    
    /// Add a simple command pattern without arguments
    fn add_simple_pattern(&mut self, triggers: &[&str], command: VoiceCommand) {
        for trigger in triggers {
            self.command_patterns.push(CommandPattern {
                patterns: vec![trigger.to_lowercase()],
                command: command.clone(),
                parameters: vec![],
            });
        }
    }
    
    /// Initialize the voice controller
    pub fn initialize(&mut self) -> AccessibilityResult<()> {
        // In a real implementation, this would initialize speech recognition
        self.state = RecognizerState::Idle;
        Ok(())
    }
    
    /// Start listening for commands
    pub fn start_listening(&mut self) -> AccessibilityResult<()> {
        if !self.config.enabled {
            return Err(AccessibilityError::Configuration("Voice control disabled".into()));
        }
        
        self.state = RecognizerState::Listening;
        Ok(())
    }
    
    /// Stop listening
    pub fn stop_listening(&mut self) {
        self.state = RecognizerState::Idle;
    }
    
    /// Process recognized speech
    pub fn process_speech(&mut self, text: &str) -> AccessibilityResult<VoiceResult> {
        let start_time = Instant::now();
        let text_lower = text.to_lowercase();
        
        // Check for wake word
        let wake_word_detected = text_lower.starts_with(&self.config.wake_word.to_lowercase());
        
        // Extract command text
        let command_text = if wake_word_detected {
            text_lower[self.config.wake_word.len()..].trim()
        } else if self.config.continuous_listening {
            text_lower.as_str()
        } else {
            // Without wake word and not continuous, return empty result
            return Ok(VoiceResult::new(text.to_string(), None, 0.0));
        };
        
        // Parse command
        let (command, confidence) = self.parse_command(command_text);
        
        // Update statistics
        if command.is_some() {
            self.stats.commands_recognized += 1;
            if let Some(ref cmd) = command {
                let cmd_name = format!("{:?}", cmd).split('(').next().unwrap_or("unknown").to_string();
                *self.stats.command_usage.entry(cmd_name).or_insert(0) += 1;
            }
        } else {
            self.stats.failed_recognitions += 1;
        }
        
        if wake_word_detected {
            self.stats.wake_word_detections += 1;
            self.last_wake_word = Some(Instant::now());
        }
        
        // Update average confidence
        let n = self.stats.commands_recognized as f32;
        self.stats.avg_confidence = 
            (self.stats.avg_confidence * (n - 1.0) + confidence) / n;
        
        let result = VoiceResult {
            text: text.to_string(),
            command,
            confidence,
            processing_time_ms: start_time.elapsed().as_millis() as u64,
            wake_word_detected,
            timestamp: Instant::now(),
        };
        
        // Store in history
        self.command_history.push_back(result.clone());
        if self.command_history.len() > 100 {
            self.command_history.pop_front();
        }
        
        Ok(result)
    }
    
    /// Parse a command from text
    fn parse_command(&self, text: &str) -> (Option<VoiceCommand>, f32) {
        let text = text.trim();
        
        if text.is_empty() {
            return (None, 0.0);
        }
        
        // Check custom commands first
        for (pattern, command) in &self.custom_commands {
            if text.starts_with(pattern) {
                return (Some(command.clone()), 0.95);
            }
        }
        
        // Check built-in patterns
        let mut best_match: Option<(VoiceCommand, f32)> = None;
        
        for pattern in &self.command_patterns {
            for pattern_text in &pattern.patterns {
                if text.starts_with(pattern_text) || text == pattern_text {
                    let confidence = if text == pattern_text { 0.95 } else { 0.85 };
                    
                    // Extract arguments if any
                    let command = if pattern.parameters.is_empty() {
                        pattern.command.clone()
                    } else {
                        let args: Vec<String> = text[pattern_text.len()..]
                            .trim()
                            .split_whitespace()
                            .map(|s| s.to_string())
                            .collect();
                        
                        // Create command with arguments
                        match &pattern.command {
                            VoiceCommand::Navigate(_) => VoiceCommand::Navigate(args.join(" ")),
                            VoiceCommand::Search(_) => VoiceCommand::Search(args.join(" ")),
                            VoiceCommand::Find(_) => VoiceCommand::Find(args.join(" ")),
                            VoiceCommand::ChangeTheme(_) => VoiceCommand::ChangeTheme(args.join(" ")),
                            VoiceCommand::OpenBookmark(_) => VoiceCommand::OpenBookmark(args.join(" ")),
                            VoiceCommand::SwitchTab(_) => VoiceCommand::SwitchTab(args.join(" ")),
                            _ => pattern.command.clone(),
                        }
                    };
                    
                    if best_match.is_none() || confidence > best_match.as_ref().unwrap().1 {
                        best_match = Some((command, confidence));
                    }
                }
            }
        }
        
        // Check for dictation mode (if no command matched)
        if best_match.is_none() && text.len() > 5 {
            // Treat as dictation
            return (Some(VoiceCommand::Dictate(text.to_string())), 0.7);
        }
        
        match best_match {
            Some((cmd, conf)) => (Some(cmd), conf),
            None => (None, 0.0),
        }
    }
    
    /// Register a custom command
    pub fn register_custom_command(&mut self, pattern: &str, command: VoiceCommand) {
        self.custom_commands.insert(pattern.to_lowercase(), command);
    }
    
    /// Speak text aloud
    pub fn speak(&mut self, text: &str) -> AccessibilityResult<()> {
        if !self.config.voice_feedback {
            return Ok(());
        }
        
        self.state = RecognizerState::Speaking;
        
        // In a real implementation, this would use TTS
        // For now, just update state
        self.state = RecognizerState::Idle;
        
        Ok(())
    }
    
    /// Get available commands
    pub fn get_available_commands(&self) -> Vec<String> {
        let mut commands = Vec::new();
        
        for pattern in &self.command_patterns {
            for p in &pattern.patterns {
                commands.push(p.clone());
            }
        }
        
        commands.sort();
        commands.dedup();
        commands
    }
    
    /// Get command history
    pub fn get_history(&self) -> &VecDeque<VoiceResult> {
        &self.command_history
    }
    
    /// Get current state
    pub fn get_state(&self) -> RecognizerState {
        self.state
    }
    
    /// Get statistics
    pub fn get_stats(&self) -> &VoiceStats {
        &self.stats
    }
    
    /// Check if wake word was recently detected
    pub fn is_wake_word_active(&self) -> bool {
        if let Some(last) = self.last_wake_word {
            last.elapsed() < Duration::from_secs(self.config.command_timeout)
        } else {
            false
        }
    }
    
    /// Clear command history
    pub fn clear_history(&mut self) {
        self.command_history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_voice_config_default() {
        let config = VoiceConfig::default();
        assert!(config.enabled);
        assert_eq!(config.wake_word, "Vantis");
    }
    
    #[test]
    fn test_voice_controller_creation() {
        let config = VoiceConfig::default();
        let controller = VoiceController::new(config);
        assert_eq!(controller.get_state(), RecognizerState::Idle);
    }
    
    #[test]
    fn test_navigation_commands() {
        let config = VoiceConfig {
            continuous_listening: true,
            ..Default::default()
        };
        let mut controller = VoiceController::new(config);
        controller.initialize().unwrap();
        
        let result = controller.process_speech("go back").unwrap();
        assert_eq!(result.command, Some(VoiceCommand::GoBack));
        assert!(result.confidence > 0.8);
    }
    
    #[test]
    fn test_wake_word_detection() {
        let config = VoiceConfig::default();
        let mut controller = VoiceController::new(config);
        controller.initialize().unwrap();
        
        let result = controller.process_speech("Vantis go home").unwrap();
        assert!(result.wake_word_detected);
        assert_eq!(result.command, Some(VoiceCommand::Home));
    }
    
    #[test]
    fn test_search_command() {
        let config = VoiceConfig {
            continuous_listening: true,
            ..Default::default()
        };
        let mut controller = VoiceController::new(config);
        controller.initialize().unwrap();
        
        let result = controller.process_speech("search for rust programming").unwrap();
        assert_eq!(result.command, Some(VoiceCommand::Search("rust programming".to_string())));
    }
    
    #[test]
    fn test_custom_command() {
        let config = VoiceConfig {
            continuous_listening: true,
            ..Default::default()
        };
        let mut controller = VoiceController::new(config);
        controller.initialize().unwrap();
        
        controller.register_custom_command("my favorite site", VoiceCommand::Navigate("https://example.com".to_string()));
        
        let result = controller.process_speech("my favorite site").unwrap();
        assert_eq!(result.command, Some(VoiceCommand::Navigate("https://example.com".to_string())));
    }
    
    #[test]
    fn test_command_history() {
        let config = VoiceConfig {
            continuous_listening: true,
            ..Default::default()
        };
        let mut controller = VoiceController::new(config);
        controller.initialize().unwrap();
        
        controller.process_speech("new tab").unwrap();
        controller.process_speech("go back").unwrap();
        
        assert_eq!(controller.get_history().len(), 2);
    }
    
    #[test]
    fn test_statistics() {
        let config = VoiceConfig {
            continuous_listening: true,
            ..Default::default()
        };
        let mut controller = VoiceController::new(config);
        controller.initialize().unwrap();
        
        controller.process_speech("new tab").unwrap();
        controller.process_speech("new tab").unwrap();
        controller.process_speech("go back").unwrap();
        
        let stats = controller.get_stats();
        assert_eq!(stats.commands_recognized, 3);
        assert!(stats.avg_confidence > 0.0);
    }
}