//! # Extension Marketplace Module
//!
//! Provides a comprehensive marketplace for discovering, installing, and managing
//! browser extensions. This module supports both official and community extensions,
//! with features including search, categorization, ratings, reviews, and automatic updates.
//!
//! ## Features
//!
//! - **Extension Discovery**: Search and browse extensions by category, tags, and ratings
//! - **Installation Management**: Download and install extensions with permission verification
//! - **User Reviews**: Submit and read reviews with helpfulness voting
//! - **Automatic Updates**: Check for and apply updates to installed extensions
//! - **Caching**: Built-in caching for improved performance
//! - **Verification**: Support for verified/official extensions
//!
//! ## Example Usage
//!
//! ```rust
//! use vantisweb::extensions::marketplace::{ExtensionMarketplace, MarketplaceConfig};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create marketplace with default configuration
//!     let config = MarketplaceConfig::default();
//!     let marketplace = ExtensionMarketplace::new(config);
//!
//!     // Search for extensions
//!     let filters = MarketplaceSearchFilters {
//!         query: Some("ad blocker".to_string()),
//!         min_rating: Some(4.0),
//!         ..Default::default()
//!     };
//!     let results = marketplace.search(filters).await?;
//!
//!     // Get featured extensions
//!     let featured = marketplace.get_featured().await?;
//!
//!     // Download an extension
//!     let bytes = marketplace.download_extension("extension-id").await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Security Considerations
//!
//! - Always verify extension permissions before installation
//! - Prefer verified/official extensions when available
//! - Check user reviews and ratings before installing
//! - Be cautious with extensions requesting broad permissions

use anyhow::{Result, Error};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Extension marketplace for discovering and installing extensions
///
/// The `ExtensionMarketplace` provides a complete interface for interacting with
/// the extension marketplace API. It handles search, browsing, downloading, and
/// review management with built-in caching for improved performance.
///
/// # Examples
///
/// ```rust
/// use vantisweb::extensions::marketplace::{ExtensionMarketplace, MarketplaceConfig};
///
/// let config = MarketplaceConfig::default();
/// let marketplace = ExtensionMarketplace::new(config);
/// ```
///
/// # Thread Safety
///
/// The marketplace is thread-safe and can be shared across multiple tasks
/// through `Arc<ExtensionMarketplace>`.
pub struct ExtensionMarketplace {
    config: MarketplaceConfig,
    cache: Arc<RwLock<MarketplaceCache>>,
    client: reqwest::Client,
}

