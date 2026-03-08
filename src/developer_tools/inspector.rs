//! Advanced DOM/CSS Inspector
//! 
//! Provides advanced inspection capabilities for DOM elements and CSS styles.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use anyhow::Result;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

use super::{
    DevToolsEvent, DOMNode, NodeType, DOMAttribute, 
    ComputedStyle, BoundingBox, CSSRule, CSSProperty,
};

/// Advanced inspector for DOM and CSS
pub struct AdvancedInspector {
    /// DOM tree cache
    dom_tree: RwLock<HashMap<u64, DOMNode>>,
    /// Root node ID
    root_node_id: RwLock<Option<u64>>,
    /// Selected node ID
    selected_node: RwLock<Option<u64>>,
    /// CSS rules cache
    css_rules: RwLock<Vec<CSSRule>>,
    /// Event sender
    event_sender: broadcast::Sender<DevToolsEvent>,
    /// Hover highlights
    highlights: RwLock<Vec<Highlight>>,
}

/// Element highlight configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Highlight {
    /// Highlight ID
    pub id: String,
    /// Node ID
    pub node_id: u64,
    /// Highlight type
    pub highlight_type: HighlightType,
    /// Color
    pub color: String,
    /// Show content
    pub show_content: bool,
    /// Show padding
    pub show_padding: bool,
    /// Show border
    pub show_border: bool,
    /// Show margin
    pub show_margin: bool,
}

/// Highlight types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HighlightType {
    Element,
    Flex,
    Grid,
    Transform,
    Gap,
    LineHeight,
    Contain,
}

/// Inspector configuration
#[derive(Debug, Clone)]
pub struct InspectorConfig {
    /// Enable live preview
    pub live_preview: bool,
    /// Show user agent styles
    pub show_user_agent_styles: bool,
    /// Show computed styles
    pub show_computed_styles: bool,
    /// Highlight on hover
    pub highlight_on_hover: bool,
    /// Auto-expand depth
    pub auto_expand_depth: u32,
}

impl Default for InspectorConfig {
    fn default() -> Self {
        Self {
            live_preview: true,
            show_user_agent_styles: false,
            show_computed_styles: true,
            highlight_on_hover: true,
            auto_expand_depth: 3,
        }
    }
}

impl AdvancedInspector {
    /// Create a new inspector
    pub async fn new(event_sender: broadcast::Sender<DevToolsEvent>) -> Result<Self> {
        Ok(Self {
            dom_tree: RwLock::new(HashMap::new()),
            root_node_id: RwLock::new(None),
            selected_node: RwLock::new(None),
            css_rules: RwLock::new(Vec::new()),
            event_sender,
            highlights: RwLock::new(Vec::new()),
        })
    }
    
    /// Set DOM tree
    pub async fn set_dom_tree(&self, nodes: Vec<DOMNode>) -> Result<()> {
        let mut tree = self.dom_tree.write().await;
        
        for node in nodes {
            if node.parent_id.is_none() {
                let mut root_id = self.root_node_id.write().await;
                *root_id = Some(node.id);
            }
            tree.insert(node.id, node);
        }
        
        Ok(())
    }
    
    /// Get DOM tree
    pub async fn get_dom_tree(&self) -> Result<Vec<DOMNode>> {
        let tree = self.dom_tree.read().await;
        Ok(tree.values().cloned().collect())
    }
    
    /// Get node by ID
    pub async fn get_node(&self, node_id: u64) -> Result<Option<DOMNode>> {
        let tree = self.dom_tree.read().await;
        Ok(tree.get(&node_id).cloned())
    }
    
    /// Select node
    pub async fn select_node(&self, node_id: u64) -> Result<()> {
        let mut selected = self.selected_node.write().await;
        *selected = Some(node_id);
        
        // Emit event
        let _ = self.event_sender.send(DevToolsEvent::ElementInspected(
            node_id.to_string()
        ));
        
        tracing::debug!("Node selected: {}", node_id);
        Ok(())
    }
    
    /// Get selected node
    pub async fn get_selected_node(&self) -> Result<Option<DOMNode>> {
        let selected = self.selected_node.read().await;
        
        if let Some(id) = *selected {
            let tree = self.dom_tree.read().await;
            return Ok(tree.get(&id).cloned());
        }
        
        Ok(None)
    }
    
    /// Deselect node
    pub async fn deselect_node(&self) -> Result<()> {
        let mut selected = self.selected_node.write().await;
        *selected = None;
        Ok(())
    }
    
    /// Search nodes by selector
    pub async fn search_nodes(&self, selector: &str) -> Result<Vec<DOMNode>> {
        let tree = self.dom_tree.read().await;
        let mut results = Vec::new();
        
        // Simple selector matching (tag name, class, id)
        for node in tree.values() {
            if self.matches_selector(node, selector) {
                results.push(node.clone());
            }
        }
        
        Ok(results)
    }
    
