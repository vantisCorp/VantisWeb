//! Extension Permissions Module
//!
//! Manages permissions for browser extensions, including requesting,
//! granting, and revoking permissions.

use anyhow::{Result, Error};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Permission manager for extensions
pub struct PermissionManager {
    granted_permissions: Arc<RwLock<HashMap<String, HashSet<Permission>>>>,
    config: PermissionConfig,
}

/// Permission configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionConfig {
    /// Auto-grant safe permissions
    pub auto_grant_safe: bool,
    /// Require user confirmation for dangerous permissions
    pub require_confirmation: bool,
    /// Allow temporary permissions
    pub allow_temporary: bool,
    /// Temporary permission duration in seconds
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    // Tabs permissions
    Tabs,
    ActiveTab,
    TabHide,
    
    // Navigation permissions
    WebNavigation,
    WebRequest,
    
    // Storage permissions
    Storage,
    UnlimitedStorage,
    
    // Network permissions
    Host(String),
    AllUrls,
    
    // Browser features
    Bookmarks,
    History,
    Downloads,
    DownloadsOpen,
    
    // UI permissions
    ContextMenus,
    Notifications,
    BrowserAction,
    PageAction,
    SidePanel,
    
    // System permissions
    ClipboardRead,
    ClipboardWrite,
    Geolocation,
    
    // Content permissions
    ContentScripts,
    declarativeContent,
    
    // Privacy-related
    Privacy,
    Proxy,
    
    // Debug
    Debugger,
    
    // Native messaging
    NativeMessaging,
}

impl Permission {
    /// Check if this is a "safe" permission that can be auto-granted
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionGrant {
    /// Extension ID
    pub extension_id: String,
    /// Granted permissions
    pub permissions: Vec<Permission>,
    /// Whether the grant is temporary
    pub temporary: bool,
    /// Expiration time (for temporary grants)
    pub expires_at: Option<i64>,
}

impl PermissionManager {
    /// Create a new permission manager
    pub fn new(config: PermissionConfig) -> Self {
        Self {
            granted_permissions: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Check if an extension has a specific permission
    pub async fn has_permission(&self, extension_id: &str, permission: &Permission) -> bool {
        let permissions = self.granted_permissions.read().await;
        if let Some(granted) = permissions.get(extension_id) {
            granted.contains(permission)
        } else {
            false
        }
    }

    /// Get all permissions for an extension
    pub async fn get_permissions(&self, extension_id: &str) -> Vec<Permission> {
        let permissions = self.granted_permissions.read().await;
        if let Some(granted) = permissions.get(extension_id) {
            granted.iter().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Request permissions for an extension
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
    pub async fn revoke_all(&self, extension_id: &str) -> Result<()> {
        let mut granted = self.granted_permissions.write().await;
        granted.remove(extension_id);
        Ok(())
    }

    /// Check if a host permission matches a URL
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