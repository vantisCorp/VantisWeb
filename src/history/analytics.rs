/// # History Analytics Module
/// 
/// Provides analytics and statistics for browser history.
/// Analyzes browsing patterns, identifies trends, and generates insights.

use std::collections::HashMap;
use chrono::{DateTime, Utc, Datelike, Weekday, Timelike};
use serde::{Deserialize, Serialize};

use super::{HistoryEntry, HistoryStatistics, Result};

/// Analytics engine for history data
pub struct HistoryAnalytics {
    /// Configuration
    config: AnalyticsConfig,
}

/// Analytics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    /// Enable category detection
    pub enable_categories: bool,
    /// Enable trending detection
    pub enable_trending: bool,
    /// Days to consider for "recent"
    pub recent_days: i64,
    /// Top N items to track
    pub top_n: usize,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            enable_categories: true,
            enable_trending: true,
            recent_days: 7,
            top_n: 10,
        }
    }
}

/// Browsing pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowsingPattern {
    /// Most active day of week
    pub most_active_day: Weekday,
    /// Most active hour of day
    pub most_active_hour: u32,
    /// Average daily visits
    pub avg_daily_visits: f64,
    /// Peak browsing times
    pub peak_times: Vec<PeakTime>,
    /// Browsing session duration
    pub avg_session_duration: f64,
}

/// Peak browsing time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakTime {
    /// Day of week
    pub day: Weekday,
    /// Hour of day
    pub hour: u32,
    /// Activity score
    pub score: f64,
}

/// Domain statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainStats {
    /// Domain name
    pub domain: String,
    /// Total visits
    pub visits: u64,
    /// Unique pages
    pub unique_pages: usize,
    /// First visit
    pub first_visit: DateTime<Utc>,
    /// Last visit
    pub last_visit: DateTime<Utc>,
    /// Time spent (estimated)
    pub estimated_time: f64,
}

/// Category breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryBreakdown {
    /// Category name
    pub category: String,
    /// Count
    pub count: usize,
    /// Percentage
    pub percentage: f64,
    /// Domains in category
    pub domains: Vec<String>,
}

