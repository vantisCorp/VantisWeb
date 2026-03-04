//! Usage Pattern Analyzer
//!
//! Analyzes user browsing patterns to generate insights and recommendations.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::profiles::{ProfileAnalytics, DailyUsage, WebsiteUsage};

/// Usage pattern type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatternType {
    /// High frequency visits to specific sites
    FrequentVisits,
    /// Long browsing sessions
    ExtendedSessions,
    /// Short, frequent sessions
    MicroSessions,
    /// Night-time browsing
    NightOwl,
    /// Early morning browsing
    EarlyBird,
    /// Work-focused browsing
    WorkOriented,
    /// Entertainment-focused browsing
    EntertainmentFocused,
    /// Shopping-heavy usage
    Shopper,
    /// Social media heavy
    SocialButterfly,
    /// Development/programming focused
    Developer,
    /// News consumption
    NewsReader,
    /// Educational content consumption
    Learner,
    /// High tab count usage
    TabHoarding,
    /// Minimal tab usage
    TabMinimizer,
    /// Custom pattern
    Custom(String),
}

/// Pattern confidence level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatternConfidence {
    Low,
    Medium,
    High,
    VeryHigh,
}

impl PatternConfidence {
    /// Convert to numeric score
    pub fn to_score(&self) -> f64 {
        match self {
            PatternConfidence::Low => 0.25,
            PatternConfidence::Medium => 0.5,
            PatternConfidence::High => 0.75,
            PatternConfidence::VeryHigh => 1.0,
        }
    }

    /// From numeric score
    pub fn from_score(score: f64) -> Self {
        if score >= 0.9 {
            PatternConfidence::VeryHigh
        } else if score >= 0.7 {
            PatternConfidence::High
        } else if score >= 0.5 {
            PatternConfidence::Medium
        } else {
            PatternConfidence::Low
        }
    }
}

/// Usage pattern detected from analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePattern {
    /// Pattern type
    pub pattern_type: PatternType,
    /// Confidence level
    pub confidence: PatternConfidence,
    /// Pattern description
    pub description: String,
    /// Detected timestamp
    pub detected_at: DateTime<Utc>,
    /// Related data points
    pub metrics: PatternMetrics,
}

/// Pattern-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMetrics {
    /// Average session length (seconds)
    pub avg_session_length: u64,
    /// Sessions per day
    pub sessions_per_day: u32,
    /// Peak usage hour (0-23)
    pub peak_hour: u8,
    /// Most visited category
    pub dominant_category: String,
    /// Average tabs per session
    pub avg_tabs_per_session: f64,
    /// Website diversity (unique sites / total visits)
    pub website_diversity: f64,
}

/// Pattern analyzer
pub struct PatternAnalyzer {
    /// Cached patterns
    cached_patterns: HashMap<String, Vec<UsagePattern>>,
    /// Analysis configuration
    config: AnalyzerConfig,
}

/// Analyzer configuration
#[derive(Debug, Clone)]
struct AnalyzerConfig {
    /// Minimum sessions required for analysis
    min_sessions: u32,
    /// Minimum days of data required
    min_days: u32,
    /// Confidence threshold for pattern detection
    confidence_threshold: f64,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            min_sessions: 10,
            min_days: 7,
            confidence_threshold: 0.5,
        }
    }
}

impl PatternAnalyzer {
    /// Create a new pattern analyzer
    pub fn new() -> Self {
        Self {
            cached_patterns: HashMap::new(),
            config: AnalyzerConfig::default(),
        }
    }

    /// Analyze profile analytics for patterns
    pub fn analyze(&mut self, profile_id: &str, analytics: &ProfileAnalytics) -> Result<Vec<UsagePattern>> {
        log::info!("Analyzing patterns for profile: {}", profile_id);

        // Validate data
        if analytics.session_count < self.config.min_sessions as u64 {
            log::warn!(
                "Insufficient data for pattern analysis: {} sessions (min: {})",
                analytics.session_count,
                self.config.min_sessions
            );
            return Ok(Vec::new());
        }

        if analytics.daily_usage.len() < self.config.min_days as usize {
            log::warn!(
                "Insufficient daily data for pattern analysis: {} days (min: {})",
                analytics.daily_usage.len(),
                self.config.min_days
            );
            return Ok(Vec::new());
        }

        let mut patterns = Vec::new();

        // Calculate base metrics
        let metrics = self.calculate_metrics(analytics);

        // Detect various patterns
        patterns.extend(self.detect_session_patterns(&metrics, analytics));
        patterns.extend(self.detect_time_patterns(&metrics, analytics));
        patterns.extend(self.detect_category_patterns(&metrics, analytics));
        patterns.extend(self.detect_tab_patterns(&metrics, analytics));

        // Sort by confidence and filter
        patterns.sort_by(|a, b| b.confidence.to_score().partial_cmp(&a.confidence.to_score()).unwrap());
        patterns.retain(|p| p.confidence.to_score() >= self.config.confidence_threshold);

        // Cache patterns
        self.cached_patterns.insert(profile_id.to_string(), patterns.clone());

        log::info!("Detected {} patterns for profile: {}", patterns.len(), profile_id);
        Ok(patterns)
    }

