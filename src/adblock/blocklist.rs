//! Block List Manager for VantisWeb Ad Blocker
//! 
//! This module manages block lists from multiple sources including:
//! - EasyList/EasyPrivacy
//! - Fanboy's Annoyance List
//! - Custom user lists
//! - Auto-updating lists from remote URLs

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use regex::Regex;
use chrono::{DateTime, Utc};

/// A block list entry containing filter rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockList {
    /// Unique identifier for this list
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// List source URL (if remote)
    pub source_url: Option<String>,
    /// List type/category
    pub list_type: BlockListType,
    /// Filter rules in this list
    pub rules: Vec<FilterRule>,
    /// When the list was last updated
    pub last_updated: DateTime<Utc>,
    /// Whether the list is enabled
    pub enabled: bool,
    /// Number of rules (cached for performance)
    pub rule_count: usize,
}

/// Types of block lists
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockListType {
    /// General advertising block list
    Advertising,
    /// Privacy/tracking protection
    Privacy,
    /// Social media widgets and buttons
    Social,
    /// Annoyances (cookie notices, newsletters)
    Annoyances,
    /// Malware/phishing protection
    Malware,
    /// Custom user-defined list
    Custom,
    /// Regional/country-specific
    Regional(String),
}

/// A single filter rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterRule {
    /// The raw rule string
    pub raw: String,
    /// Parsed rule type
    pub rule_type: RuleType,
    /// Pattern to match
    pub pattern: String,
    /// Domains this rule applies to (optional)
    pub domains: Option<Vec<String>>,
    /// Whether this is an exception rule (starts with @@)
    pub is_exception: bool,
    /// Rule options
    pub options: Vec<RuleOption>,
    /// Compiled regex (lazy)
    #[serde(skip)]
    pub compiled_regex: Option<Regex>,
}

/// Types of filter rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleType {
    /// URL pattern match
    Pattern,
    /// Domain blocking
    Domain,
    /// Element hiding (CSS selector)
    ElementHide,
    /// Extended CSS selector
    ExtendedCss,
    /// Scriptlet injection
    Scriptlet,
    /// Redirect rule
    Redirect,
    /// Remove parameter
    RemoveParam,
}

/// Rule options for fine-grained control
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleOption {
    /// Apply to third-party requests only
    ThirdParty,
    /// Apply to first-party requests only
    FirstParty,
    /// Apply to specific resource types
    ResourceType(ResourceType),
    /// Important rule (cannot be disabled by exception)
    Important,
    /// Badfilter (disables other rules)
    Badfilter,
}

/// Resource types for filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
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
    Other,
}

/// Block list manager handles loading, updating, and querying filter lists
pub struct BlockListManager {
    /// Active block lists
    lists: Arc<RwLock<HashMap<String, BlockList>>>,
    /// Fast lookup: domain -> blocked
    domain_cache: Arc<RwLock<HashSet<String>>>,
    /// Fast lookup: pattern -> compiled regex
    pattern_cache: Arc<RwLock<Vec<FilterRule>>>,
    /// Exception rules (whitelist)
    exception_cache: Arc<RwLock<Vec<FilterRule>>>,
    /// Update interval in hours
    update_interval_hours: u64,
    /// Auto-update enabled
    auto_update: bool,
}

impl BlockListManager {
    /// Create a new block list manager
    pub fn new() -> Self {
        Self {
            lists: Arc::new(RwLock::new(HashMap::new())),
            domain_cache: Arc::new(RwLock::new(HashSet::new())),
            pattern_cache: Arc::new(RwLock::new(Vec::new())),
            exception_cache: Arc::new(RwLock::new(Vec::new())),
            update_interval_hours: 24,
            auto_update: true,
        }
    }

    /// Load default block lists
    pub async fn load_defaults(&self) -> Result<(), BlockListError> {
        // EasyList (advertising)
        self.add_list(BlockList {
            id: "easylist".to_string(),
            name: "EasyList".to_string(),
            source_url: Some("https://easylist-downloads.adblockplus.org/easylist.txt".to_string()),
            list_type: BlockListType::Advertising,
            rules: vec![], // Would be loaded from URL
            last_updated: Utc::now(),
            enabled: true,
            rule_count: 0,
        }).await?;

        // EasyPrivacy
        self.add_list(BlockList {
            id: "easyprivacy".to_string(),
            name: "EasyPrivacy".to_string(),
            source_url: Some("https://easylist-downloads.adblockplus.org/easyprivacy.txt".to_string()),
            list_type: BlockListType::Privacy,
            rules: vec![],
            last_updated: Utc::now(),
            enabled: true,
            rule_count: 0,
        }).await?;

        // Fanboy's Annoyance List
        self.add_list(BlockList {
            id: "fanboy-annoyance".to_string(),
            name: "Fanboy's Annoyance List".to_string(),
            source_url: Some("https://easylist-downloads.adblockplus.org/fanboy-annoyance.txt".to_string()),
            list_type: BlockListType::Annoyances,
            rules: vec![],
            last_updated: Utc::now(),
            enabled: true,
            rule_count: 0,
        }).await?;

        Ok(())
    }

