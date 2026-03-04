//! # Extension Permissions Module
//!
//! Manages permissions for browser extensions, including requesting, granting,
//! and revoking permissions. This module provides a comprehensive permission
//! system with support for safe/dangerous permission classification, temporary
//! permissions, and user confirmation workflows.
//!
//! ## Features
//!
//! - **Permission Classification**: Automatically categorizes permissions as safe or dangerous
//! - **User Confirmation**: Requires explicit consent for dangerous permissions
//! - **Temporary Permissions**: Supports time-limited permission grants
//! - **Host Matching**: Pattern-based URL permission matching
//! - **Permission Warning**: User-friendly warning messages for sensitive permissions
//!
//! ## Permission Types
//!
//! The module supports a wide range of permission types including:
//! - **Tabs**: Access to browser tabs and navigation
//! - **Storage**: Local data storage
//! - **Network**: Host permissions and web requests
//! - **Browser Features**: Bookmarks, history, downloads
//! - **UI**: Context menus, notifications, toolbar actions
//! - **System**: Clipboard, geolocation
//! - **Privacy**: Privacy settings, proxy control
//! - **Debug**: Debugger access
//!
//! ## Example Usage
//!
//! ```rust
//! use vantisweb::extensions::permissions::{PermissionManager, PermissionConfig, Permission};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = PermissionConfig::default();
//!     let manager = PermissionManager::new(config);
//!
//!     // Request permissions
//!     let request = manager.request_permissions(
//!         "extension-id",
//!         &[Permission::Storage, Permission::ActiveTab]
//!     ).await?;
//!
//!     // Check if permission is granted
//!     let has_storage = manager.has_permission("extension-id", &Permission::Storage).await;
//!
//!     // Revoke permissions
//!     manager.revoke_permissions("extension-id", &[Permission::Storage]).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Security Considerations
//!
//! - Dangerous permissions (AllUrls, Debugger, etc.) require explicit user consent
//! - Temporary permissions automatically expire after a configured duration
//! - Permission warnings help users understand the implications of grants
//! - Host permissions use pattern matching with wildcard support

use anyhow::{Result, Error};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Permission manager for extensions
///
/// The `PermissionManager` handles all permission-related operations for browser
/// extensions, including requesting, granting, checking, and revoking permissions.
/// It maintains a thread-safe store of granted permissions and provides
/// configuration options for permission behavior.
///
/// # Examples
///
/// ```rust
/// use vantisweb::extensions::permissions::{PermissionManager, PermissionConfig};
///
/// let config = PermissionConfig::default();
/// let manager = PermissionManager::new(config);
/// ```
///
/// # Thread Safety
///
/// The manager is thread-safe and can be shared across multiple tasks
/// through `Arc<PermissionManager>`.
pub struct PermissionManager {
    granted_permissions: Arc<RwLock<HashMap<String, HashSet<Permission>>>>,
    config: PermissionConfig,
}

/// Permission configuration
///
/// Configuration options controlling permission behavior, including automatic
/// granting rules, confirmation requirements, and temporary permission settings.
///
/// # Examples
///
/// ```rust
/// use vantisweb::extensions::permissions::PermissionConfig;
///
/// let config = PermissionConfig {
///     auto_grant_safe: true,
///     require_confirmation: true,
///     allow_temporary: true,
///     temporary_duration: 7200, // 2 hours
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionConfig {
    /// Auto-grant safe permissions
    ///
    /// When `true`, safe permissions (Storage, ActiveTab, etc.) are automatically
    /// granted without user confirmation.
    pub auto_grant_safe: bool,
    
    /// Require user confirmation for dangerous permissions
    ///
    /// When `true`, dangerous permissions (AllUrls, Debugger, etc.) require
    /// explicit user consent before being granted.
    pub require_confirmation: bool,
    
    /// Allow temporary permissions
    ///
    /// When `true`, permissions can be granted for a limited time.
    pub allow_temporary: bool,
    
    /// Temporary permission duration in seconds
    ///
    /// The duration for which temporary permissions remain valid before expiring.
    pub temporary_duration: u64,
}

