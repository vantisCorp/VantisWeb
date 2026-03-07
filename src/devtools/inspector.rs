// Copyright 2024 Vantis Corporation - All Rights Reserved

//! Elements Inspector Module
//! 
//! This module provides DOM tree inspection, CSS property editing,
//! computed styles view, box model visualization, and selector generation.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// DOM element information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DOMElement {
    /// Element ID
    pub id: Uuid,
    /// Tag name
    pub tag_name: String,
    /// Element ID attribute
    pub element_id: Option<String>,
    /// Classes
    pub classes: Vec<String>,
    /// Attributes
    pub attributes: HashMap<String, String>,
    /// Computed styles
    pub computed_styles: HashMap<String, String>,
    /// Children
    pub children: Vec<Uuid>,
    /// Parent
    pub parent: Option<Uuid>,
    /// Text content
    pub text_content: Option<String>,
    /// Inner HTML
    pub inner_html: Option<String>,
    /// Box model
    pub box_model: Option<BoxModel>,
    /// Pseudo-elements
    pub pseudo_elements: Vec<PseudoElement>,
    /// Event listeners
    pub event_listeners: Vec<EventListener>,
}

/// Box model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxModel {
    /// Content box
    pub content: BoxRect,
    /// Padding box
    pub padding: BoxRect,
    /// Border box
    pub border: BoxRect,
    /// Margin box
    pub margin: BoxRect,
}

/// Box rectangle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxRect {
    /// Width
    pub width: f64,
    /// Height
    pub height: f64,
    /// Top
    pub top: f64,
    /// Left
    pub left: f64,
}

/// Pseudo-element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PseudoElement {
    /// Pseudo-element type (::before, ::after, etc.)
    pub pseudo_type: String,
    /// Content
    pub content: Option<String>,
    /// Styles
    pub styles: HashMap<String, String>,
}

/// Event listener
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventListener {
    /// Event type
    pub event_type: String,
    /// Function name
    pub function_name: Option<String>,
    /// Source URL
    pub source_url: Option<String>,
    /// Line number
    pub line_number: Option<u32>,
}

/// CSS property
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CSSProperty {
    /// Property name
    pub name: String,
    /// Property value
    pub value: String,
    /// Important flag
    pub important: bool,
    /// Source
    pub source: CSSPropertySource,
    /// Inherited
    pub inherited: bool,
    /// Computed value
    pub computed_value: String,
}

/// CSS property source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CSSPropertySource {
    /// User agent stylesheet
    UserAgent,
    /// Author stylesheet
    Author { url: String, line: u32 },
    /// Inline style
    Inline,
    /// Element style
    Element,
}

/// CSS rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CSSRule {
    /// Selector
    pub selector: String,
    /// Properties
    pub properties: Vec<CSSProperty>,
    /// Specificity
    pub specificity: u32,
    /// Media query
    pub media_query: Option<String>,
}

/// Computed styles result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputedStyles {
    /// Element
    pub element_id: Uuid,
    /// Properties
    pub properties: HashMap<String, CSSProperty>,
    /// Inherited properties
    pub inherited_from: Option<Uuid>,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Element
    pub element_id: Uuid,
    /// Match type
    pub match_type: SearchMatchType,
    /// Matched text
    pub matched_text: String,
    /// Context
    pub context: String,
}

/// Search match type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchMatchType {
    /// Tag name match
    TagName,
    /// ID match
    Id,
    /// Class match
    Class,
    /// Attribute match
    Attribute,
    /// Text content match
    Text,
}

/// Inspector selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectorSelection {
    /// Selected element
    pub element_id: Uuid,
    /// Selected at
    pub selected_at: DateTime<Utc>,
    /// Highlighted
    pub highlighted: bool,
}

/// Elements inspector
pub struct ElementsInspector {
    /// DOM tree
    dom_tree: Arc<RwLock<HashMap<Uuid, DOMElement>>>,
    /// Root element
    root_element: Arc<RwLock<Option<Uuid>>>,
    /// Selection
    selection: Arc<RwLock<Option<InspectorSelection>>>,
    /// Pinned elements
    pinned_elements: Arc<RwLock<HashSet<Uuid>>>,
    /// CSS rules
    css_rules: Arc<RwLock<Vec<CSSRule>>>,
    /// Search results
    search_results: Arc<RwLock<Vec<SearchResult>>>,
}

