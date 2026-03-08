//! Smart Search Module
//! 
//! AI-powered search engine with semantic understanding,
//! contextual ranking, and intelligent result aggregation.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};

/// Smart Search Engine
pub struct SmartSearch {
    indexer: Arc<SearchIndexer>,
    ranker: Arc<ResultRanker>,
    semantic_search: Arc<SemanticSearch>,
    config: SearchConfig,
    search_history: RwLock<Vec<SearchHistoryEntry>>,
    user_context: RwLock<UserSearchContext>,
}

/// Search configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Enable semantic search
    pub semantic_enabled: bool,
    /// Enable personalization
    pub personalization_enabled: bool,
    /// Maximum results per query
    pub max_results: usize,
    /// Enable search suggestions
    pub suggestions_enabled: bool,
    /// Minimum relevance score
    pub min_relevance: f32,
    /// Enable spell correction
    pub spell_correction: bool,
    /// Enable query expansion
    pub query_expansion: bool,
    /// Search timeout in milliseconds
    pub timeout_ms: u64,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            semantic_enabled: true,
            personalization_enabled: true,
            max_results: 100,
            suggestions_enabled: true,
            min_relevance: 0.1,
            spell_correction: true,
            query_expansion: true,
            timeout_ms: 5000,
        }
    }
}

/// Search query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Original query text
    pub text: String,
    /// Processed query terms
    pub terms: Vec<String>,
    /// Query intent
    pub intent: QueryIntent,
    /// Filters to apply
    pub filters: Vec<SearchFilter>,
    /// Sort order
    pub sort: SortOrder,
    /// Page number
    pub page: usize,
    /// Results per page
    pub per_page: usize,
}

/// Query intent classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QueryIntent {
    Informational,
    Navigational,
    Transactional,
    Comparison,
    Definition,
    HowTo,
    Local,
    News,
    Research,
    Unknown,
}

/// Search filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: FilterValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOperator {
    Equals,
    NotEquals,
    Contains,
    GreaterThan,
    LessThan,
    Between,
    In,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterValue {
    String(String),
    Number(f64),
    Boolean(bool),
    DateRange(DateTime<Utc>, DateTime<Utc>),
    List(Vec<String>),
}

/// Sort order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Relevance,
    DateDescending,
    DateAscending,
    Popularity,
    Alphabetical,
    Custom(String),
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Result ID
    pub id: String,
    /// Title
    pub title: String,
    /// URL or identifier
    pub url: String,
    /// Snippet/description
    pub snippet: String,
    /// Relevance score
    pub relevance_score: f32,
    /// Content type
    pub content_type: ContentType,
    /// Metadata
    pub metadata: HashMap<String, String>,
    /// Highlights
    pub highlights: Vec<Highlight>,
    /// Source information
    pub source: SourceInfo,
    /// When indexed
    pub indexed_at: DateTime<Utc>,
}

/// Content type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContentType {
    WebPage,
    Document,
    Image,
    Video,
    Audio,
    Code,
    Data,
    News,
    Social,
    Other,
}

/// Highlight in search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Highlight {
    pub field: String,
    pub snippet: String,
    pub positions: Vec<(usize, usize)>,
}

/// Source information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    pub domain: String,
    pub name: String,
    pub authority_score: f32,
    pub category: String,
}

/// Search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    /// Query that was executed
    pub query: SearchQuery,
    /// Search results
    pub results: Vec<SearchResult>,
    /// Total matching documents
    pub total: usize,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Query suggestions
    pub suggestions: Vec<String>,
    /// Related searches
    pub related_searches: Vec<String>,
    /// Facets for filtering
    pub facets: Vec<SearchFacet>,
    /// Did you mean
    pub spell_correction: Option<String>,
}

/// Search facet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFacet {
    pub name: String,
    pub values: Vec<FacetValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacetValue {
    pub value: String,
    pub count: usize,
    pub selected: bool,
}

/// Search history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHistoryEntry {
    pub query: String,
    pub timestamp: DateTime<Utc>,
    pub results_clicked: Vec<String>,
    pub session_id: String,
}

/// User search context for personalization
#[derive(Debug, Default)]
pub struct UserSearchContext {
    pub interests: HashMap<String, f32>,
    pub frequent_sites: HashMap<String, u32>,
    pub recent_topics: Vec<String>,
    pub preferred_content_types: HashMap<ContentType, f32>,
}

/// Search indexer
pub struct SearchIndexer {
    index: RwLock<SearchIndex>,
}

struct SearchIndex {
    documents: HashMap<String, IndexedDocument>,
    inverted_index: HashMap<String, HashSet<String>>,
    embeddings: HashMap<String, Vec<f32>>,
}

