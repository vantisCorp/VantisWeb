//! Annotation Tools
//!
//! Drawing, text, blur, highlight tools for screenshots and recordings.

use std::path::PathBuf;
use crate::capture::{Annotation, CaptureError};

/// Annotation manager
pub struct AnnotationManager {
    current_annotations: Vec<Annotation>,
}

impl AnnotationManager {
    /// Create a new annotation manager
    pub fn new() -> Self {
        Self {
            current_annotations: vec![],
        }
    }

    /// Apply annotations to an image
    pub async fn apply(&self, image_path: &PathBuf, annotations: Vec<Annotation>) -> Result<PathBuf, CaptureError> {
        // In a real implementation, this would:
        // 1. Load the image
        // 2. Apply each annotation in order
        // 3. Handle different annotation types
        // 4. Save the annotated image

        let output_path = self.get_annotated_path(image_path);

        for annotation in &annotations {
            match annotation {
                Annotation::Pen { points, color, width } => {
                    log::debug!("Drawing pen: {} points, color: {}, width: {}", points.len(), color, width);
                }
                Annotation::Arrow { start, end, color, width } => {
                    log::debug!("Drawing arrow from {:?} to {:?}, color: {}, width: {}", start, end, color, width);
                }
                Annotation::Rectangle { x, y, width, height, color, filled } => {
                    log::debug!("Drawing rectangle at ({}, {}) size {}x{}, color: {}, filled: {}", 
                        x, y, width, height, color, filled);
                }
                Annotation::Circle { x, y, radius, color, filled } => {
                    log::debug!("Drawing circle at ({}, {}) radius {}, color: {}, filled: {}", 
                        x, y, radius, color, filled);
                }
                Annotation::Text { x, y, text, font_size, color } => {
                    log::debug!("Drawing text '{}' at ({}, {}) size {}, color: {}", 
                        text, x, y, font_size, color);
                }
                Annotation::Blur { x, y, width, height, intensity } => {
                    log::debug!("Blurring area at ({}, {}) size {}x{}, intensity: {}", 
                        x, y, width, height, intensity);
                }
                Annotation::Highlight { x, y, width, height, color } => {
                    log::debug!("Highlighting area at ({}, {}) size {}x{}, color: {}", 
                        x, y, width, height, color);
                }
            }
        }

        Ok(output_path)
    }

    /// Add pen annotation
    pub fn add_pen(&mut self, points: Vec<(f32, f32)>, color: String, width: f32) {
        self.current_annotations.push(Annotation::Pen { points, color, width });
    }

    /// Add arrow annotation
    pub fn add_arrow(&mut self, start: (f32, f32), end: (f32, f32), color: String, width: f32) {
        self.current_annotations.push(Annotation::Arrow { start, end, color, width });
    }

    /// Add rectangle annotation
    pub fn add_rectangle(&mut self, x: f32, y: f32, width: f32, height: f32, color: String, filled: bool) {
        self.current_annotations.push(Annotation::Rectangle { x, y, width, height, color, filled });
    }

    /// Add circle annotation
    pub fn add_circle(&mut self, x: f32, y: f32, radius: f32, color: String, filled: bool) {
        self.current_annotations.push(Annotation::Circle { x, y, radius, color, filled });
    }

    /// Add text annotation
    pub fn add_text(&mut self, x: f32, y: f32, text: String, font_size: f32, color: String) {
        self.current_annotations.push(Annotation::Text { x, y, text, font_size, color });
    }

    /// Add blur annotation
    pub fn add_blur(&mut self, x: f32, y: f32, width: f32, height: f32, intensity: f32) {
        self.current_annotations.push(Annotation::Blur { x, y, width, height, intensity });
    }

    /// Add highlight annotation
    pub fn add_highlight(&mut self, x: f32, y: f32, width: f32, height: f32, color: String) {
        self.current_annotations.push(Annotation::Highlight { x, y, width, height, color });
    }

    /// Clear all annotations
    pub fn clear(&mut self) {
        self.current_annotations.clear();
    }

    /// Undo last annotation
    pub fn undo(&mut self) {
        self.current_annotations.pop();
    }

    /// Get current annotations
    pub fn annotations(&self) -> &[Annotation] {
        &self.current_annotations
    }

