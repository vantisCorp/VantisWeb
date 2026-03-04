use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use crate::profiles::profile_config::ProfileConfig;

/// Metadata for community profile templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    /// Unique template identifier
    pub id: String,
    /// Template name
    pub name: String,
    /// Template description
    pub description: String,
    /// Template icon (emoji or URL)
    pub icon: String,
    /// Category tags
    pub tags: Vec<String>,
    /// Creator information
    pub creator: CreatorInfo,
    /// Screenshots (URLs)
    pub screenshots: Vec<String>,
    /// Rating average (0-5)
    pub rating: f64,
    /// Total number of reviews
    pub review_count: u32,
    /// Download count
    pub downloads: u32,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Template version
    pub version: String,
    /// Compatibility version
    pub min_version: String,
}

/// Creator information for templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatorInfo {
    /// Creator username
    pub username: String,
    /// Creator display name
    pub display_name: String,
    /// Creator avatar URL
    pub avatar: Option<String>,
    /// Creator ID
    pub id: String,
}

/// A community profile template with actual profile data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityTemplate {
    /// Template metadata
    pub metadata: TemplateMetadata,
    /// Profile configuration
    pub profile: ProfileConfig,
    /// Custom CSS
    pub custom_css: Option<String>,
    /// Custom JavaScript
    pub custom_js: Option<String>,
    /// Additional resources (icons, themes, etc.)
    pub resources: HashMap<String, String>,
}

/// Template review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateReview {
    /// Review ID
    pub id: String,
    /// Template ID
    pub template_id: String,
    /// Reviewer username
    pub reviewer: String,
    /// Rating (1-5)
    pub rating: u8,
    /// Review text
    pub comment: Option<String>,
    /// Timestamp
    pub created_at: DateTime<Utc>,
}

/// Template filter options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateFilter {
    /// Search query
    pub search_query: Option<String>,
    /// Category tags
    pub tags: Option<Vec<String>>,
    /// Minimum rating
    pub min_rating: Option<f64>,
    /// Sort order
    pub sort_by: TemplateSortOrder,
    /// Minimum downloads
    pub min_downloads: Option<u32>,
}

/// Template sort order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemplateSortOrder {
    Popularity,
    Rating,
    Newest,
    Downloads,
    Name,
}

/// Template submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSubmission {
    /// Template metadata
    pub metadata: TemplateMetadata,
    /// Profile configuration
    pub profile: ProfileConfig,
    /// Custom CSS
    pub custom_css: Option<String>,
    /// Custom JavaScript
    pub custom_js: Option<String>,
    /// Additional resources
    pub resources: HashMap<String, String>,
}

/// Moderation status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModerationStatus {
    Pending,
    Approved,
    Rejected,
    Flagged,
}

/// Moderation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationReport {
    /// Report ID
    pub id: String,
    /// Template ID
    pub template_id: String,
    /// Reporter username
    pub reporter: String,
    /// Report reason
    pub reason: String,
    /// Additional details
    pub details: Option<String>,
    /// Moderation status
    pub status: ModerationStatus,
    /// Moderator notes
    pub moderator_notes: Option<String>,
    /// Timestamp
    pub created_at: DateTime<Utc>,
}

/// Community API client response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub message: Option<String>,
}

/// Template list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateListResponse {
    pub templates: Vec<TemplateMetadata>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
}

/// Template statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateStatistics {
    pub total_templates: u32,
    pub total_downloads: u32,
    pub total_reviews: u32,
    pub top_categories: Vec<(String, u32)>,
    pub active_creators: u32,
}

impl Default for TemplateFilter {
    fn default() -> Self {
        Self {
            search_query: None,
            tags: None,
            min_rating: None,
            sort_by: TemplateSortOrder::Popularity,
            min_downloads: None,
        }
    }
}

impl Default for ModerationStatus {
    fn default() -> Self {
        Self::Pending
    }
}