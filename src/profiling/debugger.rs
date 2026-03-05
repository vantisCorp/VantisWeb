//! Debugger and Diagnostics
//! 
//! This module provides debugging and diagnostic capabilities for the browser.
//! It includes breakpoints, stepping, call stack inspection, and variable monitoring.
//! 
//! # Features
//! - Breakpoint management
//! - Step-by-step execution
//! - Call stack inspection
//! - Variable watch and evaluation
//! - Source mapping support
//! - Console integration

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// Breakpoint type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BreakpointType {
    Line,
    Conditional,
    Log,
    Exception,
    DOM,
    XHR,
}

/// Breakpoint condition operator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BreakpointOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    Contains,
}

/// Breakpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    pub id: String,
    pub url: String,
    pub line_number: u32,
    pub column_number: Option<u32>,
    pub breakpoint_type: BreakpointType,
    pub condition: Option<String>,
    pub operator: Option<BreakpointOperator>,
    pub enabled: bool,
    pub hit_count: u32,
    pub ignore_count: u32,
}

impl Breakpoint {
    /// Create a new line breakpoint
    pub fn new_line(url: String, line_number: u32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            url,
            line_number,
            column_number: None,
            breakpoint_type: BreakpointType::Line,
            condition: None,
            operator: None,
            enabled: true,
            hit_count: 0,
            ignore_count: 0,
        }
    }

    /// Create a conditional breakpoint
    pub fn new_conditional(url: String, line_number: u32, condition: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            url,
            line_number,
            column_number: None,
            breakpoint_type: BreakpointType::Conditional,
            condition: Some(condition),
            operator: None,
            enabled: true,
            hit_count: 0,
            ignore_count: 0,
        }
    }

    /// Check if breakpoint should trigger
    pub fn should_trigger(&self) -> bool {
        if !self.enabled {
            return false;
        }

        if self.ignore_count > 0 {
            return false;
        }

        true
    }

    /// Increment hit count
    pub fn increment_hit(&mut self) {
        self.hit_count += 1;
    }

    /// Decrement ignore count
    pub fn decrement_ignore(&mut self) {
        if self.ignore_count > 0 {
            self.ignore_count -= 1;
        }
    }
}

/// Stack frame information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    pub id: u32,
    pub function_name: String,
    pub url: String,
    pub line_number: u32,
    pub column_number: u32,
    pub script_id: String,
    pub scope_chain: Vec<Scope>,
    pub this_object: Option<Value>,
}

/// Scope information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope {
    pub name: String,
    pub scope_type: ScopeType,
    pub object_id: String,
}

/// Scope type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScopeType {
    Global,
    Local,
    With,
    Closure,
    Catch,
    Block,
    Script,
}

/// JavaScript value representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Value {
    pub value_type: ValueType,
    pub value: String,
    pub object_id: Option<String>,
    pub description: Option<String>,
    pub subtype: Option<String>,
}

/// Value type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValueType {
    Object,
    Function,
    Undefined,
    String,
    Number,
    Boolean,
    Symbol,
    BigInt,
}

/// Variable information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub value: Value,
    pub writable: bool,
    pub configurable: bool,
    pub enumerable: bool,
}

/// Watch expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchExpression {
    pub id: String,
    pub expression: String,
    pub value: Option<Value>,
    pub error: Option<String>,
}

/// Source location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    pub script_id: String,
    pub url: String,
    pub line_number: u32,
    pub column_number: u32,
}

/// Debugger state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebuggerState {
    Running,
    Paused,
    Stepping,
    Stopped,
}

/// Step type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepType {
    Into,
    Over,
    Out,
}

/// Console message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleMessage {
    pub level: ConsoleLevel,
    pub text: String,
    pub url: Option<String>,
    pub line_number: Option<u32>,
    pub column_number: Option<u32>,
    pub timestamp: i64,
}

/// Console message level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsoleLevel {
    Log,
    Info,
    Warning,
    Error,
    Debug,
}

/// Debugger configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebuggerConfig {
    pub max_breakpoints: usize,
    pub max_call_stack_depth: u32,
    pub auto_continue_on_exception: bool,
    pub pause_on_caught_exceptions: bool,
    pub pause_on_uncaught_exceptions: bool,
    pub enable_source_maps: bool,
}

