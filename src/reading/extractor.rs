//! Article Content Extractor
//! 
//! This module extracts article content from HTML pages using various heuristics
//! and algorithms to identify the main content, remove clutter, and extract metadata.
//! 
//! # Features
//! - Main content extraction using readability algorithms
//! - Metadata extraction (title, author, date)
//! - Image and link extraction
//! - Multi-language support
//! - Video detection

use std::collections::HashMap;
use regex::Regex;
use super::{ArticleMetadata, ArticleContent, ArticleImage, ArticleLink, ArticleVideo, ReadingError, Result};

/// Article extractor
pub struct ArticleExtractor {
    content_selectors: Vec<String>,
    metadata_selectors: HashMap<String, String>,
}

impl ArticleExtractor {
    /// Create a new article extractor
    pub fn new() -> Self {
        Self {
            content_selectors: vec![
                "article".to_string(),
                "main".to_string(),
                "[role=main]".to_string(),
                ".post-content".to_string(),
                ".article-content".to_string(),
                ".entry-content".to_string(),
                ".content".to_string(),
                "#content".to_string(),
                ".story-body".to_string(),
                ".post-body".to_string(),
            ],
            metadata_selectors: {
                let mut map = HashMap::new();
                map.insert("title".to_string(), "h1, title, .title, .article-title, [itemprop=headline]".to_string());
                map.insert("author".to_string(), ".author, .byline, [itemprop=author], .post-author".to_string());
                map.insert("date".to_string(), "time, .date, .published, [itemprop=datePublished]".to_string());
                map.insert("excerpt".to_string(), ".excerpt, .summary, .article-summary, [itemprop=description]".to_string());
                map
            },
        }
    }

    /// Extract article metadata from HTML
    pub async fn extract_metadata(&self, url: &str, html: &str) -> Result<ArticleMetadata> {
        // Parse HTML (simplified - in real implementation would use scraper/HTML parser)
        let title = self.extract_title(html)?;
        let author = self.extract_author(html);
        let published_date = self.extract_date(html);
        let excerpt = self.extract_excerpt(html);
        let featured_image = self.extract_featured_image(html);
        let language = self.extract_language(html);
        let tags = self.extract_tags(html);

        let domain = self.extract_domain(url);

        Ok(ArticleMetadata {
            title,
            url: url.to_string(),
            author,
            published_date,
            excerpt,
            featured_image,
            reading_time_minutes: 0, // Will be calculated later
            word_count: 0, // Will be calculated later
            domain,
            language,
            tags,
            extracted_at: chrono::Utc::now(),
        })
    }

    /// Extract article content from HTML
    pub async fn extract_content(&self, html: &str) -> Result<ArticleContent> {
        // Remove script and style elements
        let cleaned_html = self.remove_scripts_and_styles(html);

        // Extract main content
        let content_html = self.extract_main_content(&cleaned_html)?;

        // Extract text from HTML
        let text = self.html_to_text(&content_html);

        // Extract images
        let images = self.extract_images(&content_html);

        // Extract links
        let links = self.extract_links(&content_html);

        // Extract videos
        let videos = self.extract_videos(&content_html);

        Ok(ArticleContent {
            html: content_html,
            text,
            images,
            links,
            videos,
        })
    }

    /// Count words in text
    pub fn count_words(&self, text: &str) -> u32 {
        text.split_whitespace().count() as u32
    }

    fn extract_title(&self, html: &str) -> Result<String> {
        // Try multiple selectors
        let selectors = vec![
            "h1",
            "title",
            "[property=og:title]",
            "[name=twitter:title]",
        ];

        for selector in selectors {
            if let Some(title) = self.extract_by_simple_selector(html, selector) {
                if !title.trim().is_empty() {
                    return Ok(self.clean_text(title));
                }
            }
        }

        Err(ReadingError::ExtractionFailed("Could not extract title".to_string()))
    }

    fn extract_author(&self, html: &str) -> Option<String> {
        let selectors = vec![
            ".author",
            ".byline",
            "[itemprop=author]",
            ".post-author",
            "[name=author]",
        ];

        for selector in selectors {
            if let Some(author) = self.extract_by_simple_selector(html, selector) {
                let cleaned = self.clean_text(author);
                if !cleaned.is_empty() && cleaned.len() < 100 {
                    return Some(cleaned);
                }
            }
        }

        None
    }

    fn extract_date(&self, html: &str) -> Option<chrono::DateTime<chrono::Utc>> {
        let selectors = vec![
            "time[datetime]",
            "[itemprop=datePublished]",
            ".date",
            ".published",
        ];

        for selector in selectors {
            if let Some(date_str) = self.extract_attribute_by_selector(html, selector, "datetime") {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&date_str) {
                    return Some(dt.with_timezone(&chrono::Utc));
                }
            }
        }