#[derive(Debug, Clone)]
struct IndexedDocument {
    id: String,
    title: String,
    content: String,
    url: String,
    metadata: HashMap<String, String>,
    embedding: Option<Vec<f32>>,
    indexed_at: DateTime<Utc>,
}

/// Result ranker
pub struct ResultRanker {
    scoring_weights: ScoringWeights,
}

#[derive(Debug, Clone)]
pub struct ScoringWeights {
    pub relevance: f32,
    pub authority: f32,
    pub freshness: f32,
    pub personalization: f32,
    pub click_through: f32,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            relevance: 0.4,
            authority: 0.2,
            freshness: 0.15,
            personalization: 0.15,
            click_through: 0.1,
        }
    }
}

/// Semantic search using embeddings
pub struct SemanticSearch {
    model: Arc<dyn EmbeddingModel>,
    embedding_cache: RwLock<HashMap<String, Vec<f32>>>,
}

/// Embedding model trait
#[async_trait::async_trait]
pub trait EmbeddingModel: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, SearchError>;
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, SearchError>;
}

/// Search error
#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    #[error("Index error: {0}")]
    IndexError(String),
    #[error("Query error: {0}")]
    QueryError(String),
    #[error("Embedding error: {0}")]
    EmbeddingError(String),
    #[error("Timeout")]
    Timeout,
    #[error("No results found")]
    NoResults,
}

impl SmartSearch {
    pub fn new(config: SearchConfig) -> Self {
        Self {
            indexer: Arc::new(SearchIndexer::new()),
            ranker: Arc::new(ResultRanker::new()),
            semantic_search: Arc::new(SemanticSearch::new_placeholder()),
            config,
            search_history: RwLock::new(Vec::new()),
            user_context: RwLock::new(UserSearchContext::default()),
        }
    }

    /// Execute a search query
    pub async fn search(&self, query: &str) -> Result<SearchResponse, SearchError> {
        let start = std::time::Instant::now();
        
        // Parse and process query
        let search_query = self.parse_query(query).await?;
        
        // Get initial results from indexer
        let mut results = self.indexer.search(&search_query).await?;
        
        // Apply semantic search if enabled
        if self.config.semantic_enabled {
            let semantic_results = self.semantic_search.search(&search_query).await?;
            results = self.merge_results(results, semantic_results);
        }
        
        // Rank results
        results = self.ranker.rank(results, &search_query, &self.user_context.read().await).await;
        
        // Apply pagination
        let total = results.len();
        let start_idx = search_query.page * search_query.per_page;
        let end_idx = (start_idx + search_query.per_page).min(results.len());
        results = results.into_iter().skip(start_idx).take(end_idx - start_idx).collect();
        
        // Generate suggestions
        let suggestions = if self.config.suggestions_enabled {
            self.generate_suggestions(&search_query).await?
        } else {
            Vec::new()
        };
        
        // Get related searches
        let related = self.get_related_searches(&search_query).await;
        
        // Generate facets
        let facets = self.generate_facets(&results).await;
        
        // Record search history
        self.record_search(&search_query).await;
        
        Ok(SearchResponse {
            query: search_query,
            results,
            total,
            execution_time_ms: start.elapsed().as_millis() as u64,
            suggestions,
            related_searches: related,
            facets,
            spell_correction: None,
        })
    }

    /// Parse query string into structured query
    async fn parse_query(&self, query: &str) -> Result<SearchQuery, SearchError> {
        let text = query.trim().to_lowercase();
        
        // Extract terms
        let terms = self.tokenize(&text);
        
        // Detect intent
        let intent = self.detect_intent(&text);
        
        // Apply spell correction if enabled
        let corrected = if self.config.spell_correction {
            self.correct_spelling(&text).await
        } else {
            None
        };
        
        // Expand query if enabled
        let expanded_terms = if self.config.query_expansion {
            self.expand_query(&terms).await
        } else {
            terms.clone()
        };
        
        Ok(SearchQuery {
            text: corrected.unwrap_or(text),
            terms: expanded_terms,
            intent,
            filters: Vec::new(),
            sort: SortOrder::Relevance,
            page: 0,
            per_page: self.config.max_results,
        })
    }

