//! Enhanced Console for rich output and debugging
//! 
//! Provides advanced console capabilities including:
//! - Rich output formatting
//! - Console API implementation
//! - Command history
//! - Filtering and search
//! - Performance tracking

use crate::developer_tools::models::ConsoleMessage;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use chrono::{DateTime, Utc};

/// Console log level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LogLevel {
    Verbose,
    Info,
    Warning,
    Error,
    Debug,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

/// Console message with rich formatting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichConsoleMessage {
    /// Unique message ID
    pub id: u64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Log level
    pub level: LogLevel,
    /// Message category
    pub category: MessageCategory,
    /// Text content
    pub text: String,
    /// Formatted parts (for styling)
    pub parts: Vec<MessagePart>,
    /// Source URL
    pub url: Option<String>,
    /// Line number
    pub line: Option<usize>,
    /// Column number
    pub column: Option<usize>,
    /// Stack trace (for errors)
    pub stack_trace: Option<String>,
    /// Associated data (objects, etc.)
    pub data: Vec<ConsoleValue>,
    /// Execution time (for timing operations)
    pub timing: Option<TimingInfo>,
    /// Whether message is repeat count
    pub repeat_count: u32,
    /// Whether message is collapsed in group
    pub is_collapsed: bool,
    /// Group depth
    pub group_depth: u32,
}

/// Part of a formatted message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePart {
    /// Text content
    pub text: String,
    /// Style
    pub style: MessageStyle,
}

/// Styling for message part
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageStyle {
    /// Foreground color
    pub color: Option<String>,
    /// Background color
    pub background: Option<String>,
    /// Font weight
    pub bold: bool,
    /// Font style
    pub italic: bool,
    /// Text decoration
    pub underline: bool,
}

impl Default for MessageStyle {
    fn default() -> Self {
        Self {
            color: None,
            background: None,
            bold: false,
            italic: false,
            underline: false,
        }
    }
}

/// Message category
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageCategory {
    /// JavaScript console API
    Console,
    /// Network messages
    Network,
    /// Security messages
    Security,
    /// Performance messages
    Performance,
    /// Storage messages
    Storage,
    /// User messages
    User,
    /// System messages
    System,
    /// CSS messages
    CSS,
    /// Rendering messages
    Rendering,
    /// Deprecated API usage
    Deprecated,
}

/// Console value (object representation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleValue {
    /// Value type
    pub value_type: ValueType,
    /// Display text
    pub display: String,
    /// Preview (for objects)
    pub preview: Option<String>,
    /// Child properties
    pub properties: Vec<PropertyEntry>,
    /// Array items (if array)
    pub array_items: Vec<ConsoleValue>,
    /// Raw value (for primitives)
    pub raw: Option<String>,
}

/// Value type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValueType {
    Undefined,
    Null,
    Boolean,
    Number,
    String,
    Symbol,
    Function,
    Object,
    Array,
    Map,
    Set,
    WeakMap,
    WeakSet,
    Date,
    RegExp,
    Error,
    Promise,
    TypedArray,
    ArrayBuffer,
    DataView,
    HTMLElement,
    Node,
    Window,
    Custom(String),
}

/// Property entry for object display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyEntry {
    /// Property name
    pub name: String,
    /// Property value
    pub value: ConsoleValue,
    /// Whether property is writable
    pub writable: bool,
    /// Whether property is enumerable
    pub enumerable: bool,
    /// Whether property is configurable
    pub configurable: bool,
    /// Getter function
    pub getter: Option<String>,
    /// Setter function
    pub setter: Option<String>,
}

/// Timing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingInfo {
    /// Timer name
    pub name: String,
    /// Duration in milliseconds
    pub duration_ms: f64,
    /// Start time
    pub start: DateTime<Utc>,
    /// End time
    pub end: DateTime<Utc>,
}