impl ElementsInspector {
    /// Create a new elements inspector
    pub fn new() -> Self {
        Self {
            dom_tree: Arc::new(RwLock::new(HashMap::new())),
            root_element: Arc::new(RwLock::new(None)),
            selection: Arc::new(RwLock::new(None)),
            pinned_elements: Arc::new(RwLock::new(HashSet::new())),
            css_rules: Arc::new(RwLock::new(Vec::new())),
            search_results: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Initialize the inspector
    pub fn initialize(&self) -> Result<(), InspectorError> {
        // Initialize DOM tree with document root
        Ok(())
    }

    /// Get element by ID
    pub async fn get_element(&self, id: Uuid) -> Option<DOMElement> {
        self.dom_tree.read().await.get(&id).cloned()
    }

    /// Get root element
    pub async fn get_root_element(&self) -> Option<UUID> {
        *self.root_element.read().await
    }

    /// Set root element
    pub async fn set_root_element(&self, id: Uuid) {
        *self.root_element.write().await = Some(id);
    }

    /// Add element to DOM tree
    pub async fn add_element(&self, element: DOMElement) -> Result<(), InspectorError> {
        let mut dom_tree = self.dom_tree.write().await;
        dom_tree.insert(element.id, element);
        Ok(())
    }

    /// Update element
    pub async fn update_element(&self, id: Uuid, element: DOMElement) -> Result<(), InspectorError> {
        let mut dom_tree = self.dom_tree.write().await;
        dom_tree.insert(id, element);
        Ok(())
    }

    /// Remove element
    pub async fn remove_element(&self, id: Uuid) -> Result<(), InspectorError> {
        let mut dom_tree = self.dom_tree.write().await;
        dom_tree.remove(&id);
        Ok(())
    }

    /// Select element
    pub async fn select_element(&self, id: Uuid) -> Result<(), InspectorError> {
        let selection = InspectorSelection {
            element_id: id,
            selected_at: Utc::now(),
            highlighted: true,
        };
        *self.selection.write().await = Some(selection);
        Ok(())
    }

    /// Get selected element
    pub async fn get_selected_element(&self) -> Option<DOMElement> {
        let selection = self.selection.read().await;
        if let Some(ref sel) = *selection {
            self.get_element(sel.element_id).await
        } else {
            None
        }
    }

    /// Get selection
    pub async fn get_selection(&self) -> Option<InspectorSelection> {
        self.selection.read().await.clone()
    }

    /// Clear selection
    pub async fn clear_selection(&self) {
        *self.selection.write().await = None;
    }

    /// Pin element
    pub async fn pin_element(&self, id: Uuid) {
        self.pinned_elements.write().await.insert(id);
    }

    /// Unpin element
    pub async fn unpin_element(&self, id: Uuid) -> bool {
        self.pinned_elements.write().await.remove(&id)
    }

    /// Get pinned elements
    pub async fn get_pinned_elements(&self) -> Vec<DOMElement> {
        let pinned = self.pinned_elements.read().await.clone();
        let dom_tree = self.dom_tree.read().await;
        
        pinned.iter()
            .filter_map(|id| dom_tree.get(id).cloned())
            .collect()
    }

    /// Search elements by CSS selector
    pub async fn search_by_selector(&self, selector: &str) -> Result<Vec<Uuid>, InspectorError> {
        // Parse and execute CSS selector
        let mut results = Vec::new();
        let dom_tree = self.dom_tree.read().await;
        
        for (id, element) in dom_tree.iter() {
            if self.matches_selector(element, selector) {
                results.push(*id);
            }
        }
        
        Ok(results)
    }

    /// Search elements by text
    pub async fn search_by_text(&self, text: &str) -> Vec<SearchResult> {
        let mut results = Vec::new();
        let dom_tree = self.dom_tree.read().await;
        
        for (id, element) in dom_tree.iter() {
            if let Some(ref content) = element.text_content {
                if content.contains(text) {
                    results.push(SearchResult {
                        element_id: *id,
                        match_type: SearchMatchType::Text,
                        matched_text: text.to_string(),
                        context: self.get_context(content, text, 50),
                    });
                }
            }
        }
        
        results
    }

    /// Get computed styles for element
    pub async fn get_computed_styles(&self, id: Uuid) -> Result<ComputedStyles, InspectorError> {
        let element = self.get_element(id).await.ok_or(InspectorError::ElementNotFound)?;
        
        let mut properties = HashMap::new();
        
        // Compute styles from CSS rules
        for rule in self.css_rules.read().await.iter() {
            if self.matches_selector(&element, &rule.selector) {
                for prop in &rule.properties {
                    properties.insert(prop.name.clone(), prop.clone());
                }
            }
        }
        
        Ok(ComputedStyles {
            element_id: id,
            properties,
            inherited_from: element.parent,
        })
    }

    /// Update CSS property
    pub async fn update_css_property(
        &self,
        id: Uuid,
        property: &str,
        value: &str,
        important: bool,
    ) -> Result<(), InspectorError> {
        let mut dom_tree = self.dom_tree.write().await;
        
        if let Some(mut element) = dom_tree.get_mut(&id) {
            element.computed_styles.insert(property.to_string(), value.to_string());
            Ok(())
        } else {
            Err(InspectorError::ElementNotFound)
        }
    }

    /// Remove CSS property
    pub async fn remove_css_property(&self, id: Uuid, property: &str) -> Result<(), InspectorError> {
        let mut dom_tree = self.dom_tree.write().await;
        
        if let Some(mut element) = dom_tree.get_mut(&id) {
            element.computed_styles.remove(property);
            Ok(())
        } else {
            Err(InspectorError::ElementNotFound)
        }
    }

    /// Generate CSS selector for element
    pub async fn generate_selector(&self, id: Uuid) -> Result<String, InspectorError> {
        let element = self.get_element(id).await.ok_or(InspectorError::ElementNotFound)?;
        
        let mut selector = vec![element.tag_name.clone()];
        
        if let Some(ref elem_id) = element.element_id {
            selector.push(format!("#{}", elem_id));
        }
        
        for class in &element.classes {
            selector.push(format!(".{}", class));
        }
        
        Ok(selector.join(""))
    }

    /// Generate XPath for element
    pub async fn generate_xpath(&self, id: Uuid) -> Result<String, InspectorError> {
        let mut parts = Vec::new();
        let mut current_id = Some(id);
        
        while let Some(elem_id) = current_id {
            if let Some(element) = self.get_element(elem_id).await {
                let siblings = self.get_sibling_count(elem_id).await;
                let index = self.get_element_index(elem_id).await.unwrap_or(1);
                
                parts.push(format!(
                    "/{}[{}]",
                    element.tag_name,
                    index
                ));
                
                current_id = element.parent;
            } else {
                break;
            }
        }
        
        parts.reverse();
        Ok(parts.join(""))
    }

    /// Get element children
    pub async fn get_children(&self, id: Uuid) -> Vec<DOMElement> {
        let element = self.get_element(id).await;
        
        if let Some(el) = element {
            let dom_tree = self.dom_tree.read().await;
            el.children.iter()
                .filter_map(|child_id| dom_tree.get(child_id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get element count
    pub async fn get_elements_count(&self) -> usize {
        self.dom_tree.read().await.len()
    }

    /// Get all elements matching criteria
    pub async fn find_elements<F>(&self, predicate: F) -> Vec<DOMElement>
    where
        F: Fn(&DOMElement) -> bool,
    {
        let dom_tree = self.dom_tree.read().await;
        dom_tree.values()
            .filter(|e| predicate(e))
            .cloned()
            .collect()
    }

    /// Add CSS rule
    pub async fn add_css_rule(&self, rule: CSSRule) {
        self.css_rules.write().await.push(rule);
    }

    /// Clear all data
    pub async fn clear(&self) {
        self.dom_tree.write().await.clear();
        *self.root_element.write().await = None;
        *self.selection.write().await = None;
        self.pinned_elements.write().await.clear();
        self.css_rules.write().await.clear();
        self.search_results.write().await.clear();
    }

    // Helper methods

    fn matches_selector(&self, element: &DOMElement, selector: &str) -> bool {
        // Simple selector matching (in production, use proper CSS selector parser)
        if selector.starts_with('#') {
            let id = &selector[1..];
            element.element_id.as_ref().map(|e| e.as_str()) == Some(id)
        } else if selector.starts_with('.') {
            let class = &selector[1..];
            element.classes.contains(&class.to_string())
        } else {
            element.tag_name == selector
        }
    }

    fn get_context(&self, content: &str, search: &str, context_len: usize) -> String {
        let pos = match content.find(search) {
            Some(p) => p,
            None => return content.to_string(),
        };
        
        let start = pos.saturating_sub(context_len);
        let end = (pos + search.len() + context_len).min(content.len());
        
        let mut result = String::new();
        if start > 0 {
            result.push_str("...");
        }
        result.push_str(&content[start..end]);
        if end < content.len() {
            result.push_str("...");
        }
        
        result
    }

    async fn get_sibling_count(&self, id: Uuid) -> usize {
        if let Some(element) = self.get_element(id).await {
            if let Some(parent_id) = element.parent {
                if let Some(parent) = self.get_element(parent_id).await {
                    return parent.children.len();
                }
            }
        }
        1
    }

    async fn get_element_index(&self, id: Uuid) -> Option<usize> {
        if let Some(element) = self.get_element(id).await {
            if let Some(parent_id) = element.parent {
                if let Some(parent) = self.get_element(parent_id).await {
                    return parent.children.iter().position(|&child_id| child_id == id);
                }
            }
        }
        Some(1)
    }
}

/// Inspector error
#[derive(Debug, thiserror::Error)]
pub enum InspectorError {
    #[error("Element not found")]
    ElementNotFound,
    #[error("Invalid selector: {0}")]
    InvalidSelector(String),
    #[error("Property not found: {0}")]
    PropertyNotFound(String),
    #[error("Other error: {0}")]
    Other(String),
}

// UUID alias for clarity
type UUID = uuid::Uuid;

impl Default for ElementsInspector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_inspector_initialization() {
        let inspector = ElementsInspector::new();
        assert!(inspector.initialize().is_ok());
    }

    #[tokio::test]
    async fn test_add_and_get_element() {
        let inspector = ElementsInspector::new();
        
        let element = DOMElement {
            id: Uuid::new_v4(),
            tag_name: "div".to_string(),
            element_id: Some("test-id".to_string()),
            classes: vec!["test-class".to_string()],
            attributes: HashMap::new(),
            computed_styles: HashMap::new(),
            children: Vec::new(),
            parent: None,
            text_content: Some("Test content".to_string()),
            inner_html: None,
            box_model: None,
            pseudo_elements: Vec::new(),
            event_listeners: Vec::new(),
        };
        
        let id = element.id;
        inspector.add_element(element).await.unwrap();
        
        let retrieved = inspector.get_element(id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().tag_name, "div");
    }

    #[tokio::test]
    async fn test_select_element() {
        let inspector = ElementsInspector::new();
        
        let element = DOMElement {
            id: Uuid::new_v4(),
            tag_name: "p".to_string(),
            element_id: None,
            classes: Vec::new(),
            attributes: HashMap::new(),
            computed_styles: HashMap::new(),
            children: Vec::new(),
            parent: None,
            text_content: None,
            inner_html: None,
            box_model: None,
            pseudo_elements: Vec::new(),
            event_listeners: Vec::new(),
        };
        
        let id = element.id;
        inspector.add_element(element).await.unwrap();
        
        inspector.select_element(id).await.unwrap();
        let selected = inspector.get_selected_element().await;
        
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().id, id);
    }

    #[tokio::test]
    async fn test_pin_element() {
        let inspector = ElementsInspector::new();
        
        let element = DOMElement {
            id: Uuid::new_v4(),
            tag_name: "span".to_string(),
            element_id: None,
            classes: Vec::new(),
            attributes: HashMap::new(),
            computed_styles: HashMap::new(),
            children: Vec::new(),
            parent: None,
            text_content: None,
            inner_html: None,
            box_model: None,
            pseudo_elements: Vec::new(),
            event_listeners: Vec::new(),
        };
        
        let id = element.id;
        inspector.add_element(element).await.unwrap();
        
        inspector.pin_element(id).await;
        let pinned = inspector.get_pinned_elements().await;
        
        assert_eq!(pinned.len(), 1);
        assert_eq!(pinned[0].id, id);
    }

    #[tokio::test]
    async fn test_search_by_text() {
        let inspector = ElementsInspector::new();
        
        let element = DOMElement {
            id: Uuid::new_v4(),
            tag_name: "h1".to_string(),
            element_id: None,
            classes: Vec::new(),
            attributes: HashMap::new(),
            computed_styles: HashMap::new(),
            children: Vec::new(),
            parent: None,
            text_content: Some("Hello World".to_string()),
            inner_html: None,
            box_model: None,
            pseudo_elements: Vec::new(),
            event_listeners: Vec::new(),
        };
        
        inspector.add_element(element).await.unwrap();
        
        let results = inspector.search_by_text("World").await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].match_type, SearchMatchType::Text);
    }

    #[tokio::test]
    async fn test_generate_selector() {
        let inspector = ElementsInspector::new();
        
        let element = DOMElement {
            id: Uuid::new_v4(),
            tag_name: "button".to_string(),
            element_id: Some("submit".to_string()),
            classes: vec!["btn".to_string(), "primary".to_string()],
            attributes: HashMap::new(),
            computed_styles: HashMap::new(),
            children: Vec::new(),
            parent: None,
            text_content: None,
            inner_html: None,
            box_model: None,
            pseudo_elements: Vec::new(),
            event_listeners: Vec::new(),
        };
        
        let id = element.id;
        inspector.add_element(element).await.unwrap();
        
        let selector = inspector.generate_selector(id).await.unwrap();
        assert!(selector.contains("button"));
        assert!(selector.contains("#submit"));
        assert!(selector.contains(".btn"));
    }
}