//! Analytics data export module
//! 
//! This module provides functionality to export analytics data
//! in various formats (JSON, CSV, PDF) for user analysis and reporting.

use std::path::Path;
use std::fs::File;
use std::io::{BufWriter, Write};
use serde::{Serialize, Serializer};
use serde_json::json;
use chrono::{DateTime, Utc, NaiveDate};

use super::metrics::{UsageMetrics, PerformanceMetrics, SecurityMetrics, AnalyticsSnapshot};
use super::privacy::PrivacyFilter;

/// Export format options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportFormat {
    JSON,
    JSONPretty,
    CSV,
    HTML,
    Markdown,
}

/// Time range for export
#[derive(Debug, Clone)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl TimeRange {
    /// Create a time range for the last N days
    pub fn last_days(days: i64) -> Self {
        let end = Utc::now();
        let start = end - chrono::Duration::days(days);
        Self { start, end }
    }
    
    /// Create a time range for the last N hours
    pub fn last_hours(hours: i64) -> Self {
        let end = Utc::now();
        let start = end - chrono::Duration::hours(hours);
        Self { start, end }
    }
    
    /// Create a time range for today
    pub fn today() -> Self {
        let now = Utc::now();
        let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
        Self {
            start: DateTime::from_naive_utc_and_offset(start, Utc),
            end: now,
        }
    }
    
    /// Create a time range for this week
    pub fn this_week() -> Self {
        let now = Utc::now();
        let days_since_monday = now.weekday().num_days_from_monday() as i64;
        let start = (now - chrono::Duration::days(days_since_monday))
            .date_naive()
            .and_hms_opt(0, 0, 0).unwrap();
        Self {
            start: DateTime::from_naive_utc_and_offset(start, Utc),
            end: now,
        }
    }
    
    /// Create a time range for this month
    pub fn this_month() -> Self {
        let now = Utc::now();
        let start = now.date_naive()
            .with_day(1).unwrap()
            .and_hms_opt(0, 0, 0).unwrap();
        Self {
            start: DateTime::from_naive_utc_and_offset(start, Utc),
            end: now,
        }
    }
}

/// Exportable analytics report
#[derive(Debug, Serialize)]
pub struct AnalyticsReport {
    pub generated_at: DateTime<Utc>,
    pub time_range: TimeRange,
    pub usage: UsageMetrics,
    pub performance: PerformanceMetrics,
    pub security: SecurityMetrics,
    pub insights: Vec<Insight>,
}

/// An insight derived from analytics data
#[derive(Debug, Serialize)]
pub struct Insight {
    pub category: InsightCategory,
    pub title: String,
    pub description: String,
    pub impact: ImpactLevel,
    pub actionable: bool,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub enum InsightCategory {
    Performance,
    Security,
    Productivity,
    Privacy,
    Usage,
}

#[derive(Debug, Clone, Serialize)]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Analytics data exporter
pub struct AnalyticsExporter {
    privacy_filter: PrivacyFilter,
    output_path: Option<std::path::PathBuf>,
}

impl AnalyticsExporter {
    /// Create a new exporter
    pub fn new(privacy_filter: PrivacyFilter) -> Self {
        Self {
            privacy_filter,
            output_path: None,
        }
    }
    
    /// Create an exporter with default privacy filter
    pub fn with_defaults() -> Self {
        Self::new(PrivacyFilter::with_defaults())
    }
    
    /// Set output path for exports
    pub fn with_output_path(mut self, path: impl AsRef<Path>) -> Self {
        self.output_path = Some(path.as_ref().to_path_buf());
        self
    }
    
    /// Export analytics snapshot to the specified format
    pub fn export(
        &self,
        snapshot: &AnalyticsSnapshot,
        format: ExportFormat,
        path: impl AsRef<Path>,
    ) -> Result<(), ExportError> {
        let path = path.as_ref();
        
        match format {
            ExportFormat::JSON => self.export_json(snapshot, path, false),
            ExportFormat::JSONPretty => self.export_json(snapshot, path, true),
            ExportFormat::CSV => self.export_csv(snapshot, path),
            ExportFormat::HTML => self.export_html(snapshot, path),
            ExportFormat::Markdown => self.export_markdown(snapshot, path),
        }
    }
    
    /// Export to JSON format
    fn export_json(
        &self,
        snapshot: &AnalyticsSnapshot,
        path: &Path,
        pretty: bool,
    ) -> Result<(), ExportError> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        
        let json_value = serde_json::to_value(snapshot)?;
        let output = if pretty {
            serde_json::to_string_pretty(&json_value)?
        } else {
            serde_json::to_string(&json_value)?
        };
        
