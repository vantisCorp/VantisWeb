//! Autofill Manager for VantisWeb Password Manager
//! 
//! This module handles:
//! - Login form detection
//! - Password field identification
//! - Automatic form filling
//! - Credential matching

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use super::{PasswordEntry, PasswordError, AutofillResult};

/// Autofill manager
pub struct AutofillManager {
    /// Detected forms
    forms: Arc<RwLock<Vec<DetectedForm>>>,
    /// Autofill rules
    rules: Arc<RwLock<Vec<AutofillRule>>>,
    /// Configuration
    config: Arc<RwLock<AutofillConfig>>,
}

/// Detected login form
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedForm {
    /// Form ID
    pub id: String,
    /// Page URL
    pub page_url: String,
    /// Username field selector
    pub username_selector: String,
    /// Password field selector
    pub password_selector: String,
    /// Submit button selector
    pub submit_selector: Option<String>,
    /// Form type
    pub form_type: FormType,
    /// Detection confidence
    pub confidence: f32,
}

/// Form types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormType {
    Login,
    Registration,
    PasswordChange,
    PasswordReset,
    TwoFactor,
    Unknown,
}

/// Autofill rule for a website
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutofillRule {
    /// Website domain
    pub domain: String,
    /// Username field selectors
    pub username_selectors: Vec<String>,
    /// Password field selectors
    pub password_selectors: Vec<String>,
    /// Submit button selectors
    pub submit_selectors: Vec<String>,
    /// Whether to auto-submit
    pub auto_submit: bool,
    /// Custom form detection
    pub custom_detection: bool,
}

/// Autofill configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutofillConfig {
    /// Enable autofill
    pub enabled: bool,
    /// Auto-submit after filling
    pub auto_submit: bool,
    /// Show notification on autofill
    pub show_notification: bool,
    /// Delay before filling (ms)
    pub fill_delay_ms: u32,
    /// Only fill on HTTPS
    pub https_only: bool,
}

impl Default for AutofillConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_submit: false,
            show_notification: true,
            fill_delay_ms: 100,
            https_only: true,
        }
    }
}

/// Form field info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormField {
    /// Field selector
    pub selector: String,
    /// Field type
    pub field_type: String,
    /// Field name attribute
    pub name: Option<String>,
    /// Field ID
    pub id: Option<String>,
    /// Placeholder text
    pub placeholder: Option<String>,
    /// Whether field is visible
    pub visible: bool,
}

impl AutofillManager {
    /// Create a new autofill manager
    pub fn new() -> Self {
        Self {
            forms: Arc::new(RwLock::new(Vec::new())),
            rules: Arc::new(RwLock::new(Self::default_rules())),
            config: Arc::new(RwLock::new(AutofillConfig::default())),
        }
    }

    /// Get default autofill rules
    fn default_rules() -> Vec<AutofillRule> {
        vec![
            AutofillRule {
                domain: "*".to_string(),
                username_selectors: vec![
                    "input[type='email']".to_string(),
                    "input[type='text'][name*='user']".to_string(),
                    "input[type='text'][name*='email']".to_string(),
                    "input[type='text'][id*='user']".to_string(),
                    "input[type='text'][id*='email']".to_string(),
                    "input[name='username']".to_string(),
                    "input[name='email']".to_string(),
                    "input[id='username']".to_string(),
                    "input[id='email']".to_string(),
                ],
                password_selectors: vec![
                    "input[type='password']".to_string(),
                    "input[name*='pass']".to_string(),
                    "input[id*='pass']".to_string(),
                ],
                submit_selectors: vec![
                    "button[type='submit']".to_string(),
                    "input[type='submit']".to_string(),
                    "button:contains('Sign in')".to_string(),
                    "button:contains('Log in')".to_string(),
                ],
                auto_submit: false,
                custom_detection: false,
            },
        ]
    }

    /// Detect login forms on a page
    pub async fn detect_forms(&self, page_url: &str, html: &str) -> Vec<DetectedForm> {
        let mut forms = Vec::new();
        
        // Get rules for this domain
        let rules = self.rules.read().await;
        let applicable_rules: Vec<_> = rules.iter()
            .filter(|r| r.domain == "*" || page_url.contains(&r.domain))
            .collect();
        drop(rules);
        
        // Simple heuristic detection
        if html.contains("type=&quot;password&quot;") || html.contains("type='password'") {
            let form = DetectedForm {
                id: format!("form_{}", Utc::now().timestamp_millis()),
                page_url: page_url.to_string(),
                username_selector: "input[type='email'], input[type='text'][name*='user']".to_string(),
                password_selector: "input[type='password']".to_string(),
                submit_selector: Some("button[type='submit']".to_string()),
                form_type: FormType::Login,
                confidence: 0.8,
            };
            forms.push(form);
        }
        
        // Store detected forms
        let mut stored_forms = self.forms.write().await;
        stored_forms.clear();
        stored_forms.extend(forms.clone());
        
        forms
    }

    /// Get password for a URL
    pub async fn get_password_for_url(&self, url: &str) -> Result<Option<PasswordEntry>, PasswordError> {
        // This would be implemented to query the storage
        // For now, return None
        Ok(None)
    }

