//! Annotations Module
//! 
//! This module provides annotation capabilities for articles,
//! allowing users to highlight text, add notes, and organize thoughts.
//! 
//! # Features
//! - Text highlighting with colors
//! - Add notes to highlighted text
//! - Organize annotations by tags
//! - Search and filter annotations
//! - Export and import annotations

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{Annotation, AnnotationPosition};

/// Annotation manager
pub struct AnnotationManager {
    annotations: Arc<RwLock<HashMap<String, Annotation>>>,
    annotations_by_article: Arc<RwLock<HashMap<String, Vec<String>>>>,
    colors: Vec<AnnotationColor>,
}

/// Annotation color
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationColor {
    pub name: String,
    pub hex: String,
    pub is_default: bool,
}

impl AnnotationColor {
    /// Create yellow highlight
    pub fn yellow() -> Self {
        Self {
            name: "Yellow".to_string(),
            hex: "#fff3cd".to_string(),
            is_default: true,
        }
    }

    /// Create green highlight
    pub fn green() -> Self {
        Self {
            name: "Green".to_string(),
            hex: "#d4edda".to_string(),
            is_default: false,
        }
    }

    /// Create blue highlight
    pub fn blue() -> Self {
        Self {
            name: "Blue".to_string(),
            hex: "#cce5ff".to_string(),
            is_default: false,
        }
    }

    /// Create pink highlight
    pub fn pink() -> Self {
        Self {
            name: "Pink".to_string(),
            hex: "#f8d7da".to_string(),
            is_default: false,
        }
    }

    /// Create orange highlight
    pub fn orange() -> Self {
        Self {
            name: "Orange".to_string(),
            hex: "#ffeaa7".to_string(),
            is_default: false,
        }
    }

    /// Create purple highlight
    pub fn purple() -> Self {
        Self {
            name: "Purple".to_string(),
            hex: "#e2d9f3".to_string(),
            is_default: false,
        }
    }
}

impl AnnotationManager {
    /// Create a new annotation manager
    pub fn new() -> Self {
        Self {
            annotations: Arc::new(RwLock::new(HashMap::new())),
            annotations_by_article: Arc::new(RwLock::new(HashMap::new())),
            colors: vec![
                AnnotationColor::yellow(),
                AnnotationColor::green(),
                AnnotationColor::blue(),
                AnnotationColor::pink(),
                AnnotationColor::orange(),
                AnnotationColor::purple(),
            ],
        }
    }

    /// Get available colors
    pub fn get_available_colors(&self) -> Vec<AnnotationColor> {
        self.colors.clone()
    }

    /// Create a new annotation
    pub async fn create_annotation(
        &self,
        article_id: &str,
        text: &str,
        note: Option<String>,
        color: &str,
    ) -> Result<Annotation, String> {
        // Validate color
        if !self.colors.iter().any(|c| c.hex == color) {
            return Err("Invalid color".to_string());
        }

        let id = Uuid::new_v4().to_string();
        let annotation = Annotation {
            id: id.clone(),
            article_id: article_id.to_string(),
            text: text.to_string(),
            note,
            color: color.to_string(),
            position: AnnotationPosition {
                offset: 0,
                length: text.len(),
                section: None,
            },
            created_at: Utc::now(),
        };

        // Store annotation
        self.annotations.write().await.insert(id.clone(), annotation.clone());

        // Update article index
        let mut by_article = self.annotations_by_article.write().await;
        by_article
            .entry(article_id.to_string())
            .or_insert_with(Vec::new)
            .push(id);

        Ok(annotation)
    }

    /// Create annotation with position
    pub async fn create_annotation_with_position(
        &self,
        article_id: &str,
        text: &str,
        note: Option<String>,
        color: &str,
        offset: usize,
        length: usize,
        section: Option<String>,
    ) -> Result<Annotation, String> {
        // Validate color
        if !self.colors.iter().any(|c| c.hex == color) {
            return Err("Invalid color".to_string());
        }

        let id = Uuid::new_v4().to_string();
        let annotation = Annotation {
            id: id.clone(),
            article_id: article_id.to_string(),
            text: text.to_string(),
            note,
            color: color.to_string(),
            position: AnnotationPosition {
                offset,
                length,
                section,
            },
            created_at: Utc::now(),
        };

        // Store annotation
        self.annotations.write().await.insert(id.clone(), annotation.clone());

        // Update article index
        let mut by_article = self.annotations_by_article.write().await;
        by_article
            .entry(article_id.to_string())
            .or_insert_with(Vec::new)
            .push(id);

        Ok(annotation)
    }

    /// Get annotation by ID
    pub async fn get_annotation(&self, id: &str) -> Option<Annotation> {
        self.annotations.read().await.get(id).cloned()
    }

