//! Reading Mode Module
//! 
//! This module provides a distraction-free reading experience for web articles.
//! It extracts article content, formats it for readability, and provides customization options.
//! 
//! # Features
//! - Article content extraction from web pages
//! - Distraction-free reading mode
//! - Font and theme customization
//! - Article storage for offline access
//! - Text-to-speech integration
//! - Highlighting and annotations

pub mod extractor;
pub mod formatter;
pub mod theme;
pub mod storage;
pub mod tts;
pub mod annotations;

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur in reading mode operations
#[derive(Error, Debug)]
pub enum ReadingError {
    #[error("Extraction failed: {0}")]
    ExtractionFailed(String),
    #[error("Formatting failed: {0}")]
    FormattingFailed(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("TTS error: {0}")]
    TTSError(String),
    #[error("Invalid article: {0}")]
    InvalidArticle(String),
}

/// Result type for reading mode operations
pub type Result<T> = std::result::Result<T, ReadingError>;

/// Article metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleMetadata {
    pub title: String,
    pub url: String,
    pub author: Option<String>,
    pub published_date: Option<chrono::DateTime<chrono::Utc>>,
    pub excerpt: Option<String>,
    pub featured_image: Option<String>,
    pub reading_time_minutes: u32,
    pub word_count: u32,
    pub domain: String,
    pub language: Option<String>,
    pub tags: Vec<String>,
    pub extracted_at: chrono::DateTime<chrono::Utc>,
}

/// Article content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleContent {
    pub html: String,
    pub text: String,
    pub images: Vec<ArticleImage>,
    pub links: Vec<ArticleLink>,
    pub videos: Vec<ArticleVideo>,
}

/// Image in article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleImage {
    pub url: String,
    pub caption: Option<String>,
    pub alt_text: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// Link in article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleLink {
    pub url: String,
    pub text: String,
    pub title: Option<String>,
}

/// Video in article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleVideo {
    pub url: String,
    pub thumbnail: Option<String>,
    pub caption: Option<String>,
    pub duration: Option<u32>,
}

/// Complete article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub id: String,
    pub metadata: ArticleMetadata,
    pub content: ArticleContent,
    pub reading_progress: ReadingProgress,
}

impl Article {
    /// Calculate estimated reading time
    pub fn calculate_reading_time(word_count: u32) -> u32 {
        // Average reading speed: 200-250 words per minute
        ((word_count as f64) / 225.0).ceil() as u32
    }

    /// Get reading progress percentage
    pub fn progress_percentage(&self) -> f64 {
        self.reading_progress.percentage()
    }

    /// Mark article as read
    pub fn mark_as_read(&mut self) {
        self.reading_progress.mark_complete();
    }

    /// Update reading position
    pub fn update_position(&mut self, scroll_position: f64) {
        self.reading_progress.scroll_position = scroll_position;
        self.reading_progress.last_read_at = Some(chrono::Utc::now());
    }
}

/// Reading progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingProgress {
    pub scroll_position: f64, // 0.0 to 1.0
    pub current_section: Option<String>,
    pub last_read_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_completed: bool,
    pub time_spent_minutes: u32,
}

impl ReadingProgress {
    /// Create new reading progress
    pub fn new() -> Self {
        Self {
            scroll_position: 0.0,
            current_section: None,
            last_read_at: None,
            is_completed: false,
            time_spent_minutes: 0,
        }
    }

    /// Get progress percentage
    pub fn percentage(&self) -> f64 {
        self.scroll_position * 100.0
    }

    /// Mark as complete
    pub fn mark_complete(&mut self) {
        self.is_completed = true;
        self.scroll_position = 1.0;
        self.last_read_at = Some(chrono::Utc::now());
    }

    /// Check if started
    pub fn is_started(&self) -> bool {
        self.scroll_position > 0.0 || self.last_read_at.is_some()
    }
}

impl Default for ReadingProgress {
    fn default() -> Self {
        Self::new()
    }
}

/// Reading mode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingModeConfig {
    pub theme: ReadingTheme,
    pub font_family: String,
    pub font_size: u16,
    pub line_height: f32,
    pub text_alignment: TextAlignment,
    pub margin_width: u16,
    pub show_images: bool,
    pub show_videos: bool,
    pub enable_highlights: bool,
    pub auto_scroll_enabled: bool,
    pub auto_scroll_speed: u8,
}

impl Default for ReadingModeConfig {
    fn default() -> Self {
        Self {
            theme: ReadingTheme::Light,
            font_family: "Georgia".to_string(),
            font_size: 18,
            line_height: 1.6,
            text_alignment: TextAlignment::Left,
            margin_width: 60,
            show_images: true,
            show_videos: true,
            enable_highlights: true,
            auto_scroll_enabled: false,
            auto_scroll_speed: 50,
        }
    }
}

/// Reading theme
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReadingTheme {
    Light,
    Dark,
    Sepia,
    HighContrast,
    Custom,
}

/// Text alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAlignment {
    Left,
    Center,
    Justify,
}

/// Annotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub id: String,
    pub article_id: String,
    pub text: String,
    pub note: Option<String>,
    pub color: String,
    pub position: AnnotationPosition,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Position of annotation in article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationPosition {
    pub offset: usize,
    pub length: usize,
    pub section: Option<String>,
}

