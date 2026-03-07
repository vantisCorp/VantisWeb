//! Analytics dashboard module for UI integration
//! 
//! This module provides dashboard data structures and components
//! for rendering analytics in the browser UI.

use chrono::{DateTime, Utc, NaiveDate};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use super::metrics::{UsageMetrics, PerformanceMetrics, SecurityMetrics, AnalyticsSnapshot};
use super::exporter::{Insight, InsightCategory, ImpactLevel};

/// Dashboard view configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// Default time range to display
    pub default_time_range: TimeRangePreset,
    /// Number of top domains to show
    pub top_domains_count: usize,
    /// Whether to show performance metrics
    pub show_performance: bool,
    /// Whether to show security metrics
    pub show_security: bool,
    /// Whether to show usage insights
    pub show_insights: bool,
    /// Refresh interval in seconds (0 = manual only)
    pub refresh_interval_seconds: u32,
    /// Theme for dashboard
    pub theme: DashboardTheme,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            default_time_range: TimeRangePreset::Last7Days,
            top_domains_count: 10,
            show_performance: true,
            show_security: true,
            show_insights: true,
            refresh_interval_seconds: 300, // 5 minutes
            theme: DashboardTheme::System,
        }
    }
}

/// Time range presets for dashboard
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TimeRangePreset {
    Today,
    Yesterday,
    Last7Days,
    Last30Days,
    ThisWeek,
    ThisMonth,
    AllTime,
}

/// Dashboard theme options
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DashboardTheme {
    Light,
    Dark,
    System,
}

/// Dashboard data for rendering
#[derive(Debug, Serialize)]
pub struct DashboardData {
    /// Time range for this data
    pub time_range: TimeRangePreset,
    /// Generation timestamp
    pub generated_at: DateTime<Utc>,
    /// Summary statistics
    pub summary: DashboardSummary,
    /// Usage section data
    pub usage: UsageSection,
    /// Performance section data
    pub performance: PerformanceSection,
    /// Security section data
    pub security: SecuritySection,
    /// Top visited domains
    pub top_domains: Vec<DomainStats>,
    /// Activity timeline
    pub activity_timeline: Vec<ActivityPoint>,
    /// Insights and recommendations
    pub insights: Vec<Insight>,
    /// Charts configuration
    pub charts: ChartConfig,
}

/// Summary statistics for dashboard header
#[derive(Debug, Serialize)]
pub struct DashboardSummary {
    /// Total browsing time (formatted)
    pub total_time_formatted: String,
    /// Total page visits
    pub total_visits: u64,
    /// Unique domains visited
    pub unique_domains: u64,
    /// Average page load time
    pub avg_load_time_ms: u64,
    /// Security score (0-100)
    pub security_score: u32,
    /// Privacy score (0-100)
    pub privacy_score: u32,
    /// Productivity score (0-100)
    pub productivity_score: u32,
}

/// Usage section for dashboard
#[derive(Debug, Serialize)]
pub struct UsageSection {
    /// Total visits
    pub total_visits: u64,
    /// Unique domains
    pub unique_domains: u64,
    /// Total time in milliseconds
    pub total_time_ms: u64,
    /// Average session duration
    pub avg_session_ms: u64,
    /// Most active hour of day
    pub peak_hour: Option<u32>,
    /// Most active day of week
    pub peak_day: Option<String>,
    /// Visits by hour (24 elements)
    pub visits_by_hour: Vec<u64>,
    /// Visits by day of week (7 elements)
    pub visits_by_day: Vec<u64>,
    /// Trend compared to previous period
    pub trend: Option<TrendIndicator>,
}

/// Performance section for dashboard
#[derive(Debug, Serialize)]
pub struct PerformanceSection {
    /// Average page load time
    pub avg_load_time_ms: u64,
    /// Median page load time
    pub median_load_time_ms: u64,
    /// 95th percentile load time
    pub p95_load_time_ms: u64,
    /// Number of slow pages (>3s)
    pub slow_pages_count: u64,
    /// Average memory usage in MB
    pub avg_memory_mb: u64,
    /// Peak memory usage in MB
    pub peak_memory_mb: u64,
    /// Average CPU usage percentage
    pub avg_cpu_percent: u64,
    /// Performance score (0-100)
    pub performance_score: u32,
    /// Load time distribution (buckets)
    pub load_time_distribution: Vec<LoadTimeBucket>,
}