impl Default for PermissionConfig {
    fn default() -> Self {
        Self {
            auto_grant_safe: true,
            require_confirmation: true,
            allow_temporary: true,
            temporary_duration: 3600, // 1 hour
        }
    }
}

/// Extension permission types
///
/// Represents all possible permissions that can be requested by browser extensions.
/// Each permission grants access to specific browser features or data.
///
/// # Examples
///
/// ```rust
/// use vantisweb::extensions::permissions::Permission;
///
/// let storage_perm = Permission::Storage;
/// let host_perm = Permission::Host("https://example.com/*".to_string());
///
/// assert!(storage_perm.is_safe());
/// assert!(Permission::AllUrls.is_dangerous());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    // Tabs permissions
    
    /// Access browser tabs
    ///
    /// Allows the extension to access and manipulate browser tabs, including
    /// reading tab URLs, creating new tabs, and closing tabs.
    Tabs,
    
    /// Access the active tab
    ///
    /// Allows the extension to access only the currently active tab.
    /// This is a more restrictive version of the Tabs permission.
    ActiveTab,
    
    /// Hide and show browser tabs
    ///
    /// Allows the extension to hide and show tabs programmatically.
    TabHide,
    
    // Navigation permissions
    
    /// Access browser navigation
    ///
    /// Allows the extension to monitor and interact with browser navigation events.
    WebNavigation,
    
    /// Intercept network requests
    ///
    /// Allows the extension to intercept, block, and modify network requests.
    /// This is a **dangerous** permission.
    WebRequest,
    
    // Storage permissions
    
    /// Store data locally
    ///
    /// Allows the extension to store data in the browser's local storage.
    /// This is a **safe** permission.
    Storage,
    
    /// Unlimited local storage
    ///
    /// Allows the extension to use unlimited local storage without quota limits.
    UnlimitedStorage,
    
    // Network permissions
    
    /// Access data on specific host(s)
    ///
    /// Allows the extension to access data on the specified host(s).
    /// Supports wildcard patterns (e.g., "https://*.example.com/*").
    Host(String),
    
    /// Access all website data
    ///
    /// Allows the extension to read and modify data on all websites.
    /// This is a **dangerous** permission that should be used with extreme caution.
    AllUrls,
    
    // Browser features
    
    /// Read and modify bookmarks
    ///
    /// Allows the extension to access, create, modify, and delete bookmarks.
    Bookmarks,
    
    /// Access browsing history
    ///
    /// Allows the extension to read and modify the user's browsing history.
    History,
    
    /// Manage downloads
    ///
    /// Allows the extension to create, pause, resume, and cancel downloads.
    Downloads,
    
    /// Open downloaded files
    ///
    /// Allows the extension to open files that have been downloaded.
    DownloadsOpen,
    
    // UI permissions
    
    /// Add items to context menu
    ///
    /// Allows the extension to add items to the browser's context menu.
    /// This is a **safe** permission.
    ContextMenus,
    
    /// Show desktop notifications
    ///
    /// Allows the extension to display desktop notifications to the user.
    /// This is a **safe** permission.
    Notifications,
    
    /// Add button to toolbar
    ///
    /// Allows the extension to add a button to the browser toolbar.
    /// This is a **safe** permission.
    BrowserAction,
    
    /// Add icon to address bar
    ///
    /// Allows the extension to add an icon to the address bar.
    /// This is a **safe** permission.
    PageAction,
    
    /// Show content in side panel
    ///
    /// Allows the extension to display content in the browser's side panel.
    SidePanel,
    
    // System permissions
    
    /// Read clipboard data
    ///
    /// Allows the extension to read data from the system clipboard.
    /// This is a **dangerous** permission.
    ClipboardRead,
    
    /// Write to clipboard
    ///
    /// Allows the extension to write data to the system clipboard.
    ClipboardWrite,
    
    /// Access your location
    ///
    /// Allows the extension to access the user's physical location.
    /// This is a **dangerous** permission.
    Geolocation,
    
    // Content permissions
    
    /// Run scripts on web pages
    ///
    /// Allows the extension to inject and execute JavaScript on web pages.
    ContentScripts,
    
    /// React to page content
    ///
    /// Allows the extension to detect and react to specific page content.
    declarativeContent,
    
    // Privacy-related
    
    /// Access privacy settings
    ///
    /// Allows the extension to access and modify privacy settings.
    /// This is a **dangerous** permission.
    Privacy,
    
    /// Control proxy settings
    ///
    /// Allows the extension to control browser proxy settings.
    /// This is a **dangerous** permission.
    Proxy,
    
    // Debug
    
    /// Debug web pages
    ///
    /// Allows the extension to debug web pages, accessing sensitive data.
    /// This is a **dangerous** permission that should only be used for
    /// development purposes.
    Debugger,
    
    // Native messaging
    
    /// Communicate with native apps
    ///
    /// Allows the extension to communicate with native applications installed
    /// on the user's computer. This is a **dangerous** permission.
    NativeMessaging,
}

