//! Enhanced Analytics Visualization Module
//!
//! Provides chart data generation, category tracking, heatmap data,
//! comparison views, and export functionality for analytics visualizations.

use anyhow::Result;
use chrono::{DateTime, Duration, NaiveDate, Timelike, Utc, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::analytics::{ProfileAnalytics, DailyUsage, WebsiteUsage, UsageSummary};

/// Website category for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebsiteCategory {
    /// Category name
    pub name: String,
    /// Category icon/emoji
    pub icon: String,
    /// Total time spent in category
    pub time_spent: u64,
    /// Visit count
    pub visits: u64,
    /// Percentage of total time
    pub percentage: f64,
}

/// Heatmap cell data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapCell {
    /// Hour (0-23)
    pub hour: u8,
    /// Day of week (0-6, 0 = Sunday)
    pub day: u8,
    /// Intensity (0.0 - 1.0)
    pub intensity: f64,
    /// Actual value
    pub value: u64,
}

/// Trend data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendDataPoint {
    /// Date/time label
    pub label: String,
    /// Value
    pub value: u64,
    /// Optional secondary value (for comparisons)
    pub secondary_value: Option<u64>,
}

/// Comparison data between profiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileComparison {
    /// Primary profile ID
    pub primary_profile: String,
    /// Secondary profile ID
    pub secondary_profile: String,
    /// Comparison metrics
    pub metrics: ComparisonMetrics,
}

/// Comparison metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonMetrics {
    pub total_time_diff: i64,
    pub session_count_diff: i64,
    pub websites_visited_diff: i64,
    pub avg_session_time_diff: i64,
    pub tabs_opened_diff: i64,
    pub performance_diff: f64,
}

/// Export format options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "csv")]
    Csv,
    #[serde(rename = "pdf")]
    Pdf,
    #[serde(rename = "html")]
    Html,
}

/// Date range for analytics queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl DateRange {
    /// Create a date range for the last N days
    pub fn last_days(days: u32) -> Self {
        let end = Utc::now();
        let start = end - Duration::days(days as i64);
        Self { start, end }
    }

    /// Create a date range for the current week
    pub fn current_week() -> Self {
        let now = Utc::now();
        let days_since_sunday = now.weekday().num_days_from_sunday() as i64;
        let start = now - Duration::days(days_since_sunday);
        Self { start, end: now }
    }

    /// Create a date range for the current month
    pub fn current_month() -> Self {
        let now = Utc::now();
        let start = now - Duration::days(now.day() as i64 - 1);
        Self { start, end: now }
    }
}

/// Analytics report for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsReport {
    /// Report title
    pub title: String,
    /// Generation timestamp
    pub generated_at: DateTime<Utc>,
    /// Profile ID
    pub profile_id: String,
    /// Profile name
    pub profile_name: String,
    /// Date range
    pub date_range: DateRange,
    /// Summary statistics
    pub summary: UsageSummary,
    /// Usage trends
    pub trends: Vec<TrendDataPoint>,
    /// Category distribution
    pub categories: Vec<WebsiteCategory>,
    /// Heatmap data
    pub heatmap: Vec<HeatmapCell>,
    /// Top websites
    pub top_websites: Vec<WebsiteUsage>,
    /// Daily usage breakdown
    pub daily_breakdown: Vec<DailyUsage>,
}

/// Enhanced analytics visualization manager
pub struct VisualizationManager {
    /// Category mappings for websites
    category_mappings: HashMap<String, String>,
    /// Cached category data
    category_cache: HashMap<String, WebsiteCategory>,
}

impl VisualizationManager {
    /// Creates a new visualization manager
    pub fn new() -> Self {
        Self {
            category_mappings: Self::default_category_mappings(),
            category_cache: HashMap::new(),
        }
    }

