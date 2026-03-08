//! Code Editor with syntax highlighting and IntelliSense
//! 
//! Provides advanced code editing capabilities including:
//! - Syntax highlighting for multiple languages
//! - Code completion and IntelliSense
//! - Code formatting
//! - Find and replace with regex
//! - Multiple cursor support
//! - Code folding
//! - Minimap

use crate::developer_tools::models::{EditorPosition, EditorSelection, EditorDecoration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Supported language identifiers
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    JavaScript,
    TypeScript,
    Html,
    Css,
    Json,
    Markdown,
    Rust,
    Python,
    Wasm,
    Unknown,
}

impl Default for Language {
    fn default() -> Self {
        Language::Unknown
    }
}

/// Token type for syntax highlighting
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TokenType {
    Keyword,
    String,
    Number,
    Comment,
    Function,
    Variable,
    Operator,
    Punctuation,
    Type,
    Property,
    Tag,
    Attribute,
    Value,
    Unknown,
}

/// Syntax token with position and type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxToken {
    /// Token type
    pub token_type: TokenType,
    /// Start position (byte offset)
    pub start: usize,
    /// End position (byte offset)
    pub end: usize,
    /// Text content
    pub text: String,
}

/// Code completion item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    /// Label shown in the suggestion list
    pub label: String,
    /// Kind of completion
    pub kind: CompletionKind,
    /// Detail text (type signature, etc.)
    pub detail: Option<String>,
    /// Documentation string
    pub documentation: Option<String>,
    /// Text to insert
    pub insert_text: String,
    /// Whether it's a snippet (contains $1, $2, etc.)
    pub is_snippet: bool,
    /// Sort priority (lower is higher priority)
    pub sort_priority: u32,
}

/// Kind of completion
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompletionKind {
    Function,
    Method,
    Property,
    Variable,
    Class,
    Interface,
    Module,
    Keyword,
    Snippet,
    File,
    Folder,
    Constant,
    Enum,
    EnumMember,
    Field,
    Constructor,
}

/// Code diagnostic (error, warning, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Message
    pub message: String,
    /// Severity
    pub severity: DiagnosticSeverity,
    /// Range (start line, start col, end line, end col)
    pub range: ((usize, usize), (usize, usize)),
    /// Source (linter name, etc.)
    pub source: Option<String>,
    /// Code
    pub code: Option<String>,
    /// Related information
    pub related: Vec<DiagnosticRelated>,
}

/// Diagnostic severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

/// Related diagnostic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticRelated {
    pub message: String,
    pub location: String,
}

/// Foldable region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoldingRange {
    /// Start line
    pub start_line: usize,
    /// End line
    pub end_line: usize,
    /// Kind of fold
    pub kind: FoldingKind,
}

/// Kind of folding region
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FoldingKind {
    Comment,
    Imports,
    Region,
    Function,
    Class,
    Object,
    Array,
}

/// Text edit operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextEdit {
    Insert {
        position: EditorPosition,
        text: String,
    },
    Delete {
        range: (EditorPosition, EditorPosition),
    },
    Replace {
        range: (EditorPosition, EditorPosition),
        text: String,
    },
}

/// Undo/Redo stack item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditOperation {
    /// Edit operations
    pub edits: Vec<TextEdit>,
    /// Cursor position before edit
    pub cursor_before: EditorPosition,
    /// Cursor position after edit
    pub cursor_after: EditorPosition,
    /// Description for undo menu
    pub description: String,
}

/// Find match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindMatch {
    /// Start position
    pub start: EditorPosition,
    /// End position
    pub end: EditorPosition,
    /// Matched text
    pub text: String,
    /// Capture groups (for regex)
    pub groups: Vec<String>,
}

/// Editor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    /// Tab size (spaces)
    pub tab_size: usize,
    /// Use tabs instead of spaces
    pub use_tabs: bool,
    /// Auto-indent on new lines
    pub auto_indent: bool,
    /// Show line numbers
    pub line_numbers: bool,
    /// Show minimap
    pub minimap: bool,
    /// Font size
    pub font_size: u32,
    /// Font family
    pub font_family: String,
    /// Word wrap
    pub word_wrap: bool,
    /// Auto-save delay (ms, 0 = disabled)
    pub auto_save_delay: u32,
    /// Enable autocomplete
    pub autocomplete: bool,
    /// Enable bracket matching
    pub bracket_matching: bool,
    /// Enable syntax highlighting
    pub syntax_highlighting: bool,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            tab_size: 4,
            use_tabs: false,
            auto_indent: true,
            line_numbers: true,
            minimap: true,
            font_size: 14,
            font_family: "monospace".to_string(),
            word_wrap: false,
            auto_save_delay: 1000,
            autocomplete: true,
            bracket_matching: true,
            syntax_highlighting: true,
        }
    }
}

