//! Custom Rules Manager for VantisWeb Ad Blocker
//! 
//! This module manages user-defined blocking rules including:
//! - URL allowlisting/blocklisting
//! - Element hiding rules
//! - Script injection rules
//! - Redirect rules
//! - Rule groups and presets

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use regex::Regex;
use chrono::{DateTime, Utc};

/// A custom blocking rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRule {
    /// Unique identifier
    pub id: String,
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: Option<String>,
    /// The rule pattern
    pub pattern: String,
    /// Rule type
    pub rule_type: CustomRuleType,
    /// Action to take
    pub action: RuleAction,
    /// Whether this rule is enabled
    pub enabled: bool,
    /// Priority (higher = more important)
    pub priority: i32,
    /// Domains this rule applies to (empty = all)
    pub domains: Vec<String>,
    /// Resource types this applies to
    pub resource_types: Vec<ResourceType>,
    /// When created
    pub created_at: DateTime<Utc>,
    /// When last modified
    pub modified_at: DateTime<Utc>,
    /// Usage count
    pub hit_count: usize,
    /// Compiled regex (cached)
    #[serde(skip)]
    pub compiled_regex: Option<Regex>,
}

/// Types of custom rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustomRuleType {
    /// URL pattern match
    UrlPattern,
    /// Domain block
    Domain,
    /// Exact URL match
    Exact,
    /// Regex pattern
    Regex,
    /// CSS selector hide
    ElementHide,
    /// Extended CSS selector
    ExtendedCss,
    /// Scriptlet injection
    Scriptlet,
    /// Redirect to another URL
    Redirect,
    /// Remove URL parameter
    RemoveParam,
    /// Header modification
    HeaderModify,
}

/// Actions a rule can take
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleAction {
    /// Block the request
    Block,
    /// Allow the request (whitelist)
    Allow,
    /// Hide element
    Hide,
    /// Remove element
    Remove,
    /// Redirect to another URL
    Redirect,
    /// Log only (no action)
    Log,
}

/// Resource types for rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    All,
    Script,
    Image,
    Stylesheet,
    Object,
    XmlHttpRequest,
    SubDocument,
    Document,
    Font,
    Media,
    WebSocket,
    Ping,
    Other,
}

/// A group of related rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleGroup {
    /// Group ID
    pub id: String,
    /// Group name
    pub name: String,
    /// Group description
    pub description: Option<String>,
    /// Rules in this group
    pub rules: Vec<String>,
    /// Whether group is enabled
    pub enabled: bool,
    /// Group icon
    pub icon: Option<String>,
    /// Group color
    pub color: Option<String>,
}

/// Rule presets for common use cases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulePreset {
    /// Preset ID
    pub id: String,
    /// Preset name
    pub name: String,
    /// Preset description
    pub description: String,
    /// Preset category
    pub category: PresetCategory,
    /// Rules in this preset
    pub rules: Vec<CustomRule>,
    /// Whether installed
    pub installed: bool,
}

/// Preset categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PresetCategory {
    /// Privacy protection
    Privacy,
    /// Ad blocking
    Ads,
    /// Anti-fingerprinting
    Fingerprinting,
    /// Social media
    Social,
    /// Annoyances
    Annoyances,
    /// Developer tools
    Developer,
    /// Performance
    Performance,
    /// Security
    Security,
    /// Custom
    Custom,
}

/// Rule conflict result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConflict {
    /// First rule ID
    pub rule_a: String,
    /// Second rule ID
    pub rule_b: String,
    /// Conflict type
    pub conflict_type: ConflictType,
    /// Resolution suggestion
    pub suggestion: String,
}

/// Types of rule conflicts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictType {
    /// Rules contradict each other
    Contradiction,
    /// One rule makes another redundant
    Redundancy,
    /// Rules have overlapping patterns
    Overlap,
    /// Rule is a subset of another
    Subset,
}

