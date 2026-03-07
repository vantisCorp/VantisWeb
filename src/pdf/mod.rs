//! PDF Module
//!
//! Modern PDF viewer with editing capabilities, annotations, and form filling.

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

pub mod viewer;
pub mod editor;
pub mod annotation;
pub mod form;
pub mod export;

use viewer::PDFViewer;
use editor::PDFEditor;
use annotation::AnnotationManager;
use form::FormManager;
use export::ExportManager;

/// PDF page information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PDFPage {
    pub number: u32,
    pub width: f32,
    pub height: f32,
    pub rotation: i16,
    pub has_text: bool,
    pub has_images: bool,
}

/// PDF document information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PDFDocumentInfo {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub keywords: Option<String>,
    pub creator: Option<String>,
    pub producer: Option<String>,
    pub creation_date: Option<String>,
    pub modification_date: Option<String>,
    pub page_count: u32,
    pub is_encrypted: bool,
    pub is_linearized: bool,
    pub version: String,
}

/// PDF viewer settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PDFViewerSettings {
    pub zoom_level: f32,
    pub display_mode: DisplayMode,
    pub page_layout: PageLayout,
    pub show_thumbnails: bool,
    pub show_bookmarks: bool,
    pub dark_mode: bool,
    pub page_spacing: u32,
}

impl Default for PDFViewerSettings {
    fn default() -> Self {
        Self {
            zoom_level: 1.0,
            display_mode: DisplayMode::SinglePage,
            page_layout: PageLayout::FitWidth,
            show_thumbnails: true,
            show_bookmarks: true,
            dark_mode: false,
            page_spacing: 10,
        }
    }
}

/// Display modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisplayMode {
    SinglePage,
    TwoPages,
    TwoPagesCover,
    ContinuousScroll,
    ContinuousTwoPages,
}

/// Page layout options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PageLayout {
    ActualSize,
    FitPage,
    FitWidth,
    FitHeight,
    FitVisible,
}

/// PDF manager
pub struct PDFManager {
    viewer: Arc<PDFViewer>,
    editor: Arc<PDFEditor>,
    annotation: Arc<AnnotationManager>,
    form: Arc<FormManager>,
    export: Arc<ExportManager>,
    current_document: Arc<RwLock<Option<PDFDocument>>>,
    settings: Arc<RwLock<PDFViewerSettings>>,
}

/// PDF document
#[derive(Debug, Clone)]
pub struct PDFDocument {
    pub path: PathBuf,
    pub info: PDFDocumentInfo,
    pub pages: Vec<PDFPage>,
    pub bookmarks: Vec<Bookmark>,
    pub is_modified: bool,
}

/// Bookmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub title: String,
    pub page: u32,
    pub level: u32,
    pub children: Vec<Bookmark>,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub page: u32,
    pub text: String,
    pub bounds: (f32, f32, f32, f32), // x, y, width, height
}

impl PDFManager {
    /// Create a new PDF manager
    pub fn new() -> Self {
        Self {
            viewer: Arc::new(PDFViewer::new()),
            editor: Arc::new(PDFEditor::new()),
            annotation: Arc::new(AnnotationManager::new()),
            form: Arc::new(FormManager::new()),
            export: Arc::new(ExportManager::new()),
            current_document: Arc::new(RwLock::new(None)),
            settings: Arc::new(RwLock::new(PDFViewerSettings::default())),
        }
    }

    /// Open a PDF document
    pub async fn open(&self, path: &PathBuf) -> Result<PDFDocument, PDFError> {
        let document = self.viewer.open(path).await?;
        
        let mut current = self.current_document.write().await;
        *current = Some(document.clone());
        
        Ok(document)
    }

    /// Close current document
    pub async fn close(&self) -> Result<(), PDFError> {
        let mut current = self.current_document.write().await;
        *current = None;
        Ok(())
    }

    /// Get current document
    pub async fn current_document(&self) -> Option<PDFDocument> {
        self.current_document.read().await.clone()
    }

    /// Render page to image
    pub async fn render_page(&self, page_number: u32, scale: f32) -> Result<Vec<u8>, PDFError> {
        let document = self.current_document.read().await;
        if document.is_none() {
            return Err(PDFError::NoDocumentOpen);
        }
        
        self.viewer.render_page(page_number, scale).await
    }

    /// Extract text from page
    pub async fn extract_text(&self, page_number: u32) -> Result<String, PDFError> {
        self.viewer.extract_text(page_number).await
    }

    /// Search text in document
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>, PDFError> {
        self.viewer.search(query).await
    }

    /// Get form fields
    pub async fn get_form_fields(&self) -> Result<Vec<FormField>, PDFError> {
        self.form.get_fields().await
    }

    /// Fill form field
    pub async fn fill_field(&self, field_id: &str, value: &str) -> Result<(), PDFError> {
        self.form.fill_field(field_id, value).await
    }

    /// Add annotation
    pub async fn add_annotation(&self, annotation: PDFAnnotation) -> Result<(), PDFError> {
        self.annotation.add(annotation).await
    }

