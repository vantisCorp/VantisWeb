//! PDF Annotation Tools
//!
//! Highlight, underline, strikethrough, drawing, sticky notes, stamps, signatures.

use std::collections::HashMap;
use crate::pdf::{PDFAnnotation, AnnotationType, PDFError};

/// Annotation manager
pub struct AnnotationManager {
    annotations: HashMap<u32, Vec<PDFAnnotation>>,
}

impl AnnotationManager {
    /// Create a new annotation manager
    pub fn new() -> Self {
        Self {
            annotations: HashMap::new(),
        }
    }

    /// Add annotation
    pub async fn add(&mut self, annotation: PDFAnnotation) -> Result<(), PDFError> {
        let page = annotation.page;
        let id = annotation.id.clone();
        
        self.annotations
            .entry(page)
            .or_insert_with(Vec::new)
            .push(annotation);

        log::info!("Added annotation {} to page {}", id, page);
        Ok(())
    }

    /// Remove annotation
    pub async fn remove(&mut self, annotation_id: &str) -> Result<(), PDFError> {
        for (page, annotations) in self.annotations.iter_mut() {
            if let Some(pos) = annotations.iter().position(|a| a.id == annotation_id) {
                annotations.remove(pos);
                log::info!("Removed annotation {} from page {}", annotation_id, page);
                return Ok(());
            }
        }

        Err(PDFError::InvalidOperation(format!("Annotation not found: {}", annotation_id)))
    }

    /// Get annotations for page
    pub async fn get_for_page(&self, page: u32) -> Result<Vec<PDFAnnotation>, PDFError> {
        Ok(self.annotations.get(&page).cloned().unwrap_or_default())
    }

    /// Get all annotations
    pub fn get_all(&self) -> Vec<PDFAnnotation> {
        self.annotations.values().flat_map(|v| v.iter().cloned()).collect()
    }

    /// Create highlight annotation
    pub fn create_highlight(page: u32, bounds: (f32, f32, f32, f32), color: String) -> PDFAnnotation {
        PDFAnnotation {
            id: uuid::Uuid::new_v4().to_string(),
            annotation_type: AnnotationType::Highlight,
            page,
            bounds,
            content: String::new(),
            color,
            opacity: 0.3,
            author: None,
            created_at: chrono::Utc::now(),
        }
    }

    /// Create underline annotation
    pub fn create_underline(page: u32, bounds: (f32, f32, f32, f32), color: String) -> PDFAnnotation {
        PDFAnnotation {
            id: uuid::Uuid::new_v4().to_string(),
            annotation_type: AnnotationType::Underline,
            page,
            bounds,
            content: String::new(),
            color,
            opacity: 1.0,
            author: None,
            created_at: chrono::Utc::now(),
        }
    }

    /// Create strikethrough annotation
    pub fn create_strikethrough(page: u32, bounds: (f32, f32, f32, f32), color: String) -> PDFAnnotation {
        PDFAnnotation {
            id: uuid::Uuid::new_v4().to_string(),
            annotation_type: AnnotationType::Strikethrough,
            page,
            bounds,
            content: String::new(),
            color,
            opacity: 1.0,
            author: None,
            created_at: chrono::Utc::now(),
        }
    }

    /// Create sticky note annotation
    pub fn create_sticky_note(page: u32, x: f32, y: f32, content: String, color: String) -> PDFAnnotation {
        PDFAnnotation {
            id: uuid::Uuid::new_v4().to_string(),
            annotation_type: AnnotationType::StickyNote,
            page,
            bounds: (x, y, 200.0, 150.0),
            content,
            color,
            opacity: 1.0,
            author: None,
            created_at: chrono::Utc::now(),
        }
    }

