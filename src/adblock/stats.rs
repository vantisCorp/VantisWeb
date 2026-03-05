//! Block Statistics for VantisWeb Ad Blocker
//! 
//! This module tracks and manages blocking statistics including:
//! - Real-time blocking counters
//! - Historical statistics with time-series data
//! - Bandwidth and time savings calculations
//! - Per-domain breakdown
//! - Performance metrics

use std::collections::{HashMap, BTreeMap};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration, NaiveDate};

/// Statistics manager for tracking blocking data
pub struct BlockStatistics {
    /// Current session statistics
    session: Arc<RwLock<SessionStats>>,
    /// Historical daily statistics
    daily: Arc<RwLock<BTreeMap<NaiveDate, DailyStats>>>,
    /// Per-domain statistics
    domains: Arc<RwLock<HashMap<String, DomainStats>>>,
    /// Total lifetime statistics
    lifetime: Arc<RwLock<LifetimeStats>>,
    /// Configuration
    config: Arc<RwLock<StatsConfig>>,
}

/// Session statistics (current browsing session)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    /// Session start time
    pub started_at: DateTime<Utc>,
    /// Total items blocked this session
    pub total_blocked: u64,
    /// Ads blocked
    pub ads_blocked: u64,
    /// Trackers blocked
    pub trackers_blocked: u64,
    /// Scripts blocked
    pub scripts_blocked: u64,
    /// Images blocked
    pub images_blocked: u64,
    /// Other resources blocked
    pub other_blocked: u64,
    /// Threats blocked
    pub threats_blocked: u64,
    /// Bandwidth saved (bytes)
    pub bandwidth_saved: u64,
    /// Time saved (ms)
    pub time_saved_ms: u64,
    /// Pages visited
    pub pages_visited: u64,
    /// Requests processed
    pub requests_processed: u64,
}

impl Default for SessionStats {
    fn default() -> Self {
        Self {
            started_at: Utc::now(),
            total_blocked: 0,
            ads_blocked: 0,
            trackers_blocked: 0,
            scripts_blocked: 0,
            images_blocked: 0,
            other_blocked: 0,
            threats_blocked: 0,
            bandwidth_saved: 0,
            time_saved_ms: 0,
            pages_visited: 0,
            requests_processed: 0,
        }
    }
}

/// Daily statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStats {
    /// Date
    pub date: NaiveDate,
    /// Total items blocked
    pub total_blocked: u64,
    /// Ads blocked
    pub ads_blocked: u64,
    /// Trackers blocked
    pub trackers_blocked: u64,
    /// Threats blocked
    pub threats_blocked: u64,
    /// Bandwidth saved
    pub bandwidth_saved: u64,
    /// Time saved (seconds)
    pub time_saved_seconds: u64,
    /// Unique domains
    pub unique_domains: u64,
    /// Page views
    pub page_views: u64,
}

/// Per-domain statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainStats {
    /// Domain name
    pub domain: String,
    /// Times blocked
    pub blocked_count: u64,
    /// Category of blocking
    pub category: BlockCategory,
    /// Last seen
    pub last_seen: DateTime<Utc>,
    /// First seen
    pub first_seen: DateTime<Utc>,
    /// Bandwidth saved
    pub bandwidth_saved: u64,
}

/// Block categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlockCategory {
    /// Advertisement
    Ad,
    /// Tracker
    Tracker,
    /// Analytics
    Analytics,
    /// Social widget
    Social,
    /// Malware/threat
    Malware,
    /// Cryptominer
    CryptoMiner,
    /// Fingerprinting
    Fingerprinting,
    /// Annoyance
    Annoyance,
    /// Other
    Other,
}

/// Lifetime statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifetimeStats {
    /// Total items blocked all time
    pub total_blocked: u64,
    /// Total ads blocked
    pub total_ads_blocked: u64,
    /// Total trackers blocked
    pub total_trackers_blocked: u64,
    /// Total threats blocked
    pub total_threats_blocked: u64,
    /// Total bandwidth saved (bytes)
    pub total_bandwidth_saved: u64,
    /// Total time saved (seconds)
    pub total_time_saved_seconds: u64,
    /// Installation date
    pub installed_at: DateTime<Utc>,
    /// Days active
    pub days_active: u64,
    /// Most blocked domain
    pub top_domain: Option<String>,
    /// Most blocked domain count
    pub top_domain_count: u64,
}

