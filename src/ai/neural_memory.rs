//! Neural Memory Module
//!
//! AI-powered memory for contextual browsing:
//! - Associative search across browsing history
//! - Contextual understanding of content
//! - Smart suggestions based on patterns
//! - Pattern recognition for user behavior

use anyhow::{anyhow, Result};
use log::{debug, info};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use parking_lot::RwLock;
use std::time::{Duration, Instant};

/// Memory entry representing stored information
#[derive(Debug, Clone)]
pub struct MemoryEntry {
    /// Unique identifier
    pub id: u64,
    /// URL associated with this memory
    pub url: String,
    /// Title of the page
    pub title: String,
    /// Content snippet
    pub content: String,
    /// Keywords extracted from content
    pub keywords: Vec<String>,
    /// Embedding vector (simplified)
    pub embedding: Vec<f32>,
    /// Timestamp when stored
    pub stored_at: Instant,
    /// Last accessed
    pub last_accessed: Instant,
    /// Access count
    pub access_count: u32,
    /// Importance score
    pub importance: f32,
    /// Associated tags
    pub tags: HashSet<String>,
}

/// Search query for memory
#[derive(Debug, Clone)]
pub struct MemoryQuery {
    /// Text query
    pub text: String,
    /// URL filter (optional)
    pub url_filter: Option<String>,
    /// Tag filter (optional)
    pub tag_filter: Option<String>,
    /// Time range (optional)
    pub time_range: Option<Duration>,
    /// Maximum results
    pub max_results: usize,
    /// Minimum relevance score
    pub min_relevance: f32,
}

impl Default for MemoryQuery {
    fn default() -> Self {
        Self {
            text: String::new(),
            url_filter: None,
            tag_filter: None,
            time_range: None,
            max_results: 10,
            min_relevance: 0.5,
        }
    }
}

/// Search result from memory
#[derive(Debug, Clone)]
pub struct MemorySearchResult {
    /// The memory entry
    pub entry: MemoryEntry,
    /// Relevance score (0.0 - 1.0)
    pub relevance: f32,
    /// Match highlights
    pub highlights: Vec<String>,
}

/// Context for current browsing session
#[derive(Debug, Clone)]
pub struct BrowsingContext {
    /// Current URL
    pub current_url: String,
    /// Current page title
    pub current_title: String,
    /// Current page keywords
    pub current_keywords: Vec<String>,
    /// Recent navigation history
    pub recent_urls: Vec<String>,
    /// Current session duration
    pub session_duration: Duration,
    /// Active tags
    pub active_tags: HashSet<String>,
}

/// Pattern detected in user behavior
#[derive(Debug, Clone)]
pub struct BehaviorPattern {
    /// Pattern identifier
    pub id: String,
    /// Pattern name
    pub name: String,
    /// Pattern description
    pub description: String,
    /// URLs associated with this pattern
    pub urls: HashSet<String>,
    /// Keywords associated with this pattern
    pub keywords: HashSet<String>,
    /// Confidence score
    pub confidence: f32,
    /// Pattern frequency
    pub frequency: u32,
    /// Last occurrence
    pub last_occurrence: Instant,
}

/// Smart suggestion from neural memory
#[derive(Debug, Clone)]
pub struct SmartSuggestion {
    /// Suggested URL
    pub url: String,
    /// Suggestion title
    pub title: String,
    /// Relevance score
    pub relevance: f32,
    /// Reason for suggestion
    pub reason: SuggestionReason,
    /// Related memories
    pub related_memories: Vec<u64>,
}

/// Reason for suggestion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuggestionReason {
    /// Frequently visited
    FrequentVisit,
    /// Similar to current page
    SimilarContent,
    /// Part of detected pattern
    PatternMatch,
    /// Time-based suggestion
    TimeBased,
    /// Contextually relevant
    Contextual,
    /// Recently accessed
    RecentAccess,
}