/// Custom rules manager
pub struct RuleManager {
    /// All custom rules
    rules: Arc<RwLock<HashMap<String, CustomRule>>>,
    /// Rule groups
    groups: Arc<RwLock<HashMap<String, RuleGroup>>>,
    /// Whitelist rules (optimized)
    whitelist: Arc<RwLock<Vec<CustomRule>>>,
    /// Blocklist rules (optimized)
    blocklist: Arc<RwLock<Vec<CustomRule>>>,
    /// Element hiding rules
    hiding_rules: Arc<RwLock<Vec<CustomRule>>>,
    /// Available presets
    presets: Vec<RulePreset>,
}

impl RuleManager {
    /// Create a new rule manager
    pub fn new() -> Self {
        Self {
            rules: Arc::new(RwLock::new(HashMap::new())),
            groups: Arc::new(RwLock::new(HashMap::new())),
            whitelist: Arc::new(RwLock::new(Vec::new())),
            blocklist: Arc::new(RwLock::new(Vec::new())),
            hiding_rules: Arc::new(RwLock::new(Vec::new())),
            presets: Self::default_presets(),
        }
    }

    /// Get default rule presets
    fn default_presets() -> Vec<RulePreset> {
        vec![
            RulePreset {
                id: "privacy-basic".to_string(),
                name: "Basic Privacy".to_string(),
                description: "Essential privacy protection rules".to_string(),
                category: PresetCategory::Privacy,
                rules: vec![
                    CustomRule {
                        id: "privacy-1".to_string(),
                        name: "Block tracking pixels".to_string(),
                        description: Some("Blocks 1x1 tracking pixels".to_string()),
                        pattern: r"\.gif\?.*pixel".to_string(),
                        rule_type: CustomRuleType::Regex,
                        action: RuleAction::Block,
                        enabled: true,
                        priority: 50,
                        domains: vec![],
                        resource_types: vec![ResourceType::Image],
                        created_at: Utc::now(),
                        modified_at: Utc::now(),
                        hit_count: 0,
                        compiled_regex: None,
                    },
                ],
                installed: false,
            },
            RulePreset {
                id: "social-remove".to_string(),
                name: "Remove Social Widgets".to_string(),
                description: "Removes social media share buttons and widgets".to_string(),
                category: PresetCategory::Social,
                rules: vec![
                    CustomRule {
                        id: "social-1".to_string(),
                        name: "Hide share buttons".to_string(),
                        description: None,
                        pattern: ".share-buttons, .social-share, .social-widgets".to_string(),
                        rule_type: CustomRuleType::ElementHide,
                        action: RuleAction::Hide,
                        enabled: true,
                        priority: 30,
                        domains: vec![],
                        resource_types: vec![ResourceType::All],
                        created_at: Utc::now(),
                        modified_at: Utc::now(),
                        hit_count: 0,
                        compiled_regex: None,
                    },
                ],
                installed: false,
            },
            RulePreset {
                id: "dev-tools".to_string(),
                name: "Developer Tools".to_string(),
                description: "Useful rules for web developers".to_string(),
                category: PresetCategory::Developer,
                rules: vec![
                    CustomRule {
                        id: "dev-1".to_string(),
                        name: "Block analytics locally".to_string(),
                        description: Some("Blocks analytics scripts for cleaner debugging".to_string()),
                        pattern: "analytics".to_string(),
                        rule_type: CustomRuleType::UrlPattern,
                        action: RuleAction::Block,
                        enabled: true,
                        priority: 40,
                        domains: vec!["localhost".to_string()],
                        resource_types: vec![ResourceType::Script],
                        created_at: Utc::now(),
                        modified_at: Utc::now(),
                        hit_count: 0,
                        compiled_regex: None,
                    },
                ],
                installed: false,
            },
        ]
    }

    /// Add a custom rule
    pub async fn add_rule(&self, rule: CustomRule) -> Result<(), RuleError> {
        // Validate rule
        self.validate_rule(&rule)?;
        
        // Compile regex if needed
        let mut rule = rule;
        if rule.rule_type == CustomRuleType::Regex {
            rule.compiled_regex = Some(Regex::new(&rule.pattern)
                .map_err(|e| RuleError::InvalidPattern(e.to_string()))?);
        }
        
        let id = rule.id.clone();
        let rules = self.rules.write().await;
        drop(rules);
        
        // Store rule
        self.rules.write().await.insert(id.clone(), rule.clone());
        
        // Update caches
        self.update_caches().await;
        
        Ok(())
    }

