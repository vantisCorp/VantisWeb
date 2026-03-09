//! Tracker Detector for VantisWeb Ad Blocker
//! 
//! This module identifies and blocks web tracking mechanisms including:
//! - Third-party tracking scripts
//! - Fingerprinting scripts
//! - Tracking cookies
//! - Beacon/pixel tracking
//! - Canvas/WebGL fingerprinting

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Known tracking domains and services
const TRACKING_DOMAINS: &[&str] = &[
    "google-analytics.com",
    "googletagmanager.com",
    "googlesyndication.com",
    "doubleclick.net",
    "facebook.com/tr",
    "connect.facebook.net",
    "stats.g.doubleclick.net",
    "analytics.twitter.com",
    "analytics.yahoo.com",
    "pixel.wp.com",
    "beacon.krxd.net",
    "ads.mopub.com",
    "adserver.adtech.de",
    "advertising.com",
    "criteo.com",
    "outbrain.com",
    "taboola.com",
    "scorecardresearch.com",
    "quantserve.com",
    "comscore.com",
    "chartbeat.com",
    "mixpanel.com",
    "segment.com",
    "amplitude.com",
    "fullstory.com",
    "hotjar.com",
    "mouseflow.com",
    "userzoom.com",
    "optimizely.com",
    "vwo.com",
    "abtasty.com",
    "crazyegg.com",
    "inspectlet.com",
    "sessioncam.com",
    "clicktale.com",
    "mousestats.com",
];

/// Known fingerprinting script patterns
const FINGERPRINT_SCRIPTS: &[&str] = &[
    "fingerprint",
    "fingerprint2",
    "clientjs",
    "fingerprintjs",
    "fpring",
    "device-fingerprint",
    "browserfingerprint",
    "canvas-fingerprint",
    "webgl-fingerprint",
];

/// Known tracking cookie names
const TRACKING_COOKIES: &[&str] = &[
    "_ga",
    "_gid",
    "_gat",
    "_fbp",
    "_fbc",
    "fr",
    "tr",
    "datr",
    "sb",
    "oo",
    "nuid",
    "muid",
    "_uetvid",
    "ANONCHK",
    "MR",
    "MUID",
    "MC1",
    "VISITOR_INFO1_LIVE",
    "YSC",
    "PREF",
    "SID",
    "HSID",
    "SSID",
    "APISID",
    "SAPISID",
    "BAIDUID",
    "BDORZ",
    "Hm_lvt",
    "Hm_lpvt",
];

/// A detected tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tracker {
    /// Unique identifier
    pub id: String,
    /// Tracker name/company
    pub name: String,
    /// Tracker category
    pub category: TrackerCategory,
    /// Tracker type
    pub tracker_type: TrackerType,
    /// Domain hosting the tracker
    pub domain: String,
    /// URL path
    pub url: String,
    /// When detected
    pub detected_at: DateTime<Utc>,
    /// Page where detected
    pub page_url: String,
    /// Severity level
    pub severity: Severity,
    /// Whether blocked
    pub blocked: bool,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Tracker categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackerCategory {
    /// Analytics trackers
    Analytics,
    /// Advertising trackers
    Advertising,
    /// Social media widgets
    Social,
    /// Fingerprinting scripts
    Fingerprinting,
    /// Beacons/pixels
    Beacon,
    /// Unknown tracker
    Unknown,
}

/// Tracker types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackerType {
    /// JavaScript tracking script
    Script,
    /// Tracking pixel/beacon
    Pixel,
    /// Tracking cookie
    Cookie,
    /// Web beacon
    WebBeacon,
    /// Canvas fingerprinting
    Canvas,
    /// WebGL fingerprinting
    WebGL,
    /// LocalStorage tracking
    LocalStorage,
    /// IndexedDB tracking
    IndexedDB,
    /// Unknown type
    Unknown,
}

/// Severity levels for trackers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// Low - benign tracking
    Low,
    /// Medium - typical analytics
    Medium,
    /// High - aggressive tracking
    High,
    /// Critical - fingerprinting
    Critical,
}

/// Tracker detector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerConfig {
    /// Enable tracker detection
    pub enable_detection: bool,
    /// Enable tracker blocking
    pub enable_blocking: bool,
    /// Severity threshold for blocking
    pub blocking_threshold: Severity,
    /// Whether to block analytics trackers
    pub block_analytics: bool,
    /// Whether to block advertising trackers
    pub block_ads: bool,
    /// Whether to block social widgets
    pub block_social: bool,
    /// Whether to block fingerprinting
    pub block_fingerprinting: bool,
    /// Strict mode (block all trackers)
    pub strict_mode: bool,
}

