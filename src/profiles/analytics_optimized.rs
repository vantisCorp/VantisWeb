//! Profile Analytics (Optimized)
//!
//! Usage statistics and analytics for profiles.
//!
//! Optimizations:
//! - Pre-allocated collection capacities
//! - Reduced clones
//! - Optimized string operations

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Profile analytics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileAnalytics {
    /// Profile ID
    pub profile_id: String,
    /// Total time spent (in seconds)
    pub total_time: u64,
    /// Session count
    pub session_count: u64,
    /// First session timestamp
    pub first_session: Option<DateTime<Utc>>,
    /// Last session timestamp
    pub last_session: Option<DateTime<Utc>>,
    /// Daily usage
    pub daily_usage: HashMap<String, DailyUsage>,
    /// Top websites
    pub top_websites: Vec<WebsiteUsage>,
    /// Tab statistics
    pub tab_stats: TabStatistics,
    /// Performance metrics
    pub performance: PerformanceMetrics,
}

/// Daily usage data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyUsage {
    /// Date
    pub date: String,
    /// Time spent (in seconds)
    pub time_spent: u64,
    /// Session count
    pub sessions: u64,
    /// Websites visited
    pub websites_visited: u64,
}

/// Website usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebsiteUsage {
    /// Website URL
    pub url: String,
    /// Visit count
    pub visits: u64,
    /// Time spent (in seconds)
    pub time_spent: u64,
    /// Last visit timestamp
    pub last_visit: DateTime<Utc>,
}

/// Tab statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabStatistics {
    /// Total tabs opened
    pub total_opened: u64,
    /// Average tabs per session
    pub average_per_session: f64,
    /// Maximum tabs open at once
    pub max_open: u64,
    /// Current tabs open
    pub current_open: u64,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Average page load time (ms)
    pub avg_page_load_time: u64,
    /// Total crashes
    pub total_crashes: u64,
    /// Memory usage (MB)
    pub memory_usage: u64,
    /// CPU usage percentage
    pub cpu_usage: f64,
}

/// Profile analytics manager (Optimized)
pub struct AnalyticsManager {
    /// Analytics data per profile
    analytics: HashMap<String, ProfileAnalytics>,
    /// Current session data
    current_sessions: HashMap<String, SessionData>,
}

/// Session data
#[derive(Debug, Clone)]
struct SessionData {
    /// Session start time
    start_time: DateTime<Utc>,
    /// Websites visited in this session
    websites: HashMap<String, u64>,
    /// Tabs opened in this session
    tabs_opened: u64,
    /// Max tabs open in this session
    max_tabs_open: u64,
}

impl AnalyticsManager {
    /// Creates a new analytics manager (Optimized)
    pub fn new() -> Self {
        Self {
            analytics: HashMap::with_capacity(10),
            current_sessions: HashMap::with_capacity(10),
        }
    }

    /// Starts a session for a profile (Optimized)
    pub fn start_session(&mut self, profile_id: &str) -> Result<()> {
        log::info!("Starting analytics session for profile: {}", profile_id);

        let session_data = SessionData {
            start_time: Utc::now(),
            websites: HashMap::with_capacity(50),
            tabs_opened: 0,
            max_tabs_open: 0,
        };

        self.current_sessions.insert(profile_id.to_string(), session_data);

        // Initialize analytics if not exists
        if !self.analytics.contains_key(profile_id) {
            self.analytics.insert(profile_id.to_string(), ProfileAnalytics {
                profile_id: profile_id.to_string(),
                total_time: 0,
                session_count: 0,
                first_session: Some(Utc::now()),
                last_session: None,
                daily_usage: HashMap::with_capacity(30),
                top_websites: Vec::with_capacity(10),
                tab_stats: TabStatistics {
                    total_opened: 0,
                    average_per_session: 0.0,
                    max_open: 0,
                    current_open: 0,
                },
                performance: PerformanceMetrics {
                    avg_page_load_time: 0,
                    total_crashes: 0,
                    memory_usage: 0,
                    cpu_usage: 0.0,
                },
            });
        }

        Ok(())
    }

