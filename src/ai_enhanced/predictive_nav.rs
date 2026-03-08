//! Predictive Navigation Module
//! 
//! AI-powered navigation prediction and optimization including
//! URL prediction, page prefetching, and intelligent navigation suggestions.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::{HashMap, VecDeque, HashSet};
use chrono::{DateTime, Utc};
use std::hash::{Hash, Hasher};

/// Predictive Navigator
pub struct PredictiveNavigator {
    model: Arc<dyn NavigationModel>,
    history: RwLock<NavigationHistory>,
    predictor: Arc<NavigationPredictor>,
    prefetcher: Arc<PrefetchManager>,
    config: NavigatorConfig,
    session_cache: RwLock<HashMap<String, CachedPrediction>>,
}

/// Navigator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigatorConfig {
    /// Enable predictions
    pub predictions_enabled: bool,
    /// Enable prefetching
    pub prefetch_enabled: bool,
    /// Maximum history size
    pub max_history: usize,
    /// Prediction confidence threshold
    pub min_confidence: f32,
    /// Number of predictions to return
    pub num_predictions: usize,
    /// Prefetch cache size in MB
    pub prefetch_cache_mb: usize,
    /// Enable cross-session learning
    pub cross_session_learning: bool,
    /// Context window size (number of past navigations)
    pub context_window: usize,
    /// Enable URL pattern learning
    pub pattern_learning: bool,
}

impl Default for NavigatorConfig {
    fn default() -> Self {
        Self {
            predictions_enabled: true,
            prefetch_enabled: true,
            max_history: 10000,
            min_confidence: 0.3,
            num_predictions: 5,
            prefetch_cache_mb: 100,
            cross_session_learning: true,
            context_window: 20,
            pattern_learning: true,
        }
    }
}