    /// Remove annotation
    pub async fn remove_annotation(&self, annotation_id: &str) -> Result<(), PDFError> {
        self.annotation.remove(annotation_id).await
    }

    /// Get annotations for page
    pub async fn get_annotations(&self, page: u32) -> Result<Vec<PDFAnnotation>, PDFError> {
        self.annotation.get_for_page(page).await
    }

    /// Edit page
    pub async fn edit_page(&self, page: u32, edit: PageEdit) -> Result<(), PDFError> {
        self.editor.apply_edit(page, edit).await
    }

    /// Merge PDFs
    pub async fn merge(&self, documents: &[PathBuf], output: &PathBuf) -> Result<(), PDFError> {
        self.editor.merge(documents, output).await
    }

    /// Split PDF
    pub async fn split(&self, pages: &[u32], output: &PathBuf) -> Result<(), PDFError> {
        self.editor.split(pages, output).await
    }

    /// Export to format
    pub async fn export(&self, format: ExportFormat, output: &PathBuf) -> Result<(), PDFError> {
        let document = self.current_document.read().await;
        if document.is_none() {
            return Err(PDFError::NoDocumentOpen);
        }
        
        self.export.export(document.as_ref().unwrap(), format, output).await
    }

    /// Save document
    pub async fn save(&self, path: Option<&PathBuf>) -> Result<(), PDFError> {
        let document = self.current_document.read().await;
        if document.is_none() {
            return Err(PDFError::NoDocumentOpen);
        }
        
        self.editor.save(document.as_ref().unwrap(), path).await
    }

    /// Update viewer settings
    pub async fn update_settings(&self, settings: PDFViewerSettings) {
        let mut current = self.settings.write().await;
        *current = settings;
    }

    /// Get viewer settings
    pub async fn settings(&self) -> PDFViewerSettings {
        self.settings.read().await.clone()
    }

    /// Get page thumbnail
    pub async fn get_thumbnail(&self, page: u32) -> Result<Vec<u8>, PDFError> {
        self.viewer.get_thumbnail(page).await
    }

    /// Print document
    pub async fn print(&self, pages: Option<Vec<u32>>) -> Result<(), PDFError> {
        // In a real implementation, this would send to printer
        log::info!("Printing pages: {:?}", pages);
        Ok(())
    }

    /// Check if document is modified
    pub async fn is_modified(&self) -> bool {
        self.current_document.read().await
            .as_ref()
            .map(|d| d.is_modified)
            .unwrap_or(false)
    }
}

impl Default for PDFManager {
    fn default() -> Self {
        Self::new()
    }
}

/// PDF annotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PDFAnnotation {
    pub id: String,
    pub annotation_type: AnnotationType,
    pub page: u32,
    pub bounds: (f32, f32, f32, f32),
    pub content: String,
    pub color: String,
    pub opacity: f32,
    pub author: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Annotation types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnnotationType {
    Highlight,
    Underline,
    Strikethrough,
    FreeText,
    FreeHand,
    StickyNote,
    Stamp,
    Signature,
    Line,
    Rectangle,
    Circle,
}

/// Form field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormField {
    pub id: String,
    pub field_type: FormFieldType,
    pub name: String,
    pub value: String,
    pub is_required: bool,
    pub is_readonly: bool,
    pub bounds: (f32, f32, f32, f32),
    pub page: u32,
}

/// Form field types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormFieldType {
    Text,
    Checkbox,
    RadioButton,
    Dropdown,
    ListBox,
    Signature,
    Date,
    Email,
    Number,
}

/// Page edit operation
#[derive(Debug, Clone)]
pub enum PageEdit {
    AddText { x: f32, y: f32, text: String, font_size: f32, color: String },
    AddImage { x: f32, y: f32, image_path: PathBuf, width: f32, height: f32 },
    DeleteContent { bounds: (f32, f32, f32, f32) },
    Rotate { degrees: i16 },
    Crop { bounds: (f32, f32, f32, f32) },
}

/// Export formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    PDF,
    PNG,
    JPEG,
    TIFF,
    Word,
    Excel,
    HTML,
    PlainText,
}

/// PDF errors
#[derive(Debug, thiserror::Error)]
pub enum PDFError {
    #[error("Failed to open PDF: {0}")]
    OpenFailed(String),

    #[error("Failed to render page: {0}")]
    RenderFailed(String),

    #[error("Failed to save PDF: {0}")]
    SaveFailed(String),

    #[error("No document open")]
    NoDocumentOpen,

    #[error("Page not found: {0}")]
    PageNotFound(u32),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Form error: {0}")]
    FormError(String),

    #[error("Export error: {0}")]
    ExportError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pdf_manager_creation() {
        let manager = PDFManager::new();
        assert!(manager.current_document().await.is_none());
    }

    #[test]
    fn test_viewer_settings_default() {
        let settings = PDFViewerSettings::default();
        assert_eq!(settings.zoom_level, 1.0);
        assert!(settings.show_thumbnails);
    }

    #[test]
    fn test_display_modes() {
        assert_eq!(DisplayMode::SinglePage, DisplayMode::SinglePage);
        assert_ne!(DisplayMode::SinglePage, DisplayMode::TwoPages);
    }
}