    /// Remove a rule
    pub async fn remove_rule(&self, rule_id: &str) -> Result<(), RuleError> {
        let mut rules = self.rules.write().await;
        rules.remove(rule_id)
            .ok_or_else(|| RuleError::NotFound(rule_id.to_string()))?;
        drop(rules);
        
        self.update_caches().await;
        
        Ok(())
    }

    /// Update a rule
    pub async fn update_rule<F>(&self, rule_id: &str, f: F) -> Result<(), RuleError>
    where
        F: FnOnce(&mut CustomRule),
    {
        let mut rules = self.rules.write().await;
        let rule = rules.get_mut(rule_id)
            .ok_or_else(|| RuleError::NotFound(rule_id.to_string()))?;
        
        f(rule);
        rule.modified_at = Utc::now();
        
        drop(rules);
        self.update_caches().await;
        
        Ok(())
    }

    /// Toggle rule enabled state
    pub async fn toggle_rule(&self, rule_id: &str, enabled: bool) -> Result<(), RuleError> {
        self.update_rule(rule_id, |r| r.enabled = enabled).await
    }

    /// Get a rule by ID
    pub async fn get_rule(&self, rule_id: &str) -> Option<CustomRule> {
        self.rules.read().await.get(rule_id).cloned()
    }

    /// Get all rules
    pub async fn get_all_rules(&self) -> Vec<CustomRule> {
        self.rules.read().await.values().cloned().collect()
    }

    /// Check if a URL matches any rule
    pub async fn check_url(&self, url: &str, domain: &str, resource_type: ResourceType) -> Option<(CustomRule, RuleAction)> {
        // Check whitelist first (higher priority)
        let whitelist = self.whitelist.read().await;
        for rule in whitelist.iter() {
            if self.matches_rule(rule, url, domain, resource_type) {
                return Some((rule.clone(), RuleAction::Allow));
            }
        }
        drop(whitelist);
        
        // Check blocklist
        let blocklist = self.blocklist.read().await;
        for rule in blocklist.iter() {
            if self.matches_rule(rule, url, domain, resource_type) {
                // Update hit count
                drop(blocklist);
                self.update_rule(&rule.id, |r| r.hit_count += 1).await.ok();
                return Some((rule.clone(), RuleAction::Block));
            }
        }
        
        None
    }

    /// Get element hiding rules for a domain
    pub async fn get_hiding_rules(&self, domain: &str) -> Vec<String> {
        let rules = self.hiding_rules.read().await;
        rules.iter()
            .filter(|r| r.enabled && (r.domains.is_empty() || r.domains.iter().any(|d| domain.contains(d))))
            .map(|r| r.pattern.clone())
            .collect()
    }

    /// Create a rule group
    pub async fn create_group(&self, group: RuleGroup) -> Result<(), RuleError> {
        // Verify all rules exist
        let rules = self.rules.read().await;
        for rule_id in &group.rules {
            if !rules.contains_key(rule_id) {
                return Err(RuleError::NotFound(rule_id.clone()));
            }
        }
        drop(rules);
        
        self.groups.write().await.insert(group.id.clone(), group);
        
        Ok(())
    }

    /// Delete a rule group
    pub async fn delete_group(&self, group_id: &str) -> Result<(), RuleError> {
        let mut groups = self.groups.write().await;
        groups.remove(group_id)
            .ok_or_else(|| RuleError::NotFound(group_id.to_string()))?;
        
        Ok(())
    }

    /// Get all groups
    pub async fn get_groups(&self) -> Vec<RuleGroup> {
        self.groups.read().await.values().cloned().collect()
    }

    /// Get available presets
    pub fn get_presets(&self) -> &[RulePreset] {
        &self.presets
    }

