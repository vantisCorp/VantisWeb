//! Extension Marketplace Module
//!
//! Provides a marketplace for discovering, installing, and managing
//! browser extensions. Supports both official and community extensions.

use anyhow::{Result, Error};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Extension marketplace for discovering and installing extensions
pub struct ExtensionMarketplace {
    config: MarketplaceConfig,
    cache: Arc<RwLock<MarketplaceCache>>,
    client: reqwest::Client,
}

/// Marketplace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceConfig {
    /// API endpoint for the marketplace
    pub api_url: String,
    /// Enable community extensions
    pub allow_community: bool,
    /// Enable extension ratings
    pub enable_ratings: bool,
    /// Cache duration in seconds
    pub cache_duration: u64,
    /// Auto-update extensions
    pub auto_update: bool,
}

impl Default for MarketplaceConfig {
    fn default() -> Self {
        Self {
            api_url: "https://marketplace.vantisweb.io/api/v1".to_string(),
            allow_community: true,
            enable_ratings: true,
            cache_duration: 3600, // 1 hour
            auto_update: true,
        }
    }
}

/// Extension listing in the marketplace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionListing {
    /// Unique extension ID
    pub id: String,
    /// Extension name
    pub name: String,
    /// Brief description
    pub description: String,
    /// Detailed description (markdown)
    pub long_description: Option<String>,
    /// Author/developer name
    pub author: String,
    /// Version string
    pub version: String,
    /// Download URL
    pub download_url: String,
    /// Icon URL
    pub icon_url: Option<String>,
    /// Screenshots URLs
    pub screenshots: Vec<String>,
    /// Categories
    pub categories: Vec<String>,
    /// Tags for search
    pub tags: Vec<String>,
    /// Average rating (0-5)
    pub rating: f32,
    /// Number of ratings
    pub rating_count: u32,
    /// Number of downloads
    pub download_count: u64,
    /// Last updated
    pub updated_at: DateTime<Utc>,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Homepage URL
    pub homepage: Option<String>,
    /// Support URL
    pub support_url: Option<String>,
    /// Required permissions
    pub permissions: Vec<String>,
    /// Minimum browser version
    pub min_browser_version: Option<String>,
    /// Is verified/official
    pub is_verified: bool,
    /// Is featured
    pub is_featured: bool,
}

/// Extension review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionReview {
    /// Review ID
    pub id: String,
    /// Extension ID
    pub extension_id: String,
    /// User ID
    pub user_id: String,
    /// Username
    pub username: String,
    /// Rating (1-5)
    pub rating: u8,
    /// Review title
    pub title: String,
    /// Review content
    pub content: String,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Helpful votes
    pub helpful_votes: u32,
}

/// Search filters for marketplace
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarketplaceSearchFilters {
    /// Search query
    pub query: Option<String>,
    /// Filter by category
    pub category: Option<String>,
    /// Filter by tags
    pub tags: Vec<String>,
    /// Minimum rating
    pub min_rating: Option<f32>,
    /// Only verified extensions
    pub verified_only: bool,
    /// Sort by
    pub sort_by: Option<SortBy>,
    /// Sort order
    pub sort_order: Option<SortOrder>,
}

/// Sort options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortBy {
    Relevance,
    Downloads,
    Rating,
    Updated,
    Name,
}

/// Sort order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Asc,
    Desc,
}

/// Marketplace cache
#[derive(Debug, Clone, Default)]
struct MarketplaceCache {
    extensions: HashMap<String, ExtensionListing>,
    categories: Vec<String>,
    featured: Vec<String>,
    last_updated: Option<DateTime<Utc>>,
}

/// Category with extension count
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryInfo {
    pub name: String,
    pub slug: String,
    pub extension_count: u32,
    pub icon: Option<String>,
}

impl ExtensionMarketplace {
    /// Create a new marketplace instance
    pub fn new(config: MarketplaceConfig) -> Self {
        Self {
            config,
            cache: Arc::new(RwLock::new(MarketplaceCache::default())),
            client: reqwest::Client::new(),
        }
    }

    /// Search for extensions
    pub async fn search(&self, filters: MarketplaceSearchFilters) -> Result<Vec<ExtensionListing>> {
        let url = format!("{}/extensions/search", self.config.api_url);
        
        let response = self.client
            .get(&url)
            .query(&filters)
            .send()
            .await?;

        if response.status().is_success() {
            let extensions: Vec<ExtensionListing> = response.json().await?;
            Ok(extensions)
        } else {
            Err(Error::msg(format!("Search failed: {}", response.status())))
        }
    }

