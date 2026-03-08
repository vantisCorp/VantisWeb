//! Bookmark AI Module
//! 
//! AI-powered bookmark management including automatic categorization,
//! smart organization, duplicate detection, and intelligent recommendations.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::{HashMap, HashSet, BTreeMap};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Bookmark AI Manager
pub struct BookmarkAI {
    organizer: Arc<BookmarkOrganizer>,
    categorizer: Arc<BookmarkCategorizer>,
    deduplicator: Arc<Deduplicator>,
    recommender: Arc<BookmarkRecommender>,
    config: BookmarkAIConfig,
    bookmarks: RwLock<HashMap<String, Bookmark>>,
    collections: RwLock<HashMap<String, Collection>>,
}

/// Bookmark AI Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkAIConfig {
    /// Enable auto-categorization
    pub auto_categorize: bool,
    /// Enable duplicate detection
    pub detect_duplicates: bool,
    /// Enable smart recommendations
    pub recommendations: bool,
    /// Enable auto-tagging
    pub auto_tag: bool,
    /// Maximum bookmarks
    pub max_bookmarks: usize,
    /// Enable content analysis
    pub analyze_content: bool,
    /// Enable favicon fetching
    pub fetch_favicons: bool,
    /// Duplicate similarity threshold (0.0 - 1.0)
    pub duplicate_threshold: f32,
    /// Auto-organize collections
    pub auto_organize: bool,
    /// Learning from user behavior
    pub learning_enabled: bool,
}

impl Default for BookmarkAIConfig {
    fn default() -> Self {
        Self {
            auto_categorize: true,
            detect_duplicates: true,
            recommendations: true,
            auto_tag: true,
            max_bookmarks: 10000,
            analyze_content: true,
            fetch_favicons: true,
            duplicate_threshold: 0.85,
            auto_organize: true,
            learning_enabled: true,
        }
    }
}

/// Bookmark entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    /// Unique identifier
    pub id: String,
    /// URL
    pub url: String,
    /// Title
    pub title: String,
    /// Description
    pub description: Option<String>,
    /// Tags
    pub tags: Vec<String>,
    /// Category
    pub category: Option<BookmarkCategory>,
    /// Collection IDs
    pub collections: Vec<String>,
    /// Date added
    pub added_at: DateTime<Utc>,
    /// Date last visited
    pub last_visited: Option<DateTime<Utc>>,
    /// Visit count
    pub visit_count: usize,
    /// Favicon URL
    pub favicon: Option<String>,
    /// User notes
    pub notes: Option<String>,
    /// Rating (1-5)
    pub rating: Option<u8>,
    /// Priority (0-100)
    pub priority: u8,
    /// Whether it's archived
    pub archived: bool,
    /// AI-generated metadata
    pub ai_metadata: Option<AiMetadata>,
    /// Content hash for duplicate detection
    pub content_hash: Option<String>,
}

/// AI-generated metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMetadata {
    /// Auto-generated summary
    pub summary: Option<String>,
    /// Auto-generated tags
    pub auto_tags: Vec<String>,
    /// Confidence scores
    pub category_confidence: f32,
    /// Keywords extracted
    pub keywords: Vec<String>,
    /// Reading time estimate (minutes)
    pub reading_time: Option<f32>,
    /// Content type
    pub content_type: String,
    /// Language
    pub language: String,
    /// Sentiment
    pub sentiment: String,
}

/// Bookmark category
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BookmarkCategory {
    Development,
    Design,
    Business,
    Education,
    Entertainment,
    News,
    Shopping,
    Social,
    Reference,
    Tools,
    Finance,
    Health,
    Travel,
    Food,
    Sports,
    Technology,
    Science,
    Art,
    Music,
    Reading,
    Work,
    Personal,
    Research,
    Documentation,
    Tutorial,
    API,
    Framework,
    Library,
    Blog,
    Forum,
    Other(String),
}