    /// Default category mappings for common websites
    fn default_category_mappings() -> HashMap<String, String> {
        let mut mappings = HashMap::new();
        
        // Social Media
        mappings.insert("facebook.com".to_string(), "Social Media");
        mappings.insert("twitter.com".to_string(), "Social Media");
        mappings.insert("x.com".to_string(), "Social Media");
        mappings.insert("instagram.com".to_string(), "Social Media");
        mappings.insert("linkedin.com".to_string(), "Social Media");
        mappings.insert("tiktok.com".to_string(), "Social Media");
        mappings.insert("reddit.com".to_string(), "Social Media");
        mappings.insert("pinterest.com".to_string(), "Social Media");
        
        // Development
        mappings.insert("github.com".to_string(), "Development");
        mappings.insert("gitlab.com".to_string(), "Development");
        mappings.insert("stackoverflow.com".to_string(), "Development");
        mappings.insert("codepen.io".to_string(), "Development");
        mappings.insert("dev.to".to_string(), "Development");
        mappings.insert("npmjs.com".to_string(), "Development");
        mappings.insert("crates.io".to_string(), "Development");
        
        // Entertainment
        mappings.insert("youtube.com".to_string(), "Entertainment");
        mappings.insert("netflix.com".to_string(), "Entertainment");
        mappings.insert("twitch.tv".to_string(), "Entertainment");
        mappings.insert("spotify.com".to_string(), "Entertainment");
        mappings.insert("soundcloud.com".to_string(), "Entertainment");
        mappings.insert("vimeo.com".to_string(), "Entertainment");
        
        // Shopping
        mappings.insert("amazon.com".to_string(), "Shopping");
        mappings.insert("ebay.com".to_string(), "Shopping");
        mappings.insert("aliexpress.com".to_string(), "Shopping");
        mappings.insert("etsy.com".to_string(), "Shopping");
        mappings.insert("shopify.com".to_string(), "Shopping");
        
        // News
        mappings.insert("cnn.com".to_string(), "News");
        mappings.insert("bbc.com".to_string(), "News");
        mappings.insert("nytimes.com".to_string(), "News");
        mappings.insert("theguardian.com".to_string(), "News");
        mappings.insert("reuters.com".to_string(), "News");
        
        // Productivity
        mappings.insert("google.com".to_string(), "Productivity");
        mappings.insert("docs.google.com".to_string(), "Productivity");
        mappings.insert("sheets.google.com".to_string(), "Productivity");
        mappings.insert("notion.so".to_string(), "Productivity");
        mappings.insert("trello.com".to_string(), "Productivity");
        mappings.insert("asana.com".to_string(), "Productivity");
        mappings.insert("slack.com".to_string(), "Productivity");
        mappings.insert("zoom.us".to_string(), "Productivity");
        
        // Education
        mappings.insert("coursera.org".to_string(), "Education");
        mappings.insert("udemy.com".to_string(), "Education");
        mappings.insert("edx.org".to_string(), "Education");
        mappings.insert("khanacademy.org".to_string(), "Education");
        mappings.insert("duolingo.com".to_string(), "Education");
        
        // Finance
        mappings.insert("bankofamerica.com".to_string(), "Finance");
        mappings.insert("paypal.com".to_string(), "Finance");
        mappings.insert("stripe.com".to_string(), "Finance");
        mappings.insert("coinbase.com".to_string(), "Finance");
        mappings.insert("robinhood.com".to_string(), "Finance");
        
        // Gaming
        mappings.insert("steam.com".to_string(), "Gaming");
        mappings.insert("epicgames.com".to_string(), "Gaming");
        mappings.insert("twitch.tv".to_string(), "Gaming");
        mappings.insert("discord.com".to_string(), "Gaming");
        
        mappings
    }

    /// Get category for a URL
    pub fn get_category(&self, url: &str) -> String {
        // Extract domain from URL
        let domain = self.extract_domain(url);
        
        // Look up in mappings
        if let Some(category) = self.category_mappings.get(&domain) {
            category.to_string()
        } else {
            // Try to match partial domain
            for (pattern, category) in &self.category_mappings {
                if domain.contains(pattern) || pattern.contains(&domain) {
                    return category.clone();
                }
            }
            "Other".to_string()
        }
    }

    /// Extract domain from URL
    fn extract_domain(&self, url: &str) -> String {
        // Simple domain extraction
        let url = url.trim();
        
        // Remove protocol
        let url = url.strip_prefix("https://")
            .or_else(|| url.strip_prefix("http://"))
            .unwrap_or(url);
        
        // Remove path and get domain
        let domain = url.split('/').next().unwrap_or(url);
        
        // Remove www prefix
        domain.strip_prefix("www.").unwrap_or(domain).to_string()
    }

