/// # History Visualization Module
/// 
/// Provides visualization data generation for browser history.
/// Generates data for timelines, charts, heatmaps, and other visual representations.

use std::collections::HashMap;
use chrono::{DateTime, Utc, Datelike, Timelike, Weekday};
use serde::{Deserialize, Serialize};

use super::{HistoryEntry, Result};

/// Visualization types supported
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VisualizationType {
    /// Timeline view
    Timeline,
    /// Daily activity chart
    DailyActivity,
    /// Weekly heatmap
    WeeklyHeatmap,
    /// Monthly heatmap
    MonthlyHeatmap,
    /// Category pie chart
    CategoryPie,
    /// Domain bar chart
    DomainBar,
    /// Hourly distribution
    HourlyDistribution,
    /// Visit frequency graph
    VisitFrequency,
}

/// Visualization data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationData {
    /// Visualization type
    pub viz_type: VisualizationType,
    /// Title
    pub title: String,
    /// Data points
    pub data: DataPoints,
    /// Metadata
    pub metadata: VisualizationMetadata,
}

/// Data points for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataPoints {
    /// Timeline data
    Timeline(Vec<TimelinePoint>),
    /// Chart data
    Chart(Vec<ChartPoint>),
    /// Heatmap data
    Heatmap(Vec<HeatmapCell>),
    /// Pie data
    Pie(Vec<PieSlice>),
    /// Bar data
    Bar(Vec<BarData>),
}

/// Visualization metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationMetadata {
    /// Total data points
    pub total_points: usize,
    /// Date range
    pub date_range: Option<DateRange>,
    /// Unit
    pub unit: String,
}

/// Date range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Timeline data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelinePoint {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// URL
    pub url: String,
    /// Title
    pub title: String,
    /// Visit count
    pub visits: u32,
    /// Domain
    pub domain: String,
    /// Category
    pub category: Option<String>,
}

/// Chart data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartPoint {
    /// Label
    pub label: String,
    /// Value
    pub value: f64,
    /// Secondary value (for stacked charts)
    pub secondary_value: Option<f64>,
    /// Color
    pub color: Option<String>,
}

/// Heatmap cell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapCell {
    /// Row identifier (e.g., day of week)
    pub row: String,
    /// Column identifier (e.g., hour)
    pub column: String,
    /// Value
    pub value: f64,
    /// Normalized value (0-1)
    pub normalized: f64,
}

/// Pie slice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PieSlice {
    /// Label
    pub label: String,
    /// Value
    pub value: f64,
    /// Percentage
    pub percentage: f64,
    /// Color
    pub color: String,
}

/// Bar chart data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarData {
    /// Label
    pub label: String,
    /// Values (for grouped bars)
    pub values: Vec<f64>,
    /// Total
    pub total: f64,
}

/// History visualization engine
pub struct HistoryVisualization {
    /// Color palette
    palette: Vec<String>,
}

impl HistoryVisualization {
    /// Create a new visualization engine
    pub fn new() -> Self {
        Self {
            palette: vec![
                "#4E79A7".to_string(),
                "#F28E2B".to_string(),
                "#E15759".to_string(),
                "#76B7B2".to_string(),
                "#59A14F".to_string(),
                "#EDC948".to_string(),
                "#B07AA1".to_string(),
                "#FF9DA7".to_string(),
                "#9C755F".to_string(),
                "#BAB0AC".to_string(),
            ],
        }
    }

    /// Generate visualization data
    pub async fn generate(&self, viz_type: VisualizationType, entries: &[HistoryEntry]) -> Result<VisualizationData> {
        let data = match viz_type {
            VisualizationType::Timeline => self.generate_timeline(entries),
            VisualizationType::DailyActivity => self.generate_daily_activity(entries),
            VisualizationType::WeeklyHeatmap => self.generate_weekly_heatmap(entries),
            VisualizationType::MonthlyHeatmap => self.generate_monthly_heatmap(entries),
            VisualizationType::CategoryPie => self.generate_category_pie(entries),
            VisualizationType::DomainBar => self.generate_domain_bar(entries),
            VisualizationType::HourlyDistribution => self.generate_hourly_distribution(entries),
            VisualizationType::VisitFrequency => self.generate_visit_frequency(entries),
        };

        Ok(data)
    }

    /// Generate timeline data
    fn generate_timeline(&self, entries: &[HistoryEntry]) -> VisualizationData {
        let mut points: Vec<TimelinePoint> = entries
            .iter()
            .map(|e| TimelinePoint {
                timestamp: e.timestamp,
                url: e.url.clone(),
                title: e.title.clone(),
                visits: e.visit_count,
                domain: extract_domain(&e.url),
                category: e.category.clone(),
            })
            .collect();

        points.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        let date_range = if !points.is_empty() {
            let start = points.last().unwrap().timestamp;
            let end = points.first().unwrap().timestamp;
            Some(DateRange { start, end })
        } else {
            None
        };

        VisualizationData {
            viz_type: VisualizationType::Timeline,
            title: "Browsing Timeline".to_string(),
            data: DataPoints::Timeline(points),
            metadata: VisualizationMetadata {
                total_points: entries.len(),
                date_range,
                unit: "visits".to_string(),
            },
        }
    }

