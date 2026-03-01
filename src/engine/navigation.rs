//! Navigation System
//! 
//! Navigation history management:
//! - Back/Forward navigation
//! - History entry management
//! - Session history
//! - Navigation events

use anyhow::{Context, Result};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::core::kernel::VantisKernel;

/// Navigation entry representing a page in history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationEntry {
    /// Unique identifier
    pub id: String,
    /// URL of the page
    pub url: String,
    /// Page title
    pub title: String,
    /// Timestamp of visit
    pub timestamp: i64,
    /// Scroll position (for restoration)
    pub scroll_position: (f32, f32),
    /// Form data (for restoration)
    pub form_data: Option<serde_json::Value>,
    /// User agent
    pub user_agent: String,
}

impl NavigationEntry {
    /// Create a new navigation entry
    pub fn new(url: String, title: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            url,
            title,
            timestamp: chrono::Utc::now().timestamp(),
            scroll_position: (0.0, 0.0),
            form_data: None,
            user_agent: "VantisWeb/0.2.0".to_string(),
        }
    }
}

/// Navigation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationState {
    /// Current position in history
    pub current_index: usize,
    /// Can go back
    pub can_go_back: bool,
    /// Can go forward
    pub can_go_forward: bool,
    /// Total entries
    pub total_entries: usize,
}

/// Navigation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NavigationEvent {
    /// Navigation started
    Started { url: String },
    /// Navigation completed
    Completed { url: String },
    /// Navigation failed
    Failed { url: String, error: String },
    /// Navigation to new entry
    NewEntry { entry: NavigationEntry },
    /// Navigation back
    Back { from_url: String, to_url: String },
    /// Navigation forward
    Forward { from_url: String, to_url: String },
    /// History cleared
    Cleared,
}

/// Navigation History Manager
pub struct NavigationManager {
    kernel: Arc<VantisKernel>,
    /// History entries
    history: Arc<RwLock<VecDeque<NavigationEntry>>>,
    /// Current position in history
    current_index: Arc<RwLock<usize>>,
    /// Maximum history entries
    max_entries: usize,
    /// Navigation event listeners
    listeners: Arc<RwLock<Vec<Box<dyn Fn(NavigationEvent) + Send + Sync>>>>,
}