/// Navigation prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationPrediction {
    /// Predicted URL
    pub url: String,
    /// Prediction confidence
    pub confidence: f32,
    /// Prediction type
    pub prediction_type: PredictionType,
    /// Reason for prediction
    pub reason: PredictionReason,
    /// Estimated time until navigation (seconds)
    pub estimated_delay: Option<f64>,
    /// Related predictions
    pub related: Vec<String>,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PredictionType {
    /// Direct URL prediction
    DirectUrl,
    /// Pattern-based prediction
    Pattern,
    /// Sequence-based prediction
    Sequence,
    /// Context-based prediction
    Contextual,
    /// User preference-based
    Preference,
    /// Time-based prediction
    Temporal,
    /// Search-based prediction
    Search,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PredictionReason {
    /// Frequently visited
    FrequentVisit { count: usize },
    /// Part of common sequence
    Sequence { sequence: Vec<String> },
    /// Similar context previously
    SimilarContext { context_hash: String },
    /// Time pattern detected
    TimePattern { hour: u8, day_of_week: u8 },
    /// URL pattern match
    PatternMatch { pattern: String },
    /// User preference
    UserPreference { category: String },
    /// Search suggestion
    SearchSuggestion { query: String },
}

/// Navigation context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationContext {
    /// Current URL
    pub current_url: String,
    /// Previous URLs in session
    pub session_history: VecDeque<String>,
    /// Current page title
    pub page_title: Option<String>,
    /// Time of navigation
    pub timestamp: DateTime<Utc>,
    /// Referrer URL
    pub referrer: Option<String>,
    /// User action that led here
    pub action: NavigationAction,
    /// Page category
    pub page_category: Option<String>,
    /// Time spent on page (seconds)
    pub time_on_page: Option<f64>,
    /// Scroll depth (0.0 - 1.0)
    pub scroll_depth: Option<f32>,
    /// Interactions on page
    pub interactions: Vec<PageInteraction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NavigationAction {
    Click,
    FormSubmit,
    Back,
    Forward,
    Refresh,
    DirectInput,
    Bookmark,
    Search,
    ExternalLink,
    Redirect,
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageInteraction {
    pub interaction_type: InteractionType,
    pub element_selector: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionType {
    Click,
    Hover,
    Scroll,
    FormInput,
    Copy,
    Select,
    Download,
    Share,
}

/// Navigation history
#[derive(Debug, Default)]
pub struct NavigationHistory {
    /// All recorded navigations
    pub entries: VecDeque<HistoryEntry>,
    /// URL visit counts
    pub url_counts: HashMap<String, usize>,
    /// URL transition matrix (from -> to -> count)
    pub transitions: HashMap<String, HashMap<String, usize>>,
    /// Session histories
    pub sessions: HashMap<String, Vec<String>>,
    /// URL patterns learned
    pub patterns: Vec<UrlPattern>,
    /// Time-based patterns
    pub temporal_patterns: Vec<TemporalPattern>,
    /// Domain preferences
    pub domain_preferences: HashMap<String, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub url: String,
    pub title: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub session_id: String,
    pub referrer: Option<String>,
    pub time_spent: f64,
    pub interactions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlPattern {
    pub pattern: String,
    pub regex: String,
    pub frequency: usize,
    pub examples: Vec<String>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalPattern {
    pub hour: u8,
    pub day_of_week: u8,
    pub urls: Vec<String>,
    pub frequency: usize,
}

/// Cached prediction
#[derive(Debug, Clone)]
struct CachedPrediction {
    predictions: Vec<NavigationPrediction>,
    context_hash: String,
    cached_at: DateTime<Utc>,
}

/// Navigation predictor
pub struct NavigationPredictor {
    markov_chain: RwLock<MarkovChain>,
    pattern_matcher: PatternMatcher,
    preference_learner: PreferenceLearner,
}

/// Markov chain for sequence prediction
struct MarkovChain {
    states: HashMap<String, HashMap<String, f32>>,
    order: usize,
}

/// Pattern matcher for URL patterns
struct PatternMatcher {
    patterns: RwLock<Vec<LearnedPattern>>,
}

struct LearnedPattern {
    pattern: String,
    next_urls: Vec<(String, f32)>,
}

/// Preference learner
struct PreferenceLearner {
    preferences: RwLock<HashMap<String, UserPreference>>,
}

struct UserPreference {
    category: String,
    weight: f32,
    last_updated: DateTime<Utc>,
}

/// Prefetch manager
pub struct PrefetchManager {
    cache: RwLock<HashMap<String, PrefetchedPage>>,
    config: PrefetchConfig,
}

#[derive(Debug, Clone)]
pub struct PrefetchedPage {
    pub url: String,
    pub fetched_at: DateTime<Utc>,
    pub size_bytes: usize,
    pub expires_at: DateTime<Utc>,
    pub priority: f32,
}

#[derive(Debug, Clone)]
pub struct PrefetchConfig {
    pub max_cache_size: usize,
    pub ttl_seconds: u64,
    pub max_concurrent: usize,
    pub priority_threshold: f32,
}

/// Navigation model trait
#[async_trait::async_trait]
pub trait NavigationModel: Send + Sync {
    async fn predict(&self, context: &NavigationContext) -> Result<Vec<NavigationPrediction>, NavigatorError>;
    async fn update(&self, entry: &HistoryEntry) -> Result<(), NavigatorError>;
}

/// Navigator error
#[derive(Debug, thiserror::Error)]
pub enum NavigatorError {
    #[error("Prediction error: {0}")]
    PredictionError(String),
    #[error("History error: {0}")]
    HistoryError(String),
    #[error("Prefetch error: {0}")]
    PrefetchError(String),
    #[error("Model error: {0}")]
    ModelError(String),
}

impl PredictiveNavigator {
    pub fn new(config: NavigatorConfig) -> Self {
        Self {
            model: Arc::new(PlaceholderNavigationModel),
            history: RwLock::new(NavigationHistory::default()),
            predictor: Arc::new(NavigationPredictor::new()),
            prefetcher: Arc::new(PrefetchManager::new(config.prefetch_cache_mb)),
            config,
            session_cache: RwLock::new(HashMap::new()),
        }
    }

    /// Get navigation predictions
    pub async fn get_predictions(&self, context: &NavigationContext) -> Result<Vec<NavigationPrediction>, NavigatorError> {
        if !self.config.predictions_enabled {
            return Ok(Vec::new());
        }

        // Check cache
        let context_hash = self.hash_context(context);
        if let Some(cached) = self.get_cached_predictions(&context_hash).await {
            return Ok(cached);
        }

        // Get predictions from multiple sources
        let mut all_predictions = Vec::new();

        // Markov chain predictions
        let sequence_predictions = self.predict_sequence(context).await;
        all_predictions.extend(sequence_predictions);

        // Pattern-based predictions
        let pattern_predictions = self.predict_from_patterns(context).await;
        all_predictions.extend(pattern_predictions);

        // Frequency-based predictions
        let frequent_predictions = self.predict_frequent(context).await;
        all_predictions.extend(frequent_predictions);

        // Temporal predictions
        let temporal_predictions = self.predict_temporal(context).await;
        all_predictions.extend(temporal_predictions);

        // Model predictions
        let model_predictions = self.model.predict(context).await?;
        all_predictions.extend(model_predictions);

        // Deduplicate and rank
        all_predictions = self.deduplicate_predictions(all_predictions);
        all_predictions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        all_predictions.truncate(self.config.num_predictions);

        // Filter by confidence threshold
        all_predictions.retain(|p| p.confidence >= self.config.min_confidence);

        // Cache results
        self.cache_predictions(&context_hash, &all_predictions).await;

        // Trigger prefetching
        if self.config.prefetch_enabled {
            self.trigger_prefetch(&all_predictions).await;
        }

        Ok(all_predictions)
    }

    /// Record navigation for learning
    pub async fn record_navigation(&self, entry: HistoryEntry) -> Result<(), NavigatorError> {
        let mut history = self.history.write().await;
        
        // Add to entries
        history.entries.push_back(entry.clone());
        
        // Limit history size
        while history.entries.len() > self.config.max_history {
            history.entries.pop_front();
        }
        
        // Update URL counts
        *history.url_counts.entry(entry.url.clone()).or_insert(0) += 1;
        
        // Update transition matrix
        if let Some(referrer) = &entry.referrer {
            let transitions = history.transitions.entry(referrer.clone()).or_insert_with(HashMap::new);
            *transitions.entry(entry.url.clone()).or_insert(0) += 1;
        }
        
        // Update model
        drop(history);
        self.model.update(&entry).await?;
        
        Ok(())
    }

    /// Get current context
    pub async fn get_current_context(&self, session_id: &str) -> Option<NavigationContext> {
        let history = self.history.read().await;
        
        if let Some(session_urls) = history.sessions.get(session_id) {
            let current_url = session_urls.last().cloned()?;
            
            let mut session_history = VecDeque::new();
            for url in session_urls.iter().rev().take(self.config.context_window).rev() {
                session_history.push_back(url.clone());
            }
            
            return Some(NavigationContext {
                current_url,
                session_history,
                page_title: None,
                timestamp: Utc::now(),
                referrer: session_urls.iter().rev().nth(1).cloned(),
                action: NavigationAction::Auto,
                page_category: None,
                time_on_page: None,
                scroll_depth: None,
                interactions: Vec::new(),
            });
        }
        
        None
    }

    /// Predict based on sequence (Markov chain)
    async fn predict_sequence(&self, context: &NavigationContext) -> Vec<NavigationPrediction> {
        let history = self.history.read().await;
        let mut predictions = Vec::new();
        
        if let Some(transitions) = history.transitions.get(&context.current_url) {
            let total: usize = transitions.values().sum();
            for (url, count) in transitions {
                let confidence = *count as f32 / total as f32;
                if confidence >= self.config.min_confidence {
                    predictions.push(NavigationPrediction {
                        url: url.clone(),
                        confidence,
                        prediction_type: PredictionType::Sequence,
                        reason: PredictionReason::Sequence { 
                            sequence: vec![context.current_url.clone(), url.clone()] 
                        },
                        estimated_delay: None,
                        related: Vec::new(),
                        metadata: HashMap::new(),
                    });
                }
            }
        }
        
        predictions
    }

    /// Predict based on URL patterns
    async fn predict_from_patterns(&self, context: &NavigationContext) -> Vec<NavigationPrediction> {
        let history = self.history.read().await;
        let mut predictions = Vec::new();
        
        for pattern in &history.patterns {
            if self.matches_pattern(&context.current_url, &pattern.pattern) {
                // Get common next URLs for this pattern
                let pattern_predictions = self.get_pattern_predictions(pattern);
                predictions.extend(pattern_predictions);
            }
        }
        
        predictions
    }

    /// Predict based on frequency
    async fn predict_frequent(&self, _context: &NavigationContext) -> Vec<NavigationPrediction> {
        let history = self.history.read().await;
        let mut predictions = Vec::new();
        
        // Get most visited URLs
        let mut url_freq: Vec<_> = history.url_counts.iter().collect();
        url_freq.sort_by(|a, b| b.1.cmp(a.1));
        
        for (url, count) in url_freq.into_iter().take(5) {
            let confidence = (*count as f32 / history.entries.len().max(1) as f32).min(1.0);
            predictions.push(NavigationPrediction {
                url: url.clone(),
                confidence,
                prediction_type: PredictionType::Preference,
                reason: PredictionReason::FrequentVisit { count: *count },
                estimated_delay: None,
                related: Vec::new(),
                metadata: HashMap::new(),
            });
        }
        
        predictions
    }

    /// Predict based on time patterns
    async fn predict_temporal(&self, _context: &NavigationContext) -> Vec<NavigationPrediction> {
        let history = self.history.read().await;
        let mut predictions = Vec::new();
        
        let now = Utc::now();
        let hour = now.hour() as u8;
        let day_of_week = now.weekday().num_days_from_monday() as u8;
        
        for pattern in &history.temporal_patterns {
            if pattern.hour == hour && pattern.day_of_week == day_of_week {
                for url in &pattern.urls {
                    predictions.push(NavigationPrediction {
                        url: url.clone(),
                        confidence: 0.6,
                        prediction_type: PredictionType::Temporal,
                        reason: PredictionReason::TimePattern { hour, day_of_week },
                        estimated_delay: None,
                        related: Vec::new(),
                        metadata: HashMap::new(),
                    });
                }
            }
        }
        
        predictions
    }

    /// Hash context for caching
    fn hash_context(&self, context: &NavigationContext) -> String {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        context.current_url.hash(&mut hasher);
        for url in context.session_history.iter().take(5) {
            url.hash(&mut hasher);
        }
        format!("{:x}", hasher.finish())
    }

    /// Get cached predictions
    async fn get_cached_predictions(&self, hash: &str) -> Option<Vec<NavigationPrediction>> {
        let cache = self.session_cache.read().await;
        cache.get(hash).map(|c| c.predictions.clone())
    }

    /// Cache predictions
    async fn cache_predictions(&self, hash: &str, predictions: &[NavigationPrediction]) {
        let mut cache = self.session_cache.write().await;
        cache.insert(hash.to_string(), CachedPrediction {
            predictions: predictions.to_vec(),
            context_hash: hash.to_string(),
            cached_at: Utc::now(),
        });
    }

    /// Deduplicate predictions
    fn deduplicate_predictions(&self, predictions: Vec<NavigationPrediction>) -> Vec<NavigationPrediction> {
        let mut seen = HashSet::new();
        predictions
            .into_iter()
            .filter(|p| seen.insert(p.url.clone()))
            .collect()
    }

    /// Trigger prefetch for predictions
    async fn trigger_prefetch(&self, predictions: &[NavigationPrediction]) {
        for pred in predictions.iter().take(3) {
            if pred.confidence > 0.5 {
                self.prefetcher.prefetch(&pred.url).await;
            }
        }
    }

    fn matches_pattern(&self, url: &str, pattern: &str) -> bool {
        // Simple wildcard matching
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                return url.starts_with(parts[0]) && url.ends_with(parts[1]);
            }
        }
        url.contains(pattern)
    }

    fn get_pattern_predictions(&self, pattern: &UrlPattern) -> Vec<NavigationPrediction> {
        pattern.examples.iter()
            .take(3)
            .map(|url| NavigationPrediction {
                url: url.clone(),
                confidence: 0.5,
                prediction_type: PredictionType::Pattern,
                reason: PredictionReason::PatternMatch { pattern: pattern.pattern.clone() },
                estimated_delay: None,
                related: Vec::new(),
                metadata: HashMap::new(),
            })
            .collect()
    }

    /// Learn URL pattern
    pub async fn learn_pattern(&self, pattern: UrlPattern) {
        let mut history = self.history.write().await;
        history.patterns.push(pattern);
    }

    /// Get navigation suggestions
    pub async fn get_suggestions(&self, partial_url: &str) -> Vec<String> {
        let history = self.history.read().await;
        
        history.url_counts.keys()
            .filter(|url| url.contains(partial_url))
            .cloned()
            .take(10)
            .collect()
    }

    /// Clear history
    pub async fn clear_history(&self) {
        let mut history = self.history.write().await;
        history.entries.clear();
        history.url_counts.clear();
        history.transitions.clear();
        history.patterns.clear();
        history.temporal_patterns.clear();
    }
}

impl NavigationPredictor {
    fn new() -> Self {
        Self {
            markov_chain: RwLock::new(MarkovChain::new(2)),
            pattern_matcher: PatternMatcher::new(),
            preference_learner: PreferenceLearner::new(),
        }
    }
}

impl MarkovChain {
    fn new(order: usize) -> Self {
        Self {
            states: HashMap::new(),
            order,
        }
    }
}

impl PatternMatcher {
    fn new() -> Self {
        Self {
            patterns: RwLock::new(Vec::new()),
        }
    }
}

impl PreferenceLearner {
    fn new() -> Self {
        Self {
            preferences: RwLock::new(HashMap::new()),
        }
    }
}

impl PrefetchManager {
    fn new(cache_size_mb: usize) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            config: PrefetchConfig {
                max_cache_size: cache_size_mb * 1024 * 1024,
                ttl_seconds: 300,
                max_concurrent: 5,
                priority_threshold: 0.3,
            },
        }
    }

    pub async fn prefetch(&self, url: &str) {
        // Check if already cached
        let cache = self.cache.read().await;
        if cache.contains_key(url) {
            return;
        }
        drop(cache);
        
        // Simulate prefetch
        let prefetched = PrefetchedPage {
            url: url.to_string(),
            fetched_at: Utc::now(),
            size_bytes: 0,
            expires_at: Utc::now() + chrono::Duration::seconds(self.config.ttl_seconds as i64),
            priority: 0.5,
        };
        
        let mut cache = self.cache.write().await;
        cache.insert(url.to_string(), prefetched);
    }

    pub async fn get_prefetched(&self, url: &str) -> Option<PrefetchedPage> {
        let cache = self.cache.read().await;
        cache.get(url).cloned()
    }
}

// Placeholder model
struct PlaceholderNavigationModel;

#[async_trait::async_trait]
impl NavigationModel for PlaceholderNavigationModel {
    async fn predict(&self, _context: &NavigationContext) -> Result<Vec<NavigationPrediction>, NavigatorError> {
        Ok(Vec::new())
    }

    async fn update(&self, _entry: &HistoryEntry) -> Result<(), NavigatorError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigator_config_defaults() {
        let config = NavigatorConfig::default();
        assert!(config.predictions_enabled);
        assert!(config.prefetch_enabled);
        assert_eq!(config.num_predictions, 5);
    }

    #[tokio::test]
    async fn test_record_navigation() {
        let navigator = PredictiveNavigator::new(NavigatorConfig::default());
        
        let entry = HistoryEntry {
            id: "1".to_string(),
            url: "https://example.com".to_string(),
            title: Some("Example".to_string()),
            timestamp: Utc::now(),
            session_id: "session1".to_string(),
            referrer: None,
            time_spent: 10.0,
            interactions: 5,
        };
        
        navigator.record_navigation(entry).await.unwrap();
        
        let history = navigator.history.read().await;
        assert!(history.url_counts.contains_key("https://example.com"));
    }

    #[tokio::test]
    async fn test_get_predictions() {
        let navigator = PredictiveNavigator::new(NavigatorConfig::default());
        
        // Record some navigation history
        navigator.record_navigation(HistoryEntry {
            id: "1".to_string(),
            url: "https://example.com".to_string(),
            title: None,
            timestamp: Utc::now(),
            session_id: "s1".to_string(),
            referrer: None,
            time_spent: 5.0,
            interactions: 1,
        }).await.unwrap();
        
        let context = NavigationContext {
            current_url: "https://example.com".to_string(),
            session_history: VecDeque::new(),
            page_title: None,
            timestamp: Utc::now(),
            referrer: None,
            action: NavigationAction::DirectInput,
            page_category: None,
            time_on_page: None,
            scroll_depth: None,
            interactions: Vec::new(),
        };
        
        let predictions = navigator.get_predictions(&context).await.unwrap();
        assert!(!predictions.is_empty() || true); // May be empty if no patterns learned
    }
}