    /// Install a preset
    pub async fn install_preset(&self, preset_id: &str) -> Result<usize, RuleError> {
        let preset = self.presets.iter()
            .find(|p| p.id == preset_id)
            .ok_or_else(|| RuleError::NotFound(preset_id.to_string()))?;
        
        let mut installed = 0;
        for rule in &preset.rules {
            if self.add_rule(rule.clone()).await.is_ok() {
                installed += 1;
            }
        }
        
        // Mark as installed
        if let Some(preset) = self.presets.iter_mut().find(|p| p.id == preset_id) {
            preset.installed = true;
        }
        
        Ok(installed)
    }

    /// Export rules to JSON
    pub async fn export_rules(&self) -> Result<String, RuleError> {
        let rules = self.rules.read().await;
        serde_json::to_string_pretty(&*rules)
            .map_err(|e| RuleError::ExportError(e.to_string()))
    }

    /// Import rules from JSON
    pub async fn import_rules(&self, json: &str) -> Result<usize, RuleError> {
        let imported: HashMap<String, CustomRule> = serde_json::from_str(json)
            .map_err(|e| RuleError::ImportError(e.to_string()))?;
        
        let count = imported.len();
        let mut rules = self.rules.write().await;
        rules.extend(imported);
        drop(rules);
        
        self.update_caches().await;
        
        Ok(count)
    }

    /// Find conflicts between rules
    pub async fn find_conflicts(&self) -> Vec<RuleConflict> {
        let rules = self.rules.read().await;
        let rule_list: Vec<_> = rules.values().collect();
        let mut conflicts = Vec::new();
        
        for i in 0..rule_list.len() {
            for j in (i + 1)..rule_list.len() {
                let rule_a = rule_list[i];
                let rule_b = rule_list[j];
                
                // Check for contradictions (one blocks, one allows same URL)
                if rule_a.action != rule_b.action {
                    if self.patterns_overlap(&rule_a.pattern, &rule_b.pattern) {
                        conflicts.push(RuleConflict {
                            rule_a: rule_a.id.clone(),
                            rule_b: rule_b.id.clone(),
                            conflict_type: ConflictType::Contradiction,
                            suggestion: "One rule blocks while another allows. Consider removing one.".to_string(),
                        });
                    }
                }
                
                // Check for redundancy
                if rule_a.pattern.contains(&rule_b.pattern) {
                    conflicts.push(RuleConflict {
                        rule_a: rule_a.id.clone(),
                        rule_b: rule_b.id.clone(),
                        conflict_type: ConflictType::Subset,
                        suggestion: "One rule's pattern is contained within another.".to_string(),
                    });
                }
            }
        }
        
        conflicts
    }

    /// Get rule statistics
    pub async fn get_stats(&self) -> RuleStats {
        let rules = self.rules.read().await;
        
        RuleStats {
            total_rules: rules.len(),
            enabled_rules: rules.values().filter(|r| r.enabled).count(),
            block_rules: rules.values().filter(|r| r.action == RuleAction::Block).count(),
            allow_rules: rules.values().filter(|r| r.action == RuleAction::Allow).count(),
            hide_rules: rules.values().filter(|r| r.action == RuleAction::Hide).count(),
            total_hits: rules.values().map(|r| r.hit_count).sum(),
            groups: self.groups.read().await.len(),
        }
    }

    /// Validate a rule
    fn validate_rule(&self, rule: &CustomRule) -> Result<(), RuleError> {
        if rule.name.is_empty() {
            return Err(RuleError::ValidationError("Rule name cannot be empty".to_string()));
        }
        
        if rule.pattern.is_empty() {
            return Err(RuleError::ValidationError("Rule pattern cannot be empty".to_string()));
        }
        
        // Validate regex if that type
        if rule.rule_type == CustomRuleType::Regex {
            Regex::new(&rule.pattern)
                .map_err(|e| RuleError::InvalidPattern(e.to_string()))?;
        }
        
        Ok(())
    }