/// Load time distribution bucket
#[derive(Debug, Serialize)]
pub struct LoadTimeBucket {
    pub range: String,
    pub count: u64,
}

/// Security section for dashboard
#[derive(Debug, Serialize)]
pub struct SecuritySection {
    /// Total threats blocked
    pub threats_blocked: u64,
    /// HTTPS upgrades performed
    pub https_upgrades: u64,
    /// Trackers blocked
    pub trackers_blocked: u64,
    /// Third-party cookies blocked
    pub cookies_blocked: u64,
    /// Phishing attempts blocked
    pub phishing_blocked: u64,
    /// Malware downloads blocked
    pub malware_blocked: u64,
    /// Security score (0-100)
    pub security_score: u32,
    /// Recent security events
    pub recent_events: Vec<SecurityEvent>,
}

/// Security event for display
#[derive(Debug, Serialize)]
pub struct SecurityEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: SecurityEventType,
    pub description: String,
    pub severity: EventSeverity,
}

#[derive(Debug, Clone, Serialize)]
pub enum SecurityEventType {
    ThreatBlocked,
    TrackerBlocked,
    HttpsUpgrade,
    CookieBlocked,
    PhishingBlocked,
    MalwareBlocked,
}

#[derive(Debug, Clone, Serialize)]
pub enum EventSeverity {
    Info,
    Warning,
    Critical,
}

/// Domain statistics for top domains list
#[derive(Debug, Serialize)]
pub struct DomainStats {
    pub domain: String,
    pub visits: u64,
    pub time_ms: u64,
    pub category: Option<String>,
    pub percentage: f64,
}

/// Activity timeline data point
#[derive(Debug, Serialize)]
pub struct ActivityPoint {
    pub timestamp: DateTime<Utc>,
    pub visits: u64,
    pub time_ms: u64,
}

/// Trend indicator for comparing periods
#[derive(Debug, Serialize)]
pub struct TrendIndicator {
    pub direction: TrendDirection,
    pub percentage: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub enum TrendDirection {
    Up,
    Down,
    Stable,
}

/// Chart configuration for frontend rendering
#[derive(Debug, Serialize)]
pub struct ChartConfig {
    pub usage_chart: ChartSettings,
    pub performance_chart: ChartSettings,
    pub security_chart: ChartSettings,
    pub timeline_chart: ChartSettings,
}

#[derive(Debug, Serialize)]
pub struct ChartSettings {
    pub chart_type: ChartType,
    pub colors: Vec<String>,
    pub show_legend: bool,
    pub animated: bool,
}

#[derive(Debug, Clone, Serialize)]
pub enum ChartType {
    Line,
    Bar,
    Pie,
    Doughnut,
    Area,
}

impl Default for ChartConfig {
    fn default() -> Self {
        Self {
            usage_chart: ChartSettings {
                chart_type: ChartType::Bar,
                colors: vec![
                    "#4F46E5".to_string(),
                    "#10B981".to_string(),
                    "#F59E0B".to_string(),
                ],
                show_legend: true,
                animated: true,
            },
            performance_chart: ChartSettings {
                chart_type: ChartType::Line,
                colors: vec![
                    "#4F46E5".to_string(),
                ],
                show_legend: false,
                animated: true,
            },
            security_chart: ChartSettings {
                chart_type: ChartType::Doughnut,
                colors: vec![
                    "#EF4444".to_string(),
                    "#F59E0B".to_string(),
                    "#10B981".to_string(),
                    "#4F46E5".to_string(),
                ],
                show_legend: true,
                animated: true,
            },
            timeline_chart: ChartSettings {
                chart_type: ChartType::Area,
                colors: vec![
                    "#4F46E5".to_string(),
                ],
                show_legend: false,
                animated: true,
            },
        }
    }
}

/// Dashboard builder for constructing dashboard data
pub struct DashboardBuilder {
    config: DashboardConfig,
    time_range: TimeRangePreset,
}

impl DashboardBuilder {
    /// Create a new dashboard builder
    pub fn new(config: DashboardConfig) -> Self {
        Self {
            config,
            time_range: TimeRangePreset::Last7Days,
        }
    }
    