    /// Create freehand drawing annotation
    pub fn create_freehand(page: u32, points: Vec<(f32, f32)>, color: String, width: f32) -> PDFAnnotation {
        // Calculate bounds from points
        let min_x = points.iter().map(|p| p.0).fold(f32::MAX, f32::min);
        let max_x = points.iter().map(|p| p.0).fold(f32::MIN, f32::max);
        let min_y = points.iter().map(|p| p.1).fold(f32::MAX, f32::min);
        let max_y = points.iter().map(|p| p.1).fold(f32::MIN, f32::max);

        PDFAnnotation {
            id: uuid::Uuid::new_v4().to_string(),
            annotation_type: AnnotationType::FreeHand,
            page,
            bounds: (min_x, min_y, max_x - min_x, max_y - min_y),
            content: format!("{:?}", points), // Store points as content
            color,
            opacity: 1.0,
            author: None,
            created_at: chrono::Utc::now(),
        }
    }

    /// Create stamp annotation
    pub fn create_stamp(page: u32, x: f32, y: f32, stamp_type: StampType) -> PDFAnnotation {
        let (text, color) = match stamp_type {
            StampType::Approved => ("APPROVED", "#00FF00"),
            StampType::Rejected => ("REJECTED", "#FF0000"),
            StampType::Draft => ("DRAFT", "#808080"),
            StampType::Confidential => ("CONFIDENTIAL", "#FFA500"),
            StampType::Final => ("FINAL", "#0000FF"),
            StampType::Custom(text, custom_color) => (text.as_str(), custom_color.as_str()),
        };

        PDFAnnotation {
            id: uuid::Uuid::new_v4().to_string(),
            annotation_type: AnnotationType::Stamp,
            page,
            bounds: (x, y, 120.0, 40.0),
            content: text.to_string(),
            color: color.to_string(),
            opacity: 0.7,
            author: None,
            created_at: chrono::Utc::now(),
        }
    }

    /// Create signature annotation
    pub fn create_signature(page: u32, x: f32, y: f32, signature_data: Vec<u8>) -> PDFAnnotation {
        PDFAnnotation {
            id: uuid::Uuid::new_v4().to_string(),
            annotation_type: AnnotationType::Signature,
            page,
            bounds: (x, y, 150.0, 50.0),
            content: base64::encode(&signature_data),
            color: "#000000".to_string(),
            opacity: 1.0,
            author: None,
            created_at: chrono::Utc::now(),
        }
    }

    /// Update annotation
    pub async fn update(&mut self, annotation_id: &str, updates: AnnotationUpdate) -> Result<(), PDFError> {
        for annotations in self.annotations.values_mut() {
            if let Some(annotation) = annotations.iter_mut().find(|a| a.id == annotation_id) {
                if let Some(color) = &updates.color {
                    annotation.color = color.clone();
                }
                if let Some(opacity) = updates.opacity {
                    annotation.opacity = opacity;
                }
                if let Some(content) = &updates.content {
                    annotation.content = content.clone();
                }
                log::info!("Updated annotation {}", annotation_id);
                return Ok(());
            }
        }

        Err(PDFError::InvalidOperation(format!("Annotation not found: {}", annotation_id)))
    }

    /// Clear all annotations
    pub fn clear(&mut self) {
        self.annotations.clear();
    }

    /// Clear annotations for page
    pub fn clear_page(&mut self, page: u32) {
        self.annotations.remove(&page);
    }

    /// Get annotation count
    pub fn count(&self) -> usize {
        self.annotations.values().map(|v| v.len()).sum()
    }

    /// Export annotations to FDF format
    pub fn export_fdf(&self) -> Result<String, PDFError> {
        // In a real implementation, this would generate FDF format
        let mut fdf = String::from("%FDF-1.2\n");
        for annotation in self.get_all() {
            fdf.push_str(&format!("<< /Type /Annot /Subtype /{:?} >>\n", annotation.annotation_type));
        }
        fdf.push_str("%%EOF\n");
        Ok(fdf)
    }
}

/// Stamp types
#[derive(Debug, Clone)]
pub enum StampType {
    Approved,
    Rejected,
    Draft,
    Confidential,
    Final,
    Custom(String, String), // (text, color)
}

/// Annotation update
#[derive(Debug, Clone, Default)]
pub struct AnnotationUpdate {
    pub color: Option<String>,
    pub opacity: Option<f32>,
    pub content: Option<String>,
}