    /// Generate usage trends data for line chart
    pub fn generate_usage_trends(
        &self,
        analytics: &ProfileAnalytics,
        date_range: &DateRange,
    ) -> Vec<TrendDataPoint> {
        let mut trends = Vec::new();
        
        // Sort daily usage by date
        let mut sorted_days: Vec<_> = analytics.daily_usage.iter().collect();
        sorted_days.sort_by(|a, b| a.0.cmp(b.0));
        
        // Filter by date range
        for (date, usage) in sorted_days {
            if let Ok(naive_date) = NaiveDate::parse_from_str(date, "%Y-%m-%d") {
                let datetime = naive_date.and_hms_opt(0, 0, 0).unwrap();
                let dt_utc = DateTime::<Utc>::from_utc(datetime, Utc);
                
                if dt_utc >= date_range.start && dt_utc <= date_range.end {
                    trends.push(TrendDataPoint {
                        label: date.clone(),
                        value: usage.time_spent,
                        secondary_value: Some(usage.sessions),
                    });
                }
            }
        }
        
        trends
    }

    /// Generate category distribution for pie chart
    pub fn generate_category_distribution(
        &self,
        analytics: &ProfileAnalytics,
    ) -> Vec<WebsiteCategory> {
        let mut categories: HashMap<String, WebsiteCategory> = HashMap::new();
        let total_time: u64 = analytics.top_websites.iter().map(|w| w.time_spent).sum();
        
        for website in &analytics.top_websites {
            let category_name = self.get_category(&website.url);
            
            let entry = categories.entry(category_name.clone()).or_insert(WebsiteCategory {
                name: category_name,
                icon: self.category_icon(&category_name),
                time_spent: 0,
                visits: 0,
                percentage: 0.0,
            });
            
            entry.time_spent += website.time_spent;
            entry.visits += website.visits;
        }
        
        // Calculate percentages
        let mut result: Vec<_> = categories.into_values().collect();
        for cat in &mut result {
            if total_time > 0 {
                cat.percentage = (cat.time_spent as f64 / total_time as f64) * 100.0;
            }
        }
        
        // Sort by time spent
        result.sort_by(|a, b| b.time_spent.cmp(&a.time_spent));
        
        result
    }

    /// Get icon for category
    fn category_icon(&self, category: &str) -> String {
        match category {
            "Social Media" => "📱".to_string(),
            "Development" => "💻".to_string(),
            "Entertainment" => "🎬".to_string(),
            "Shopping" => "🛒".to_string(),
            "News" => "📰".to_string(),
            "Productivity" => "📊".to_string(),
            "Education" => "🎓".to_string(),
            "Finance" => "💰".to_string(),
            "Gaming" => "🎮".to_string(),
            _ => "🌐".to_string(),
        }
    }

    /// Generate activity heatmap data
    pub fn generate_heatmap(
        &self,
        analytics: &ProfileAnalytics,
    ) -> Vec<HeatmapCell> {
        let mut heatmap = Vec::new();
        let mut hourly_data: HashMap<(u8, u8), u64> = HashMap::new();
        
        // Aggregate hourly usage from daily data
        // In a real implementation, this would use more granular data
        for (_, daily) in &analytics.daily_usage {
            // Simulate hourly distribution based on daily usage
            // Real implementation would track actual hourly data
            let base_hours = 8; // Active hours per day
            let time_per_hour = daily.time_spent / base_hours;
            
            // Distribute across typical active hours (9 AM - 6 PM)
            for hour in 9..=18u8 {
                let day = self.get_day_of_week(&daily.date);
                *hourly_data.entry((hour, day)).or_insert(0) += time_per_hour;
            }
        }
        
        // Calculate max for intensity normalization
        let max_value = hourly_data.values().copied().max().unwrap_or(1);
        
        // Generate heatmap cells
        for day in 0..7u8 {
            for hour in 0..24u8 {
                let value = hourly_data.get(&(hour, day)).copied().unwrap_or(0);
                let intensity = if max_value > 0 {
                    value as f64 / max_value as f64
                } else {
                    0.0
                };
                
                heatmap.push(HeatmapCell {
                    hour,
                    day,
                    intensity,
                    value,
                });
            }
        }
        
        heatmap
    }

