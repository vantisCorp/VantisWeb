//! Tab Hibernation
//!
//! Hibernate inactive tabs to save memory.

use std::collections::HashMap;
use std::time::Duration;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Tab hibernation manager
pub struct TabHibernationManager {
    hibernated_tabs: Arc<RwLock<HashMap<String, HibernatedTab>>>,
    settings: Arc<RwLock<HibernationSettings>>,
}

/// Hibernated tab state
#[derive(Debug, Clone)]
pub struct HibernatedTab {
    pub tab_id: String,
    pub url: String,
    pub title: String,
    pub favicon: Option<String>,
    pub scroll_position: Option<f64>,
    pub form_data: Option<String>,
    pub hibernated_at: chrono::DateTime<chrono::Utc>,
    pub estimated_memory_saved: u64, // in bytes
}

/// Hibernation settings
#[derive(Debug, Clone, Copy)]
pub struct HibernationSettings {
    pub auto_hibernate: bool,
    pub idle_timeout_seconds: u64,
    pub max_hibernated_tabs: usize,
    pub preserve_form_data: bool,
    pub preserve_scroll_position: bool,
}

impl Default for HibernationSettings {
    fn default() -> Self {
        Self {
            auto_hibernate: true,
            idle_timeout_seconds: 300, // 5 minutes
            max_hibernated_tabs: 50,
            preserve_form_data: true,
            preserve_scroll_position: true,
        }
    }
}

impl TabHibernationManager {
    /// Create a new tab hibernation manager
    pub fn new() -> Self {
        Self {
            hibernated_tabs: Arc::new(RwLock::new(HashMap::new())),
            settings: Arc::new(RwLock::new(HibernationSettings::default())),
        }
    }

    /// Hibernate a tab
    pub async fn hibernate(&self, tab_id: String, url: String, title: String, favicon: Option<String>) -> HibernatedTab {
        let now = chrono::Utc::now();
        
        let hibernated = HibernatedTab {
            tab_id: tab_id.clone(),
            url: url.clone(),
            title: title.clone(),
            favicon,
            scroll_position: None,
            form_data: None,
            hibernated_at: now,
            estimated_memory_saved: Self::estimate_memory_savings(&url),
        };

        let mut tabs = self.hibernated_tabs.write().await;
        tabs.insert(tab_id.clone(), hibernated.clone());
        
        log::info!("Hibernated tab: {} ({})", title, url);
        hibernated
    }

    /// Wake up a hibernated tab
    pub async fn wake_up(&self, tab_id: &str) -> Option<HibernatedTab> {
        let mut tabs = self.hibernated_tabs.write().await;
        let hibernated = tabs.remove(tab_id)?;
        
        log::info!("Woke up tab: {}", hibernated.title);
        Some(hibernated)
    }

    /// Check if tab is hibernated
    pub async fn is_hibernated(&self, tab_id: &str) -> bool {
        self.hibernated_tabs.read().await.contains_key(tab_id)
    }

    /// Get hibernated tab
    pub async fn get(&self, tab_id: &str) -> Option<HibernatedTab> {
        self.hibernated_tabs.read().await.get(tab_id).cloned()
    }

    /// Get all hibernated tabs
    pub async fn get_all(&self) -> Vec<HibernatedTab> {
        self.hibernated_tabs.read().await.values().cloned().collect()
    }

    /// Remove hibernated tab
    pub async fn remove(&self, tab_id: &str) {
        let mut tabs = self.hibernated_tabs.write().await;
        tabs.remove(tab_id);
    }

    /// Clear all hibernated tabs
    pub async fn clear(&self) {
        let mut tabs = self.hibernated_tabs.write().await;
        tabs.clear();
    }

    /// Get hibernation settings
    pub async fn get_settings(&self) -> HibernationSettings {
        *self.settings.read().await
    }

    /// Update hibernation settings
    pub async fn update_settings(&self, settings: HibernationSettings) {
        let mut current = self.settings.write().await;
        *current = settings;
    }

    /// Auto-hibernate idle tabs
    pub async fn auto_hibernate_idle(&self, tab_access_times: &HashMap<String, chrono::DateTime<chrono::Utc>>) -> Vec<String> {
        let settings = self.get_settings().await;
        if !settings.auto_hibernate {
            return vec![];
        }

        let mut hibernated = vec![];
        let now = chrono::Utc::now();
        let timeout = Duration::from_secs(settings.idle_timeout_seconds);

        for (tab_id, last_accessed) in tab_access_times {
            let idle_duration = now.signed_duration_since(*last_accessed);
            if idle_duration > chrono::Duration::from_std(timeout).unwrap() {
                // In a real implementation, would get tab info and hibernate
                hibernated.push(tab_id.clone());
            }
        }

        hibernated
    }