    /// Check if node matches selector
    fn matches_selector(&self, node: &DOMNode, selector: &str) -> bool {
        let selector = selector.trim();
        
        // ID selector
        if selector.starts_with('#') {
            let id = &selector[1..];
            return node.attributes.iter().any(|a| a.name == "id" && a.value == id);
        }
        
        // Class selector
        if selector.starts_with('.') {
            let class = &selector[1..];
            return node.attributes.iter().any(|a| {
                a.name == "class" && a.value.split_whitespace().any(|c| c == class)
            });
        }
        
        // Tag selector
        if let Some(tag) = &node.tag_name {
            if tag.eq_ignore_ascii_case(selector) {
                return true;
            }
        }
        
        false
    }
    
    /// Get node path (XPath-like)
    pub async fn get_node_path(&self, node_id: u64) -> Result<String> {
        let tree = self.dom_tree.read().await;
        let mut path = Vec::new();
        let mut current_id = Some(node_id);
        
        while let Some(id) = current_id {
            if let Some(node) = tree.get(&id) {
                path.push(node.tag_name.clone().unwrap_or_else(|| "node".to_string()));
                current_id = node.parent_id;
            } else {
                break;
            }
        }
        
        path.reverse();
        Ok(path.join(" > "))
    }
    
    /// Get node children
    pub async fn get_children(&self, node_id: u64) -> Result<Vec<DOMNode>> {
        let tree = self.dom_tree.read().await;
        
        if let Some(node) = tree.get(&node_id) {
            let children: Vec<DOMNode> = node.children.iter()
                .filter_map(|id| tree.get(id).cloned())
                .collect();
            return Ok(children);
        }
        
        Ok(Vec::new())
    }
    
    /// Modify node attribute
    pub async fn modify_attribute(
        &self,
        node_id: u64,
        name: &str,
        value: &str,
    ) -> Result<()> {
        let mut tree = self.dom_tree.write().await;
        
        if let Some(node) = tree.get_mut(&node_id) {
            // Update or add attribute
            if let Some(attr) = node.attributes.iter_mut().find(|a| a.name == name) {
                attr.value = value.to_string();
            } else {
                node.attributes.push(DOMAttribute {
                    name: name.to_string(),
                    value: value.to_string(),
                    is_event_handler: name.starts_with("on"),
                });
            }
        }
        
        Ok(())
    }
    
    /// Remove node attribute
    pub async fn remove_attribute(&self, node_id: u64, name: &str) -> Result<()> {
        let mut tree = self.dom_tree.write().await;
        
        if let Some(node) = tree.get_mut(&node_id) {
            node.attributes.retain(|a| a.name != name);
        }
        
        Ok(())
    }
    
    /// Add highlight to element
    pub async fn add_highlight(
        &self,
        node_id: u64,
        highlight_type: HighlightType,
        color: String,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        
        let highlight = Highlight {
            id: id.clone(),
            node_id,
            highlight_type,
            color,
            show_content: true,
            show_padding: true,
            show_border: true,
            show_margin: true,
        };
        
        let mut highlights = self.highlights.write().await;
        highlights.push(highlight);
        
        Ok(id)
    }
    
    /// Remove highlight
    pub async fn remove_highlight(&self, highlight_id: &str) -> Result<()> {
        let mut highlights = self.highlights.write().await;
        highlights.retain(|h| h.id != highlight_id);
        Ok(())
    }
    
    /// Clear all highlights
    pub async fn clear_highlights(&self) -> Result<()> {
        let mut highlights = self.highlights.write().await;
        highlights.clear();
        Ok(())
    }
    
    /// Set CSS rules
    pub async fn set_css_rules(&self, rules: Vec<CSSRule>) -> Result<()> {
        let mut css_rules = self.css_rules.write().await;
        *css_rules = rules;
        Ok(())
    }
    
    /// Get CSS rules
    pub async fn get_css_rules(&self) -> Result<Vec<CSSRule>> {
        let css_rules = self.css_rules.read().await;
        Ok(css_rules.clone())
    }
    
    /// Get matching CSS rules for node
    pub async fn get_matching_rules(&self, node_id: u64) -> Result<Vec<CSSRule>> {
        let tree = self.dom_tree.read().await;
        let css_rules = self.css_rules.read().await;
        
        if let Some(node) = tree.get(&node_id) {
            let mut matching = Vec::new();
            
            for rule in css_rules.iter() {
                if let Some(selector) = &rule.selector {
                    if self.matches_selector(node, selector) {
                        matching.push(rule.clone());
                    }
                }
            }
            
            return Ok(matching);
        }
        
        Ok(Vec::new())
    }
    