    /// Generate daily activity chart
    fn generate_daily_activity(&self, entries: &[HistoryEntry]) -> VisualizationData {
        let mut daily_counts: HashMap<String, usize> = HashMap::new();

        for entry in entries {
            let date_key = entry.timestamp.format("%Y-%m-%d").to_string();
            *daily_counts.entry(date_key).or_insert(0) += 1;
        }

        let mut points: Vec<ChartPoint> = daily_counts
            .iter()
            .map(|(date, count)| ChartPoint {
                label: date.clone(),
                value: *count as f64,
                secondary_value: None,
                color: None,
            })
            .collect();

        points.sort_by(|a, b| a.label.cmp(&b.label));

        VisualizationData {
            viz_type: VisualizationType::DailyActivity,
            title: "Daily Activity".to_string(),
            data: DataPoints::Chart(points),
            metadata: VisualizationMetadata {
                total_points: entries.len(),
                date_range: None,
                unit: "visits".to_string(),
            },
        }
    }

    /// Generate weekly heatmap
    fn generate_weekly_heatmap(&self, entries: &[HistoryEntry]) -> VisualizationData {
        // 24 hours x 7 days
        let mut heatmap: HashMap<(String, String), usize> = HashMap::new();

        for entry in entries {
            let day = weekday_name(entry.timestamp.weekday());
            let hour = format!("{:02}:00", entry.timestamp.hour());
            *heatmap.entry((day.clone(), hour.clone())).or_insert(0) += 1;
        }

        let max_value = heatmap.values().copied().max().unwrap_or(1);

        let cells: Vec<HeatmapCell> = (0..7)
            .flat_map(|day_idx| {
                let day = weekday_name(Weekday::try_from(day_idx).unwrap());
                (0..24).map(move |hour| {
                    let hour_str = format!("{:02}:00", hour);
                    let value = heatmap.get(&(day.clone(), hour_str.clone())).copied().unwrap_or(0);
                    HeatmapCell {
                        row: day.clone(),
                        column: hour_str,
                        value: value as f64,
                        normalized: value as f64 / max_value as f64,
                    }
                })
            })
            .collect();

        VisualizationData {
            viz_type: VisualizationType::WeeklyHeatmap,
            title: "Weekly Activity Heatmap".to_string(),
            data: DataPoints::Heatmap(cells),
            metadata: VisualizationMetadata {
                total_points: entries.len(),
                date_range: None,
                unit: "visits".to_string(),
            },
        }
    }

    /// Generate monthly heatmap
    fn generate_monthly_heatmap(&self, entries: &[HistoryEntry]) -> VisualizationData {
        let mut heatmap: HashMap<(String, String), usize> = HashMap::new();

        for entry in entries {
            let month = entry.timestamp.format("%B").to_string();
            let day = entry.timestamp.format("%d").to_string();
            *heatmap.entry((month.clone(), day.clone())).or_insert(0) += 1;
        }

        let max_value = heatmap.values().copied().max().unwrap_or(1);

        let cells: Vec<HeatmapCell> = heatmap
            .iter()
            .map(|((month, day), value)| HeatmapCell {
                row: month.clone(),
                column: day.clone(),
                value: *value as f64,
                normalized: *value as f64 / max_value as f64,
            })
            .collect();

        VisualizationData {
            viz_type: VisualizationType::MonthlyHeatmap,
            title: "Monthly Activity Heatmap".to_string(),
            data: DataPoints::Heatmap(cells),
            metadata: VisualizationMetadata {
                total_points: entries.len(),
                date_range: None,
                unit: "visits".to_string(),
            },
        }
    }

    /// Generate category pie chart
    fn generate_category_pie(&self, entries: &[HistoryEntry]) -> VisualizationData {
        let mut categories: HashMap<String, usize> = HashMap::new();
        let total = entries.len();

        for entry in entries {
            let category = entry.category.clone().unwrap_or_else(|| "Other".to_string());
            *categories.entry(category).or_insert(0) += 1;
        }

        let slices: Vec<PieSlice> = categories
            .iter()
            .enumerate()
            .map(|(i, (category, count))| PieSlice {
                label: category.clone(),
                value: *count as f64,
                percentage: if total > 0 { *count as f64 / total as f64 * 100.0 } else { 0.0 },
                color: self.palette[i % self.palette.len()].clone(),
            })
            .collect();

        VisualizationData {
            viz_type: VisualizationType::CategoryPie,
            title: "Category Distribution".to_string(),
            data: DataPoints::Pie(slices),
            metadata: VisualizationMetadata {
                total_points: entries.len(),
                date_range: None,
                unit: "%".to_string(),
            },
        }
    }