    /// Get annotated file path
    fn get_annotated_path(&self, original_path: &PathBuf) -> PathBuf {
        let mut path = original_path.clone();
        let stem = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
        let extension = path.extension().unwrap_or_default().to_string_lossy().to_string();
        path.set_file_name(format!("{}_annotated.{}", stem, extension));
        path
    }
}

/// Annotation tool configuration
#[derive(Debug, Clone)]
pub struct AnnotationTool {
    pub tool_type: AnnotationToolType,
    pub color: String,
    pub width: f32,
    pub opacity: f32,
}

/// Annotation tool types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnotationToolType {
    Pen,
    Arrow,
    Rectangle,
    Circle,
    Text,
    Blur,
    Highlight,
    Eraser,
}

impl Default for AnnotationTool {
    fn default() -> Self {
        Self {
            tool_type: AnnotationToolType::Pen,
            color: "#FF0000".to_string(),
            width: 2.0,
            opacity: 1.0,
        }
    }
}

/// Preset color palettes
pub struct ColorPalettes;

impl ColorPalettes {
    pub fn standard() -> Vec<String> {
        vec![
            "#FF0000".to_string(), // Red
            "#00FF00".to_string(), // Green
            "#0000FF".to_string(), // Blue
            "#FFFF00".to_string(), // Yellow
            "#FF00FF".to_string(), // Magenta
            "#00FFFF".to_string(), // Cyan
            "#000000".to_string(), // Black
            "#FFFFFF".to_string(), // White
        ]
    }

    pub fn highlighter() -> Vec<String> {
        vec![
            "#FFFF0080".to_string(), // Yellow transparent
            "#00FF0080".to_string(), // Green transparent
            "#00FFFF80".to_string(), // Cyan transparent
            "#FF00FF80".to_string(), // Magenta transparent
        ]
    }

    pub fn blue_light() -> Vec<String> {
        vec![
            "#3B82F6".to_string(),
            "#EF4444".to_string(),
            "#10B981".to_string(),
            "#F59E0B".to_string(),
            "#6366F1".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_annotation_manager_creation() {
        let manager = AnnotationManager::new();
        assert_eq!(manager.annotations().len(), 0);
    }

    #[test]
    fn test_add_pen() {
        let mut manager = AnnotationManager::new();
        manager.add_pen(vec![(0.0, 0.0), (100.0, 100.0)], "#FF0000".to_string(), 2.0);
        assert_eq!(manager.annotations().len(), 1);
    }

    #[test]
    fn test_add_multiple_annotations() {
        let mut manager = AnnotationManager::new();
        manager.add_pen(vec![(0.0, 0.0), (100.0, 100.0)], "#FF0000".to_string(), 2.0);
        manager.add_arrow((0.0, 0.0), (100.0, 100.0), "#00FF00".to_string(), 3.0);
        manager.add_text(10.0, 10.0, "Test".to_string(), 16.0, "#000000".to_string());
        assert_eq!(manager.annotations().len(), 3);
    }

    #[test]
    fn test_clear() {
        let mut manager = AnnotationManager::new();
        manager.add_pen(vec![(0.0, 0.0)], "#FF0000".to_string(), 2.0);
        manager.clear();
        assert_eq!(manager.annotations().len(), 0);
    }

    #[test]
    fn test_undo() {
        let mut manager = AnnotationManager::new();
        manager.add_pen(vec![(0.0, 0.0)], "#FF0000".to_string(), 2.0);
        manager.add_arrow((0.0, 0.0), (100.0, 100.0), "#00FF00".to_string(), 3.0);
        manager.undo();
        assert_eq!(manager.annotations().len(), 1);
    }

    #[tokio::test]
    async fn test_apply_annotations() {
        let manager = AnnotationManager::new();
        let path = PathBuf::from("/tmp/test.png");
        let annotations = vec![
            Annotation::Text {
                x: 10.0,
                y: 10.0,
                text: "Test".to_string(),
                font_size: 16.0,
                color: "#000000".to_string(),
            }
        ];
        
        let result = manager.apply(&path, annotations).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_color_palettes() {
        let standard = ColorPalettes::standard();
        assert_eq!(standard.len(), 8);
        
        let highlighter = ColorPalettes::highlighter();
        assert_eq!(highlighter.len(), 4);
    }

    #[test]
    fn test_annotation_tool_default() {
        let tool = AnnotationTool::default();
        assert_eq!(tool.tool_type, AnnotationToolType::Pen);
        assert_eq!(tool.color, "#FF0000");
        assert_eq!(tool.width, 2.0);
    }
}