    /// Get memory savings
    pub async fn get_memory_savings(&self) -> u64 {
        let tabs = self.hibernated_tabs.read().await;
        tabs.values().map(|t| t.estimated_memory_saved).sum()
    }

    /// Get hibernation statistics
    pub async fn get_stats(&self) -> HibernationStats {
        let tabs = self.hibernated_tabs.read().await;
        let count = tabs.len();
        let total_saved = tabs.values().map(|t| t.estimated_memory_saved).sum();
        
        // Calculate average hibernation time
        let now = chrono::Utc::now();
        let total_duration: i64 = tabs.values()
            .map(|t| (now - t.hibernated_at).num_seconds())
            .sum();
        let avg_duration = if count > 0 { total_duration / count as i64 } else { 0 };

        HibernationStats {
            hibernated_count: count,
            total_memory_saved: total_saved,
            average_hibernation_duration_seconds: avg_duration,
        }
    }

    /// Estimate memory savings for a tab
    fn estimate_memory_savings(url: &str) -> u64 {
        // Estimate based on URL type
        let base = 50 * 1024 * 1024; // 50MB base
        
        if url.contains("youtube.com") || url.contains("netflix.com") {
            base * 3 // Video sites use more memory
        } else if url.contains("facebook.com") || url.contains("twitter.com") {
            base * 2 // Social media sites
        } else {
            base
        }
    }

    /// Wake up all hibernated tabs
    pub async fn wake_all(&self) -> Vec<HibernatedTab> {
        let mut tabs = self.hibernated_tabs.write().await;
        let hibernated: Vec<HibernatedTab> = tabs.drain().map(|(_, t)| t).collect();
        
        log::info!("Woke up {} hibernated tabs", hibernated.len());
        hibernated
    }

    /// Hibernate all tabs except specified ones
    pub async fn hibernate_all_except(&self, except: &[String], tab_data: &HashMap<String, (String, String, Option<String>)>) -> Vec<HibernatedTab> {
        let mut hibernated = vec![];
        
        for (tab_id, (url, title, favicon)) in tab_data {
            if !except.contains(tab_id) {
                let h = self.hibernate(tab_id.clone(), url.clone(), title.clone(), favicon.clone()).await;
                hibernated.push(h);
            }
        }
        
        hibernated
    }
}

impl Default for TabHibernationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Hibernation statistics
#[derive(Debug, Clone, Copy)]
pub struct HibernationStats {
    pub hibernated_count: usize,
    pub total_memory_saved: u64,
    pub average_hibernation_duration_seconds: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hibernation_manager_creation() {
        let manager = TabHibernationManager::new();
        assert_eq!(manager.get_all().await.len(), 0);
    }

    #[tokio::test]
    async fn test_hibernate_tab() {
        let manager = TabHibernationManager::new();
        let hibernated = manager.hibernate(
            "tab-1".to_string(),
            "https://example.com".to_string(),
            "Example".to_string(),
            None
        ).await;
        
        assert_eq!(hibernated.tab_id, "tab-1");
        assert!(manager.is_hibernated("tab-1").await);
    }

    #[tokio::test]
    async fn test_wake_up() {
        let manager = TabHibernationManager::new();
        manager.hibernate(
            "tab-1".to_string(),
            "https://example.com".to_string(),
            "Example".to_string(),
            None
        ).await;
        
        let woke = manager.wake_up("tab-1").await;
        assert!(woke.is_some());
        assert!(!manager.is_hibernated("tab-1").await);
    }

    #[tokio::test]
    async fn test_memory_savings() {
        let manager = TabHibernationManager::new();
        manager.hibernate(
            "tab-1".to_string(),
            "https://youtube.com".to_string(),
            "YouTube".to_string(),
            None
        ).await;
        
        let saved = manager.get_memory_savings().await;
        assert!(saved > 100 * 1024 * 1024); // Should be > 100MB for YouTube
    }

    #[tokio::test]
    async fn test_settings() {
        let manager = TabHibernationManager::new();
        let settings = HibernationSettings {
            auto_hibernate: false,
            idle_timeout_seconds: 600,
            max_hibernated_tabs: 20,
            preserve_form_data: false,
            preserve_scroll_position: false,
        };
        
        manager.update_settings(settings).await;
        let loaded = manager.get_settings().await;
        assert!(!loaded.auto_hibernate);
        assert_eq!(loaded.idle_timeout_seconds, 600);
    }

    #[tokio::test]
    async fn test_stats() {
        let manager = TabHibernationManager::new();
        manager.hibernate("tab-1".to_string(), "https://example.com".to_string(), "Example".to_string(), None).await;
        
        let stats = manager.get_stats().await;
        assert_eq!(stats.hibernated_count, 1);
        assert!(stats.total_memory_saved > 0);
    }
}