    /// Add a block list
    pub async fn add_list(&self, list: BlockList) -> Result<(), BlockListError> {
        let mut lists = self.lists.write().await;
        
        // If list has source URL, fetch and parse rules
        let mut list = list;
        if let Some(url) = &list.source_url {
            let rules = self.fetch_list(url).await?;
            list.rules = rules;
            list.rule_count = list.rules.len();
        }
        
        // Rebuild caches
        lists.insert(list.id.clone(), list);
        drop(lists);
        
        self.rebuild_caches().await;
        
        Ok(())
    }

    /// Remove a block list
    pub async fn remove_list(&self, list_id: &str) -> Result<(), BlockListError> {
        let mut lists = self.lists.write().await;
        lists.remove(list_id);
        drop(lists);
        
        self.rebuild_caches().await;
        
        Ok(())
    }

    /// Enable or disable a list
    pub async fn toggle_list(&self, list_id: &str, enabled: bool) -> Result<(), BlockListError> {
        let mut lists = self.lists.write().await;
        if let Some(list) = lists.get_mut(list_id) {
            list.enabled = enabled;
        }
        drop(lists);
        
        self.rebuild_caches().await;
        
        Ok(())
    }

    /// Check if a URL should be blocked
    pub async fn should_block(&self, url: &str, source_domain: &str, resource_type: ResourceType) -> Option<FilterRule> {
        // First check exceptions
        let exceptions = self.exception_cache.read().await;
        for rule in exceptions.iter() {
            if self.matches_rule(url, source_domain, resource_type, rule) {
                return None; // Exception found, don't block
            }
        }
        drop(exceptions);
        
        // Check domain cache for quick lookup
        let domain_cache = self.domain_cache.read().await;
        if let Ok(parsed) = url::Url::parse(url) {
            if let Some(domain) = parsed.domain() {
                if domain_cache.contains(domain) {
                    return Some(FilterRule {
                        raw: format!("||{}^", domain),
                        rule_type: RuleType::Domain,
                        pattern: domain.to_string(),
                        domains: None,
                        is_exception: false,
                        options: vec![],
                        compiled_regex: None,
                    });
                }
            }
        }
        drop(domain_cache);
        
        // Check pattern rules
        let patterns = self.pattern_cache.read().await;
        for rule in patterns.iter() {
            if self.matches_rule(url, source_domain, resource_type, rule) {
                return Some(rule.clone());
            }
        }
        
        None
    }

    /// Match URL against a filter rule
    fn matches_rule(&self, url: &str, source_domain: &str, resource_type: ResourceType, rule: &FilterRule) -> bool {
        // Check domain restrictions
        if let Some(domains) = &rule.domains {
            let matches_domain = domains.iter().any(|d| {
                source_domain.ends_with(d.trim_start_matches('~'))
            });
            
            // If domain starts with ~, it's an exclusion
            for d in domains {
                if d.starts_with('~') && source_domain.ends_with(&d[1..]) {
                    return false;
                }
            }
            
            if !matches_domain {
                return false;
            }
        }
        
        // Check resource type
        let type_match = rule.options.iter().any(|opt| {
            matches!(opt, RuleOption::ResourceType(rt) if *rt == resource_type)
        });
        
        if rule.options.iter().any(|opt| matches!(opt, RuleOption::ResourceType(_))) && !type_match {
            return false;
        }
        
        // Check pattern
        match &rule.compiled_regex {
            Some(regex) => regex.is_match(url),
            None => url.contains(&rule.pattern),
        }
    }

    /// Fetch a block list from a URL
    async fn fetch_list(&self, url: &str) -> Result<Vec<FilterRule>, BlockListError> {
        // Simulated fetch - in production would use HTTP client
        // For now, return some sample rules
        Ok(vec![
            FilterRule {
                raw: "||googleads.com^".to_string(),
                rule_type: RuleType::Domain,
                pattern: "googleads.com".to_string(),
                domains: None,
                is_exception: false,
                options: vec![],
                compiled_regex: None,
            },
            FilterRule {
                raw: "||doubleclick.net^".to_string(),
                rule_type: RuleType::Domain,
                pattern: "doubleclick.net".to_string(),
                domains: None,
                is_exception: false,
                options: vec![],
                compiled_regex: None,
            },
        ])
    }

