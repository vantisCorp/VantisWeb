//! AI-Powered Ad Blocking Module
//! 
//! This module provides intelligent ad and tracker blocking using machine learning
//! and heuristic detection methods.
//! 
//! # Features
//! - ML-based ad detection using TensorFlow
//! - Customizable block lists and rules
//! - Tracker detection and blocking
//! - Malware and phishing protection
//! - User interface for rule management
//! - Performance-optimized blocking

pub mod detector;
pub mod blocklist;
pub mod tracker;
pub mod malware;
pub mod rules;
pub mod ui;
pub mod stats;

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur in ad blocking operations
#[derive(Error, Debug)]
pub enum AdBlockError {
    #[error("Detection failed: {0}")]
    DetectionFailed(String),
    #[error("Block list error: {0}")]
    BlockListError(String),
    #[error("Rule error: {0}")]
    RuleError(String),
    #[error("ML model error: {0}")]
    ModelError(String),
}

/// Result type for ad blocking operations
pub type Result<T> = std::result::Result<T, AdBlockError>;

/// Block action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockAction {
    Block,
    Allow,
    Redirect,
    Modify,
}

/// Resource type for blocking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    Advertisement,
    Tracker,
    Malware,
    Phishing,
    SocialMediaWidget,
    CommentSection,
    Annoyance,
    CookieNotice,
    Popup,
    Other,
}

/// Block reason
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockReason {
    pub resource_type: ResourceType,
    pub rule_id: String,
    pub confidence: f32,
    pub description: String,
}

/// Block result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockResult {
    pub url: String,
    pub should_block: bool,
    pub action: BlockAction,
    pub reason: Option<BlockReason>,
    pub redirect_url: Option<String>,
}

/// Ad blocking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdBlockConfig {
    pub enabled: bool,
    pub block_ads: bool,
    pub block_trackers: bool,
    pub block_malware: bool,
    pub block_popups: bool,
    pub enable_ml_detection: bool,
    pub custom_rules_enabled: bool,
    pub update_frequency: UpdateFrequency,
}

/// Update frequency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateFrequency {
    Hourly,
    Daily,
    Weekly,
    Monthly,
}

impl Default for AdBlockConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            block_ads: true,
            block_trackers: true,
            block_malware: true,
            block_popups: true,
            enable_ml_detection: true,
            custom_rules_enabled: true,
            update_frequency: UpdateFrequency::Daily,
        }
    }
}

/// Ad blocking manager
pub struct AdBlockManager {
    config: Arc<RwLock<AdBlockConfig>>,
    detector: Arc<detector::AdDetector>,
    blocklist: Arc<blocklist::BlockListManager>,
    tracker: Arc<tracker::TrackerDetector>,
    malware: Arc<malware::MalwareDetector>,
    rules: Arc<rules::RuleManager>,
    stats: Arc<stats::BlockStatistics>,
}

impl AdBlockManager {
    /// Create a new ad block manager
    pub fn new(config: AdBlockConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            detector: Arc::new(detector::AdDetector::new()),
            blocklist: Arc::new(blocklist::BlockListManager::new()),
            tracker: Arc::new(tracker::TrackerDetector::new()),
            malware: Arc::new(malware::MalwareDetector::new()),
            rules: Arc::new(rules::RuleManager::new()),
            stats: Arc::new(stats::BlockStatistics::new()),
        }
    }

    /// Check if a URL should be blocked
    pub async fn should_block(&self, url: &str) -> Result<BlockResult> {
        let config = self.config.read().await;

        if !config.enabled {
            return Ok(BlockResult {
                url: url.to_string(),
                should_block: false,
                action: BlockAction::Allow,
                reason: None,
                redirect_url: None,
            });
        }

        // Check malware first (highest priority)
        if config.block_malware {
            if let Some(reason) = self.malware.check_url(url).await? {
                self.stats.record_block(ResourceType::Malware).await;
                return Ok(BlockResult {
                    url: url.to_string(),
                    should_block: true,
                    action: BlockAction::Block,
                    reason: Some(reason),
                    redirect_url: None,
                });
            }
        }

        // Check trackers
        if config.block_trackers {
            if let Some(reason) = self.tracker.check_url(url).await? {
                self.stats.record_block(ResourceType::Tracker).await;
                return Ok(BlockResult {
                    url: url.to_string(),
                    should_block: true,
                    action: BlockAction::Block,
                    reason: Some(reason),
                    redirect_url: None,
                });
            }
        }

        // Check block lists
        if let Some(reason) = self.blocklist.check_url(url).await? {
            self.stats.record_block(ResourceType::Advertisement).await;
            return Ok(BlockResult {
                url: url.to_string(),
                should_block: true,
                action: BlockAction::Block,
                reason: Some(reason),
                redirect_url: None,
            });
        }

        // Check custom rules
        if config.custom_rules_enabled {
            if let Some(reason) = self.rules.check_url(url).await? {
                self.stats.record_block(reason.resource_type).await;
                return Ok(BlockResult {
                    url: url.to_string(),
                    should_block: true,
                    action: BlockAction::Block,
                    reason: Some(reason),
                    redirect_url: None,
                });
            }
        }

        // ML-based detection
        if config.enable_ml_detection {
            if let Some(reason) = self.detector.detect_ad(url).await? {
                self.stats.record_block(ResourceType::Advertisement).await;
                return Ok(BlockResult {
                    url: url.to_string(),
                    should_block: true,
                    action: BlockAction::Block,
                    reason: Some(reason),
                    redirect_url: None,
                });
            }
        }

        self.stats.record_allow().await;

        Ok(BlockResult {
            url: url.to_string(),
            should_block: false,
            action: BlockAction::Allow,
            reason: None,
            redirect_url: None,
        })
    }

    /// Enable or disable ad blocking
    pub async fn set_enabled(&self, enabled: bool) {
        let mut config = self.config.write().await;
        config.enabled = enabled;
    }

    /// Update configuration
    pub async fn update_config(&self, config: AdBlockConfig) {
        *self.config.write().await = config;
    }

    /// Get current configuration
    pub async fn get_config(&self) -> AdBlockConfig {
        self.config.read().await.clone()
    }

    /// Get blocking statistics
    pub async fn get_statistics(&self) -> stats::Statistics {
        self.stats.get().await
    }

    /// Reset statistics
    pub async fn reset_statistics(&self) {
        self.stats.reset().await;
    }

    /// Update block lists
    pub async fn update_block_lists(&self) -> Result<()> {
        self.blocklist.update().await
    }

    /// Add custom rule
    pub async fn add_rule(&self, rule: rules::CustomRule) -> Result<()> {
        self.rules.add_rule(rule).await
    }

    /// Remove custom rule
    pub async fn remove_rule(&self, rule_id: &str) -> Result<()> {
        self.rules.remove_rule(rule_id).await
    }

    /// Get all custom rules
    pub async fn get_rules(&self) -> Vec<rules::CustomRule> {
        self.rules.get_rules().await
    }

    /// Import rules from file
    pub async fn import_rules(&self, content: &str) -> Result<usize> {
        self.rules.import(content).await
    }

    /// Export rules to file
    pub async fn export_rules(&self) -> Result<String> {
        self.rules.export().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_action_serialization() {
        let action = BlockAction::Block;
        let serialized = serde_json::to_string(&action).unwrap();
        let deserialized: BlockAction = serde_json::from_str(&serialized).unwrap();
        assert_eq!(action, deserialized);
    }

    #[test]
    fn test_config_default() {
        let config = AdBlockConfig::default();
        assert!(config.enabled);
        assert!(config.block_ads);
        assert!(config.block_trackers);
    }
}