        None
    }

    fn extract_excerpt(&self, html: &str) -> Option<String> {
        let selectors = vec![
            "[property=og:description]",
            "[name=description]",
            "[itemprop=description]",
            ".excerpt",
            ".summary",
        ];

        for selector in selectors {
            if let Some(excerpt) = self.extract_by_simple_selector(html, selector) {
                let cleaned = self.clean_text(excerpt);
                if !cleaned.is_empty() {
                    return Some(cleaned.chars().take(300).collect());
                }
            }
        }

        None
    }

    fn extract_featured_image(&self, html: &str) -> Option<String> {
        let selectors = vec![
            "[property=og:image]",
            "[name=twitter:image]",
            "article img:first-of-type",
            ".featured-image img",
        ];

        for selector in selectors {
            if let Some(img_url) = self.extract_attribute_by_selector(html, selector, "src") {
                if !img_url.trim().is_empty() {
                    return Some(img_url);
                }
            }
        }

        None
    }

    fn extract_language(&self, html: &str) -> Option<String> {
        self.extract_attribute_by_selector(html, "html", "lang")
            .map(|lang| lang.split(',').next().unwrap_or("").to_string())
            .filter(|lang| !lang.is_empty())
    }

    fn extract_tags(&self, html: &str) -> Vec<String> {
        let mut tags = Vec::new();

        // Extract meta keywords
        if let Some(keywords) = self.extract_attribute_by_selector(html, "[name=keywords]", "content") {
            tags.extend(keywords.split(',').map(|s| s.trim().to_string()));
        }

        // Extract article tags
        if let Some(tag_text) = self.extract_by_simple_selector(html, ".tags, .post-tags") {
            let tag_regex = Regex::new(r"[^a-zA-Z0-9\s]").unwrap();
            for word in tag_text.split_whitespace() {
                let cleaned = tag_regex.replace_all(word, "").to_string();
                if cleaned.len() > 2 {
                    tags.push(cleaned);
                }
            }
        }

        tags.into_iter().take(10).collect()
    }

    fn extract_domain(&self, url: &str) -> String {
        if let Ok(parsed) = url::Url::parse(url) {
            parsed.host_str().unwrap_or("unknown").to_string()
        } else {
            "unknown".to_string()
        }
    }

    fn remove_scripts_and_styles(&self, html: &str) -> String {
        let script_regex = Regex::new(r"<script[^>]*>.*?</script>").unwrap();
        let style_regex = Regex::new(r"<style[^>]*>.*?</style>").unwrap();
        let comment_regex = Regex::new(r"<!--.*?-->").unwrap();

        let html = script_regex.replace_all(html, "");
        let html = style_regex.replace_all(&html, "");
        let html = comment_regex.replace_all(&html, "");

        html.to_string()
    }

    fn extract_main_content(&self, html: &str) -> Result<String> {
        // Simplified content extraction - would use proper HTML parsing in production
        // Try to find article, main, or content divs

        for selector in &self.content_selectors {
            if let Some(content) = self.extract_by_simple_selector(html, selector) {
                if self.is_valid_content(&content) {
                    return Ok(content);
                }
            }
        }

        // Fallback: extract paragraphs
        let paragraph_regex = Regex::new(r"<p[^>]*>.*?</p>").unwrap();
        let paragraphs: Vec<&str> = paragraph_regex.find_iter(html)
            .map(|m| m.as_str())
            .collect();

        if paragraphs.is_empty() {
            Err(ReadingError::ExtractionFailed("Could not extract article content".to_string()))
        } else {
            Ok(paragraphs.join("\n"))
        }
    }

    fn is_valid_content(&self, html: &str) -> bool {
        let text = self.html_to_text(html);
        let word_count = text.split_whitespace().count();
        word_count >= 100
    }

    fn html_to_text(&self, html: &str) -> String {
        // Remove HTML tags
        let tag_regex = Regex::new(r"<[^>]+>").unwrap();
        let text = tag_regex.replace_all(html, " ");

        // Decode HTML entities
        let text = text
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "&quot;")
            .replace("&apos;", "'")
            .replace("&nbsp;", " ");

        // Normalize whitespace
        let whitespace_regex = Regex::new(r"\s+").unwrap();
        whitespace_regex.replace_all(&text, " ").trim().to_string()
    }

    fn extract_images(&self, html: &str) -> Vec<ArticleImage> {
        let img_regex = Regex::new(r#"<img[^>]+src=["']([^"']+)["'][^>]*>"#).unwrap();
        let mut images = Vec::new();

        for cap in img_regex.captures_iter(html) {
            if let Some(url) = cap.get(1) {
                let img_tag = cap.get(0).unwrap().as_str();
                
                let alt = self.extract_attribute(img_tag, "alt")
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string());

                let width = self.extract_attribute(img_tag, "width")
                    .and_then(|s| s.parse::<u32>().ok());

                let height = self.extract_attribute(img_tag, "height")
                    .and_then(|s| s.parse::<u32>().ok());

                images.push(ArticleImage {
                    url: url.as_str().to_string(),
                    caption: None,
                    alt_text: alt,
                    width,
                    height,
                });
            }
        }

        images
    }

    fn extract_links(&self, html: &str) -> Vec<ArticleLink> {
        let link_regex = Regex::new(r#"<a[^>]+href=["']([^"']+)["'][^>]*>(.*?)</a>"#).unwrap();
        let mut links = Vec::new();

        for cap in link_regex.captures_iter(html) {
            let url = cap.get(1).unwrap().as_str().to_string();
            let text = self.html_to_text(cap.get(2).unwrap().as_str());

            if !url.is_empty() && !text.is_empty() {
                links.push(ArticleLink {
                    url,
                    text,
                    title: None,
                });
            }
        }

        links
    }

    fn extract_videos(&self, html: &str) -> Vec<ArticleVideo> {
        let mut videos = Vec::new();

        // YouTube videos
        let youtube_regex = Regex::new(r#"(?:youtube\.com/watch\?v=|youtu\.be/)([a-zA-Z0-9_-]+)"#).unwrap();
        for cap in youtube_regex.captures_iter(html) {
            let video_id = cap.get(1).unwrap().as_str();
            videos.push(ArticleVideo {
                url: format!("https://www.youtube.com/watch?v={}", video_id),
                thumbnail: Some(format!("https://img.youtube.com/vi/{}/0.jpg", video_id)),
                caption: None,
                duration: None,
            });
        }

        // HTML5 video tags
        let video_regex = Regex::new(r#"<video[^>]+src=["']([^"']+)["'][^>]*>"#).unwrap();
        for cap in video_regex.captures_iter(html) {
            videos.push(ArticleVideo {
                url: cap.get(1).unwrap().as_str().to_string(),
                thumbnail: None,
                caption: None,
                duration: None,
            });
        }

        videos
    }

    fn extract_by_simple_selector(&self, html: &str, selector: &str) -> Option<String> {
        // Simplified selector matching - production would use proper HTML parser
        let tag = selector.split(',').next()?
            .trim()
            .trim_start_matches('.')
            .trim_start_matches('#')
            .split_whitespace()
            .next()?;

        let open_tag = format!("<{}", tag);
        let close_tag = format!("</{}>", tag);

        if let Some(start) = html.find(&open_tag) {
            if let Some(end) = html[start..].find(&close_tag) {
                let content_start = start + open_tag.len();
                let content = &html[content_start..start + end];
                return Some(content.trim().to_string());
            }
        }

        None
    }

    fn extract_attribute_by_selector(&self, html: &str, selector: &str, attr: &str) -> Option<String> {
        if let Some(element) = self.extract_element(html, selector) {
            self.extract_attribute(&element, attr)
                .map(|s| s.to_string())
        } else {
            None
        }
    }

    fn extract_element(&self, html: &str, selector: &str) -> Option<&str> {
        // Simplified - would use proper parser
        let tag = selector.split('[').next()?.trim();
        self.extract_by_simple_selector(html, tag)
            .map(|s| Box::leak(s.into_boxed_str()) as &str)
    }

    fn extract_attribute(&self, element: &str, attr: &str) -> Option<&str> {
        let pattern = format!(r#"{}=["']([^"']*)["']"#, attr);
        let regex = Regex::new(&pattern).ok()?;
        
        regex.captures(element)?.get(1).map(|m| m.as_str())
    }

    fn clean_text(&self, text: String) -> String {
        let whitespace_regex = Regex::new(r"\s+").unwrap();
        whitespace_regex.replace_all(&text, " ").trim().to_string()
    }
}

impl Default for ArticleExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_extract_title() {
        let extractor = ArticleExtractor::new();
        let html = r#"<html><head><title>Test Article</title></head><body></body></html>"#;
        let metadata = extractor.extract_metadata("https://example.com/article", html).await;
        assert!(metadata.is_ok());
        assert_eq!(metadata.unwrap().title, "Test Article");
    }

    #[test]
    fn test_count_words() {
        let extractor = ArticleExtractor::new();
        let text = "This is a test string with some words.";
        assert_eq!(extractor.count_words(text), 8);
    }

    #[test]
    fn test_html_to_text() {
        let extractor = ArticleExtractor::new();
        let html = "<p>This is <strong>bold</strong> text.</p>";
        let text = extractor.html_to_text(html);
        assert_eq!(text, "This is bold text.");
    }
}