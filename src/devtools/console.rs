// Copyright 2024 Vantis Corporation - All Rights Reserved

//! JavaScript Console Module
//! 
//! This module provides an interactive JavaScript console with REPL,
//  logging support, object inspection, command history, and auto-completion.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Console message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleMessage {
    /// Message ID
    pub id: Uuid,
    /// Message level
    pub level: LogLevel,
    /// Message text
    pub text: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Source
    pub source: MessageSource,
    /// Arguments
    pub arguments: Vec<ConsoleArgument>,
    /// Stack trace
    pub stack_trace: Option<Vec<StackFrame>>,
    /// Repeat count
    pub repeat_count: u32,
    /// URL
    pub url: Option<String>,
    /// Line number
    pub line_number: Option<u32>,
    /// Column number
    pub column_number: Option<u32>,
}

/// Log level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogLevel {
    Verbose,
    Debug,
    Info,
    Warning,
    Error,
}

/// Message source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageSource {
    ConsoleAPI,
    Network,
    Security,
    Storage,
    Worker,
    Other,
}

/// Console argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleArgument {
    /// Argument type
    pub arg_type: ArgumentType,
    /// Value
    pub value: serde_json::Value,
    /// String representation
    pub string_representation: String,
}

/// Argument type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArgumentType {
    String,
    Number,
    Boolean,
    Null,
    Undefined,
    Object,
    Array,
    Function,
    Symbol,
}

/// Stack frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    /// Function name
    pub function_name: String,
    /// Source URL
    pub source_url: String,
    /// Line number
    pub line_number: u32,
    /// Column number
    pub column_number: u32,
}

/// Command result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    /// Result ID
    pub id: Uuid,
    /// Command
    pub command: String,
    /// Result
    pub result: Option<serde_json::Value>,
    /// Error
    pub error: Option<String>,
    /// Is async
    pub is_async: bool,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Execution time (ms)
    pub execution_time: f64,
}

/// Console history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Command
    pub command: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Success
    pub success: bool,
}

/// Auto-completion suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionSuggestion {
    /// Suggestion text
    pub text: String,
    /// Suggestion type
    pub suggestion_type: CompletionType,
    /// Description
    pub description: Option<String>,
}

/// Completion type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompletionType {
    Property,
    Method,
    Variable,
    Keyword,
    Snippet,
}

/// Object inspection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectInspection {
    /// Object ID
    pub id: Uuid,
    /// Object type
    pub object_type: String,
    /// Properties
    pub properties: Vec<ObjectProperty>,
    /// Methods
    pub methods: Vec<ObjectMethod>,
    /// Prototype
    pub prototype: Option<String>,
    /// Size
    pub size: Option<usize>,
}

/// Object property
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectProperty {
    /// Property name
    pub name: String,
    /// Property value
    pub value: serde_json::Value,
    /// Is enumerable
    pub enumerable: bool,
    /// Is writable
    pub writable: bool,
    /// Is configurable
    pub configurable: bool,
}

/// Object method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectMethod {
    /// Method name
    pub name: String,
    /// Parameter list
    pub parameters: Vec<String>,
    /// Return type
    pub return_type: Option<String>,
}

/// Console settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleSettings {
    /// Max messages
    pub max_messages: usize,
    /// Preserve logs
    pub preserve_logs: bool,
    /// Log timing
    pub log_timing: bool,
    /// Show timestamps
    pub show_timestamps: bool,
    /// Auto-scroll
    pub auto_scroll: bool,
}

impl Default for ConsoleSettings {
    fn default() -> Self {
        Self {
            max_messages: 1000,
            preserve_logs: false,
            log_timing: false,
            show_timestamps: false,
            auto_scroll: true,
        }
    }
}

/// JavaScript console
pub struct JSConsole {
    /// Messages
    messages: Arc<RwLock<Vec<ConsoleMessage>>>,
    /// Command history
    history: Arc<RwLock<Vec<HistoryEntry>>>,
    /// History index
    history_index: Arc<RwLock<usize>>,
    /// Completions cache
    completions: Arc<RwLock<HashMap<String, Vec<CompletionSuggestion>>>>,
    /// Object cache
    object_cache: Arc<RwLock<HashMap<Uuid, ObjectInspection>>>,
    /// Settings
    settings: Arc<RwLock<ConsoleSettings>>,
    /// Console API implementation
    console_api: Arc<RwLock<ConsoleAPI>>,
}

/// Console API implementation
pub struct ConsoleAPI {
    /// Log level filters
    log_levels: Arc<RwLock<HashMap<String, LogLevel>>>,
    /// Custom formatters
    formatters: Arc<RwLock<HashMap<String, Box<dyn Fn(&serde_json::Value) -> String + Send + Sync>>>>,
}