impl Default for LifetimeStats {
    fn default() -> Self {
        Self {
            total_blocked: 0,
            total_ads_blocked: 0,
            total_trackers_blocked: 0,
            total_threats_blocked: 0,
            total_bandwidth_saved: 0,
            total_time_saved_seconds: 0,
            installed_at: Utc::now(),
            days_active: 0,
            top_domain: None,
            top_domain_count: 0,
        }
    }
}

/// Statistics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsConfig {
    /// Enable statistics tracking
    pub enabled: bool,
    /// Retention period for daily stats (days)
    pub retention_days: u32,
    /// Track per-domain stats
    pub track_domains: bool,
    /// Average ad size for bandwidth calculations (bytes)
    pub avg_ad_size: u32,
    /// Average time saved per blocked ad (ms)
    pub avg_time_saved_ms: u32,
    /// Maximum domains to track
    pub max_domains: usize,
    /// Send anonymous usage stats
    pub send_anonymous_stats: bool,
}

impl Default for StatsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            retention_days: 90,
            track_domains: true,
            avg_ad_size: 50000, // 50KB
            avg_time_saved_ms: 50,
            max_domains: 1000,
            send_anonymous_stats: false,
        }
    }
}

/// Blocking event for real-time tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockEvent {
    /// Event ID
    pub id: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Blocked URL
    pub url: String,
    /// Domain
    pub domain: String,
    /// Resource type
    pub resource_type: ResourceType,
    /// Block category
    pub category: BlockCategory,
    /// Page URL where blocked
    pub page_url: String,
    /// Estimated size (bytes)
    pub estimated_size: u64,
    /// Filter rule matched
    pub filter_rule: Option<String>,
}

/// Resource types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    Script,
    Image,
    Stylesheet,
    Object,
    XmlHttpRequest,
    SubDocument,
    Document,
    Font,
    Media,
    WebSocket,
    Ping,
    Other,
}

/// Statistics summary for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsSummary {
    /// Session stats
    pub session: SessionStats,
    /// Today's stats
    pub today: DailyStats,
    /// Last 7 days total
    pub last_7_days: AggregatedStats,
    /// Last 30 days total
    pub last_30_days: AggregatedStats,
    /// Lifetime stats
    pub lifetime: LifetimeStats,
    /// Top blocked domains
    pub top_domains: Vec<DomainStats>,
    /// Category breakdown
    pub by_category: HashMap<BlockCategory, u64>,
}

/// Aggregated statistics for a time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedStats {
    /// Total blocked
    pub total_blocked: u64,
    /// Ads blocked
    pub ads_blocked: u64,
    /// Trackers blocked
    pub trackers_blocked: u64,
    /// Threats blocked
    pub threats_blocked: u64,
    /// Bandwidth saved
    pub bandwidth_saved: u64,
    /// Time saved (seconds)
    pub time_saved_seconds: u64,
    /// Average per day
    pub avg_per_day: f64,
    /// Number of days
    pub days: u32,
}

impl BlockStatistics {
    /// Create a new statistics manager
    pub fn new() -> Self {
        Self {
            session: Arc::new(RwLock::new(SessionStats::default())),
            daily: Arc::new(RwLock::new(BTreeMap::new())),
            domains: Arc::new(RwLock::new(HashMap::new())),
            lifetime: Arc::new(RwLock::new(LifetimeStats::default())),
            config: Arc::new(RwLock::new(StatsConfig::default())),
        }
    }