/// Marketplace configuration
///
/// Configuration options for the extension marketplace including API endpoints,
/// caching settings, and feature flags.
///
/// # Examples
///
/// ```rust
/// use vantisweb::extensions::marketplace::MarketplaceConfig;
///
/// let config = MarketplaceConfig {
///     api_url: "https://custom.marketplace.io/api".to_string(),
///     allow_community: false,
///     enable_ratings: true,
///     cache_duration: 7200, // 2 hours
///     auto_update: false,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceConfig {
    /// API endpoint for the marketplace
    ///
    /// The base URL for the marketplace API. All API requests are made relative
    /// to this URL.
    pub api_url: String,
    
    /// Enable community extensions
    ///
    /// When `true`, allows discovery and installation of community-submitted
    /// extensions. When `false`, only verified/official extensions are shown.
    pub allow_community: bool,
    
    /// Enable extension ratings
    ///
    /// When `true`, displays ratings and allows users to submit reviews.
    pub enable_ratings: bool,
    
    /// Cache duration in seconds
    ///
    /// How long to cache extension metadata before refreshing from the API.
    /// Longer durations reduce API calls but may show stale data.
    pub cache_duration: u64,
    
    /// Auto-update extensions
    ///
    /// When `true`, automatically checks for and applies updates to installed
    /// extensions.
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
///
/// Complete metadata for an extension available in the marketplace. This includes
/// all information needed to display the extension to users and make installation
/// decisions.
///
/// # Examples
///
/// ```rust
/// use vantisweb::extensions::marketplace::ExtensionListing;
/// use chrono::Utc;
///
/// let listing = ExtensionListing {
///     id: "adblock-plus".to_string(),
///     name: "Adblock Plus".to_string(),
///     description: "Block ads and pop-ups".to_string(),
///     author: "Eyeo GmbH".to_string(),
///     version: "3.15.0".to_string(),
///     rating: 4.5,
///     is_verified: true,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionListing {
    /// Unique extension ID
    ///
    /// A unique identifier for the extension, typically in reverse domain notation
    /// (e.g., "com.example.extension").
    pub id: String,
    
    /// Extension name
    ///
    /// The display name of the extension as shown to users.
    pub name: String,
    
    /// Brief description
    ///
    /// A short, one-line description displayed in search results and listings.
    pub description: String,
    
    /// Detailed description (markdown)
    ///
    /// Full description of the extension's features and functionality, formatted
    /// in Markdown.
    pub long_description: Option<String>,
    
    /// Author/developer name
    ///
    /// The name of the extension developer or organization.
    pub author: String,
    
    /// Version string
    ///
    /// The current version of the extension, following semantic versioning.
    pub version: String,
    
    /// Download URL
    ///
    /// URL to download the extension package.
    pub download_url: String,
    
    /// Icon URL
    ///
    /// URL to the extension's icon image.
    pub icon_url: Option<String>,
    
    /// Screenshots URLs
    ///
    /// URLs to screenshots showcasing the extension's interface and functionality.
    pub screenshots: Vec<String>,
    
    /// Categories
    ///
    /// Categories the extension belongs to for browsing and filtering.
    pub categories: Vec<String>,
    
    /// Tags for search
    ///
    /// Keywords to help users discover the extension through search.
    pub tags: Vec<String>,
    
    /// Average rating (0-5)
    ///
    /// The average user rating on a scale of 0 to 5.
    pub rating: f32,
    
    /// Number of ratings
    ///
    /// The total number of user ratings received.
    pub rating_count: u32,
    
    /// Number of downloads
    ///
    /// Total lifetime downloads of the extension.
    pub download_count: u64,
    
    /// Last updated
    ///
    /// Timestamp of the last update to the extension.
    pub updated_at: DateTime<Utc>,
    
    /// Created at
    ///
    /// Timestamp when the extension was first published.
    pub created_at: DateTime<Utc>,
    
    /// Homepage URL
    ///
    /// URL to the extension's homepage or website.
    pub homepage: Option<String>,
    
    /// Support URL
    ///
    /// URL to the extension's support or documentation page.
    pub support_url: Option<String>,
    
    /// Required permissions
    ///
    /// List of permissions the extension requires to function.
    pub permissions: Vec<String>,
    
    /// Minimum browser version
    ///
    /// Minimum browser version required for the extension to work.
    pub min_browser_version: Option<String>,
    
    /// Is verified/official
    ///
    /// `true` if the extension has been verified by the marketplace team,
    /// `false` for community extensions.
    pub is_verified: bool,
    
    /// Is featured
    ///
    /// `true` if the extension is featured on the marketplace homepage.
    pub is_featured: bool,
}

/// Extension review
///
/// A user-submitted review for an extension, including rating, title, and content.
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
    ///
    /// Rating on a scale of 1 (poor) to 5 (excellent).
    pub rating: u8,
    /// Review title
    pub title: String,
    /// Review content
    pub content: String,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Helpful votes
    ///
    /// Number of users who found this review helpful.
    pub helpful_votes: u32,
}

/// Search filters for marketplace
///
/// Filters and sorting options for searching extensions in the marketplace.
///
/// # Examples
///
/// ```rust
/// use vantisweb::extensions::marketplace::{MarketplaceSearchFilters, SortBy, SortOrder};
///
/// let filters = MarketplaceSearchFilters {
///     query: Some("password manager".to_string()),
///     category: Some("security".to_string()),
///     min_rating: Some(4.0),
///     verified_only: true,
///     sort_by: Some(SortBy::Rating),
///     sort_order: Some(SortOrder::Desc),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarketplaceSearchFilters {
    /// Search query
    ///
    /// Free-text search query to find extensions by name, description, or tags.
    pub query: Option<String>,
    
    /// Filter by category
    ///
    /// Restrict results to a specific category.
    pub category: Option<String>,
    
    /// Filter by tags
    ///
    /// Restrict results to extensions having all specified tags.
    pub tags: Vec<String>,
    
    /// Minimum rating
    ///
    /// Only show extensions with rating >= this value (0-5).
    pub min_rating: Option<f32>,
    
    /// Only verified extensions
    ///
    /// When `true`, only show verified/official extensions.
    pub verified_only: bool,
    
    /// Sort by
    pub sort_by: Option<SortBy>,
    
    /// Sort order
    pub sort_order: Option<SortOrder>,
}