/// Console table entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableEntry {
    /// Column values
    pub columns: Vec<(String, ConsoleValue)>,
    /// Index
    pub index: ConsoleValue,
}

/// Console group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleGroup {
    /// Group label
    pub label: String,
    /// Group depth
    pub depth: u32,
    /// Whether collapsed
    pub collapsed: bool,
    /// Messages in group
    pub message_count: u32,
}

/// Console configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleConfig {
    /// Maximum messages to keep
    pub max_messages: usize,
    /// Maximum command history
    pub max_history: usize,
    /// Show timestamps
    pub show_timestamps: bool,
    /// Enable filtering
    pub enable_filter: bool,
    /// Group similar messages
    pub group_similar: bool,
    /// Maximum string length before truncation
    pub max_string_length: usize,
    /// Maximum object depth
    pub max_object_depth: usize,
    /// Enable timestamps for all messages
    pub timestamp_all: bool,
}

impl Default for ConsoleConfig {
    fn default() -> Self {
        Self {
            max_messages: 10000,
            max_history: 1000,
            show_timestamps: true,
            enable_filter: true,
            group_similar: true,
            max_string_length: 10000,
            max_object_depth: 5,
            timestamp_all: false,
        }
    }
}

/// Console event for subscribers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsoleEvent {
    /// New message added
    Message(RichConsoleMessage),
    /// Console cleared
    Cleared,
    /// Group started
    GroupStarted(ConsoleGroup),
    /// Group ended
    GroupEnded { depth: u32 },
    /// Command executed
    CommandExecuted { command: String, result: Option<ConsoleValue> },
}

/// Enhanced Console
pub struct Console {
    /// Configuration
    config: ConsoleConfig,
    /// Messages
    messages: Arc<RwLock<VecDeque<RichConsoleMessage>>>,
    /// Command history
    history: Arc<RwLock<VecDeque<String>>>,
    /// Active timers
    timers: Arc<RwLock<HashMap<String, std::time::Instant>>>,
    /// Active counters
    counters: Arc<RwLock<HashMap<String, u32>>>,
    /// Active groups
    groups: Arc<RwLock<Vec<ConsoleGroup>>>,
    /// Message ID counter
    message_id: Arc<RwLock<u64>>,
    /// Event broadcaster
    events: broadcast::Sender<ConsoleEvent>,
}

use std::collections::HashMap;

impl Console {
    /// Create a new console
    pub fn new(config: ConsoleConfig) -> Self {
        let (events, _) = broadcast::channel(1000);
        
        Self {
            config,
            messages: Arc::new(RwLock::new(VecDeque::new())),
            history: Arc::new(RwLock::new(VecDeque::new())),
            timers: Arc::new(RwLock::new(HashMap::new())),
            counters: Arc::new(RwLock::new(HashMap::new())),
            groups: Arc::new(RwLock::new(Vec::new())),
            message_id: Arc::new(RwLock::new(0)),
            events,
        }
    }

    /// Get next message ID
    async fn next_id(&self) -> u64 {
        let mut id = self.message_id.write().await;
        *id += 1;
        *id
    }

    /// Log a message
    pub async fn log(&self, args: &[ConsoleValue]) {
        self.log_with_level(LogLevel::Info, args, None).await;
    }

    /// Log info message
    pub async fn info(&self, args: &[ConsoleValue]) {
        self.log_with_level(LogLevel::Info, args, None).await;
    }

    /// Log warning
    pub async fn warn(&self, args: &[ConsoleValue]) {
        self.log_with_level(LogLevel::Warning, args, None).await;
    }

    /// Log error
    pub async fn error(&self, args: &[ConsoleValue]) {
        self.log_with_level(LogLevel::Error, args, None).await;
    }

    /// Log debug message
    pub async fn debug(&self, args: &[ConsoleValue]) {
        self.log_with_level(LogLevel::Debug, args, None).await;
    }

    /// Log verbose message
    pub async fn verbose(&self, args: &[ConsoleValue]) {
        self.log_with_level(LogLevel::Verbose, args, None).await;
    }