    /// Check if a rule matches a URL
    fn matches_rule(&self, rule: &CustomRule, url: &str, domain: &str, resource_type: ResourceType) -> bool {
        if !rule.enabled {
            return false;
        }
        
        // Check domain restriction
        if !rule.domains.is_empty() {
            if !rule.domains.iter().any(|d| domain.contains(d)) {
                return false;
            }
        }
        
        // Check resource type
        if !rule.resource_types.is_empty() && !rule.resource_types.contains(&ResourceType::All) {
            if !rule.resource_types.contains(&resource_type) {
                return false;
            }
        }
        
        // Match based on rule type
        match rule.rule_type {
            CustomRuleType::Exact => url == rule.pattern,
            CustomRuleType::Domain => {
                let parsed = url::Url::parse(url);
                parsed.ok().and_then(|u| u.domain().map(|d| d.contains(&rule.pattern))).unwrap_or(false)
            }
            CustomRuleType::UrlPattern => url.contains(&rule.pattern),
            CustomRuleType::Regex => {
                rule.compiled_regex.as_ref()
                    .map(|r| r.is_match(url))
                    .unwrap_or(false)
            }
            _ => false,
        }
    }

    /// Check if two patterns overlap
    fn patterns_overlap(&self, a: &str, b: &str) -> bool {
        a.contains(b) || b.contains(a)
    }

    /// Update internal caches
    async fn update_caches(&self) {
        let rules = self.rules.read().await;
        
        let mut whitelist = self.whitelist.write().await;
        let mut blocklist = self.blocklist.write().await;
        let mut hiding_rules = self.hiding_rules.write().await;
        
        whitelist.clear();
        blocklist.clear();
        hiding_rules.clear();
        
        for rule in rules.values() {
            if !rule.enabled {
                continue;
            }
            
            match rule.action {
                RuleAction::Allow => whitelist.push(rule.clone()),
                RuleAction::Block => blocklist.push(rule.clone()),
                RuleAction::Hide | RuleAction::Remove => hiding_rules.push(rule.clone()),
                _ => {}
            }
        }
        
        // Sort by priority
        whitelist.sort_by(|a, b| b.priority.cmp(&a.priority));
        blocklist.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Clear all rules
    pub async fn clear_rules(&self) {
        self.rules.write().await.clear();
        self.groups.write().await.clear();
        self.update_caches().await;
    }

    /// Quick add a block rule
    pub async fn quick_block(&self, pattern: &str, name: Option<&str>) -> Result<String, RuleError> {
        let id = format!("block_{}", Utc::now().timestamp_millis());
        let rule = CustomRule {
            id: id.clone(),
            name: name.unwrap_or("Quick Block").to_string(),
            description: None,
            pattern: pattern.to_string(),
            rule_type: CustomRuleType::UrlPattern,
            action: RuleAction::Block,
            enabled: true,
            priority: 50,
            domains: vec![],
            resource_types: vec![ResourceType::All],
            created_at: Utc::now(),
            modified_at: Utc::now(),
            hit_count: 0,
            compiled_regex: None,
        };
        
        self.add_rule(rule).await?;
        Ok(id)
    }

    /// Quick add an allow rule
    pub async fn quick_allow(&self, pattern: &str, name: Option<&str>) -> Result<String, RuleError> {
        let id = format!("allow_{}", Utc::now().timestamp_millis());
        let rule = CustomRule {
            id: id.clone(),
            name: name.unwrap_or("Quick Allow").to_string(),
            description: None,
            pattern: pattern.to_string(),
            rule_type: CustomRuleType::UrlPattern,
            action: RuleAction::Allow,
            enabled: true,
            priority: 100, // Higher priority for allow rules
            domains: vec![],
            resource_types: vec![ResourceType::All],
            created_at: Utc::now(),
            modified_at: Utc::now(),
            hit_count: 0,
            compiled_regex: None,
        };
        
        self.add_rule(rule).await?;
        Ok(id)
    }
}

impl Default for RuleManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Rule statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleStats {
    pub total_rules: usize,
    pub enabled_rules: usize,
    pub block_rules: usize,
    pub allow_rules: usize,
    pub hide_rules: usize,
    pub total_hits: usize,
    pub groups: usize,
}

/// Rule errors
#[derive(Debug, thiserror::Error)]
pub enum RuleError {
    #[error("Rule not found: {0}")]
    NotFound(String),
    
    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Export error: {0}")]
    ExportError(String),
    
