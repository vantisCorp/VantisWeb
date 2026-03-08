//! Developer Tools Data Models
//! 
//! Data structures for developer tools functionality.

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

// Re-export main types from mod.rs
pub use super::{
    DevToolsConfig, DevToolsEvent, SecuritySeverity,
    ConsoleMessage, ConsoleMessageType, StackFrame,
    Breakpoint, WatchExpression, PageSnapshot,
    NetworkRequestInfo, MemorySnapshot, PerformanceMetric,
    DiagnosticReport, SecurityIssue, MemoryAnalysis,
    PerformanceAnalysis, NetworkAnalysis,
};

/// DOM Node representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DOMNode {
    /// Node ID
    pub id: u64,
    /// Node type
    pub node_type: NodeType,
    /// Tag name (for elements)
    pub tag_name: Option<String>,
    /// Node value (for text/comments)
    pub node_value: Option<String>,
    /// Attributes
    pub attributes: Vec<DOMAttribute>,
    /// Child node IDs
    pub children: Vec<u64>,
    /// Parent node ID
    pub parent_id: Option<u64>,
    /// Computed styles
    pub computed_styles: Vec<ComputedStyle>,
    /// Bounding box
    pub bounding_box: Option<BoundingBox>,
}

/// DOM Node types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    Element,
    Text,
    Comment,
    Document,
    DocumentType,
    DocumentFragment,
    CDataSection,
}

/// DOM Attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DOMAttribute {
    /// Attribute name
    pub name: String,
    /// Attribute value
    pub value: String,
    /// Is event handler
    pub is_event_handler: bool,
}

/// Computed style
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputedStyle {
    /// Property name
    pub property: String,
    /// Property value
    pub value: String,
    /// Priority (important)
    pub important: bool,
    /// Source stylesheet
    pub source: Option<StyleSource>,
}

/// Style source information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleSource {
    /// Stylesheet URL
    pub stylesheet_url: Option<String>,
    /// Line number
    pub line: u32,
    /// Column number
    pub column: u32,
    /// Selector
    pub selector: Option<String>,
}

/// Bounding box
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoundingBox {
    /// X position
    pub x: f64,
    /// Y position
    pub y: f64,
    /// Width
    pub width: f64,
    /// Height
    pub height: f64,
}

/// CSS Rule representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CSSRule {
    /// Rule ID
    pub id: String,
    /// Rule type
    pub rule_type: CSSRuleType,
    /// Selector text
    pub selector: Option<String>,
    /// Properties
    pub properties: Vec<CSSProperty>,
    /// Source location
    pub source: StyleSource,
    /// Is user agent stylesheet
    pub is_user_agent: bool,
    /// Is inline style
    pub is_inline: bool,
}

/// CSS Rule types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CSSRuleType {
    Style,
    Import,
    Media,
    FontFace,
    Page,
    Keyframes,
    Keyframe,
    Namespace,
    CounterStyle,
    Supports,
    Document,
    FontFeatureValues,
}

/// CSS Property
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CSSProperty {
    /// Property name
    pub name: String,
    /// Property value
    pub value: String,
    /// Is important
    pub important: bool,
    /// Is disabled
    pub disabled: bool,
    /// Parsed value
    pub parsed_value: Option<CSSParsedValue>,
}

/// Parsed CSS value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CSSParsedValue {
    /// Value type
    pub value_type: String,
    /// Numeric value (if applicable)
    pub numeric_value: Option<f64>,
    /// Unit (if applicable)
    pub unit: Option<String>,
    /// Color (if applicable)
    pub color: Option<ColorValue>,
}

/// Color value
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ColorValue {
    /// Red component
    pub r: u8,
    /// Green component
    pub g: u8,
    /// Blue component
    pub b: u8,
    /// Alpha component
    pub a: u8,
}

/// Network request details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequest {
    /// Request ID
    pub id: String,
    /// URL
    pub url: String,
    /// HTTP method
    pub method: String,
    /// Request headers
    pub request_headers: Vec<HTTPHeader>,
    /// Request body
    pub request_body: Option<Vec<u8>>,
    /// Response status
    pub response_status: Option<u16>,
    /// Response status text
    pub response_status_text: Option<String>,
    /// Response headers
    pub response_headers: Vec<HTTPHeader>,
    /// Response body
    pub response_body: Option<Vec<u8>>,
    /// Response body preview
    pub response_preview: Option<String>,
    /// Timing information
    pub timing: RequestTiming,
    /// Resource type
    pub resource_type: ResourceType,
    /// Priority
    pub priority: RequestPriority,
    /// Was cached
    pub from_cache: bool,
    /// Initiator
    pub initiator: Option<RequestInitiator>,
    /// Error (if any)
    pub error: Option<String>,
}

/// HTTP Header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HTTPHeader {
    /// Header name
    pub name: String,
    /// Header value
    pub value: String,
}

/// Request timing information
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RequestTiming {
    /// DNS lookup time
    pub dns_time_ms: u64,
    /// Connection time
    pub connect_time_ms: u64,
    /// SSL time
    pub ssl_time_ms: u64,
    /// Request sent time
    pub send_time_ms: u64,
    /// Waiting time (TTFB)
    pub wait_time_ms: u64,
    /// Content download time
    pub receive_time_ms: u64,
    /// Total time
    pub total_time_ms: u64,
    /// Start time
    pub start_time: DateTime<Utc>,
    /// End time
    pub end_time: DateTime<Utc>,
}

