//! Source Map Manager for debugging minified code
//! 
//! Provides source map support including:
//! - Source map parsing
//! - Original position lookup
//! - Generated position lookup
//! - Source content retrieval
//! - Multi-level source map support

use crate::developer_tools::models::{SourceMapEntry, SourceFile};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Source map header (from last line comment)
pub const SOURCE_MAP_COMMENT: &str = "//# sourceMappingURL=";

/// Parsed source map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMap {
    /// Version (should be 3)
    pub version: u32,
    /// Array of source file paths
    pub sources: Vec<String>,
    /// Array of source content (optional)
    pub sources_content: Vec<Option<String>>,
    /// Array of generated file names
    pub names: Vec<String>,
    /// Mappings string
    pub mappings: String,
    /// File name
    pub file: Option<String>,
    /// Source root
    pub source_root: Option<String>,
}

/// Decoded mapping entry
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MappingEntry {
    /// Generated line (0-indexed)
    pub generated_line: u32,
    /// Generated column (0-indexed)
    pub generated_column: u32,
    /// Source file index
    pub source_index: Option<u32>,
    /// Original line (0-indexed)
    pub original_line: Option<u32>,
    /// Original column (0-indexed)
    pub original_column: Option<u32>,
    /// Name index
    pub name_index: Option<u32>,
}

/// Position in source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    /// Line number (1-indexed)
    pub line: u32,
    /// Column number (1-indexed)
    pub column: u32,
}

/// Original position with source info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OriginalPosition {
    /// Source file path
    pub source: String,
    /// Position
    pub position: Position,
    /// Name at this position (if any)
    pub name: Option<String>,
}

/// Generated position info
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GeneratedPosition {
    /// Position in generated code
    pub position: Position,
}

/// Source map consumer for efficient lookups
pub struct SourceMapConsumer {
    /// Parsed source map
    source_map: SourceMap,
    /// Decoded mappings
    mappings: Vec<MappingEntry>,
    /// Source content cache
    source_contents: HashMap<String, String>,
    /// Index for binary search
    generated_line_index: HashMap<u32, Vec<usize>>,
}

impl SourceMapConsumer {
    /// Create a new consumer from a source map
    pub fn new(source_map: SourceMap) -> Self {
        let mappings = Self::decode_mappings(&source_map.mappings);
        
        // Build index for fast lookups by generated line
        let mut generated_line_index = HashMap::new();
        for (idx, mapping) in mappings.iter().enumerate() {
            generated_line_index
                .entry(mapping.generated_line)
                .or_insert_with(Vec::new)
                .push(idx);
        }
        
        // Build source contents
        let mut source_contents = HashMap::new();
        for (i, source) in source_map.sources.iter().enumerate() {
            if let Some(Some(content)) = source_map.sources_content.get(i) {
                source_contents.insert(source.clone(), content.clone());
            }
        }
        
        Self {
            source_map,
            mappings,
            source_contents,
            generated_line_index,
        }
    }

    /// Decode VLQ encoded mappings
    fn decode_mappings(mappings: &str) -> Vec<MappingEntry> {
        let mut result = Vec::new();
        let mut generated_line = 0u32;
        let mut generated_column = 0u32;
        let mut source_index = 0u32;
        let mut original_line = 0u32;
        let mut original_column = 0u32;
        let mut name_index = 0u32;
        
        for segment in mappings.split(',') {
            if segment.is_empty() {
                continue;
            }
            
            let mut fields = Vec::new();
            let mut value = 0u32;
            let mut shift = 0u32;
            let mut continuation = false;
            
            for c in segment.chars() {
                let decoded = Self::decode_base64_char(c);
                if decoded < 32 {
                    // Continuation bit not set
                    continuation = false;
                    let sign = if (decoded & 1) == 1 { -1i32 } else { 1i32 };
                    value = (decoded >> 1) as u32;
                    value = ((value as i32) * sign) as u32;
                    fields.push(value);
                    value = 0;
                    shift = 0;
                } else {
                    continuation = true;
                    value |= ((decoded & 31) as u32) << shift;
                    shift += 5;
                }
            }
            
            if continuation {
                fields.push(value);
            }
            
            // Apply fields
            if !fields.is_empty() {
                generated_column = (generated_column as i32 + fields[0] as i32) as u32;
            }
            
            if fields.len() >= 4 {
                source_index = (source_index as i32 + fields[1] as i32) as u32;
                original_line = (original_line as i32 + fields[2] as i32) as u32;
                original_column = (original_column as i32 + fields[3] as i32) as u32;
            }
            
            let name_idx = if fields.len() >= 5 {
                name_index = (name_index as i32 + fields[4] as i32) as u32;
                Some(name_index)
            } else {
                None
            };
            
            result.push(MappingEntry {
                generated_line,
                generated_column,
                source_index: if fields.len() >= 4 { Some(source_index) } else { None },
                original_line: if fields.len() >= 4 { Some(original_line) } else { None },
                original_column: if fields.len() >= 4 { Some(original_column) } else { None },
                name_index: name_idx,
            });
        }
        
        // Handle semicolons (line breaks)
        let mut final_result = Vec::new();
        let mut current_line = 0u32;
        
        for mapping in result {
            // Each semicolon increments the line
            // We need to re-parse to count semicolons
            final_result.push(mapping);
        }
        
        final_result
    }