impl Default for TrackerConfig {
    fn default() -> Self {
        Self {
            enable_detection: true,
            enable_blocking: true,
            blocking_threshold: Severity::Medium,
            block_analytics: true,
            block_ads: true,
            block_social: false,
            block_fingerprinting: true,
            strict_mode: false,
        }
    }
}

/// Statistics about tracker detection and blocking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerStats {
    /// Total trackers detected
    pub total_detected: usize,
    /// Total trackers blocked
    pub total_blocked: usize,
    /// Count by category
    pub by_category: HashMap<TrackerCategory, usize>,
    /// Count by type
    pub by_type: HashMap<TrackerType, usize>,
    /// Most common trackers
    pub top_trackers: Vec<(String, usize)>,
    /// Tracking domains encountered
    pub tracking_domains: HashSet<String>,
    /// Start time of statistics
    pub since: DateTime<Utc>,
}

/// Tracker detector
pub struct TrackerDetector {
    /// Configuration
    config: Arc<RwLock<TrackerConfig>>,
    /// Known tracking domains
    tracking_domains: HashSet<String>,
    /// Known fingerprinting patterns
    fingerprint_patterns: HashSet<String>,
    /// Known tracking cookie names
    tracking_cookies: HashSet<String>,
    /// Detected trackers
    detected: Arc<RwLock<Vec<Tracker>>>,
    /// Statistics
    stats: Arc<RwLock<TrackerStats>>,
}