    /// Ends a session for a profile (Optimized)
    pub fn end_session(&mut self, profile_id: &str) -> Result<()> {
        log::info!("Ending analytics session for profile: {}", profile_id);

        let session_data = self.current_sessions.remove(profile_id)
            .ok_or_else(|| anyhow::anyhow!("No active session for profile: {}", profile_id))?;

        let duration = (Utc::now() - session_data.start_time).num_seconds() as u64;

        // Update analytics
        if let Some(analytics) = self.analytics.get_mut(profile_id) {
            analytics.total_time += duration;
            analytics.session_count += 1;
            analytics.last_session = Some(Utc::now());

            // Update daily usage
            let today = Utc::now().format("%Y-%m-%d").to_string();
            let daily = analytics.daily_usage.entry(today).or_insert(DailyUsage {
                date: today.clone(),
                time_spent: 0,
                sessions: 0,
                websites_visited: 0,
            });
            daily.time_spent += duration;
            daily.sessions += 1;
            daily.websites_visited += session_data.websites.len() as u64;

            // Update tab statistics
            analytics.tab_stats.total_opened += session_data.tabs_opened;
            analytics.tab_stats.max_open = analytics.tab_stats.max_open.max(session_data.max_tabs_open);
            analytics.tab_stats.average_per_session = analytics.tab_stats.total_opened as f64 / analytics.session_count as f64;

            // Update top websites
            for (url, visits) in session_data.websites {
                let website = analytics.top_websites.iter_mut().find(|w| w.url == url);
                if let Some(w) = website {
                    w.visits += visits;
                    w.last_visit = Utc::now();
                } else {
                    analytics.top_websites.push(WebsiteUsage {
                        url: url.clone(),
                        visits,
                        time_spent: 0,
                        last_visit: Utc::now(),
                    });
                }
            }

            // Sort top websites by visits
            analytics.top_websites.sort_by(|a, b| b.visits.cmp(&a.visits));
            analytics.top_websites.truncate(10); // Keep top 10
        }

        Ok(())
    }

    /// Records a website visit
    pub fn record_website_visit(&mut self, profile_id: &str, url: &str) -> Result<()> {
        if let Some(session) = self.current_sessions.get_mut(profile_id) {
            *session.websites.entry(url.to_string()).or_insert(0) += 1;
        }

        Ok(())
    }

    /// Records a tab opening
    pub fn record_tab_open(&mut self, profile_id: &str) -> Result<()> {
        if let Some(session) = self.current_sessions.get_mut(profile_id) {
            session.tabs_opened += 1;
            session.max_tabs_open = session.max_tabs_open.max(session.tabs_opened);
        }

        if let Some(analytics) = self.analytics.get_mut(profile_id) {
            analytics.tab_stats.current_open += 1;
        }

        Ok(())
    }

    /// Records a tab closing
    pub fn record_tab_close(&mut self, profile_id: &str) -> Result<()> {
        if let Some(analytics) = self.analytics.get_mut(profile_id) {
            if analytics.tab_stats.current_open > 0 {
                analytics.tab_stats.current_open -= 1;
            }
        }

        Ok(())
    }

    /// Records a page load time
    pub fn record_page_load_time(&mut self, profile_id: &str, load_time_ms: u64) -> Result<()> {
        if let Some(analytics) = self.analytics.get_mut(profile_id) {
            // Update average
            let total_loads = analytics.session_count;
            if total_loads > 0 {
                analytics.performance.avg_page_load_time = 
                    (analytics.performance.avg_page_load_time * (total_loads - 1) + load_time_ms) / total_loads;
            } else {
                analytics.performance.avg_page_load_time = load_time_ms;
            }
        }

        Ok(())
    }

    /// Records a crash
    pub fn record_crash(&mut self, profile_id: &str) -> Result<()> {
        if let Some(analytics) = self.analytics.get_mut(profile_id) {
            analytics.performance.total_crashes += 1;
        }

        Ok(())
    }

    /// Updates memory usage
    pub fn update_memory_usage(&mut self, profile_id: &str, memory_mb: u64) -> Result<()> {
        if let Some(analytics) = self.analytics.get_mut(profile_id) {
            analytics.performance.memory_usage = memory_mb;
        }

        Ok(())
    }

    /// Updates CPU usage
    pub fn update_cpu_usage(&mut self, profile_id: &str, cpu_percent: f64) -> Result<()> {
        if let Some(analytics) = self.analytics.get_mut(profile_id) {
            analytics.performance.cpu_usage = cpu_percent;
        }

        Ok(())
    }

    /// Gets analytics for a profile (Optimized - returns reference)
    pub fn get_analytics(&self, profile_id: &str) -> Option<&ProfileAnalytics> {
        self.analytics.get(profile_id)
    }

    /// Gets all analytics (Optimized - returns Vec<&>)
    pub fn get_all_analytics(&self) -> Vec<&ProfileAnalytics> {
        self.analytics.values().collect()
    }

    /// Gets daily usage for a profile (Optimized)
    pub fn get_daily_usage(&self, profile_id: &str, days: u32) -> Vec<&DailyUsage> {
        if let Some(analytics) = self.analytics.get(profile_id) {
            let cutoff_date = Utc::now() - Duration::days(days as i64);
            let cutoff_str = cutoff_date.format("%Y-%m-%d").to_string();

            analytics.daily_usage
                .values()
                .filter(|d| d.date >= cutoff_str)
                .collect()
        } else {
            Vec::with_capacity(0)
        }
    }