/// Reading mode manager
pub struct ReadingModeManager {
    config: Arc<RwLock<ReadingModeConfig>>,
    extractor: Arc<extractor::ArticleExtractor>,
    formatter: Arc<formatter::ArticleFormatter>,
    storage: Arc<storage::ArticleStorage>,
    tts: Arc<tts::TextToSpeech>,
    is_enabled: Arc<RwLock<bool>>,
}

impl ReadingModeManager {
    /// Create a new reading mode manager
    pub fn new(config: ReadingModeConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            extractor: Arc::new(extractor::ArticleExtractor::new()),
            formatter: Arc::new(formatter::ArticleFormatter::new()),
            storage: Arc::new(storage::ArticleStorage::new()),
            tts: Arc::new(tts::TextToSpeech::new()),
            is_enabled: Arc::new(RwLock::new(false)),
        }
    }

    /// Enable reading mode
    pub async fn enable(&self) {
        *self.is_enabled.write().await = true;
    }

    /// Disable reading mode
    pub async fn disable(&self) {
        *self.is_enabled.write().await = false;
    }

    /// Check if reading mode is enabled
    pub async fn is_enabled(&self) -> bool {
        *self.is_enabled.read().await
    }

    /// Extract article from URL
    pub async fn extract_article(&self, url: &str, html: &str) -> Result<Article> {
        if !self.is_enabled().await {
            return Err(ReadingError::InvalidArticle(
                "Reading mode is not enabled".to_string()
            ));
        }

        let metadata = self.extractor.extract_metadata(url, html).await?;
        let content = self.extractor.extract_content(html).await?;

        let word_count = self.extractor.count_words(&content.text);
        let reading_time = Article::calculate_reading_time(word_count);

        let mut metadata = metadata;
        metadata.reading_time_minutes = reading_time;
        metadata.word_count = word_count;
        metadata.extracted_at = chrono::Utc::now();

        let article = Article {
            id: uuid::Uuid::new_v4().to_string(),
            metadata,
            content,
            reading_progress: ReadingProgress::new(),
        };

        Ok(article)
    }

    /// Format article for reading
    pub async fn format_article(&self, article: &Article) -> Result<String> {
        let config = self.config.read().await;
        self.formatter.format(article, &config).await
    }

    /// Save article for offline reading
    pub async fn save_article(&self, article: &Article) -> Result<()> {
        self.storage.save(article).await
    }

    /// Load saved article
    pub async fn load_article(&self, id: &str) -> Result<Option<Article>> {
        self.storage.load(id).await
    }

    /// Get all saved articles
    pub async fn get_saved_articles(&self) -> Result<Vec<Article>> {
        self.storage.list().await
    }

    /// Delete saved article
    pub async fn delete_article(&self, id: &str) -> Result<()> {
        self.storage.delete(id).await
    }

    /// Update reading mode configuration
    pub async fn update_config(&self, config: ReadingModeConfig) {
        *self.config.write().await = config;
    }

    /// Get current configuration
    pub async fn get_config(&self) -> ReadingModeConfig {
        self.config.read().await.clone()
    }

    /// Search saved articles
    pub async fn search_articles(&self, query: &str) -> Result<Vec<Article>> {
        self.storage.search(query).await
    }

    /// Get reading statistics
    pub async fn get_statistics(&self) -> ReadingStatistics {
        let articles = self.storage.list().await.unwrap_or_default();
        let total_articles = articles.len();
        let completed_articles = articles.iter()
            .filter(|a| a.reading_progress.is_completed)
            .count();

        let total_words: u32 = articles.iter()
            .map(|a| a.metadata.word_count)
            .sum();

        let total_reading_time: u32 = articles.iter()
            .map(|a| a.metadata.reading_time_minutes)
            .sum();

        let total_time_spent: u32 = articles.iter()
            .map(|a| a.reading_progress.time_spent_minutes)
            .sum();

        ReadingStatistics {
            total_articles,
            completed_articles,
            total_words,
            total_reading_time,
            total_time_spent,
            average_reading_time: if total_articles > 0 {
                total_reading_time / total_articles as u32
            } else {
                0
            },
        }
    }
}

/// Reading statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingStatistics {
    pub total_articles: usize,
    pub completed_articles: usize,
    pub total_words: u32,
    pub total_reading_time: u32,
    pub total_time_spent: u32,
    pub average_reading_time: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reading_progress() {
        let mut progress = ReadingProgress::new();
        assert_eq!(progress.percentage(), 0.0);
        assert!(!progress.is_started());

        progress.scroll_position = 0.5;
        assert_eq!(progress.percentage(), 50.0);

        progress.mark_complete();
        assert_eq!(progress.percentage(), 100.0);
        assert!(progress.is_completed);
    }

    #[test]
    fn test_reading_time_calculation() {
        let time_200_words = Article::calculate_reading_time(200);
        assert_eq!(time_200_words, 1);

        let time_450_words = Article::calculate_reading_time(450);
        assert_eq!(time_450_words, 2);

        let time_225_words = Article::calculate_reading_time(225);
        assert_eq!(time_225_words, 1);
    }
}