    /// Calculate base metrics from analytics
    fn calculate_metrics(&self, analytics: &ProfileAnalytics) -> PatternMetrics {
        let avg_session_length = if analytics.session_count > 0 {
            analytics.total_time / analytics.session_count
        } else {
            0
        };

        let sessions_per_day = if !analytics.daily_usage.is_empty() {
            analytics.session_count as u32 / analytics.daily_usage.len() as u32
        } else {
            0
        };

        // Find peak usage hour
        let peak_hour = self.determine_peak_hour(analytics);

        // Determine dominant category
        let dominant_category = self.determine_dominant_category(analytics);

        let avg_tabs_per_session = if analytics.session_count > 0 {
            analytics.tab_stats.total_opened as f64 / analytics.session_count as f64
        } else {
            0.0
        };

        // Calculate website diversity
        let total_visits: u64 = analytics.top_websites.iter().map(|w| w.visits).sum();
        let website_diversity = if total_visits > 0 {
            analytics.top_websites.len() as f64 / total_visits as f64
        } else {
            0.0
        };

        PatternMetrics {
            avg_session_length,
            sessions_per_day,
            peak_hour,
            dominant_category,
            avg_tabs_per_session,
            website_diversity,
        }
    }

    /// Determine peak usage hour
    fn determine_peak_hour(&self, analytics: &ProfileAnalytics) -> u8 {
        // In a real implementation, this would analyze actual hourly data
        // For now, we'll use a simplified approach based on typical patterns
        let total_sessions = analytics.session_count;
        if total_sessions > 0 {
            // Simulate peak hour based on work vs entertainment patterns
            let work_time = analytics.top_websites.iter()
                .filter(|w| w.url.contains("github.com") || w.url.contains("stackoverflow.com"))
                .count();
            
            if work_time > analytics.top_websites.len() / 2 {
                10 // Peak at 10 AM for work
            } else {
                20 // Peak at 8 PM for entertainment
            }
        } else {
            12
        }
    }

