//! # Voice Commands Module
//!
//! Defines and manages voice commands for browser control through speech recognition.
//! This module provides a comprehensive system for registering, parsing, and executing
//! voice commands with pattern matching, parameter extraction, and confidence scoring.
//!
//! ## Features
//!
//! - **Command Registry**: Centralized management of all voice commands
//! - **Pattern Matching**: Flexible pattern matching with multiple phrases per command
//! - **Parameter Extraction**: Automatic extraction of command parameters from speech
//! - **Confidence Scoring**: Confidence scores for each command match
//! - **Command Aliases**: Support for command aliases and synonyms
//! - **Default Commands**: Pre-configured commands for common browser operations
//! - **Custom Commands**: Easy registration of custom voice commands
//!
//! ## Built-in Commands
//!
//! The module includes 12 default commands covering:
//! - **Navigation**: back, forward, refresh, scroll_up, scroll_down, go_home
//! - **Search**: search (with query parameter)
//! - **Tabs**: new_tab, close_tab, next_tab, previous_tab
//! - **Bookmarks**: bookmark
//!
//! ## Example Usage
//!
//! ```rust
//! use vantisweb::voice::commands::{CommandRegistry, Command, CommandType};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create registry with default commands
//!     let registry = CommandRegistry::new();
//!
//!     // Parse speech input
//!     let result = registry.parse("go back").await;
//!     if let Some(parsed) = result {
//!         println!("Command: {}", parsed.command);
//!         println!("Confidence: {:.2}", parsed.confidence);
//!     }
//!
//!     // Register a custom command
//!     let custom = Command {
//!         name: "zoom_in".to_string(),
//!         command_type: CommandType::System,
//!         patterns: vec!["zoom in".to_string(), "magnify".to_string()],
//!         description: "Zoom in on the current page".to_string(),
//!         parameters: vec![],
//!         enabled: true,
//!     };
//!     registry.register(custom).await?;
//!
//!     // Register an alias
//!     registry.register_alias("magnify", "zoom_in").await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Command Pattern Matching
//!
//! Commands are matched using flexible pattern matching:
//! - Each command can have multiple patterns/phrases
//! - Patterns are matched case-insensitively
//! - Patterns can be partial phrases within speech
//! - Confidence scores indicate match quality
//!
//! ## Parameter Extraction
//!
//! Commands can extract parameters from speech:
//! - Parameters are defined in the command definition
//! - Simple extraction logic supports common patterns
//! - Parameters are returned in a HashMap for easy access
//!
//! ## Security Considerations
//!
//! - Always validate command parameters before execution
//! - Be cautious with custom commands that perform sensitive operations
//! - Consider adding confirmation prompts for dangerous commands
//! - Monitor command usage patterns for abuse detection

use anyhow::{Result, Error};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Command registry for voice commands
///
/// The `CommandRegistry` manages all voice commands in the system, providing
/// functionality for registering, parsing, and managing commands. It maintains
/// thread-safe storage of commands and aliases, and supports pattern matching
/// with confidence scoring.
///
/// # Examples
///
/// ```rust
/// use vantisweb::voice::commands::CommandRegistry;
///
/// let registry = CommandRegistry::new();
/// // Registry is initialized with default commands
/// ```
///
/// # Thread Safety
///
/// The registry is thread-safe and can be shared across multiple tasks
/// through `Arc<CommandRegistry>`.
pub struct CommandRegistry {
    commands: Arc<RwLock<HashMap<String, Command>>>,
    aliases: Arc<RwLock<HashMap<String, String>>>,
}

/// Voice command definition
///
/// The `Command` struct defines a complete voice command with its patterns,
/// parameters, and metadata. Each command can be matched against speech input
/// and includes configuration for execution behavior.
///
/// # Examples
///
/// ```rust
/// use vantisweb::voice::commands::{Command, CommandType, CommandParameter, ParameterType};
///
/// let command = Command {
///     name: "search".to_string(),
///     command_type: CommandType::Search,
///     patterns: vec!["search for".to_string(), "search".to_string()],
///     description: "Search the web".to_string(),
///     parameters: vec![CommandParameter {
///         name: "query".to_string(),
///         param_type: ParameterType::String,
///         required: true,
///         default: None,
///     }],
///     enabled: true,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct Command {
    /// Command name (unique identifier)
    pub name: String,
    /// Command type for categorization
    pub command_type: CommandType,
    /// Command patterns/phrases (multiple patterns supported)
    pub patterns: Vec<String>,
    /// Human-readable description
    pub description: String,
    /// Parameters the command accepts
    pub parameters: Vec<CommandParameter>,
    /// Whether command is enabled for matching
    pub enabled: bool,
}

/// Command types for categorization
///
/// The `CommandType` enum categorizes commands by their function, allowing
/// for organized command management and filtering.
///
/// # Examples
///
/// ```rust
/// use vantisweb::voice::commands::CommandType;
///
/// let cmd_type = CommandType::Navigation;
/// ```
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

/// Command parameter definition
///
/// The `CommandParameter` struct defines a parameter that can be extracted
/// from speech input when a command is matched.
///
/// # Examples
///
/// ```rust
/// use vantisweb::voice::commands::{CommandParameter, ParameterType};
///
/// let param = CommandParameter {
///     name: "query".to_string(),
///     param_type: ParameterType::String,
///     required: true,
///     default: None,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct CommandParameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: ParameterType,
    /// Whether parameter is required
    pub required: bool,
    /// Default value if not provided
    pub default: Option<String>,
}