/// Neural Memory configuration
#[derive(Debug, Clone)]
pub struct NeuralMemoryConfig {
    /// Maximum memories to store
    pub max_memories: usize,
    /// Embedding dimension
    pub embedding_dim: usize,
    /// Memory expiration time
    pub expiration: Duration,
    /// Enable auto-tagging
    pub auto_tagging: bool,
    /// Enable pattern detection
    pub pattern_detection: bool,
    /// Minimum importance to keep
    pub min_importance: f32,
}

impl Default for NeuralMemoryConfig {
    fn default() -> Self {
        Self {
            max_memories: 10000,
            embedding_dim: 128,
            expiration: Duration::from_secs(30 * 24 * 60 * 60), // 30 days
            auto_tagging: true,
            pattern_detection: true,
            min_importance: 0.1,
        }
    }
}

/// Neural Memory Engine
pub struct NeuralMemory {
    /// Configuration
    config: NeuralMemoryConfig,
    /// Memory entries
    memories: Arc<RwLock<HashMap<u64, MemoryEntry>>>,
    /// URL index
    url_index: Arc<RwLock<HashMap<String, HashSet<u64>>>>,
    /// Keyword index
    keyword_index: Arc<RwLock<HashMap<String, HashSet<u64>>>>,
    /// Tag index
    tag_index: Arc<RwLock<HashMap<String, HashSet<u64>>>>,
    /// Behavior patterns
    patterns: Arc<RwLock<Vec<BehaviorPattern>>>,
    /// Current context
    context: Arc<RwLock<Option<BrowsingContext>>>,
    /// Next ID
    next_id: Arc<RwLock<u64>>,
}

impl NeuralMemory {
    /// Create a new neural memory
    pub fn new() -> Result<Self> {
        Self::with_config(NeuralMemoryConfig::default())
    }
    
    /// Create with custom configuration
    pub fn with_config(config: NeuralMemoryConfig) -> Result<Self> {
        info!("Initializing Neural Memory...");
        
        let memory = Self {
            config,
            memories: Arc::new(RwLock::new(HashMap::new())),
            url_index: Arc::new(RwLock::new(HashMap::new())),
            keyword_index: Arc::new(RwLock::new(HashMap::new())),
            tag_index: Arc::new(RwLock::new(HashMap::new())),
            patterns: Arc::new(RwLock::new(Vec::new())),
            context: Arc::new(RwLock::new(None)),
            next_id: Arc::new(RwLock::new(1)),
        };
        
        info!("Neural Memory initialized with capacity for {} entries", config.max_memories);
        Ok(memory)
    }
    
    /// Store a new memory
    pub fn store(&self, url: String, title: String, content: String) -> Result<u64> {
        debug!("Storing memory for: {}", url);
        
        let id = {
            let mut next_id = self.next_id.write();
            let id = *next_id;
            *next_id += 1;
            id
        };
        
        // Extract keywords
        let keywords = self.extract_keywords(&content);
        
        // Generate embedding
        let embedding = self.generate_embedding(&content);
        
        // Auto-tag if enabled
        let tags = if self.config.auto_tagging {
            self.auto_tag(&url, &title, &keywords)
        } else {
            HashSet::new()
        };
        
        // Calculate initial importance
        let importance = self.calculate_importance(&keywords, &tags);
        
        let entry = MemoryEntry {
            id,
            url: url.clone(),
            title,
            content,
            keywords: keywords.clone(),
            embedding,
            stored_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 0,
            importance,
            tags: tags.clone(),
        };
        
        // Store entry
        {
            let mut memories = self.memories.write();
            
            // Check capacity
            if memories.len() >= self.config.max_memories {
                self.evict_memories();
            }
            
            memories.insert(id, entry);
        }
        
        // Update indices
        {
            let mut url_index = self.url_index.write();
            url_index.entry(url.clone()).or_default().insert(id);
        }
        
        {
            let mut keyword_index = self.keyword_index.write();
            for keyword in &keywords {
                keyword_index.entry(keyword.clone()).or_default().insert(id);
            }
        }
        
        {
            let mut tag_index = self.tag_index.write();
            for tag in &tags {
                tag_index.entry(tag.clone()).or_default().insert(id);
            }
        }
        
        debug!("Memory {} stored with {} keywords and {} tags", id, keywords.len(), tags.len());
        Ok(id)
    }
    