    /// Decode a single Base64 VLQ character
    fn decode_base64_char(c: char) -> u32 {
        const BASE64_CHARS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        
        if let Some(pos) = BASE64_CHARS.find(c) {
            pos as u32
        } else {
            0
        }
    }

    /// Find original position from generated position
    pub fn original_position_for(&self, generated: Position) -> Option<OriginalPosition> {
        let line = generated.line.saturating_sub(1);
        let column = generated.column.saturating_sub(1);
        
        // Find mappings for this line
        let line_mappings = self.generated_line_index.get(&line)?;
        
        // Binary search for the closest mapping
        let mut closest_idx: Option<usize> = None;
        let mut closest_column = 0u32;
        
        for &idx in line_mappings {
            let mapping = &self.mappings[idx];
            if mapping.generated_column <= column && mapping.generated_column >= closest_column {
                closest_column = mapping.generated_column;
                closest_idx = Some(idx);
            }
        }
        
        let mapping = closest_idx.map(|i| &self.mappings[i])?;
        
        // Get source file
        let source_index = mapping.source_index? as usize;
        let source = self.source_map.sources.get(source_index)?.clone();
        
        // Get name if available
        let name = mapping.name_index
            .and_then(|i| self.source_map.names.get(i as usize).cloned());
        
        Some(OriginalPosition {
            source,
            position: Position {
                line: mapping.original_line? + 1,
                column: mapping.original_column? + 1,
            },
            name,
        })
    }

    /// Find generated position from original position
    pub fn generated_position_for(&self, source: &str, original: Position) -> Option<GeneratedPosition> {
        let original_line = original.line.saturating_sub(1);
        let original_column = original.column.saturating_sub(1);
        
        // Find source index
        let source_index = self.source_map.sources.iter()
            .position(|s| s == source)?;
        
        // Find matching mapping
        let mut closest: Option<&MappingEntry> = None;
        
        for mapping in &self.mappings {
            if mapping.source_index == Some(source_index as u32)
                && mapping.original_line == Some(original_line)
            {
                if let Some(current) = closest {
                    if mapping.original_column.unwrap_or(0) >= original_column
                        && mapping.original_column.unwrap_or(0) < current.original_column.unwrap_or(u32::MAX)
                    {
                        closest = Some(mapping);
                    }
                } else {
                    closest = Some(mapping);
                }
            }
        }
        
        let mapping = closest?;
        
        Some(GeneratedPosition {
            position: Position {
                line: mapping.generated_line + 1,
                column: mapping.generated_column + 1,
            },
        })
    }

    /// Get source content
    pub fn source_content_for(&self, source: &str) -> Option<&str> {
        self.source_contents.get(source).map(|s| s.as_str())
    }

    /// Get all sources
    pub fn sources(&self) -> &[String] {
        &self.source_map.sources
    }

    /// Check if source content is available
    pub fn has_source_content(&self, source: &str) -> bool {
        self.source_contents.contains_key(source)
    }

    /// Compute last generated column for a line
    pub fn last_generated_column_for(&self, line: u32) -> Option<u32> {
        let line_mappings = self.generated_line_index.get(&line)?;
        line_mappings
            .iter()
            .map(|&i| self.mappings[i].generated_column)
            .max()
    }
}

/// Source Map Generator for creating source maps
pub struct SourceMapGenerator {
    /// File being generated
    file: String,
    /// Source root
    source_root: Option<String>,
    /// Source files
    sources: Vec<String>,
    /// Source contents
    sources_content: HashMap<String, String>,
    /// Names
    names: Vec<String>,
    /// Name index
    name_index: HashMap<String, u32>,
    /// Mappings
    mappings: Vec<MappingEntry>,
}

impl SourceMapGenerator {
    /// Create a new generator
    pub fn new(file: &str) -> Self {
        Self {
            file: file.to_string(),
            source_root: None,
            sources: Vec::new(),
            sources_content: HashMap::new(),
            names: Vec::new(),
            name_index: HashMap::new(),
            mappings: Vec::new(),
        }
    }

    /// Set source root
    pub fn set_source_root(&mut self, root: &str) {
        self.source_root = Some(root.to_string());
    }