    /// Record a blocked item
    pub async fn record_block(&self, event: BlockEvent) {
        let config = self.config.read().await;
        if !config.enabled {
            return;
        }
        drop(config);
        
        // Update session stats
        {
            let mut session = self.session.write().await;
            session.total_blocked += 1;
            session.bandwidth_saved += event.estimated_size;
            
            match event.category {
                BlockCategory::Ad => session.ads_blocked += 1,
                BlockCategory::Tracker => session.trackers_blocked += 1,
                BlockCategory::Malware | BlockCategory::CryptoMiner | BlockCategory::Fingerprinting => {
                    session.threats_blocked += 1;
                }
                BlockCategory::Analytics => session.trackers_blocked += 1,
                _ => session.other_blocked += 1,
            }
            
            match event.resource_type {
                ResourceType::Script => session.scripts_blocked += 1,
                ResourceType::Image => session.images_blocked += 1,
                _ => {}
            }
        }
        
        // Update daily stats
        let today = Utc::now().date_naive();
        {
            let mut daily = self.daily.write().await;
            let today_stats = daily.entry(today).or_insert_with(|| DailyStats {
                date: today,
                total_blocked: 0,
                ads_blocked: 0,
                trackers_blocked: 0,
                threats_blocked: 0,
                bandwidth_saved: 0,
                time_saved_seconds: 0,
                unique_domains: 0,
                page_views: 0,
            });
            
            today_stats.total_blocked += 1;
            today_stats.bandwidth_saved += event.estimated_size;
            
            match event.category {
                BlockCategory::Ad => today_stats.ads_blocked += 1,
                BlockCategory::Tracker | BlockCategory::Analytics => today_stats.trackers_blocked += 1,
                BlockCategory::Malware | BlockCategory::CryptoMiner | BlockCategory::Fingerprinting => {
                    today_stats.threats_blocked += 1;
                }
                _ => {}
            }
        }
        
        // Update domain stats
        let config = self.config.read().await;
        if config.track_domains {
            drop(config);
            let mut domains = self.domains.write().await;
            let domain_entry = domains.entry(event.domain.clone()).or_insert_with(|| DomainStats {
                domain: event.domain.clone(),
                blocked_count: 0,
                category: event.category,
                first_seen: Utc::now(),
                last_seen: Utc::now(),
                bandwidth_saved: 0,
            });
            
            domain_entry.blocked_count += 1;
            domain_entry.last_seen = Utc::now();
            domain_entry.bandwidth_saved += event.estimated_size;
            
            // Enforce max domains limit
            let config = self.config.read().await;
            if domains.len() > config.max_domains {
                // Remove oldest/least used
                if let Some((oldest_domain, _)) = domains.iter()
                    .min_by_key(|(_, s)| s.blocked_count)
                    .map(|(d, s)| (d.clone(), s.blocked_count))
                {
                    domains.remove(&oldest_domain);
                }
            }
        }
        
        // Update lifetime stats
        {
            let mut lifetime = self.lifetime.write().await;
            lifetime.total_blocked += 1;
            lifetime.total_bandwidth_saved += event.estimated_size;
            
            match event.category {
                BlockCategory::Ad => lifetime.total_ads_blocked += 1,
                BlockCategory::Tracker | BlockCategory::Analytics => lifetime.total_trackers_blocked += 1,
                BlockCategory::Malware | BlockCategory::CryptoMiner | BlockCategory::Fingerprinting => {
                    lifetime.total_threats_blocked += 1;
                }
                _ => {}
            }
            
            // Update top domain
            let domains = self.domains.read().await;
            if let Some((top_domain, top_count)) = domains.iter()
                .max_by_key(|(_, s)| s.blocked_count)
                .map(|(d, s)| (d.clone(), s.blocked_count))
            {
                lifetime.top_domain = Some(top_domain);
                lifetime.top_domain_count = top_count;
            }
        }
    }

    /// Record a page view
    pub async fn record_page_view(&self) {
        let mut session = self.session.write().await;
        session.pages_visited += 1;
        
        let today = Utc::now().date_naive();
        let mut daily = self.daily.write().await;
        let today_stats = daily.entry(today).or_insert_with(|| DailyStats {
            date: today,
            total_blocked: 0,
            ads_blocked: 0,
            trackers_blocked: 0,
            threats_blocked: 0,
            bandwidth_saved: 0,
            time_saved_seconds: 0,
            unique_domains: 0,
            page_views: 0,
        });
        today_stats.page_views += 1;
    }

