//! AI Module (AI Nexus)
//! 
//! Artificial Intelligence features:
//! - NPU Acceleration
//! - Predictive Branching
//! - Live Dubbing
//! - Neural Memory
//! - Vantis Cluster
//! - Profile Recommendations
//! - Usage Pattern Analysis
//! - Smart Suggestions
//! - AI-Powered Ad Blocking

pub mod recommendations;
pub mod pattern_analyzer;
pub mod suggestions;
pub mod ad_blocker;

pub use recommendations::{RecommendationEngine, Recommendation, RecommendationType, RecommendationPriority};
pub use pattern_analyzer::{PatternAnalyzer, UsagePattern, PatternType, PatternConfidence};
pub use suggestions::{SuggestionEngine, Suggestion, SuggestionCategory, SuggestionAction};
pub use ad_blocker::{AdBlocker, AdBlockerConfig, BlockDecision, BlockReason, CustomRule};