    /// Modify CSS property
    pub async fn modify_css_property(
        &self,
        rule_id: &str,
        property_name: &str,
        property_value: &str,
    ) -> Result<()> {
        let mut css_rules = self.css_rules.write().await;
        
        if let Some(rule) = css_rules.iter_mut().find(|r| r.id == rule_id) {
            if let Some(prop) = rule.properties.iter_mut().find(|p| p.name == property_name) {
                prop.value = property_value.to_string();
            } else {
                rule.properties.push(CSSProperty {
                    name: property_name.to_string(),
                    value: property_value.to_string(),
                    important: false,
                    disabled: false,
                    parsed_value: None,
                });
            }
        }
        
        Ok(())
    }
    
    /// Capture DOM snapshot
    pub async fn capture_dom(&self) -> Result<String> {
        let tree = self.dom_tree.read().await;
        let root_id = self.root_node_id.read().await;
        
        if let Some(root_id) = *root_id {
            if let Some(root) = tree.get(&root_id) {
                return Ok(self.node_to_html(root, &tree));
            }
        }
        
        Ok(String::new())
    }
    
    /// Convert node to HTML string
    fn node_to_html(&self, node: &DOMNode, tree: &HashMap<u64, DOMNode>) -> String {
        let mut html = String::new();
        
        match node.node_type {
            NodeType::Element => {
                html.push_str(&format!("<{}", node.tag_name.as_deref().unwrap_or("div")));
                
                for attr in &node.attributes {
                    html.push_str(&format!(" {}=&quot;{}&quot;", attr.name, attr.value));
                }
                
                html.push('>');
                
                for child_id in &node.children {
                    if let Some(child) = tree.get(child_id) {
                        html.push_str(&self.node_to_html(child, tree));
                    }
                }
                
                html.push_str(&format!("</{}>", node.tag_name.as_deref().unwrap_or("div")));
            }
            NodeType::Text => {
                html.push_str(&node.node_value.clone().unwrap_or_default());
            }
            NodeType::Comment => {
                html.push_str(&format!("<!--{}-->", node.node_value.clone().unwrap_or_default()));
            }
            _ => {}
        }
        
        html
    }
    
    /// Capture styles
    pub async fn capture_styles(&self) -> Result<Vec<String>> {
        let css_rules = self.css_rules.read().await;
        
        let styles: Vec<String> = css_rules.iter().map(|rule| {
            let props: Vec<String> = rule.properties.iter()
                .map(|p| format!("  {}: {};", p.name, p.value))
                .collect();
            
            format!("{} {{\n{}\n}}", rule.selector.as_deref().unwrap_or("*"), props.join("\n"))
        }).collect();
        
        Ok(styles)
    }
    
    /// Get box model
    pub async fn get_box_model(&self, node_id: u64) -> Result<Option<BoxModel>> {
        let tree = self.dom_tree.read().await;
        
        if let Some(node) = tree.get(&node_id) {
            if let Some(bbox) = &node.bounding_box {
                return Ok(Some(BoxModel {
                    content: BoxDimensions {
                        x: bbox.x,
                        y: bbox.y,
                        width: bbox.width,
                        height: bbox.height,
                    },
                    padding: BoxDimensions {
                        x: bbox.x,
                        y: bbox.y,
                        width: bbox.width,
                        height: bbox.height,
                    },
                    border: BoxDimensions {
                        x: bbox.x,
                        y: bbox.y,
                        width: bbox.width,
                        height: bbox.height,
                    },
                    margin: BoxDimensions {
                        x: bbox.x,
                        y: bbox.y,
                        width: bbox.width,
                        height: bbox.height,
                    },
                }));
            }
        }
        
        Ok(None)
    }
}

/// Box model representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxModel {
    /// Content box
    pub content: BoxDimensions,
    /// Padding box
    pub padding: BoxDimensions,
    /// Border box
    pub border: BoxDimensions,
    /// Margin box
    pub margin: BoxDimensions,
}

/// Box dimensions
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoxDimensions {
    /// X position
    pub x: f64,
    /// Y position
    pub y: f64,
    /// Width
    pub width: f64,
    /// Height
    pub height: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_inspector_creation() {
        let (tx, _rx) = broadcast::channel(16);
        let inspector = AdvancedInspector::new(tx).await.unwrap();
        
        let tree = inspector.get_dom_tree().await.unwrap();
        assert!(tree.is_empty());
    }
    
    #[tokio::test]
    async fn test_node_selection() {
        let (tx, _rx) = broadcast::channel(16);
        let inspector = AdvancedInspector::new(tx).await.unwrap();
        
        let node = DOMNode {
            id: 1,
            node_type: NodeType::Element,
            tag_name: Some("div".to_string()),
            node_value: None,
            attributes: vec![],
            children: vec![],
            parent_id: None,
            computed_styles: vec![],
            bounding_box: None,
        };
        
        inspector.set_dom_tree(vec![node]).await.unwrap();
        inspector.select_node(1).await.unwrap();
        
        let selected = inspector.get_selected_node().await.unwrap();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().id, 1);
    }
}