    /// Record a processed request
    pub async fn record_request(&self) {
        let mut session = self.session.write().await;
        session.requests_processed += 1;
    }

    /// Get current session statistics
    pub async fn get_session_stats(&self) -> SessionStats {
        self.session.read().await.clone()
    }

    /// Get today's statistics
    pub async fn get_today_stats(&self) -> DailyStats {
        let today = Utc::now().date_naive();
        self.daily.read().await
            .get(&today)
            .cloned()
            .unwrap_or_else(|| DailyStats {
                date: today,
                total_blocked: 0,
                ads_blocked: 0,
                trackers_blocked: 0,
                threats_blocked: 0,
                bandwidth_saved: 0,
                time_saved_seconds: 0,
                unique_domains: 0,
                page_views: 0,
            })
    }

    /// Get daily statistics for a range
    pub async fn get_daily_stats(&self, from: NaiveDate, to: NaiveDate) -> Vec<DailyStats> {
        self.daily.read().await
            .range(from..=to)
            .map(|(_, v)| v.clone())
            .collect()
    }

    /// Get top blocked domains
    pub async fn get_top_domains(&self, limit: usize) -> Vec<DomainStats> {
        let mut domains: Vec<_> = self.domains.read().await.values().cloned().collect();
        domains.sort_by(|a, b| b.blocked_count.cmp(&a.blocked_count));
        domains.truncate(limit);
        domains
    }

    /// Get lifetime statistics
    pub async fn get_lifetime_stats(&self) -> LifetimeStats {
        self.lifetime.read().await.clone()
    }

    /// Get statistics summary
    pub async fn get_summary(&self) -> StatsSummary {
        let session = self.get_session_stats().await;
        let today = self.get_today_stats().await;
        let lifetime = self.get_lifetime_stats().await;
        
        // Calculate last 7 days
        let now = Utc::now().date_naive();
        let week_ago = now - Duration::days(7);
        let last_7_days = self.aggregate_range(week_ago, now).await;
        
        // Calculate last 30 days
        let month_ago = now - Duration::days(30);
        let last_30_days = self.aggregate_range(month_ago, now).await;
        
        // Get top domains
        let top_domains = self.get_top_domains(10).await;
        
        // Calculate category breakdown
        let domains = self.domains.read().await;
        let mut by_category: HashMap<BlockCategory, u64> = HashMap::new();
        for domain in domains.values() {
            *by_category.entry(domain.category).or_insert(0) += domain.blocked_count;
        }
        
        StatsSummary {
            session,
            today,
            last_7_days,
            last_30_days,
            lifetime,
            top_domains,
            by_category,
        }
    }

    /// Aggregate statistics for a date range
    async fn aggregate_range(&self, from: NaiveDate, to: NaiveDate) -> AggregatedStats {
        let daily = self.daily.read().await;
        let stats: Vec<_> = daily.range(from..=to).collect();
        
        let days = stats.len() as u32;
        let total_blocked: u64 = stats.iter().map(|(_, s)| s.total_blocked).sum();
        
        AggregatedStats {
            total_blocked,
            ads_blocked: stats.iter().map(|(_, s)| s.ads_blocked).sum(),
            trackers_blocked: stats.iter().map(|(_, s)| s.trackers_blocked).sum(),
            threats_blocked: stats.iter().map(|(_, s)| s.threats_blocked).sum(),
            bandwidth_saved: stats.iter().map(|(_, s)| s.bandwidth_saved).sum(),
            time_saved_seconds: stats.iter().map(|(_, s)| s.time_saved_seconds).sum(),
            avg_per_day: if days > 0 { total_blocked as f64 / days as f64 } else { 0.0 },
            days,
        }
    }

    /// Reset session statistics
    pub async fn reset_session(&self) {
        *self.session.write().await = SessionStats::default();
    }

    /// Reset all statistics
    pub async fn reset_all(&self) {
        *self.session.write().await = SessionStats::default();
        self.daily.write().await.clear();
        self.domains.write().await.clear();
        *self.lifetime.write().await = LifetimeStats::default();
    }

