//! Voice Commands Module
//!
//! Defines and manages voice commands for browser control.

use anyhow::{Result, Error};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Command registry for voice commands
pub struct CommandRegistry {
    commands: Arc<RwLock<HashMap<String, Command>>>,
    aliases: Arc<RwLock<HashMap<String, String>>>,
}

/// Voice command definition
#[derive(Debug, Clone)]
pub struct Command {
    /// Command name
    pub name: String,
    /// Command type
    pub command_type: CommandType,
    /// Command patterns/phrases
    pub patterns: Vec<String>,
    /// Description
    pub description: String,
    /// Parameters
    pub parameters: Vec<CommandParameter>,
    /// Whether command is enabled
    pub enabled: bool,
}

/// Command types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandType {
    /// Navigation commands (back, forward, refresh)
    Navigation,
    /// Search commands
    Search,
    /// Tab management
    Tabs,
    /// Bookmark commands
    Bookmarks,
    /// System commands
    System,
    /// Custom commands
    Custom,
}

/// Command parameter
#[derive(Debug, Clone)]
pub struct CommandParameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: ParameterType,
    /// Whether parameter is required
    pub required: bool,
    /// Default value
    pub default: Option<String>,
}

/// Parameter types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    Url,
    TabIndex,
}

/// Parsed command with parameters
#[derive(Debug, Clone)]
pub struct ParsedCommand {
    /// Command name
    pub command: String,
    /// Parameters extracted from speech
    pub parameters: HashMap<String, String>,
    /// Matched pattern
    pub pattern: String,
    /// Confidence score
    pub confidence: f32,
}

impl CommandRegistry {
    /// Create a new command registry with default commands
    pub fn new() -> Self {
        let registry = Self {
            commands: Arc::new(RwLock::new(HashMap::new())),
            aliases: Arc::new(RwLock::new(HashMap::new())),
        };

        registry.register_default_commands();
        registry
    }

    /// Register a command
    pub async fn register(&self, command: Command) -> Result<()> {
        let mut commands = self.commands.write().await;
        commands.insert(command.name.clone(), command);
        Ok(())
    }

    /// Register an alias for a command
    pub async fn register_alias(&self, alias: &str, command: &str) -> Result<()> {
        let mut aliases = self.aliases.write().await;
        aliases.insert(alias.to_string(), command.to_string());
        Ok(())
    }

    /// Unregister a command
    pub async fn unregister(&self, name: &str) -> Result<()> {
        let mut commands = self.commands.write().await;
        commands.remove(name);
        Ok(())
    }

    /// Parse speech into a command
    pub async fn parse(&self, speech: &str) -> Option<ParsedCommand> {
        let commands = self.commands.read().await;
        let aliases = self.aliases.read().await;
        
        let speech_lower = speech.to_lowercase();

        for (name, command) in commands.iter() {
            if !command.enabled {
                continue;
            }

            for pattern in &command.patterns {
                let pattern_lower = pattern.to_lowercase();
                
                if speech_lower.contains(&pattern_lower) {
                    return Some(ParsedCommand {
                        command: name.clone(),
                        parameters: Self::extract_parameters(speech, command),
                        pattern: pattern.clone(),
                        confidence: Self::calculate_confidence(speech, pattern),
                    });
                }
            }
        }

        None
    }

    /// Extract parameters from speech
    fn extract_parameters(speech: &str, command: &Command) -> HashMap<String, String> {
        let mut params = HashMap::new();

        for param in &command.parameters {
            // Simple parameter extraction logic
            if param.name == "query" {
                if let Some(idx) = speech.find("search for") {
                    params.insert(
                        param.name.clone(),
                        speech[idx + 11..].trim().to_string(),
                    );
                }
            }
        }

        params
    }

    /// Calculate confidence score for a match
    fn calculate_confidence(speech: &str, pattern: &str) -> f32 {
        let speech_words: Vec<&str> = speech.split_whitespace().collect();
        let pattern_words: Vec<&str> = pattern.split_whitespace().collect();

        let mut matches = 0;
        for pattern_word in &pattern_words {
            for speech_word in &speech_words {
                if speech_word.to_lowercase().contains(&pattern_word.to_lowercase()) {
                    matches += 1;
                    break;
                }
            }
        }

        if pattern_words.is_empty() {
            0.0
        } else {
            matches as f32 / pattern_words.len() as f32
        }
    }

    /// Get all commands
    pub async fn get_all_commands(&self) -> Vec<Command> {
        let commands = self.commands.read().await;
        commands.values().cloned().collect()
    }

    /// Get command by name
    pub async fn get_command(&self, name: &str) -> Option<Command> {
        let commands = self.commands.read().await;
        commands.get(name).cloned()
    }

