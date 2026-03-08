//! AI Enhanced Module
//! 
//! Advanced AI-powered features for next-generation browsing experience
//! 
//! # Features
//! - AI Assistant for browsing help
//! - Smart Search with semantic understanding
//! - Content Analysis and summarization
//! - Predictive Navigation
//! - Intelligent Bookmark Organization

pub mod assistant;
pub mod smart_search;
pub mod content_analyzer;
pub mod predictive_nav;
pub mod bookmark_ai;
pub mod models;

use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

pub use assistant::AIAssistant;
pub use smart_search::SmartSearch;
pub use content_analyzer::ContentAnalyzer;
pub use predictive_nav::PredictiveNavigator;
pub use bookmark_ai::BookmarkAI;
pub use models::*;

/// AI Enhanced Manager - Central hub for AI features
pub struct AIEnhancedManager {
    /// AI Assistant
    assistant: Arc<AIAssistant>,
    /// Smart Search Engine
    smart_search: Arc<SmartSearch>,
    /// Content Analyzer
    content_analyzer: Arc<ContentAnalyzer>,
    /// Predictive Navigator
    navigator: Arc<PredictiveNavigator>,
    /// Bookmark AI
    bookmark_ai: Arc<BookmarkAI>,
    /// Configuration
    config: AIConfig,
    /// User preferences
    preferences: RwLock<UserPreferences>,
}

/// AI Configuration
#[derive(Debug, Clone)]
pub struct AIConfig {
    /// Enable AI features
    pub enabled: bool,
    /// API endpoint for AI services
    pub api_endpoint: Option<String>,
    /// Model to use
    pub model: AIModel,
    /// Maximum tokens for responses
    pub max_tokens: usize,
    /// Temperature for generation
    pub temperature: f32,
    /// Enable local processing
    pub local_processing: bool,
    /// Privacy mode
    pub privacy_mode: PrivacyMode,
    /// Language preference
    pub language: String,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            api_endpoint: None,
            model: AIModel::Default,
            max_tokens: 2048,
            temperature: 0.7,
            local_processing: true,
            privacy_mode: PrivacyMode::Balanced,
            language: "en".to_string(),
        }
    }
}

/// AI Models available
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AIModel {
    Default,
    Fast,
    Advanced,
    Custom,
}

/// Privacy modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrivacyMode {
    /// Full AI features with cloud processing
    Full,
    /// Balanced - local when possible
    Balanced,
    /// Strict - local only
    Strict,
    /// Disabled
    Disabled,
}

/// User preferences for AI
#[derive(Debug, Clone, Default)]
pub struct UserPreferences {
    /// Preferred summary length
    pub summary_length: SummaryLength,
    /// Enable auto-summarize
    pub auto_summarize: bool,
    /// Enable predictive navigation
    pub predictive_nav: bool,
    /// Enable smart suggestions
    pub smart_suggestions: bool,
    /// Content filtering preferences
    pub content_filter: ContentFilter,
    /// Custom prompts
    pub custom_prompts: HashMap<String, String>,
}

/// Summary length preference
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SummaryLength {
    Brief,      // 1-2 sentences
    Standard,   // 1 paragraph
    Detailed,   // Multiple paragraphs
    Full,       // Comprehensive
}

impl Default for SummaryLength {
    fn default() -> Self {
        Self::Standard
    }
}

/// Content filtering options
#[derive(Debug, Clone, Default)]
pub struct ContentFilter {
    /// Filter adult content
    pub adult_filter: bool,
    /// Filter violence
    pub violence_filter: bool,
    /// Filter spam
    pub spam_filter: bool,
    /// Custom blocked topics
    pub blocked_topics: Vec<String>,
}

impl AIEnhancedManager {
    /// Create a new AI Enhanced Manager
    pub fn new(config: AIConfig) -> Self {
        let assistant = Arc::new(AIAssistant::new(config.clone()));
        let smart_search = Arc::new(SmartSearch::new(config.clone()));
        let content_analyzer = Arc::new(ContentAnalyzer::new(config.clone()));
        let navigator = Arc::new(PredictiveNavigator::new(config.clone()));
        let bookmark_ai = Arc::new(BookmarkAI::new(config.clone()));
        
        Self {
            assistant,
            smart_search,
            content_analyzer,
            navigator,
            bookmark_ai,
            config,
            preferences: RwLock::new(UserPreferences::default()),
        }
    }
    