impl NavigationManager {
    /// Create a new navigation manager
    pub fn new(kernel: Arc<VantisKernel>) -> Self {
        info!("Initializing Navigation Manager...");

        Self {
            kernel,
            history: Arc::new(RwLock::new(VecDeque::with_capacity(100))),
            current_index: Arc::new(RwLock::new(0)),
            max_entries: 100,
            listeners: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Navigate to a new URL
    pub async fn navigate(&self, url: String, title: Option<String>) -> Result<()> {
        info!("Navigating to: {}", url);
        
        // Emit start event
        self.emit_event(NavigationEvent::Started {
            url: url.clone(),
        }).await;

        // Create new entry
        let entry = NavigationEntry::new(
            url.clone(),
            title.unwrap_or_else(|| "Untitled".to_string()),
        );

        // Remove all entries after current index
        {
            let mut history = self.history.write().await;
            let current_index = *self.current_index.read().await;
            
            while history.len() > current_index + 1 {
                history.pop_back();
            }

            // Add new entry
            history.push_back(entry.clone());

            // Trim if too many entries
            if history.len() > self.max_entries {
                history.pop_front();
            }
        }

        // Update current index
        *self.current_index.write().await = self.history.read().await.len() - 1;

        // Emit new entry event
        self.emit_event(NavigationEvent::NewEntry { entry }).await;

        // Emit completed event
        self.emit_event(NavigationEvent::Completed {
            url: url.clone(),
        }).await;

        debug!("Navigation completed to: {}", url);

        Ok(())
    }

    /// Go back in history
    pub async fn go_back(&self) -> Result<Option<NavigationEntry>> {
        let current_index = *self.current_index.read().await;
        
        if current_index == 0 {
            warn!("Cannot go back - at beginning of history");
            return Ok(None);
        }

        let from_url = self.get_current_entry().await.map(|e| e.url.clone());
        
        // Update index
        *self.current_index.write().await = current_index - 1;
        
        let to_entry = self.get_current_entry().await;
        let to_entry_clone = to_entry.clone();

        if let (Some(from), Some(ref to)) = (from_url, to_entry_clone) {
            self.emit_event(NavigationEvent::Back {
                from_url: from,
                to_url: to.url.clone(),
            }).await;
        }

        info!("Navigated back to: {:?}", to_entry.as_ref().map(|e| &e.url));
        Ok(to_entry)
    }

    /// Go forward in history
    pub async fn go_forward(&self) -> Result<Option<NavigationEntry>> {
        let current_index = *self.current_index.read().await;
        let history_len = self.history.read().await.len();
        
        if current_index >= history_len - 1 {
            warn!("Cannot go forward - at end of history");
            return Ok(None);
        }

        let from_url = self.get_current_entry().await.map(|e| e.url.clone());
        
        // Update index
        *self.current_index.write().await = current_index + 1;
        
        let to_entry = self.get_current_entry().await;
        let to_entry_clone = to_entry.clone();

        if let (Some(from), Some(ref to)) = (from_url, to_entry_clone) {
            self.emit_event(NavigationEvent::Forward {
                from_url: from,
                to_url: to.url.clone(),
            }).await;
        }

        info!("Navigated forward to: {:?}", to_entry.as_ref().map(|e| &e.url));
        Ok(to_entry)
    }

    /// Get current navigation entry
    pub async fn get_current_entry(&self) -> Option<NavigationEntry> {
        let current_index = *self.current_index.read().await;
        let history = self.history.read().await;
        
        if current_index < history.len() {
            Some(history[current_index].clone())
        } else {
            None
        }
    }

    /// Get navigation state
    pub async fn get_state(&self) -> NavigationState {
        let current_index = *self.current_index.read().await;
        let history_len = self.history.read().await.len();
        
        NavigationState {
            current_index,
            can_go_back: current_index > 0,
            can_go_forward: current_index < history_len - 1,
            total_entries: history_len,
        }
    }

    /// Get all history entries
    pub async fn get_history(&self) -> Vec<NavigationEntry> {
        self.history.read().await.iter().cloned().collect()
    }

    /// Clear history
    pub async fn clear_history(&self) -> Result<()> {
        info!("Clearing navigation history");
        
        self.history.write().await.clear();
        *self.current_index.write().await = 0;
        
        self.emit_event(NavigationEvent::Cleared).await;
        
        Ok(())
    }

    /// Go to specific index in history
    pub async fn go_to_index(&self, index: usize) -> Result<Option<NavigationEntry>> {
        let history_len = self.history.read().await.len();
        
        if index >= history_len {
            warn!("Invalid history index: {}", index);
            return Ok(None);
        }

        *self.current_index.write().await = index;
        
        let entry = self.get_current_entry().await;
        
        info!("Navigated to index {}: {:?}", index, entry.as_ref().map(|e| &e.url));
        Ok(entry)
    }

    /// Add navigation event listener
    pub async fn add_listener<F>(&self, listener: F)
    where
        F: Fn(NavigationEvent) + Send + Sync + 'static,
    {
        self.listeners.write().await.push(Box::new(listener));
    }

    /// Remove all listeners
    pub async fn clear_listeners(&self) {
        self.listeners.write().await.clear();
    }

    /// Emit navigation event
    async fn emit_event(&self, event: NavigationEvent) {
        let listeners = self.listeners.read().await;
        for listener in listeners.iter() {
            listener(event.clone());
        }
    }

    /// Get history length
    pub async fn len(&self) -> usize {
        self.history.read().await.len()
    }

    /// Check if history is empty
    pub async fn is_empty(&self) -> bool {
        self.history.read().await.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_test_manager() -> NavigationManager {
        let kernel = Arc::new(VantisKernel::new().await.unwrap());
        NavigationManager::new(kernel)
    }

    #[tokio::test]
    async fn test_navigation_manager_creation() {
        let manager = create_test_manager().await;
        
        assert_eq!(manager.len().await, 0);
        assert!(manager.is_empty().await);
    }

    #[tokio::test]
    async fn test_navigate() {
        let manager = create_test_manager().await;
        
        manager.navigate(
            "https://example.com".to_string(),
            Some("Example".to_string()),
        ).await.unwrap();
        
        assert_eq!(manager.len().await, 1);
        
        let entry = manager.get_current_entry().await;
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().url, "https://example.com");
    }

    #[tokio::test]
    async fn test_navigate_multiple() {
        let manager = create_test_manager().await;
        
        manager.navigate(
            "https://example.com".to_string(),
            None,
        ).await.unwrap();
        
        manager.navigate(
            "https://example.com/page1".to_string(),
            None,
        ).await.unwrap();
        
        manager.navigate(
            "https://example.com/page2".to_string(),
            None,
        ).await.unwrap();
        
        assert_eq!(manager.len().await, 3);
        
        let state = manager.get_state().await;
        assert_eq!(state.current_index, 2);
        assert!(state.can_go_back);
        assert!(!state.can_go_forward);
    }

    #[tokio::test]
    async fn test_go_back() {
        let manager = create_test_manager().await;
        
        manager.navigate(
            "https://example.com".to_string(),
            None,
        ).await.unwrap();
        
        manager.navigate(
            "https://example.com/page1".to_string(),
            None,
        ).await.unwrap();
        
        let entry = manager.go_back().await.unwrap();
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().url, "https://example.com");
        
        let state = manager.get_state().await;
        assert_eq!(state.current_index, 0);
        assert!(!state.can_go_back);
        assert!(state.can_go_forward);
    }

    #[tokio::test]
    async fn test_go_forward() {
        let manager = create_test_manager().await;
        
        manager.navigate(
            "https://example.com".to_string(),
            None,
        ).await.unwrap();
        
        manager.navigate(
            "https://example.com/page1".to_string(),
            None,
        ).await.unwrap();
        
        manager.go_back().await.unwrap();
        
        let entry = manager.go_forward().await.unwrap();
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().url, "https://example.com/page1");
    }