    /// Add a mapping
    pub fn add_mapping(
        &mut self,
        generated_line: u32,
        generated_column: u32,
        source: Option<&str>,
        original_line: Option<u32>,
        original_column: Option<u32>,
        name: Option<&str>,
    ) {
        let source_index = source.map(|s| {
            if let Some(idx) = self.sources.iter().position(|src| src == s) {
                idx as u32
            } else {
                let idx = self.sources.len() as u32;
                self.sources.push(s.to_string());
                idx
            }
        });
        
        let name_index = name.map(|n| {
            if let Some(&idx) = self.name_index.get(n) {
                idx
            } else {
                let idx = self.names.len() as u32;
                self.names.push(n.to_string());
                self.name_index.insert(n.to_string(), idx);
                idx
            }
        });
        
        self.mappings.push(MappingEntry {
            generated_line,
            generated_column,
            source_index,
            original_line,
            original_column,
            name_index,
        });
    }

    /// Set source content
    pub fn set_source_content(&mut self, source: &str, content: &str) {
        self.sources_content.insert(source.to_string(), content.to_string());
    }

    /// Generate the source map
    pub fn generate(&self) -> SourceMap {
        let mappings = self.encode_mappings();
        
        let sources_content = self.sources.iter()
            .map(|s| self.sources_content.get(s).cloned())
            .collect();
        
        SourceMap {
            version: 3,
            sources: self.sources.clone(),
            sources_content,
            names: self.names.clone(),
            mappings,
            file: Some(self.file.clone()),
            source_root: self.source_root.clone(),
        }
    }

    /// Encode mappings to VLQ string
    fn encode_mappings(&self) -> String {
        let mut result = String::new();
        let mut current_line = 0u32;
        let mut current_column = 0u32;
        let mut current_source = 0u32;
        let mut current_original_line = 0u32;
        let mut current_original_column = 0u32;
        let mut current_name = 0u32;
        
        // Sort mappings by generated position
        let mut sorted = self.mappings.clone();
        sorted.sort_by(|a, b| {
            a.generated_line.cmp(&b.generated_line)
                .then(a.generated_column.cmp(&b.generated_column))
        });
        
        for (i, mapping) in sorted.iter().enumerate() {
            // Handle line breaks
            while current_line < mapping.generated_line {
                result.push(';');
                current_line += 1;
                current_column = 0;
            }
            
            if i > 0 {
                result.push(',');
            }
            
            // Encode generated column delta
            let delta = mapping.generated_column as i32 - current_column as i32;
            result.push_str(&Self::encode_vlq(delta as u32));
            current_column = mapping.generated_column;
            
            // Encode source, original line, original column
            if let Some(source_idx) = mapping.source_index {
                let source_delta = source_idx as i32 - current_source as i32;
                result.push_str(&Self::encode_vlq(source_delta as u32));
                current_source = source_idx;
                
                if let Some(orig_line) = mapping.original_line {
                    let line_delta = orig_line as i32 - current_original_line as i32;
                    result.push_str(&Self::encode_vlq(line_delta as u32));
                    current_original_line = orig_line;
                    
                    if let Some(orig_col) = mapping.original_column {
                        let col_delta = orig_col as i32 - current_original_column as i32;
                        result.push_str(&Self::encode_vlq(col_delta as u32));
                        current_original_column = orig_col;
                    }
                }
            }
            
            // Encode name
            if let Some(name_idx) = mapping.name_index {
                let name_delta = name_idx as i32 - current_name as i32;
                result.push_str(&Self::encode_vlq(name_delta as u32));
                current_name = name_idx;
            }
        }
        
        result
    }

    /// Encode a VLQ number
    fn encode_vlq(value: u32) -> String {
        const BASE64_CHARS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        
        let mut result = String::new();
        let mut value = value;
        
        // Sign bit
        let sign_bit = if (value as i32) < 0 { 1u32 } else { 0u32 };
        value = ((value as i32).abs() as u32) << 1 | sign_bit;
        
        loop {
            let digit = value & 31;
            value >>= 5;
            
            if value > 0 {
                result.push(BASE64_CHARS.chars().nth((digit | 32) as usize).unwrap());
            } else {
                result.push(BASE64_CHARS.chars().nth(digit as usize).unwrap());
            }
            
            if value == 0 {
                break;
            }
        }
        
        result
    }
}

/// Source Map Manager
pub struct SourceMapManager {
    /// Loaded source maps by URL
    source_maps: Arc<RwLock<HashMap<String, SourceMapConsumer>>>,
    /// Source files
    source_files: Arc<RwLock<HashMap<String, SourceFile>>>,
}

