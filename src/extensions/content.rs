// Copyright 2024 Vantis Corporation - All Rights Reserved

//! Content Scripts Management
//! 
//! This module provides comprehensive content script management including:
//! - Dynamic script injection
//! - CSS injection
//! - Programmatic injection
//! - User script support
//! - Isolated worlds for scripts
//! - Cross-origin communication
//! - DOM event interception

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Content script information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentScript {
    /// Script ID
    pub id: Uuid,
    /// Extension ID
    pub extension_id: Uuid,
    /// Script type
    pub script_type: ScriptType,
    /// Source code
    pub source: String,
    /// File path (if loaded from file)
    pub file_path: Option<String>,
    /// URL to inject (for external scripts)
    pub url: Option<String>,
    /// World ID (for isolation)
    pub world_id: String,
    /// Match patterns
    pub matches: Vec<String>,
    /// Exclude match patterns
    pub exclude_matches: Vec<String>,
    /// Run at
    pub run_at: RunAt,
    /// All frames
    pub all_frames: bool,
    /// Match about blank
    pub match_about_blank: bool,
    /// Match origin as fallback
    pub match_origin_as_fallback: bool,
    /// CSS files to inject
    pub css: Vec<String>,
    /// Enabled
    pub enabled: bool,
    /// Creation time
    pub created_at: DateTime<Utc>,
}

/// Script type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScriptType {
    /// JavaScript
    JavaScript,
    /// CSS
    CSS,
    /// User script (userscripts.org format)
    UserScript,
}

/// When to run the script
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RunAt {
    /// Document idle
    DocumentIdle,
    /// Document end
    DocumentEnd,
    /// Document start
    DocumentStart,
}

/// Script injection context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionContext {
    /// Tab ID
    pub tab_id: Uuid,
    /// Frame ID
    pub frame_id: Uuid,
    /// URL
    pub url: String,
    /// Origin
    pub origin: String,
    /// Is main frame
    pub is_main_frame: bool,
    /// Injection timestamp
    pub timestamp: DateTime<Utc>,
}

/// Script execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Script ID
    pub script_id: Uuid,
    /// Context
    pub context: InjectionContext,
    /// Success
    pub success: bool,
    /// Error message
    pub error: Option<String>,
    /// Execution time (ms)
    pub execution_time: f64,
    /// Return value
    pub return_value: Option<serde_json::Value>,
}

/// Message from content script
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentScriptMessage {
    /// Message ID
    pub id: Uuid,
    /// Sender extension ID
    pub extension_id: Uuid,
    /// Sender script ID
    pub script_id: Option<Uuid>,
    /// Target extension ID (optional)
    pub target_extension_id: Option<Uuid>,
    /// Message type
    pub message_type: String,
    /// Data
    pub data: serde_json::Value,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// DOM event interception
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DOMEventInterception {
    /// Interception ID
    pub id: Uuid,
    /// Extension ID
    pub extension_id: Uuid,
    /// Event type
    pub event_type: String,
    /// Selector
    pub selector: Option<String>,
    /// Prevent default
    pub prevent_default: bool,
    /// Stop propagation
    pub stop_propagation: bool,
    /// Handler code
    pub handler: String,
    /// Enabled
    pub enabled: bool,
}

/// User script metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserScriptMetadata {
    /// Script name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Version
    pub version: Option<String>,
    /// Author
    pub author: Option<String>,
    /// License
    pub license: Option<String>,
    /// Homepage
    pub homepage: Option<String>,
    /// Update URL
    pub update_url: Option<String>,
    /// Download URL
    pub download_url: Option<String>,
    /// Support URL
    pub support_url: Option<String>,
    /// Namespace
    pub namespace: Option<String>,
    /// Grant
    pub grant: Vec<String>,
    /// Resource
    pub resource: HashMap<String, String>,
    /// Require
    pub require: Vec<String>,
    /// Icon
    pub icon: Option<String>,
}

/// Content script manager
pub struct ContentScriptManager {
    /// Registered content scripts
    scripts: Arc<RwLock<HashMap<Uuid, ContentScript>>>,
    /// Scripts by extension
    scripts_by_extension: Arc<RwLock<HashMap<Uuid, HashSet<Uuid>>>>,
    /// Execution history
    execution_history: Arc<RwLock<Vec<ExecutionResult>>>,
    /// Active interceptors
    interceptors: Arc<RwLock<HashMap<Uuid, DOMEventInterception>>>,
    /// Message handlers
    message_handlers: Arc<RwLock<HashMap<String, Box<dyn ContentMessageHandler>>>>,
    /// Isolated worlds
    isolated_worlds: Arc<RwLock<HashMap<String, IsolatedWorld>>>,
}