impl Default for DebuggerConfig {
    fn default() -> Self {
        Self {
            max_breakpoints: 1000,
            max_call_stack_depth: 50,
            auto_continue_on_exception: false,
            pause_on_caught_exceptions: false,
            pause_on_uncaught_exceptions: true,
            enable_source_maps: true,
        }
    }
}

/// Main debugger
pub struct Debugger {
    config: Arc<RwLock<DebuggerConfig>>,
    breakpoints: Arc<RwLock<HashMap<String, Breakpoint>>>,
    breakpoint_lines: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    call_stack: Arc<RwLock<Vec<StackFrame>>>,
    watch_expressions: Arc<RwLock<Vec<WatchExpression>>>,
    state: Arc<RwLock<DebuggerState>>,
    current_location: Arc<RwLock<Option<SourceLocation>>>,
    console_messages: Arc<RwLock<Vec<ConsoleMessage>>>,
    scripts: Arc<RwLock<HashMap<String, String>>>,
    variables: Arc<RwLock<HashMap<String, Variable>>>,
}

impl Debugger {
    /// Create a new debugger
    pub fn new(config: DebuggerConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            breakpoints: Arc::new(RwLock::new(HashMap::new())),
            breakpoint_lines: Arc::new(RwLock::new(HashMap::new())),
            call_stack: Arc::new(RwLock::new(Vec::new())),
            watch_expressions: Arc::new(RwLock::new(Vec::new())),
            state: Arc::new(RwLock::new(DebuggerState::Stopped)),
            current_location: Arc::new(RwLock::new(None)),
            console_messages: Arc::new(RwLock::new(Vec::new())),
            scripts: Arc::new(RwLock::new(HashMap::new())),
            variables: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Enable the debugger
    pub async fn enable(&self) {
        *self.state.write().await = DebuggerState::Running;
    }

    /// Disable the debugger
    pub async fn disable(&self) {
        *self.state.write().await = DebuggerState::Stopped;
        self.breakpoints.write().await.clear();
        self.breakpoint_lines.write().await.clear();
        self.call_stack.write().await.clear();
    }

    /// Check if debugger is enabled
    pub async fn is_enabled(&self) -> bool {
        *self.state.read().await != DebuggerState::Stopped
    }

    /// Set debugger state
    pub async fn set_state(&self, state: DebuggerState) {
        *self.state.write().await = state;
    }

    /// Get current debugger state
    pub async fn get_state(&self) -> DebuggerState {
        *self.state.read().await
    }

    /// Add a breakpoint
    pub async fn add_breakpoint(&self, breakpoint: Breakpoint) -> Result<(), String> {
        let config = self.config.read().await;
        if self.breakpoints.read().await.len() >= config.max_breakpoints {
            return Err("Maximum breakpoints reached".to_string());
        }

        let id = breakpoint.id.clone();
        let line_key = format!("{}:{}", breakpoint.url, breakpoint.line_number);

        self.breakpoints.write().await.insert(id.clone(), breakpoint);
        self.breakpoint_lines.write().await
            .entry(line_key)
            .or_insert_with(HashSet::new)
            .insert(id);

        Ok(())
    }

    /// Remove a breakpoint
    pub async fn remove_breakpoint(&self, id: &str) -> Result<(), String> {
        let mut breakpoints = self.breakpoints.write().await;
        let mut breakpoint_lines = self.breakpoint_lines.write().await;

        if let Some(breakpoint) = breakpoints.remove(id) {
            let line_key = format!("{}:{}", breakpoint.url, breakpoint.line_number);
            if let Some(set) = breakpoint_lines.get_mut(&line_key) {
                set.remove(id);
                if set.is_empty() {
                    breakpoint_lines.remove(&line_key);
                }
            }
            Ok(())
        } else {
            Err(format!("Breakpoint not found: {}", id))
        }
    }

    /// Get all breakpoints
    pub async fn get_breakpoints(&self) -> Vec<Breakpoint> {
        self.breakpoints.read().await.values().cloned().collect()
    }

    /// Get breakpoints for a URL
    pub async fn get_breakpoints_for_url(&self, url: &str) -> Vec<Breakpoint> {
        self.breakpoints.read().await
            .values()
            .filter(|b| b.url == url)
            .cloned()
            .collect()
    }

    /// Check if there's a breakpoint at a location
    pub async fn has_breakpoint(&self, url: &str, line_number: u32) -> bool {
        let line_key = format!("{}:{}", url, line_number);
        self.breakpoint_lines.read().await
            .get(&line_key)
            .map(|set| !set.is_empty())
            .unwrap_or(false)
    }

    /// Get breakpoints at a location
    pub async fn get_breakpoints_at(&self, url: &str, line_number: u32) -> Vec<Breakpoint> {
        let line_key = format!("{}:{}", url, line_number);
        if let Some(ids) = self.breakpoint_lines.read().await.get(&line_key) {
            let breakpoints = self.breakpoints.read().await;
            ids.iter()
                .filter_map(|id| breakpoints.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Enable a breakpoint
    pub async fn enable_breakpoint(&self, id: &str) -> Result<(), String> {
        let mut breakpoints = self.breakpoints.write().await;
        if let Some(breakpoint) = breakpoints.get_mut(id) {
            breakpoint.enabled = true;
            Ok(())
        } else {
            Err(format!("Breakpoint not found: {}", id))
        }
    }

    /// Disable a breakpoint
    pub async fn disable_breakpoint(&self, id: &str) -> Result<(), String> {
        let mut breakpoints = self.breakpoints.write().await;
        if let Some(breakpoint) = breakpoints.get_mut(id) {
            breakpoint.enabled = false;
            Ok(())
        } else {
            Err(format!("Breakpoint not found: {}", id))
        }
    }

    /// Set breakpoint condition
    pub async fn set_breakpoint_condition(&self, id: &str, condition: String) -> Result<(), String> {
        let mut breakpoints = self.breakpoints.write().await;
        if let Some(breakpoint) = breakpoints.get_mut(id) {
            breakpoint.condition = Some(condition);
            breakpoint.breakpoint_type = BreakpointType::Conditional;
            Ok(())
        } else {
            Err(format!("Breakpoint not found: {}", id))
        }
    }

    /// Set current location
    pub async fn set_current_location(&self, location: SourceLocation) {
        *self.current_location.write().await = Some(location);
    }

    /// Get current location
    pub async fn get_current_location(&self) -> Option<SourceLocation> {
        self.current_location.read().await.clone()
    }

    /// Update call stack
    pub async fn update_call_stack(&self, stack: Vec<StackFrame>) {
        let mut call_stack = self.call_stack.write().await;
        let config = self.config.read().await;

        let truncated: Vec<StackFrame> = stack
            .into_iter()
            .take(config.max_call_stack_depth as usize)
            .collect();

        *call_stack = truncated;
    }

    /// Get current call stack
    pub async fn get_call_stack(&self) -> Vec<StackFrame> {
        self.call_stack.read().await.clone()
    }

    /// Add a watch expression
    pub async fn add_watch_expression(&self, expression: String) -> String {
        let watch = WatchExpression {
            id: Uuid::new_v4().to_string(),
            expression,
            value: None,
            error: None,
        };

        let id = watch.id.clone();
        self.watch_expressions.write().await.push(watch);
        id
    }

    /// Remove a watch expression
    pub async fn remove_watch_expression(&self, id: &str) -> Result<(), String> {
        let mut watches = self.watch_expressions.write().await;
        if let Some(pos) = watches.iter().position(|w| w.id == id) {
            watches.remove(pos);
            Ok(())
        } else {
            Err(format!("Watch expression not found: {}", id))
        }
    }

    /// Get all watch expressions
    pub async fn get_watch_expressions(&self) -> Vec<WatchExpression> {
        self.watch_expressions.read().await.clone()
    }

    /// Evaluate a watch expression
    pub async fn evaluate_watch(&self, id: &str, value: Value) {
        let mut watches = self.watch_expressions.write().await;
        if let Some(watch) = watches.iter_mut().find(|w| w.id == id) {
            watch.value = Some(value);
            watch.error = None;
        }
    }

    /// Set watch error
    pub async fn set_watch_error(&self, id: &str, error: String) {
        let mut watches = self.watch_expressions.write().await;
        if let Some(watch) = watches.iter_mut().find(|w| w.id == id) {
            watch.value = None;
            watch.error = Some(error);
        }
    }

    /// Add a console message
    pub async fn add_console_message(&self, message: ConsoleMessage) {
        let mut console = self.console_messages.write().await;
        
        // Limit console messages to prevent memory issues
        if console.len() >= 1000 {
            console.remove(0);
        }
        
        console.push(message);
    }

    /// Get console messages
    pub async fn get_console_messages(&self) -> Vec<ConsoleMessage> {
        self.console_messages.read().await.clone()
    }

    /// Clear console messages
    pub async fn clear_console(&self) {
        self.console_messages.write().await.clear();
    }

    /// Register a script
    pub async fn register_script(&self, script_id: String, url: String) {
        self.scripts.write().await.insert(script_id, url);
    }

    /// Unregister a script
    pub async fn unregister_script(&self, script_id: &str) {
        self.scripts.write().await.remove(script_id);
    }

    /// Get all scripts
    pub async fn get_scripts(&self) -> HashMap<String, String> {
        self.scripts.read().await.clone()
    }

    /// Get script URL by ID
    pub async fn get_script_url(&self, script_id: &str) -> Option<String> {
        self.scripts.read().await.get(script_id).cloned()
    }

    /// Set a variable
    pub async fn set_variable(&self, variable: Variable) {
        self.variables.write().await.insert(variable.name.clone(), variable);
    }

    /// Get a variable
    pub async fn get_variable(&self, name: &str) -> Option<Variable> {
        self.variables.read().await.get(name).cloned()
    }

    /// Get all variables
    pub async fn get_variables(&self) -> HashMap<String, Variable> {
        self.variables.read().await.clone()
    }

    /// Evaluate an expression
    pub async fn evaluate(&self, expression: &str) -> Result<Value, String> {
        // In a real implementation, this would use a JavaScript engine
        // For now, return a placeholder
        Ok(Value {
            value_type: ValueType::String,
            value: expression.to_string(),
            object_id: None,
            description: Some(format!("Evaluated: {}", expression)),
            subtype: None,
        })
    }

    /// Get debugger summary
    pub async fn get_summary(&self) -> DebuggerSummary {
        let breakpoints = self.breakpoints.read().await;
        let call_stack = self.call_stack.read().await;
        let state = *self.state.read().await;

        let enabled_breakpoints = breakpoints.values()
            .filter(|b| b.enabled)
            .count();

        let total_hit_count: u32 = breakpoints.values()
            .map(|b| b.hit_count)
            .sum();

        DebuggerSummary {
            state,
            total_breakpoints: breakpoints.len(),
            enabled_breakpoints,
            total_hit_count,
            call_stack_depth: call_stack.len(),
            current_location: self.current_location.read().await.clone(),
        }
    }

    /// Clear all data
    pub async fn clear(&self) {
        self.breakpoints.write().await.clear();
        self.breakpoint_lines.write().await.clear();
        self.call_stack.write().await.clear();
        self.console_messages.write().await.clear();
        self.variables.write().await.clear();
        *self.current_location.write().await = None;
    }
}

/// Debugger summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebuggerSummary {
    pub state: DebuggerState,
    pub total_breakpoints: usize,
    pub enabled_breakpoints: usize,
    pub total_hit_count: u32,
    pub call_stack_depth: usize,
    pub current_location: Option<SourceLocation>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_breakpoint_creation() {
        let breakpoint = Breakpoint::new_line("test.js".to_string(), 10);
        assert_eq!(breakpoint.url, "test.js");
        assert_eq!(breakpoint.line_number, 10);
        assert!(breakpoint.enabled);
        assert!(breakpoint.should_trigger());
    }

    #[tokio::test]
    async fn test_debugger_basic() {
        let debugger = Debugger::new(DebuggerConfig::default());
        debugger.enable().await;

        let breakpoint = Breakpoint::new_line("test.js".to_string(), 10);
        debugger.add_breakpoint(breakpoint).await.unwrap();

        let breakpoints = debugger.get_breakpoints().await;
        assert_eq!(breakpoints.len(), 1);
        assert!(debugger.has_breakpoint("test.js", 10).await);
    }

    #[test]
    fn test_watch_expression() {
        let watch = WatchExpression {
            id: "test".to_string(),
            expression: "x + y".to_string(),
            value: None,
            error: None,
        };

        assert_eq!(watch.expression, "x + y");
        assert!(watch.value.is_none());
    }
}