impl TrackerDetector {
    /// Create a new tracker detector
    pub fn new() -> Self {
        let tracking_domains = TRACKING_DOMAINS.iter().map(|s| s.to_string()).collect();
        let fingerprint_patterns = FINGERPRINT_SCRIPTS.iter().map(|s| s.to_string()).collect();
        let tracking_cookies = TRACKING_COOKIES.iter().map(|s| s.to_string()).collect();
        
        Self {
            config: Arc::new(RwLock::new(TrackerConfig::default())),
            tracking_domains,
            fingerprint_patterns,
            tracking_cookies,
            detected: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(TrackerStats {
                total_detected: 0,
                total_blocked: 0,
                by_category: HashMap::new(),
                by_type: HashMap::new(),
                top_trackers: Vec::new(),
                tracking_domains: HashSet::new(),
                since: Utc::now(),
            })),
        }
    }

    /// Check if a URL is a tracker
    pub async fn check_url(&self, url: &str, page_url: &str) -> Option<Tracker> {
        let config = self.config.read().await;
        if !config.enable_detection {
            return None;
        }
        drop(config);
        
        // Parse URL
        let parsed = url::Url::parse(url).ok()?;
        let domain = parsed.domain()?;
        
        // Check if it's a known tracking domain
        if self.is_tracking_domain(domain) {
            return Some(self.create_tracker(url, page_url, domain));
        }
        
        // Check for fingerprinting patterns in URL path
        let path = parsed.path();
        if self.is_fingerprinting_script(path) {
            return Some(self.create_tracker(url, page_url, domain));
        }
        
        None
    }

    /// Check if a cookie is a tracking cookie
    pub async fn check_cookie(&self, name: &str, domain: &str, page_url: &str) -> Option<Tracker> {
        let config = self.config.read().await;
        if !config.enable_detection {
            return None;
        }
        drop(config);
        
        if self.tracking_cookies.contains(name) {
            Some(Tracker {
                id: format!("cookie_{}_{}", name, Utc::now().timestamp_millis()),
                name: name.to_string(),
                category: TrackerCategory::Unknown,
                tracker_type: TrackerType::Cookie,
                domain: domain.to_string(),
                url: format!("cookie://{}?{}", domain, name),
                detected_at: Utc::now(),
                page_url: page_url.to_string(),
                severity: Severity::Medium,
                blocked: false,
                metadata: HashMap::new(),
            })
        } else {
            None
        }
    }

    /// Check if a script is fingerprinting
    pub async fn check_script(&self, script_content: &str, script_url: &str, page_url: &str) -> Option<Tracker> {
        let config = self.config.read().await;
        if !config.enable_detection {
            return None;
        }
        drop(config);
        
        // Check for canvas fingerprinting
        if script_content.contains("canvas") && 
           (script_content.contains("toDataURL") || script_content.contains("getImageData")) {
            let parsed = url::Url::parse(script_url).ok()?;
            let domain = parsed.domain().unwrap_or("unknown");
            
            return Some(Tracker {
                id: format!("canvas_{}", Utc::now().timestamp_millis()),
                name: "Canvas Fingerprinting".to_string(),
                category: TrackerCategory::Fingerprinting,
                tracker_type: TrackerType::Canvas,
                domain: domain.to_string(),
                url: script_url.to_string(),
                detected_at: Utc::now(),
                page_url: page_url.to_string(),
                severity: Severity::Critical,
                blocked: false,
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("method".to_string(), "canvas".to_string());
                    meta
                },
            });
        }
        
        // Check for WebGL fingerprinting
        if script_content.contains("webgl") && 
           (script_content.contains("getSupportedExtensions") || script_content.contains("getParameter")) {
            let parsed = url::Url::parse(script_url).ok()?;
            let domain = parsed.domain().unwrap_or("unknown");
            
            return Some(Tracker {
                id: format!("webgl_{}", Utc::now().timestamp_millis()),
                name: "WebGL Fingerprinting".to_string(),
                category: TrackerCategory::Fingerprinting,
                tracker_type: TrackerType::WebGL,
                domain: domain.to_string(),
                url: script_url.to_string(),
                detected_at: Utc::now(),
                page_url: page_url.to_string(),
                severity: Severity::Critical,
                blocked: false,
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("method".to_string(), "webgl".to_string());
                    meta
                },
            });
        }
        
        None
    }

    /// Check if LocalStorage is being used for tracking
    pub async fn check_localstorage(&self, key: &str, value: &str, domain: &str, page_url: &str) -> Option<Tracker> {
        // Check for suspicious keys
        let tracking_keys = vec![
            "fingerprint", "device", "session", "tracking", "visitor",
            "analytics", "telemetry", "metrics", "userid", "guid",
        ];
        
        if tracking_keys.iter().any(|k| key.to_lowercase().contains(k)) {
            Some(Tracker {
                id: format!("ls_{}_{}", key, Utc::now().timestamp_millis()),
                name: format!("LocalStorage Tracking: {}", key),
                category: TrackerCategory::Unknown,
                tracker_type: TrackerType::LocalStorage,
                domain: domain.to_string(),
                url: format!("localStorage://{}?{}", domain, key),
                detected_at: Utc::now(),
                page_url: page_url.to_string(),
                severity: Severity::Medium,
                blocked: false,
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("key".to_string(), key.to_string());
                    meta.insert("value_length".to_string(), value.len().to_string());
                    meta
                },
            })
        } else {
            None
        }
    }

    /// Record a detected tracker
    pub async fn record_tracker(&self, tracker: Tracker) {
        let config = self.config.read().await;
        
        // Check if it should be blocked
        let should_block = config.enable_blocking && 
            (config.strict_mode || 
             tracker.severity >= config.blocking_threshold ||
             match tracker.category {
                 TrackerCategory::Fingerprinting => config.block_fingerprinting,
                 TrackerCategory::Analytics => config.block_analytics,
                 TrackerCategory::Advertising => config.block_ads,
                 TrackerCategory::Social => config.block_social,
                 _ => false,
             });
        drop(config);
        
        let mut tracker = tracker;
        tracker.blocked = should_block;
        
        // Update statistics
        let mut stats = self.stats.write().await;
        stats.total_detected += 1;
        if should_block {
            stats.total_blocked += 1;
        }
        *stats.by_category.entry(tracker.category).or_insert(0) += 1;
        *stats.by_type.entry(tracker.tracker_type).or_insert(0) += 1;
        stats.tracking_domains.insert(tracker.domain.clone());
        
        // Update top trackers
        let tracker_count = stats.top_trackers.iter()
            .find(|(n, _)| n == &tracker.name)
            .map(|(_, c)| *c)
            .unwrap_or(0) + 1;
        
        stats.top_trackers.retain(|(n, _)| n != &tracker.name);
        stats.top_trackers.push((tracker.name.clone(), tracker_count));
        stats.top_trackers.sort_by(|a, b| b.1.cmp(&a.1));
        stats.top_trackers.truncate(10);
        
        drop(stats);
        
        // Store tracker
        let mut detected = self.detected.write().await;
        detected.push(tracker);
    }

    /// Check if a URL should be blocked
    pub async fn should_block_url(&self, url: &str, page_url: &str) -> bool {
        if let Some(tracker) = self.check_url(url, page_url).await {
            self.record_tracker(tracker.clone()).await;
            tracker.blocked
        } else {
            false
        }
    }

    /// Get detected trackers for a page
    pub async fn get_page_trackers(&self, page_url: &str) -> Vec<Tracker> {
        let detected = self.detected.read().await;
        detected.iter()
            .filter(|t| t.page_url == page_url)
            .cloned()
            .collect()
    }

    /// Get statistics
    pub async fn get_stats(&self) -> TrackerStats {
        self.stats.read().await.clone()
    }

    /// Clear all detected trackers
    pub async fn clear_detected(&self) {
        self.detected.write().await.clear();
    }

    /// Update configuration
    pub async fn update_config<F>(&self, f: F)
    where
        F: FnOnce(&mut TrackerConfig),
    {
        let mut config = self.config.write().await;
        f(&mut config);
    }

    /// Get configuration
    pub async fn get_config(&self) -> TrackerConfig {
        self.config.read().await.clone()
    }

    /// Check if a domain is a known tracking domain
    fn is_tracking_domain(&self, domain: &str) -> bool {
        self.tracking_domains.contains(domain) || 
        self.tracking_domains.iter().any(|d| domain.ends_with(d))
    }

    /// Check if a path contains fingerprinting patterns
    fn is_fingerprinting_script(&self, path: &str) -> bool {
        let path_lower = path.to_lowercase();
        self.fingerprint_patterns.iter().any(|p| path_lower.contains(p))
    }

    /// Create a tracker object
    fn create_tracker(&self, url: &str, page_url: &str, domain: &str) -> Tracker {
        let (category, severity) = if self.is_tracking_domain(domain) {
            if domain.contains("analytics") {
                (TrackerCategory::Analytics, Severity::Medium)
            } else if domain.contains("ad") {
                (TrackerCategory::Advertising, Severity::High)
            } else {
                (TrackerCategory::Unknown, Severity::Medium)
            }
        } else {
            (TrackerCategory::Fingerprinting, Severity::Critical)
        };
        
        Tracker {
            id: format!("tracker_{}_{}", domain, Utc::now().timestamp_millis()),
            name: domain.to_string(),
            category,
            tracker_type: TrackerType::Script,
            domain: domain.to_string(),
            url: url.to_string(),
            detected_at: Utc::now(),
            page_url: page_url.to_string(),
            severity,
            blocked: false,
            metadata: HashMap::new(),
        }
    }

    /// Export trackers to JSON
    pub async fn export_trackers(&self) -> Result<String, serde_json::Error> {
        let detected = self.detected.read().await;
        serde_json::to_string_pretty(&*detected)
    }

    /// Import trackers from JSON
    pub async fn import_trackers(&self, json: &str) -> Result<usize, serde_json::Error> {
        let imported: Vec<Tracker> = serde_json::from_str(json)?;
        let count = imported.len();
        
        let mut detected = self.detected.write().await;
        detected.extend(imported);
        
        Ok(count)
    }
}