    /// Log with specific level
    async fn log_with_level(&self, level: LogLevel, args: &[ConsoleValue], url: Option<&str>) {
        let id = self.next_id().await;
        let group_depth = self.groups.read().await.len() as u32;
        
        // Format message text
        let text = args.iter()
            .map(|v| v.display.clone())
            .collect::<Vec<_>>()
            .join(" ");
        
        // Check for similar messages (grouping)
        let (repeat_count, should_add) = {
            let mut messages = self.messages.write().await;
            let mut repeat_count = 1;
            let mut should_add = true;
            
            if self.config.group_similar && !messages.is_empty() {
                let last = messages.back_mut().unwrap();
                if last.text == text && last.level == level {
                    last.repeat_count += 1;
                    should_add = false;
                }
            }
            
            if should_add && messages.len() >= self.config.max_messages {
                messages.pop_front();
            }
            
            (repeat_count, should_add)
        };
        
        if !should_add {
            return;
        }
        
        let message = RichConsoleMessage {
            id,
            timestamp: Utc::now(),
            level,
            category: MessageCategory::Console,
            text,
            parts: vec![],
            url: url.map(String::from),
            line: None,
            column: None,
            stack_trace: None,
            data: args.to_vec(),
            timing: None,
            repeat_count,
            is_collapsed: false,
            group_depth,
        };
        
        // Add to messages
        {
            let mut messages = self.messages.write().await;
            messages.push_back(message.clone());
        }
        
        // Broadcast event
        let _ = self.events.send(ConsoleEvent::Message(message));
    }

    /// Log with styling (console.log with %c)
    pub async fn log_styled(&self, text: &str, styles: &[MessageStyle]) {
        let id = self.next_id().await;
        let group_depth = self.groups.read().await.len() as u32;
        
        // Parse %c placeholders
        let mut parts = Vec::new();
        let mut style_idx = 0;
        let mut current_text = String::new();
        
        for part in text.split("%c") {
            if !current_text.is_empty() {
                parts.push(MessagePart {
                    text: current_text.clone(),
                    style: MessageStyle::default(),
                });
            }
            current_text = part.to_string();
            
            if style_idx < styles.len() {
                parts.push(MessagePart {
                    text: current_text.clone(),
                    style: styles[style_idx].clone(),
                });
                style_idx += 1;
                current_text.clear();
            }
        }
        
        if !current_text.is_empty() {
            parts.push(MessagePart {
                text: current_text,
                style: MessageStyle::default(),
            });
        }
        
        let message = RichConsoleMessage {
            id,
            timestamp: Utc::now(),
            level: LogLevel::Info,
            category: MessageCategory::Console,
            text: text.to_string(),
            parts,
            url: None,
            line: None,
            column: None,
            stack_trace: None,
            data: vec![],
            timing: None,
            repeat_count: 1,
            is_collapsed: false,
            group_depth,
        };
        
        {
            let mut messages = self.messages.write().await;
            if messages.len() >= self.config.max_messages {
                messages.pop_front();
            }
            messages.push_back(message.clone());
        }
        
        let _ = self.events.send(ConsoleEvent::Message(message));
    }

    /// Log a table
    pub async fn table(&self, data: &[ConsoleValue], columns: Option<&[String]>) {
        let id = self.next_id().await;
        let group_depth = self.groups.read().await.len() as u32;
        
        let text = format!("[Table: {} items]", data.len());
        
        // In real implementation, would render as actual table
        let message = RichConsoleMessage {
            id,
            timestamp: Utc::now(),
            level: LogLevel::Info,
            category: MessageCategory::Console,
            text,
            parts: vec![],
            url: None,
            line: None,
            column: None,
            stack_trace: None,
            data: data.to_vec(),
            timing: None,
            repeat_count: 1,
            is_collapsed: false,
            group_depth,
        };
        
        {
            let mut messages = self.messages.write().await;
            if messages.len() >= self.config.max_messages {
                messages.pop_front();
            }
            messages.push_back(message.clone());
        }
        
        let _ = self.events.send(ConsoleEvent::Message(message));
    }