impl JSConsole {
    /// Create a new JavaScript console
    pub fn new() -> Self {
        Self {
            messages: Arc::new(RwLock::new(Vec::new())),
            history: Arc::new(RwLock::new(Vec::new())),
            history_index: Arc::new(RwLock::new(0)),
            completions: Arc::new(RwLock::new(HashMap::new())),
            object_cache: Arc::new(RwLock::new(HashMap::new())),
            settings: Arc::new(RwLock::new(ConsoleSettings::default())),
            console_api: Arc::new(RwLock::new(ConsoleAPI::new())),
        }
    }

    /// Initialize the console
    pub fn initialize(&self) -> Result<(), ConsoleError> {
        Ok(())
    }

    /// Log a message
    pub async fn log(&self, level: LogLevel, text: String, arguments: Vec<ConsoleArgument>) {
        let message = ConsoleMessage {
            id: Uuid::new_v4(),
            level,
            text,
            arguments,
            timestamp: Utc::now(),
            source: MessageSource::ConsoleAPI,
            stack_trace: None,
            repeat_count: 1,
            url: None,
            line_number: None,
            column_number: None,
        };
        
        let mut messages = self.messages.write().await;
        
        // Check for repeat messages
        if let Some(last) = messages.last() {
            if last.text == message.text && last.level == message.level {
                messages.last_mut().unwrap().repeat_count += 1;
                return;
            }
        }
        
        messages.push(message);
        
        // Enforce max messages limit
        let settings = self.settings.read().await;
        while messages.len() > settings.max_messages {
            messages.remove(0);
        }
    }

    /// Execute a command
    pub async fn execute_command(&self, command: String) -> Result<CommandResult, ConsoleError> {
        let start = std::time::Instant::now();
        let timestamp = Utc::now();
        
        // Parse and execute command (simplified - in production use actual JS engine)
        let result = self.evaluate_javascript(&command).await?;
        
        let execution_time = start.elapsed().as_secs_f64();
        
        let command_result = CommandResult {
            id: Uuid::new_v4(),
            command: command.clone(),
            result: Some(result),
            error: None,
            is_async: false,
            timestamp,
            execution_time,
        };
        
        // Add to history
        let mut history = self.history.write().await;
        history.push(HistoryEntry {
            command,
            timestamp,
            success: command_result.error.is_none(),
        });
        
        // Limit history size
        if history.len() > 1000 {
            history.remove(0);
        }
        
        *self.history_index.write().await = history.len();
        
        Ok(command_result)
    }

    /// Get all messages
    pub async fn get_messages(&self) -> Vec<ConsoleMessage> {
        self.messages.read().await.clone()
    }

    /// Get filtered messages
    pub async fn get_filtered_messages(&self, level_filter: Option<LogLevel>) -> Vec<ConsoleMessage> {
        let messages = self.messages.read().await;
        
        match level_filter {
            Some(filter) => messages.iter()
                .filter(|m| m.level == filter)
                .cloned()
                .collect(),
            None => messages.clone(),
        }
    }

    /// Clear messages
    pub async fn clear(&self) -> Result<(), ConsoleError> {
        self.messages.write().await.clear();
        Ok(())
    }

    /// Get message count
    pub async fn get_message_count(&self) -> usize {
        self.messages.read().await.len()
    }

    /// Get history
    pub async fn get_history(&self) -> Vec<HistoryEntry> {
        self.history.read().await.clone()
    }

    /// Navigate history
    pub async fn navigate_history(&self, direction: HistoryDirection) -> Option<String> {
        let mut index = self.history_index.write().await;
        let history = self.history.read().await;
        
        match direction {
            HistoryDirection::Previous => {
                if *index > 0 {
                    *index -= 1;
                    history.get(*index).map(|h| h.command.clone())
                } else {
                    None
                }
            }
            HistoryDirection::Next => {
                if *index < history.len() {
                    *index += 1;
                    history.get(*index).map(|h| h.command.clone())
                } else {
                    None
                }
            }
        }
    }

    /// Get auto-completions
    pub async fn get_completions(&self, prefix: &str) -> Vec<CompletionSuggestion> {
        let completions = self.completions.read().await;
        
        completions.get(prefix)
            .cloned()
            .unwrap_or_else(|| self.generate_completions(prefix))
    }

    /// Inspect object
    pub async fn inspect_object(&self, object_id: Uuid) -> Option<ObjectInspection> {
        self.object_cache.read().await.get(&object_id).cloned()
    }

    /// Get settings
    pub async fn get_settings(&self) -> ConsoleSettings {
        self.settings.read().await.clone()
    }

    /// Update settings
    pub async fn update_settings(&self, settings: ConsoleSettings) {
        *self.settings.write().await = settings;
    }

    // Helper methods