    /// Generate domain bar chart
    fn generate_domain_bar(&self, entries: &[HistoryEntry]) -> VisualizationData {
        let mut domains: HashMap<String, usize> = HashMap::new();

        for entry in entries {
            let domain = extract_domain(&entry.url);
            *domains.entry(domain).or_insert(0) += 1;
        }

        let mut bars: Vec<BarData> = domains
            .iter()
            .map(|(domain, count)| BarData {
                label: domain.clone(),
                values: vec![*count as f64],
                total: *count as f64,
            })
            .collect();

        bars.sort_by(|a, b| b.total.partial_cmp(&a.total).unwrap());
        bars.truncate(20); // Top 20 domains

        VisualizationData {
            viz_type: VisualizationType::DomainBar,
            title: "Most Visited Domains".to_string(),
            data: DataPoints::Bar(bars),
            metadata: VisualizationMetadata {
                total_points: entries.len(),
                date_range: None,
                unit: "visits".to_string(),
            },
        }
    }

    /// Generate hourly distribution
    fn generate_hourly_distribution(&self, entries: &[HistoryEntry]) -> VisualizationData {
        let mut hourly: HashMap<u32, usize> = HashMap::new();

        for entry in entries {
            let hour = entry.timestamp.hour();
            *hourly.entry(hour).or_insert(0) += 1;
        }

        let points: Vec<ChartPoint> = (0..24)
            .map(|hour| ChartPoint {
                label: format!("{:02}:00", hour),
                value: hourly.get(&hour).copied().unwrap_or(0) as f64,
                secondary_value: None,
                color: None,
            })
            .collect();

        VisualizationData {
            viz_type: VisualizationType::HourlyDistribution,
            title: "Hourly Browsing Distribution".to_string(),
            data: DataPoints::Chart(points),
            metadata: VisualizationMetadata {
                total_points: entries.len(),
                date_range: None,
                unit: "visits".to_string(),
            },
        }
    }

    /// Generate visit frequency graph
    fn generate_visit_frequency(&self, entries: &[HistoryEntry]) -> VisualizationData {
        let mut visit_counts: HashMap<u32, usize> = HashMap::new();

        for entry in entries {
            *visit_counts.entry(entry.visit_count).or_insert(0) += 1;
        }

        let max_visits = visit_counts.keys().copied().max().unwrap_or(1);

        let points: Vec<ChartPoint> = (1..=max_visits)
            .map(|visits| ChartPoint {
                label: format!("{} visits", visits),
                value: visit_counts.get(&visits).copied().unwrap_or(0) as f64,
                secondary_value: None,
                color: None,
            })
            .collect();

        VisualizationData {
            viz_type: VisualizationType::VisitFrequency,
            title: "Visit Frequency Distribution".to_string(),
            data: DataPoints::Chart(points),
            metadata: VisualizationMetadata {
                total_points: entries.len(),
                date_range: None,
                unit: "pages".to_string(),
            },
        }
    }
}

impl Default for HistoryVisualization {
    fn default() -> Self {
        Self::new()
    }
}

/// Get weekday name
fn weekday_name(weekday: Weekday) -> String {
    match weekday {
        Weekday::Mon => "Monday".to_string(),
        Weekday::Tue => "Tuesday".to_string(),
        Weekday::Wed => "Wednesday".to_string(),
        Weekday::Thu => "Thursday".to_string(),
        Weekday::Fri => "Friday".to_string(),
        Weekday::Sat => "Saturday".to_string(),
        Weekday::Sun => "Sunday".to_string(),
    }
}

/// Extract domain from URL
fn extract_domain(url: &str) -> String {
    let url = url.trim_start_matches("https://")
                .trim_start_matches("http://")
                .trim_start_matches("www.");
    url.split('/').next().unwrap_or(url).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_timeline() {
        let viz = HistoryVisualization::new();
        
        let entries = vec![
            HistoryEntry::new("https://example.com".to_string(), "Example".to_string(), false),
        ];

        let data = viz.generate(VisualizationType::Timeline, &entries).await.unwrap();
        
        assert_eq!(data.viz_type, VisualizationType::Timeline);
    }

    #[tokio::test]
    async fn test_generate_category_pie() {
        let viz = HistoryVisualization::new();
        
        let entries = vec![
            HistoryEntry::new("https://example.com".to_string(), "Example".to_string(), false),
        ];

        let data = viz.generate(VisualizationType::CategoryPie, &entries).await.unwrap();
        
        assert_eq!(data.viz_type, VisualizationType::CategoryPie);
    }

    #[tokio::test]
    async fn test_generate_weekly_heatmap() {
        let viz = HistoryVisualization::new();
        
        let entries = vec![
            HistoryEntry::new("https://example.com".to_string(), "Example".to_string(), false),
        ];

        let data = viz.generate(VisualizationType::WeeklyHeatmap, &entries).await.unwrap();
        
        // Should have 24 * 7 = 168 cells
        if let DataPoints::Heatmap(cells) = data.data {
            assert_eq!(cells.len(), 168);
        }
    }
}