/// Code Editor
pub struct CodeEditor {
    /// Editor configuration
    config: EditorConfig,
    /// Current file path
    file_path: Option<String>,
    /// Current content
    content: Arc<RwLock<String>>,
    /// Detected language
    language: Arc<RwLock<Language>>,
    /// Syntax tokens (cached)
    tokens: Arc<RwLock<Vec<SyntaxToken>>>,
    /// Diagnostics
    diagnostics: Arc<RwLock<Vec<Diagnostic>>>,
    /// Undo stack
    undo_stack: Arc<RwLock<Vec<EditOperation>>>,
    /// Redo stack
    redo_stack: Arc<RwLock<Vec<EditOperation>>>,
    /// Cursor position
    cursor: Arc<RwLock<EditorPosition>>,
    /// Selection (if any)
    selection: Arc<RwLock<Option<EditorSelection>>>,
    /// Decorations (highlights, etc.)
    decorations: Arc<RwLock<Vec<EditorDecoration>>>,
    /// Folding ranges
    folds: Arc<RwLock<Vec<FoldingRange>>>,
}

impl CodeEditor {
    /// Create a new code editor
    pub fn new(config: EditorConfig) -> Self {
        Self {
            config,
            file_path: None,
            content: Arc::new(RwLock::new(String::new())),
            language: Arc::new(RwLock::new(Language::Unknown)),
            tokens: Arc::new(RwLock::new(Vec::new())),
            diagnostics: Arc::new(RwLock::new(Vec::new())),
            undo_stack: Arc::new(RwLock::new(Vec::new())),
            redo_stack: Arc::new(RwLock::new(Vec::new())),
            cursor: Arc::new(RwLock::new(EditorPosition { line: 0, column: 0 })),
            selection: Arc::new(RwLock::new(None)),
            decorations: Arc::new(RwLock::new(Vec::new())),
            folds: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Open a file
    pub async fn open_file(&mut self, path: &str, content: &str) {
        self.file_path = Some(path.to_string());
        
        let mut c = self.content.write().await;
        *c = content.to_string();
        
        // Detect language from file extension
        let lang = Self::detect_language(path);
        let mut l = self.language.write().await;
        *l = lang;
        
        // Tokenize
        drop(c);
        drop(l);
        self.tokenize().await;
    }

    /// Detect language from file extension
    fn detect_language(path: &str) -> Language {
        let path_lower = path.to_lowercase();
        if path_lower.ends_with(".js") || path_lower.ends_with(".mjs") || path_lower.ends_with(".cjs") {
            Language::JavaScript
        } else if path_lower.ends_with(".ts") || path_lower.ends_with(".tsx") {
            Language::TypeScript
        } else if path_lower.ends_with(".html") || path_lower.ends_with(".htm") {
            Language::Html
        } else if path_lower.ends_with(".css") {
            Language::Css
        } else if path_lower.ends_with(".json") {
            Language::Json
        } else if path_lower.ends_with(".md") {
            Language::Markdown
        } else if path_lower.ends_with(".rs") {
            Language::Rust
        } else if path_lower.ends_with(".py") {
            Language::Python
        } else if path_lower.ends_with(".wasm") {
            Language::Wasm
        } else {
            Language::Unknown
        }
    }

    /// Get current content
    pub async fn get_content(&self) -> String {
        let content = self.content.read().await;
        content.clone()
    }

    /// Set content
    pub async fn set_content(&self, new_content: &str) {
        let mut content = self.content.write().await;
        *content = new_content.to_string();
    }

    /// Insert text at cursor position
    pub async fn insert_text(&self, text: &str) {
        let mut content = self.content.write().await;
        let cursor = self.cursor.read().await;
        
        // Find byte offset from line/column
        let offset = self.position_to_offset(&content, &cursor);
        
        content.insert_str(offset, text);
    }

    /// Delete text in range
    pub async fn delete_range(&self, start: &EditorPosition, end: &EditorPosition) {
        let mut content = self.content.write().await;
        
        let start_offset = self.position_to_offset(&content, start);
        let end_offset = self.position_to_offset(&content, end);
        
        if start_offset < end_offset {
            content.replace_range(start_offset..end_offset, "");
        }
    }

    /// Convert position to byte offset
    fn position_to_offset(&self, content: &str, pos: &EditorPosition) -> usize {
        let mut offset = 0;
        let mut current_line = 0;
        
        for c in content.chars() {
            if current_line == pos.line {
                break;
            }
            offset += c.len_utf8();
            if c == '\n' {
                current_line += 1;
            }
        }
        
        // Add column offset
        let remaining: String = content.chars().skip(offset).take(pos.column).collect();
        offset += remaining.len();
        
        offset
    }

    /// Tokenize current content
    pub async fn tokenize(&self) {
        let content = self.content.read().await;
        let language = self.language.read().await;
        
        let tokens = match *language {
            Language::JavaScript | Language::TypeScript => self.tokenize_js(&content),
            Language::Html => self.tokenize_html(&content),
            Language::Css => self.tokenize_css(&content),
            Language::Json => self.tokenize_json(&content),
            _ => vec![],
        };
        
        let mut t = self.tokens.write().await;
        *t = tokens;
    }

    /// Tokenize JavaScript/TypeScript
    fn tokenize_js(&self, code: &str) -> Vec<SyntaxToken> {
        let mut tokens = Vec::new();
        let keywords = [
            "async", "await", "break", "case", "catch", "class", "const", "continue",
            "debugger", "default", "delete", "do", "else", "export", "extends", "false",
            "finally", "for", "function", "if", "import", "in", "instanceof", "let",
            "new", "null", "return", "static", "super", "switch", "this", "throw",
            "true", "try", "typeof", "undefined", "var", "void", "while", "with", "yield",
        ];
        
        // Simple tokenizer - in real impl would use proper lexer
        let mut current = String::new();
        let mut start = 0;
        let mut in_string = false;
        let mut string_char = ' ';
        let mut in_comment = false;
        
        for (i, c) in code.char_indices() {
            if in_comment {
                if c == '\n' {
                    in_comment = false;
                    if !current.is_empty() {
                        tokens.push(SyntaxToken {
                            token_type: TokenType::Comment,
                            start,
                            end: i,
                            text: current.clone(),
                        });
                        current.clear();
                    }
                }
                current.push(c);
                continue;
            }
            
            if in_string {
                current.push(c);
                if c == string_char && !current.ends_with('\\') {
                    tokens.push(SyntaxToken {
                        token_type: TokenType::String,
                        start,
                        end: i + 1,
                        text: current.clone(),
                    });
                    current.clear();
                    in_string = false;
                }
                continue;
            }
            
            if c == '/' && code.chars().nth(i + 1) == Some('/') {
                if !current.is_empty() {
                    self.add_token(&mut tokens, &current, start, i);
                    current.clear();
                }
                start = i;
                in_comment = true;
                current.push(c);
                continue;
            }
            
            if c == '"' || c == '\'' || c == '`' {
                if !current.is_empty() {
                    self.add_token(&mut tokens, &current, start, i);
                    current.clear();
                }
                start = i;
                in_string = true;
                string_char = c;
                current.push(c);
                continue;
            }
            
            if c.is_whitespace() || "{}()[];,.:=+-*/<>!&|".contains(c) {
                if !current.is_empty() {
                    self.add_token(&mut tokens, &current, start, i);
                    current.clear();
                }
                if !c.is_whitespace() {
                    tokens.push(SyntaxToken {
                        token_type: TokenType::Punctuation,
                        start: i,
                        end: i + c.len_utf8(),
                        text: c.to_string(),
                    });
                }
                start = i + c.len_utf8();
                continue;
            }
            
            if current.is_empty() {
                start = i;
            }
            current.push(c);
        }
        
        tokens
    }

    /// Add token based on content
    fn add_token(&self, tokens: &mut Vec<SyntaxToken>, text: &str, start: usize, end: usize) {
        let token_type = if text.starts_with('"') || text.starts_with('\'') {
            TokenType::String
        } else if text.chars().next().map(|c| c.is_numeric()).unwrap_or(false) {
            TokenType::Number
        } else if [
            "async", "await", "break", "case", "catch", "class", "const", "continue",
            "debugger", "default", "delete", "do", "else", "export", "extends", "false",
            "finally", "for", "function", "if", "import", "in", "instanceof", "let",
            "new", "null", "return", "static", "super", "switch", "this", "throw",
            "true", "try", "typeof", "undefined", "var", "void", "while", "with", "yield",
        ].contains(&text) {
            TokenType::Keyword
        } else {
            TokenType::Variable
        };
        
        tokens.push(SyntaxToken {
            token_type,
            start,
            end,
            text: text.to_string(),
        });
    }

    /// Tokenize HTML
    fn tokenize_html(&self, html: &str) -> Vec<SyntaxToken> {
        let mut tokens = Vec::new();
        // Simplified HTML tokenizer
        let mut in_tag = false;
        let mut in_string = false;
        let mut current = String::new();
        let mut start = 0;
        
        for (i, c) in html.char_indices() {
            if in_string {
                current.push(c);
                if c == '"' {
                    tokens.push(SyntaxToken {
                        token_type: TokenType::String,
                        start,
                        end: i + 1,
                        text: current.clone(),
                    });
                    current.clear();
                    in_string = false;
                }
                continue;
            }
            
            if c == '<' {
                if !current.is_empty() {
                    tokens.push(SyntaxToken {
                        token_type: TokenType::Unknown,
                        start,
                        end: i,
                        text: current.clone(),
                    });
                    current.clear();
                }
                in_tag = true;
                start = i;
                current.push(c);
                continue;
            }
            
            if c == '>' && in_tag {
                current.push(c);
                tokens.push(SyntaxToken {
                    token_type: TokenType::Tag,
                    start,
                    end: i + 1,
                    text: current.clone(),
                });
                current.clear();
                in_tag = false;
                start = i + 1;
                continue;
            }
            
            if c == '"' && in_tag {
                if !current.is_empty() {
                    tokens.push(SyntaxToken {
                        token_type: TokenType::Attribute,
                        start,
                        end: i,
                        text: current.clone(),
                    });
                    current.clear();
                }
                start = i;
                in_string = true;
                current.push(c);
                continue;
            }
            
            if current.is_empty() {
                start = i;
            }
            current.push(c);
        }
        
        tokens
    }

    /// Tokenize CSS
    fn tokenize_css(&self, css: &str) -> Vec<SyntaxToken> {
        // Simplified CSS tokenizer
        let mut tokens = Vec::new();
        let mut current = String::new();
        let mut start = 0;
        
        for (i, c) in css.char_indices() {
            if "{}:;".contains(c) {
                if !current.is_empty() {
                    let token_type = if current.starts_with('.') || current.starts_with('#') {
                        TokenType::Type
                    } else {
                        TokenType::Property
                    };
                    tokens.push(SyntaxToken {
                        token_type,
                        start,
                        end: i,
                        text: current.clone(),
                    });
                    current.clear();
                }
                tokens.push(SyntaxToken {
                    token_type: TokenType::Punctuation,
                    start: i,
                    end: i + 1,
                    text: c.to_string(),
                });
                start = i + 1;
                continue;
            }
            
            if current.is_empty() {
                start = i;
            }
            current.push(c);
        }
        
        tokens
    }

    /// Tokenize JSON
    fn tokenize_json(&self, json: &str) -> Vec<SyntaxToken> {
        let mut tokens = Vec::new();
        let mut in_string = false;
        let mut current = String::new();
        let mut start = 0;
        
        for (i, c) in json.char_indices() {
            if in_string {
                current.push(c);
                if c == '"' && !current.ends_with("\\&quot;") {
                    tokens.push(SyntaxToken {
                        token_type: TokenType::String,
                        start,
                        end: i + 1,
                        text: current.clone(),
                    });
                    current.clear();
                    in_string = false;
                }
                continue;
            }
            
            if c == '"' {
                start = i;
                in_string = true;
                current.push(c);
                continue;
            }
            
            if "{}[]:,".contains(c) {
                if !current.is_empty() {
                    let token_type = if current == "true" || current == "false" || current == "null" {
                        TokenType::Keyword
                    } else if current.chars().next().map(|c| c.is_numeric()).unwrap_or(false) {
                        TokenType::Number
                    } else {
                        TokenType::Unknown
                    };
                    tokens.push(SyntaxToken {
                        token_type,
                        start,
                        end: i,
                        text: current.clone(),
                    });
                    current.clear();
                }
                tokens.push(SyntaxToken {
                    token_type: TokenType::Punctuation,
                    start: i,
                    end: i + 1,
                    text: c.to_string(),
                });
                start = i + 1;
                continue;
            }
            
            if c.is_whitespace() {
                if !current.is_empty() {
                    let token_type = if current == "true" || current == "false" || current == "null" {
                        TokenType::Keyword
                    } else if current.chars().next().map(|c| c.is_numeric()).unwrap_or(false) {
                        TokenType::Number
                    } else {
                        TokenType::Unknown
                    };
                    tokens.push(SyntaxToken {
                        token_type,
                        start,
                        end: i,
                        text: current.clone(),
                    });
                    current.clear();
                }
                start = i + 1;
                continue;
            }
            
            if current.is_empty() {
                start = i;
            }
            current.push(c);
        }
        
        tokens
    }

    /// Get syntax tokens
    pub async fn get_tokens(&self) -> Vec<SyntaxToken> {
        let tokens = self.tokens.read().await;
        tokens.clone()
    }

    /// Get completions at cursor
    pub async fn get_completions(&self, _position: &EditorPosition) -> Vec<CompletionItem> {
        // In real implementation, this would use language server
        let language = self.language.read().await;
        
        match *language {
            Language::JavaScript | Language::TypeScript => {
                vec![
                    CompletionItem {
                        label: "console".to_string(),
                        kind: CompletionKind::Variable,
                        detail: Some("Console API".to_string()),
                        documentation: Some("Debugging console".to_string()),
                        insert_text: "console".to_string(),
                        is_snippet: false,
                        sort_priority: 1,
                    },
                    CompletionItem {
                        label: "document".to_string(),
                        kind: CompletionKind::Variable,
                        detail: Some("Document object".to_string()),
                        documentation: Some("The current document".to_string()),
                        insert_text: "document".to_string(),
                        is_snippet: false,
                        sort_priority: 2,
                    },
                    CompletionItem {
                        label: "querySelector".to_string(),
                        kind: CompletionKind::Method,
                        detail: Some("(selector: string): Element | null".to_string()),
                        documentation: Some("Find first matching element".to_string()),
                        insert_text: "querySelector('$1')".to_string(),
                        is_snippet: true,
                        sort_priority: 3,
                    },
                ]
            }
            _ => vec![],
        }
    }

    /// Find text in document
    pub async fn find(&self, pattern: &str, use_regex: bool, case_sensitive: bool) -> Vec<FindMatch> {
        let content = self.content.read().await;
        let mut matches = Vec::new();
        
        if use_regex {
            // Regex search
            if let Ok(re) = regex::RegexBuilder::new(pattern)
                .case_insensitive(!case_sensitive)
                .build()
            {
                for cap in re.captures_iter(&content) {
                    if let Some(m) = cap.get(0) {
                        let start = self.offset_to_position(&content, m.start());
                        let end = self.offset_to_position(&content, m.end());
                        let groups: Vec<String> = cap.iter().skip(1)
                            .filter_map(|g| g.map(|g| g.as_str().to_string()))
                            .collect();
                        
                        matches.push(FindMatch {
                            start,
                            end,
                            text: m.as_str().to_string(),
                            groups,
                        });
                    }
                }
            }
        } else {
            // Plain text search
            let search = if case_sensitive {
                pattern.to_string()
            } else {
                pattern.to_lowercase()
            };
            let content_lower = if case_sensitive {
                content.clone()
            } else {
                content.to_lowercase()
            };
            
            let mut pos = 0;
            while let Some(idx) = content_lower[pos..].find(&search) {
                let abs_idx = pos + idx;
                let start = self.offset_to_position(&content, abs_idx);
                let end = self.offset_to_position(&content, abs_idx + search.len());
                
                matches.push(FindMatch {
                    start,
                    end,
                    text: content[abs_idx..abs_idx + search.len()].to_string(),
                    groups: vec![],
                });
                
                pos = abs_idx + search.len();
            }
        }
        
        matches
    }

    /// Convert byte offset to position
    fn offset_to_position(&self, content: &str, offset: usize) -> EditorPosition {
        let mut line = 0;
        let mut col = 0;
        let mut current = 0;
        
        for c in content.chars() {
            if current >= offset {
                break;
            }
            
            if c == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
            
            current += c.len_utf8();
        }
        
        EditorPosition { line, column: col }
    }

    /// Replace all matches
    pub async fn replace_all(&self, matches: &[FindMatch], replacement: &str) {
        let mut content = self.content.write().await;
        
        // Replace from end to start to preserve offsets
        for m in matches.iter().rev() {
            let start = self.position_to_offset(&content, &m.start);
            let end = self.position_to_offset(&content, &m.end);
            content.replace_range(start..end, replacement);
        }
    }

    /// Format document
    pub async fn format(&self) {
        let language = self.language.read().await;
        let mut content = self.content.write().await;
        
        match *language {
            Language::Json => {
                // Simple JSON formatting
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) {
                    *content = serde_json::to_string_pretty(&value).unwrap_or(content.clone());
                }
            }
            _ => {
                // Basic formatting: fix indentation
                let lines: Vec<&str> = content.lines().collect();
                let mut formatted = String::new();
                let mut indent = 0;
                
                for line in lines {
                    let trimmed = line.trim();
                    if trimmed.starts_with('}') || trimmed.starts_with(']') || trimmed.starts_with(')') {
                        indent = indent.saturating_sub(1);
                    }
                    
                    formatted.push_str(&" ".repeat(indent * self.config.tab_size));
                    formatted.push_str(trimmed);
                    formatted.push('\n');
                    
                    if trimmed.ends_with('{') || trimmed.ends_with('[') || trimmed.ends_with('(') {
                        indent += 1;
                    }
                }
                
                *content = formatted;
            }
        }
    }

    /// Undo last operation
    pub async fn undo(&self) -> Option<EditOperation> {
        let mut undo_stack = self.undo_stack.write().await;
        undo_stack.pop()
    }

    /// Redo last undone operation
    pub async fn redo(&self) -> Option<EditOperation> {
        let mut redo_stack = self.redo_stack.write().await;
        redo_stack.pop()
    }

    /// Get folding ranges
    pub async fn get_folding_ranges(&self) -> Vec<FoldingRange> {
        let content = self.content.read().await;
        let language = self.language.read().await;
        
        let mut ranges = Vec::new();
        let mut brace_stack: Vec<usize> = Vec::new();
        
        for (line_num, line) in content.lines().enumerate() {
            // Track braces for folding
            for c in line.chars() {
                if c == '{' {
                    brace_stack.push(line_num);
                } else if c == '}' {
                    if let Some(start) = brace_stack.pop() {
                        if start != line_num {
                            let kind = match *language {
                                Language::JavaScript | Language::TypeScript => {
                                    if line.contains("function") || line.contains("=>") {
                                        FoldingKind::Function
                                    } else if line.contains("class") {
                                        FoldingKind::Class
                                    } else {
                                        FoldingKind::Object
                                    }
                                }
                                _ => FoldingKind::Region,
                            };
                            ranges.push(FoldingRange {
                                start_line: start,
                                end_line: line_num,
                                kind,
                            });
                        }
                    }
                }
            }
        }
        
        ranges
    }

    /// Get diagnostics
    pub async fn get_diagnostics(&self) -> Vec<Diagnostic> {
        let diagnostics = self.diagnostics.read().await;
        diagnostics.clone()
    }

    /// Get current cursor position
    pub async fn get_cursor(&self) -> EditorPosition {
        let cursor = self.cursor.read().await;
        cursor.clone()
    }

    /// Set cursor position
    pub async fn set_cursor(&self, position: EditorPosition) {
        let mut cursor = self.cursor.write().await;
        *cursor = position;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_detect_language() {
        assert_eq!(CodeEditor::detect_language("app.js"), Language::JavaScript);
        assert_eq!(CodeEditor::detect_language("app.ts"), Language::TypeScript);
        assert_eq!(CodeEditor::detect_language("index.html"), Language::Html);
        assert_eq!(CodeEditor::detect_language("style.css"), Language::Css);
        assert_eq!(CodeEditor::detect_language("data.json"), Language::Json);
    }

    #[tokio::test]
    async fn test_open_file() {
        let mut editor = CodeEditor::new(EditorConfig::default());
        editor.open_file("test.js", "const x = 1;").await;
        
        assert_eq!(editor.get_content().await, "const x = 1;");
    }

    #[tokio::test]
    async fn test_find() {
        let editor = CodeEditor::new(EditorConfig::default());
        editor.set_content("hello world hello").await;
        
        let matches = editor.find("hello", false, false).await;
        assert_eq!(matches.len(), 2);
    }
}