    /// Tokenize query text
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.split_whitespace()
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|s| !s.is_empty() && s.len() > 1)
            .collect()
    }

    /// Detect query intent
    fn detect_intent(&self, query: &str) -> QueryIntent {
        let query_lower = query.to_lowercase();
        
        // Check for navigational intent
        if query_lower.contains("go to") || query_lower.contains("open") {
            return QueryIntent::Navigational;
        }
        
        // Check for transactional intent
        if query_lower.contains("buy") || query_lower.contains("purchase") || query_lower.contains("order") {
            return QueryIntent::Transactional;
        }
        
        // Check for comparison intent
        if query_lower.contains("vs") || query_lower.contains("compare") || query_lower.contains("difference") {
            return QueryIntent::Comparison;
        }
        
        // Check for definition intent
        if query_lower.starts_with("what is") || query_lower.starts_with("define") {
            return QueryIntent::Definition;
        }
        
        // Check for how-to intent
        if query_lower.starts_with("how to") || query_lower.starts_with("how do") {
            return QueryIntent::HowTo;
        }
        
        // Check for local intent
        if query_lower.contains("near me") || query_lower.contains("nearby") {
            return QueryIntent::Local;
        }
        
        // Check for news intent
        if query_lower.contains("news") || query_lower.contains("latest") {
            return QueryIntent::News;
        }
        
        // Check for research intent
        if query_lower.contains("research") || query_lower.contains("study") || query_lower.contains("paper") {
            return QueryIntent::Research;
        }
        
        QueryIntent::Informational
    }

    /// Correct spelling
    async fn correct_spelling(&self, _query: &str) -> Option<String> {
        // Placeholder - would integrate with spell checker
        None
    }

    /// Expand query with related terms
    async fn expand_query(&self, terms: &[String]) -> Vec<String> {
        // Placeholder - would use word embeddings or thesaurus
        terms.to_vec()
    }

    /// Generate search suggestions
    async fn generate_suggestions(&self, query: &SearchQuery) -> Result<Vec<String>, SearchError> {
        let mut suggestions = Vec::new();
        
        // Prefix-based suggestions
        for term in &query.terms {
            suggestions.push(format!("{} tutorial", term));
            suggestions.push(format!("{} guide", term));
            suggestions.push(format!("best {}", term));
        }
        
        Ok(suggestions.into_iter().take(5).collect())
    }

    /// Get related searches
    async fn get_related_searches(&self, query: &SearchQuery) -> Vec<String> {
        let mut related = Vec::new();
        
        for term in &query.terms {
            related.push(format!("{} examples", term));
            related.push(format!("{} alternatives", term));
        }
        
        related.into_iter().take(5).collect()
    }

    /// Generate facets from results
    async fn generate_facets(&self, results: &[SearchResult]) -> Vec<SearchFacet> {
        let mut content_types: HashMap<String, usize> = HashMap::new();
        
        for result in results {
            *content_types.entry(format!("{:?}", result.content_type)).or_insert(0) += 1;
        }
        
        vec![SearchFacet {
            name: "content_type".to_string(),
            values: content_types
                .into_iter()
                .map(|(k, v)| FacetValue {
                    value: k,
                    count: v,
                    selected: false,
                })
                .collect(),
        }]
    }

    /// Merge traditional and semantic results
    fn merge_results(&self, traditional: Vec<SearchResult>, semantic: Vec<SearchResult>) -> Vec<SearchResult> {
        let mut seen: HashSet<String> = HashSet::new();
        let mut merged = Vec::new();
        
        for result in traditional.into_iter().chain(semantic.into_iter()) {
            if seen.insert(result.id.clone()) {
                merged.push(result);
            }
        }
        
        merged
    }

    /// Record search in history
    async fn record_search(&self, query: &SearchQuery) {
        let mut history = self.search_history.write().await;
        history.push(SearchHistoryEntry {
            query: query.text.clone(),
            timestamp: Utc::now(),
            results_clicked: Vec::new(),
            session_id: uuid::Uuid::new_v4().to_string(),
        });
        
        // Limit history size
        if history.len() > 1000 {
            history.remove(0);
        }
    }

    /// Index a document
    pub async fn index_document(&self, doc: DocumentToIndex) -> Result<(), SearchError> {
        self.indexer.index(doc).await
    }

    /// Learn from user interaction
    pub async fn record_click(&self, result_id: &str, query: &str) {
        let mut context = self.user_context.write().await;
        
        // Update interests based on clicked result
        *context.interests.entry(query.to_string()).or_insert(0.0) += 0.1;
        
        // Record in history
        let mut history = self.search_history.write().await;
        if let Some(last) = history.last_mut() {
            if last.query == query {
                last.results_clicked.push(result_id.to_string());
            }
        }
    }
}

/// Document to be indexed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentToIndex {
    pub id: String,
    pub title: String,
    pub content: String,
    pub url: String,
    pub content_type: ContentType,
    pub metadata: HashMap<String, String>,
}

impl SearchIndexer {
    pub fn new() -> Self {
        Self {
            index: RwLock::new(SearchIndex {
                documents: HashMap::new(),
                inverted_index: HashMap::new(),
                embeddings: HashMap::new(),
            }),
        }
    }