    /// Clean up old statistics based on retention policy
    pub async fn cleanup(&self) {
        let config = self.config.read().await;
        let cutoff = Utc::now().date_naive() - Duration::days(config.retention_days as i64);
        drop(config);
        
        self.daily.write().await.retain(|date, _| *date >= cutoff);
    }

    /// Update configuration
    pub async fn update_config<F>(&self, f: F)
    where
        F: FnOnce(&mut StatsConfig),
    {
        let mut config = self.config.write().await;
        f(&mut config);
    }

    /// Get configuration
    pub async fn get_config(&self) -> StatsConfig {
        self.config.read().await.clone()
    }

    /// Export statistics to JSON
    pub async fn export_stats(&self) -> Result<String, serde_json::Error> {
        let summary = self.get_summary().await;
        serde_json::to_string_pretty(&summary)
    }

    /// Calculate bandwidth saved in human-readable format
    pub fn format_bandwidth(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        const TB: u64 = GB * 1024;
        
        if bytes >= TB {
            format!("{:.2} TB", bytes as f64 / TB as f64)
        } else if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }

    /// Calculate time saved in human-readable format
    pub fn format_time_saved(seconds: u64) -> String {
        const MINUTE: u64 = 60;
        const HOUR: u64 = MINUTE * 60;
        const DAY: u64 = HOUR * 24;
        
        if seconds >= DAY {
            let days = seconds / DAY;
            let hours = (seconds % DAY) / HOUR;
            format!("{}d {}h", days, hours)
        } else if seconds >= HOUR {
            let hours = seconds / HOUR;
            let minutes = (seconds % HOUR) / MINUTE;
            format!("{}h {}m", hours, minutes)
        } else if seconds >= MINUTE {
            let minutes = seconds / MINUTE;
            format!("{}m", minutes)
        } else {
            format!("{}s", seconds)
        }
    }

    /// Get blocking rate (blocked / total requests)
    pub async fn get_block_rate(&self) -> f64 {
        let session = self.session.read().await;
        if session.requests_processed > 0 {
            session.total_blocked as f64 / session.requests_processed as f64 * 100.0
        } else {
            0.0
        }
    }

    /// Create a block event
    pub fn create_event(
        url: String,
        domain: String,
        resource_type: ResourceType,
        category: BlockCategory,
        page_url: String,
        estimated_size: u64,
        filter_rule: Option<String>,
    ) -> BlockEvent {
        BlockEvent {
            id: format!("evt_{}", Utc::now().timestamp_millis()),
            timestamp: Utc::now(),
            url,
            domain,
            resource_type,
            category,
            page_url,
            estimated_size,
            filter_rule,
        }
    }

    /// Estimate size for a resource type
    pub fn estimate_size(resource_type: ResourceType, category: BlockCategory) -> u64 {
        let config = StatsConfig::default();
        
        match resource_type {
            ResourceType::Script => 30000, // 30KB average script
            ResourceType::Image => match category {
                BlockCategory::Ad => config.avg_ad_size as u64,
                _ => 50000,
            },
            ResourceType::Stylesheet => 20000,
            ResourceType::Media => 500000, // 500KB average media
            ResourceType::Font => 50000,
            _ => config.avg_ad_size as u64,
        }
    }
}