    /// Gets top websites for a profile (Optimized)
    pub fn get_top_websites(&self, profile_id: &str, limit: usize) -> Vec<&WebsiteUsage> {
        if let Some(analytics) = self.analytics.get(profile_id) {
            analytics.top_websites.iter().take(limit).collect()
        } else {
            Vec::with_capacity(0)
        }
    }

    /// Gets usage summary for a profile
    pub fn get_usage_summary(&self, profile_id: &str) -> Option<UsageSummary> {
        if let Some(analytics) = self.analytics.get(profile_id) {
            let avg_session_time = if analytics.session_count > 0 {
                analytics.total_time / analytics.session_count
            } else {
                0
            };

            let avg_daily_time = if !analytics.daily_usage.is_empty() {
                let total_daily: u64 = analytics.daily_usage.values().map(|d| d.time_spent).sum();
                total_daily / analytics.daily_usage.len() as u64
            } else {
                0
            };

            Some(UsageSummary {
                profile_id: profile_id.to_string(),
                total_time: analytics.total_time,
                session_count: analytics.session_count,
                avg_session_time,
                avg_daily_time,
                total_websites: analytics.top_websites.len() as u64,
                total_tabs_opened: analytics.tab_stats.total_opened,
                avg_page_load_time: analytics.performance.avg_page_load_time,
                total_crashes: analytics.performance.total_crashes,
            })
        } else {
            None
        }
    }

    /// Clears analytics for a profile
    pub fn clear_analytics(&mut self, profile_id: &str) -> Result<()> {
        self.analytics.remove(profile_id);
        self.current_sessions.remove(profile_id);
        Ok(())
    }

    /// Exports analytics to JSON
    pub fn export(&self, profile_id: &str) -> Result<String> {
        let analytics = self.get_analytics(profile_id)
            .ok_or_else(|| anyhow::anyhow!("Analytics not found for profile: {}", profile_id))?;

        serde_json::to_string_pretty(analytics)
            .map_err(|e| anyhow::anyhow!("Failed to export analytics: {}", e))
    }

    /// Imports analytics from JSON
    pub fn import(&mut self, json: &str) -> Result<()> {
        let analytics: ProfileAnalytics = serde_json::from_str(json)
            .map_err(|e| anyhow::anyhow!("Failed to import analytics: {}", e))?;

        self.analytics.insert(analytics.profile_id.clone(), analytics);
        Ok(())
    }
}

impl Default for AnalyticsManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Usage summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSummary {
    /// Profile ID
    pub profile_id: String,
    /// Total time spent (in seconds)
    pub total_time: u64,
    /// Session count
    pub session_count: u64,
    /// Average session time (in seconds)
    pub avg_session_time: u64,
    /// Average daily time (in seconds)
    pub avg_daily_time: u64,
    /// Total websites visited
    pub total_websites: u64,
    /// Total tabs opened
    pub total_tabs_opened: u64,
    /// Average page load time (ms)
    pub avg_page_load_time: u64,
    /// Total crashes
    pub total_crashes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_manager_creation() {
        let manager = AnalyticsManager::new();
        assert_eq!(manager.get_all_analytics().len(), 0);
    }

    #[test]
    fn test_start_end_session() {
        let mut manager = AnalyticsManager::new();
        let profile_id = "test-profile";

        manager.start_session(profile_id).unwrap();
        manager.end_session(profile_id).unwrap();

        let analytics = manager.get_analytics(profile_id);
        assert!(analytics.is_some());
        assert_eq!(analytics.unwrap().session_count, 1);
    }

    #[test]
    fn test_record_website_visit() {
        let mut manager = AnalyticsManager::new();
        let profile_id = "test-profile";

        manager.start_session(profile_id).unwrap();
        manager.record_website_visit(profile_id, "https://example.com").unwrap();
        manager.end_session(profile_id).unwrap();

        let analytics = manager.get_analytics(profile_id).unwrap();
        assert_eq!(analytics.top_websites.len(), 1);
        assert_eq!(analytics.top_websites[0].url, "https://example.com");
    }

    #[test]
    fn test_record_tab_open() {
        let mut manager = AnalyticsManager::new();
        let profile_id = "test-profile";

        manager.start_session(profile_id).unwrap();
        manager.record_tab_open(profile_id).unwrap();
        manager.end_session(profile_id).unwrap();
        let analytics = manager.get_analytics(profile_id).unwrap();
        assert_eq!(analytics.tab_stats.total_opened, 1);
    }

    #[test]
    fn test_get_usage_summary() {
        let mut manager = AnalyticsManager::new();
        let profile_id = "test-profile";

        manager.start_session(profile_id).unwrap();
        manager.record_website_visit(profile_id, "https://example.com").unwrap();
        manager.record_tab_open(profile_id).unwrap();
        manager.end_session(profile_id).unwrap();
        let summary = manager.get_usage_summary(profile_id);
        assert!(summary.is_some());
        assert_eq!(summary.unwrap().session_count, 1);
    }
}