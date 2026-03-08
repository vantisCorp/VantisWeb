//! Content Analyzer Module
//! 
//! AI-powered content analysis including sentiment analysis,
//! entity extraction, summarization, and content classification.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Content Analyzer
pub struct ContentAnalyzer {
    sentiment_analyzer: Arc<SentimentAnalyzer>,
    entity_extractor: Arc<EntityExtractor>,
    summarizer: Arc<ContentSummarizer>,
    classifier: Arc<ContentClassifier>,
    config: AnalyzerConfig,
    cache: RwLock<HashMap<String, CachedAnalysis>>,
}

/// Analyzer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzerConfig {
    /// Enable sentiment analysis
    pub sentiment_enabled: bool,
    /// Enable entity extraction
    pub entity_extraction_enabled: bool,
    /// Enable summarization
    pub summarization_enabled: bool,
    /// Enable classification
    pub classification_enabled: bool,
    /// Maximum content length to analyze
    pub max_content_length: usize,
    /// Summary length (number of sentences)
    pub summary_sentences: usize,
    /// Enable caching
    pub cache_enabled: bool,
    /// Cache TTL in seconds
    pub cache_ttl: u64,
    /// Language detection enabled
    pub language_detection: bool,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            sentiment_enabled: true,
            entity_extraction_enabled: true,
            summarization_enabled: true,
            classification_enabled: true,
            max_content_length: 1000000,
            summary_sentences: 3,
            cache_enabled: true,
            cache_ttl: 3600,
            language_detection: true,
        }
    }
}

/// Analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// Content hash for caching
    pub content_hash: String,
    /// Detected language
    pub language: String,
    /// Sentiment analysis result
    pub sentiment: SentimentResult,
    /// Extracted entities
    pub entities: Vec<ExtractedEntity>,
    /// Content summary
    pub summary: Option<String>,
    /// Content classification
    pub classification: ContentClassification,
    /// Key phrases
    pub key_phrases: Vec<KeyPhrase>,
    /// Topics
    pub topics: Vec<Topic>,
    /// Readability metrics
    pub readability: ReadabilityMetrics,
    /// Analysis timestamp
    pub analyzed_at: DateTime<Utc>,
}

/// Sentiment analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentResult {
    /// Overall sentiment label
    pub label: SentimentLabel,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Positive score (0.0 - 1.0)
    pub positive_score: f32,
    /// Negative score (0.0 - 1.0)
    pub negative_score: f32,
    /// Neutral score (0.0 - 1.0)
    pub neutral_score: f32,
    /// Sentence-level sentiment
    pub sentence_sentiments: Vec<SentenceSentiment>,
    /// Emotion scores
    pub emotions: EmotionScores,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SentimentLabel {
    Positive,
    Negative,
    Neutral,
    Mixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentenceSentiment {
    pub sentence: String,
    pub label: SentimentLabel,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionScores {
    pub joy: f32,
    pub sadness: f32,
    pub anger: f32,
    pub fear: f32,
    pub surprise: f32,
    pub disgust: f32,
    pub trust: f32,
    pub anticipation: f32,
}

impl Default for EmotionScores {
    fn default() -> Self {
        Self {
            joy: 0.0,
            sadness: 0.0,
            anger: 0.0,
            fear: 0.0,
            surprise: 0.0,
            disgust: 0.0,
            trust: 0.0,
            anticipation: 0.0,
        }
    }
}

/// Extracted entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntity {
    /// Entity text
    pub text: String,
    /// Entity type
    pub entity_type: EntityType,
    /// Start position in text
    pub start: usize,
    /// End position in text
    pub end: usize,
    /// Confidence score
    pub confidence: f32,
    /// Normalized/normalized form
    pub normalized: Option<String>,
    /// Wikipedia/Wikidata ID
    pub knowledge_base_id: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntityType {
    Person,
    Organization,
    Location,
    Date,
    Time,
    Money,
    Percentage,
    Product,
    Event,
    WorkOfArt,
    Language,
    Nationality,
    Religion,
    Title,
    Phone,
    Email,
    Url,
    IPAddress,
    CreditCard,
    Custom(String),
}

/// Key phrase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPhrase {
    pub phrase: String,
    pub importance: f32,
    pub frequency: usize,
    pub positions: Vec<usize>,
}

/// Topic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    pub name: String,
    pub confidence: f32,
    pub related_terms: Vec<String>,
}