    /// Get day of week from date string
    fn get_day_of_week(&self, date_str: &str) -> u8 {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            date.weekday().num_days_from_sunday()
        } else {
            0
        }
    }

    /// Generate top categories bar chart data
    pub fn generate_top_categories(
        &self,
        analytics: &ProfileAnalytics,
        limit: usize,
    ) -> Vec<WebsiteCategory> {
        let categories = self.generate_category_distribution(analytics);
        categories.into_iter().take(limit).collect()
    }

    /// Generate profile comparison data
    pub fn generate_comparison(
        &self,
        primary: &ProfileAnalytics,
        secondary: &ProfileAnalytics,
    ) -> ProfileComparison {
        let metrics = ComparisonMetrics {
            total_time_diff: primary.total_time as i64 - secondary.total_time as i64,
            session_count_diff: primary.session_count as i64 - secondary.session_count as i64,
            websites_visited_diff: primary.top_websites.len() as i64 - secondary.top_websites.len() as i64,
            avg_session_time_diff: if primary.session_count > 0 && secondary.session_count > 0 {
                (primary.total_time / primary.session_count) as i64 
                    - (secondary.total_time / secondary.session_count) as i64
            } else {
                0
            },
            tabs_opened_diff: primary.tab_stats.total_opened as i64 
                - secondary.tab_stats.total_opened as i64,
            performance_diff: primary.performance.avg_page_load_time as f64 
                - secondary.performance.avg_page_load_time as f64,
        };

        ProfileComparison {
            primary_profile: primary.profile_id.clone(),
            secondary_profile: secondary.profile_id.clone(),
            metrics,
        }
    }

    /// Generate analytics report for export
    pub fn generate_report(
        &self,
        analytics: &ProfileAnalytics,
        profile_name: &str,
        date_range: DateRange,
    ) -> AnalyticsReport {
        let summary = UsageSummary {
            profile_id: analytics.profile_id.clone(),
            total_time: analytics.total_time,
            session_count: analytics.session_count,
            avg_session_time: if analytics.session_count > 0 {
                analytics.total_time / analytics.session_count
            } else {
                0
            },
            avg_daily_time: if !analytics.daily_usage.is_empty() {
                let total: u64 = analytics.daily_usage.values().map(|d| d.time_spent).sum();
                total / analytics.daily_usage.len() as u64
            } else {
                0
            },
            total_websites: analytics.top_websites.len() as u64,
            total_tabs_opened: analytics.tab_stats.total_opened,
            avg_page_load_time: analytics.performance.avg_page_load_time,
            total_crashes: analytics.performance.total_crashes,
        };

        AnalyticsReport {
            title: format!("Analytics Report - {}", profile_name),
            generated_at: Utc::now(),
            profile_id: analytics.profile_id.clone(),
            profile_name: profile_name.to_string(),
            date_range,
            summary,
            trends: self.generate_usage_trends(analytics, &DateRange::last_days(30)),
            categories: self.generate_category_distribution(analytics),
            heatmap: self.generate_heatmap(analytics),
            top_websites: analytics.top_websites.clone(),
            daily_breakdown: analytics.daily_usage.values().cloned().collect(),
        }
    }

    /// Export report to specified format
    pub fn export_report(
        &self,
        report: &AnalyticsReport,
        format: ExportFormat,
    ) -> Result<Vec<u8>> {
        match format {
            ExportFormat::Json => {
                let json = serde_json::to_string_pretty(report)
                    .map_err(|e| anyhow::anyhow!("Failed to serialize report: {}", e))?;
                Ok(json.into_bytes())
            }
            ExportFormat::Csv => {
                let csv = self.report_to_csv(report)?;
                Ok(csv.into_bytes())
            }
            ExportFormat::Html => {
                let html = self.report_to_html(report)?;
                Ok(html.into_bytes())
            }
            ExportFormat::Pdf => {
                // PDF generation would require additional dependencies
                // For now, return HTML that can be converted to PDF
                let html = self.report_to_html(report)?;
                Ok(html.into_bytes())
            }
        }
    }

    /// Convert report to CSV format
    fn report_to_csv(&self, report: &AnalyticsReport) -> Result<String> {
        let mut csv = String::new();
        
        // Header
        csv.push_str("VantisWeb Analytics Report\n");
        csv.push_str(&format!("Profile: {}\n", report.profile_name));
        csv.push_str(&format!("Generated: {}\n\n", report.generated_at));
        
        // Summary section
        csv.push_str("=== SUMMARY ===\n");
        csv.push_str("Metric,Value\n");
        csv.push_str(&format!("Total Time (minutes),{}\n", report.summary.total_time / 60));
        csv.push_str(&format!("Sessions,{}\n", report.summary.session_count));
        csv.push_str(&format!("Avg Session (minutes),{}\n", report.summary.avg_session_time / 60));
        csv.push_str(&format!("Websites Visited,{}\n", report.summary.total_websites));
        csv.push_str(&format!("Tabs Opened,{}\n", report.summary.total_tabs_opened));
        csv.push_str(&format!("Avg Page Load (ms),{}\n", report.summary.avg_page_load_time));
        
        // Top websites section
        csv.push_str("\n=== TOP WEBSITES ===\n");
        csv.push_str("Rank,URL,Visits,Time (minutes)\n");
        for (i, site) in report.top_websites.iter().enumerate() {
            csv.push_str(&format!("{},{},{},{}\n", 
                i + 1,
                site.url,
                site.visits,
                site.time_spent / 60
            ));
        }
        
        // Categories section
        csv.push_str("\n=== CATEGORY DISTRIBUTION ===\n");
        csv.push_str("Category,Time (minutes),Visits,Percentage\n");
        for cat in &report.categories {
            csv.push_str(&format!("{},{},{},{:.1}%\n",
                cat.name,
                cat.time_spent / 60,
                cat.visits,
                cat.percentage
            ));
        }
        
        Ok(csv)
    }

    /// Convert report to HTML format
    fn report_to_html(&self, report: &AnalyticsReport) -> Result<String> {
        let mut html = String::new();
        
        html.push_str(r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>VantisWeb Analytics Report</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #0a0a0a; color: #fff; padding: 20px; }
        .container { max-width: 1200px; margin: 0 auto; }
        .header { text-align: center; padding: 40px 0; border-bottom: 1px solid #333; margin-bottom: 40px; }
        .header h1 { font-size: 2.5em; color: #dc2626; margin-bottom: 10px; }
        .header p { color: #888; }
        .summary-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin-bottom: 40px; }
        .summary-card { background: linear-gradient(135deg, #1a1a1a, #0a0a0a); border: 1px solid #333; border-radius: 12px; padding: 24px; text-align: center; }
        .summary-card h3 { color: #dc2626; font-size: 2em; margin-bottom: 8px; }
        .summary-card p { color: #888; font-size: 0.9em; }
        .section { margin-bottom: 40px; }
        .section h2 { color: #fff; margin-bottom: 20px; padding-bottom: 10px; border-bottom: 2px solid #dc2626; }
        .websites-table { width: 100%; border-collapse: collapse; }
        .websites-table th, .websites-table td { padding: 12px; text-align: left; border-bottom: 1px solid #333; }
        .websites-table th { color: #dc2626; font-weight: 600; }
        .websites-table tr:hover { background: rgba(220, 38, 38, 0.1); }
        .category-bar { display: flex; align-items: center; margin-bottom: 12px; }
        .category-bar .label { width: 150px; font-weight: 500; }
        .category-bar .bar-container { flex: 1; height: 24px; background: #1a1a1a; border-radius: 4px; overflow: hidden; }
        .category-bar .bar { height: 100%; background: linear-gradient(90deg, #dc2626, #ef4444); border-radius: 4px; }
        .category-bar .percentage { width: 60px; text-align: right; color: #888; }
        .footer { text-align: center; padding: 40px 0; border-top: 1px solid #333; margin-top: 40px; color: #666; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🔥 VantisWeb Analytics Report</h1>
            <p>Profile: "##);
        html.push_str(&report.profile_name);
        html.push_str(r##" | Generated: "##);
        html.push_str(&report.generated_at.format("%Y-%m-%d %H:%M UTC").to_string());
        html.push_str(r##"</p>
        </div>
        
        <div class="section">
            <h2>📊 Summary</h2>
            <div class="summary-grid">
                <div class="summary-card">
                    <h3>"##);
        html.push_str(&format!("{}", report.summary.total_time / 3600));
        html.push_str(r##"</h3>
                    <p>Hours Total</p>
                </div>
                <div class="summary-card">
                    <h3>"##);
        html.push_str(&format!("{}", report.summary.session_count));
        html.push_str(r##"</h3>
                    <p>Sessions</p>
                </div>
                <div class="summary-card">
                    <h3>"##);
        html.push_str(&format!("{}", report.summary.avg_session_time / 60));
        html.push_str(r##"</h3>
                    <p>Avg Session (min)</p>
                </div>
                <div class="summary-card">
                    <h3>"##);
        html.push_str(&format!("{}", report.summary.total_websites));
        html.push_str(r##"</h3>
                    <p>Websites Visited</p>
                </div>
                <div class="summary-card">
                    <h3>"##);
        html.push_str(&format!("{}", report.summary.total_tabs_opened));
        html.push_str(r##"</h3>
                    <p>Tabs Opened</p>
                </div>
                <div class="summary-card">
                    <h3>"##);
        html.push_str(&format!("{}ms", report.summary.avg_page_load_time));
        html.push_str(r##"</h3>
                    <p>Avg Page Load</p>
                </div>
            </div>
        </div>
        
        <div class="section">
            <h2>🌐 Top Websites</h2>
            <table class="websites-table">
                <thead>
                    <tr>
                        <th>#</th>
                        <th>Website</th>
                        <th>Visits</th>
                        <th>Time (min)</th>
                    </tr>
                </thead>
                <tbody>
"##);

        for (i, site) in report.top_websites.iter().enumerate() {
            html.push_str(&format!(
                "                    <tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                i + 1,
                site.url,
                site.visits,
                site.time_spent / 60
            ));
        }

        html.push_str(r##"                </tbody>
            </table>
        </div>
        
        <div class="section">
            <h2>📁 Category Distribution</h2>
"##);

        for cat in &report.categories {
            html.push_str(&format!(
                r##"            <div class="category-bar">
                <div class="label">{} {}</div>
                <div class="bar-container">
                    <div class="bar" style="width: {:.1}%"></div>
                </div>
                <div class="percentage">{:.1}%</div>
            </div>
"##,
                cat.icon,
                cat.name,
                cat.percentage,
                cat.percentage
            ));
        }

        html.push_str(r##"        </div>
        
        <div class="footer">
            <p>Generated by VantisWeb Browser Analytics</p>
        </div>
    </div>
</body>
</html>"##);

        Ok(html)
    }

    /// Add custom category mapping
    pub fn add_category_mapping(&mut self, domain: String, category: String) {
        self.category_mappings.insert(domain, category);
    }

    /// Set multiple category mappings
    pub fn set_category_mappings(&mut self, mappings: HashMap<String, String>) {
        self.category_mappings.extend(mappings);
    }
}

impl Default for VisualizationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::analytics::*;
    use chrono::Utc;

    fn create_test_analytics_with_data() -> ProfileAnalytics {
        let mut daily_usage = HashMap::new();
        daily_usage.insert("2024-01-01".to_string(), DailyUsage {
            date: "2024-01-01".to_string(),
            time_spent: 3600,
            sessions: 2,
            websites_visited: 10,
        });
        let mut top_websites = Vec::new();
        top_websites.push(WebsiteUsage {
            url: "https://github.com".to_string(),
            visits: 20,
            time_spent: 1800,
            last_visit: Utc::now(),
        });
        ProfileAnalytics {
            profile_id: "test-full".to_string(),
            total_time: 3600,
            session_count: 2,
            first_session: None,
            last_session: None,
            daily_usage,
            top_websites,
            tab_stats: super::super::analytics::TabStatistics {
                total_opened: 20,
                average_per_session: 10.0,
                max_open: 12,
                current_open: 2,
            },
            performance: super::super::analytics::PerformanceMetrics {
                avg_page_load_time: 100,
                total_crashes: 0,
                memory_usage: 400,
                cpu_usage: 20.0,
            },
        }
    }

    #[test]
    fn test_visualization_manager_creation() {
        let manager = VisualizationManager::new();
        assert!(!manager.category_mappings.is_empty());
    }

    #[test]
    fn test_category_detection() {
        let manager = VisualizationManager::new();
        
        assert_eq!(manager.get_category("https://github.com"), "Development");
        assert_eq!(manager.get_category("https://youtube.com"), "Entertainment");
        assert_eq!(manager.get_category("https://unknown-site.xyz"), "Other");
    }

    #[test]
    fn test_date_range_creation() {
        let range = DateRange::last_days(7);
        assert!(range.end > range.start);
    }

    #[test]
    fn test_category_distribution() {
        let manager = VisualizationManager::new();
        let mut analytics = ProfileAnalytics {
            profile_id: "test".to_string(),
            total_time: 1000,
            session_count: 5,
            first_session: None,
            last_session: None,
            daily_usage: HashMap::new(),
            top_websites: vec![
                WebsiteUsage {
                    url: "https://github.com".to_string(),
                    visits: 10,
                    time_spent: 500,
                    last_visit: Utc::now(),
                },
                WebsiteUsage {
                    url: "https://youtube.com".to_string(),
                    visits: 5,
                    time_spent: 300,
                    last_visit: Utc::now(),
                },
            ],
            tab_stats: super::super::analytics::TabStatistics {
                total_opened: 50,
                average_per_session: 10.0,
                max_open: 15,
                current_open: 3,
            },
            performance: super::super::analytics::PerformanceMetrics {
                avg_page_load_time: 150,
                total_crashes: 0,
                memory_usage: 500,
                cpu_usage: 25.0,
            },
        };

        let categories = manager.generate_category_distribution(&analytics);
        assert!(!categories.is_empty());
        
        // Should have Development and Entertainment categories
        let category_names: Vec<&str> = categories.iter().map(|c| c.name.as_str()).collect();
        assert!(category_names.contains(&"Development"));
        assert!(category_names.contains(&"Entertainment"));
    }

    #[test]
    fn test_report_generation() {
        let manager = VisualizationManager::new();
        let analytics = ProfileAnalytics {
            profile_id: "test".to_string(),
            total_time: 3600,
            session_count: 10,
            first_session: None,
            last_session: None,
            daily_usage: HashMap::new(),
            top_websites: vec![],
            tab_stats: super::super::analytics::TabStatistics {
                total_opened: 100,
                average_per_session: 10.0,
                max_open: 20,
                current_open: 5,
            },
            performance: super::super::analytics::PerformanceMetrics {
                avg_page_load_time: 200,
                total_crashes: 0,
                memory_usage: 600,
                cpu_usage: 30.0,
            },
        };

        let report = manager.generate_report(&analytics, "Test Profile", DateRange::last_days(7));
        
        assert_eq!(report.profile_name, "Test Profile");
        assert_eq!(report.summary.session_count, 10);
    }

    #[test]
    fn test_export_json() {
        let manager = VisualizationManager::new();
        let analytics = ProfileAnalytics {
            profile_id: "test".to_string(),
            total_time: 3600,
            session_count: 10,
            first_session: None,
            last_session: None,
            daily_usage: HashMap::new(),
            top_websites: vec![],
            tab_stats: super::super::analytics::TabStatistics {
                total_opened: 100,
                average_per_session: 10.0,
                max_open: 20,
                current_open: 5,
            },
            performance: super::super::analytics::PerformanceMetrics {
                avg_page_load_time: 200,
                total_crashes: 0,
                memory_usage: 600,
                cpu_usage: 30.0,
            },
        };

        let report = manager.generate_report(&analytics, "Test", DateRange::last_days(7));
        let result = manager.export_report(&report, ExportFormat::Json);
        
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(!data.is_empty());
    }

    #[test]
    fn test_export_csv() {
        let manager = VisualizationManager::new();
        let analytics = ProfileAnalytics {
            profile_id: "test".to_string(),
            total_time: 3600,
            session_count: 10,
            first_session: None,
            last_session: None,
            daily_usage: HashMap::new(),
            top_websites: vec![],
            tab_stats: super::super::analytics::TabStatistics {
                total_opened: 100,
                average_per_session: 10.0,
                max_open: 20,
                current_open: 5,
            },
            performance: super::super::analytics::PerformanceMetrics {
                avg_page_load_time: 200,
                total_crashes: 0,
                memory_usage: 600,
                cpu_usage: 30.0,
            },
        };

        let report = manager.generate_report(&analytics, "Test", DateRange::last_days(7));
        let result = manager.export_report(&report, ExportFormat::Csv);
        
        assert!(result.is_ok());
        let data = String::from_utf8(result.unwrap()).unwrap();
        assert!(data.contains("SUMMARY"));
    }
}