//! Metrics collection and aggregation for analytics
//! 
//! This module provides structures and methods for collecting,
//! aggregating, and querying analytics metrics.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};

/// Usage metrics for browsing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageMetrics {
    /// Total visits tracked
    pub total_visits: u64,
    /// Total time spent browsing (ms)
    pub total_time_ms: u64,
    /// Average time per page (ms)
    pub average_page_time_ms: u64,
    /// Unique domains visited
    pub unique_domains: u64,
    /// Visits per domain
    pub domain_visits: HashMap<String, DomainStats>,
    /// Daily statistics
    daily_stats: HashMap<String, DailyStats>,
    /// Category statistics (News, Social, Work, etc.)
    category_stats: HashMap<String, CategoryStats>,
    /// Visit timestamps for trend analysis
    visit_history: Vec<VisitRecord>,
    /// Peak usage hours (hour -> count)
    hourly_distribution: [u64; 24],
    /// Peak usage days (day of week -> count)
    daily_distribution: [u64; 7],
    /// Top domains (cached for performance)
    pub top_domains: Vec<(String, DomainVisitStats)>,
}

/// Statistics for a specific domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainStats {
    pub domain: String,
    pub visits: u64,
    pub total_time_ms: u64,
    pub avg_time_ms: f64,
    pub first_visit: DateTime<Utc>,
    pub last_visit: DateTime<Utc>,
    pub category: Option<String>,
}

/// Daily statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStats {
    pub date: String,
    pub visits: u64,
    pub time_ms: u64,
    pub unique_domains: u64,
    pub pages_per_session: f64,
}

/// Category statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStats {
    pub category: String,
    pub visits: u64,
    pub time_ms: u64,
    pub domains: Vec<String>,
}

/// Individual visit record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisitRecord {
    pub timestamp: DateTime<Utc>,
    pub domain: String,
    pub duration_ms: u64,
    pub category: Option<String>,
}

impl UsageMetrics {
    /// Create new usage metrics
    pub fn new() -> Self {
        Self {
            total_visits: 0,
            total_time_ms: 0,
            average_page_time_ms: 0,
            unique_domains: 0,
            domain_visits: HashMap::new(),
            daily_stats: HashMap::new(),
            category_stats: HashMap::new(),
            visit_history: Vec::new(),
            hourly_distribution: [0; 24],
            daily_distribution: [0; 7],
            top_domains: Vec::new(),
        }
    }

    /// Record a visit to a URL
    pub fn record_visit(&mut self, url: String, duration_ms: u64) {
        let domain = Self::extract_domain(&url);
        let now = Utc::now();
        let date_key = now.format("%Y-%m-%d").to_string();
        let hour = now.hour() as usize;
        let day = now.weekday().num_days_from_monday() as usize;

        // Update totals
        self.total_visits += 1;
        self.total_time_ms += duration_ms;
        self.average_page_time_ms = if self.total_visits > 0 {
            self.total_time_ms / self.total_visits
        } else {
            0
        };
        self.hourly_distribution[hour] += 1;
        self.daily_distribution[day] += 1;

        // Check if this is a new domain
        let is_new_domain = !self.domain_visits.contains_key(&domain);
        if is_new_domain {
            self.unique_domains += 1;
        }

        // Update domain stats
        let domain_entry = self.domain_visits.entry(domain.clone()).or_insert_with(|| {
            DomainStats {
                domain: domain.clone(),
                visits: 0,
                total_time_ms: 0,
                avg_time_ms: 0.0,
                first_visit: now,
                last_visit: now,
                category: None,
            }
        });
        domain_entry.visits += 1;
        domain_entry.total_time_ms += duration_ms;
        domain_entry.avg_time_ms = domain_entry.total_time_ms as f64 / domain_entry.visits as f64;
        domain_entry.last_visit = now;

        // Update daily stats
        let daily_entry = self.daily_stats.entry(date_key).or_insert_with(|| {
            DailyStats {
                date: now.format("%Y-%m-%d").to_string(),
                visits: 0,
                time_ms: 0,
                unique_domains: 0,
                pages_per_session: 0.0,
            }
        });
        daily_entry.visits += 1;
        daily_entry.time_ms += duration_ms;

        // Record visit in history
        self.visit_history.push(VisitRecord {
            timestamp: now,
            domain,
            duration_ms,
            category: None,
        });

        // Limit history size
        if self.visit_history.len() > 10000 {
            self.visit_history.drain(0..1000);
        }
    }