    /// Register default commands
    fn register_default_commands(&self) {
        let defaults = vec![
            Command {
                name: "back".to_string(),
                command_type: CommandType::Navigation,
                patterns: vec!["go back".to_string(), "back".to_string(), "previous".to_string()],
                description: "Go back to the previous page".to_string(),
                parameters: vec![],
                enabled: true,
            },
            Command {
                name: "forward".to_string(),
                command_type: CommandType::Navigation,
                patterns: vec!["go forward".to_string(), "forward".to_string(), "next".to_string()],
                description: "Go forward to the next page".to_string(),
                parameters: vec![],
                enabled: true,
            },
            Command {
                name: "refresh".to_string(),
                command_type: CommandType::Navigation,
                patterns: vec!["refresh".to_string(), "reload".to_string(), "reload page".to_string()],
                description: "Refresh the current page".to_string(),
                parameters: vec![],
                enabled: true,
            },
            Command {
                name: "search".to_string(),
                command_type: CommandType::Search,
                patterns: vec!["search for".to_string(), "search".to_string()],
                description: "Search the web".to_string(),
                parameters: vec![CommandParameter {
                    name: "query".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    default: None,
                }],
                enabled: true,
            },
            Command {
                name: "new_tab".to_string(),
                command_type: CommandType::Tabs,
                patterns: vec!["new tab".to_string(), "open new tab".to_string(), "create tab".to_string()],
                description: "Open a new tab".to_string(),
                parameters: vec![],
                enabled: true,
            },
            Command {
                name: "close_tab".to_string(),
                command_type: CommandType::Tabs,
                patterns: vec!["close tab".to_string(), "close".to_string()],
                description: "Close the current tab".to_string(),
                parameters: vec![],
                enabled: true,
            },
            Command {
                name: "next_tab".to_string(),
                command_type: CommandType::Tabs,
                patterns: vec!["next tab".to_string(), "switch to next tab".to_string()],
                description: "Switch to the next tab".to_string(),
                parameters: vec![],
                enabled: true,
            },
            Command {
                name: "previous_tab".to_string(),
                command_type: CommandType::Tabs,
                patterns: vec!["previous tab".to_string(), "switch to previous tab".to_string()],
                description: "Switch to the previous tab".to_string(),
                parameters: vec![],
                enabled: true,
            },
            Command {
                name: "bookmark".to_string(),
                command_type: CommandType::Bookmarks,
                patterns: vec!["bookmark this".to_string(), "add bookmark".to_string(), "save bookmark".to_string()],
                description: "Bookmark the current page".to_string(),
                parameters: vec![],
                enabled: true,
            },
            Command {
                name: "scroll_down".to_string(),
                command_type: CommandType::Navigation,
                patterns: vec!["scroll down".to_string(), "page down".to_string()],
                description: "Scroll down the page".to_string(),
                parameters: vec![],
                enabled: true,
            },
            Command {
                name: "scroll_up".to_string(),
                command_type: CommandType::Navigation,
                patterns: vec!["scroll up".to_string(), "page up".to_string()],
                description: "Scroll up the page".to_string(),
                parameters: vec![],
                enabled: true,
            },
            Command {
                name: "go_home".to_string(),
                command_type: CommandType::Navigation,
                patterns: vec!["go home".to_string(), "home".to_string()],
                description: "Go to the homepage".to_string(),
                parameters: vec![],
                enabled: true,
            },
        ];

        for command in defaults {
            tokio::spawn(async move {
                let registry = CommandRegistry {
                    commands: Arc::new(RwLock::new(HashMap::new())),
                    aliases: Arc::new(RwLock::new(HashMap::new())),
                };
                let _ = registry.register(command).await;
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command() {
        let command = Command {
            name: "test".to_string(),
            command_type: CommandType::Navigation,
            patterns: vec!["test pattern".to_string()],
            description: "Test command".to_string(),
            parameters: vec![],
            enabled: true,
        };
        assert_eq!(command.name, "test");
        assert!(command.enabled);
    }

    #[tokio::test]
    async fn test_registry_parse() {
        let registry = CommandRegistry::new();
        let result = registry.parse("go back").await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().command, "back");
    }

    #[tokio::test]
    async fn test_registry_parse_search() {
        let registry = CommandRegistry::new();
        let result = registry.parse("search for rust programming").await;
        assert!(result.is_some());
        let parsed = result.unwrap();
        assert_eq!(parsed.command, "search");
    }

    #[tokio::test]
    async fn test_registry_register() {
        let registry = CommandRegistry::new();
        let command = Command {
            name: "custom".to_string(),
            command_type: CommandType::Custom,
            patterns: vec!["custom command".to_string()],
            description: "Custom".to_string(),
            parameters: vec![],
            enabled: true,
        };
        registry.register(command).await.unwrap();
        let result = registry.parse("custom command").await;
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_registry_alias() {
        let registry = CommandRegistry::new();
        registry.register_alias("backwards", "back").await.unwrap();
        // Note: This test would need to be updated to check aliases
    }
}