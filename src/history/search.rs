/// # History Search Module
/// 
/// Provides search and query functionality for browser history.
/// Supports full-text search, filtering, and ranking.

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{HistoryEntry, HistoryQuery, SortOrder, storage::HistoryStorage, Result};

/// Search errors
#[derive(Error, Debug)]
pub enum SearchError {
    #[error("Invalid search query: {0}")]
    InvalidQuery(String),
    #[error("Search failed: {0}")]
    SearchFailed(String),
}

/// Search result with ranking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// History entry
    pub entry: HistoryEntry,
    /// Relevance score (0-1)
    pub score: f64,
    /// Matched fields
    pub matched_fields: Vec<MatchedField>,
}

/// Matched field in search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchedField {
    /// Field name
    pub field: String,
    /// Matched text snippet
    pub snippet: String,
    /// Highlight positions (start, end)
    pub highlights: Vec<(usize, usize)>,
}

/// Search index for fast queries
pub struct SearchIndex {
    /// Term to entry IDs mapping
    term_index: HashMap<String, Vec<String>>,
    /// URL index
    url_index: HashMap<String, Vec<String>>,
    /// Domain index
    domain_index: HashMap<String, Vec<String>>,
}

impl SearchIndex {
    /// Create a new search index
    pub fn new() -> Self {
        Self {
            term_index: HashMap::new(),
            url_index: HashMap::new(),
            domain_index: HashMap::new(),
        }
    }

    /// Index an entry
    pub fn index_entry(&mut self, entry: &HistoryEntry) {
        let id = &entry.id;

        // Index URL terms
        for term in tokenize(&entry.url) {
            self.term_index
                .entry(term.to_lowercase())
                .or_insert_with(Vec::new)
                .push(id.clone());
        }

        // Index title terms
        for term in tokenize(&entry.title) {
            self.term_index
                .entry(term.to_lowercase())
                .or_insert_with(Vec::new)
                .push(id.clone());
        }

        // Index URL
        self.url_index
            .entry(entry.url.clone())
            .or_insert_with(Vec::new)
            .push(id.clone());

        // Index domain
        let domain = extract_domain(&entry.url);
        self.domain_index
            .entry(domain)
            .or_insert_with(Vec::new)
            .push(id.clone());
    }

    /// Search for entries by term
    pub fn search_term(&self, term: &str) -> Vec<String> {
        let term_lower = term.to_lowercase();
        self.term_index
            .get(&term_lower)
            .cloned()
            .unwrap_or_default()
    }

    /// Clear the index
    pub fn clear(&mut self) {
        self.term_index.clear();
        self.url_index.clear();
        self.domain_index.clear();
    }
}

impl Default for SearchIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// History search engine
pub struct HistorySearch {
    /// Search index
    index: SearchIndex,
    /// Enable fuzzy matching
    fuzzy_enabled: bool,
    /// Maximum results
    max_results: usize,
}

impl HistorySearch {
    /// Create a new search engine
    pub fn new() -> Self {
        Self {
            index: SearchIndex::new(),
            fuzzy_enabled: true,
            max_results: 100,
        }
    }

    /// Rebuild search index
    pub async fn rebuild_index(&mut self, entries: &[HistoryEntry]) {
        self.index.clear();
        for entry in entries {
            self.index.index_entry(entry);
        }
    }

    /// Query history entries
    pub async fn query(&self, storage: &HistoryStorage, query: HistoryQuery) -> Result<Vec<HistoryEntry>> {
        let mut entries = if let Some(ref search) = query.search {
            // Use search index
            let ids = self.search(search);
            let mut results = Vec::new();
            for id in ids {
                if let Some(entry) = storage.get_entry(&id).await {
                    results.push(entry);
                }
            }
            results
        } else {
            // Get all entries
            storage.get_all_entries().await?
        };

        // Apply filters
        entries = self.apply_filters(entries, &query);

        // Sort results
        self.sort_entries(&mut entries, query.sort);

        // Apply limit
        if let Some(limit) = query.limit {
            entries.truncate(limit);
        }

        Ok(entries)
    }

    /// Apply filters to entries
    fn apply_filters(&self, entries: Vec<HistoryEntry>, query: &HistoryQuery) -> Vec<HistoryEntry> {
        entries
            .into_iter()
            .filter(|e| {
                // Filter by date range
                if let Some(start) = query.start_date {
                    if e.timestamp < start {
                        return false;
                    }
                }
                if let Some(end) = query.end_date {
                    if e.timestamp > end {
                        return false;
                    }
                }

                // Filter by category
                if let Some(ref category) = query.category {
                    if e.category.as_ref() != Some(category) {
                        return false;
                    }
                }

                // Filter by tags
                if let Some(ref tags) = query.tags {
                    if !tags.iter().any(|t| e.tags.contains(t)) {
                        return false;
                    }
                }

                // Filter private entries
                if !query.include_private && e.is_private {
                    return false;
                }

                true
            })
            .collect()
    }