    /// Extract domain from URL
    fn extract_domain(url: &str) -> String {
        url::Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.to_string()))
            .unwrap_or_else(|| url.to_string())
    }

    /// Get top N visited domains
    pub fn top_domains(&self, n: usize) -> Vec<&DomainStats> {
        let mut domains: Vec<_> = self.domain_visits.values().collect();
        domains.sort_by(|a, b| b.visits.cmp(&a.visits));
        domains.into_iter().take(n).collect()
    }

    /// Get browsing time for a period
    pub fn time_for_period(&self, days: u32) -> u64 {
        let cutoff = Utc::now() - Duration::days(days as i64);
        self.visit_history
            .iter()
            .filter(|v| v.timestamp > cutoff)
            .map(|v| v.duration_ms)
            .sum()
    }

    /// Get visits for a period
    pub fn visits_for_period(&self, days: u32) -> u64 {
        let cutoff = Utc::now() - Duration::days(days as i64);
        self.visit_history
            .iter()
            .filter(|v| v.timestamp > cutoff)
            .count() as u64
    }

    /// Get average session length
    pub fn avg_session_length(&self) -> f64 {
        if self.total_visits == 0 {
            0.0
        } else {
            self.total_time_ms as f64 / self.total_visits as f64
        }
    }

    /// Clear all metrics
    pub fn clear(&mut self) {
        *self = Self::new();
    }
}

impl Default for UsageMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Page load records
    page_loads: Vec<PageLoadRecord>,
    /// Memory usage samples
    memory_samples: Vec<MemorySample>,
    /// CPU usage samples
    cpu_samples: Vec<CpuSample>,
    /// Network statistics
    network_stats: NetworkStats,
    /// Aggregated metrics
    aggregated: AggregatedPerformance,
    // Public fields for dashboard access
    /// Average page load time in ms
    pub average_load_time_ms: u64,
    /// Median page load time in ms
    pub median_load_time_ms: u64,
    /// 95th percentile load time in ms
    pub p95_load_time_ms: u64,
    /// Number of slow pages (>3s)
    pub slow_pages: u64,
    /// Average memory usage in MB
    pub average_memory_mb: u64,
    /// Peak memory usage in MB
    pub peak_memory_mb: u64,
    /// Average CPU usage percent
    pub average_cpu_percent: u64,
}

/// Individual page load record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageLoadRecord {
    pub url: String,
    pub timestamp: DateTime<Utc>,
    pub dns_time_ms: u64,
    pub connect_time_ms: u64,
    pub ttfb_ms: u64,
    pub dom_content_loaded_ms: u64,
    pub load_complete_ms: u64,
    pub total_bytes: u64,
    pub request_count: u32,
}

/// Memory usage sample
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySample {
    pub timestamp: DateTime<Utc>,
    pub used_mb: f64,
    pub available_mb: f64,
}

/// CPU usage sample
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuSample {
    pub timestamp: DateTime<Utc>,
    pub percent: f64,
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub request_count: u64,
    pub error_count: u64,
}

/// Aggregated performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedPerformance {
    pub avg_page_load_ms: f64,
    pub median_page_load_ms: f64,
    pub p95_page_load_ms: f64,
    pub avg_memory_mb: f64,
    pub avg_cpu_percent: f64,
}

impl PerformanceMetrics {
    /// Create new performance metrics
    pub fn new() -> Self {
        Self {
            page_loads: Vec::new(),
            memory_samples: Vec::new(),
            cpu_samples: Vec::new(),
            network_stats: NetworkStats {
                total_bytes_sent: 0,
                total_bytes_received: 0,
                request_count: 0,
                error_count: 0,
            },
            aggregated: AggregatedPerformance {
                avg_page_load_ms: 0.0,
                median_page_load_ms: 0.0,
                p95_page_load_ms: 0.0,
                avg_memory_mb: 0.0,
                avg_cpu_percent: 0.0,
            },
            average_load_time_ms: 0,
            median_load_time_ms: 0,
            p95_load_time_ms: 0,
            slow_pages: 0,
            average_memory_mb: 0,
            peak_memory_mb: 0,
            average_cpu_percent: 0,
        }
    }

    /// Record a page load
    pub fn record_page_load(&mut self, record: PageLoadRecord) {
        self.page_loads.push(record);
        self.recalculate_aggregates();
        
        // Limit stored records
        if self.page_loads.len() > 1000 {
            self.page_loads.drain(0..100);
        }
    }

    /// Record memory usage
    pub fn record_memory(&mut self, used_mb: f64, available_mb: f64) {
        self.memory_samples.push(MemorySample {
            timestamp: Utc::now(),
            used_mb,
            available_mb,
        });
        
        if self.memory_samples.len() > 1000 {
            self.memory_samples.drain(0..100);
        }
        
        self.recalculate_aggregates();
    }