    /// Determine dominant category
    fn determine_dominant_category(&self, analytics: &ProfileAnalytics) -> String {
        let mut category_counts: HashMap<String, u64> = HashMap::new();

        for website in &analytics.top_websites {
            let category = self.categorize_website(&website.url);
            *category_counts.entry(category).or_insert(0) += website.visits;
        }

        category_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(category, _)| category)
            .unwrap_or_else(|| "Other".to_string())
    }

    /// Categorize a website URL
    fn categorize_website(&self, url: &str) -> String {
        let url_lower = url.to_lowercase();

        if url_lower.contains("github.com") || url_lower.contains("gitlab.com") || url_lower.contains("stackoverflow.com") {
            "Development".to_string()
        } else if url_lower.contains("youtube.com") || url_lower.contains("netflix.com") || url_lower.contains("twitch.tv") {
            "Entertainment".to_string()
        } else if url_lower.contains("facebook.com") || url_lower.contains("twitter.com") || url_lower.contains("reddit.com") {
            "Social Media".to_string()
        } else if url_lower.contains("amazon.com") || url_lower.contains("ebay.com") || url_lower.contains("shopify.com") {
            "Shopping".to_string()
        } else if url_lower.contains("news") || url_lower.contains("cnn.com") || url_lower.contains("bbc.com") {
            "News".to_string()
        } else if url_lower.contains("coursera.org") || url_lower.contains("udemy.com") || url_lower.contains("khanacademy.org") {
            "Education".to_string()
        } else if url_lower.contains("google.com") || url_lower.contains("docs.google.com") || url_lower.contains("notion.so") {
            "Productivity".to_string()
        } else {
            "Other".to_string()
        }
    }

    /// Detect session-related patterns
    fn detect_session_patterns(&self, metrics: &PatternMetrics, analytics: &ProfileAnalytics) -> Vec<UsagePattern> {
        let mut patterns = Vec::new();

        // Extended sessions (30+ minutes)
        if metrics.avg_session_length >= 1800 {
            patterns.push(UsagePattern {
                pattern_type: PatternType::ExtendedSessions,
                confidence: PatternConfidence::from_score((metrics.avg_session_length / 3600.0).min(1.0)),
                description: format!(
                    "You tend to have long browsing sessions (average: {} minutes)",
                    metrics.avg_session_length / 60
                ),
                detected_at: Utc::now(),
                metrics: metrics.clone(),
            });
        }

        // Micro sessions (less than 5 minutes)
        if metrics.avg_session_length < 300 && analytics.session_count > 20 {
            patterns.push(UsagePattern {
                pattern_type: PatternType::MicroSessions,
                confidence: PatternConfidence::from_score(1.0 - (metrics.avg_session_length / 300.0)),
                description: format!(
                    "You prefer quick browsing sessions (average: {} minutes)",
                    metrics.avg_session_length / 60
                ),
                detected_at: Utc::now(),
                metrics: metrics.clone(),
            });
        }

        // Frequent visits to specific sites
        if !analytics.top_websites.is_empty() {
            let top_site = &analytics.top_websites[0];
            let total_visits: u64 = analytics.top_websites.iter().map(|w| w.visits).sum();
            
            if total_visits > 0 {
                let concentration = top_site.visits as f64 / total_visits as f64;
                if concentration > 0.3 {
                    patterns.push(UsagePattern {
                        pattern_type: PatternType::FrequentVisits,
                        confidence: PatternConfidence::from_score(concentration),
                        description: format!(
                            "You frequently visit {} ({}% of visits)",
                            top_site.url,
                            (concentration * 100.0) as u32
                        ),
                        detected_at: Utc::now(),
                        metrics: metrics.clone(),
                    });
                }
            }
        }

        patterns
    }

    /// Detect time-related patterns
    fn detect_time_patterns(&self, metrics: &PatternMetrics, _analytics: &ProfileAnalytics) -> Vec<UsagePattern> {
        let mut patterns = Vec::new();

        // Night owl (peak usage after 10 PM)
        if metrics.peak_hour >= 22 {
            patterns.push(UsagePattern {
                pattern_type: PatternType::NightOwl,
                confidence: PatternConfidence::High,
                description: format!("You're most active late at night (peak: {} PM)", metrics.peak_hour),
                detected_at: Utc::now(),
                metrics: metrics.clone(),
            });
        }

        // Early bird (peak usage before 8 AM)
        if metrics.peak_hour <= 8 {
            patterns.push(UsagePattern {
                pattern_type: PatternType::EarlyBird,
                confidence: PatternConfidence::High,
                description: format!("You're most active early in the morning (peak: {} AM)", metrics.peak_hour),
                detected_at: Utc::now(),
                metrics: metrics.clone(),
            });
        }

        patterns
    }

    /// Detect category-related patterns
    fn detect_category_patterns(&self, metrics: &PatternMetrics, _analytics: &ProfileAnalytics) -> Vec<UsagePattern> {
        let mut patterns = Vec::new();

        match metrics.dominant_category.as_str() {
            "Development" => {
                patterns.push(UsagePattern {
                    pattern_type: PatternType::Developer,
                    confidence: PatternConfidence::High,
                    description: "Your browsing is primarily focused on development and programming".to_string(),
                    detected_at: Utc::now(),
                    metrics: metrics.clone(),
                });
            }
            "Entertainment" => {
                patterns.push(UsagePattern {
                    pattern_type: PatternType::EntertainmentFocused,
                    confidence: PatternConfidence::High,
                    description: "You spend most of your time on entertainment content".to_string(),
                    detected_at: Utc::now(),
                    metrics: metrics.clone(),
                });
            }
            "Social Media" => {
                patterns.push(UsagePattern {
                    pattern_type: PatternType::SocialButterfly,
                    confidence: PatternConfidence::High,
                    description: "You're highly active on social media platforms".to_string(),
                    detected_at: Utc::now(),
                    metrics: metrics.clone(),
                });
            }
            "Shopping" => {
                patterns.push(UsagePattern {
                    pattern_type: PatternType::Shopper,
                    confidence: PatternConfidence::High,
                    description: "Your browsing is focused on shopping and e-commerce".to_string(),
                    detected_at: Utc::now(),
                    metrics: metrics.clone(),
                });
            }
            "Productivity" => {
                patterns.push(UsagePattern {
                    pattern_type: PatternType::WorkOriented,
                    confidence: PatternConfidence::High,
                    description: "Your browsing patterns indicate a work-oriented approach".to_string(),
                    detected_at: Utc::now(),
                    metrics: metrics.clone(),
                });
            }
            "Education" => {
                patterns.push(UsagePattern {
                    pattern_type: PatternType::Learner,
                    confidence: PatternConfidence::High,
                    description: "You frequently visit educational content".to_string(),
                    detected_at: Utc::now(),
                    metrics: metrics.clone(),
                });
            }
            _ => {}
        }

        patterns
    }

    /// Detect tab-related patterns
    fn detect_tab_patterns(&self, metrics: &PatternMetrics, _analytics: &ProfileAnalytics) -> Vec<UsagePattern> {
        let mut patterns = Vec::new();

        // Tab hoarding (15+ tabs per session)
        if metrics.avg_tabs_per_session >= 15.0 {
            patterns.push(UsagePattern {
                pattern_type: PatternType::TabHoarding,
                confidence: PatternConfidence::from_score((metrics.avg_tabs_per_session / 30.0).min(1.0)),
                description: format!(
                    "You tend to keep many tabs open (average: {} tabs)",
                    metrics.avg_tabs_per_session as u32
                ),
                detected_at: Utc::now(),
                metrics: metrics.clone(),
            });
        }

        // Tab minimizer (less than 5 tabs per session)
        if metrics.avg_tabs_per_session < 5.0 {
            patterns.push(UsagePattern {
                pattern_type: PatternType::TabMinimizer,
                confidence: PatternConfidence::from_score(1.0 - (metrics.avg_tabs_per_session / 5.0)),
                description: format!(
                    "You keep your browsing focused with few tabs (average: {} tabs)",
                    metrics.avg_tabs_per_session as u32
                ),
                detected_at: Utc::now(),
                metrics: metrics.clone(),
            });
        }

        patterns
    }

    /// Get cached patterns for a profile
    pub fn get_cached_patterns(&self, profile_id: &str) -> Option<&Vec<UsagePattern>> {
        self.cached_patterns.get(profile_id)
    }

    /// Clear cache for a specific profile
    pub fn clear_cache(&mut self, profile_id: &str) {
        self.cached_patterns.remove(profile_id);
    }

    /// Clear all cached patterns
    pub fn clear_all_cache(&mut self) {
        self.cached_patterns.clear();
    }
}