/// Isolated world context
#[derive(Debug, Clone)]
pub struct IsolatedWorld {
    /// World ID
    pub world_id: String,
    /// Extension ID
    pub extension_id: Uuid,
    /// Scripts in this world
    pub scripts: HashSet<Uuid>,
    /// Created at
    pub created_at: DateTime<Utc>,
}

/// Content message handler trait
pub trait ContentMessageHandler: Send + Sync {
    /// Handle message
    fn handle_message(&self, message: ContentScriptMessage) -> Result<serde_json::Value, String>;
}

impl ContentScriptManager {
    /// Create a new content script manager
    pub fn new() -> Self {
        Self {
            scripts: Arc::new(RwLock::new(HashMap::new())),
            scripts_by_extension: Arc::new(RwLock::new(HashMap::new())),
            execution_history: Arc::new(RwLock::new(Vec::new())),
            interceptors: Arc::new(RwLock::new(HashMap::new())),
            message_handlers: Arc::new(RwLock::new(HashMap::new())),
            isolated_worlds: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a content script
    pub async fn register_script(&self, script: ContentScript) -> Result<(), String> {
        let script_id = script.id;
        let extension_id = script.extension_id;

        // Add to scripts map
        self.scripts.write().await.insert(script_id, script);

        // Add to extension scripts
        let mut scripts_by_ext = self.scripts_by_extension.write().await;
        scripts_by_ext
            .entry(extension_id)
            .or_insert_with(HashSet::new)
            .insert(script_id);

        Ok(())
    }

    /// Unregister a content script
    pub async fn unregister_script(&self, script_id: Uuid) -> Result<(), String> {
        // Get script extension ID
        let extension_id = if let Some(script) = self.scripts.read().await.get(&script_id) {
            Some(script.extension_id)
        } else {
            return Err("Script not found".to_string());
        };

        // Remove from scripts
        self.scripts.write().await.remove(&script_id);

        // Remove from extension scripts
        let mut scripts_by_ext = self.scripts_by_extension.write().await;
        if let Some(ref mut scripts) = scripts_by_ext.get_mut(&extension_id.unwrap()) {
            scripts.remove(&script_id);
        }

        Ok(())
    }

    /// Get scripts for extension
    pub async fn get_extension_scripts(&self, extension_id: Uuid) -> Vec<ContentScript> {
        let scripts_by_ext = self.scripts_by_extension.read().await;
        let scripts = self.scripts.read().await;

        if let Some(script_ids) = scripts_by_ext.get(&extension_id) {
            script_ids
                .iter()
                .filter_map(|id| scripts.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get scripts for URL
    pub async fn get_scripts_for_url(&self, url: &str) -> Vec<ContentScript> {
        let scripts = self.scripts.read().await;

        scripts
            .values()
            .filter(|script| self.script_matches_url(script, url))
            .filter(|script| script.enabled)
            .cloned()
            .collect()
    }

    /// Inject script into context
    pub async fn inject_script(
        &self,
        script_id: Uuid,
        context: InjectionContext,
    ) -> Result<ExecutionResult, String> {
        let script = self.scripts.read().await.get(&script_id)
            .ok_or_else(|| "Script not found".to_string())?
            .clone();

        let start = std::time::Instant::now();

        // Execute script (simplified - in production, use actual JS engine)
        let result = match script.script_type {
            ScriptType::JavaScript => self.execute_javascript(&script, &context).await,
            ScriptType::CSS => self.inject_css(&script, &context).await,
            ScriptType::UserScript => self.execute_user_script(&script, &context).await,
        };

        let execution_time = start.elapsed().as_secs_f64();

        let execution_result = ExecutionResult {
            script_id,
            context,
            success: result.is_ok(),
            error: result.err(),
            execution_time,
            return_value: result.ok(),
        };

        // Record execution
        self.execution_history.write().await.push(execution_result.clone());

        Ok(execution_result)
    }

    /// Register DOM event interceptor
    pub async fn register_interceptor(&self, interceptor: DOMEventInterception) {
        self.interceptors.write().await.insert(interceptor.id, interceptor);
    }

    /// Unregister interceptor
    pub async fn unregister_interceptor(&self, interceptor_id: Uuid) -> bool {
        self.interceptors.write().await.remove(&interceptor_id).is_some()
    }

    /// Get interceptors for event
    pub async fn get_interceptors_for_event(&self, event_type: &str) -> Vec<DOMEventInterception> {
        let interceptors = self.interceptors.read().await;

        interceptors
            .values()
            .filter(|i| i.event_type == event_type && i.enabled)
            .cloned()
            .collect()
    }

    /// Add message handler
    pub async fn add_message_handler(
        &self,
        message_type: String,
        handler: Box<dyn ContentMessageHandler>,
    ) {
        self.message_handlers.write().await.insert(message_type, handler);
    }

    /// Handle content script message
    pub async fn handle_message(&self, message: ContentScriptMessage) -> Result<serde_json::Value, String> {
        let handlers = self.message_handlers.read().await;

        if let Some(handler) = handlers.get(&message.message_type) {
            handler.handle_message(message)
        } else {
            Err("No handler for message type".to_string())
        }
    }

    /// Create isolated world
    pub async fn create_isolated_world(&self, world_id: String, extension_id: Uuid) -> IsolatedWorld {
        let world = IsolatedWorld {
            world_id: world_id.clone(),
            extension_id,
            scripts: HashSet::new(),
            created_at: Utc::now(),
        };

        self.isolated_worlds.write().await.insert(world_id.clone(), world.clone());
        world
    }

    /// Get isolated world
    pub async fn get_isolated_world(&self, world_id: &str) -> Option<IsolatedWorld> {
        self.isolated_worlds.read().await.get(world_id).cloned()
    }

    /// Parse user script
    pub async fn parse_user_script(&self, content: &str) -> Result<(UserScriptMetadata, ContentScript), String> {
        let metadata = self.parse_user_script_metadata(content)?;
        let script = self.create_user_script_content(content, &metadata).await?;

        Ok((metadata, script))
    }

    /// Get execution history
    pub async fn get_execution_history(&self, limit: usize) -> Vec<ExecutionResult> {
        let history = self.execution_history.read().await;
        history.iter().rev().take(limit).cloned().collect()
    }

    /// Clear execution history
    pub async fn clear_execution_history(&self) {
        self.execution_history.write().await.clear();
    }

    /// Get all scripts
    pub async fn get_all_scripts(&self) -> Vec<ContentScript> {
        self.scripts.read().await.values().cloned().collect()
    }

    /// Enable/disable script
    pub async fn set_script_enabled(&self, script_id: Uuid, enabled: bool) -> Result<(), String> {
        let mut scripts = self.scripts.write().await;
        if let Some(script) = scripts.get_mut(&script_id) {
            script.enabled = enabled;
            Ok(())
        } else {
            Err("Script not found".to_string())
        }
    }

    // Helper methods

    fn script_matches_url(&self, script: &ContentScript, url: &str) -> bool {
        // Check include patterns
        if !script.matches.is_empty() {
            let matches_any = script.matches.iter().any(|pattern| {
                self.match_pattern(pattern, url)
            });

            if !matches_any {
                return false;
            }
        }

        // Check exclude patterns
        if !script.exclude_matches.is_empty() {
            let matches_any = script.exclude_matches.iter().any(|pattern| {
                self.match_pattern(pattern, url)
            });

            if matches_any {
                return false;
            }
        }

        true
    }

    fn match_pattern(&self, pattern: &str, url: &str) -> bool {
        // Simplified pattern matching
        // In production, use proper URL pattern matching
        if pattern == "<all_urls>" {
            return true;
        }

        if pattern.contains('*') {
            let pattern_parts: Vec<&str> = pattern.split('*').collect();
            if pattern_parts.len() == 2 {
                return url.starts_with(pattern_parts[0]) && url.ends_with(pattern_parts[1]);
            }
        }

        url == pattern || url.starts_with(&format!("{}/", pattern))
    }

    async fn execute_javascript(&self, script: &ContentScript, context: &InjectionContext) -> Result<serde_json::Value, String> {
        // In production, integrate with actual JavaScript engine
        // This is a simplified version
        Ok(serde_json::Value::String(format!("Executed script {}", script.id)))
    }

    async fn inject_css(&self, script: &ContentScript, context: &InjectionContext) -> Result<serde_json::Value, String> {
        // In production, inject CSS into the page
        Ok(serde_json::Value::String(format!("Injected CSS {}", script.id)))
    }

    async fn execute_user_script(&self, script: &ContentScript, context: &InjectionContext) -> Result<serde_json::Value, String> {
        // User scripts may have special grants and metadata
        Ok(serde_json::Value::String(format!("Executed user script {}", script.id)))
    }

    fn parse_user_script_metadata(&self, content: &str) -> Result<UserScriptMetadata, String> {
        // Parse user script metadata block
        let mut metadata = UserScriptMetadata {
            name: "Unnamed".to_string(),
            description: None,
            version: None,
            author: None,
            license: None,
            homepage: None,
            update_url: None,
            download_url: None,
            support_url: None,
            namespace: None,
            grant: vec![],
            resource: HashMap::new(),
            require: vec![],
            icon: None,
        };

        // Simple metadata parsing
        for line in content.lines() {
            if line.trim_start().starts_with("// @") {
                let parts: Vec<&str> = line.trim_start_matches("// @").split_whitespace().collect();
                if parts.len() >= 2 {
                    let key = parts[0].to_lowercase();
                    let value = parts[1..].join(" ");

                    match key.as_str() {
                        "name" => metadata.name = value,
                        "description" => metadata.description = Some(value),
                        "version" => metadata.version = Some(value),
                        "author" => metadata.author = Some(value),
                        "license" => metadata.license = Some(value),
                        "homepage" => metadata.homepage = Some(value),
                        "updateurl" => metadata.update_url = Some(value),
                        "downloadurl" => metadata.download_url = Some(value),
                        "supporturl" => metadata.support_url = Some(value),
                        "namespace" => metadata.namespace = Some(value),
                        "grant" => metadata.grant.push(value),
                        "require" => metadata.require.push(value),
                        "icon" => metadata.icon = Some(value),
                        _ => {}
                    }
                }
            }
        }

        Ok(metadata)
    }

    async fn create_user_script_content(&self, content: &str, metadata: &UserScriptMetadata) -> Result<ContentScript, String> {
        Ok(ContentScript {
            id: Uuid::new_v4(),
            extension_id: Uuid::new_v4(), // Would be actual extension ID
            script_type: ScriptType::UserScript,
            source: content.to_string(),
            file_path: None,
            url: None,
            world_id: format!("userscript_{}", metadata.name.replace(' ', "_")),
            matches: vec!["<all_urls>".to_string()], // User scripts typically match all URLs
            exclude_matches: vec![],
            run_at: RunAt::DocumentIdle,
            all_frames: false,
            match_about_blank: false,
            match_origin_as_fallback: false,
            css: vec![],
            enabled: true,
            created_at: Utc::now(),
        })
    }
}

impl Default for ContentScriptManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_script() {
        let manager = ContentScriptManager::new();
        
        let script = ContentScript {
            id: Uuid::new_v4(),
            extension_id: Uuid::new_v4(),
            script_type: ScriptType::JavaScript,
            source: "console.log('Hello')".to_string(),
            file_path: None,
            url: None,
            world_id: "world1".to_string(),
            matches: vec!["*://example.com/*".to_string()],
            exclude_matches: vec![],
            run_at: RunAt::DocumentEnd,
            all_frames: false,
            match_about_blank: false,
            match_origin_as_fallback: false,
            css: vec![],
            enabled: true,
            created_at: Utc::now(),
        };
        
        assert!(manager.register_script(script).await.is_ok());
    }

    #[tokio::test]
    async fn test_get_scripts_for_url() {
        let manager = ContentScriptManager::new();
        
        let script = ContentScript {
            id: Uuid::new_v4(),
            extension_id: Uuid::new_v4(),
            script_type: ScriptType::JavaScript,
            source: "test".to_string(),
            file_path: None,
            url: None,
            world_id: "world1".to_string(),
            matches: vec!["*://example.com/*".to_string()],
            exclude_matches: vec![],
            run_at: RunAt::DocumentEnd,
            all_frames: false,
            match_about_blank: false,
            match_origin_as_fallback: false,
            css: vec![],
            enabled: true,
            created_at: Utc::now(),
        };
        
        let script_id = script.id;
        manager.register_script(script).await.unwrap();
        
        let scripts = manager.get_scripts_for_url("https://example.com/page").await;
        assert_eq!(scripts.len(), 1);
        assert_eq!(scripts[0].id, script_id);
    }

    #[tokio::test]
    async fn test_parse_user_script() {
        let manager = ContentScriptManager::new();
        
        let user_script = r#"
// ==UserScript==
// @name         Test Script
// @namespace    http://tampermonkey.net/
// @version      0.1
// @description  try to take over the world!
// @author       You
// @match        http://*/*
// @grant        none
// ==/UserScript==

(function() {
    'use strict';
    console.log('Hello from user script!');
})();
"#;

        let result = manager.parse_user_script(user_script).await;
        assert!(result.is_ok());
        
        let (metadata, _) = result.unwrap();
        assert_eq!(metadata.name, "Test Script");
        assert_eq!(metadata.version, Some("0.1".to_string()));
    }

    #[tokio::test]
    async fn test_isolated_worlds() {
        let manager = ContentScriptManager::new();
        let extension_id = Uuid::new_v4();
        
        let world = manager.create_isolated_world("world1".to_string(), extension_id).await;
        assert_eq!(world.world_id, "world1");
        
        let retrieved = manager.get_isolated_world("world1").await;
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_interceptors() {
        let manager = ContentScriptManager::new();
        
        let interceptor = DOMEventInterception {
            id: Uuid::new_v4(),
            extension_id: Uuid::new_v4(),
            event_type: "click".to_string(),
            selector: Some("button.submit".to_string()),
            prevent_default: false,
            stop_propagation: false,
            handler: "console.log('Click intercepted')".to_string(),
            enabled: true,
        };
        
        manager.register_interceptor(interceptor).await;
        let interceptors = manager.get_interceptors_for_event("click").await;
        assert_eq!(interceptors.len(), 1);
    }
}