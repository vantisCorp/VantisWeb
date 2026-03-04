use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::community_templates::{
    CommunityTemplate, CreatorInfo, ModerationReport, ModerationStatus,
    TemplateApiResponse, TemplateFilter, TemplateListResponse, TemplateMetadata,
    TemplateReview, TemplateSortOrder, TemplateStatistics, TemplateSubmission,
};
use super::profile_config::ProfileConfig;

/// Template manager for handling community templates
pub struct TemplateManager {
    /// Cache of downloaded templates
    templates: Arc<RwLock<HashMap<String, CommunityTemplate>>>,
    /// Cache of template metadata
    metadata_cache: Arc<RwLock<HashMap<String, TemplateMetadata>>>,
    /// Local templates directory
    templates_dir: PathBuf,
    /// API endpoint base URL
    api_endpoint: String,
    /// Current user ID (for submissions)
    user_id: Option<String>,
}

/// Error type for template operations
#[derive(Debug)]
pub enum TemplateError {
    NetworkError(String),
    FileNotFoundError(String),
    ParseError(String),
    ValidationError(String),
    PermissionError(String),
    ModerationError(String),
    StorageError(String),
}

impl std::fmt::Display for TemplateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
            Self::FileNotFoundError(msg) => write!(f, "File not found: {}", msg),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            Self::PermissionError(msg) => write!(f, "Permission error: {}", msg),
            Self::ModerationError(msg) => write!(f, "Moderation error: {}", msg),
            Self::StorageError(msg) => write!(f, "Storage error: {}", msg),
        }
    }
}

impl std::error::Error for TemplateError {}

impl TemplateManager {
    /// Create a new TemplateManager
    pub fn new(templates_dir: PathBuf, api_endpoint: String) -> Self {
        Self {
            templates: Arc::new(RwLock::new(HashMap::new())),
            metadata_cache: Arc::new(RwLock::new(HashMap::new())),
            templates_dir,
            api_endpoint,
            user_id: None,
        }
    }

    /// Set the current user
    pub fn set_user(&mut self, user_id: String) {
        self.user_id = Some(user_id);
    }

    /// Get all template categories
    pub fn get_categories() -> Vec<String> {
        vec![
            "Productivity".to_string(),
            "Development".to_string(),
            "Design".to_string(),
            "Social Media".to_string(),
            "Entertainment".to_string(),
            "Shopping".to_string(),
            "News".to_string(),
            "Education".to_string(),
            "Finance".to_string(),
            "Gaming".to_string(),
            "Privacy & Security".to_string(),
            "Accessibility".to_string(),
            "Minimal".to_string(),
            "Power User".to_string(),
            "Work".to_string(),
            "Personal".to_string(),
        ]
    }