impl HistoryAnalytics {
    /// Create a new analytics engine
    pub fn new() -> Self {
        Self {
            config: AnalyticsConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: AnalyticsConfig) -> Self {
        Self { config }
    }

    /// Calculate history statistics
    pub async fn calculate_statistics(&self, entries: &[HistoryEntry]) -> Result<HistoryStatistics> {
        // Count unique domains
        let mut domains: HashMap<String, usize> = HashMap::new();
        let mut category_dist: HashMap<String, usize> = HashMap::new();
        let mut daily_visits: HashMap<String, usize> = HashMap::new();
        let mut weekly_visits: HashMap<String, usize> = HashMap::new();
        let mut monthly_visits: HashMap<String, usize> = HashMap::new();

        let mut total_visits = 0u64;

        for entry in entries {
            // Count domain
            let domain = extract_domain(&entry.url);
            *domains.entry(domain).or_insert(0) += 1;

            // Count category
            if let Some(ref category) = entry.category {
                *category_dist.entry(category.clone()).or_insert(0) += 1;
            }

            // Time-based counts
            let date = entry.timestamp.date_naive();
            daily_visits.entry(date.format("%Y-%m-%d").to_string())
                .and_modify(|c| *c += 1)
                .or_insert(1);

            weekly_visits.entry(date.format("%Y-W%V").to_string())
                .and_modify(|c| *c += 1)
                .or_insert(1);

            monthly_visits.entry(date.format("%Y-%m").to_string())
                .and_modify(|c| *c += 1)
                .or_insert(1);

            total_visits += entry.visit_count as u64;
        }

        // Find most visited
        let mut entries_sorted = entries.to_vec();
        entries_sorted.sort_by(|a, b| b.visit_count.cmp(&a.visit_count));
        let most_visited: Vec<HistoryEntry> = entries_sorted
            .iter()
            .take(self.config.top_n)
            .cloned()
            .collect();

        Ok(HistoryStatistics {
            total_entries: entries.len(),
            total_visits,
            unique_domains: domains.len(),
            most_visited,
            category_distribution: category_dist,
            daily_visits,
            weekly_visits,
            monthly_visits,
        })
    }

    /// Analyze browsing patterns
    pub async fn analyze_patterns(&self, entries: &[HistoryEntry]) -> BrowsingPattern {
        // Analyze by day of week and hour
        let mut day_counts: HashMap<Weekday, usize> = HashMap::new();
        let mut hour_counts: HashMap<u32, usize> = HashMap::new();
        let mut day_hour_counts: HashMap<(Weekday, u32), usize> = HashMap::new();

        for entry in entries {
            let weekday = entry.timestamp.weekday();
            let hour = entry.timestamp.hour();

            *day_counts.entry(weekday).or_insert(0) += 1;
            *hour_counts.entry(hour).or_insert(0) += 1;
            *day_hour_counts.entry((weekday, hour)).or_insert(0) += 1;
        }

        // Find most active day
        let most_active_day = day_counts
            .iter()
            .max_by_key(|(_, c)| *c)
            .map(|(d, _)| *d)
            .unwrap_or(Weekday::Monday);

        // Find most active hour
        let most_active_hour = hour_counts
            .iter()
            .max_by_key(|(_, c)| *c)
            .map(|(h, _)| *h)
            .unwrap_or(12);

        // Find peak times
        let mut peak_times: Vec<PeakTime> = day_hour_counts
            .iter()
            .map(|((day, hour), count)| PeakTime {
                day: *day,
                hour: *hour,
                score: *count as f64,
            })
            .collect();
        peak_times.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        peak_times.truncate(5);

        // Calculate average daily visits
        let total_days = entries.iter()
            .map(|e| e.timestamp.date_naive())
            .collect::<std::collections::HashSet<_>>()
            .len();
        let avg_daily_visits = if total_days > 0 {
            entries.len() as f64 / total_days as f64
        } else {
            0.0
        };

        // Estimate session duration (simplified)
        let avg_session_duration = estimate_session_duration(entries);

        BrowsingPattern {
            most_active_day,
            most_active_hour,
            avg_daily_visits,
            peak_times,
            avg_session_duration,
        }
    }

    /// Get domain statistics
    pub async fn get_domain_stats(&self, entries: &[HistoryEntry]) -> Vec<DomainStats> {
        let mut domain_data: HashMap<String, DomainStatsBuilder> = HashMap::new();

        for entry in entries {
            let domain = extract_domain(&entry.url);
            
            let builder = domain_data.entry(domain.clone())
                .or_insert_with(|| DomainStatsBuilder {
                    domain: domain.clone(),
                    visits: 0,
                    pages: std::collections::HashSet::new(),
                    first_visit: entry.timestamp,
                    last_visit: entry.timestamp,
                });

            builder.visits += entry.visit_count as u64;
            builder.pages.insert(entry.url.clone());
            
            if entry.timestamp < builder.first_visit {
                builder.first_visit = entry.timestamp;
            }
            if entry.timestamp > builder.last_visit {
                builder.last_visit = entry.timestamp;
            }
        }

        let mut stats: Vec<DomainStats> = domain_data
            .into_values()
            .map(|b| DomainStats {
                domain: b.domain,
                visits: b.visits,
                unique_pages: b.pages.len(),
                first_visit: b.first_visit,
                last_visit: b.last_visit,
                estimated_time: b.visits as f64 * 0.5, // Rough estimate
            })
            .collect();

        stats.sort_by(|a, b| b.visits.cmp(&a.visits));
        stats
    }

    /// Get category breakdown
    pub async fn get_category_breakdown(&self, entries: &[HistoryEntry]) -> Vec<CategoryBreakdown> {
        let mut categories: HashMap<String, CategoryBuilder> = HashMap::new();
        let total = entries.len();

        for entry in entries {
            let category = entry.category.clone().unwrap_or_else(|| detect_category(&entry.url));
            
            let builder = categories.entry(category.clone())
                .or_insert_with(|| CategoryBuilder {
                    category: category.clone(),
                    count: 0,
                    domains: std::collections::HashSet::new(),
                });

            builder.count += 1;
            builder.domains.insert(extract_domain(&entry.url));
        }

        let mut breakdown: Vec<CategoryBreakdown> = categories
            .into_values()
            .map(|b| CategoryBreakdown {
                category: b.category,
                count: b.count,
                percentage: if total > 0 { b.count as f64 / total as f64 * 100.0 } else { 0.0 },
                domains: b.domains.into_iter().collect(),
            })
            .collect();

        breakdown.sort_by(|a, b| b.count.cmp(&a.count));
        breakdown
    }

    /// Find trending sites (increased visit frequency)
    pub async fn find_trending(&self, entries: &[HistoryEntry]) -> Vec<TrendingSite> {
        let now = Utc::now();
        let recent_threshold = now - chrono::Duration::days(self.config.recent_days);

        let mut recent_counts: HashMap<String, usize> = HashMap::new();
        let mut older_counts: HashMap<String, usize> = HashMap::new();

        for entry in entries {
            let domain = extract_domain(&entry.url);
            
            if entry.timestamp > recent_threshold {
                *recent_counts.entry(domain.clone()).or_insert(0) += 1;
            } else {
                *older_counts.entry(domain.clone()).or_insert(0) += 1;
            }
        }

        let mut trending: Vec<TrendingSite> = recent_counts
            .iter()
            .filter_map(|(domain, recent)| {
                let older = older_counts.get(domain).copied().unwrap_or(0);
                
                // Calculate trend score (recent vs older)
                let score = if older > 0 {
                    (*recent as f64 / older as f64) - 1.0
                } else {
                    1.0 // New site
                };

                if score > 0.0 {
                    Some(TrendingSite {
                        domain: domain.clone(),
                        recent_visits: *recent,
                        older_visits: older,
                        trend_score: score,
                    })
                } else {
                    None
                }
            })
            .collect();

        trending.sort_by(|a, b| b.trend_score.partial_cmp(&a.trend_score).unwrap());
        trending.truncate(self.config.top_n);
        trending
    }
}

/// Helper struct for building domain stats
struct DomainStatsBuilder {
    domain: String,
    visits: u64,
    pages: std::collections::HashSet<String>,
    first_visit: DateTime<Utc>,
    last_visit: DateTime<Utc>,
}

/// Helper struct for building category stats
struct CategoryBuilder {
    category: String,
    count: usize,
    domains: std::collections::HashSet<String>,
}

/// Trending site information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingSite {
    /// Domain name
    pub domain: String,
    /// Recent visits
    pub recent_visits: usize,
    /// Older visits
    pub older_visits: usize,
    /// Trend score (positive = trending up)
    pub trend_score: f64,
}