    /// Set the time range for the dashboard
    pub fn with_time_range(mut self, range: TimeRangePreset) -> Self {
        self.time_range = range;
        self
    }
    
    /// Build dashboard data from analytics snapshot
    pub fn build(&self, snapshot: &AnalyticsSnapshot) -> DashboardData {
        let summary = self.build_summary(snapshot);
        let usage = self.build_usage_section(snapshot);
        let performance = self.build_performance_section(snapshot);
        let security = self.build_security_section(snapshot);
        let top_domains = self.build_top_domains(snapshot);
        let activity_timeline = self.build_activity_timeline(snapshot);
        let insights = self.generate_insights(snapshot);
        
        DashboardData {
            time_range: self.time_range,
            generated_at: Utc::now(),
            summary,
            usage,
            performance,
            security,
            top_domains,
            activity_timeline,
            insights,
            charts: ChartConfig::default(),
        }
    }
    
    fn build_summary(&self, snapshot: &AnalyticsSnapshot) -> DashboardSummary {
        let total_time_ms = snapshot.usage.total_time_ms;
        let hours = total_time_ms / (1000 * 60 * 60);
        let minutes = (total_time_ms % (1000 * 60 * 60)) / (1000 * 60);
        
        DashboardSummary {
            total_time_formatted: format!("{}h {}m", hours, minutes),
            total_visits: snapshot.usage.total_visits,
            unique_domains: snapshot.usage.unique_domains,
            avg_load_time_ms: snapshot.performance.average_load_time_ms,
            security_score: self.calculate_security_score(snapshot),
            privacy_score: self.calculate_privacy_score(snapshot),
            productivity_score: self.calculate_productivity_score(snapshot),
        }
    }
    
    fn build_usage_section(&self, snapshot: &AnalyticsSnapshot) -> UsageSection {
        UsageSection {
            total_visits: snapshot.usage.total_visits,
            unique_domains: snapshot.usage.unique_domains,
            total_time_ms: snapshot.usage.total_time_ms,
            avg_session_ms: snapshot.usage.average_page_time_ms,
            peak_hour: Some(14), // Placeholder - would be calculated from data
            peak_day: Some("Wednesday".to_string()),
            visits_by_hour: vec![0; 24], // Placeholder
            visits_by_day: vec![0; 7], // Placeholder
            trend: None,
        }
    }
    
    fn build_performance_section(&self, snapshot: &AnalyticsSnapshot) -> PerformanceSection {
        PerformanceSection {
            avg_load_time_ms: snapshot.performance.average_load_time_ms,
            median_load_time_ms: snapshot.performance.median_load_time_ms,
            p95_load_time_ms: snapshot.performance.p95_load_time_ms,
            slow_pages_count: snapshot.performance.slow_pages,
            avg_memory_mb: snapshot.performance.average_memory_mb,
            peak_memory_mb: snapshot.performance.peak_memory_mb,
            avg_cpu_percent: snapshot.performance.average_cpu_percent,
            performance_score: self.calculate_performance_score(snapshot),
            load_time_distribution: vec![
                LoadTimeBucket { range: "0-1s".to_string(), count: 50 },
                LoadTimeBucket { range: "1-2s".to_string(), count: 30 },
                LoadTimeBucket { range: "2-3s".to_string(), count: 15 },
                LoadTimeBucket { range: ">3s".to_string(), count: 5 },
            ],
        }
    }
    
    fn build_security_section(&self, snapshot: &AnalyticsSnapshot) -> SecuritySection {
        SecuritySection {
            threats_blocked: snapshot.security.threats_blocked,
            https_upgrades: snapshot.security.https_upgrades,
            trackers_blocked: snapshot.security.trackers_blocked,
            cookies_blocked: snapshot.security.cookies_blocked,
            phishing_blocked: snapshot.security.phishing_blocked,
            malware_blocked: snapshot.security.malware_blocked,
            security_score: self.calculate_security_score(snapshot),
            recent_events: vec![], // Would be populated from storage
        }
    }
    