    /// Search memories
    pub fn search(&self, query: MemoryQuery) -> Result<Vec<MemorySearchResult>> {
        debug!("Searching for: {}", query.text);
        
        let memories = self.memories.read();
        let mut results = Vec::new();
        
        // Generate query embedding
        let query_embedding = self.generate_embedding(&query.text);
        let query_keywords = self.extract_keywords(&query.text);
        
        // Find candidates
        let mut candidates: HashSet<u64> = HashSet::new();
        
        // Search by keywords
        {
            let keyword_index = self.keyword_index.read();
            for keyword in &query_keywords {
                if let Some(ids) = keyword_index.get(keyword) {
                    candidates.extend(ids);
                }
            }
        }
        
        // Filter by URL if specified
        if let Some(url_filter) = &query.url_filter {
            let url_index = self.url_index.read();
            if let Some(url_ids) = url_index.get(url_filter) {
                candidates.retain(|id| url_ids.contains(id));
            }
        }
        
        // Filter by tag if specified
        if let Some(tag_filter) = &query.tag_filter {
            let tag_index = self.tag_index.read();
            if let Some(tag_ids) = tag_index.get(tag_filter) {
                candidates.retain(|id| tag_ids.contains(id));
            }
        }
        
        // Score and rank results
        for id in candidates {
            if let Some(entry) = memories.get(&id) {
                let relevance = self.calculate_relevance(&query_embedding, &query_keywords, entry);
                
                if relevance >= query.min_relevance {
                    let highlights = self.extract_highlights(&query.text, entry);
                    
                    results.push(MemorySearchResult {
                        entry: entry.clone(),
                        relevance,
                        highlights,
                    });
                }
            }
        }
        
        // Sort by relevance
        results.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap());
        results.truncate(query.max_results);
        