    /// Parse Adblock Plus filter rule format
    pub fn parse_rule(&self, line: &str) -> Option<FilterRule> {
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('!') || line.starts_with('[') {
            return None;
        }
        
        let is_exception = line.starts_with("@@");
        let rule_str = if is_exception { &line[2..] } else { line };
        
        // Parse options
        let (pattern, options) = if let Some(pos) = rule_str.find('$') {
            let (p, opts) = rule_str.split_at(pos);
            (p, Some(&opts[1..]))
        } else {
            (rule_str, None)
        };
        
        // Determine rule type
        let rule_type = if pattern.starts_with("##") || pattern.starts_with("#@#") {
            RuleType::ElementHide
        } else if pattern.starts_with("#?#") {
            RuleType::ExtendedCss
        } else if pattern.starts_with("||") {
            RuleType::Domain
        } else if pattern.starts_with("|") {
            RuleType::Pattern
        } else {
            RuleType::Pattern
        };
        
        // Parse options
        let parsed_options = options.map(|o| self.parse_options(o)).unwrap_or_default();
        
        // Extract domains from pattern
        let domains = if pattern.contains('~') || pattern.contains(',') {
            Some(pattern.split('|').filter(|d| !d.is_empty()).map(String::from).collect())
        } else {
            None
        };
        
        // Clean pattern
        let clean_pattern = pattern
            .trim_start_matches('|')
            .trim_end_matches('^')
            .trim_start_matches("||")
            .to_string();
        
        Some(FilterRule {
            raw: line.to_string(),
            rule_type,
            pattern: clean_pattern,
            domains,
            is_exception,
            options: parsed_options,
            compiled_regex: None,
        })
    }

    /// Parse rule options
    fn parse_options(&self, opts: &str) -> Vec<RuleOption> {
        opts.split(',')
            .filter_map(|opt| {
                let opt = opt.trim();
                match opt {
                    "third-party" | "3p" => Some(RuleOption::ThirdParty),
                    "first-party" | "1p" => Some(RuleOption::FirstParty),
                    "script" => Some(RuleOption::ResourceType(ResourceType::Script)),
                    "image" => Some(RuleOption::ResourceType(ResourceType::Image)),
                    "stylesheet" | "css" => Some(RuleOption::ResourceType(ResourceType::Stylesheet)),
                    "object" => Some(RuleOption::ResourceType(ResourceType::Object)),
                    "xmlhttprequest" | "xhr" => Some(RuleOption::ResourceType(ResourceType::XmlHttpRequest)),
                    "subdocument" | "frame" => Some(RuleOption::ResourceType(ResourceType::SubDocument)),
                    "document" => Some(RuleOption::ResourceType(ResourceType::Document)),
                    "font" => Some(RuleOption::ResourceType(ResourceType::Font)),
                    "media" => Some(RuleOption::ResourceType(ResourceType::Media)),
                    "websocket" => Some(RuleOption::ResourceType(ResourceType::WebSocket)),
                    "important" => Some(RuleOption::Important),
                    "badfilter" => Some(RuleOption::Badfilter),
                    _ => None,
                }
            })
            .collect()
    }

    /// Rebuild internal caches for fast lookups
    async fn rebuild_caches(&self) {
        let lists = self.lists.read().await;
        
        let mut domain_cache = self.domain_cache.write().await;
        let mut pattern_cache = self.pattern_cache.write().await;
        let mut exception_cache = self.exception_cache.write().await;
        
        domain_cache.clear();
        pattern_cache.clear();
        exception_cache.clear();
        
        for list in lists.values() {
            if !list.enabled {
                continue;
            }
            
            for rule in &list.rules {
                if rule.is_exception {
                    exception_cache.push(rule.clone());
                } else {
                    match rule.rule_type {
                        RuleType::Domain => {
                            domain_cache.insert(rule.pattern.clone());
                        }
                        _ => {
                            pattern_cache.push(rule.clone());
                        }
                    }
                }
            }
        }
    }

