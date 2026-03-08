//! Data models for AI Enhanced module

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Search result from smart search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Result title
    pub title: String,
    /// URL
    pub url: String,
    /// Snippet/description
    pub snippet: String,
    /// Relevance score (0.0 - 1.0)
    pub relevance: f32,
    /// Source type
    pub source: SourceType,
    /// Timestamp
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Source type for search results
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SourceType {
    Web,
    Bookmark,
    History,
    Download,
    Tab,
}

/// Content summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSummary {
    /// Original URL
    pub url: String,
    /// Summary title
    pub title: String,
    /// Brief summary
    pub summary: String,
    /// Key points
    pub key_points: Vec<String>,
    /// Topics/tags
    pub topics: Vec<String>,
    /// Reading time estimate (minutes)
    pub reading_time: u32,
    /// Sentiment
    pub sentiment: Sentiment,
    /// Language
    pub language: String,
    /// Word count
    pub word_count: u32,
}

/// Sentiment analysis result
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Sentiment {
    Positive,
    Negative,
    Neutral,
    Mixed,
}

/// Bookmark suggestion from AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkSuggestion {
    /// Suggested folder/category
    pub folder: String,
    /// Suggested tags
    pub tags: Vec<String>,
    /// Confidence score
    pub confidence: f32,
    /// Similar bookmarks
    pub similar: Vec<SimilarBookmark>,
    /// Reason for suggestion
    pub reason: String,
}

/// Similar bookmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarBookmark {
    pub title: String,
    pub url: String,
    pub similarity: f32,
}

/// Navigation prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationPrediction {
    /// Predicted URL
    pub url: String,
    /// Title
    pub title: Option<String>,
    /// Confidence score
    pub confidence: f32,
    /// Reason for prediction
    pub reason: PredictionReason,
    /// Context
    pub context: String,
}

/// Prediction reason
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PredictionReason {
    FrequentVisit,
    TimeOfDay,
    SimilarContext,
    RelatedToCurrent,
    Sequential,
    SearchHistory,
}

/// Content analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentAnalysis {
    /// URL analyzed
    pub url: String,
    /// Content type
    pub content_type: ContentType,
    /// Main topics
    pub topics: Vec<Topic>,
    /// Entities mentioned
    pub entities: Vec<Entity>,
    /// Readability score
    pub readability: ReadabilityScore,
    /// Quality indicators
    pub quality: QualityIndicators,
    /// Safety assessment
    pub safety: SafetyAssessment,
}

/// Content type classification
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ContentType {
    Article,
    Blog,
    News,
    Documentation,
    Tutorial,
    Product,
    Video,
    Social,
    Forum,
    Other,
}

/// Topic with relevance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    pub name: String,
    pub relevance: f32,
    pub category: Option<String>,
}

/// Named entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    pub entity_type: EntityType,
    pub mentions: u32,
    pub sentiment: Sentiment,
}

/// Entity types
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EntityType {
    Person,
    Organization,
    Location,
    Date,
    Event,
    Product,
    Concept,
}

/// Readability score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadabilityScore {
    /// Flesch reading ease
    pub flesch_ease: f32,
    /// Grade level
    pub grade_level: f32,
    /// Reading time (minutes)
    pub reading_time: u32,
    /// Complex words percentage
    pub complex_words: f32,
}

/// Quality indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIndicators {
    /// Has citations
    pub has_citations: bool,
    /// Author credibility
    pub author_credibility: f32,
    /// Source reputation
    pub source_reputation: f32,
    /// Content freshness
    pub freshness: f32,
    /// Overall quality score
    pub overall: f32,
}

/// Safety assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyAssessment {
    /// Is safe
    pub is_safe: bool,
    /// Risk level
    pub risk_level: RiskLevel,
    /// Categories of concern
    pub concerns: Vec<String>,
    /// Age appropriateness
    pub age_appropriate: AgeRating,
}

/// Risk levels
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RiskLevel {
    Safe,
    Low,
    Medium,
    High,
    Critical,
}

/// Age rating
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AgeRating {
    Everyone,
    Teen,
    Mature,
    Adult,
    Unrated,
}

/// Conversation message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Message role
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// Conversation context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationContext {
    pub id: String,
    pub messages: Vec<ConversationMessage>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

/// Smart suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartSuggestion {
    pub suggestion_type: SuggestionType,
    pub title: String,
    pub description: String,
    pub action: SuggestionAction,
    pub relevance: f32,
}

/// Suggestion types
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SuggestionType {
    RelatedArticle,
    SimilarSite,
    Translation,
    Definition,
    FactCheck,
    Alternative,
}

/// Suggested action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionAction {
    OpenUrl(String),
    Search(String),
    Translate { from: String, to: String },
    Define(String),
    Compare { urls: Vec<String> },
}

/// Learning feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningFeedback {
    pub interaction_id: String,
    pub feedback_type: FeedbackType,
    pub rating: Option<u8>,
    pub comment: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Feedback types
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FeedbackType {
    Helpful,
    NotHelpful,
    Incorrect,
    Offensive,
    Other,
}

/// AI model statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AIStats {
    pub total_queries: u64,
    pub successful_queries: u64,
    pub failed_queries: u64,
    pub average_response_time_ms: f64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub tokens_used: u64,
}