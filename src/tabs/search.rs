use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchableTab {
    pub id: Uuid,
    pub url: String,
    pub title: String,
    pub favicon: Option<String>,
    pub group_name: Option<String>,
    pub workspace_name: Option<String>,
    pub last_accessed: Option<DateTime<Utc>>,
    pub visit_count: usize,
    pub is_pinned: bool,
    pub is_hibernated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub tab: SearchableTab,
    pub relevance_score: f64,
    pub match_type: MatchType,
    pub matched_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchType {
    TitleExact,
    TitlePartial,
    UrlExact,
    UrlPartial,
    DomainMatch,
    FuzzyMatch,
}

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub search_titles: bool,
    pub search_urls: bool,
    pub search_domains: bool,
    pub case_sensitive: bool,
    pub fuzzy_search: bool,
    pub max_results: usize,
    pub include_closed_tabs: bool,
}

pub struct TabSearchIndex {
    tabs: Arc<RwLock<HashMap<Uuid, SearchableTab>>>,
    title_index: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
    url_index: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
    domain_index: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
    recent_searches: Arc<RwLock<Vec<SearchQuery>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub timestamp: DateTime<Utc>,
    pub results_count: usize,
}

impl TabSearchIndex {
    pub fn new() -> Self {
        Self {
            tabs: Arc::new(RwLock::new(HashMap::new())),
            title_index: Arc::new(RwLock::new(HashMap::new())),
            url_index: Arc::new(RwLock::new(HashMap::new())),
            domain_index: Arc::new(RwLock::new(HashMap::new())),
            recent_searches: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn index_tab(&self, tab: SearchableTab) {
        let tab_id = tab.id;
        let title = tab.title.to_lowercase();
        let url = tab.url.to_lowercase();
        let domain = extract_domain(&tab.url).to_lowercase();

        // Add to main storage
        self.tabs.write().await.insert(tab_id, tab);

        // Update title index
        let mut title_idx = self.title_index.write().await;
        for word in title.split_whitespace() {
            title_idx.entry(word.to_string())
                .or_insert_with(Vec::new)
                .push(tab_id);
        }

        // Update URL index
        let mut url_idx = self.url_index.write().await;
        for segment in url.split(&['/', '?', '=', '&', '-', '_'][..]) {
            if !segment.is_empty() {
                url_idx.entry(segment.to_string())
                    .or_insert_with(Vec::new)
                    .push(tab_id);
            }
        }

        // Update domain index
        let mut domain_idx = self.domain_index.write().await;
        domain_idx.entry(domain)
            .or_insert_with(Vec::new)
            .push(tab_id);
    }

    pub async fn remove_tab(&self, tab_id: Uuid) {
        self.tabs.write().await.remove(&tab_id);
        // Note: In production, would also clean up indexes
    }

    pub async fn update_tab(&self, tab: SearchableTab) {
        self.index_tab(tab).await;
    }

    pub async fn search(&self, query: &str, options: SearchOptions) -> Vec<SearchResult> {
        let start_time = std::time::Instant::now();
        let mut results = Vec::new();
        let tabs = self.tabs.read().await;
        let query_lower = if options.case_sensitive {
            query.to_string()
        } else {
            query.to_lowercase()
        };

        for tab in tabs.values() {
            let mut relevance_score = 0.0;
            let mut match_type = MatchType::FuzzyMatch;
            let mut matched_text = String::new();

            let title_lower = if options.case_sensitive {
                tab.title.clone()
            } else {
                tab.title.to_lowercase()
            };

            let url_lower = if options.case_sensitive {
                tab.url.clone()
            } else {
                tab.url.to_lowercase()
            };

            // Check title matches
            if options.search_titles {
                if title_lower == query_lower {
                    relevance_score = 1.0;
                    match_type = MatchType::TitleExact;
                    matched_text = tab.title.clone();
                } else if title_lower.contains(&query_lower) {
                    relevance_score = 0.9;
                    match_type = MatchType::TitlePartial;
                    matched_text = tab.title.clone();
                }
            }

            // Check URL matches
            if options.search_urls && relevance_score < 0.9 {
                if url_lower == query_lower {
                    relevance_score = 0.95;
                    match_type = MatchType::UrlExact;
                    matched_text = tab.url.clone();
                } else if url_lower.contains(&query_lower) {
                    relevance_score = 0.8;
                    match_type = MatchType::UrlPartial;
                    matched_text = tab.url.clone();
                }
            }

            // Check domain matches
            if options.search_domains && relevance_score < 0.8 {
                let domain = extract_domain(&tab.url).to_lowercase();
                if domain.contains(&query_lower) {
                    relevance_score = 0.7;
                    match_type = MatchType::DomainMatch;
                    matched_text = domain;
                }
            }

            // Fuzzy match
            if options.fuzzy_search && relevance_score == 0.0 {
                let fuzzy_score = calculate_fuzzy_score(&query_lower, &title_lower);
                if fuzzy_score > 0.5 {
                    relevance_score = fuzzy_score * 0.6;
                    match_type = MatchType::FuzzyMatch;
                    matched_text = tab.title.clone();
                }
            }

            if relevance_score > 0.0 {
                // Boost by visit count and recency
                let visit_boost = (tab.visit_count as f64).min(10.0) / 100.0;
                let recency_boost = calculate_recency_boost(tab.last_accessed);
                relevance_score += visit_boost + recency_boost;

                // Boost pinned tabs slightly
                if tab.is_pinned {
                    relevance_score += 0.05;
                }

                results.push(SearchResult {
                    tab: tab.clone(),
                    relevance_score,
                    match_type,
                    matched_text,
                });
            }
        }

        // Sort by relevance
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

        // Record search
        let search_record = SearchQuery {
            query: query.to_string(),
            timestamp: Utc::now(),
            results_count: results.len(),
        };
        self.recent_searches.write().await.push(search_record);

        // Limit results
        results.into_iter().take(options.max_results).collect()
    }

    pub async fn search_by_domain(&self, domain: &str) -> Vec<SearchResult> {
        let domain_lower = domain.to_lowercase();
        let mut results = Vec::new();
        let tabs = self.tabs.read().await;

        for tab in tabs.values() {
            let tab_domain = extract_domain(&tab.url).to_lowercase();
            if tab_domain.contains(&domain_lower) {
                results.push(SearchResult {
                    tab: tab.clone(),
                    relevance_score: 1.0,
                    match_type: MatchType::DomainMatch,
                    matched_text: tab_domain,
                });
            }
        }

        results
    }

    pub async fn search_by_group(&self, group_name: &str) -> Vec<SearchResult> {
        let group_lower = group_name.to_lowercase();
        let mut results = Vec::new();
        let tabs = self.tabs.read().await;

        for tab in tabs.values() {
            if let Some(ref gn) = tab.group_name {
                if gn.to_lowercase().contains(&group_lower) {
                    results.push(SearchResult {
                        tab: tab.clone(),
                        relevance_score: 1.0,
                        match_type: MatchType::TitlePartial,
                        matched_text: gn.clone(),
                    });
                }
            }
        }

        results
    }

    pub async fn get_recent_tabs(&self, limit: usize) -> Vec<SearchableTab> {
        let mut tabs: Vec<_> = self.tabs.read().await.values().cloned().collect();
        tabs.sort_by(|a, b| {
            b.last_accessed.unwrap_or(DateTime::UNIX_EPOCH)
                .cmp(&a.last_accessed.unwrap_or(DateTime::UNIX_EPOCH))
        });
        tabs.into_iter().take(limit).collect()
    }

    pub async fn get_most_visited(&self, limit: usize) -> Vec<SearchableTab> {
        let mut tabs: Vec<_> = self.tabs.read().await.values().cloned().collect();
        tabs.sort_by(|a, b| b.visit_count.cmp(&a.visit_count));
        tabs.into_iter().take(limit).collect()
    }

    pub async fn get_recent_searches(&self, limit: usize) -> Vec<SearchQuery> {
        let searches = self.recent_searches.read().await;
        searches.iter().rev().take(limit).cloned().collect()
    }

    pub async fn clear_recent_searches(&self) {
        self.recent_searches.write().await.clear();
    }

    pub async fn get_suggestions(&self, partial: &str, limit: usize) -> Vec<String> {
        let mut suggestions = Vec::new();
        let partial_lower = partial.to_lowercase();
        let tabs = self.tabs.read().await;

        // Collect matching titles and URLs
        let mut seen = std::collections::HashSet::new();
        for tab in tabs.values() {
            if tab.title.to_lowercase().starts_with(&partial_lower) && seen.insert(tab.title.clone()) {
                suggestions.push(tab.title.clone());
            }
            if suggestions.len() >= limit {
                break;
            }
        }

        suggestions
    }

    pub async fn get_search_stats(&self) -> SearchStats {
        let tabs = self.tabs.read().await;
        let searches = self.recent_searches.read().await;
        
        SearchStats {
            indexed_tabs: tabs.len(),
            total_searches: searches.len(),
            unique_domains: self.domain_index.read().await.len(),
        }
    }
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
    let query_chars: Vec<char> = query.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();
    
    if query_chars.is_empty() || text_chars.is_empty() {
        return 0.0;
    }

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

fn calculate_recency_boost(last_accessed: Option<DateTime<Utc>>) -> f64 {
    match last_accessed {
        Some(dt) => {
            let now = Utc::now();
            let hours_ago = (now - dt).num_hours().max(0) as f64;
            (1.0 / (1.0 + hours_ago / 24.0)) * 0.1
        }
        None => 0.0,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchStats {
    pub indexed_tabs: usize,
    pub total_searches: usize,
    pub unique_domains: usize,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            search_titles: true,
            search_urls: true,
            search_domains: true,
            case_sensitive: false,
            fuzzy_search: true,
            max_results: 50,
            include_closed_tabs: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_index_creation() {
        let index = TabSearchIndex::new();
        let stats = index.get_search_stats().await;
        assert_eq!(stats.indexed_tabs, 0);
    }

    #[tokio::test]
    async fn test_index_and_search() {
        let index = TabSearchIndex::new();
        let tab = SearchableTab {
            id: Uuid::new_v4(),
            url: "https://example.com/page".to_string(),
            title: "Example Page".to_string(),
            favicon: None,
            group_name: None,
            workspace_name: None,
            last_accessed: Some(Utc::now()),
            visit_count: 5,
            is_pinned: false,
            is_hibernated: false,
        };

        index.index_tab(tab).await;
        
        let results = index.search("Example", SearchOptions::default()).await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].match_type, MatchType::TitlePartial);
    }

    #[tokio::test]
    async fn test_search_by_domain() {
        let index = TabSearchIndex::new();
        let tab = SearchableTab {
            id: Uuid::new_v4(),
            url: "https://github.com/rust-lang/rust".to_string(),
            title: "Rust Programming Language".to_string(),
            favicon: None,
            group_name: None,
            workspace_name: None,
            last_accessed: Some(Utc::now()),
            visit_count: 10,
            is_pinned: false,
            is_hibernated: false,
        };

        index.index_tab(tab).await;
        
        let results = index.search_by_domain("github.com").await;
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_recent_tabs() {
        let index = TabSearchIndex::new();
        
        let tab1 = SearchableTab {
            id: Uuid::new_v4(),
            url: "https://example1.com".to_string(),
            title: "Example 1".to_string(),
            favicon: None,
            group_name: None,
            workspace_name: None,
            last_accessed: Some(Utc::now() - chrono::Duration::hours(1)),
            visit_count: 1,
            is_pinned: false,
            is_hibernated: false,
        };

        let tab2 = SearchableTab {
            id: Uuid::new_v4(),
            url: "https://example2.com".to_string(),
            title: "Example 2".to_string(),
            favicon: None,
            group_name: None,
            workspace_name: None,
            last_accessed: Some(Utc::now()),
            visit_count: 1,
            is_pinned: false,
            is_hibernated: false,
        };

        index.index_tab(tab1).await;
        index.index_tab(tab2).await;

        let recent = index.get_recent_tabs(10).await;
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].title, "Example 2"); // Most recent first
    }

    #[tokio::test]
    async fn test_get_suggestions() {
        let index = TabSearchIndex::new();
        
        let tab = SearchableTab {
            id: Uuid::new_v4(),
            url: "https://rust-lang.org".to_string(),
            title: "Rust Programming Language".to_string(),
            favicon: None,
            group_name: None,
            workspace_name: None,
            last_accessed: Some(Utc::now()),
            visit_count: 5,
            is_pinned: false,
            is_hibernated: false,
        };

        index.index_tab(tab).await;
        
        let suggestions = index.get_suggestions("Rust", 5).await;
        assert_eq!(suggestions.len(), 1);
        assert!(suggestions[0].contains("Rust"));
    }

    #[tokio::test]
    async fn test_fuzzy_search() {
        let index = TabSearchIndex::new();
        
        let tab = SearchableTab {
            id: Uuid::new_v4(),
            url: "https://example.com".to_string(),
            title: "Programming Tutorial".to_string(),
            favicon: None,
            group_name: None,
            workspace_name: None,
            last_accessed: Some(Utc::now()),
            visit_count: 1,
            is_pinned: false,
            is_hibernated: false,
        };

        index.index_tab(tab).await;
        
        let options = SearchOptions {
            fuzzy_search: true,
            search_titles: true,
            search_urls: false,
            search_domains: false,
            case_sensitive: false,
            max_results: 10,
            include_closed_tabs: false,
        };
        
        let results = index.search("prgrmmng", options).await;
        assert!(results.len() > 0); // Fuzzy match should find it
    }
}