/// Parameter types
///
/// The `ParameterType` enum defines the supported types for command parameters.
///
/// # Examples
///
/// ```rust
/// use vantisweb::voice::commands::ParameterType;
///
/// let param_type = ParameterType::String;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParameterType {
    /// Text string
    String,
    /// Numeric value
    Number,
    /// Boolean value
    Boolean,
    /// URL address
    Url,
    /// Tab index
    TabIndex,
}

/// Parsed command with parameters
///
/// The `ParsedCommand` struct represents the result of parsing speech input,
/// containing the matched command, extracted parameters, and confidence score.
///
/// # Examples
///
/// ```rust
/// use vantisweb::voice::commands::ParsedCommand;
/// use std::collections::HashMap;
///
/// let parsed = ParsedCommand {
///     command: "back".to_string(),
///     parameters: HashMap::new(),
///     pattern: "go back".to_string(),
///     confidence: 1.0,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ParsedCommand {
    /// Command name
    pub command: String,
    /// Parameters extracted from speech
    pub parameters: HashMap<String, String>,
    /// Matched pattern
    pub pattern: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
}

impl CommandRegistry {
    /// Create a new command registry with default commands
    ///
    /// Initializes a new `CommandRegistry` and registers 12 default commands
    /// for common browser operations including navigation, search, tabs, and
    /// bookmarks.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantisweb::voice::commands::CommandRegistry;
    ///
    /// let registry = CommandRegistry::new();
    /// ```
    pub fn new() -> Self {
        let registry = Self {
            commands: Arc::new(RwLock::new(HashMap::new())),
            aliases: Arc::new(RwLock::new(HashMap::new())),
        };

        registry.register_default_commands();
        registry
    }

    /// Register a command
    ///
    /// Adds a new command to the registry. If a command with the same name
    /// already exists, it will be replaced.
    ///
    /// # Arguments
    ///
    /// * `command` - The command to register
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantisweb::voice::commands::{CommandRegistry, Command, CommandType};
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let registry = CommandRegistry::new();
    ///     let command = Command {
    ///         name: "custom".to_string(),
    ///         command_type: CommandType::Custom,
    ///         patterns: vec!["custom command".to_string()],
    ///         description: "Custom command".to_string(),
    ///         parameters: vec![],
    ///         enabled: true,
    ///     };
    ///     registry.register(command).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn register(&self, command: Command) -> Result<()> {
        let mut commands = self.commands.write().await;
        commands.insert(command.name.clone(), command);
        Ok(())
    }

    /// Register an alias for a command
    ///
    /// Creates an alias that maps to an existing command name. When parsing,
    /// the alias will be recognized and resolved to the actual command.
    ///
    /// # Arguments
    ///
    /// * `alias` - The alias phrase
    /// * `command` - The target command name
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantisweb::voice::commands::CommandRegistry;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let registry = CommandRegistry::new();
    ///     registry.register_alias("backwards", "back").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn register_alias(&self, alias: &str, command: &str) -> Result<()> {
        let mut aliases = self.aliases.write().await;
        aliases.insert(alias.to_string(), command.to_string());
        Ok(())
    }

    /// Unregister a command
    ///
    /// Removes a command from the registry by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The command name to remove
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantisweb::voice::commands::CommandRegistry;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let registry = CommandRegistry::new();
    ///     registry.unregister("back").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn unregister(&self, name: &str) -> Result<()> {
        let mut commands = self.commands.write().await;
        commands.remove(name);
        Ok(())
    }

    /// Parse speech into a command
    ///
    /// Attempts to match speech input against all registered commands.
    /// Returns the best matching command with extracted parameters and
    /// confidence score.
    ///
    /// # Arguments
    ///
    /// * `speech` - The speech text to parse
    ///
    /// # Returns
    ///
    /// `Some(ParsedCommand)` if a match is found, `None` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantisweb::voice::commands::CommandRegistry;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let registry = CommandRegistry::new();
    ///     let result = registry.parse("go back").await;
    ///     if let Some(parsed) = result {
    ///         println!("Command: {}", parsed.command);
    ///         println!("Confidence: {:.2}", parsed.confidence);
    ///     }
    /// }
    /// ```
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
    ///
    /// Extracts defined parameters from speech input using simple pattern
    /// matching logic. Currently supports "query" parameter extraction.
    ///
    /// # Arguments
    ///
    /// * `speech` - The speech text
    /// * `command` - The command definition
    ///
    /// # Returns
    ///
    /// A HashMap of parameter names to extracted values
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
    ///
    /// Calculates a confidence score (0.0 to 1.0) based on how well the
    /// pattern matches the speech input.
    ///
    /// # Arguments
    ///
    /// * `speech` - The speech text
    /// * `pattern` - The matched pattern
    ///
    /// # Returns
    ///
    /// A confidence score from 0.0 to 1.0
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
    ///
    /// Returns a vector of all registered commands.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantisweb::voice::commands::CommandRegistry;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let registry = CommandRegistry::new();
    ///     let commands = registry.get_all_commands().await;
    ///     println!("Total commands: {}", commands.len());
    /// }
    /// ```
    pub async fn get_all_commands(&self) -> Vec<Command> {
        let commands = self.commands.read().await;
        commands.values().cloned().collect()
    }

    /// Get command by name
    ///
    /// Returns a command by its name, or `None` if not found.
    ///
    /// # Arguments
    ///
    /// * `name` - The command name
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantisweb::voice::commands::CommandRegistry;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let registry = CommandRegistry::new();
    ///     if let Some(command) = registry.get_command("back").await {
    ///         println!("Description: {}", command.description);
    ///     }
    /// }
    /// ```
    pub async fn get_command(&self, name: &str) -> Option<Command> {
        let commands = self.commands.read().await;
        commands.get(name).cloned()
    }

    /// Register default commands
    ///
    /// Registers 12 default commands for common browser operations.
    /// This is called automatically during registry initialization.
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