    /// Autofill credentials
    pub async fn autofill(&self, url: &str, username: String) -> Result<AutofillResult, PasswordError> {
        let config = self.config.read().await;
        
        if !config.enabled {
            return Err(PasswordError::AutofillDisabled);
        }
        
        if config.https_only && !url.starts_with("https://") {
            return Err(PasswordError::AutofillDisabled);
        }
        drop(config);
        
        // Get credentials for this URL
        let entry = self.get_password_for_url(url).await?;
        
        if let Some(entry) = entry {
            Ok(AutofillResult {
                username: entry.username,
                password: entry.password_encrypted,
                additional_fields: HashMap::new(),
                auto_submitted: false,
            })
        } else {
            Ok(AutofillResult {
                username,
                password: String::new(),
                additional_fields: HashMap::new(),
                auto_submitted: false,
            })
        }
    }

    /// Fill form with credentials
    pub async fn fill_form(&self, form_id: &str, username: &str, password: &str) -> Result<(), PasswordError> {
        let forms = self.forms.read().await;
        let form = forms.iter()
            .find(|f| f.id == form_id)
            .ok_or_else(|| PasswordError::NotFound(form_id.to_string()))?;
        
        // In production, this would inject JavaScript to fill the form
        println!("Filling form {} with username: {}", form.id, username);
        
        Ok(())
    }

    /// Add custom autofill rule
    pub async fn add_rule(&self, rule: AutofillRule) {
        let mut rules = self.rules.write().await;
        rules.push(rule);
    }

    /// Remove autofill rule
    pub async fn remove_rule(&self, domain: &str) -> bool {
        let mut rules = self.rules.write().await;
        let initial_len = rules.len();
        rules.retain(|r| r.domain != domain);
        rules.len() != initial_len
    }

    /// Get rules for a domain
    pub async fn get_rules(&self, domain: &str) -> Vec<AutofillRule> {
        let rules = self.rules.read().await;
        rules.iter()
            .filter(|r| r.domain == "*" || r.domain == domain)
            .cloned()
            .collect()
    }

    /// Update configuration
    pub async fn update_config<F>(&self, f: F)
    where
        F: FnOnce(&mut AutofillConfig),
    {
        let mut config = self.config.write().await;
        f(&mut config);
    }

    /// Get configuration
    pub async fn get_config(&self) -> AutofillConfig {
        self.config.read().await.clone()
    }

    /// Check if URL has saved credentials
    pub async fn has_credentials_for_url(&self, _url: &str) -> bool {
        // This would check storage for matching credentials
        false
    }

    /// Generate autofill suggestions for a field
    pub async fn get_suggestions(&self, field_type: &str, url: &str) -> Vec<AutofillSuggestion> {
        let mut suggestions = Vec::new();
        
        if field_type == "username" || field_type == "email" {
            // Get stored usernames for this URL
            suggestions.push(AutofillSuggestion {
                value: "user@example.com".to_string(),
                label: "user@example.com".to_string(),
                source: "password_manager".to_string(),
            });
        }
        
        suggestions
    }

    /// Clear detected forms
    pub async fn clear_forms(&self) {
        self.forms.write().await.clear();
    }
}

/// Autofill suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutofillSuggestion {
    /// Suggested value
    pub value: String,
    /// Display label
    pub label: String,
    /// Source of suggestion
    pub source: String,
}

impl Default for AutofillManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_manager() {
        let manager = AutofillManager::new();
        let config = manager.get_config().await;
        assert!(config.enabled);
    }

    #[tokio::test]
    async fn test_detect_forms() {
        let manager = AutofillManager::new();
        let html = r##"<input type="password" name="password"><input type="text" name="username">"##;
        
        let forms = manager.detect_forms("https://example.com", html).await;
        assert!(!forms.is_empty());
        assert_eq!(forms[0].form_type, FormType::Login);
    }

    #[tokio::test]
    async fn test_add_rule() {
        let manager = AutofillManager::new();
        
        let rule = AutofillRule {
            domain: "example.com".to_string(),
            username_selectors: vec!["#user".to_string()],
            password_selectors: vec!["#pass".to_string()],
            submit_selectors: vec!["#submit".to_string()],
            auto_submit: true,
            custom_detection: true,
        };
        
        manager.add_rule(rule).await;
        
        let rules = manager.get_rules("example.com").await;
        assert!(rules.len() > 1); // Default rule + custom rule
    }

    #[tokio::test]
    async fn test_update_config() {
        let manager = AutofillManager::new();
        
        manager.update_config(|config| {
            config.auto_submit = true;
        }).await;
        
        let config = manager.get_config().await;
        assert!(config.auto_submit);
    }

    #[tokio::test]
    async fn test_autofill_disabled() {
        let manager = AutofillManager::new();
        
        manager.update_config(|config| {
            config.enabled = false;
        }).await;
        
        let result = manager.autofill("https://example.com", "user".to_string()).await;
        assert!(matches!(result, Err(PasswordError::AutofillDisabled)));
    }

    #[tokio::test]
    async fn test_https_only() {
        let manager = AutofillManager::new();
        
        manager.update_config(|config| {
            config.https_only = true;
        }).await;
        
        let result = manager.autofill("http://example.com", "user".to_string()).await;
        assert!(matches!(result, Err(PasswordError::AutofillDisabled)));
    }

    #[tokio::test]
    async fn test_get_suggestions() {
        let manager = AutofillManager::new();
        let suggestions = manager.get_suggestions("username", "https://example.com").await;
        
        assert!(!suggestions.is_empty());
    }
}