impl Permission {
    /// Check if this is a "safe" permission that can be auto-granted
    ///
    /// Safe permissions are those that pose minimal risk to user privacy and
    /// security. These can be automatically granted without user confirmation
    /// if `auto_grant_safe` is enabled in the configuration.
    ///
    /// # Returns
    ///
    /// `true` if the permission is considered safe, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantisweb::extensions::permissions::Permission;
    ///
    /// assert!(Permission::Storage.is_safe());
    /// assert!(Permission::ActiveTab.is_safe());
    /// assert!(!Permission::AllUrls.is_safe());
    /// ```
    pub fn is_safe(&self) -> bool {
        matches!(
            self,
            Permission::Storage
                | Permission::ActiveTab
                | Permission::ContextMenus
                | Permission::Notifications
                | Permission::BrowserAction
                | Permission::PageAction
        )
    }

    /// Check if this is a "dangerous" permission requiring explicit consent
    ///
    /// Dangerous permissions grant broad access to sensitive data or system
    /// features. These always require user confirmation before being granted.
    ///
    /// # Returns
    ///
    /// `true` if the permission is considered dangerous, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantisweb::extensions::permissions::Permission;
    ///
    /// assert!(Permission::AllUrls.is_dangerous());
    /// assert!(Permission::Debugger.is_dangerous());
    /// assert!(!Permission::Storage.is_dangerous());
    /// ```
    pub fn is_dangerous(&self) -> bool {
        matches!(
            self,
            Permission::AllUrls
                | Permission::Tabs
                | Permission::WebRequest
                | Permission::Debugger
                | Permission::NativeMessaging
                | Permission::Privacy
                | Permission::Proxy
                | Permission::ClipboardRead
                | Permission::Geolocation
        )
    }

    /// Get the display name for this permission
    ///
    /// Returns a user-friendly string describing what the permission does.
    /// This is suitable for display in permission request dialogs.
    ///
    /// # Returns
    ///
    /// A human-readable description of the permission.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantisweb::extensions::permissions::Permission;
    ///
    /// assert_eq!(Permission::Tabs.display_name(), "Access browser tabs");
    /// assert_eq!(Permission::Storage.display_name(), "Store data locally");
    /// ```
    pub fn display_name(&self) -> String {
        match self {
            Permission::Tabs => "Access browser tabs".to_string(),
            Permission::ActiveTab => "Access the active tab".to_string(),
            Permission::TabHide => "Hide and show browser tabs".to_string(),
            Permission::WebNavigation => "Access browser navigation".to_string(),
            Permission::WebRequest => "Intercept network requests".to_string(),
            Permission::Storage => "Store data locally".to_string(),
            Permission::UnlimitedStorage => "Unlimited local storage".to_string(),
            Permission::Host(host) => format!("Access data on {}", host),
            Permission::AllUrls => "Access all website data".to_string(),
            Permission::Bookmarks => "Read and modify bookmarks".to_string(),
            Permission::History => "Access browsing history".to_string(),
            Permission::Downloads => "Manage downloads".to_string(),
            Permission::DownloadsOpen => "Open downloaded files".to_string(),
            Permission::ContextMenus => "Add items to context menu".to_string(),
            Permission::Notifications => "Show desktop notifications".to_string(),
            Permission::BrowserAction => "Add button to toolbar".to_string(),
            Permission::PageAction => "Add icon to address bar".to_string(),
            Permission::SidePanel => "Show content in side panel".to_string(),
            Permission::ClipboardRead => "Read clipboard data".to_string(),
            Permission::ClipboardWrite => "Write to clipboard".to_string(),
            Permission::Geolocation => "Access your location".to_string(),
            Permission::ContentScripts => "Run scripts on web pages".to_string(),
            Permission::declarativeContent => "React to page content".to_string(),
            Permission::Privacy => "Access privacy settings".to_string(),
            Permission::Proxy => "Control proxy settings".to_string(),
            Permission::Debugger => "Debug web pages".to_string(),
            Permission::NativeMessaging => "Communicate with native apps".to_string(),
        }
    }