        debug!("Found {} results", results.len());
        Ok(results)
    }
    
    /// Get smart suggestions based on current context
    pub fn get_suggestions(&self) -> Result<Vec<SmartSuggestion>> {
        let context = self.context.read().clone();
        let context = context.ok_or_else(|| anyhow!("No browsing context set"))?;
        
        let mut suggestions = Vec::new();
        let memories = self.memories.read();
        
        // Find similar content
        let current_embedding = self.generate_embedding(&context.current_title);
        let current_keywords = &context.current_keywords;
        
        for entry in memories.values() {
            let similarity = self.embedding_similarity(&current_embedding, &entry.embedding);
            
            if similarity > 0.7 && entry.url != context.current_url {
                suggestions.push(SmartSuggestion {
                    url: entry.url.clone(),
                    title: entry.title.clone(),
                    relevance: similarity,
                    reason: SuggestionReason::SimilarContent,
                    related_memories: vec![entry.id],
                });
            }
        }
        
        // Add pattern-based suggestions
        if self.config.pattern_detection {
            let patterns = self.patterns.read();
            for pattern in patterns.iter() {
                if pattern.urls.contains(&context.current_url) {
                    // Find related URLs from this pattern
                    for url in &pattern.urls {
                        if url != &context.current_url {
                            if let Some(entry) = memories.values().find(|e| &e.url == url) {
                                suggestions.push(SmartSuggestion {
                                    url: url.clone(),
                                    title: entry.title.clone(),
                                    relevance: pattern.confidence,
                                    reason: SuggestionReason::PatternMatch,
                                    related_memories: vec![entry.id],
                                });
                            }
                        }
                    }
                }
            }
        }
        
        // Sort and deduplicate
        suggestions.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap());
        suggestions.truncate(10);
        
        // Remove duplicates by URL
        let mut seen_urls = HashSet::new();
        suggestions.retain(|s| seen_urls.insert(s.url.clone()));
        
        Ok(suggestions)
    }
    
    /// Update browsing context
    pub fn update_context(&self, context: BrowsingContext) -> Result<()> {
        *self.context.write() = Some(context);
        Ok(())
    }
    
    /// Detect patterns in user behavior
    pub fn detect_patterns(&self) -> Result<Vec<BehaviorPattern>> {
        if !self.config.pattern_detection {
            return Ok(Vec::new());
        }
        
        debug!("Detecting behavior patterns...");
        
        let memories = self.memories.read();
        let mut new_patterns = Vec::new();
        
        // Group memories by URL
        let mut url_groups: HashMap<String, Vec<&MemoryEntry>> = HashMap::new();
        for entry in memories.values() {
            url_groups.entry(entry.url.clone()).or_default().push(entry);
        }
        
        // Find frequent URLs (potential patterns)
        for (url, entries) in &url_groups {
            if entries.len() >= 3 {
                // This URL is visited frequently
                let mut keywords: HashSet<String> = HashSet::new();
                for entry in entries {
                    keywords.extend(entry.keywords.iter().cloned());
                }
                
                let pattern = BehaviorPattern {
                    id: format!("pattern_{}", url.hashCode()),
                    name: format!("Frequent visit to {}", url),
                    description: format!("User visits {} frequently", url),
                    urls: std::iter::once(url.clone()).collect(),
                    keywords,
                    confidence: 0.8,
                    frequency: entries.len() as u32,
                    last_occurrence: entries.iter().map(|e| e.last_accessed).max().unwrap(),
                };
                
                new_patterns.push(pattern);
            }
        }
        
        // Find sequential patterns (URLs often visited together)
        // This is a simplified version - in production, use more sophisticated algorithms
        
        *self.patterns.write() = new_patterns.clone();
        
        debug!("Detected {} patterns", new_patterns.len());
        Ok(new_patterns)
    }
    
    /// Access a memory (updates access time and count)
    pub fn access(&self, id: u64) -> Result<MemoryEntry> {
        let mut memories = self.memories.write();
        
        let entry = memories.get_mut(&id)
            .ok_or_else(|| anyhow!("Memory {} not found", id))?;
        
        entry.last_accessed = Instant::now();
        entry.access_count += 1;
        
        // Update importance based on access
        entry.importance = (entry.importance + 0.1).min(1.0);
        
        Ok(entry.clone())
    }
    
    /// Forget a memory
    pub fn forget(&self, id: u64) -> Result<()> {
        debug!("Forgetting memory {}", id);
        
        let entry = {
            let mut memories = self.memories.write();
            memories.remove(&id)
        };
        
        if let Some(entry) = entry {
            // Remove from indices
            {
                let mut url_index = self.url_index.write();
                if let Some(ids) = url_index.get_mut(&entry.url) {
                    ids.remove(&id);
                }
            }
            
            {
                let mut keyword_index = self.keyword_index.write();
                for keyword in &entry.keywords {
                    if let Some(ids) = keyword_index.get_mut(keyword) {
                        ids.remove(&id);
                    }
                }
            }
            
            {
                let mut tag_index = self.tag_index.write();
                for tag in &entry.tags {
                    if let Some(ids) = tag_index.get_mut(tag) {
                        ids.remove(&id);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Extract keywords from content
    fn extract_keywords(&self, content: &str) -> Vec<String> {
        // Simple keyword extraction
        // In production, use NLP libraries
        
        let stop_words = ["the", "a", "an", "is", "are", "was", "were", "be", "been", 
                         "being", "have", "has", "had", "do", "does", "did", "will",
                         "would", "could", "should", "may", "might", "must", "shall",
                         "can", "need", "dare", "ought", "used", "to", "of", "in",
                         "for", "on", "with", "at", "by", "from", "as", "into", "through",
                         "during", "before", "after", "above", "below", "between",
                         "and", "but", "or", "nor", "so", "yet", "both", "either",
                         "neither", "not", "only", "own", "same", "than", "too", "very"];
        
        content
            .to_lowercase()
            .split_whitespace()
            .filter(|word| word.len() > 3 && !stop_words.contains(&word.as_str()))
            .map(|word| word.chars().filter(|c| c.is_alphanumeric()).collect())
            .filter(|word: &String| !word.is_empty())
            .take(20)
            .collect()
    }
    
    /// Generate embedding for content (simplified)
    fn generate_embedding(&self, content: &str) -> Vec<f32> {
        // Simple hash-based embedding
        // In production, use actual ML models
        
        let mut embedding = vec![0.0f32; self.config.embedding_dim];
        
        for (i, c) in content.chars().enumerate() {
            let idx = (i * c as usize) % self.config.embedding_dim;
            embedding[idx] = (embedding[idx] + c as f32 / 255.0) % 1.0;
        }
        
        // Normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in embedding.iter_mut() {
                *val /= norm;
            }
        }
        
        embedding
    }
    
    /// Calculate similarity between embeddings
    fn embedding_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }
        
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm_a > 0.0 && norm_b > 0.0 {
            dot / (norm_a * norm_b)
        } else {
            0.0
        }
    }
    
    /// Auto-tag a memory
    fn auto_tag(&self, url: &str, title: &str, keywords: &[String]) -> HashSet<String> {
        let mut tags = HashSet::new();
        
        // Tag based on URL
        if url.contains("github.com") {
            tags.insert("code".to_string());
            tags.insert("development".to_string());
        }
        if url.contains("youtube.com") {
            tags.insert("video".to_string());
            tags.insert("entertainment".to_string());
        }
        if url.contains("reddit.com") {
            tags.insert("social".to_string());
            tags.insert("discussion".to_string());
        }
        
        // Tag based on title
        let title_lower = title.to_lowercase();
        if title_lower.contains("tutorial") {
            tags.insert("learning".to_string());
        }
        if title_lower.contains("news") {
            tags.insert("news".to_string());
        }
        
        // Tag based on keywords
        if keywords.contains(&"api".to_string()) {
            tags.insert("technical".to_string());
        }
        
        tags
    }
    
    /// Calculate initial importance
    fn calculate_importance(&self, keywords: &[String], tags: &HashSet<String>) -> f32 {
        let mut importance = 0.5;
        
        // More keywords = more detailed content
        importance += keywords.len() as f32 * 0.01;
        
        // More tags = more categorizable
        importance += tags.len() as f32 * 0.05;
        
        importance.min(1.0)
    }
    
    /// Calculate relevance score
    fn calculate_relevance(&self, query_embedding: &[f32], query_keywords: &[String], entry: &MemoryEntry) -> f32 {
        // Semantic similarity
        let semantic = self.embedding_similarity(query_embedding, &entry.embedding);
        
        // Keyword overlap
        let keyword_overlap = if query_keywords.is_empty() {
            0.0
        } else {
            let matches = query_keywords.iter()
                .filter(|k| entry.keywords.contains(k))
                .count();
            matches as f32 / query_keywords.len() as f32
        };
        
        // Combine scores
        semantic * 0.7 + keyword_overlap * 0.3
    }
    
    /// Extract highlights from matching content
    fn extract_highlights(&self, query: &str, entry: &MemoryEntry) -> Vec<String> {
        let query_words: HashSet<&str> = query.split_whitespace().collect();
        
        entry.content
            .split('.')
            .filter(|sentence| {
                sentence.split_whitespace()
                    .any(|word| query_words.contains(word))
            })
            .take(3)
            .map(|s| s.trim().to_string())
            .collect()
    }
    
    /// Evict low-importance memories
    fn evict_memories(&self) {
        let mut memories = self.memories.write();
        
        // Find lowest importance entries
        let mut entries: Vec<_> = memories.iter()
            .map(|(id, entry)| (*id, entry.importance, entry.last_accessed))
            .collect();
        
        entries.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        
        // Remove 10% lowest importance
        let to_remove = (memories.len() as f32 * 0.1) as usize;
        for (id, _, _) in entries.into_iter().take(to_remove) {
            memories.remove(&id);
        }
        
        debug!("Evicted {} memories", to_remove);
    }
    
    /// Get statistics
    pub fn get_stats(&self) -> NeuralMemoryStats {
        let memories = self.memories.read();
        let patterns = self.patterns.read();
        
        NeuralMemoryStats {
            total_memories: memories.len(),
            total_patterns: patterns.len(),
            unique_urls: self.url_index.read().len(),
            unique_keywords: self.keyword_index.read().len(),
            unique_tags: self.tag_index.read().len(),
        }
    }
}

