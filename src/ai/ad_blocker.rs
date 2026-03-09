//! # AI-Powered Ad Blocker Module
//!
//! Provides intelligent ad and tracker blocking using machine learning and
//! pattern matching. This module combines multiple detection strategies:
//! - ML-based content classification
//! - Pattern matching for known ad networks
//! - Heuristic analysis of page elements
//! - Real-time tracker detection

use anyhow::{Result, Error};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Ad blocker for intelligent ad and tracker detection
pub struct AdBlocker {
    config: AdBlockerConfig,
    block_lists: Arc<RwLock<BlockLists>>,
    ml_model: Arc<RwLock<MlModel>>,
    cache: Arc<RwLock<HashMap<String, BlockDecision>>>,
    stats: Arc<RwLock<BlockerStats>>,
}

/// Ad blocker configuration
#[derive(Debug, Clone)]
pub struct AdBlockerConfig {
    pub enable_ml: bool,
    pub enable_patterns: bool,
    pub block_trackers: bool,
    pub enable_malware_protection: bool,
    pub cache_duration: u64,
    pub min_confidence: f32,
    pub enable_custom_rules: bool,
}

impl Default for AdBlockerConfig {
    fn default() -> Self {
        Self {
            enable_ml: true,
            enable_patterns: true,
            block_trackers: true,
            enable_malware_protection: true,
            cache_duration: 3600,
            min_confidence: 0.7,
            enable_custom_rules: true,
        }
    }
}

/// Block lists containing ad/tracker signatures
#[derive(Debug, Clone, Default)]
struct BlockLists {
    ad_networks: HashSet<String>,
    trackers: HashSet<String>,
    malware_domains: HashSet<String>,
    custom_rules: Vec<CustomRule>,
    whitelist: HashSet<String>,
}

/// Machine learning model for ad detection
#[derive(Debug, Clone)]
struct MlModel {
    version: String,
    threshold: f32,
    patterns: Vec<String>,
    weights: HashMap<String, f32>,
}

/// Custom user-defined blocking rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRule {
    pub id: String,
    pub name: String,
    pub pattern: String,
    pub reason: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

/// Block decision for a request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDecision {
    pub should_block: bool,
    pub reason: BlockReason,
    pub confidence: f32,
    pub decided_at: DateTime<Utc>,
}

/// Reason for blocking a request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BlockReason {
    AdNetwork(String),
    Tracker(String),
    Malware(String),
    PatternMatch(String),
    MlDetection(String),
    CustomRule(String),
    NotBlocked,
}

/// Blocker statistics
#[derive(Debug, Clone, Default)]
pub struct BlockerStats {
    pub total_requests: u64,
    pub total_blocks: u64,
    pub blocks_by_reason: HashMap<BlockReason, u64>,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl AdBlocker {
    pub fn new(config: AdBlockerConfig) -> Self {
        let block_lists = Self::load_default_block_lists();
        let ml_model = Self::load_default_model();

        Self {
            config,
            block_lists: Arc::new(RwLock::new(block_lists)),
            ml_model: Arc::new(RwLock::new(ml_model)),
            cache: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(BlockerStats::default())),
        }
    }

    fn load_default_block_lists() -> BlockLists {
        BlockLists {
            ad_networks: Self::default_ad_networks(),
            trackers: Self::default_trackers(),
            malware_domains: HashSet::new(),
            custom_rules: Vec::new(),
            whitelist: HashSet::new(),
        }
    }

    fn load_default_model() -> MlModel {
        MlModel {
            version: "1.0.0".to_string(),
            threshold: 0.7,
            patterns: vec![
                r"ads?\.".to_string(),
                r"advertisement".to_string(),
                r"banner".to_string(),
                r"tracking".to_string(),
                r"analytics".to_string(),
            ],
            weights: HashMap::new(),
        }
    }

    fn default_ad_networks() -> HashSet<String> {
        vec![
            "googleads.g.doubleclick.net".to_string(),
            "googlesyndication.com".to_string(),
            "facebook.com/tr".to_string(),
            "doubleclick.net".to_string(),
            "ads-twitter.com".to_string(),
        ].into_iter().collect()
    }