    /// Update all lists from their sources
    pub async fn update_lists(&self) -> Result<usize, BlockListError> {
        let mut updated = 0;
        let lists = self.lists.read().await;
        
        for list in lists.values() {
            if let Some(url) = &list.source_url {
                if let Ok(rules) = self.fetch_list(url).await {
                    drop(lists);
                    
                    let mut lists = self.lists.write().await;
                    if let Some(list) = lists.get_mut(&list.id) {
                        list.rules = rules;
                        list.rule_count = list.rules.len();
                        list.last_updated = Utc::now();
                        updated += 1;
                    }
                    
                    self.rebuild_caches().await;
                    return Ok(updated);
                }
            }
        }
        
        Ok(updated)
    }

    /// Get list statistics
    pub async fn get_stats(&self) -> BlockListStats {
        let lists = self.lists.read().await;
        
        let total_rules: usize = lists.values()
            .filter(|l| l.enabled)
            .map(|l| l.rule_count)
            .sum();
        
        BlockListStats {
            total_lists: lists.len(),
            enabled_lists: lists.values().filter(|l| l.enabled).count(),
            total_rules,
            domain_rules: self.domain_cache.read().await.len(),
            pattern_rules: self.pattern_cache.read().await.len(),
            exception_rules: self.exception_cache.read().await.len(),
        }
    }

    /// Export lists to JSON
    pub async fn export_lists(&self) -> Result<String, BlockListError> {
        let lists = self.lists.read().await;
        serde_json::to_string_pretty(&*lists)
            .map_err(|e| BlockListError::ExportError(e.to_string()))
    }

    /// Import lists from JSON
    pub async fn import_lists(&self, json: &str) -> Result<usize, BlockListError> {
        let imported: HashMap<String, BlockList> = serde_json::from_str(json)
            .map_err(|e| BlockListError::ImportError(e.to_string()))?;
        
        let count = imported.len();
        let mut lists = self.lists.write().await;
        lists.extend(imported);
        drop(lists);
        
        self.rebuild_caches().await;
        
        Ok(count)
    }
}

impl Default for BlockListManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Block list statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockListStats {
    pub total_lists: usize,
    pub enabled_lists: usize,
    pub total_rules: usize,
    pub domain_rules: usize,
    pub pattern_rules: usize,
    pub exception_rules: usize,
}

/// Block list errors
#[derive(Debug, thiserror::Error)]
pub enum BlockListError {
    #[error("Failed to fetch list: {0}")]
    FetchError(String),
    
    #[error("Failed to parse rule: {0}")]
    ParseError(String),
    
    #[error("List not found: {0}")]
    NotFound(String),
    
    #[error("Export error: {0}")]
    ExportError(String),
    
    #[error("Import error: {0}")]
    ImportError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_manager() {
        let manager = BlockListManager::new();
        let stats = manager.get_stats().await;
        assert_eq!(stats.total_lists, 0);
    }

    #[tokio::test]
    async fn test_add_list() {
        let manager = BlockListManager::new();
        let list = BlockList {
            id: "test".to_string(),
            name: "Test List".to_string(),
            source_url: None,
            list_type: BlockListType::Custom,
            rules: vec![],
            last_updated: Utc::now(),
            enabled: true,
            rule_count: 0,
        };
        
        manager.add_list(list).await.unwrap();
        let stats = manager.get_stats().await;
        assert_eq!(stats.total_lists, 1);
    }

    #[test]
    fn test_parse_simple_rule() {
        let manager = BlockListManager::new();
        let rule = manager.parse_rule("||example.com^");
        
        assert!(rule.is_some());
        let rule = rule.unwrap();
        assert_eq!(rule.rule_type, RuleType::Domain);
        assert_eq!(rule.pattern, "example.com");
        assert!(!rule.is_exception);
    }

    #[test]
    fn test_parse_exception_rule() {
        let manager = BlockListManager::new();
        let rule = manager.parse_rule("@@||example.com^");
        
        assert!(rule.is_some());
        let rule = rule.unwrap();
        assert!(rule.is_exception);
    }

    #[test]
    fn test_parse_rule_with_options() {
        let manager = BlockListManager::new();
        let rule = manager.parse_rule("||ads.com^$script,third-party");
        
        assert!(rule.is_some());
        let rule = manager.parse_rule("||ads.com^$script,third-party").unwrap();
        assert!(rule.options.contains(&RuleOption::ThirdParty));
        assert!(rule.options.contains(&RuleOption::ResourceType(ResourceType::Script)));
    }

    #[test]
    fn test_parse_element_hiding() {
        let manager = BlockListManager::new();
        let rule = manager.parse_rule("##.ad-banner");
        
        assert!(rule.is_some());
        let rule = rule.unwrap();
        assert_eq!(rule.rule_type, RuleType::ElementHide);
    }
}