/// Resource types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    Document,
    Stylesheet,
    Image,
    Media,
    Font,
    Script,
    TextTrack,
    XHR,
    Fetch,
    EventSource,
    WebSocket,
    Manifest,
    SignedExchange,
    Ping,
    CSPViolationReport,
    Preflight,
    Other,
}

/// Request priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RequestPriority {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}

/// Request initiator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestInitiator {
    /// Initiator type
    pub initiator_type: InitiatorType,
    /// Stack trace
    pub stack: Option<Vec<StackFrame>>,
    /// URL
    pub url: Option<String>,
    /// Line number
    pub line_number: Option<u32>,
}

/// Initiator types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InitiatorType {
    Parser,
    Script,
    Preload,
    Redirect,
    Other,
}

/// Performance timeline entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceEntry {
    /// Entry ID
    pub id: String,
    /// Entry type
    pub entry_type: PerformanceEntryType,
    /// Entry name
    pub name: String,
    /// Start time
    pub start_time: f64,
    /// Duration
    pub duration: f64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Additional data
    pub data: serde_json::Value,
}

/// Performance entry types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PerformanceEntryType {
    Navigation,
    Resource,
    Mark,
    Measure,
    Paint,
    LayoutShift,
    LongTask,
    FirstInput,
    Element,
    Event,
}

/// Heap snapshot node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeapNode {
    /// Node ID
    pub id: u64,
    /// Node type
    pub node_type: HeapNodeType,
    /// Node name
    pub name: String,
    /// Self size
    pub self_size: u64,
    /// Retained size
    pub retained_size: u64,
    /// Edge count
    pub edge_count: u32,
    /// Distance from root
    pub distance: u32,
}

/// Heap node types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeapNodeType {
    Hidden,
    Array,
    String,
    Object,
    Code,
    Closure,
    RegExp,
    Number,
    Native,
    Synthetic,
    ConcatenatedString,
    SlicedString,
    Symbol,
    BigInt,
}

/// Heap edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeapEdge {
    /// Edge type
    pub edge_type: HeapEdgeType,
    /// Name or index
    pub name_or_index: serde_json::Value,
    /// Target node ID
    pub target_node: u64,
}

/// Heap edge types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeapEdgeType {
    Context,
    Element,
    Property,
    Internal,
    Hidden,
    Shortcut,
    Weak,
}

/// Source map entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMapEntry {
    /// Generated line
    pub generated_line: u32,
    /// Generated column
    pub generated_column: u32,
    /// Original file
    pub original_file: Option<String>,
    /// Original line
    pub original_line: Option<u32>,
    /// Original column
    pub original_column: Option<u32>,
    /// Original name
    pub original_name: Option<String>,
}

/// Source file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFile {
    /// File URL
    pub url: String,
    /// File content
    pub content: Option<String>,
    /// Source map URL
    pub source_map_url: Option<String>,
    /// Parsed source map
    pub source_map: Option<Vec<SourceMapEntry>>,
    /// Content type
    pub content_type: String,
    /// Is blackboxed
    pub is_blackboxed: bool,
    /// Is content script
    pub is_content_script: bool,
}

/// Test case definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    /// Test ID
    pub id: String,
    /// Test name
    pub name: String,
    /// Test description
    pub description: Option<String>,
    /// Test suite
    pub suite: String,
    /// Test code
    pub code: String,
    /// Expected result
    pub expected: Option<String>,
    /// Timeout in ms
    pub timeout_ms: u64,
    /// Is skipped
    pub skipped: bool,
    /// Tags
    pub tags: Vec<String>,
}

/// Test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Test ID
    pub test_id: String,
    /// Test name
    pub name: String,
    /// Pass/fail
    pub passed: bool,
    /// Error message
    pub error: Option<String>,
    /// Duration in ms
    pub duration_ms: u64,
    /// Assertions made
    pub assertions: u32,
    /// Screenshots
    pub screenshots: Vec<String>,
    /// Console output
    pub console_output: Vec<String>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Code editor position
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EditorPosition {
    /// Line number (0-based)
    pub line: u32,
    /// Column number (0-based)
    pub column: u32,
}

/// Code editor selection
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EditorSelection {
    /// Start position
    pub start: EditorPosition,
    /// End position
    pub end: EditorPosition,
}

/// Code editor decoration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorDecoration {
    /// Decoration ID
    pub id: String,
    /// Decoration type
    pub decoration_type: DecorationType,
    /// Range
    pub range: EditorSelection,
    /// Options
    pub options: DecorationOptions,
}

/// Decoration types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecorationType {
    Error,
    Warning,
    Info,
    Breakpoint,
    CurrentLine,
    SearchResult,
    Link,
}

/// Decoration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecorationOptions {
    /// Hover message
    pub hover_message: Option<String>,
    /// Inline class name
    pub inline_class: Option<String>,
    /// Line class name
    pub line_class: Option<String>,
    /// Gutter icon
    pub gutter_icon: Option<String>,
    /// Is visible
    pub visible: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dom_node_creation() {
        let node = DOMNode {
            id: 1,
            node_type: NodeType::Element,
            tag_name: Some("div".to_string()),
            node_value: None,
            attributes: vec![],
            children: vec![2, 3],
            parent_id: None,
            computed_styles: vec![],
            bounding_box: None,
        };
        
        assert_eq!(node.id, 1);
        assert_eq!(node.tag_name, Some("div".to_string()));
    }
    
    #[test]
    fn test_resource_type() {
        assert_ne!(ResourceType::Document, ResourceType::Script);
        assert_eq!(ResourceType::XHR, ResourceType::XHR);
    }
}