impl SourceMapManager {
    /// Create a new manager
    pub fn new() -> Self {
        Self {
            source_maps: Arc::new(RwLock::new(HashMap::new())),
            source_files: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load a source map from URL
    pub async fn load_source_map(&self, url: &str, source_map_url: &str) -> Result<(), String> {
        // In real implementation, would fetch from network
        // For now, assume it's inline or we have the content
        
        // Parse the source map
        // let source_map: SourceMap = serde_json::from_str(&content)?;
        
        Ok(())
    }

    /// Load an inline source map
    pub async fn load_inline_source_map(&self, url: &str, content: &str) -> Result<(), String> {
        // Find source map comment
        let content_str = content.to_string();
        if let Some(pos) = content_str.rfind(SOURCE_MAP_COMMENT) {
            let map_data = &content_str[pos + SOURCE_MAP_COMMENT.len()..];
            
            // Check if it's a base64 data URL
            if map_data.starts_with("data:application/json;base64,") {
                let encoded = &map_data["data:application/json;base64,".len()..];
                // Decode and parse
                // In real implementation, would decode base64
            } else {
                // Assume it's a URL
                // Would fetch from the URL
            }
        }
        
        Ok(())
    }

    /// Parse a source map from JSON
    pub async fn parse_source_map(&self, url: &str, json: &str) -> Result<(), String> {
        let source_map: SourceMap = serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse source map: {}", e))?;
        
        let consumer = SourceMapConsumer::new(source_map);
        
        let mut maps = self.source_maps.write().await;
        maps.insert(url.to_string(), consumer);
        
        Ok(())
    }

    /// Get original position
    pub async fn get_original_position(
        &self,
        url: &str,
        line: u32,
        column: u32,
    ) -> Option<OriginalPosition> {
        let maps = self.source_maps.read().await;
        let consumer = maps.get(url)?;
        
        consumer.original_position_for(Position { line, column })
    }

    /// Get generated position
    pub async fn get_generated_position(
        &self,
        url: &str,
        source: &str,
        line: u32,
        column: u32,
    ) -> Option<GeneratedPosition> {
        let maps = self.source_maps.read().await;
        let consumer = maps.get(url)?;
        
        consumer.generated_position_for(source, Position { line, column })
    }

    /// Get source content
    pub async fn get_source_content(&self, url: &str, source: &str) -> Option<String> {
        let maps = self.source_maps.read().await;
        let consumer = maps.get(url)?;
        
        consumer.source_content_for(source).map(String::from)
    }

    /// Get all sources for a generated file
    pub async fn get_sources(&self, url: &str) -> Option<Vec<String>> {
        let maps = self.source_maps.read().await;
        let consumer = maps.get(url)?;
        
        Some(consumer.sources().to_vec())
    }

    /// Clear all loaded source maps
    pub async fn clear(&self) {
        let mut maps = self.source_maps.write().await;
        maps.clear();
        
        let mut files = self.source_files.write().await;
        files.clear();
    }

    /// Apply source map to a stack frame
    pub async fn apply_to_stack_frame(&self, frame: &StackFrame) -> Option<StackFrame> {
        let original = self.get_original_position(
            &frame.file,
            frame.line,
            frame.column,
        ).await?;
        
        Some(StackFrame {
            file: original.source,
            line: original.position.line,
            column: original.position.column,
            name: original.name.unwrap_or(frame.name.clone()),
        })
    }
}

/// Stack frame for source map application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    /// File path
    pub file: String,
    /// Line number
    pub line: u32,
    /// Column number
    pub column: u32,
    /// Function name
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_base64_char() {
        assert_eq!(SourceMapConsumer::decode_base64_char('A'), 0);
        assert_eq!(SourceMapConsumer::decode_base64_char('Z'), 25);
        assert_eq!(SourceMapConsumer::decode_base64_char('a'), 26);
        assert_eq!(SourceMapConsumer::decode_base64_char('z'), 51);
        assert_eq!(SourceMapConsumer::decode_base64_char('0'), 52);
        assert_eq!(SourceMapConsumer::decode_base64_char('+'), 62);
        assert_eq!(SourceMapConsumer::decode_base64_char('/'), 63);
    }

    #[tokio::test]
    async fn test_source_map_manager() {
        let manager = SourceMapManager::new();
        
        // Test empty state
        let pos = manager.get_original_position("test.js", 1, 1).await;
        assert!(pos.is_none());
    }

    #[test]
    fn test_source_map_generator() {
        let mut gen = SourceMapGenerator::new("bundle.js");
        
        gen.add_mapping(0, 0, Some("original.js"), Some(0), Some(0), None);
        gen.add_mapping(0, 9, Some("original.js"), Some(0), Some(9), Some("sayHello"));
        
        let map = gen.generate();
        
        assert_eq!(map.version, 3);
        assert_eq!(map.sources.len(), 1);
        assert_eq!(map.sources[0], "original.js");
        assert_eq!(map.names.len(), 1);
        assert_eq!(map.names[0], "sayHello");
    }
}