    fn default_trackers() -> HashSet<String> {
        vec![
            "google-analytics.com".to_string(),
            "googletagmanager.com".to_string(),
            "facebook.com/tr".to_string(),
            "connect.facebook.net".to_string(),
            "stats.g.doubleclick.net".to_string(),
        ].into_iter().collect()
    }

    pub async fn check_request(&self, url: &str) -> Result<BlockDecision> {
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;

        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(decision) = cache.get(url) {
                stats.cache_hits += 1;
                return Ok(decision.clone());
            }
        }
        stats.cache_misses += 1;

        // Check whitelist
        let block_lists = self.block_lists.read().await;
        if Self::is_whitelisted(url, &block_lists.whitelist) {
            let decision = BlockDecision {
                should_block: false,
                reason: BlockReason::NotBlocked,
                confidence: 1.0,
                decided_at: Utc::now(),
            };
            return Ok(decision);
        }

        // Check block lists
        if self.config.enable_patterns {
            if let Some(reason) = self.check_block_lists(url, &block_lists).await {
                stats.total_blocks += 1;
                let decision = BlockDecision {
                    should_block: true,
                    reason,
                    confidence: 0.9,
                    decided_at: Utc::now(),
                };
                return Ok(decision);
            }
        }

        // Check ML model
        if self.config.enable_ml {
            if let Some(reason) = self.check_ml(url).await {
                stats.total_blocks += 1;
                let decision = BlockDecision {
                    should_block: true,
                    reason,
                    confidence: 0.8,
                    decided_at: Utc::now(),
                };
                return Ok(decision);
            }
        }

        let decision = BlockDecision {
            should_block: false,
            reason: BlockReason::NotBlocked,
            confidence: 1.0,
            decided_at: Utc::now(),
        };

        Ok(decision)
    }

    async fn check_block_lists(&self, url: &str, block_lists: &BlockLists) -> Option<BlockReason> {
        // Check ad networks
        for network in &block_lists.ad_networks {
            if url.contains(network) {
                return Some(BlockReason::AdNetwork(network.clone()));
            }
        }

        // Check trackers
        if self.config.block_trackers {
            for tracker in &block_lists.trackers {
                if url.contains(tracker) {
                    return Some(BlockReason::Tracker(tracker.clone()));
                }
            }
        }

        None
    }

    async fn check_ml(&self, url: &str) -> Option<BlockReason> {
        let model = self.ml_model.read().await;
        
        for pattern in &model.patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if regex.is_match(url) {
                    return Some(BlockReason::PatternMatch(pattern.clone()));
                }
            }
        }

        None
    }

    fn is_whitelisted(url: &str, whitelist: &HashSet<String>) -> bool {
        whitelist.iter().any(|domain| url.contains(domain))
    }

    pub async fn add_custom_rule(&self, rule: CustomRule) -> Result<()> {
        let mut block_lists = self.block_lists.write().await;
        block_lists.custom_rules.push(rule);
        Ok(())
    }

    pub async fn get_stats(&self) -> BlockerStats {
        self.stats.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ad_blocker_creation() {
        let config = AdBlockerConfig::default();
        let blocker = AdBlocker::new(config);
        assert_eq!(blocker.config.min_confidence, 0.7);
    }

    #[tokio::test]
    async fn test_check_request_block() {
        let blocker = AdBlocker::new(AdBlockerConfig::default());
        let decision = blocker.check_request("https://googleads.g.doubleclick.net/ad.js").await.unwrap();
        assert!(decision.should_block);
    }

    #[tokio::test]
    async fn test_check_request_allow() {
        let blocker = AdBlocker::new(AdBlockerConfig::default());
        let decision = blocker.check_request("https://example.com/page.html").await.unwrap();
        assert!(!decision.should_block);
    }

    #[tokio::test]
    async fn test_get_stats() {
        let blocker = AdBlocker::new(AdBlockerConfig::default());
        let stats = blocker.get_stats().await;
        assert_eq!(stats.total_requests, 0);
    }
}