    /// Log a directory tree
    pub async fn dir(&self, value: &ConsoleValue, options: Option<DirOptions>) {
        let id = self.next_id().await;
        let group_depth = self.groups.read().await.len() as u32;
        
        let depth = options.map(|o| o.depth).unwrap_or(self.config.max_object_depth);
        let text = Self::format_tree(value, depth);
        
        let message = RichConsoleMessage {
            id,
            timestamp: Utc::now(),
            level: LogLevel::Info,
            category: MessageCategory::Console,
            text,
            parts: vec![],
            url: None,
            line: None,
            column: None,
            stack_trace: None,
            data: vec![value.clone()],
            timing: None,
            repeat_count: 1,
            is_collapsed: false,
            group_depth,
        };
        
        {
            let mut messages = self.messages.write().await;
            if messages.len() >= self.config.max_messages {
                messages.pop_front();
            }
            messages.push_back(message.clone());
        }
        
        let _ = self.events.send(ConsoleEvent::Message(message));
    }

    /// Format value as tree
    fn format_tree(value: &ConsoleValue, depth: usize) -> String {
        if depth == 0 {
            return value.display.clone();
        }
        
        match value.value_type {
            ValueType::Object | ValueType::Array => {
                let mut result = format!("{} {{", value.display);
                for prop in &value.properties {
                    result.push_str(&format!("\n  {}: {}", prop.name, Self::format_tree(&prop.value, depth - 1)));
                }
                result.push_str("\n}");
                result
            }
            _ => value.display.clone(),
        }
    }

    /// Start a timer
    pub async fn time(&self, name: &str) {
        let mut timers = self.timers.write().await;
        timers.insert(name.to_string(), std::time::Instant::now());
    }

    /// End a timer and log result
    pub async fn time_end(&self, name: &str) -> Option<f64> {
        let mut timers = self.timers.write().await;
        if let Some(start) = timers.remove(name) {
            let duration = start.elapsed().as_secs_f64() * 1000.0;
            
            let id = self.next_id().await;
            let group_depth = self.groups.read().await.len() as u32;
            
            let message = RichConsoleMessage {
                id,
                timestamp: Utc::now(),
                level: LogLevel::Info,
                category: MessageCategory::Console,
                text: format!("{}: {} ms", name, duration),
                parts: vec![],
                url: None,
                line: None,
                column: None,
                stack_trace: None,
                data: vec![],
                timing: Some(TimingInfo {
                    name: name.to_string(),
                    duration_ms: duration,
                    start: DateTime::from(start),
                    end: Utc::now(),
                }),
                repeat_count: 1,
                is_collapsed: false,
                group_depth,
            };
            
            {
                let mut messages = self.messages.write().await;
                if messages.len() >= self.config.max_messages {
                    messages.pop_front();
                }
                messages.push_back(message.clone());
            }
            
            let _ = self.events.send(ConsoleEvent::Message(message));
            
            return Some(duration);
        }
        None
    }

    /// Count with label
    pub async fn count(&self, label: &str) {
        let mut counters = self.counters.write().await;
        let count = counters.entry(label.to_string()).or_insert(0);
        *count += 1;
        
        let id = self.next_id().await;
        let group_depth = self.groups.read().await.len() as u32;
        
        let message = RichConsoleMessage {
            id,
            timestamp: Utc::now(),
            level: LogLevel::Info,
            category: MessageCategory::Console,
            text: format!("{}: {}", label, count),
            parts: vec![],
            url: None,
            line: None,
            column: None,
            stack_trace: None,
            data: vec![],
            timing: None,
            repeat_count: 1,
            is_collapsed: false,
            group_depth,
        };
        
        {
            let mut messages = self.messages.write().await;
            if messages.len() >= self.config.max_messages {
                messages.pop_front();
            }
            messages.push_back(message.clone());
        }
        
        let _ = self.events.send(ConsoleEvent::Message(message));
    }