    /// Sort entries by order
    fn sort_entries(&self, entries: &mut [HistoryEntry], sort: SortOrder) {
        match sort {
            SortOrder::NewestFirst => entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp)),
            SortOrder::OldestFirst => entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp)),
            SortOrder::MostVisited => entries.sort_by(|a, b| b.visit_count.cmp(&a.visit_count)),
            SortOrder::LeastVisited => entries.sort_by(|a, b| a.visit_count.cmp(&b.visit_count)),
            SortOrder::TitleAsc => entries.sort_by(|a, b| a.title.cmp(&b.title)),
            SortOrder::TitleDesc => entries.sort_by(|a, b| b.title.cmp(&a.title)),
        }
    }

    /// Search for entries
    pub fn search(&self, query: &str) -> Vec<String> {
        let terms = tokenize(query);
        let mut results: HashMap<String, f64> = HashMap::new();

        for term in terms {
            let term_results = self.index.search_term(&term);
            
            for id in term_results {
                *results.entry(id).or_insert(0.0) += 1.0;
            }
        }

        // Sort by score
        let mut ranked: Vec<_> = results.into_iter().collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        ranked.iter()
            .take(self.max_results)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Search with detailed results
    pub async fn search_detailed(&self, storage: &HistoryStorage, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let ids = self.search(query);
        let mut results = Vec::new();

        for id in ids.iter().take(limit) {
            if let Some(entry) = storage.get_entry(id).await {
                let score = self.calculate_score(&entry, query);
                let matched_fields = self.find_matches(&entry, query);
                
                results.push(SearchResult {
                    entry,
                    score,
                    matched_fields,
                });
            }
        }

        // Sort by score
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        Ok(results)
    }

    /// Calculate relevance score
    fn calculate_score(&self, entry: &HistoryEntry, query: &str) -> f64 {
        let terms = tokenize(query);
        let mut score = 0.0;

        for term in &terms {
            let term_lower = term.to_lowercase();
            
            // Title match (higher weight)
            if entry.title.to_lowercase().contains(&term_lower) {
                score += 2.0;
            }
            
            // URL match
            if entry.url.to_lowercase().contains(&term_lower) {
                score += 1.0;
            }
        }

        // Boost by visit count
        score += (entry.visit_count as f64).log2() * 0.1;

        // Normalize to 0-1
        score / (terms.len() as f64 * 3.0)
    }

    /// Find matched fields
    fn find_matches(&self, entry: &HistoryEntry, query: &str) -> Vec<MatchedField> {
        let terms = tokenize(query);
        let mut matches = Vec::new();

        // Check title
        let title_lower = entry.title.to_lowercase();
        for term in &terms {
            let term_lower = term.to_lowercase();
            if title_lower.contains(&term_lower) {
                if let Some(pos) = title_lower.find(&term_lower) {
                    let snippet = create_snippet(&entry.title, pos, term.len());
                    matches.push(MatchedField {
                        field: "title".to_string(),
                        snippet,
                        highlights: vec![(pos, pos + term.len())],
                    });
                }
            }
        }

        // Check URL
        let url_lower = entry.url.to_lowercase();
        for term in &terms {
            let term_lower = term.to_lowercase();
            if url_lower.contains(&term_lower) {
                if let Some(pos) = url_lower.find(&term_lower) {
                    let snippet = create_snippet(&entry.url, pos, term.len());
                    matches.push(MatchedField {
                        field: "url".to_string(),
                        snippet,
                        highlights: vec![(pos, pos + term.len())],
                    });
                }
            }
        }

        matches
    }

    /// Suggest completions for partial query
    pub fn suggest(&self, partial: &str, limit: usize) -> Vec<String> {
        let partial_lower = partial.to_lowercase();
        
        // Collect matching terms from index
        let mut suggestions: Vec<String> = self.index.term_index
            .keys()
            .filter(|term| term.starts_with(&partial_lower))
            .take(limit)
            .cloned()
            .collect();

        // Also suggest from URLs and domains
        for url in self.index.url_index.keys() {
            if url.to_lowercase().starts_with(&partial_lower) {
                suggestions.push(url.clone());
            }
        }

        for domain in self.index.domain_index.keys() {
            if domain.to_lowercase().starts_with(&partial_lower) {
                suggestions.push(domain.clone());
            }
        }

        suggestions.truncate(limit);
        suggestions
    }
}

impl Default for HistorySearch {
    fn default() -> Self {
        Self::new()
    }
}

/// Tokenize text into search terms
fn tokenize(text: &str) -> Vec<&str> {
    text.split_whitespace()
        .filter(|s| s.len() >= 2)
        .collect()
}

/// Extract domain from URL
fn extract_domain(url: &str) -> String {
    let url = url.trim_start_matches("https://")
                .trim_start_matches("http://")
                .trim_start_matches("www.");
    url.split('/').next().unwrap_or(url).to_string()
}

/// Create snippet around match
fn create_snippet(text: &str, pos: usize, match_len: usize) -> String {
    let start = pos.saturating_sub(20);
    let end = (pos + match_len + 20).min(text.len());
    
    let mut snippet = String::new();
    if start > 0 {
        snippet.push_str("...");
    }
    snippet.push_str(&text[start..end]);
    if end < text.len() {
        snippet.push_str("...");
    }
    snippet
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let terms = tokenize("hello world test");
        assert_eq!(terms, vec!["hello", "world", "test"]);
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(extract_domain("https://example.com/page"), "example.com");
    }

    #[test]
    fn test_search_index() {
        let mut index = SearchIndex::new();
        
        let entry = HistoryEntry::new(
            "https://example.com".to_string(),
            "Example Site".to_string(),
            false,
        );
        
        index.index_entry(&entry);
        
        let results = index.search_term("example");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_create_snippet() {
        let text = "This is a long text with a match here and more text";
        let snippet = create_snippet(text, 25, 5);
        assert!(snippet.contains("match"));
    }
}