    async fn evaluate_javascript(&self, code: &str) -> Result<serde_json::Value, ConsoleError> {
        // Simplified JavaScript evaluation
        // In production, integrate with actual JavaScript engine (V8, SpiderMonkey, etc.)
        
        let code = code.trim();
        
        if code.is_empty() {
            return Ok(serde_json::Value::Null);
        }
        
        // Simple numeric evaluation
        if let Ok(num) = code.parse::<f64>() {
            return Ok(serde_json::json!(num));
        }
        
        // Simple string evaluation
        if code.starts_with('"') || code.starts_with('\'') {
            return Ok(serde_json::Value::String(code[1..code.len()-1].to_string()));
        }
        
        // Boolean values
        match code {
            "true" => Ok(serde_json::Value::Bool(true)),
            "false" => Ok(serde_json::Value::Bool(false)),
            "null" => Ok(serde_json::Value::Null),
            "undefined" => Ok(serde_json::Value::Null),
            _ => Ok(serde_json::Value::String(format!("// Result of: {}", code))),
        }
    }

    fn generate_completions(&self, prefix: &str) -> Vec<CompletionSuggestion> {
        let mut suggestions = Vec::new();
        
        // Common JavaScript globals
        let globals = vec![
            "console", "window", "document", "navigator", "location",
            "history", "screen", "localStorage", "sessionStorage",
            "fetch", "XMLHttpRequest", "WebSocket", "EventSource",
            "setTimeout", "setInterval", "clearTimeout", "clearInterval",
            "Promise", "Array", "Object", "String", "Number", "Boolean",
            "Date", "Math", "JSON", "RegExp", "Error",
        ];
        
        for global in globals {
            if global.starts_with(prefix) {
                suggestions.push(CompletionSuggestion {
                    text: global.to_string(),
                    suggestion_type: CompletionType::Variable,
                    description: Some("Global object".to_string()),
                });
            }
        }
        
        // Console methods
        let console_methods = vec![
            "log", "info", "warn", "error", "debug", "trace",
            "table", "group", "groupEnd", "groupCollapsed",
            "time", "timeLog", "timeEnd", "count", "countReset",
            "assert", "clear", "dir", "dirxml",
        ];
        
        if prefix.starts_with("console.") {
            let method_prefix = &prefix[8..];
            for method in console_methods {
                if method.starts_with(method_prefix) {
                    suggestions.push(CompletionSuggestion {
                        text: format!("console.{}", method),
                        suggestion_type: CompletionType::Method,
                        description: Some("Console method".to_string()),
                    });
                }
            }
        }
        
        suggestions
    }
}

impl ConsoleAPI {
    /// Create a new console API
    pub fn new() -> Self {
        Self {
            log_levels: Arc::new(RwLock::new(HashMap::new())),
            formatters: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

/// History direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HistoryDirection {
    Previous,
    Next,
}

/// Console error
#[derive(Debug, thiserror::Error)]
pub enum ConsoleError {
    #[error("Command execution error: {0}")]
    ExecutionError(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Other error: {0}")]
    Other(String),
}

impl Default for JSConsole {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_console_initialization() {
        let console = JSConsole::new();
        assert!(console.initialize().is_ok());
    }

    #[tokio::test]
    async fn test_log_message() {
        let console = JSConsole::new();
        console.log(LogLevel::Info, "Test message".to_string(), vec![]).await;
        
        let messages = console.get_messages().await;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].text, "Test message");
    }

    #[tokio::test]
    async fn test_execute_command() {
        let console = JSConsole::new();
        let result = console.execute_command("42".to_string()).await.unwrap();
        
        assert_eq!(result.command, "42");
        assert_eq!(result.result, Some(serde_json::Value::Number(42.into())));
    }

    #[tokio::test]
    async fn test_clear_messages() {
        let console = JSConsole::new();
        console.log(LogLevel::Info, "Test".to_string(), vec![]).await;
        console.clear().await.unwrap();
        
        assert_eq!(console.get_message_count().await, 0);
    }

    #[tokio::test]
    async fn test_history_navigation() {
        let console = JSConsole::new();
        console.execute_command("cmd1".to_string()).await.unwrap();
        console.execute_command("cmd2".to_string()).await.unwrap();
        
        let previous = console.navigate_history(HistoryDirection::Previous).await;
        assert_eq!(previous, Some("cmd2".to_string()));
        
        let previous = console.navigate_history(HistoryDirection::Previous).await;
        assert_eq!(previous, Some("cmd1".to_string()));
    }

    #[tokio::test]
    async fn test_get_completions() {
        let console = JSConsole::new();
        let completions = console.get_completions("con").await;
        
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.text.starts_with("con")));
    }

    #[tokio::test]
    async fn test_log_level_filtering() {
        let console = JSConsole::new();
        console.log(LogLevel::Info, "Info".to_string(), vec![]).await;
        console.log(LogLevel::Error, "Error".to_string(), vec![]).await;
        
        let errors = console.get_filtered_messages(Some(LogLevel::Error)).await;
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].level, LogLevel::Error);
    }

    #[tokio::test]
    async fn test_repeat_messages() {
        let console = JSConsole::new();
        console.log(LogLevel::Info, "Test".to_string(), vec![]).await;
        console.log(LogLevel::Info, "Test".to_string(), vec![]).await;
        
        let messages = console.get_messages().await;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].repeat_count, 2);
    }
}