    fn build_top_domains(&self, snapshot: &AnalyticsSnapshot) -> Vec<DomainStats> {
        let total_visits = snapshot.usage.total_visits.max(1);
        
        snapshot.usage.top_domains
            .iter()
            .take(self.config.top_domains_count)
            .map(|(domain, stats)| {
                DomainStats {
                    domain: domain.clone(),
                    visits: stats.visits,
                    time_ms: stats.time_ms,
                    category: stats.category.clone(),
                    percentage: (stats.visits as f64 / total_visits as f64) * 100.0,
                }
            })
            .collect()
    }
    
    fn build_activity_timeline(&self, snapshot: &AnalyticsSnapshot) -> Vec<ActivityPoint> {
        // Generate placeholder activity timeline
        let mut timeline = Vec::new();
        let now = Utc::now();
        
        for i in 0..7 {
            let timestamp = now - chrono::Duration::days(i);
            timeline.push(ActivityPoint {
                timestamp,
                visits: 0,
                time_ms: 0,
            });
        }
        
        timeline
    }
    
    fn generate_insights(&self, snapshot: &AnalyticsSnapshot) -> Vec<Insight> {
        let mut insights = Vec::new();
        
        // Performance insight
        if snapshot.performance.average_load_time_ms > 3000 {
            insights.push(Insight {
                category: InsightCategory::Performance,
                title: "Slow page load times".to_string(),
                description: "Consider enabling hardware acceleration for faster browsing".to_string(),
                impact: ImpactLevel::Medium,
                actionable: true,
                recommendations: vec!["Enable hardware acceleration".to_string()],
            });
        }
        
        // Security insight
        if snapshot.security.trackers_blocked > 100 {
            insights.push(Insight {
                category: InsightCategory::Privacy,
                title: "High tracker activity blocked".to_string(),
                description: format!("{} tracking attempts were blocked", snapshot.security.trackers_blocked),
                impact: ImpactLevel::Low,
                actionable: false,
                recommendations: vec![],
            });
        }
        
        insights
    }
    
    fn calculate_security_score(&self, snapshot: &AnalyticsSnapshot) -> u32 {
        let mut score = 100u32;
        
        // Deduct points for security issues
        if snapshot.security.threats_blocked == 0 {
            score = score.saturating_sub(10); // No threats doesn't mean secure
        }
        
        // Bonus for HTTPS upgrades
        score = score.saturating_add((snapshot.security.https_upgrades / 10) as u32).min(100);
        
        score
    }
    
    fn calculate_privacy_score(&self, snapshot: &AnalyticsSnapshot) -> u32 {
        let mut score = 50u32;
        
        // Add points for trackers blocked
        score = score.saturating_add((snapshot.security.trackers_blocked / 5) as u32).min(100);
        
        // Add points for cookies blocked
        score = score.saturating_add((snapshot.security.cookies_blocked / 10) as u32).min(100);
        
        score
    }
    
    fn calculate_productivity_score(&self, snapshot: &AnalyticsSnapshot) -> u32 {
        // Based on average time per page and total visits
        let avg_time = snapshot.usage.average_page_time_ms;
        
        if avg_time < 30000 {
            80 // Quick browsing - might be productive
        } else if avg_time < 120000 {
            90 // Good balance
        } else {
            70 // Long page times - might be reading or might be distracted
        }
    }
}

/// Dashboard component for rendering in UI
#[derive(Debug, Serialize)]
pub struct DashboardComponent {
    pub component_type: ComponentType,
    pub title: String,
    pub data: serde_json::Value,
    pub actions: Vec<ComponentAction>,
}

#[derive(Debug, Clone, Serialize)]
pub enum ComponentType {
    StatCard,
    Chart,
    Table,
    List,
    Timeline,
    Insight,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComponentAction {
    pub label: String,
    pub action: String,
    pub icon: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dashboard_config_default() {
        let config = DashboardConfig::default();
        assert_eq!(config.top_domains_count, 10);
        assert!(config.show_performance);
    }
    
    #[test]
    fn test_time_range_presets() {
        let config = DashboardConfig::default();
        assert!(matches!(config.default_time_range, TimeRangePreset::Last7Days));
    }
    
    #[test]
    fn test_dashboard_builder() {
        let builder = DashboardBuilder::new(DashboardConfig::default());
        let snapshot = AnalyticsSnapshot::default();
        let dashboard = builder.build(&snapshot);
        
        assert!(dashboard.summary.total_visits >= 0);
    }
}