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
pub mod npu_acceleration;
pub mod predictive_branching;
pub mod neural_memory;

pub use recommendations::{RecommendationEngine, Recommendation, RecommendationType, RecommendationPriority};
pub use pattern_analyzer::{PatternAnalyzer, UsagePattern, PatternType, PatternConfidence};
pub use suggestions::{SuggestionEngine, Suggestion, SuggestionCategory, SuggestionAction};
pub use ad_blocker::{AdBlocker, AdBlockerConfig, BlockDecision, BlockReason, CustomRule};
pub use npu_acceleration::{NpuAccelerator, NpuBackend, NpuDevice, NpuConfig, InferenceResult, ModelMetadata, NpuStats};
pub use predictive_branching::{PredictiveBranching, PredictiveConfig, PredictedNavigation, NavigationEvent, NavigationType, PredictionReason, PredictiveStats};
pub use neural_memory::{NeuralMemory, NeuralMemoryConfig, MemoryEntry, MemoryQuery, MemorySearchResult, BrowsingContext, BehaviorPattern, SmartSuggestion};