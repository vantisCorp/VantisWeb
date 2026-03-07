use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use super::Bookmark;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkSearchResult {
    pub bookmark: Bookmark,
    pub relevance_score: f64,
    pub match_details: Vec<MatchDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchDetail {
    pub field: String,
    pub matched_text: String,
    pub match_type: MatchType,
    pub position: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchType {
    Exact,
    Prefix,
    Contains,
    Fuzzy,
}

#[derive(Debug, Clone, Copy)]
pub enum SearchField {
    Title,
    Url,
    Description,
    Tags,
    All,
}

#[derive(Debug, Clone)]
pub struct BookmarkSearchIndex {
    bookmarks: Arc<RwLock<HashMap<Uuid, Bookmark>>>,
    title_index: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
    url_index: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
    tag_index: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
    domain_index: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
    recent_searches: Arc<RwLock<Vec<SearchQuery>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub timestamp: DateTime<Utc>,
    pub results_count: usize,
    pub search_fields: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub fields: Vec<SearchField>,
    pub case_sensitive: bool,
    pub fuzzy_search: bool,
    pub max_results: usize,
    pub include_deleted: bool,
    pub min_relevance: f64,
    pub boost_favorites: bool,
    pub boost_recent: bool,
}

pub struct BookmarkSearchEngine {
    index: Arc<BookmarkSearchIndex>,
}

impl BookmarkSearchEngine {
    pub fn new() -> Self {
        Self {
            index: Arc::new(BookmarkSearchIndex::new()),
        }
    }

    pub async fn index_bookmark(&self, bookmark: &Bookmark) {
        self.index.index_bookmark(bookmark).await;
    }

    pub async fn remove_bookmark(&self, id: Uuid) {
        self.index.remove_bookmark(id).await;
    }

    pub async fn search(&self, query: &str, options: SearchOptions) -> Vec<BookmarkSearchResult> {
        self.index.search(query, options).await
    }

    pub async fn search_by_url(&self, url: &str) -> Vec<BookmarkSearchResult> {
        self.index.search_by_url(url).await
    }

    pub async fn search_by_domain(&self, domain: &str) -> Vec<BookmarkSearchResult> {
        self.index.search_by_domain(domain).await
    }

    pub async fn search_by_tag(&self, tag: &str) -> Vec<BookmarkSearchResult> {
        self.index.search_by_tag(tag).await
    }

    pub async fn get_suggestions(&self, partial: &str, limit: usize) -> Vec<String> {
        self.index.get_suggestions(partial, limit).await
    }

    pub async fn get_recent_searches(&self, limit: usize) -> Vec<SearchQuery> {
        self.index.get_recent_searches(limit).await
    }

    pub async fn clear_recent_searches(&self) {
        self.index.clear_recent_searches().await;
    }

    pub async fn get_search_stats(&self) -> SearchStats {
        self.index.get_search_stats().await
    }
}

impl BookmarkSearchIndex {
    pub fn new() -> Self {
        Self {
            bookmarks: Arc::new(RwLock::new(HashMap::new())),
            title_index: Arc::new(RwLock::new(HashMap::new())),
            url_index: Arc::new(RwLock::new(HashMap::new())),
            tag_index: Arc::new(RwLock::new(HashMap::new())),
            domain_index: Arc::new(RwLock::new(HashMap::new())),
            recent_searches: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn index_bookmark(&self, bookmark: &Bookmark) {
        let id = bookmark.id;
        
        // Add to main storage
        let mut bookmarks = self.bookmarks.write().await;
        bookmarks.insert(id, bookmark.clone());
        drop(bookmarks);
        
        // Index title
        let mut title_idx = self.title_index.write().await;
        for word in tokenize_text(&bookmark.title) {
            title_idx.entry(word)
                .or_insert_with(Vec::new)
                .push(id);
        }
        drop(title_idx);
        
        // Index URL
        let mut url_idx = self.url_index.write().await;
        for segment in tokenize_url(&bookmark.url) {
            url_idx.entry(segment)
                .or_insert_with(Vec::new)
                .push(id);
        }
        drop(url_idx);
        
        // Index tags
        let mut tag_idx = self.tag_index.write().await;
        for tag in &bookmark.tags {
            tag_idx.entry(tag.to_lowercase())
                .or_insert_with(Vec::new)
                .push(id);
        }
        drop(tag_idx);
        
        // Index domain
        let domain = extract_domain(&bookmark.url).to_lowercase();
        let mut domain_idx = self.domain_index.write().await;
        domain_idx.entry(domain)
            .or_insert_with(Vec::new)
            .push(id);
    }

    pub async fn remove_bookmark(&self, id: Uuid) {
        self.bookmarks.write().await.remove(&id);
        // Note: In production, would also clean up indexes
    }

    pub async fn search(&self, query: &str, options: SearchOptions) -> Vec<BookmarkSearchResult> {
        let query_lower = if options.case_sensitive {
            query.to_string()
        } else {
            query.to_lowercase()
        };
        
        let bookmarks = self.bookmarks.read().await;
        let mut results = Vec::new();
        
        for bookmark in bookmarks.values() {
            let mut match_details = Vec::new();
            let mut relevance_score = 0.0;
            
            // Search title
            if options.fields.contains(&SearchField::Title) || options.fields.contains(&SearchField::All) {
                let title = if options.case_sensitive {
                    bookmark.title.clone()
                } else {
                    bookmark.title.to_lowercase()
                };
                
                if title == query_lower {
                    relevance_score += 1.0;
                    match_details.push(MatchDetail {
                        field: "title".to_string(),
                        matched_text: bookmark.title.clone(),
                        match_type: MatchType::Exact,
                        position: Some(title.find(&query_lower).unwrap()),
                    });
                } else if title.starts_with(&query_lower) {
                    relevance_score += 0.8;
                    match_details.push(MatchDetail {
                        field: "title".to_string(),
                        matched_text: bookmark.title.clone(),
                        match_type: MatchType::Prefix,
                        position: Some(0),
                    });
                } else if title.contains(&query_lower) {
                    relevance_score += 0.6;
                    match_details.push(MatchDetail {
                        field: "title".to_string(),
                        matched_text: bookmark.title.clone(),
                        match_type: MatchType::Contains,
                        position: title.find(&query_lower),
                    });
                }
            }
            
            // Search URL
            if options.fields.contains(&SearchField::Url) || options.fields.contains(&SearchField::All) {
                let url = if options.case_sensitive {
                    bookmark.url.clone()
                } else {
                    bookmark.url.to_lowercase()
                };
                
                if url == query_lower {
                    relevance_score += 0.95;
                    match_details.push(MatchDetail {
                        field: "url".to_string(),
                        matched_text: bookmark.url.clone(),
                        match_type: MatchType::Exact,
                        position: Some(url.find(&query_lower).unwrap()),
                    });
                } else if url.contains(&query_lower) {
                    relevance_score += 0.5;
                    match_details.push(MatchDetail {
                        field: "url".to_string(),
                        matched_text: bookmark.url.clone(),
                        match_type: MatchType::Contains,
                        position: url.find(&query_lower),
                    });
                }
            }
            
            // Search description
            if options.fields.contains(&SearchField::All) {
                if let Some(ref desc) = bookmark.description {
                    let desc_lower = if options.case_sensitive {
                        desc.clone()
                    } else {
                        desc.to_lowercase()
                    };
                    
                    if desc_lower.contains(&query_lower) {
                        relevance_score += 0.4;
                    }
                }
            }
            
            // Search tags
            if options.fields.contains(&SearchField::Tags) || options.fields.contains(&SearchField::All) {
                for tag in &bookmark.tags {
                    let tag_lower = if options.case_sensitive {
                        tag.clone()
                    } else {
                        tag.to_lowercase()
                    };
                    
                    if tag_lower == query_lower {
                        relevance_score += 0.9;
                    } else if tag_lower.contains(&query_lower) {
                        relevance_score += 0.7;
                    }
                }
            }
            
            // Fuzzy search
            if options.fuzzy_search && relevance_score < 0.3 {
                let fuzzy_score = calculate_fuzzy_score(&query_lower, &bookmark.title.to_lowercase());
                if fuzzy_score > 0.6 {
                    relevance_score = fuzzy_score * 0.5;
                    match_details.push(MatchDetail {
                        field: "title".to_string(),
                        matched_text: bookmark.title.clone(),
                        match_type: MatchType::Fuzzy,
                        position: None,
                    });
                }
            }
            
            // Apply boosts
            if relevance_score > 0.0 {
                if options.boost_favorites && bookmark.is_favorite {
                    relevance_score += 0.1;
                }
                
                if options.boost_recent {
                    let days_ago = (Utc::now() - bookmark.created_at).num_days();
                    let recency_boost = 1.0 / (1.0 + days_ago as f64) * 0.05;
                    relevance_score += recency_boost;
                }
                
                // Visit count boost
                let visit_boost = (bookmark.visit_count as f64).min(100.0) / 100.0 * 0.1;
                relevance_score += visit_boost;
            }
            
            if relevance_score >= options.min_relevance {
                results.push(BookmarkSearchResult {
                    bookmark: bookmark.clone(),
                    relevance_score,
                    match_details,
                });
            }
        }
        
        // Record search
        let search_record = SearchQuery {
            query: query.to_string(),
            timestamp: Utc::now(),
            results_count: results.len(),
            search_fields: options.fields.iter()
                .map(|f| format!("{:?}", f))
                .collect(),
        };
        self.recent_searches.write().await.push(search_record);
        
        // Sort by relevance and limit
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        results.truncate(options.max_results);
        
        results
    }

    pub async fn search_by_url(&self, url: &str) -> Vec<BookmarkSearchResult> {
        let url_lower = url.to_lowercase();
        let bookmarks = self.bookmarks.read().await;
        
        let mut results = Vec::new();
        for bookmark in bookmarks.values() {
            if bookmark.url.to_lowercase() == url_lower {
                results.push(BookmarkSearchResult {
                    bookmark: bookmark.clone(),
                    relevance_score: 1.0,
                    match_details: vec![MatchDetail {
                        field: "url".to_string(),
                        matched_text: bookmark.url.clone(),
                        match_type: MatchType::Exact,
                        position: None,
                    }],
                });
            }
        }
        
        results
    }

    pub async fn search_by_domain(&self, domain: &str) -> Vec<BookmarkSearchResult> {
        let domain_lower = domain.to_lowercase();
        let bookmarks = self.bookmarks.read().await;
        
        let mut results = Vec::new();
        for bookmark in bookmarks.values() {
            let bookmark_domain = extract_domain(&bookmark.url).to_lowercase();
            if bookmark_domain == domain_lower || bookmark_domain.contains(&domain_lower) {
                results.push(BookmarkSearchResult {
                    bookmark: bookmark.clone(),
                    relevance_score: 1.0,
                    match_details: vec![MatchDetail {
                        field: "domain".to_string(),
                        matched_text: bookmark_domain,
                        match_type: MatchType::Contains,
                        position: None,
                    }],
                });
            }
        }
        
        results
    }

    pub async fn search_by_tag(&self, tag: &str) -> Vec<BookmarkSearchResult> {
        let tag_lower = tag.to_lowercase();
        let bookmarks = self.bookmarks.read().await;
        
        let mut results = Vec::new();
        for bookmark in bookmarks.values() {
            if bookmark.tags.iter().any(|t| t.to_lowercase() == tag_lower) {
                results.push(BookmarkSearchResult {
                    bookmark: bookmark.clone(),
                    relevance_score: 1.0,
                    match_details: vec![MatchDetail {
                        field: "tags".to_string(),
                        matched_text: tag.to_string(),
                        match_type: MatchType::Exact,
                        position: None,
                    }],
                });
            }
        }
        
        results
    }

    pub async fn get_suggestions(&self, partial: &str, limit: usize) -> Vec<String> {
        let partial_lower = partial.to_lowercase();
        let bookmarks = self.bookmarks.read().await;
        let mut suggestions = Vec::new();
        let mut seen = std::collections::HashSet::new();
        
        // Suggest from titles
        for bookmark in bookmarks.values() {
            if bookmark.title.to_lowercase().starts_with(&partial_lower) {
                if seen.insert(bookmark.title.clone()) {
                    suggestions.push(bookmark.title.clone());
                }
            }
            
            if suggestions.len() >= limit {
                break;
            }
        }
        
        suggestions
    }

    pub async fn get_recent_searches(&self, limit: usize) -> Vec<SearchQuery> {
        let searches = self.recent_searches.read().await;
        searches.iter().rev().take(limit).cloned().collect()
    }

    pub async fn clear_recent_searches(&self) {
        self.recent_searches.write().await.clear();
    }

    pub async fn get_search_stats(&self) -> SearchStats {
        let bookmarks = self.bookmarks.read().await;
        let searches = self.recent_searches.read().await;
        
        SearchStats {
            indexed_bookmarks: bookmarks.len(),
            total_searches: searches.len(),
            unique_domains: self.domain_index.read().await.len(),
            unique_tags: self.tag_index.read().await.len(),
        }
    }
}

fn tokenize_text(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
}

fn tokenize_url(url: &str) -> Vec<String> {
    url.to_lowercase()
        .split(&['/', '?', '=', '&', '-', '_', '.'][..])
        .filter(|s| !s.is_empty() && s.len() > 2)
        .map(|s| s.to_string())
        .collect()
}

fn extract_domain(url: &str) -> String {
    let url = url.trim();
    let url = url.strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .or_else(|| url.strip_prefix("www."))
        .unwrap_or(url);
    
    url.split('/').next().unwrap_or(url).to_string()
}

fn calculate_fuzzy_score(query: &str, text: &str) -> f64 {
    if query.is_empty() || text.is_empty() {
        return 0.0;
    }
    
    // Simple fuzzy matching - count matching characters in order
    let query_chars: Vec<char> = query.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();
    
    let mut matches = 0;
    let mut query_idx = 0;
    
    for c in &text_chars {
        if query_idx < query_chars.len() && *c == query_chars[query_idx] {
            matches += 1;
            query_idx += 1;
        }
    }
    
    matches as f64 / query_chars.len() as f64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchStats {
    pub indexed_bookmarks: usize,
    pub total_searches: usize,
    pub unique_domains: usize,
    pub unique_tags: usize,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            fields: vec![SearchField::All],
            case_sensitive: false,
            fuzzy_search: true,
            max_results: 50,
            include_deleted: false,
            min_relevance: 0.3,
            boost_favorites: true,
            boost_recent: true,
        }
    }
}

impl Default for BookmarkSearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_engine_creation() {
        let engine = BookmarkSearchEngine::new();
        let stats = engine.get_search_stats().await;
        assert_eq!(stats.indexed_bookmarks, 0);
    }

    #[tokio::test]
    async fn test_index_and_search() {
        let engine = BookmarkSearchEngine::new();
        
        let bookmark = Bookmark {
            id: Uuid::new_v4(),
            url: "https://rust-lang.org".to_string(),
            title: "Rust Programming Language".to_string(),
            description: Some("Official Rust website".to_string()),
            favicon: None,
            folder_id: None,
            tags: vec!["programming".to_string(), "rust".to_string()],
            is_favorite: true,
            is_read_later: false,
            visit_count: 5,
            last_accessed: Some(Utc::now()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        engine.index_bookmark(&bookmark).await;
        
        let results = engine.search("rust", SearchOptions::default()).await;
        assert!(results.len() > 0);
    }

    #[tokio::test]
    async fn test_search_by_tag() {
        let engine = BookmarkSearchEngine::new();
        
        let bookmark = Bookmark {
            id: Uuid::new_v4(),
            url: "https://example.com".to_string(),
            title: "Example Site".to_string(),
            description: None,
            favicon: None,
            folder_id: None,
            tags: vec!["tech".to_string()],
            is_favorite: false,
            is_read_later: false,
            visit_count: 1,
            last_accessed: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        engine.index_bookmark(&bookmark).await;
        
        let results = engine.search_by_tag("tech").await;
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_suggestions() {
        let engine = BookmarkSearchEngine::new();
        
        let bookmark = Bookmark {
            id: Uuid::new_v4(),
            url: "https://github.com".to_string(),
            title: "GitHub - Where the world builds software".to_string(),
            description: None,
            favicon: None,
            folder_id: None,
            tags: Vec::new(),
            is_favorite: false,
            is_read_later: false,
            visit_count: 0,
            last_accessed: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        engine.index_bookmark(&bookmark).await;
        
        let suggestions = engine.get_suggestions("Git", 5).await;
        assert_eq!(suggestions.len(), 1);
        assert!(suggestions[0].contains("GitHub"));
    }

    #[tokio::test]
    async fn test_fuzzy_search() {
        let engine = BookmarkSearchEngine::new();
        
        let bookmark = Bookmark {
            id: Uuid::new_v4(),
            url: "https://example.com".to_string(),
            title: "Programming Tutorial".to_string(),
            description: None,
            favicon: None,
            folder_id: None,
            tags: Vec::new(),
            is_favorite: false,
            is_read_later: false,
            visit_count: 0,
            last_accessed: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        engine.index_bookmark(&bookmark).await;
        
        let options = SearchOptions {
            fuzzy_search: true,
            ..Default::default()
        };
        
        let results = engine.search("prog", options).await;
        assert!(results.len() > 0);
    }
}