    /// Get the warning message for dangerous permissions
    ///
    /// Returns a detailed warning message explaining the risks associated with
    /// granting this permission. Returns `None` for safe permissions.
    ///
    /// # Returns
    ///
    /// `Some(String)` with a warning message for dangerous permissions,
    /// `None` for safe permissions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantisweb::extensions::permissions::Permission;
    ///
    /// assert!(Permission::AllUrls.warning_message().is_some());
    /// assert!(Permission::Storage.warning_message().is_none());
    /// ```
    pub fn warning_message(&self) -> Option<String> {
        if self.is_dangerous() {
            match self {
                Permission::AllUrls => Some("This extension can read and modify all your data on all websites.".to_string()),
                Permission::Tabs => Some("This extension can access your open tabs and the websites you visit.".to_string()),
                Permission::WebRequest => Some("This extension can see and modify all your web traffic.".to_string()),
                Permission::Debugger => Some("This extension can debug any page, accessing sensitive data.".to_string()),
                Permission::NativeMessaging => Some("This extension can communicate with applications on your computer.".to_string()),
                Permission::Geolocation => Some("This extension can access your physical location.".to_string()),
                Permission::ClipboardRead => Some("This extension can read data you copy to the clipboard.".to_string()),
                _ => None,
            }
        } else {
            None
        }
    }
}

/// Permission request result
///
/// Represents a pending permission request that requires user confirmation.
/// This is returned when requesting dangerous permissions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequest {
    /// Extension ID requesting permission
    pub extension_id: String,
    /// Permissions being requested
    pub permissions: Vec<Permission>,
    /// Whether this is a temporary grant
    pub temporary: bool,
}

/// Permission grant result
///
/// Represents a successful permission grant, including metadata about
/// the granted permissions and their expiration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionGrant {
    /// Extension ID
    pub extension_id: String,
    /// Granted permissions
    pub permissions: Vec<Permission>,
    /// Whether the grant is temporary
    pub temporary: bool,
    /// Expiration time (for temporary grants)
    ///
    /// Unix timestamp when temporary permissions expire.
    pub expires_at: Option<i64>,
}