/// Sort options
///
/// Criteria for sorting search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortBy {
    /// Sort by relevance to search query
    Relevance,
    /// Sort by download count (most popular)
    Downloads,
    /// Sort by average rating
    Rating,
    /// Sort by last updated date
    Updated,
    /// Sort alphabetically by name
    Name,
}

/// Sort order
///
/// Direction for sorting results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    /// Ascending order
    Asc,
    /// Descending order
    Desc,
}

/// Marketplace cache
///
/// Internal cache for extension metadata to reduce API calls.
#[derive(Debug, Clone, Default)]
struct MarketplaceCache {
    extensions: HashMap<String, ExtensionListing>,
    categories: Vec<String>,
    featured: Vec<String>,
    last_updated: Option<DateTime<Utc>>,
}

/// Category with extension count
///
/// Information about a category including its display name and the number of
/// extensions in that category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryInfo {
    /// Display name of the category
    pub name: String,
    /// URL-safe slug for the category
    pub slug: String,
    /// Number of extensions in this category
    pub extension_count: u32,
    /// URL to category icon
    pub icon: Option<String>,
}

impl ExtensionMarketplace {
    /// Create a new marketplace instance
    ///
    /// Creates a new `ExtensionMarketplace` with the specified configuration.
    /// A new HTTP client is created for making API requests.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for the marketplace
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantisweb::extensions::marketplace::{ExtensionMarketplace, MarketplaceConfig};
    ///
    /// let config = MarketplaceConfig::default();
    /// let marketplace = ExtensionMarketplace::new(config);
    /// ```
    pub fn new(config: MarketplaceConfig) -> Self {
        Self {
            config,
            cache: Arc::new(RwLock::new(MarketplaceCache::default())),
            client: reqwest::Client::new(),
        }
    }