    /// Get all annotations for an article
    pub async fn get_annotations_for_article(&self, article_id: &str) -> Vec<Annotation> {
        let by_article = self.annotations_by_article.read().await;
        
        if let Some(ids) = by_article.get(article_id) {
            let annotations = self.annotations.read().await;
            ids.iter()
                .filter_map(|id| annotations.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Update annotation
    pub async fn update_annotation(&self, id: &str, note: Option<String>) -> Result<(), String> {
        let mut annotations = self.annotations.write().await;
        
        if let Some(annotation) = annotations.get_mut(id) {
            annotation.note = note;
            Ok(())
        } else {
            Err("Annotation not found".to_string())
        }
    }

    /// Update annotation color
    pub async fn update_annotation_color(&self, id: &str, color: &str) -> Result<(), String> {
        if !self.colors.iter().any(|c| c.hex == color) {
            return Err("Invalid color".to_string());
        }

        let mut annotations = self.annotations.write().await;
        
        if let Some(annotation) = annotations.get_mut(id) {
            annotation.color = color.to_string();
            Ok(())
        } else {
            Err("Annotation not found".to_string())
        }
    }

    /// Delete annotation
    pub async fn delete_annotation(&self, id: &str) -> Result<(), String> {
        // Get article_id before removing
        let article_id = {
            let annotations = self.annotations.read().await;
            annotations.get(id)
                .map(|a| a.article_id.clone())
        };

        // Remove annotation
        self.annotations.write().await.remove(id);

        // Update article index
        if let Some(article_id) = article_id {
            let mut by_article = self.annotations_by_article.write().await;
            if let Some(ids) = by_article.get_mut(&article_id) {
                ids.retain(|x| x != id);
                if ids.is_empty() {
                    by_article.remove(&article_id);
                }
            }
        }

        Ok(())
    }

    /// Search annotations by text
    pub async fn search_annotations(&self, query: &str) -> Vec<Annotation> {
        let query_lower = query.to_lowercase();
        let annotations = self.annotations.read().await;
        
        annotations.values()
            .filter(|a| {
                a.text.to_lowercase().contains(&query_lower)
                    || a.note.as_ref()
                        .map(|n| n.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    /// Get annotations by color
    pub async fn get_annotations_by_color(&self, color: &str) -> Vec<Annotation> {
        let annotations = self.annotations.read().await;
        
        annotations.values()
            .filter(|a| a.color == color)
            .cloned()
            .collect()
    }

    /// Get all annotations
    pub async fn get_all_annotations(&self) -> Vec<Annotation> {
        self.annotations.read().await.values().cloned().collect()
    }

    /// Get annotation statistics
    pub async fn get_statistics(&self) -> AnnotationStatistics {
        let annotations = self.annotations.read().await;
        
        let total_annotations = annotations.len();
        let unique_articles = self.annotations_by_article.read().await.len();
        
        let mut color_counts: HashMap<String, usize> = HashMap::new();
        for annotation in annotations.values() {
            *color_counts.entry(annotation.color.clone()).or_insert(0) += 1;
        }

        let total_notes = annotations.values()
            .filter(|a| a.note.is_some())
            .count();

        AnnotationStatistics {
            total_annotations,
            unique_articles,
            color_counts,
            total_notes,
        }
    }

    /// Export annotations to JSON
    pub async fn export(&self, article_id: Option<&str>) -> Result<String, String> {
        let annotations = if let Some(id) = article_id {
            self.get_annotations_for_article(id).await
        } else {
            self.get_all_annotations().await
        };

        serde_json::to_string_pretty(&annotations)
            .map_err(|e| format!("Export failed: {}", e))
    }

    /// Import annotations from JSON
    pub async fn import(&self, json: &str) -> Result<usize, String> {
        let imported: Vec<Annotation> = serde_json::from_str(json)
            .map_err(|e| format!("Import failed: {}", e))?;
        
        let count = imported.len();
        
        for annotation in imported {
            let id = annotation.id.clone();
            let article_id = annotation.article_id.clone();
            
            // Store annotation
            self.annotations.write().await.insert(id.clone(), annotation);

            // Update article index
            let mut by_article = self.annotations_by_article.write().await;
            by_article
                .entry(article_id)
                .or_insert_with(Vec::new)
                .push(id);
        }
        
        Ok(count)
    }

    /// Clear all annotations for an article
    pub async fn clear_annotations_for_article(&self, article_id: &str) {
        let by_article = self.annotations_by_article.read().await;
        
        if let Some(ids) = by_article.get(article_id) {
            // Remove all annotations
            let mut annotations = self.annotations.write().await;
            for id in ids {
                annotations.remove(id);
            }
        }

        // Clear article index
        self.annotations_by_article.write().await.remove(article_id);
    }

    /// Clear all annotations
    pub async fn clear_all(&self) {
        self.annotations.write().await.clear();
        self.annotations_by_article.write().await.clear();
    }
}

impl Default for AnnotationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Annotation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationStatistics {
    pub total_annotations: usize,
    pub unique_articles: usize,
    pub color_counts: HashMap<String, usize>,
    pub total_notes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_annotation() {
        let manager = AnnotationManager::new();
        
        let annotation = manager.create_annotation(
            "article-1",
            "This is highlighted text",
            Some("My note".to_string()),
            "#fff3cd",
        ).await.unwrap();
        
        assert_eq!(annotation.article_id, "article-1");
        assert_eq!(annotation.text, "This is highlighted text");
        assert_eq!(annotation.note, Some("My note".to_string()));
    }

    #[tokio::test]
    async fn test_get_annotations_for_article() {
        let manager = AnnotationManager::new();
        
        manager.create_annotation("article-1", "Text 1", None, "#fff3cd").await.unwrap();
        manager.create_annotation("article-1", "Text 2", None, "#d4edda").await.unwrap();
        manager.create_annotation("article-2", "Text 3", None, "#cce5ff").await.unwrap();
        
        let annotations = manager.get_annotations_for_article("article-1").await;
        assert_eq!(annotations.len(), 2);
    }

    #[tokio::test]
    async fn test_search_annotations() {
        let manager = AnnotationManager::new();
        
        manager.create_annotation(
            "article-1",
            "Important information",
            Some("Remember this".to_string()),
            "#fff3cd",
        ).await.unwrap();
        
        manager.create_annotation(
            "article-1",
            "Regular text",
            None,
            "#d4edda",
        ).await.unwrap();
        
        let results = manager.search_annotations("important").await;
        assert_eq!(results.len(), 1);
        
        let results = manager.search_annotations("Remember").await;
        assert_eq!(results.len(), 1);
    }
}