impl Default for AnnotationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_annotation_manager_creation() {
        let manager = AnnotationManager::new();
        assert_eq!(manager.count(), 0);
    }

    #[tokio::test]
    async fn test_add_annotation() {
        let mut manager = AnnotationManager::new();
        let annotation = PDFAnnotation {
            id: "test-1".to_string(),
            annotation_type: AnnotationType::Highlight,
            page: 1,
            bounds: (100.0, 100.0, 200.0, 50.0),
            content: String::new(),
            color: "#FFFF00".to_string(),
            opacity: 0.3,
            author: None,
            created_at: chrono::Utc::now(),
        };
        
        manager.add(annotation).await.unwrap();
        assert_eq!(manager.count(), 1);
    }

    #[tokio::test]
    async fn test_remove_annotation() {
        let mut manager = AnnotationManager::new();
        let annotation = PDFAnnotation {
            id: "test-1".to_string(),
            annotation_type: AnnotationType::Highlight,
            page: 1,
            bounds: (100.0, 100.0, 200.0, 50.0),
            content: String::new(),
            color: "#FFFF00".to_string(),
            opacity: 0.3,
            author: None,
            created_at: chrono::Utc::now(),
        };
        
        manager.add(annotation).await.unwrap();
        manager.remove("test-1").await.unwrap();
        assert_eq!(manager.count(), 0);
    }

    #[tokio::test]
    async fn test_get_for_page() {
        let mut manager = AnnotationManager::new();
        
        manager.add(PDFAnnotation {
            id: "test-1".to_string(),
            annotation_type: AnnotationType::Highlight,
            page: 1,
            bounds: (100.0, 100.0, 200.0, 50.0),
            content: String::new(),
            color: "#FFFF00".to_string(),
            opacity: 0.3,
            author: None,
            created_at: chrono::Utc::now(),
        }).await.unwrap();
        
        let annotations = manager.get_for_page(1).await.unwrap();
        assert_eq!(annotations.len(), 1);
        
        let annotations = manager.get_for_page(2).await.unwrap();
        assert_eq!(annotations.len(), 0);
    }

    #[test]
    fn test_create_highlight() {
        let annotation = AnnotationManager::create_highlight(
            1, (100.0, 100.0, 200.0, 50.0), "#FFFF00".to_string()
        );
        assert_eq!(annotation.annotation_type, AnnotationType::Highlight);
    }

    #[test]
    fn test_create_sticky_note() {
        let annotation = AnnotationManager::create_sticky_note(
            1, 100.0, 100.0, "Note text".to_string(), "#FFFF00".to_string()
        );
        assert_eq!(annotation.annotation_type, AnnotationType::StickyNote);
        assert_eq!(annotation.content, "Note text");
    }

    #[test]
    fn test_create_stamp() {
        let annotation = AnnotationManager::create_stamp(1, 100.0, 100.0, StampType::Approved);
        assert_eq!(annotation.content, "APPROVED");
        assert_eq!(annotation.color, "#00FF00");
    }

    #[tokio::test]
    async fn test_update_annotation() {
        let mut manager = AnnotationManager::new();
        let annotation = PDFAnnotation {
            id: "test-1".to_string(),
            annotation_type: AnnotationType::Highlight,
            page: 1,
            bounds: (100.0, 100.0, 200.0, 50.0),
            content: String::new(),
            color: "#FFFF00".to_string(),
            opacity: 0.3,
            author: None,
            created_at: chrono::Utc::now(),
        };
        
        manager.add(annotation).await.unwrap();
        
        let updates = AnnotationUpdate {
            color: Some("#FF0000".to_string()),
            opacity: Some(0.5),
            content: None,
        };
        
        manager.update("test-1", updates).await.unwrap();
        
        let annotations = manager.get_for_page(1).await.unwrap();
        assert_eq!(annotations[0].color, "#FF0000");
        assert_eq!(annotations[0].opacity, 0.5);
    }
}