/// Content classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentClassification {
    /// Primary category
    pub primary_category: ContentCategory,
    /// All categories with scores
    pub categories: Vec<CategoryScore>,
    /// Content type
    pub content_type: ClassifiedContentType,
    /// Quality score
    pub quality_score: f32,
    /// Safety flags
    pub safety_flags: Vec<SafetyFlag>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContentCategory {
    Technology,
    Science,
    Business,
    Sports,
    Entertainment,
    Politics,
    Health,
    Education,
    Travel,
    Food,
    Fashion,
    Finance,
    News,
    Opinion,
    Tutorial,
    Documentation,
    Marketing,
    Legal,
    Medical,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryScore {
    pub category: ContentCategory,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClassifiedContentType {
    Article,
    Blog,
    News,
    Research,
    Tutorial,
    Documentation,
    Review,
    Opinion,
    Social,
    Forum,
    Product,
    Landing,
    About,
    Contact,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyFlag {
    pub flag_type: SafetyFlagType,
    pub severity: Severity,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SafetyFlagType {
    AdultContent,
    Violence,
    Hate,
    Spam,
    Malware,
    Phishing,
    Misinformation,
    SelfHarm,
    Drugs,
    Gambling,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Readability metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadabilityMetrics {
    /// Flesch Reading Ease score
    pub flesch_reading_ease: f32,
    /// Flesch-Kincaid Grade Level
    pub flesch_kincaid_grade: f32,
    /// Gunning Fog Index
    pub gunning_fog: f32,
    /// SMOG Index
    pub smog: f32,
    /// Coleman-Liau Index
    pub coleman_liau: f32,
    /// Automated Readability Index
    pub automated_readability: f32,
    /// Average sentence length
    pub avg_sentence_length: f32,
    /// Average word length
    pub avg_word_length: f32,
    /// Complex word percentage
    pub complex_word_percent: f32,
    /// Total words
    pub total_words: usize,
    /// Total sentences
    pub total_sentences: usize,
    /// Total syllables
    pub total_syllables: usize,
}

/// Cached analysis
#[derive(Debug, Clone)]
struct CachedAnalysis {
    result: AnalysisResult,
    cached_at: DateTime<Utc>,
}

/// Sentiment analyzer
pub struct SentimentAnalyzer {
    model: Arc<dyn SentimentModel>,
}

/// Entity extractor
pub struct EntityExtractor {
    ner_model: Arc<dyn NERModel>,
    patterns: Vec<EntityPattern>,
}

/// Content summarizer
pub struct ContentSummarizer {
    model: Arc<dyn SummarizationModel>,
    config: SummarizerConfig,
}

/// Content classifier
pub struct ContentClassifier {
    model: Arc<dyn ClassificationModel>,
    taxonomy: Vec<TaxonomyNode>,
}

// Trait definitions
#[async_trait::async_trait]
pub trait SentimentModel: Send + Sync {
    async fn analyze(&self, text: &str) -> Result<SentimentResult, AnalyzerError>;
}

#[async_trait::async_trait]
pub trait NERModel: Send + Sync {
    async fn extract(&self, text: &str) -> Result<Vec<ExtractedEntity>, AnalyzerError>;
}

#[async_trait::async_trait]
pub trait SummarizationModel: Send + Sync {
    async fn summarize(&self, text: &str, sentences: usize) -> Result<String, AnalyzerError>;
}

#[async_trait::async_trait]
pub trait ClassificationModel: Send + Sync {
    async fn classify(&self, text: &str) -> Result<ContentClassification, AnalyzerError>;
}

/// Analyzer error
#[derive(Debug, thiserror::Error)]
pub enum AnalyzerError {
    #[error("Content too long: {0} bytes")]
    ContentTooLong(usize),
    #[error("Model error: {0}")]
    ModelError(String),
    #[error("Processing error: {0}")]
    ProcessingError(String),
    #[error("Language not supported: {0}")]
    LanguageNotSupported(String),
    #[error("Cache error: {0}")]
    CacheError(String),
}

struct EntityPattern {
    entity_type: EntityType,
    pattern: String,
}

#[derive(Debug, Clone)]
struct SummarizerConfig {
    min_length: usize,
    max_length: usize,
    extractive: bool,
    abstractive: bool,
}

#[derive(Debug, Clone)]
struct TaxonomyNode {
    category: ContentCategory,
    keywords: Vec<String>,
    children: Vec<TaxonomyNode>,
}

impl ContentAnalyzer {
    pub fn new(config: AnalyzerConfig) -> Self {
        Self {
            sentiment_analyzer: Arc::new(SentimentAnalyzer::new()),
            entity_extractor: Arc::new(EntityExtractor::new()),
            summarizer: Arc::new(ContentSummarizer::new(config.summary_sentences)),
            classifier: Arc::new(ContentClassifier::new()),
            config,
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// Analyze content
    pub async fn analyze(&self, content: &str) -> Result<AnalysisResult, AnalyzerError> {
        // Check content length
        if content.len() > self.config.max_content_length {
            return Err(AnalyzerError::ContentTooLong(content.len()));
        }

        // Check cache
        if self.config.cache_enabled {
            let hash = self.compute_hash(content);
            if let Some(cached) = self.get_cached(&hash).await {
                return Ok(cached);
            }
        }

        // Perform analysis
        let mut result = AnalysisResult {
            content_hash: self.compute_hash(content),
            language: self.detect_language(content).await,
            sentiment: SentimentResult::default(),
            entities: Vec::new(),
            summary: None,
            classification: ContentClassification::default(),
            key_phrases: Vec::new(),
            topics: Vec::new(),
            readability: self.calculate_readability(content),
            analyzed_at: Utc::now(),
        };

        // Sentiment analysis
        if self.config.sentiment_enabled {
            result.sentiment = self.sentiment_analyzer.analyze(content).await?;
        }

        // Entity extraction
        if self.config.entity_extraction_enabled {
            result.entities = self.entity_extractor.extract(content).await?;
        }

        // Summarization
        if self.config.summarization_enabled {
            result.summary = Some(self.summarizer.summarize(content, self.config.summary_sentences).await?);
        }

        // Classification
        if self.config.classification_enabled {
            result.classification = self.classifier.classify(content).await?;
        }

        // Extract key phrases
        result.key_phrases = self.extract_key_phrases(content);

        // Extract topics
        result.topics = self.extract_topics(content);

        // Cache result
        if self.config.cache_enabled {
            self.cache_result(&result).await;
        }

        Ok(result)
    }

    /// Analyze with specific features
    pub async fn analyze_features(
        &self,
        content: &str,
        features: &[AnalysisFeature],
    ) -> Result<PartialAnalysis, AnalyzerError> {
        let mut result = PartialAnalysis::default();

        for feature in features {
            match feature {
                AnalysisFeature::Sentiment => {
                    result.sentiment = Some(self.sentiment_analyzer.analyze(content).await?);
                }
                AnalysisFeature::Entities => {
                    result.entities = Some(self.entity_extractor.extract(content).await?);
                }
                AnalysisFeature::Summary => {
                    result.summary = Some(self.summarizer.summarize(content, self.config.summary_sentences).await?);
                }
                AnalysisFeature::Classification => {
                    result.classification = Some(self.classifier.classify(content).await?);
                }
                AnalysisFeature::KeyPhrases => {
                    result.key_phrases = Some(self.extract_key_phrases(content));
                }
                AnalysisFeature::Readability => {
                    result.readability = Some(self.calculate_readability(content));
                }
            }
        }

        Ok(result)
    }

    /// Detect language
    async fn detect_language(&self, content: &str) -> String {
        // Simple heuristic-based detection
        // In production, would use a proper language detection library
        let sample: String = content.chars().take(500).collect();
        
        // Check for common patterns
        if sample.contains("the ") || sample.contains("and ") {
            "en".to_string()
        } else if sample.contains(" el ") || sample.contains(" la ") {
            "es".to_string()
        } else if sample.contains(" le ") || sample.contains(" la ") {
            "fr".to_string()
        } else if sample.contains(" der ") || sample.contains(" die ") {
            "de".to_string()
        } else {
            "en".to_string() // Default to English
        }
    }

    /// Compute content hash
    fn compute_hash(&self, content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Get cached result
    async fn get_cached(&self, hash: &str) -> Option<AnalysisResult> {
        let cache = self.cache.read().await;
        if let Some(cached) = cache.get(hash) {
            let age = (Utc::now() - cached.cached_at).num_seconds() as u64;
            if age < self.config.cache_ttl {
                return Some(cached.result.clone());
            }
        }
        None
    }

    /// Cache result
    async fn cache_result(&self, result: &AnalysisResult) {
        let mut cache = self.cache.write().await;
        cache.insert(result.content_hash.clone(), CachedAnalysis {
            result: result.clone(),
            cached_at: Utc::now(),
        });
    }

    /// Extract key phrases
    fn extract_key_phrases(&self, content: &str) -> Vec<KeyPhrase> {
        // Simple TF-IDF based extraction
        let words: Vec<&str> = content.split_whitespace().collect();
        let mut word_freq: HashMap<&str, usize> = HashMap::new();
        
        for word in words {
            let normalized = word.to_lowercase().trim_matches(|c: char| !c.is_alphanumeric()).to_string();
            if normalized.len() > 3 {
                *word_freq.entry(Box::leak(normalized.into_boxed_str())).or_insert(0) += 1;
            }
        }
        
        let total_words = words.len() as f32;
        
        word_freq
            .into_iter()
            .filter(|(_, freq)| *freq > 2)
            .map(|(word, freq)| KeyPhrase {
                phrase: word.to_string(),
                importance: freq as f32 / total_words,
                frequency: freq,
                positions: Vec::new(),
            })
            .take(10)
            .collect()
    }

    /// Extract topics
    fn extract_topics(&self, _content: &str) -> Vec<Topic> {
        // Placeholder - would use LDA or similar
        Vec::new()
    }

    /// Calculate readability metrics
    fn calculate_readability(&self, content: &str) -> ReadabilityMetrics {
        let words: Vec<&str> = content.split_whitespace().collect();
        let sentences: Vec<&str> = content.split(&['.', '!', '?'][..]).collect();
        let syllables = self.count_syllables(content);
        
        let total_words = words.len();
        let total_sentences = sentences.len().max(1);
        let total_syllables = syllables;
        
        let avg_sentence_length = total_words as f32 / total_sentences as f32;
        let avg_word_length = total_words as f32 / total_words as f32;
        let complex_words = words.iter().filter(|w| self.count_syllables_in_word(w) > 2).count();
        let complex_word_percent = (complex_words as f32 / total_words as f32) * 100.0;
        
        // Flesch Reading Ease
        let flesch_reading_ease = 206.835 
            - (1.015 * avg_sentence_length) 
            - (84.6 * total_syllables as f32 / total_words as f32);
        
        // Flesch-Kincaid Grade Level
        let flesch_kincaid_grade = (0.39 * avg_sentence_length) 
            + (11.8 * total_syllables as f32 / total_words as f32) 
            - 15.59;
        
        // Gunning Fog Index
        let gunning_fog = 0.4 * (avg_sentence_length + complex_word_percent / 100.0);
        
        ReadabilityMetrics {
            flesch_reading_ease: flesch_reading_ease.max(0.0).min(100.0),
            flesch_kincaid_grade: flesch_kincaid_grade.max(0.0),
            gunning_fog: gunning_fog.max(0.0),
            smog: 0.0,
            coleman_liau: 0.0,
            automated_readability: 0.0,
            avg_sentence_length,
            avg_word_length,
            complex_word_percent,
            total_words,
            total_sentences,
            total_syllables,
        }
    }

    fn count_syllables(&self, content: &str) -> usize {
        content.split_whitespace()
            .map(|w| self.count_syllables_in_word(w))
            .sum()
    }

    fn count_syllables_in_word(&self, word: &str) -> usize {
        let word = word.to_lowercase();
        let vowels = ['a', 'e', 'i', 'o', 'u', 'y'];
        let mut count = 0;
        let mut prev_is_vowel = false;
        
        for c in word.chars() {
            let is_vowel = vowels.contains(&c);
            if is_vowel && !prev_is_vowel {
                count += 1;
            }
            prev_is_vowel = is_vowel;
        }
        
        // Adjust for silent e
        if word.ends_with('e') && count > 1 {
            count -= 1;
        }
        
        count.max(1)
    }
}

/// Analysis feature selector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisFeature {
    Sentiment,
    Entities,
    Summary,
    Classification,
    KeyPhrases,
    Readability,
}

/// Partial analysis result
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PartialAnalysis {
    pub sentiment: Option<SentimentResult>,
    pub entities: Option<Vec<ExtractedEntity>>,
    pub summary: Option<String>,
    pub classification: Option<ContentClassification>,
    pub key_phrases: Option<Vec<KeyPhrase>>,
    pub readability: Option<ReadabilityMetrics>,
}

// Default implementations
impl Default for SentimentResult {
    fn default() -> Self {
        Self {
            label: SentimentLabel::Neutral,
            confidence: 0.5,
            positive_score: 0.33,
            negative_score: 0.33,
            neutral_score: 0.34,
            sentence_sentiments: Vec::new(),
            emotions: EmotionScores::default(),
        }
    }
}

impl Default for ContentClassification {
    fn default() -> Self {
        Self {
            primary_category: ContentCategory::Other,
            categories: Vec::new(),
            content_type: ClassifiedContentType::Other,
            quality_score: 0.5,
            safety_flags: Vec::new(),
        }
    }
}

// Placeholder implementations
impl SentimentAnalyzer {
    fn new() -> Self {
        Self {
            model: Arc::new(PlaceholderSentimentModel),
        }
    }

    pub async fn analyze(&self, text: &str) -> Result<SentimentResult, AnalyzerError> {
        self.model.analyze(text).await
    }
}

impl EntityExtractor {
    fn new() -> Self {
        Self {
            ner_model: Arc::new(PlaceholderNERModel),
            patterns: Vec::new(),
        }
    }

    pub async fn extract(&self, text: &str) -> Result<Vec<ExtractedEntity>, AnalyzerError> {
        self.ner_model.extract(text).await
    }
}

impl ContentSummarizer {
    fn new(sentences: usize) -> Self {
        Self {
            model: Arc::new(PlaceholderSummarizationModel),
            config: SummarizerConfig {
                min_length: 50,
                max_length: 500,
                extractive: true,
                abstractive: false,
            },
        }
    }

    pub async fn summarize(&self, text: &str, _sentences: usize) -> Result<String, AnalyzerError> {
        self.model.summarize(text, 3).await
    }
}

impl ContentClassifier {
    fn new() -> Self {
        Self {
            model: Arc::new(PlaceholderClassificationModel),
            taxonomy: Vec::new(),
        }
    }

    pub async fn classify(&self, text: &str) -> Result<ContentClassification, AnalyzerError> {
        self.model.classify(text).await
    }
}

// Placeholder models
struct PlaceholderSentimentModel;
struct PlaceholderNERModel;
struct PlaceholderSummarizationModel;
struct PlaceholderClassificationModel;

#[async_trait::async_trait]
impl SentimentModel for PlaceholderSentimentModel {
    async fn analyze(&self, text: &str) -> Result<SentimentResult, AnalyzerError> {
        let positive_words = ["good", "great", "excellent", "amazing", "wonderful", "best"];
        let negative_words = ["bad", "terrible", "awful", "worst", "horrible", "poor"];
        
        let text_lower = text.to_lowercase();
        let positive_count = positive_words.iter().filter(|w| text_lower.contains(*w)).count();
        let negative_count = negative_words.iter().filter(|w| text_lower.contains(*w)).count();
        
        let total = (positive_count + negative_count).max(1) as f32;
        let positive_score = positive_count as f32 / total;
        let negative_score = negative_count as f32 / total;
        
        let label = if positive_score > 0.6 {
            SentimentLabel::Positive
        } else if negative_score > 0.6 {
            SentimentLabel::Negative
        } else if positive_count > 0 && negative_count > 0 {
            SentimentLabel::Mixed
        } else {
            SentimentLabel::Neutral
        };
        
        Ok(SentimentResult {
            label,
            confidence: 0.7,
            positive_score,
            negative_score,
            neutral_score: 1.0 - positive_score - negative_score,
            sentence_sentiments: Vec::new(),
            emotions: EmotionScores::default(),
        })
    }
}

#[async_trait::async_trait]
impl NERModel for PlaceholderNERModel {
    async fn extract(&self, text: &str) -> Result<Vec<ExtractedEntity>, AnalyzerError> {
        // Simple regex-based entity extraction
        let mut entities = Vec::new();
        
        // Email detection
        let email_pattern = regex::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap();
        for cap in email_pattern.captures_iter(text) {
            entities.push(ExtractedEntity {
                text: cap[0].to_string(),
                entity_type: EntityType::Email,
                start: cap.get(0).unwrap().start(),
                end: cap.get(0).unwrap().end(),
                confidence: 0.95,
                normalized: None,
                knowledge_base_id: None,
                metadata: HashMap::new(),
            });
        }
        
        // URL detection
        let url_pattern = regex::Regex::new(r"https?://[^\s]+").unwrap();
        for cap in url_pattern.captures_iter(text) {
            entities.push(ExtractedEntity {
                text: cap[0].to_string(),
                entity_type: EntityType::Url,
                start: cap.get(0).unwrap().start(),
                end: cap.get(0).unwrap().end(),
                confidence: 0.95,
                normalized: None,
                knowledge_base_id: None,
                metadata: HashMap::new(),
            });
        }
        
        Ok(entities)
    }
}

#[async_trait::async_trait]
impl SummarizationModel for PlaceholderSummarizationModel {
    async fn summarize(&self, text: &str, _sentences: usize) -> Result<String, AnalyzerError> {
        // Simple extractive summarization - take first few sentences
        let sentences: Vec<&str> = text.split(&['.', '!', '?'][..]).collect();
        let summary = sentences.iter().take(3).cloned().collect::<Vec<_>>().join(". ");
        Ok(format!("{}.", summary))
    }
}

#[async_trait::async_trait]
impl ClassificationModel for PlaceholderClassificationModel {
    async fn classify(&self, text: &str) -> Result<ContentClassification, AnalyzerError> {
        let text_lower = text.to_lowercase();
        
        // Simple keyword-based classification
        let category = if text_lower.contains("code") || text_lower.contains("programming") {
            ContentCategory::Technology
        } else if text_lower.contains("research") || text_lower.contains("study") {
            ContentCategory::Science
        } else if text_lower.contains("business") || text_lower.contains("company") {
            ContentCategory::Business
        } else if text_lower.contains("sport") || text_lower.contains("game") {
            ContentCategory::Sports
        } else if text_lower.contains("movie") || text_lower.contains("music") {
            ContentCategory::Entertainment
        } else {
            ContentCategory::Other
        };
        
        Ok(ContentClassification {
            primary_category: category,
            categories: vec![CategoryScore { category, score: 0.8 }],
            content_type: ClassifiedContentType::Article,
            quality_score: 0.7,
            safety_flags: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_config_defaults() {
        let config = AnalyzerConfig::default();
        assert!(config.sentiment_enabled);
        assert!(config.entity_extraction_enabled);
        assert!(config.summarization_enabled);
        assert!(config.classification_enabled);
    }

    #[tokio::test]
    async fn test_sentiment_analysis() {
        let analyzer = SentimentAnalyzer::new();
        let result = analyzer.analyze("This is a great and wonderful product!").await.unwrap();
        assert_eq!(result.label, SentimentLabel::Positive);
    }

    #[test]
    fn test_syllable_counting() {
        let content_analyzer = ContentAnalyzer::new(AnalyzerConfig::default());
        assert_eq!(content_analyzer.count_syllables_in_word("hello"), 2);
        assert_eq!(content_analyzer.count_syllables_in_word("beautiful"), 3);
    }
}