impl Default for NeuralMemory {
    fn default() -> Self {
        Self::new().expect("Failed to create default NeuralMemory")
    }
}

/// Neural Memory statistics
#[derive(Debug, Clone)]
pub struct NeuralMemoryStats {
    pub total_memories: usize,
    pub total_patterns: usize,
    pub unique_urls: usize,
    pub unique_keywords: usize,
    pub unique_tags: usize,
}

// Helper trait for hashing
trait HashCode {
    fn hashCode(&self) -> u64;
}

impl HashCode for str {
    fn hashCode(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_creation() {
        let memory = NeuralMemory::new().unwrap();
        let stats = memory.get_stats();
        assert_eq!(stats.total_memories, 0);
    }
    
    #[test]
    fn test_store() {
        let memory = NeuralMemory::new().unwrap();
        
        let id = memory.store(
            "https://example.com".to_string(),
            "Example".to_string(),
            "This is example content with some keywords".to_string(),
        ).unwrap();
        
        assert!(id > 0);
        
        let stats = memory.get_stats();
        assert_eq!(stats.total_memories, 1);
    }
    
    #[test]
    fn test_search() {
        let memory = NeuralMemory::new().unwrap();
        
        memory.store(
            "https://rust-lang.org".to_string(),
            "Rust Programming".to_string(),
            "Rust is a systems programming language".to_string(),
        ).unwrap();
        
        memory.store(
            "https://python.org".to_string(),
            "Python Programming".to_string(),
            "Python is a scripting language".to_string(),
        ).unwrap();
        
        let results = memory.search(MemoryQuery {
            text: "programming language".to_string(),
            max_results: 10,
            ..Default::default()
        }).unwrap();
        
        assert!(!results.is_empty());
    }
    
    #[test]
    fn test_context_suggestions() {
        let memory = NeuralMemory::new().unwrap();
        
        // Store some memories
        memory.store(
            "https://rust-lang.org".to_string(),
            "Rust Programming".to_string(),
            "Rust is a systems programming language".to_string(),
        ).unwrap();
        
        memory.store(
            "https://doc.rust-lang.org".to_string(),
            "Rust Documentation".to_string(),
            "Official Rust documentation and tutorials".to_string(),
        ).unwrap();
        
        // Set context
        memory.update_context(BrowsingContext {
            current_url: "https://rust-lang.org".to_string(),
            current_title: "Rust Programming".to_string(),
            current_keywords: vec!["rust".to_string(), "programming".to_string()],
            recent_urls: vec!["https://rust-lang.org".to_string()],
            session_duration: Duration::from_secs(60),
            active_tags: HashSet::new(),
        }).unwrap();
        
        let suggestions = memory.get_suggestions().unwrap();
        assert!(!suggestions.is_empty());
    }
    
    #[test]
    fn test_pattern_detection() {
        let memory = NeuralMemory::new().unwrap();
        
        // Store multiple visits to same URL
        for _ in 0..5 {
            memory.store(
                "https://github.com".to_string(),
                "GitHub".to_string(),
                "Where the world builds software".to_string(),
            ).unwrap();
        }
        
        let patterns = memory.detect_patterns().unwrap();
        assert!(!patterns.is_empty());
    }
    
    #[test]
    fn test_forget() {
        let memory = NeuralMemory::new().unwrap();
        
        let id = memory.store(
            "https://example.com".to_string(),
            "Example".to_string(),
            "Content".to_string(),
        ).unwrap();
        
        memory.forget(id).unwrap();
        
        let stats = memory.get_stats();
        assert_eq!(stats.total_memories, 0);
    }
}