        writer.write_all(output.as_bytes())?;
        writer.flush()?;
        
        Ok(())
    }
    
    /// Export to CSV format
    fn export_csv(
        &self,
        snapshot: &AnalyticsSnapshot,
        path: &Path,
    ) -> Result<(), ExportError> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        
        // Write header
        writeln!(writer, "category,metric,value,unit,timestamp")?;
        
        // Write usage metrics
        let usage = &snapshot.usage;
        writeln!(writer, "usage,total_visits,{},count,{}", usage.total_visits, snapshot.timestamp)?;
        writeln!(writer, "usage,unique_domains,{},count,{}", usage.unique_domains, snapshot.timestamp)?;
        writeln!(writer, "usage,total_time,{},ms,{}", usage.total_time_ms, snapshot.timestamp)?;
        writeln!(writer, "usage,average_page_time,{},ms,{}", usage.average_page_time_ms, snapshot.timestamp)?;
        
        // Write performance metrics
        let perf = &snapshot.performance;
        writeln!(writer, "performance,average_load_time,{},ms,{}", perf.average_load_time_ms, snapshot.timestamp)?;
        writeln!(writer, "performance,slow_pages,{},count,{}", perf.slow_pages, snapshot.timestamp)?;
        writeln!(writer, "performance,average_memory,{},mb,{}", perf.average_memory_mb, snapshot.timestamp)?;
        writeln!(writer, "performance,average_cpu,{},percent,{}", perf.average_cpu_percent, snapshot.timestamp)?;
        
        // Write security metrics
        let sec = &snapshot.security;
        writeln!(writer, "security,threats_blocked,{},count,{}", sec.threats_blocked, snapshot.timestamp)?;
        writeln!(writer, "security,https_upgrades,{},count,{}", sec.https_upgrades, snapshot.timestamp)?;
        writeln!(writer, "security,trackers_blocked,{},count,{}", sec.trackers_blocked, snapshot.timestamp)?;
        
        writer.flush()?;
        