    /// Reset counter
    pub async fn count_reset(&self, label: &str) {
        let mut counters = self.counters.write().await;
        counters.remove(label);
    }

    /// Start a group
    pub async fn group(&self, label: &str) {
        let mut groups = self.groups.write().await;
        let depth = groups.len() as u32;
        
        let group = ConsoleGroup {
            label: label.to_string(),
            depth,
            collapsed: false,
            message_count: 0,
        };
        
        groups.push(group.clone());
        let _ = self.events.send(ConsoleEvent::GroupStarted(group));
    }

    /// Start a collapsed group
    pub async fn group_collapsed(&self, label: &str) {
        let mut groups = self.groups.write().await;
        let depth = groups.len() as u32;
        
        let group = ConsoleGroup {
            label: label.to_string(),
            depth,
            collapsed: true,
            message_count: 0,
        };
        
        groups.push(group.clone());
        let _ = self.events.send(ConsoleEvent::GroupStarted(group));
    }

    /// End current group
    pub async fn group_end(&self) {
        let mut groups = self.groups.write().await;
        if let Some(group) = groups.pop() {
            let _ = self.events.send(ConsoleEvent::GroupEnded { depth: group.depth });
        }
    }

    /// Clear console
    pub async fn clear(&self) {
        {
            let mut messages = self.messages.write().await;
            messages.clear();
        }
        
        {
            let mut groups = self.groups.write().await;
            groups.clear();
        }
        
        let _ = self.events.send(ConsoleEvent::Cleared);
    }

    /// Assert condition
    pub async fn assert(&self, condition: bool, args: &[ConsoleValue]) {
        if !condition {
            self.log_with_level(LogLevel::Error, args, None).await;
        }
    }

    /// Log trace
    pub async fn trace(&self, args: &[ConsoleValue]) {
        let id = self.next_id().await;
        let group_depth = self.groups.read().await.len() as u32;
        
        // Capture stack trace
        let text = args.iter()
            .map(|v| v.display.clone())
            .collect::<Vec<_>>()
            .join(" ");
        
        let message = RichConsoleMessage {
            id,
            timestamp: Utc::now(),
            level: LogLevel::Info,
            category: MessageCategory::Console,
            text,
            parts: vec![],
            url: None,
            line: None,
            column: None,
            stack_trace: Some("Stack trace would be here".to_string()),
            data: args.to_vec(),
            timing: None,
            repeat_count: 1,
            is_collapsed: false,
            group_depth,
        };
        
        {
            let mut messages = self.messages.write().await;
            if messages.len() >= self.config.max_messages {
                messages.pop_front();
            }
            messages.push_back(message.clone());
        }
        
        let _ = self.events.send(ConsoleEvent::Message(message));
    }

    /// Profile start
    pub async fn profile(&self, name: &str) {
        // In real implementation, would start CPU profiler
        let id = self.next_id().await;
        let message = RichConsoleMessage {
            id,
            timestamp: Utc::now(),
            level: LogLevel::Info,
            category: MessageCategory::Performance,
            text: format!("Profile '{}' started", name),
            parts: vec![],
            url: None,
            line: None,
            column: None,
            stack_trace: None,
            data: vec![],
            timing: None,
            repeat_count: 1,
            is_collapsed: false,
            group_depth: 0,
        };
        
        {
            let mut messages = self.messages.write().await;
            messages.push_back(message);
        }
    }