    /// Browse templates from community marketplace
    pub async fn browse_templates(&self, filter: &TemplateFilter, page: u32, page_size: u32) 
        -> Result<TemplateListResponse, TemplateError> 
    {
        let mut templates: Vec<TemplateMetadata> = self.fetch_templates_from_api(filter, page, page_size)
            .await?;
        
        // Apply filters
        if let Some(query) = &filter.search_query {
            let query_lower = query.to_lowercase();
            templates.retain(|t| {
                t.name.to_lowercase().contains(&query_lower) ||
                t.description.to_lowercase().contains(&query_lower) ||
                t.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
            });
        }

        if let Some(tags) = &filter.tags {
            templates.retain(|t| {
                tags.iter().any(|tag| t.tags.contains(tag))
            });
        }

        if let Some(min_rating) = filter.min_rating {
            templates.retain(|t| t.rating >= min_rating);
        }

        if let Some(min_downloads) = filter.min_downloads {
            templates.retain(|t| t.downloads >= min_downloads);
        }

        // Apply sorting
        match filter.sort_by {
            TemplateSortOrder::Popularity => {
                templates.sort_by(|a, b| {
                    let score_a = a.rating * a.downloads as f64;
                    let score_b = b.rating * b.downloads as f64;
                    score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            TemplateSortOrder::Rating => {
                templates.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap_or(std::cmp::Ordering::Equal));
            }
            TemplateSortOrder::Newest => {
                templates.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            }
            TemplateSortOrder::Downloads => {
                templates.sort_by(|a, b| b.downloads.cmp(&a.downloads));
            }
            TemplateSortOrder::Name => {
                templates.sort_by(|a, b| a.name.cmp(&b.name));
            }
        }

        let total = templates.len() as u32;
        let start = ((page - 1) * page_size) as usize;
        let end = std::cmp::min(start + page_size as usize, templates.len());
        
        let paginated: Vec<TemplateMetadata> = templates.into_iter().skip(start).take(end - start).collect();

        Ok(TemplateListResponse {
            templates: paginated,
            total,
            page,
            page_size,
        })
    }

    /// Get a specific template by ID
    pub async fn get_template(&self, id: &str) -> Result<CommunityTemplate, TemplateError> {
        // Check local cache first
        if let Ok(templates) = self.templates.read() {
            if let Some(template) = templates.get(id) {
                return Ok(template.clone());
            }
        }

        // Fetch from API
        let template = self.fetch_template_from_api(id).await?;
        
        // Cache it
        if let Ok(mut templates) = self.templates.write() {
            templates.insert(id.to_string(), template.clone());
        }

        Ok(template)
    }

    /// Download a template to local storage
    pub async fn download_template(&self, template_id: &str) -> Result<PathBuf, TemplateError> {
        let template = self.get_template(template_id).await?;
        
        // Create templates directory if it doesn't exist
        std::fs::create_dir_all(&self.templates_dir)
            .map_err(|e| TemplateError::StorageError(e.to_string()))?;

        // Save template
        let template_path = self.templates_dir.join(format!("{}.json", template_id));
        let content = serde_json::to_string_pretty(&template)
            .map_err(|e| TemplateError::ParseError(e.to_string()))?;
        
        std::fs::write(&template_path, content)
            .map_err(|e| TemplateError::StorageError(e.to_string()))?;

        // Update download count on server
        self.increment_download_count(template_id).await?;

        Ok(template_path)
    }

    /// Install a template (convert to profile)
    pub async fn install_template(&self, template_id: &str, profile_name: Option<String>) 
        -> Result<ProfileConfig, TemplateError> 
    {
        let template = self.get_template(template_id).await?;
        
        // Create profile from template
        let mut profile = template.profile.clone();
        profile.id = Uuid::new_v4().to_string();
        profile.name = profile_name.unwrap_or_else(|| template.metadata.name.clone());
        profile.created_at = Utc::now();
        profile.updated_at = Utc::now();

        Ok(profile)
    }

    /// Preview a template without downloading
    pub async fn preview_template(&self, template_id: &str) -> Result<CommunityTemplate, TemplateError> {
        self.get_template(template_id).await
    }

    /// Submit a template to the community marketplace
    pub async fn submit_template(&self, submission: TemplateSubmission) 
        -> Result<String, TemplateError> 
    {
        // Validate submission
        self.validate_submission(&submission)?;

        // Submit to API
        let result = self.submit_to_api(&submission).await?;
        
        Ok(result)
    }

    /// Rate a template
    pub async fn rate_template(&self, template_id: &str, rating: u8, comment: Option<String>) 
        -> Result<(), TemplateError> 
    {
        if rating < 1 || rating > 5 {
            return Err(TemplateError::ValidationError("Rating must be between 1 and 5".to_string()));
        }

        let review = TemplateReview {
            id: Uuid::new_v4().to_string(),
            template_id: template_id.to_string(),
            reviewer: self.user_id.clone().unwrap_or_default(),
            rating,
            comment,
            created_at: Utc::now(),
        };

        self.submit_review(&review).await
    }

    /// Get reviews for a template
    pub async fn get_reviews(&self, template_id: &str, page: u32, page_size: u32) 
        -> Result<Vec<TemplateReview>, TemplateError> 
    {
        self.fetch_reviews_from_api(template_id, page, page_size).await
    }

    /// Report a template
    pub async fn report_template(&self, template_id: &str, reason: &str, details: Option<String>) 
        -> Result<(), TemplateError> 
    {
        let report = ModerationReport {
            id: Uuid::new_v4().to_string(),
            template_id: template_id.to_string(),
            reporter: self.user_id.clone().unwrap_or_default(),
            reason: reason.to_string(),
            details,
            status: ModerationStatus::Pending,
            moderator_notes: None,
            created_at: Utc::now(),
        };

        self.submit_report(&report).await
    }

    /// Get template statistics
    pub async fn get_statistics(&self) -> Result<TemplateStatistics, TemplateError> {
        self.fetch_statistics_from_api().await
    }

    /// Get featured templates
    pub async fn get_featured_templates(&self, count: u32) -> Result<Vec<TemplateMetadata>, TemplateError> {
        let response = self.browse_templates(&TemplateFilter {
            sort_by: TemplateSortOrder::Popularity,
            min_rating: Some(4.0),
            min_downloads: Some(100),
            ..Default::default()
        }, 1, count).await?;

        Ok(response.templates)
    }

    /// Get templates by category
    pub async fn get_templates_by_category(&self, category: &str, page: u32, page_size: u32) 
        -> Result<TemplateListResponse, TemplateError> 
    {
        self.browse_templates(&TemplateFilter {
            tags: Some(vec![category.to_string()]),
            sort_by: TemplateSortOrder::Popularity,
            ..Default::default()
        }, page, page_size).await
    }

    /// Get templates by creator
    pub async fn get_templates_by_creator(&self, creator_id: &str, page: u32, page_size: u32) 
        -> Result<TemplateListResponse, TemplateError> 
    {
        self.fetch_templates_by_creator(creator_id, page, page_size).await
    }

    /// Search templates
    pub async fn search_templates(&self, query: &str, page: u32, page_size: u32) 
        -> Result<TemplateListResponse, TemplateError> 
    {
        self.browse_templates(&TemplateFilter {
            search_query: Some(query.to_string()),
            sort_by: TemplateSortOrder::Popularity,
            ..Default::default()
        }, page, page_size).await
    }

    // Private helper methods

    fn validate_submission(&self, submission: &TemplateSubmission) -> Result<(), TemplateError> {
        if submission.metadata.name.is_empty() {
            return Err(TemplateError::ValidationError("Template name is required".to_string()));
        }
        if submission.metadata.name.len() > 100 {
            return Err(TemplateError::ValidationError("Template name must be 100 characters or less".to_string()));
        }
        if submission.metadata.description.is_empty() {
            return Err(TemplateError::ValidationError("Template description is required".to_string()));
        }
        if submission.metadata.description.len() > 500 {
            return Err(TemplateError::ValidationError("Description must be 500 characters or less".to_string()));
        }
        if submission.metadata.tags.is_empty() {
            return Err(TemplateError::ValidationError("At least one category tag is required".to_string()));
        }
        if submission.metadata.tags.len() > 5 {
            return Err(TemplateError::ValidationError("Maximum 5 tags allowed".to_string()));
        }
        if submission.metadata.screenshots.len() > 5 {
            return Err(TemplateError::ValidationError("Maximum 5 screenshots allowed".to_string()));
        }
        Ok(())
    }

    async fn fetch_templates_from_api(&self, filter: &TemplateFilter, page: u32, page_size: u32) 
        -> Result<Vec<TemplateMetadata>, TemplateError> 
    {
        // In a real implementation, this would make an HTTP request
        // For now, return sample data
        Ok(self.get_sample_templates())
    }

    async fn fetch_template_from_api(&self, id: &str) -> Result<CommunityTemplate, TemplateError> {
        // In a real implementation, this would make an HTTP request
        // For now, return a sample template
        let samples = self.get_sample_templates_with_profiles();
        samples.into_iter()
            .find(|t| t.metadata.id == id)
            .ok_or_else(|| TemplateError::FileNotFoundError(format!("Template {} not found", id)))
    }

    async fn increment_download_count(&self, template_id: &str) -> Result<(), TemplateError> {
        // In a real implementation, this would make an HTTP request
        Ok(())
    }

    async fn submit_to_api(&self, submission: &TemplateSubmission) -> Result<String, TemplateError> {
        // In a real implementation, this would make an HTTP request
        Ok(Uuid::new_v4().to_string())
    }

    async fn submit_review(&self, review: &TemplateReview) -> Result<(), TemplateError> {
        // In a real implementation, this would make an HTTP request
        Ok(())
    }

    async fn fetch_reviews_from_api(&self, template_id: &str, page: u32, page_size: u32) 
        -> Result<Vec<TemplateReview>, TemplateError> 
    {
        // In a real implementation, this would make an HTTP request
        Ok(vec![])
    }

    async fn submit_report(&self, report: &ModerationReport) -> Result<(), TemplateError> {
        // In a real implementation, this would make an HTTP request
        Ok(())
    }

    async fn fetch_statistics_from_api(&self) -> Result<TemplateStatistics, TemplateError> {
        // In a real implementation, this would make an HTTP request
        Ok(TemplateStatistics {
            total_templates: 150,
            total_downloads: 50000,
            total_reviews: 2500,
            top_categories: vec![
                ("Productivity".to_string(), 45),
                ("Development".to_string(), 38),
                ("Design".to_string(), 25),
                ("Privacy & Security".to_string(), 20),
            ],
            active_creators: 75,
        })
    }

    async fn fetch_templates_by_creator(&self, creator_id: &str, page: u32, page_size: u32) 
        -> Result<TemplateListResponse, TemplateError> 
    {
        // In a real implementation, this would make an HTTP request
        Ok(TemplateListResponse {
            templates: vec![],
            total: 0,
            page,
            page_size,
        })
    }

    fn get_sample_templates(&self) -> Vec<TemplateMetadata> {
        vec![
            TemplateMetadata {
                id: "dev-pro-2024".to_string(),
                name: "Developer Pro".to_string(),
                description: "Optimized profile for web developers with dev tools, documentation sites, and productivity extensions.".to_string(),
                icon: "👨‍💻".to_string(),
                tags: vec!["Development".to_string(), "Productivity".to_string()],
                creator: CreatorInfo {
                    username: "devmaster".to_string(),
                    display_name: "Dev Master".to_string(),
                    avatar: None,
                    id: "creator-1".to_string(),
                },
                screenshots: vec![],
                rating: 4.8,
                review_count: 156,
                downloads: 5420,
                created_at: Utc::now() - chrono::Duration::days(90),
                updated_at: Utc::now() - chrono::Duration::days(5),
                version: "1.2.0".to_string(),
                min_version: "1.0.0".to_string(),
            },
            TemplateMetadata {
                id: "minimal-focus".to_string(),
                name: "Minimal Focus".to_string(),
                description: "Distraction-free browsing with minimal UI, focus mode, and essential tools only.".to_string(),
                icon: "🎯".to_string(),
                tags: vec!["Minimal".to_string(), "Productivity".to_string()],
                creator: CreatorInfo {
                    username: "minimalist".to_string(),
                    display_name: "Minimalist".to_string(),
                    avatar: None,
                    id: "creator-2".to_string(),
                },
                screenshots: vec![],
                rating: 4.6,
                review_count: 89,
                downloads: 3200,
                created_at: Utc::now() - chrono::Duration::days(60),
                updated_at: Utc::now() - chrono::Duration::days(10),
                version: "1.0.0".to_string(),
                min_version: "1.0.0".to_string(),
            },
            TemplateMetadata {
                id: "privacy-shield".to_string(),
                name: "Privacy Shield".to_string(),
                description: "Maximum privacy protection with tracker blocking, encrypted connections, and privacy-focused search.".to_string(),
                icon: "🛡️".to_string(),
                tags: vec!["Privacy & Security".to_string()],
                creator: CreatorInfo {
                    username: "privacypro".to_string(),
                    display_name: "Privacy Pro".to_string(),
                    avatar: None,
                    id: "creator-3".to_string(),
                },
                screenshots: vec![],
                rating: 4.9,
                review_count: 234,
                downloads: 8900,
                created_at: Utc::now() - chrono::Duration::days(120),
                updated_at: Utc::now() - chrono::Duration::days(2),
                version: "2.0.0".to_string(),
                min_version: "1.1.0".to_string(),
            },
        ]
    }

    fn get_sample_templates_with_profiles(&self) -> Vec<CommunityTemplate> {
        // Returns sample templates with actual profile configurations
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_template_manager_creation() {
        let dir = tempdir().unwrap();
        let manager = TemplateManager::new(
            dir.path().to_path_buf(),
            "https://api.vantisweb.com".to_string()
        );
        assert!(manager.templates.read().is_ok());
    }

    #[test]
    fn test_get_categories() {
        let categories = TemplateManager::get_categories();
        assert!(!categories.is_empty());
        assert!(categories.contains(&"Development".to_string()));
    }

    #[test]
    fn test_template_filter_default() {
        let filter = TemplateFilter::default();
        assert!(filter.search_query.is_none());
        assert!(filter.tags.is_none());
        assert!(filter.min_rating.is_none());
    }

    #[test]
    fn test_validate_submission() {
        let dir = tempdir().unwrap();
        let manager = TemplateManager::new(
            dir.path().to_path_buf(),
            "https://api.vantisweb.com".to_string()
        );

        let valid_submission = TemplateSubmission {
            metadata: TemplateMetadata {
                id: "test".to_string(),
                name: "Test Template".to_string(),
                description: "A test template for validation".to_string(),
                icon: "🔧".to_string(),
                tags: vec!["Test".to_string()],
                creator: CreatorInfo {
                    username: "test".to_string(),
                    display_name: "Test".to_string(),
                    avatar: None,
                    id: "test-id".to_string(),
                },
                screenshots: vec![],
                rating: 0.0,
                review_count: 0,
                downloads: 0,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                version: "1.0.0".to_string(),
                min_version: "1.0.0".to_string(),
            },
            profile: ProfileConfig::default(),
            custom_css: None,
            custom_js: None,
            resources: HashMap::new(),
        };

        assert!(manager.validate_submission(&valid_submission).is_ok());
    }
}