    /// Record CPU usage
    pub fn record_cpu(&mut self, percent: f64) {
        self.cpu_samples.push(CpuSample {
            timestamp: Utc::now(),
            percent,
        });
        
        if self.cpu_samples.len() > 1000 {
            self.cpu_samples.drain(0..100);
        }
        
        self.recalculate_aggregates();
    }

    /// Recalculate aggregated metrics
    fn recalculate_aggregates(&mut self) {
        if self.page_loads.is_empty() {
            return;
        }

        let mut load_times: Vec<u64> = self.page_loads.iter()
            .map(|p| p.load_complete_ms)
            .collect();
        load_times.sort();

        let count = load_times.len();
        let sum: u64 = load_times.iter().sum();
        
        self.aggregated.avg_page_load_ms = sum as f64 / count as f64;
        self.aggregated.median_page_load_ms = load_times[count / 2] as f64;
        self.aggregated.p95_page_load_ms = load_times[(count as f64 * 0.95) as usize] as f64;
        
        // Update public fields
        self.average_load_time_ms = self.aggregated.avg_page_load_ms as u64;
        self.median_load_time_ms = self.aggregated.median_page_load_ms as u64;
        self.p95_load_time_ms = self.aggregated.p95_page_load_ms as u64;
        self.slow_pages = load_times.iter().filter(|&&t| t > 3000).count() as u64;
        
        if !self.memory_samples.is_empty() {
            self.aggregated.avg_memory_mb = self.memory_samples.iter()
                .map(|m| m.used_mb)
                .sum::<f64>() / self.memory_samples.len() as f64;
            self.average_memory_mb = self.aggregated.avg_memory_mb as u64;
            self.peak_memory_mb = self.memory_samples.iter()
                .map(|m| m.used_mb as u64)
                .max()
                .unwrap_or(0);
        }
        
        if !self.cpu_samples.is_empty() {
            self.aggregated.avg_cpu_percent = self.cpu_samples.iter()
                .map(|c| c.percent)
                .sum::<f64>() / self.cpu_samples.len() as f64;
            self.average_cpu_percent = self.aggregated.avg_cpu_percent as u64;
        }
    }

    /// Get summary
    pub fn summary(&self) -> super::PerformanceSummary {
        let slowest_pages: Vec<(String, u64)> = self.page_loads.iter()
            .map(|p| (p.url.clone(), p.load_complete_ms))
            .collect();
        
        let mut slowest = slowest_pages;
        slowest.sort_by(|a, b| b.1.cmp(&a.1));
        slowest.truncate(10);

        super::PerformanceSummary {
            avg_page_load_ms: self.aggregated.avg_page_load_ms,
            avg_memory_mb: self.aggregated.avg_memory_mb,
            avg_cpu_percent: self.aggregated.avg_cpu_percent,
            total_data_transferred_mb: self.network_stats.total_bytes_received as f64 / 1_048_576.0,
            slowest_pages: slowest,
        }
    }

    /// Clear all metrics
    pub fn clear(&mut self) {
        *self = Self::new();
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Security metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMetrics {
    /// Total threats blocked
    pub threats_blocked: u64,
    /// HTTPS upgrades
    pub https_upgrades: u64,
    /// Trackers blocked
    pub trackers_blocked: u64,
    /// Cookies blocked
    pub cookies_blocked: u64,
    /// Fingerprinting attempts blocked
    pub fingerprinting_blocked: u64,
    /// Malicious sites blocked
    pub malicious_sites_blocked: u64,
    /// Phishing attempts blocked
    pub phishing_blocked: u64,
    /// Malware blocked
    pub malware_blocked: u64,
    /// Event history
    event_history: Vec<SecurityEventRecord>,
    /// Threats by type
    threats_by_type: HashMap<String, u64>,
    /// Trackers by domain
    trackers_by_domain: HashMap<String, u64>,
}

/// Security event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEventRecord {
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub details: String,
    pub url: Option<String>,
}

impl SecurityMetrics {
    /// Create new security metrics
    pub fn new() -> Self {
        Self {
            threats_blocked: 0,
            https_upgrades: 0,
            trackers_blocked: 0,
            cookies_blocked: 0,
            fingerprinting_blocked: 0,
            malicious_sites_blocked: 0,
            phishing_blocked: 0,
            malware_blocked: 0,
            event_history: Vec::new(),
            threats_by_type: HashMap::new(),
            trackers_by_domain: HashMap::new(),
        }
    }