    /// Profile end
    pub async fn profile_end(&self, name: &str) {
        // In real implementation, would stop CPU profiler
        let id = self.next_id().await;
        let message = RichConsoleMessage {
            id,
            timestamp: Utc::now(),
            level: LogLevel::Info,
            category: MessageCategory::Performance,
            text: format!("Profile '{}' finished", name),
            parts: vec![],
            url: None,
            line: None,
            column: None,
            stack_trace: None,
            data: vec![],
            timing: None,
            repeat_count: 1,
            is_collapsed: false,
            group_depth: 0,
        };
        
        {
            let mut messages = self.messages.write().await;
            messages.push_back(message);
        }
    }

    /// Execute command
    pub async fn execute(&self, command: &str) -> Option<ConsoleValue> {
        // Add to history
        {
            let mut history = self.history.write().await;
            history.push_back(command.to_string());
            if history.len() >= self.config.max_history {
                history.pop_front();
            }
        }
        
        // Parse and execute (simplified)
        let result = self.evaluate_expression(command).await;
        
        let _ = self.events.send(ConsoleEvent::CommandExecuted {
            command: command.to_string(),
            result: result.clone(),
        });
        
        result
    }

    /// Evaluate expression (simplified)
    async fn evaluate_expression(&self, expr: &str) -> Option<ConsoleValue> {
        // Simplified evaluation - in real impl would use JS engine
        Some(ConsoleValue {
            value_type: ValueType::String,
            display: expr.to_string(),
            preview: None,
            properties: vec![],
            array_items: vec![],
            raw: Some(expr.to_string()),
        })
    }

    /// Get messages
    pub async fn get_messages(&self) -> Vec<RichConsoleMessage> {
        let messages = self.messages.read().await;
        messages.iter().cloned().collect()
    }

    /// Get command history
    pub async fn get_history(&self) -> Vec<String> {
        let history = self.history.read().await;
        history.iter().cloned().collect()
    }

    /// Filter messages
    pub async fn filter(&self, filter: &ConsoleFilter) -> Vec<RichConsoleMessage> {
        let messages = self.messages.read().await;
        
        messages
            .iter()
            .filter(|m| {
                // Level filter
                if let Some(level) = filter.level {
                    if m.level != level {
                        return false;
                    }
                }
                
                // Category filter
                if let Some(category) = &filter.category {
                    if &m.category != category {
                        return false;
                    }
                }
                
                // Text filter
                if let Some(text) = &filter.text {
                    if !m.text.to_lowercase().contains(&text.to_lowercase()) {
                        return false;
                    }
                }
                
                true
            })
            .cloned()
            .collect()
    }

    /// Subscribe to console events
    pub fn subscribe(&self) -> broadcast::Receiver<ConsoleEvent> {
        self.events.subscribe()
    }
}

/// Options for dir()
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DirOptions {
    /// Maximum depth
    pub depth: usize,
}

/// Console filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleFilter {
    /// Filter by level
    pub level: Option<LogLevel>,
    /// Filter by category
    pub category: Option<MessageCategory>,
    /// Filter by text
    pub text: Option<String>,
    /// Filter by URL
    pub url: Option<String>,
}

impl Default for ConsoleFilter {
    fn default() -> Self {
        Self {
            level: None,
            category: None,
            text: None,
            url: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_log() {
        let console = Console::new(ConsoleConfig::default());
        console.log(&[ConsoleValue {
            value_type: ValueType::String,
            display: "Hello".to_string(),
            preview: None,
            properties: vec![],
            array_items: vec![],
            raw: Some("Hello".to_string()),
        }]).await;
        
        let messages = console.get_messages().await;
        assert_eq!(messages.len(), 1);
    }

    #[tokio::test]
    async fn test_time() {
        let console = Console::new(ConsoleConfig::default());
        console.time("test").await;
        let duration = console.time_end("test").await;
        assert!(duration.is_some());
    }

    #[tokio::test]
    async fn test_count() {
        let console = Console::new(ConsoleConfig::default());
        console.count("x").await;
        console.count("x").await;
        console.count("x").await;
        
        let messages = console.get_messages().await;
        assert_eq!(messages.len(), 3);
    }
}