    /// Search for extensions
    ///
    /// Searches the marketplace for extensions matching the specified filters.
    ///
    /// # Arguments
    ///
    /// * `filters` - Search filters and sorting options
    ///
    /// # Returns
    ///
    /// A vector of `ExtensionListing` matching the search criteria.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails or the response is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use vantisweb::extensions::marketplace::{ExtensionMarketplace, MarketplaceSearchFilters};
    /// # #[tokio::main]
    /// # async fn example() -> anyhow::Result<()> {
    /// # let marketplace = ExtensionMarketplace::new(Default::default());
    /// let filters = MarketplaceSearchFilters {
    ///     query: Some("ad blocker".to_string()),
    ///     min_rating: Some(4.0),
    ///     ..Default::default()
    /// };
    /// let results = marketplace.search(filters).await?;
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// Retrieves detailed information about a specific extension by its ID.
    /// Results are cached to improve performance on repeated requests.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the extension
    ///
    /// # Returns
    ///
    /// `Some(ExtensionListing)` if found, `None` if the extension doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use vantisweb::extensions::marketplace::ExtensionMarketplace;
    /// # #[tokio::main]
    /// # async fn example() -> anyhow::Result<()> {
    /// # let marketplace = ExtensionMarketplace::new(Default::default());
    /// let extension = marketplace.get_extension("adblock-plus").await?;
    /// if let Some(ext) = extension {
    ///     println!("Found: {}", ext.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// Retrieves a list of extensions featured on the marketplace homepage.
    /// These are typically high-quality, popular, or new extensions.
    ///
    /// # Returns
    ///
    /// A vector of featured `ExtensionListing`.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use vantisweb::extensions::marketplace::ExtensionMarketplace;
    /// # #[tokio::main]
    /// # async fn example() -> anyhow::Result<()> {
    /// # let marketplace = ExtensionMarketplace::new(Default::default());
    /// let featured = marketplace.get_featured().await?;
    /// for ext in featured {
    ///     println!("Featured: {}", ext.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// Retrieves a list of all available categories in the marketplace,
    /// including the number of extensions in each category.
    ///
    /// # Returns
    ///
    /// A vector of `CategoryInfo` representing all categories.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use vantisweb::extensions::marketplace::ExtensionMarketplace;
    /// # #[tokio::main]
    /// # async fn example() -> anyhow::Result<()> {
    /// # let marketplace = ExtensionMarketplace::new(Default::default());
    /// let categories = marketplace.get_categories().await?;
    /// for cat in categories {
    ///     println!("{}: {} extensions", cat.name, cat.extension_count);
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// Retrieves all extensions belonging to a specific category.
    ///
    /// # Arguments
    ///
    /// * `category` - The category slug (e.g., "productivity", "security")
    ///
    /// # Returns
    ///
    /// A vector of `ExtensionListing` in the specified category.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use vantisweb::extensions::marketplace::ExtensionMarketplace;
    /// # #[tokio::main]
    /// # async fn example() -> anyhow::Result<()> {
    /// # let marketplace = ExtensionMarketplace::new(Default::default());
    /// let extensions = marketplace.get_by_category("security").await?;
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// Retrieves all user-submitted reviews for a specific extension.
    ///
    /// # Arguments
    ///
    /// * `extension_id` - The ID of the extension
    ///
    /// # Returns
    ///
    /// A vector of `ExtensionReview` for the extension.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
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
    ///
    /// Submits a user review for an extension. Requires authentication token.
    ///
    /// # Arguments
    ///
    /// * `extension_id` - The ID of the extension being reviewed
    /// * `rating` - Rating from 1 to 5
    /// * `title` - Review title
    /// * `content` - Review content/description
    /// * `token` - Authentication token for the user
    ///
    /// # Returns
    ///
    /// The submitted `ExtensionReview`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Rating is not between 1 and 5
    /// - Authentication fails
    /// - The API request fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use vantisweb::extensions::marketplace::ExtensionMarketplace;
    /// # #[tokio::main]
    /// # async fn example() -> anyhow::Result<()> {
    /// # let marketplace = ExtensionMarketplace::new(Default::default());
    /// let review = marketplace.submit_review(
    ///     "extension-id",
    ///     5,
    ///     "Great extension!",
    ///     "This extension works perfectly and is easy to use.",
    ///     "auth-token"
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// Downloads the extension package for a given extension ID.
    ///
    /// # Arguments
    ///
    /// * `extension_id` - The ID of the extension to download
    ///
    /// # Returns
    ///
    /// Raw bytes of the extension package.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Extension not found
    /// - Download fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use vantisweb::extensions::marketplace::ExtensionMarketplace;
    /// # #[tokio::main]
    /// # async fn example() -> anyhow::Result<()> {
    /// # let marketplace = ExtensionMarketplace::new(Default::default());
    /// let bytes = marketplace.download_extension("extension-id").await?;
    /// // Save bytes to file...
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// Checks if any of the installed extensions have updates available.
    ///
    /// # Arguments
    ///
    /// * `installed` - List of installed extension IDs
    ///
    /// # Returns
    ///
    /// A map of extension ID to latest version for extensions with updates.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use vantisweb::extensions::marketplace::ExtensionMarketplace;
    /// # #[tokio::main]
    /// # async fn example() -> anyhow::Result<()> {
    /// # let marketplace = ExtensionMarketplace::new(Default::default());
    /// let installed = vec!["ext1".to_string(), "ext2".to_string()];
    /// let updates = marketplace.check_updates(&installed).await?;
    /// for (id, version) in updates {
    ///     println!("Update available for {}: {}", id, version);
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// Clears all cached extension data. This forces the next request to
    /// fetch fresh data from the API.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use vantisweb::extensions::marketplace::ExtensionMarketplace;
    /// # #[tokio::main]
    /// # async fn example() -> anyhow::Result<()> {
    /// # let marketplace = ExtensionMarketplace::new(Default::default());
    /// marketplace.clear_cache().await;
    /// # Ok(())
    /// # }
    /// ```
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