    pub async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, SearchError> {
        let index = self.index.read().await;
        let mut results = Vec::new();
        
        // Find documents matching query terms
        let mut matching_docs: HashSet<String> = HashSet::new();
        for term in &query.terms {
            if let Some(doc_ids) = index.inverted_index.get(term) {
                matching_docs.extend(doc_ids.iter().cloned());
            }
        }
        
        // Convert to search results
        for doc_id in matching_docs {
            if let Some(doc) = index.documents.get(&doc_id) {
                results.push(SearchResult {
                    id: doc.id.clone(),
                    title: doc.title.clone(),
                    url: doc.url.clone(),
                    snippet: doc.content.chars().take(200).collect(),
                    relevance_score: 1.0, // Will be adjusted by ranker
                    content_type: ContentType::WebPage,
                    metadata: doc.metadata.clone(),
                    highlights: Vec::new(),
                    source: SourceInfo {
                        domain: "example.com".to_string(),
                        name: "Example".to_string(),
                        authority_score: 0.8,
                        category: "general".to_string(),
                    },
                    indexed_at: doc.indexed_at,
                });
            }
        }
        
        Ok(results)
    }

    pub async fn index(&self, doc: DocumentToIndex) -> Result<(), SearchError> {
        let mut index = self.index.write().await;
        
        // Add to documents
        let indexed_doc = IndexedDocument {
            id: doc.id.clone(),
            title: doc.title,
            content: doc.content.clone(),
            url: doc.url,
            metadata: doc.metadata,
            embedding: None,
            indexed_at: Utc::now(),
        };
        
        // Build inverted index
        let terms = doc.content.to_lowercase().split_whitespace().collect::<Vec<_>>();
        for term in terms {
            index
                .inverted_index
                .entry(term.to_string())
                .or_insert_with(HashSet::new)
                .insert(doc.id.clone());
        }
        
        index.documents.insert(doc.id, indexed_doc);
        
        Ok(())
    }
}

impl ResultRanker {
    pub fn new() -> Self {
        Self {
            scoring_weights: ScoringWeights::default(),
        }
    }

    pub async fn rank(
        &self,
        results: Vec<SearchResult>,
        query: &SearchQuery,
        user_context: &UserSearchContext,
    ) -> Vec<SearchResult> {
        let mut scored_results: Vec<(SearchResult, f32)> = results
            .into_iter()
            .map(|r| {
                let score = self.calculate_score(&r, query, user_context);
                (r, score)
            })
            .collect();
        
        // Sort by score descending
        scored_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Update relevance scores
        scored_results
            .into_iter()
            .map(|(mut r, score)| {
                r.relevance_score = score;
                r
            })
            .collect()
    }

    fn calculate_score(
        &self,
        result: &SearchResult,
        _query: &SearchQuery,
        user_context: &UserSearchContext,
    ) -> f32 {
        let mut score = result.relevance_score * self.scoring_weights.relevance;
        
        // Add authority score
        score += result.source.authority_score * self.scoring_weights.authority;
        
        // Add personalization score
        if let Some(interest) = user_context.interests.get(&result.title.to_lowercase()) {
            score += interest * self.scoring_weights.personalization;
        }
        
        // Add freshness score
        let age_hours = (Utc::now() - result.indexed_at).num_hours() as f32;
        let freshness = 1.0 / (1.0 + age_hours / 168.0); // Decay over a week
        score += freshness * self.scoring_weights.freshness;
        
        score
    }
}

impl SemanticSearch {
    fn new_placeholder() -> Self {
        Self {
            model: Arc::new(PlaceholderEmbeddingModel),
            embedding_cache: RwLock::new(HashMap::new()),
        }
    }

    pub async fn search(&self, _query: &SearchQuery) -> Result<Vec<SearchResult>, SearchError> {
        // Placeholder - would use embedding similarity
        Ok(Vec::new())
    }
}

struct PlaceholderEmbeddingModel;

#[async_trait::async_trait]
impl EmbeddingModel for PlaceholderEmbeddingModel {
    async fn embed(&self, _text: &str) -> Result<Vec<f32>, SearchError> {
        Ok(vec![0.0; 768])
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, SearchError> {
        Ok(texts.iter().map(|_| vec![0.0; 768]).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_config_defaults() {
        let config = SearchConfig::default();
        assert!(config.semantic_enabled);
        assert!(config.personalization_enabled);
        assert_eq!(config.max_results, 100);
    }

    #[test]
    fn test_intent_detection() {
        let search = SmartSearch::new(SearchConfig::default());
        
        assert_eq!(search.detect_intent("how to code"), QueryIntent::HowTo);
        assert_eq!(search.detect_intent("what is rust"), QueryIntent::Definition);
        assert_eq!(search.detect_intent("buy laptop"), QueryIntent::Transactional);
    }
}