    /// Get extension by ID
    pub async fn get_extension(&self, id: &str) -> Result<Option<ExtensionListing>> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(ext) = cache.extensions.get(id) {
                return Ok(Some(ext.clone()));
            }
        }

        let url = format!("{}/extensions/{}", self.config.api_url, id);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;

        if response.status().is_success() {
            let extension: ExtensionListing = response.json().await?;
            
            // Update cache
            let mut cache = self.cache.write().await;
            cache.extensions.insert(id.to_string(), extension.clone());
            
            Ok(Some(extension))
        } else if response.status().as_u16() == 404 {
            Ok(None)
        } else {
            Err(Error::msg(format!("Failed to get extension: {}", response.status())))
        }
    }

    /// Get featured extensions
    pub async fn get_featured(&self) -> Result<Vec<ExtensionListing>> {
        let url = format!("{}/extensions/featured", self.config.api_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;

        if response.status().is_success() {
            let extensions: Vec<ExtensionListing> = response.json().await?;
            Ok(extensions)
        } else {
            Err(Error::msg(format!("Failed to get featured: {}", response.status())))
        }
    }

    /// Get all categories
    pub async fn get_categories(&self) -> Result<Vec<CategoryInfo>> {
        let url = format!("{}/categories", self.config.api_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;

        if response.status().is_success() {
            let categories: Vec<CategoryInfo> = response.json().await?;
            Ok(categories)
        } else {
            Err(Error::msg(format!("Failed to get categories: {}", response.status())))
        }
    }

    /// Get extensions by category
    pub async fn get_by_category(&self, category: &str) -> Result<Vec<ExtensionListing>> {
        let url = format!("{}/categories/{}/extensions", self.config.api_url, category);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;

        if response.status().is_success() {
            let extensions: Vec<ExtensionListing> = response.json().await?;
            Ok(extensions)
        } else {
            Err(Error::msg(format!("Failed to get category extensions: {}", response.status())))
        }
    }

    /// Get reviews for an extension
    pub async fn get_reviews(&self, extension_id: &str) -> Result<Vec<ExtensionReview>> {
        let url = format!("{}/extensions/{}/reviews", self.config.api_url, extension_id);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;

        if response.status().is_success() {
            let reviews: Vec<ExtensionReview> = response.json().await?;
            Ok(reviews)
        } else {
            Err(Error::msg(format!("Failed to get reviews: {}", response.status())))
        }
    }

    /// Submit a review
    pub async fn submit_review(
        &self,
        extension_id: &str,
        rating: u8,
        title: &str,
        content: &str,
        token: &str,
    ) -> Result<ExtensionReview> {
        if rating < 1 || rating > 5 {
            return Err(Error::msg("Rating must be between 1 and 5"));
        }

        let url = format!("{}/extensions/{}/reviews", self.config.api_url, extension_id);
        
        #[derive(Serialize)]
        struct ReviewRequest {
            rating: u8,
            title: String,
            content: String,
        }

        let response = self.client
            .post(&url)
            .bearer_auth(token)
            .json(&ReviewRequest {
                rating,
                title: title.to_string(),
                content: content.to_string(),
            })
            .send()
            .await?;

        if response.status().is_success() {
            let review: ExtensionReview = response.json().await?;
            Ok(review)
        } else {
            Err(Error::msg(format!("Failed to submit review: {}", response.status())))
        }
    }

    /// Download an extension
    pub async fn download_extension(&self, extension_id: &str) -> Result<Vec<u8>> {
        let listing = self.get_extension(extension_id)
            .await?
            .ok_or_else(|| Error::msg("Extension not found"))?;

        let response = self.client
            .get(&listing.download_url)
            .send()
            .await?;

        if response.status().is_success() {
            let bytes = response.bytes().await?;
            Ok(bytes.to_vec())
        } else {
            Err(Error::msg(format!("Download failed: {}", response.status())))
        }
    }

    /// Check for updates for installed extensions
    pub async fn check_updates(&self, installed: &[String]) -> Result<HashMap<String, String>> {
        let url = format!("{}/extensions/check-updates", self.config.api_url);
        
        #[derive(Serialize)]
        struct CheckUpdatesRequest<'a> {
            extensions: &'a [String],
        }

        #[derive(Deserialize)]
        struct CheckUpdatesResponse {
            updates: HashMap<String, String>,
        }

        let response = self.client
            .post(&url)
            .json(&CheckUpdatesRequest { extensions: installed })
            .send()
            .await?;

        if response.status().is_success() {
            let result: CheckUpdatesResponse = response.json().await?;
            Ok(result.updates)
        } else {
            Err(Error::msg(format!("Update check failed: {}", response.status())))
        }
    }

    /// Clear cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        *cache = MarketplaceCache::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marketplace_config_default() {
        let config = MarketplaceConfig::default();
        assert!(!config.api_url.is_empty());
        assert!(config.allow_community);
        assert!(config.enable_ratings);
    }

    #[test]
    fn test_search_filters_default() {
        let filters = MarketplaceSearchFilters::default();
        assert!(filters.query.is_none());
        assert!(filters.category.is_none());
    }

    #[test]
    fn test_category_info() {
        let category = CategoryInfo {
            name: "Productivity".to_string(),
            slug: "productivity".to_string(),
            extension_count: 42,
            icon: Some("icon.png".to_string()),
        };
        assert_eq!(category.name, "Productivity");
        assert_eq!(category.extension_count, 42);
    }
}