impl Default for TrackerDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_detector() {
        let detector = TrackerDetector::new();
        let stats = detector.get_stats().await;
        assert_eq!(stats.total_detected, 0);
    }

    #[tokio::test]
    async fn test_detect_tracking_domain() {
        let detector = TrackerDetector::new();
        let tracker = detector.check_url("https://google-analytics.com/collect", "https://example.com").await;
        
        assert!(tracker.is_some());
        let tracker = tracker.unwrap();
        assert_eq!(tracker.category, TrackerCategory::Analytics);
    }

    #[tokio::test]
    async fn test_detect_tracking_cookie() {
        let detector = TrackerDetector::new();
        let tracker = detector.check_cookie("_ga", "example.com", "https://example.com").await;
        
        assert!(tracker.is_some());
        let tracker = tracker.unwrap();
        assert_eq!(tracker.tracker_type, TrackerType::Cookie);
    }

    #[tokio::test]
    async fn test_detect_canvas_fingerprinting() {
        let detector = TrackerDetector::new();
        let script = r##"
            var canvas = document.createElement('canvas');
            var ctx = canvas.getContext('2d');
            var data = ctx.getImageData(0, 0, canvas.width, canvas.height);
        "##;
        let tracker = detector.check_script(script, "https://example.com/script.js", "https://example.com").await;
        
        assert!(tracker.is_some());
        let tracker = tracker.unwrap();
        assert_eq!(tracker.tracker_type, TrackerType::Canvas);
        assert_eq!(tracker.severity, Severity::Critical);
    }

    #[tokio::test]
    async fn test_blocking_config() {
        let detector = TrackerDetector::new();
        
        detector.update_config(|config| {
            config.blocking_threshold = Severity::Critical;
        }).await;
        
        let tracker = detector.should_block_url("https://google-analytics.com/collect", "https://example.com").await;
        assert!(!tracker); // Medium severity shouldn't be blocked
    }

    #[tokio::test]
    async fn test_strict_mode() {
        let detector = TrackerDetector::new();
        
        detector.update_config(|config| {
            config.strict_mode = true;
        }).await;
        
        let tracker = detector.should_block_url("https://google-analytics.com/collect", "https://example.com").await;
        assert!(tracker); // Should block in strict mode
    }
}