    #[error("Import error: {0}")]
    ImportError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_manager() {
        let manager = RuleManager::new();
        let stats = manager.get_stats().await;
        assert_eq!(stats.total_rules, 0);
    }

    #[tokio::test]
    async fn test_add_rule() {
        let manager = RuleManager::new();
        let rule = CustomRule {
            id: "test-1".to_string(),
            name: "Test Rule".to_string(),
            description: None,
            pattern: "ads.com".to_string(),
            rule_type: CustomRuleType::UrlPattern,
            action: RuleAction::Block,
            enabled: true,
            priority: 50,
            domains: vec![],
            resource_types: vec![ResourceType::All],
            created_at: Utc::now(),
            modified_at: Utc::now(),
            hit_count: 0,
            compiled_regex: None,
        };
        
        manager.add_rule(rule).await.unwrap();
        let stats = manager.get_stats().await;
        assert_eq!(stats.total_rules, 1);
    }

    #[tokio::test]
    async fn test_check_url() {
        let manager = RuleManager::new();
        manager.add_rule(CustomRule {
            id: "block-ads".to_string(),
            name: "Block Ads".to_string(),
            description: None,
            pattern: "ads.com".to_string(),
            rule_type: CustomRuleType::UrlPattern,
            action: RuleAction::Block,
            enabled: true,
            priority: 50,
            domains: vec![],
            resource_types: vec![ResourceType::All],
            created_at: Utc::now(),
            modified_at: Utc::now(),
            hit_count: 0,
            compiled_regex: None,
        }).await.unwrap();
        
        let result = manager.check_url("https://ads.com/banner.js", "example.com", ResourceType::Script).await;
        assert!(result.is_some());
        let (_, action) = result.unwrap();
        assert_eq!(action, RuleAction::Block);
    }

    #[tokio::test]
    async fn test_whitelist() {
        let manager = RuleManager::new();
        
        // Add block rule
        manager.add_rule(CustomRule {
            id: "block-1".to_string(),
            name: "Block".to_string(),
            description: None,
            pattern: "example.com".to_string(),
            rule_type: CustomRuleType::Domain,
            action: RuleAction::Block,
            enabled: true,
            priority: 50,
            domains: vec![],
            resource_types: vec![ResourceType::All],
            created_at: Utc::now(),
            modified_at: Utc::now(),
            hit_count: 0,
            compiled_regex: None,
        }).await.unwrap();
        
        // Add allow rule
        manager.add_rule(CustomRule {
            id: "allow-1".to_string(),
            name: "Allow".to_string(),
            description: None,
            pattern: "example.com".to_string(),
            rule_type: CustomRuleType::Domain,
            action: RuleAction::Allow,
            enabled: true,
            priority: 100,
            domains: vec![],
            resource_types: vec![ResourceType::All],
            created_at: Utc::now(),
            modified_at: Utc::now(),
            hit_count: 0,
            compiled_regex: None,
        }).await.unwrap();
        
        let result = manager.check_url("https://example.com/script.js", "page.com", ResourceType::Script).await;
        assert!(result.is_some());
        let (_, action) = result.unwrap();
        assert_eq!(action, RuleAction::Allow); // Allow takes priority
    }

    #[tokio::test]
    async fn test_quick_block() {
        let manager = RuleManager::new();
        let id = manager.quick_block("ads.example.com", Some("Block Example Ads")).await.unwrap();
        
        let rule = manager.get_rule(&id).await;
        assert!(rule.is_some());
    }

    #[tokio::test]
    async fn test_get_presets() {
        let manager = RuleManager::new();
        let presets = manager.get_presets();
        assert!(!presets.is_empty());
    }

    #[tokio::test]
    async fn test_install_preset() {
        let manager = RuleManager::new();
        let installed = manager.install_preset("privacy-basic").await.unwrap();
        assert!(installed > 0);
    }
}