/// Collection of bookmarks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    /// Collection ID
    pub id: String,
    /// Collection name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Color for UI
    pub color: Option<String>,
    /// Icon
    pub icon: Option<String>,
    /// Bookmark IDs in this collection
    pub bookmarks: Vec<String>,
    /// Whether it's auto-generated
    pub auto_generated: bool,
    /// Creation date
    pub created_at: DateTime<Utc>,
    /// Last updated
    pub updated_at: DateTime<Utc>,
    /// Sort order
    pub sort_order: i32,
    /// Parent collection (for nested collections)
    pub parent_id: Option<String>,
}

/// Duplicate detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    /// Group ID
    pub id: String,
    /// Original bookmark (best match)
    pub original: Bookmark,
    /// Duplicate bookmarks
    pub duplicates: Vec<DuplicateEntry>,
    /// Similarity score
    pub similarity: f32,
    /// Suggested action
    pub suggested_action: DuplicateAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateEntry {
    pub bookmark: Bookmark,
    pub similarity: f32,
    pub match_type: MatchType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchType {
    ExactUrl,
    NormalizedUrl,
    ContentSimilar,
    TitleSimilar,
    Redirect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DuplicateAction {
    Merge,
    KeepNewest,
    KeepMostVisited,
    KeepBoth,
    DeleteDuplicates,
}

/// Bookmark recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkRecommendation {
    /// Recommended bookmark
    pub bookmark: Bookmark,
    /// Recommendation reason
    pub reason: RecommendationReason,
    /// Relevance score
    pub relevance: f32,
    /// Related bookmarks
    pub related: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationReason {
    SimilarTo { bookmark_id: String },
    FrequentlyVisited,
    RecentlyAdded,
    Trending,
    RelatedToSearch { query: String },
    SameCategory { category: BookmarkCategory },
    TimeBased { context: String },
    ContentSimilarity { topic: String },
}

/// Smart folder definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartFolder {
    pub id: String,
    pub name: String,
    pub query: SmartQuery,
    pub auto_update: bool,
    pub created_at: DateTime<Utc>,
}

/// Smart query for dynamic folders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartQuery {
    /// Tags to match
    pub tags: Vec<String>,
    /// Categories to include
    pub categories: Vec<BookmarkCategory>,
    /// Date range
    pub date_range: Option<DateRange>,
    /// Minimum visits
    pub min_visits: Option<usize>,
    /// Keywords in title/description
    pub keywords: Vec<String>,
    /// URL patterns
    pub url_patterns: Vec<String>,
    /// Exclude conditions
    pub exclude: Vec<SmartQuery>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Bookmark organizer
pub struct BookmarkOrganizer {
    rules: RwLock<Vec<OrganizationRule>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OrganizationRule {
    id: String,
    name: String,
    condition: RuleCondition,
    action: RuleAction,
    priority: i32,
    enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum RuleCondition {
    UrlContains(String),
    TitleContains(String),
    TagEquals(String),
    CategoryEquals(BookmarkCategory),
    DomainEquals(String),
    And(Vec<RuleCondition>),
    Or(Vec<RuleCondition>),
    Not(Box<RuleCondition>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum RuleAction {
    AddToCollection(String),
    AddTag(String),
    SetCategory(BookmarkCategory),
    SetPriority(u8),
    Archive,
}

/// Bookmark categorizer
pub struct BookmarkCategorizer {
    model: Arc<dyn CategorizationModel>,
    category_keywords: HashMap<BookmarkCategory, Vec<String>>,
}

/// Deduplicator
pub struct Deduplicator {
    url_normalizer: UrlNormalizer,
    content_hasher: ContentHasher,
}

/// Bookmark recommender
pub struct BookmarkRecommender {
    similarity_model: Arc<dyn SimilarityModel>,
    user_behavior: RwLock<UserBehavior>,
}

/// URL normalizer
struct UrlNormalizer;

/// Content hasher
struct ContentHasher;

/// User behavior tracking
#[derive(Debug, Default)]
struct UserBehavior {
    clicks: HashMap<String, usize>,
    time_spent: HashMap<String, f64>,
    searches: Vec<String>,
}

// Trait definitions
#[async_trait::async_trait]
pub trait CategorizationModel: Send + Sync {
    async fn categorize(&self, bookmark: &Bookmark) -> Result<(BookmarkCategory, f32), BookmarkError>;
}

#[async_trait::async_trait]
pub trait SimilarityModel: Send + Sync {
    async fn compute_similarity(&self, a: &Bookmark, b: &Bookmark) -> Result<f32, BookmarkError>;
}

/// Bookmark error
#[derive(Debug, thiserror::Error)]
pub enum BookmarkError {
    #[error("Bookmark not found: {0}")]
    NotFound(String),
    #[error("Duplicate URL: {0}")]
    DuplicateUrl(String),
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Categorization error: {0}")]
    CategorizationError(String),
    #[error("Storage error: {0}")]
    StorageError(String),
}

impl BookmarkAI {
    pub fn new(config: BookmarkAIConfig) -> Self {
        Self {
            organizer: Arc::new(BookmarkOrganizer::new()),
            categorizer: Arc::new(BookmarkCategorizer::new()),
            deduplicator: Arc::new(Deduplicator::new()),
            recommender: Arc::new(BookmarkRecommender::new()),
            config,
            bookmarks: RwLock::new(HashMap::new()),
            collections: RwLock::new(HashMap::new()),
        }
    }

    /// Add a new bookmark
    pub async fn add_bookmark(&self, mut bookmark: Bookmark) -> Result<Bookmark, BookmarkError> {
        // Validate URL
        if !self.is_valid_url(&bookmark.url) {
            return Err(BookmarkError::InvalidUrl(bookmark.url));
        }

        // Check for duplicates
        if self.config.detect_duplicates {
            let duplicates = self.find_duplicates(&bookmark.url).await;
            if !duplicates.is_empty() {
                return Err(BookmarkError::DuplicateUrl(bookmark.url));
            }
        }

        // Auto-categorize
        if self.config.auto_categorize && bookmark.category.is_none() {
            let (category, confidence) = self.categorizer.categorize(&bookmark).await?;
            if confidence > 0.5 {
                bookmark.category = Some(category);
            }
        }

        // Auto-tag
        if self.config.auto_tag {
            let tags = self.generate_tags(&bookmark).await;
            bookmark.tags.extend(tags);
            bookmark.tags.sort();
            bookmark.tags.dedup();
        }

        // Generate AI metadata
        if self.config.analyze_content {
            bookmark.ai_metadata = Some(self.analyze_bookmark_content(&bookmark).await);
        }

        // Apply organization rules
        let collections = self.organizer.apply_rules(&bookmark).await;
        bookmark.collections = collections;

        // Set ID and timestamp
        if bookmark.id.is_empty() {
            bookmark.id = Uuid::new_v4().to_string();
        }
        bookmark.added_at = Utc::now();

        // Store bookmark
        let mut bookmarks = self.bookmarks.write().await;
        bookmarks.insert(bookmark.id.clone(), bookmark.clone());

        Ok(bookmark)
    }

    /// Get bookmark by ID
    pub async fn get_bookmark(&self, id: &str) -> Option<Bookmark> {
        let bookmarks = self.bookmarks.read().await;
        bookmarks.get(id).cloned()
    }

    /// Update bookmark
    pub async fn update_bookmark(&self, id: &str, updates: BookmarkUpdates) -> Result<Bookmark, BookmarkError> {
        let mut bookmarks = self.bookmarks.write().await;
        
        if let Some(bookmark) = bookmarks.get_mut(id) {
            if let Some(title) = updates.title {
                bookmark.title = title;
            }
            if let Some(description) = updates.description {
                bookmark.description = Some(description);
            }
            if let Some(tags) = updates.tags {
                bookmark.tags = tags;
            }
            if let Some(category) = updates.category {
                bookmark.category = Some(category);
            }
            if let Some(notes) = updates.notes {
                bookmark.notes = Some(notes);
            }
            if let Some(rating) = updates.rating {
                bookmark.rating = Some(rating);
            }
            if let Some(priority) = updates.priority {
                bookmark.priority = priority;
            }
            if let Some(archived) = updates.archived {
                bookmark.archived = archived;
            }
            
            return Ok(bookmark.clone());
        }
        
        Err(BookmarkError::NotFound(id.to_string()))
    }

    /// Delete bookmark
    pub async fn delete_bookmark(&self, id: &str) -> Result<(), BookmarkError> {
        let mut bookmarks = self.bookmarks.write().await;
        
        if bookmarks.remove(id).is_some() {
            // Also remove from collections
            let mut collections = self.collections.write().await;
            for collection in collections.values_mut() {
                collection.bookmarks.retain(|bid| bid != id);
            }
            Ok(())
        } else {
            Err(BookmarkError::NotFound(id.to_string()))
        }
    }

    /// Search bookmarks
    pub async fn search(&self, query: &str) -> Vec<Bookmark> {
        let bookmarks = self.bookmarks.read().await;
        let query_lower = query.to_lowercase();
        
        let mut results: Vec<Bookmark> = bookmarks
            .values()
            .filter(|b| {
                b.title.to_lowercase().contains(&query_lower)
                    || b.url.to_lowercase().contains(&query_lower)
                    || b.description.as_ref().map_or(false, |d| d.to_lowercase().contains(&query_lower))
                    || b.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect();
        
        // Sort by relevance
        results.sort_by(|a, b| {
            let a_score = self.compute_search_score(a, &query_lower);
            let b_score = self.compute_search_score(b, &query_lower);
            b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        results
    }

    /// Find duplicates
    pub async fn find_duplicates(&self, url: &str) -> Vec<Bookmark> {
        let bookmarks = self.bookmarks.read().await;
        let normalized = self.deduplicator.normalize_url(url);
        
        bookmarks
            .values()
            .filter(|b| {
                let b_normalized = self.deduplicator.normalize_url(&b.url);
                b_normalized == normalized || self.deduplicator.are_similar(&b.url, url)
            })
            .cloned()
            .collect()
    }

    /// Get all duplicates
    pub async fn get_all_duplicates(&self) -> Vec<DuplicateGroup> {
        let bookmarks = self.bookmarks.read().await;
        let mut groups: Vec<DuplicateGroup> = Vec::new();
        let mut processed: HashSet<String> = HashSet::new();
        
        for bookmark in bookmarks.values() {
            if processed.contains(&bookmark.id) {
                continue;
            }
            
            let mut duplicates = Vec::new();
            
            for other in bookmarks.values() {
                if other.id != bookmark.id && !processed.contains(&other.id) {
                    let similarity = self.compute_similarity(bookmark, other).await;
                    if similarity >= self.config.duplicate_threshold {
                        duplicates.push(DuplicateEntry {
                            bookmark: other.clone(),
                            similarity,
                            match_type: MatchType::ContentSimilar,
                        });
                        processed.insert(other.id.clone());
                    }
                }
            }
            
            if !duplicates.is_empty() {
                processed.insert(bookmark.id.clone());
                groups.push(DuplicateGroup {
                    id: Uuid::new_v4().to_string(),
                    original: bookmark.clone(),
                    duplicates,
                    similarity: 1.0,
                    suggested_action: DuplicateAction::KeepMostVisited,
                });
            }
        }
        
        groups
    }

    /// Get recommendations
    pub async fn get_recommendations(&self, context: &str) -> Vec<BookmarkRecommendation> {
        if !self.config.recommendations {
            return Vec::new();
        }
        
        self.recommender.get_recommendations(context, &self.bookmarks.read().await).await
    }

    /// Create collection
    pub async fn create_collection(&self, name: &str, description: Option<String>) -> Collection {
        let collection = Collection {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            description,
            color: None,
            icon: None,
            bookmarks: Vec::new(),
            auto_generated: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            sort_order: 0,
            parent_id: None,
        };
        
        let mut collections = self.collections.write().await;
        collections.insert(collection.id.clone(), collection.clone());
        
        collection
    }

    /// Add bookmark to collection
    pub async fn add_to_collection(&self, bookmark_id: &str, collection_id: &str) -> Result<(), BookmarkError> {
        let mut collections = self.collections.write().await;
        
        if let Some(collection) = collections.get_mut(collection_id) {
            if !collection.bookmarks.contains(&bookmark_id.to_string()) {
                collection.bookmarks.push(bookmark_id.to_string());
                collection.updated_at = Utc::now();
            }
            Ok(())
        } else {
            Err(BookmarkError::NotFound(collection_id.to_string()))
        }
    }

    /// Get bookmarks by category
    pub async fn get_by_category(&self, category: &BookmarkCategory) -> Vec<Bookmark> {
        let bookmarks = self.bookmarks.read().await;
        bookmarks
            .values()
            .filter(|b| b.category.as_ref() == Some(category))
            .cloned()
            .collect()
    }

    /// Get bookmarks by tag
    pub async fn get_by_tag(&self, tag: &str) -> Vec<Bookmark> {
        let bookmarks = self.bookmarks.read().await;
        bookmarks
            .values()
            .filter(|b| b.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)))
            .cloned()
            .collect()
    }

    /// Get all tags
    pub async fn get_all_tags(&self) -> Vec<(String, usize)> {
        let bookmarks = self.bookmarks.read().await;
        let mut tag_counts: HashMap<String, usize> = HashMap::new();
        
        for bookmark in bookmarks.values() {
            for tag in &bookmark.tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }
        
        let mut tags: Vec<_> = tag_counts.into_iter().collect();
        tags.sort_by(|a, b| b.1.cmp(&a.1));
        tags
    }

    /// Auto-organize all bookmarks
    pub async fn auto_organize(&self) -> Result<OrganizationResult, BookmarkError> {
        let mut result = OrganizationResult::default();
        let bookmarks = self.bookmarks.read().await;
        
        for bookmark in bookmarks.values() {
            let suggested_collections = self.organizer.apply_rules(bookmark).await;
            for collection_id in suggested_collections {
                result.moved_bookmarks.push((bookmark.id.clone(), collection_id));
            }
        }
        
        Ok(result)
    }

    /// Record visit
    pub async fn record_visit(&self, id: &str) {
        let mut bookmarks = self.bookmarks.write().await;
        if let Some(bookmark) = bookmarks.get_mut(id) {
            bookmark.visit_count += 1;
            bookmark.last_visited = Some(Utc::now());
        }
    }

    // Helper methods
    fn is_valid_url(&self, url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://")
    }

    async fn generate_tags(&self, bookmark: &Bookmark) -> Vec<String> {
        let mut tags = Vec::new();
        
        // Extract domain as tag
        if let Some(domain) = self.extract_domain(&bookmark.url) {
            tags.push(domain);
        }
        
        // Add category as tag
        if let Some(category) = &bookmark.category {
            tags.push(format!("{:?}", category).to_lowercase());
        }
        
        tags
    }

    fn extract_domain(&self, url: &str) -> Option<String> {
        url::Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.to_string()))
    }

    async fn analyze_bookmark_content(&self, bookmark: &Bookmark) -> AiMetadata {
        AiMetadata {
            summary: bookmark.description.clone(),
            auto_tags: bookmark.tags.clone(),
            category_confidence: 0.8,
            keywords: Vec::new(),
            reading_time: None,
            content_type: "article".to_string(),
            language: "en".to_string(),
            sentiment: "neutral".to_string(),
        }
    }

    async fn compute_similarity(&self, a: &Bookmark, b: &Bookmark) -> f32 {
        // URL similarity
        let url_sim = if a.url == b.url { 1.0 } else { 0.0 };
        
        // Title similarity
        let title_sim = if a.title.to_lowercase() == b.title.to_lowercase() { 1.0 } else { 0.0 };
        
        // Tag similarity
        let tags_a: HashSet<_> = a.tags.iter().cloned().collect();
        let tags_b: HashSet<_> = b.tags.iter().cloned().collect();
        let tag_sim = if tags_a.is_empty() || tags_b.is_empty() {
            0.0
        } else {
            tags_a.intersection(&tags_b).count() as f32 / tags_a.union(&tags_b).count() as f32
        };
        
        0.5 * url_sim + 0.3 * title_sim + 0.2 * tag_sim
    }

    fn compute_search_score(&self, bookmark: &Bookmark, query: &str) -> f32 {
        let mut score = 0.0;
        
        // Title match
        if bookmark.title.to_lowercase().contains(query) {
            score += 0.4;
        }
        
        // URL match
        if bookmark.url.to_lowercase().contains(query) {
            score += 0.2;
        }
        
        // Tag match
        if bookmark.tags.iter().any(|t| t.to_lowercase().contains(query)) {
            score += 0.2;
        }
        
        // Visit count boost
        score += (bookmark.visit_count as f32).min(0.2);
        
        score
    }
}

/// Bookmark updates
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BookmarkUpdates {
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub category: Option<BookmarkCategory>,
    pub notes: Option<String>,
    pub rating: Option<u8>,
    pub priority: Option<u8>,
    pub archived: Option<bool>,
}

/// Organization result
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrganizationResult {
    pub moved_bookmarks: Vec<(String, String)>,
    pub tags_added: usize,
    pub categories_set: usize,
}

impl BookmarkOrganizer {
    fn new() -> Self {
        Self {
            rules: RwLock::new(Vec::new()),
        }
    }

    pub async fn apply_rules(&self, _bookmark: &Bookmark) -> Vec<String> {
        // Placeholder - would apply organization rules
        Vec::new()
    }
}

impl BookmarkCategorizer {
    fn new() -> Self {
        Self {
            model: Arc::new(PlaceholderCategorizationModel),
            category_keywords: Self::init_keywords(),
        }
    }

    fn init_keywords() -> HashMap<BookmarkCategory, Vec<String>> {
        let mut map = HashMap::new();
        
        map.insert(BookmarkCategory::Development, vec![
            "github", "stackoverflow", "npm", "api", "code", "programming"
        ]);
        map.insert(BookmarkCategory::Technology, vec![
            "tech", "software", "hardware", "computer", "digital"
        ]);
        map.insert(BookmarkCategory::News, vec![
            "news", "article", "breaking", "report"
        ]);
        map.insert(BookmarkCategory::Education, vec![
            "course", "tutorial", "learn", "education", "university"
        ]);
        map.insert(BookmarkCategory::Shopping, vec![
            "shop", "store", "buy", "cart", "product"
        ]);
        
        map
    }

    pub async fn categorize(&self, bookmark: &Bookmark) -> Result<(BookmarkCategory, f32), BookmarkError> {
        self.model.categorize(bookmark).await
    }
}

impl Deduplicator {
    fn new() -> Self {
        Self {
            url_normalizer: UrlNormalizer,
            content_hasher: ContentHasher,
        }
    }

    pub fn normalize_url(&self, url: &str) -> String {
        let mut normalized = url.to_lowercase();
        
        // Remove protocol
        normalized = normalized.replace("https://", "").replace("http://", "");
        
        // Remove www
        normalized = normalized.replace("www.", "");
        
        // Remove trailing slash
        if normalized.ends_with('/') {
            normalized.pop();
        }
        
        // Remove common tracking parameters
        if let Some(idx) = normalized.find('?') {
            let mut params: Vec<&str> = normalized[idx + 1..].split('&').collect();
            params.retain(|p| !p.starts_with("utm_") && !p.starts_with("ref="));
            if params.is_empty() {
                normalized = normalized[..idx].to_string();
            } else {
                normalized = format!("{}?{}", &normalized[..idx], params.join("&"));
            }
        }
        
        normalized
    }

    pub fn are_similar(&self, url1: &str, url2: &str) -> bool {
        self.normalize_url(url1) == self.normalize_url(url2)
    }
}

impl BookmarkRecommender {
    fn new() -> Self {
        Self {
            similarity_model: Arc::new(PlaceholderSimilarityModel),
            user_behavior: RwLock::new(UserBehavior::default()),
        }
    }

    pub async fn get_recommendations(&self, _context: &str, bookmarks: &HashMap<String, Bookmark>) -> Vec<BookmarkRecommendation> {
        let mut recommendations = Vec::new();
        
        // Get most visited bookmarks
        let mut sorted: Vec<_> = bookmarks.values().collect();
        sorted.sort_by(|a, b| b.visit_count.cmp(&a.visit_count));
        
        for bookmark in sorted.into_iter().take(5) {
            recommendations.push(BookmarkRecommendation {
                bookmark: bookmark.clone(),
                reason: RecommendationReason::FrequentlyVisited,
                relevance: bookmark.visit_count as f32 / 100.0,
                related: Vec::new(),
            });
        }
        
        recommendations
    }
}

// Placeholder implementations
struct PlaceholderCategorizationModel;
struct PlaceholderSimilarityModel;

#[async_trait::async_trait]
impl CategorizationModel for PlaceholderCategorizationModel {
    async fn categorize(&self, bookmark: &Bookmark) -> Result<(BookmarkCategory, f32), BookmarkError> {
        let url_lower = bookmark.url.to_lowercase();
        let title_lower = bookmark.title.to_lowercase();
        
        // Simple keyword-based categorization
        let category = if url_lower.contains("github") || url_lower.contains("stackoverflow") {
            BookmarkCategory::Development
        } else if url_lower.contains("news") || title_lower.contains("news") {
            BookmarkCategory::News
        } else if url_lower.contains("shop") || url_lower.contains("store") {
            BookmarkCategory::Shopping
        } else if url_lower.contains("tutorial") || url_lower.contains("learn") {
            BookmarkCategory::Education
        } else {
            BookmarkCategory::Other("general".to_string())
        };
        
        Ok((category, 0.7))
    }
}

#[async_trait::async_trait]
impl SimilarityModel for PlaceholderSimilarityModel {
    async fn compute_similarity(&self, a: &Bookmark, b: &Bookmark) -> Result<f32, BookmarkError> {
        let url_sim = if a.url == b.url { 1.0 } else { 0.0 };
        let title_sim = if a.title.to_lowercase() == b.title.to_lowercase() { 1.0 } else { 0.0 };
        Ok(0.5 * url_sim + 0.5 * title_sim)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = BookmarkAIConfig::default();
        assert!(config.auto_categorize);
        assert!(config.detect_duplicates);
        assert!(config.recommendations);
    }

    #[tokio::test]
    async fn test_add_bookmark() {
        let bookmark_ai = BookmarkAI::new(BookmarkAIConfig::default());
        
        let bookmark = Bookmark {
            id: String::new(),
            url: "https://example.com".to_string(),
            title: "Example Site".to_string(),
            description: None,
            tags: vec!["example".to_string()],
            category: None,
            collections: Vec::new(),
            added_at: Utc::now(),
            last_visited: None,
            visit_count: 0,
            favicon: None,
            notes: None,
            rating: None,
            priority: 50,
            archived: false,
            ai_metadata: None,
            content_hash: None,
        };
        
        let result = bookmark_ai.add_bookmark(bookmark).await.unwrap();
        assert!(!result.id.is_empty());
    }

    #[test]
    fn test_url_normalization() {
        let deduplicator = Deduplicator::new();
        
        assert_eq!(
            deduplicator.normalize_url("https://www.example.com/"),
            "example.com"
        );
        
        assert_eq!(
            deduplicator.normalize_url("http://Example.COM/page?utm_source=test"),
            "example.com/page"
        );
    }
}