    #[tokio::test]
    async fn test_clear_history() {
        let manager = create_test_manager().await;
        
        manager.navigate(
            "https://example.com".to_string(),
            None,
        ).await.unwrap();
        
        manager.navigate(
            "https://example.com/page1".to_string(),
            None,
        ).await.unwrap();
        
        manager.clear_history().await.unwrap();
        
        assert_eq!(manager.len().await, 0);
        assert!(manager.is_empty().await);
    }

    #[tokio::test]
    async fn test_navigate_removes_forward_history() {
        let manager = create_test_manager().await;
        
        manager.navigate(
            "https://example.com".to_string(),
            None,
        ).await.unwrap();
        
        manager.navigate(
            "https://example.com/page1".to_string(),
            None,
        ).await.unwrap();
        
        manager.go_back().await.unwrap();
        
        // Navigate to new URL should remove forward history
        manager.navigate(
            "https://example.com/new".to_string(),
            None,
        ).await.unwrap();
        
        assert_eq!(manager.len().await, 2);
        assert!(!manager.get_state().await.can_go_forward);
    }

    #[tokio::test]
    async fn test_navigation_event_listener() {
        let manager = create_test_manager().await;
        let events = Arc::new(RwLock::new(Vec::new()));
        let events_clone = events.clone();
        
        manager.add_listener(move |event| {
            let mut events = events_clone.blocking_write();
            events.push(event);
        }).await;
        
        manager.navigate(
            "https://example.com".to_string(),
            None,
        ).await.unwrap();
        
        let events = events.read().await;
        assert!(events.len() >= 2); // Started and Completed
    }
}