impl Default for PatternAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_analytics() -> ProfileAnalytics {
        let mut daily_usage = HashMap::new();
        daily_usage.insert("2024-01-01".to_string(), DailyUsage {
            date: "2024-01-01".to_string(),
            time_spent: 3600,
            sessions: 2,
            websites_visited: 10,
        });
        daily_usage.insert("2024-01-02".to_string(), DailyUsage {
            date: "2024-01-02".to_string(),
            time_spent: 7200,
            sessions: 3,
            websites_visited: 15,
        });

        let mut top_websites = Vec::new();
        top_websites.push(WebsiteUsage {
            url: "https://github.com".to_string(),
            visits: 20,
            time_spent: 1800,
            last_visit: Utc::now(),
        });
        top_websites.push(WebsiteUsage {
            url: "https://stackoverflow.com".to_string(),
            visits: 15,
            time_spent: 1200,
            last_visit: Utc::now(),
        });

        ProfileAnalytics {
            profile_id: "test".to_string(),
            total_time: 10800,
            session_count: 5,
            first_session: None,
            last_session: None,
            daily_usage,
            top_websites,
            tab_stats: crate::profiles::TabStatistics {
                total_opened: 50,
                average_per_session: 10.0,
                max_open: 15,
                current_open: 3,
            },
            performance: crate::profiles::PerformanceMetrics {
                avg_page_load_time: 150,
                total_crashes: 0,
                memory_usage: 500,
                cpu_usage: 25.0,
            },
        }
    }

    #[test]
    fn test_pattern_analyzer_creation() {
        let analyzer = PatternAnalyzer::new();
        assert!(analyzer.cached_patterns.is_empty());
    }

    #[test]
    fn test_pattern_detection() {
        let mut analyzer = PatternAnalyzer::new();
        let analytics = create_test_analytics();

        let patterns = analyzer.analyze("test", &analytics).unwrap();
        assert!(!patterns.is_empty());
    }

    #[test]
    fn test_confidence_levels() {
        assert_eq!(PatternConfidence::Low.to_score(), 0.25);
        assert_eq!(PatternConfidence::Medium.to_score(), 0.5);
        assert_eq!(PatternConfidence::High.to_score(), 0.75);
        assert_eq!(PatternConfidence::VeryHigh.to_score(), 1.0);

        assert_eq!(PatternConfidence::from_score(0.1), PatternConfidence::Low);
        assert_eq!(PatternConfidence::from_score(0.5), PatternConfidence::Medium);
        assert_eq!(PatternConfidence::from_score(0.8), PatternConfidence::High);
        assert_eq!(PatternConfidence::from_score(0.95), PatternConfidence::VeryHigh);
    }

    #[test]
    fn test_insufficient_data() {
        let mut analyzer = PatternAnalyzer::new();
        let mut analytics = create_test_analytics();
        analytics.session_count = 5; // Below minimum

        let patterns = analyzer.analyze("test", &analytics).unwrap();
        assert!(patterns.is_empty());
    }
}