/// Extract domain from URL
fn extract_domain(url: &str) -> String {
    let url = url.trim_start_matches("https://")
                .trim_start_matches("http://")
                .trim_start_matches("www.");
    url.split('/').next().unwrap_or(url).to_string()
}

/// Detect category from URL
fn detect_category(url: &str) -> String {
    let url_lower = url.to_lowercase();
    
    if url_lower.contains("github") || url_lower.contains("stackoverflow") || url_lower.contains("dev.to") {
        "Development".to_string()
    } else if url_lower.contains("youtube") || url_lower.contains("netflix") || url_lower.contains("twitch") {
        "Entertainment".to_string()
    } else if url_lower.contains("twitter") || url_lower.contains("facebook") || url_lower.contains("linkedin") {
        "Social".to_string()
    } else if url_lower.contains("amazon") || url_lower.contains("ebay") || url_lower.contains("shop") {
        "Shopping".to_string()
    } else if url_lower.contains("news") || url_lower.contains("cnn") || url_lower.contains("bbc") {
        "News".to_string()
    } else {
        "Other".to_string()
    }
}

/// Estimate average session duration
fn estimate_session_duration(entries: &[HistoryEntry]) -> f64 {
    if entries.len() < 2 {
        return 0.0;
    }

    let mut sorted_entries = entries.to_vec();
    sorted_entries.sort_by_key(|e| e.timestamp);

    let mut sessions: Vec<Vec<&HistoryEntry>> = vec![];
    let mut current_session: Vec<&HistoryEntry> = vec![&sorted_entries[0]];

    for i in 1..sorted_entries.len() {
        let prev = &sorted_entries[i - 1];
        let curr = &sorted_entries[i];
        
        // If gap is less than 30 minutes, consider same session
        let gap = curr.timestamp - prev.timestamp;
        if gap.num_minutes() < 30 {
            current_session.push(&sorted_entries[i]);
        } else {
            sessions.push(current_session);
            current_session = vec![&sorted_entries[i]];
        }
    }
    sessions.push(current_session);

    // Calculate average duration
    let total_duration: i64 = sessions.iter()
        .map(|s| {
            if s.len() >= 2 {
                (s.last().unwrap().timestamp - s.first().unwrap().timestamp).num_seconds()
            } else {
                60 // Assume 1 minute for single-page sessions
            }
        })
        .sum();

    if sessions.is_empty() {
        0.0
    } else {
        (total_duration as f64 / sessions.len() as f64) / 60.0 // In minutes
    }
}

impl Default for HistoryAnalytics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_calculate_statistics() {
        let analytics = HistoryAnalytics::new();
        
        let entries = vec![
            HistoryEntry::new("https://example.com".to_string(), "Example".to_string(), false),
            HistoryEntry::new("https://test.com".to_string(), "Test".to_string(), false),
        ];

        let stats = analytics.calculate_statistics(&entries).await.unwrap();
        
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.unique_domains, 2);
    }

    #[tokio::test]
    async fn test_analyze_patterns() {
        let analytics = HistoryAnalytics::new();
        
        let entries = vec![
            HistoryEntry::new("https://example.com".to_string(), "Example".to_string(), false),
        ];

        let patterns = analytics.analyze_patterns(&entries).await;
        
        assert!(patterns.avg_daily_visits > 0.0);
    }

    #[test]
    fn test_detect_category() {
        assert_eq!(detect_category("https://github.com/user/repo"), "Development");
        assert_eq!(detect_category("https://youtube.com/watch"), "Entertainment");
        assert_eq!(detect_category("https://twitter.com/user"), "Social");
    }
}