impl Default for BlockStatistics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_stats() {
        let stats = BlockStatistics::new();
        let session = stats.get_session_stats().await;
        assert_eq!(session.total_blocked, 0);
    }

    #[tokio::test]
    async fn test_record_block() {
        let stats = BlockStatistics::new();
        let event = BlockStatistics::create_event(
            "https://ads.com/banner.js".to_string(),
            "ads.com".to_string(),
            ResourceType::Script,
            BlockCategory::Ad,
            "https://example.com".to_string(),
            30000,
            Some("||ads.com^".to_string()),
        );
        
        stats.record_block(event).await;
        
        let session = stats.get_session_stats().await;
        assert_eq!(session.total_blocked, 1);
        assert_eq!(session.ads_blocked, 1);
        assert_eq!(session.bandwidth_saved, 30000);
    }

    #[tokio::test]
    async fn test_record_multiple_blocks() {
        let stats = BlockStatistics::new();
        
        // Record multiple events
        for i in 0..5 {
            let event = BlockStatistics::create_event(
                format!("https://tracker{}.com/script.js", i),
                format!("tracker{}.com", i),
                ResourceType::Script,
                BlockCategory::Tracker,
                "https://example.com".to_string(),
                25000,
                None,
            );
            stats.record_block(event).await;
        }
        
        let session = stats.get_session_stats().await;
        assert_eq!(session.total_blocked, 5);
        assert_eq!(session.trackers_blocked, 5);
    }

    #[tokio::test]
    async fn test_domain_stats() {
        let stats = BlockStatistics::new();
        
        for _ in 0..10 {
            let event = BlockStatistics::create_event(
                "https://ads.com/banner.js".to_string(),
                "ads.com".to_string(),
                ResourceType::Script,
                BlockCategory::Ad,
                "https://example.com".to_string(),
                10000,
                None,
            );
            stats.record_block(event).await;
        }
        
        let top_domains = stats.get_top_domains(5).await;
        assert_eq!(top_domains.len(), 1);
        assert_eq!(top_domains[0].domain, "ads.com");
        assert_eq!(top_domains[0].blocked_count, 10);
    }

    #[tokio::test]
    async fn test_summary() {
        let stats = BlockStatistics::new();
        
        let event = BlockStatistics::create_event(
            "https://ads.com/banner.js".to_string(),
            "ads.com".to_string(),
            ResourceType::Image,
            BlockCategory::Ad,
            "https://example.com".to_string(),
            50000,
            None,
        );
        stats.record_block(event).await;
        
        let summary = stats.get_summary().await;
        assert_eq!(summary.session.total_blocked, 1);
        assert_eq!(summary.today.total_blocked, 1);
        assert_eq!(summary.lifetime.total_blocked, 1);
    }

    #[tokio::test]
    async fn test_format_bandwidth() {
        assert_eq!(BlockStatistics::format_bandwidth(500), "500 B");
        assert_eq!(BlockStatistics::format_bandwidth(1024), "1.00 KB");
        assert_eq!(BlockStatistics::format_bandwidth(1048576), "1.00 MB");
        assert_eq!(BlockStatistics::format_bandwidth(1073741824), "1.00 GB");
    }

    #[tokio::test]
    async fn test_format_time() {
        assert_eq!(BlockStatistics::format_time_saved(30), "30s");
        assert_eq!(BlockStatistics::format_time_saved(90), "1m");
        assert_eq!(BlockStatistics::format_time_saved(3661), "1h 1m");
        assert_eq!(BlockStatistics::format_time_saved(90061), "1d 1h");
    }

    #[tokio::test]
    async fn test_block_rate() {
        let stats = BlockStatistics::new();
        
        // Record some requests
        for _ in 0..100 {
            stats.record_request().await;
        }
        
        // Record some blocks
        for _ in 0..25 {
            let event = BlockStatistics::create_event(
                "https://ads.com/ad.js".to_string(),
                "ads.com".to_string(),
                ResourceType::Script,
                BlockCategory::Ad,
                "https://example.com".to_string(),
                10000,
                None,
            );
            stats.record_block(event).await;
        }
        
        let rate = stats.get_block_rate().await;
        assert_eq!(rate, 25.0);
    }

    #[tokio::test]
    async fn test_reset() {
        let stats = BlockStatistics::new();
        
        let event = BlockStatistics::create_event(
            "https://ads.com/ad.js".to_string(),
            "ads.com".to_string(),
            ResourceType::Script,
            BlockCategory::Ad,
            "https://example.com".to_string(),
            10000,
            None,
        );
        stats.record_block(event).await;
        
        stats.reset_session().await;
        let session = stats.get_session_stats().await;
        assert_eq!(session.total_blocked, 0);
    }

    #[tokio::test]
    async fn test_estimate_size() {
        let script_size = BlockStatistics::estimate_size(ResourceType::Script, BlockCategory::Ad);
        assert!(script_size > 0);
        
        let image_size = BlockStatistics::estimate_size(ResourceType::Image, BlockCategory::Ad);
        assert!(image_size > 0);
    }
}