    /// Get the AI Assistant
    pub fn assistant(&self) -> &AIAssistant {
        &self.assistant
    }
    
    /// Get Smart Search
    pub fn smart_search(&self) -> &SmartSearch {
        &self.smart_search
    }
    
    /// Get Content Analyzer
    pub fn content_analyzer(&self) -> &ContentAnalyzer {
        &self.content_analyzer
    }
    
    /// Get Predictive Navigator
    pub fn navigator(&self) -> &PredictiveNavigator {
        &self.navigator
    }
    
    /// Get Bookmark AI
    pub fn bookmark_ai(&self) -> &BookmarkAI {
        &self.bookmark_ai
    }
    
    /// Update user preferences
    pub async fn update_preferences(&self, prefs: UserPreferences) {
        let mut preferences = self.preferences.write().await;
        *preferences = prefs;
    }
    
    /// Get current preferences
    pub async fn get_preferences(&self) -> UserPreferences {
        self.preferences.read().await.clone()
    }
    
    /// Process a natural language query
    pub async fn process_query(&self, query: &str) -> Result<AIResponse, AIError> {
        // Determine intent
        let intent = self.classify_intent(query).await?;
        
        match intent {
            QueryIntent::Search => {
                let results = self.smart_search.search(query).await?;
                Ok(AIResponse::SearchResults(results))
            }
            QueryIntent::Summarize => {
                let summary = self.content_analyzer.summarize_current().await?;
                Ok(AIResponse::Summary(summary))
            }
            QueryIntent::Navigate => {
                let url = self.navigator.predict_destination(query).await?;
                Ok(AIResponse::Navigation(url))
            }
            QueryIntent::Bookmark => {
                let suggestion = self.bookmark_ai.suggest_category(query).await?;
                Ok(AIResponse::BookmarkSuggestion(suggestion))
            }
            QueryIntent::Question => {
                let answer = self.assistant.ask(query).await?;
                Ok(AIResponse::Answer(answer))
            }
            QueryIntent::Unknown => {
                let response = self.assistant.chat(query).await?;
                Ok(AIResponse::Chat(response))
            }
        }
    }
    
    /// Classify query intent
    async fn classify_intent(&self, query: &str) -> Result<QueryIntent, AIError> {
        let query_lower = query.to_lowercase();
        
        if query_lower.starts_with("search for") || query_lower.starts_with("find") {
            return Ok(QueryIntent::Search);
        }
        if query_lower.contains("summarize") || query_lower.contains("summary") {
            return Ok(QueryIntent::Summarize);
        }
        if query_lower.starts_with("go to") || query_lower.starts_with("navigate") {
            return Ok(QueryIntent::Navigate);
        }
        if query_lower.contains("bookmark") || query_lower.contains("save") {
            return Ok(QueryIntent::Bookmark);
        }
        if query_lower.ends_with("?") {
            return Ok(QueryIntent::Question);
        }
        
        Ok(QueryIntent::Unknown)
    }
    
    /// Enable/disable AI features
    pub async fn set_enabled(&self, enabled: bool) {
        let _ = enabled;
        // Would update config
    }
}

/// Query intent classification
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QueryIntent {
    Search,
    Summarize,
    Navigate,
    Bookmark,
    Question,
    Unknown,
}

/// AI Response types
#[derive(Debug, Clone)]
pub enum AIResponse {
    SearchResults(Vec<SearchResult>),
    Summary(ContentSummary),
    Navigation(String),
    BookmarkSuggestion(BookmarkSuggestion),
    Answer(String),
    Chat(String),
}

/// AI Error
#[derive(Debug, thiserror::Error)]
pub enum AIError {
    #[error("AI service unavailable: {0}")]
    ServiceUnavailable(String),
    
    #[error("Processing error: {0}")]
    ProcessingError(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Privacy restriction: {0}")]
    PrivacyRestriction(String),
    
    #[error("Model error: {0}")]
    ModelError(String),
    
    #[error("Rate limited")]
    RateLimited,
}