    /// Record a security event
    pub fn record_event(&mut self, event: super::SecurityEvent) {
        let now = Utc::now();
        
        match event {
            super::SecurityEvent::ThreatBlocked { threat_type, url } => {
                self.threats_blocked += 1;
                *self.threats_by_type.entry(threat_type.clone()).or_insert(0) += 1;
                self.event_history.push(SecurityEventRecord {
                    timestamp: now,
                    event_type: "threat_blocked".to_string(),
                    details: threat_type,
                    url: Some(url),
                });
            }
            super::SecurityEvent::HttpsUpgrade { url } => {
                self.https_upgrades += 1;
                self.event_history.push(SecurityEventRecord {
                    timestamp: now,
                    event_type: "https_upgrade".to_string(),
                    details: "Upgraded to HTTPS".to_string(),
                    url: Some(url),
                });
            }
            super::SecurityEvent::TrackingAttempt { tracker, url } => {
                self.trackers_blocked += 1;
                *self.trackers_by_domain.entry(tracker.clone()).or_insert(0) += 1;
                self.event_history.push(SecurityEventRecord {
                    timestamp: now,
                    event_type: "tracking_blocked".to_string(),
                    details: tracker,
                    url: Some(url),
                });
            }
            super::SecurityEvent::MaliciousSiteBlocked { url, reason } => {
                self.malicious_sites_blocked += 1;
                self.event_history.push(SecurityEventRecord {
                    timestamp: now,
                    event_type: "malicious_blocked".to_string(),
                    details: reason,
                    url: Some(url),
                });
            }
            super::SecurityEvent::PhishingAttemptBlocked { url } => {
                self.phishing_blocked += 1;
                self.event_history.push(SecurityEventRecord {
                    timestamp: now,
                    event_type: "phishing_blocked".to_string(),
                    details: "Phishing attempt blocked".to_string(),
                    url: Some(url),
                });
            }
            super::SecurityEvent::CookieBlocked { domain } => {
                self.cookies_blocked += 1;
                self.event_history.push(SecurityEventRecord {
                    timestamp: now,
                    event_type: "cookie_blocked".to_string(),
                    details: domain,
                    url: None,
                });
            }
            super::SecurityEvent::FingerprintingAttempt { technique } => {
                self.fingerprinting_blocked += 1;
                self.event_history.push(SecurityEventRecord {
                    timestamp: now,
                    event_type: "fingerprinting_blocked".to_string(),
                    details: technique,
                    url: None,
                });
            }
        }

        // Limit history
        if self.event_history.len() > 1000 {
            self.event_history.drain(0..100);
        }
    }

    /// Calculate security score (0-100)
    pub fn calculate_score(&self) -> f64 {
        // Base score
        let mut score = 100.0;
        
        // Deduct for any security events that weren't blocked
        // This is a simplified scoring - real implementation would be more sophisticated
        if self.threats_blocked > 0 {
            score = (score + 5.0).min(100.0); // Bonus for blocking threats
        }
        
        score
    }

    /// Get summary
    pub fn summary(&self) -> super::SecuritySummary {
        super::SecuritySummary {
            total_threats_blocked: self.threats_blocked,
            https_upgrades: self.https_upgrades,
            trackers_blocked: self.trackers_blocked,
            cookies_blocked: self.cookies_blocked,
            security_score: self.calculate_score(),
        }
    }

    /// Clear all metrics
    pub fn clear(&mut self) {
        *self = Self::new();
    }
}

impl Default for SecurityMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Analytics snapshot for a point in time
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalyticsSnapshot {
    /// Timestamp when snapshot was taken
    pub timestamp: DateTime<Utc>,
    /// Usage metrics at this point
    pub usage: UsageMetrics,
    /// Performance metrics at this point
    pub performance: PerformanceMetrics,
    /// Security metrics at this point
    pub security: SecurityMetrics,
}

impl AnalyticsSnapshot {
    /// Create a new snapshot
    pub fn new() -> Self {
        Self {
            timestamp: Utc::now(),
            usage: UsageMetrics::new(),
            performance: PerformanceMetrics::new(),
            security: SecurityMetrics::new(),
        }
    }
    
    /// Create a snapshot from components
    pub fn from_components(
        usage: UsageMetrics,
        performance: PerformanceMetrics,
        security: SecurityMetrics,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            usage,
            performance,
            security,
        }
    }
}

/// Extended domain stats for dashboard display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainVisitStats {
    pub domain: String,
    pub visits: u64,
    pub time_ms: u64,
    pub category: Option<String>,
}

impl From<DomainStats> for DomainVisitStats {
    fn from(stats: DomainStats) -> Self {
        Self {
            domain: stats.domain,
            visits: stats.visits,
            time_ms: stats.total_time_ms,
            category: stats.category,
        }
    }
}