        Ok(())
    }
    
    /// Export to HTML format with interactive charts
    fn export_html(
        &self,
        snapshot: &AnalyticsSnapshot,
        path: &Path,
    ) -> Result<(), ExportError> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        
        let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>VantisWeb Analytics Report</title>
    <style>
        :root {{
            --primary: #4F46E5;
            --success: #10B981;
            --warning: #F59E0B;
            --danger: #EF4444;
            --bg: #F9FAFB;
            --card: #FFFFFF;
            --text: #1F2937;
        }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: var(--bg);
            color: var(--text);
            margin: 0;
            padding: 20px;
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
        }}
        h1 {{
            color: var(--primary);
            margin-bottom: 30px;
        }}
        .grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }}
        .card {{
            background: var(--card);
            border-radius: 12px;
            padding: 20px;
            box-shadow: 0 1px 3px rgba(0,0,0,0.1);
        }}
        .card h2 {{
            margin-top: 0;
            color: var(--primary);
            font-size: 1.1rem;
        }}
        .metric {{
            display: flex;
            justify-content: space-between;
            padding: 10px 0;
            border-bottom: 1px solid #E5E7EB;
        }}
        .metric:last-child {{
            border-bottom: none;
        }}
        .metric-value {{
            font-weight: 600;
            color: var(--primary);
        }}
        .insight {{
            padding: 15px;
            border-radius: 8px;
            margin-bottom: 10px;
        }}
        .insight.high {{ background: #FEF3C7; border-left: 4px solid var(--warning); }}
        .insight.critical {{ background: #FEE2E2; border-left: 4px solid var(--danger); }}
        .insight.low {{ background: #ECFDF5; border-left: 4px solid var(--success); }}
        .footer {{
            text-align: center;
            color: #6B7280;
            margin-top: 30px;
            font-size: 0.9rem;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🌐 VantisWeb Analytics Report</h1>
        <p>Generated: {}</p>
        
        <div class="grid">
            <div class="card">
                <h2>📊 Usage Metrics</h2>
                <div class="metric"><span>Total Visits</span><span class="metric-value">{}</span></div>
                <div class="metric"><span>Unique Domains</span><span class="metric-value">{}</span></div>
                <div class="metric"><span>Total Time</span><span class="metric-value">{}</span></div>
                <div class="metric"><span>Avg Page Time</span><span class="metric-value">{} ms</span></div>
            </div>
            
            <div class="card">
                <h2>⚡ Performance</h2>
                <div class="metric"><span>Avg Load Time</span><span class="metric-value">{} ms</span></div>
                <div class="metric"><span>Slow Pages</span><span class="metric-value">{}</span></div>
                <div class="metric"><span>Avg Memory</span><span class="metric-value">{} MB</span></div>
                <div class="metric"><span>Avg CPU</span><span class="metric-value">{}%</span></div>
            </div>
            
            <div class="card">
                <h2>🔒 Security</h2>
                <div class="metric"><span>Threats Blocked</span><span class="metric-value">{}</span></div>
                <div class="metric"><span>HTTPS Upgrades</span><span class="metric-value">{}</span></div>
                <div class="metric"><span>Trackers Blocked</span><span class="metric-value">{}</span></div>
                <div class="metric"><span>Cookies Blocked</span><span class="metric-value">{}</span></div>
            </div>
        </div>
        
        <div class="card">
            <h2>💡 Insights</h2>
            <p>Analytics insights and recommendations will appear here.</p>
        </div>
        
        <div class="footer">
            VantisWeb Analytics • Privacy-First Browser
        </div>
    </div>
</body>
</html>"#,
            snapshot.timestamp,
            snapshot.usage.total_visits,
            snapshot.usage.unique_domains,
            format_duration(snapshot.usage.total_time_ms),
            snapshot.usage.average_page_time_ms,
            snapshot.performance.average_load_time_ms,
            snapshot.performance.slow_pages,
            snapshot.performance.average_memory_mb,
            snapshot.performance.average_cpu_percent,
            snapshot.security.threats_blocked,
            snapshot.security.https_upgrades,
            snapshot.security.trackers_blocked,
            snapshot.security.cookies_blocked,
        );
        
        writer.write_all(html.as_bytes())?;
        writer.flush()?;
        
        Ok(())
    }
    
    /// Export to Markdown format
    fn export_markdown(
        &self,
        snapshot: &AnalyticsSnapshot,
        path: &Path,
    ) -> Result<(), ExportError> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        
        writeln!(writer, "# VantisWeb Analytics Report")?;
        writeln!(writer)?;
        writeln!(writer, "**Generated:** {}", snapshot.timestamp)?;
        writeln!(writer)?;
        
        writeln!(writer, "## 📊 Usage Metrics")?;
        writeln!(writer)?;
        writeln!(writer, "| Metric | Value |")?;
        writeln!(writer, "|--------|-------|")?;
        writeln!(writer, "| Total Visits | {} |", snapshot.usage.total_visits)?;
        writeln!(writer, "| Unique Domains | {} |", snapshot.usage.unique_domains)?;
        writeln!(writer, "| Total Time | {} |", format_duration(snapshot.usage.total_time_ms))?;
        writeln!(writer, "| Avg Page Time | {} ms |", snapshot.usage.average_page_time_ms)?;
        writeln!(writer)?;
        
        writeln!(writer, "## ⚡ Performance Metrics")?;
        writeln!(writer)?;
        writeln!(writer, "| Metric | Value |")?;
        writeln!(writer, "|--------|-------|")?;
        writeln!(writer, "| Avg Load Time | {} ms |", snapshot.performance.average_load_time_ms)?;
        writeln!(writer, "| Slow Pages | {} |", snapshot.performance.slow_pages)?;
        writeln!(writer, "| Avg Memory | {} MB |", snapshot.performance.average_memory_mb)?;
        writeln!(writer, "| Avg CPU | {}% |", snapshot.performance.average_cpu_percent)?;
        writeln!(writer)?;
        
        writeln!(writer, "## 🔒 Security Metrics")?;
        writeln!(writer)?;
        writeln!(writer, "| Metric | Value |")?;
        writeln!(writer, "|--------|-------|")?;
        writeln!(writer, "| Threats Blocked | {} |", snapshot.security.threats_blocked)?;
        writeln!(writer, "| HTTPS Upgrades | {} |", snapshot.security.https_upgrades)?;
        writeln!(writer, "| Trackers Blocked | {} |", snapshot.security.trackers_blocked)?;
        writeln!(writer, "| Cookies Blocked | {} |", snapshot.security.cookies_blocked)?;
        writeln!(writer)?;
        
        writeln!(writer, "---")?;
        writeln!(writer)?;
        writeln!(writer, "*VantisWeb Analytics • Privacy-First Browser*")?;
        
        writer.flush()?;
        
        Ok(())
    }
    
    /// Generate insights from analytics data
    pub fn generate_insights(&self, snapshot: &AnalyticsSnapshot) -> Vec<Insight> {
        let mut insights = Vec::new();
        
        // Performance insights
        if snapshot.performance.average_load_time_ms > 3000 {
            insights.push(Insight {
                category: InsightCategory::Performance,
                title: "Slow page load times detected".to_string(),
                description: format!(
                    "Average page load time is {}ms, which is above the recommended 3000ms threshold.",
                    snapshot.performance.average_load_time_ms
                ),
                impact: ImpactLevel::High,
                actionable: true,
                recommendations: vec![
                    "Consider enabling hardware acceleration".to_string(),
                    "Clear browser cache and cookies".to_string(),
                    "Disable unnecessary extensions".to_string(),
                ],
            });
        }
        
        // Memory insights
        if snapshot.performance.average_memory_mb > 1000 {
            insights.push(Insight {
                category: InsightCategory::Performance,
                title: "High memory usage".to_string(),
                description: format!(
                    "Average memory usage is {}MB. Consider closing unused tabs.",
                    snapshot.performance.average_memory_mb
                ),
                impact: ImpactLevel::Medium,
                actionable: true,
                recommendations: vec![
                    "Close unused tabs to free memory".to_string(),
                    "Enable memory saver mode".to_string(),
                    "Check for memory-heavy extensions".to_string(),
                ],
            });
        }
        
        // Security insights
        if snapshot.security.threats_blocked > 0 {
            insights.push(Insight {
                category: InsightCategory::Security,
                title: "Security threats blocked".to_string(),
                description: format!(
                    "{} potential threats were blocked while browsing.",
                    snapshot.security.threats_blocked
                ),
                impact: ImpactLevel::Low,
                actionable: false,
                recommendations: vec![],
            });
        }
        
        // Privacy insights
        if snapshot.security.trackers_blocked > 50 {
            insights.push(Insight {
                category: InsightCategory::Privacy,
                title: "High tracker activity".to_string(),
                description: format!(
                    "{} tracking attempts were blocked. Your privacy is being protected.",
                    snapshot.security.trackers_blocked
                ),
                impact: ImpactLevel::Medium,
                actionable: true,
                recommendations: vec![
                    "Review tracker blocking settings".to_string(),
                    "Consider stricter privacy mode".to_string(),
                ],
            });
        }
        
        // Productivity insights
        let hours_browsed = snapshot.usage.total_time_ms as f64 / (1000.0 * 60.0 * 60.0);
        if hours_browsed > 8.0 {
            insights.push(Insight {
                category: InsightCategory::Productivity,
                title: "Extended browsing session".to_string(),
                description: format!(
                    "You've browsed for {:.1} hours. Consider taking a break.",
                    hours_browsed
                ),
                impact: ImpactLevel::Low,
                actionable: true,
                recommendations: vec![
                    "Take regular breaks from screen time".to_string(),
                    "Enable focus mode for distraction-free browsing".to_string(),
                ],
            });
        }
        
        insights
    }
}

/// Format duration in human-readable format
fn format_duration(ms: u64) -> String {
    let seconds = ms / 1000;
    let minutes = seconds / 60;
    let hours = minutes / 60;
    
    if hours > 0 {
        format!("{}h {}m", hours, minutes % 60)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds % 60)
    } else {
        format!("{}s", seconds)
    }
}

/// Export error types
#[derive(Debug)]
pub enum ExportError {
    Io(std::io::Error),
    Json(serde_json::Error),
    InvalidPath,
}

impl From<std::io::Error> for ExportError {
    fn from(e: std::io::Error) -> Self {
        ExportError::Io(e)
    }
}

impl From<serde_json::Error> for ExportError {
    fn from(e: serde_json::Error) -> Self {
        ExportError::Json(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_time_range_last_days() {
        let range = TimeRange::last_days(7);
        let duration = range.end - range.start;
        assert_eq!(duration.num_days(), 7);
    }
    
    #[test]
    fn test_time_range_today() {
        let range = TimeRange::today();
        assert!(range.start < range.end);
    }
    
    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(5000), "5s");
        assert_eq!(format_duration(65000), "1m 5s");
        assert_eq!(format_duration(3665000), "1h 1m");
    }
    
    #[test]
    fn test_exporter_creation() {
        let exporter = AnalyticsExporter::with_defaults();
        assert!(exporter.output_path.is_none());
    }
}