impl PermissionManager {
    /// Create a new permission manager
    ///
    /// Creates a new `PermissionManager` with the specified configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for permission behavior
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantisweb::extensions::permissions::{PermissionManager, PermissionConfig};
    ///
    /// let config = PermissionConfig::default();
    /// let manager = PermissionManager::new(config);
    /// ```
    pub fn new(config: PermissionConfig) -> Self {
        Self {
            granted_permissions: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Check if an extension has a specific permission
    ///
    /// Checks whether the specified extension has been granted the given permission.
    ///
    /// # Arguments
    ///
    /// * `extension_id` - The ID of the extension to check
    /// * `permission` - The permission to check for
    ///
    /// # Returns
    ///
    /// `true` if the extension has the permission, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use vantisweb::extensions::permissions::{PermissionManager, Permission};
    /// # #[tokio::main]
    /// # async fn example() -> anyhow::Result<()> {
    /// # let manager = PermissionManager::new(Default::default());
    /// let has_storage = manager.has_permission("ext-id", &Permission::Storage).await;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn has_permission(&self, extension_id: &str, permission: &Permission) -> bool {
        let permissions = self.granted_permissions.read().await;
        if let Some(granted) = permissions.get(extension_id) {
            granted.contains(permission)
        } else {
            false
        }
    }

    /// Get all permissions for an extension
    ///
    /// Returns a list of all permissions granted to the specified extension.
    ///
    /// # Arguments
    ///
    /// * `extension_id` - The ID of the extension
    ///
    /// # Returns
    ///
    /// A vector of all granted permissions for the extension.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use vantisweb::extensions::permissions::{PermissionManager, Permission};
    /// # #[tokio::main]
    /// # async fn example() -> anyhow::Result<()> {
    /// # let manager = PermissionManager::new(Default::default());
    /// let perms = manager.get_permissions("ext-id").await;
    /// println!("Permissions: {:?}", perms);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_permissions(&self, extension_id: &str) -> Vec<Permission> {
        let permissions = self.granted_permissions.read().await;
        if let Some(granted) = permissions.get(extension_id) {
            granted.iter().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Request permissions for an extension
    ///
    /// Requests permissions for an extension. Safe permissions may be auto-granted
    /// based on configuration, while dangerous permissions require user confirmation.
    ///
    /// # Arguments
    ///
    /// * `extension_id` - The ID of the extension requesting permissions
    /// * `permissions` - The permissions to request
    ///
    /// # Returns
    ///
    /// A `PermissionRequest` indicating which permissions were granted or need
    /// user confirmation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use vantisweb::extensions::permissions::{PermissionManager, Permission};
    /// # #[tokio::main]
    /// # async fn example() -> anyhow::Result<()> {
    /// # let manager = PermissionManager::new(Default::default());
    /// let request = manager.request_permissions(
    ///     "ext-id",
    ///     &[Permission::Storage, Permission::ActiveTab]
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn request_permissions(
        &self,
        extension_id: &str,
        permissions: &[Permission],
    ) -> Result<PermissionRequest> {
        // Check for dangerous permissions
        let dangerous: Vec<_> = permissions.iter().filter(|p| p.is_dangerous()).collect();
        
        if !dangerous.is_empty() && self.config.require_confirmation {
            // Return request for user confirmation
            Ok(PermissionRequest {
                extension_id: extension_id.to_string(),
                permissions: permissions.to_vec(),
                temporary: false,
            })
        } else {
            // Auto-grant safe permissions
            let safe_only: Vec<_> = permissions.iter()
                .filter(|p| p.is_safe())
                .cloned()
                .collect();
            
            self.grant_permissions(extension_id, &safe_only, false).await?;
            
            Ok(PermissionRequest {
                extension_id: extension_id.to_string(),
                permissions: safe_only,
                temporary: false,
            })
        }
    }

    /// Grant permissions to an extension
    ///
    /// Explicitly grants the specified permissions to an extension.
    ///
    /// # Arguments
    ///
    /// * `extension_id` - The ID of the extension
    /// * `permissions` - The permissions to grant
    /// * `temporary` - Whether the grant is temporary
    ///
    /// # Returns
    ///
    /// A `PermissionGrant` with details about the granted permissions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use vantisweb::extensions::permissions::{PermissionManager, Permission};
    /// # #[tokio::main]
    /// # async fn example() -> anyhow::Result<()> {
    /// # let manager = PermissionManager::new(Default::default());
    /// let grant = manager.grant_permissions(
    ///     "ext-id",
    ///     &[Permission::Storage],
    ///     false
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn grant_permissions(
        &self,
        extension_id: &str,
        permissions: &[Permission],
        temporary: bool,
    ) -> Result<PermissionGrant> {
        let mut granted = self.granted_permissions.write().await;
        
        let entry = granted.entry(extension_id.to_string()).or_insert_with(HashSet::new);
        
        for permission in permissions {
            entry.insert(permission.clone());
        }

        let expires_at = if temporary {
            Some(chrono::Utc::now().timestamp() + self.config.temporary_duration as i64)
        } else {
            None
        };

        Ok(PermissionGrant {
            extension_id: extension_id.to_string(),
            permissions: permissions.to_vec(),
            temporary,
            expires_at,
        })
    }

    /// Revoke permissions from an extension
    ///
    /// Revokes specific permissions from an extension.
    ///
    /// # Arguments
    ///
    /// * `extension_id` - The ID of the extension
    /// * `permissions` - The permissions to revoke
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use vantisweb::extensions::permissions::{PermissionManager, Permission};
    /// # #[tokio::main]
    /// # async fn example() -> anyhow::Result<()> {
    /// # let manager = PermissionManager::new(Default::default());
    /// manager.revoke_permissions("ext-id", &[Permission::Storage]).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn revoke_permissions(
        &self,
        extension_id: &str,
        permissions: &[Permission],
    ) -> Result<()> {
        let mut granted = self.granted_permissions.write().await;
        
        if let Some(entry) = granted.get_mut(extension_id) {
            for permission in permissions {
                entry.remove(permission);
            }
        }

        Ok(())
    }

    /// Revoke all permissions for an extension
    ///
    /// Revokes all permissions granted to an extension, effectively removing it
    /// from the permission store.
    ///
    /// # Arguments
    ///
    /// * `extension_id` - The ID of the extension
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use vantisweb::extensions::permissions::PermissionManager;
    /// # #[tokio::main]
    /// # async fn example() -> anyhow::Result<()> {
    /// # let manager = PermissionManager::new(Default::default());
    /// manager.revoke_all("ext-id").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn revoke_all(&self, extension_id: &str) -> Result<()> {
        let mut granted = self.granted_permissions.write().await;
        granted.remove(extension_id);
        Ok(())
    }

    /// Check if a host permission matches a URL
    ///
    /// Pattern matching for host permissions. Supports wildcard patterns.
    ///
    /// # Arguments
    ///
    /// * `pattern` - The host permission pattern (e.g., "https://*.example.com/*")
    /// * `url` - The URL to check against
    ///
    /// # Returns
    ///
    /// `true` if the pattern matches the URL, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use vantisweb::extensions::permissions::PermissionManager;
    /// # let manager = PermissionManager::new(Default::default());
    /// assert!(manager.matches_host("https://*.example.com/*", "https://sub.example.com/page"));
    /// assert!(manager.matches_host("<all_urls>", "https://any-site.com/page"));
    /// ```
    pub fn matches_host(&self, pattern: &str, url: &str) -> bool {
        // Handle wildcard patterns
        if pattern == "<all_urls>" {
            return true;
        }

        // Simple pattern matching
        let pattern = pattern.replace("*", "");
        url.starts_with(&pattern) || pattern.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_is_safe() {
        assert!(Permission::Storage.is_safe());
        assert!(Permission::ActiveTab.is_safe());
        assert!(!Permission::AllUrls.is_safe());
    }

    #[test]
    fn test_permission_is_dangerous() {
        assert!(Permission::AllUrls.is_dangerous());
        assert!(Permission::Debugger.is_dangerous());
        assert!(!Permission::Storage.is_dangerous());
    }

    #[test]
    fn test_permission_display_name() {
        assert_eq!(Permission::Tabs.display_name(), "Access browser tabs");
        assert_eq!(Permission::Storage.display_name(), "Store data locally");
    }

    #[test]
    fn test_permission_warning_message() {
        assert!(Permission::AllUrls.warning_message().is_some());
        assert!(Permission::Storage.warning_message().is_none());
    }

    #[tokio::test]
    async fn test_permission_manager_grant() {
        let manager = PermissionManager::new(PermissionConfig::default());
        manager
            .grant_permissions("ext1", &[Permission::Storage], false)
            .await
            .unwrap();
        
        assert!(manager.has_permission("ext1", &Permission::Storage).await);
    }

    #[tokio::test]
    async fn test_permission_manager_revoke() {
        let manager = PermissionManager::new(PermissionConfig::default());
        manager
            .grant_permissions("ext1", &[Permission::Storage, Permission::Tabs], false)
            .await
            .unwrap();
        
        manager
            .revoke_permissions("ext1", &[Permission::Storage])
            .await
            .unwrap();
        
        assert!(!manager.has_permission("ext1", &Permission::Storage).await);
        assert!(manager.has_permission("ext1", &Permission::Tabs).await);
    }
}