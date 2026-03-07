//! ML-based Ad Detector
//! 
//! This module uses machine learning to detect advertisements based on
//! URL patterns, content analysis, and behavioral heuristics.
//! 
//! # Features
//! - TensorFlow model for ad detection
//! - URL pattern analysis
//! - Content-based detection
//! - Confidence scoring
//! - Model training and updating

use std::collections::HashSet;
use regex::Regex;
use super::{Result, AdBlockError, BlockReason, ResourceType};

/// Ad detector with ML capabilities
pub struct AdDetector {
    model: Option<TensorFlowModel>,
    url_patterns: HashSet<Regex>,
    ad_domains: HashSet<String>,
    confidence_threshold: f32,
}

/// Simulated TensorFlow model
struct TensorFlowModel {
    is_loaded: bool,
    model_path: Option<String>,
}

impl TensorFlowModel {
    fn new() -> Self {
        Self {
            is_loaded: false,
            model_path: None,
        }
    }

    fn load_model(&mut self, _path: &str) -> Result<()> {
        // In a real implementation, this would load a TensorFlow model
        self.is_loaded = true;
        Ok(())
    }

    fn predict(&self, features: &DetectionFeatures) -> f32 {
        // Simulated ML prediction
        let mut score = 0.0;

        // URL-based features
        if features.has_ad_keywords {
            score += 0.4;
        }
        if features.has_tracker_keywords {
            score += 0.3;
        }
        if features.length > 50 {
            score += 0.1;
        }

        score.min(1.0)
    }
}

/// Detection features for ML model
#[derive(Debug, Clone)]
struct DetectionFeatures {
    has_ad_keywords: bool,
    has_tracker_keywords: bool,
    length: usize,
    has_numbers: bool,
    subdomain_count: usize,
}

impl AdDetector {
    /// Create a new ad detector
    pub fn new() -> Self {
        let url_patterns = Self::build_url_patterns();
        let ad_domains = Self::build_ad_domains();

        Self {
            model: None,
            url_patterns,
            ad_domains,
            confidence_threshold: 0.7,
        }
    }

    /// Create detector with ML model
    pub fn with_model(mut self, model_path: &str) -> Result<Self> {
        let mut model = TensorFlowModel::new();
        model.load_model(model_path)?;
        self.model = Some(model);
        Ok(self)
    }

    /// Detect if a URL is an advertisement
    pub async fn detect_ad(&self, url: &str) -> Result<Option<BlockReason>> {
        // Fast path: Check against known ad domains
        if self.is_ad_domain(url) {
            return Ok(Some(BlockReason {
                resource_type: ResourceType::Advertisement,
                rule_id: "domain-block".to_string(),
                confidence: 1.0,
                description: "Known ad domain".to_string(),
            }));
        }

        // Check URL patterns
        if let Some(pattern) = self.check_url_patterns(url) {
            return Ok(Some(BlockReason {
                resource_type: ResourceType::Advertisement,
                rule_id: "pattern-block".to_string(),
                confidence: 0.9,
                description: format!("Matches pattern: {}", pattern),
            }));
        }

        // ML-based detection
        if let Some(ref model) = self.model {
            let features = self.extract_features(url);
            let confidence = model.predict(&features);

            if confidence >= self.confidence_threshold {
                return Ok(Some(BlockReason {
                    resource_type: ResourceType::Advertisement,
                    rule_id: "ml-detection".to_string(),
                    confidence,
                    description: "ML-based detection".to_string(),
                }));
            }
        }

        Ok(None)
    }

    /// Check if URL contains known ad domain
    fn is_ad_domain(&self, url: &str) -> bool {
        if let Ok(parsed) = url::Url::parse(url) {
            if let Some(domain) = parsed.host_str() {
                // Check exact domain
                if self.ad_domains.contains(domain) {
                    return true;
                }

                // Check subdomains
                for ad_domain in &self.ad_domains {
                    if domain.ends_with(&format!(".{}", ad_domain)) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Check URL against patterns
    fn check_url_patterns(&self, url: &str) -> Option<String> {
        for pattern in &self.url_patterns {
            if pattern.is_match(url) {
                return Some(pattern.as_str().to_string());
            }
        }

        None
    }

    /// Extract features for ML model
    fn extract_features(&self, url: &str) -> DetectionFeatures {
        let ad_keywords = ["ad", "ads", "advertisement", "banner", "promo", "sponsor"];
        let tracker_keywords = ["track", "pixel", "beacon", "analytics", "telemetry"];

        let url_lower = url.to_lowercase();

        let has_ad_keywords = ad_keywords.iter().any(|k| url_lower.contains(k));
        let has_tracker_keywords = tracker_keywords.iter().any(|k| url_lower.contains(k));

        let subdomain_count = url.split('.').count().saturating_sub(2);

        let has_numbers = url.chars().any(|c| c.is_numeric());

        DetectionFeatures {
            has_ad_keywords,
            has_tracker_keywords,
            length: url.len(),
            has_numbers,
            subdomain_count,
        }
    }

    /// Build URL patterns for ad detection
    fn build_url_patterns() -> HashSet<Regex> {
        let patterns = vec![
            // Common ad patterns
            r"/ad[s]?\.",
            r"/banner[s]?/",
            r"/promo[s]?/",
            r"/sponsor[s]?/",
            r"/affiliate[s]?/",
            r"/doubleclick\.net",
            r"/googleadservices\.com",
            r"/googlesyndication\.com",
            r"/facebook\.com/tr/",
            r"/analytics\.google\.com",
            r"/tracking\.",
            r"/pixel\.",
            r"/beacon\.",
            // Size patterns
            r"/468x60",
            r"/728x90",
            r"/300x250",
            r"/160x600",
            // Parameter patterns
            r"[?&](ad_|banner|campaign|referral)=",
        ];

        patterns
            .into_iter()
            .filter_map(|p| Regex::new(p).ok())
            .collect()
    }

    /// Build list of known ad domains
    fn build_ad_domains() -> HashSet<String> {
        vec![
            // Ad networks
            "doubleclick.net",
            "googleadservices.com",
            "googlesyndication.com",
            "facebook.com",
            "amazon-adsystem.com",
            "taboola.com",
            "outbrain.com",
            "taboola.com",
            "advertising.com",
            "adsymptotic.com",
            "adtech.com",
            "adsrvr.org",
            "automattic.com",
            // Tracking domains
            "google-analytics.com",
            "analytics.google.com",
            "stats.g.doubleclick.net",
            "pixel.facebook.com",
            "connect.facebook.net",
            "pixel.wp.com",
            "analytics.twitter.com",
            "sb.scorecardresearch.com",
            // Ad servers
            "adserver.com",
            "adsystem.com",
            "adnetwork.com",
            "adserve.com",
            "adskeeper.co.uk",
            "popads.net",
            "propellerads.com",
            "revcontent.com",
            "mgid.com",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    }

    /// Set confidence threshold
    pub fn set_confidence_threshold(&mut self, threshold: f32) {
        self.confidence_threshold = threshold.clamp(0.0, 1.0);
    }

    /// Get current confidence threshold
    pub fn get_confidence_threshold(&self) -> f32 {
        self.confidence_threshold
    }

    /// Update URL patterns
    pub fn update_patterns(&mut self, new_patterns: Vec<String>) {
        let mut patterns = self.url_patterns.clone();
        for pattern in new_patterns {
            if let Ok(regex) = Regex::new(&pattern) {
                patterns.insert(regex);
            }
        }
        self.url_patterns = patterns;
    }

    /// Add ad domain to block list
    pub fn add_ad_domain(&mut self, domain: String) {
        self.ad_domains.insert(domain);
    }

    /// Remove ad domain from block list
    pub fn remove_ad_domain(&mut self, domain: &str) {
        self.ad_domains.remove(domain);
    }

    /// Get statistics about the detector
    pub fn get_statistics(&self) -> DetectorStatistics {
        DetectorStatistics {
            model_loaded: self.model.as_ref().map(|m| m.is_loaded).unwrap_or(false),
            url_patterns_count: self.url_patterns.len(),
            ad_domains_count: self.ad_domains.len(),
            confidence_threshold: self.confidence_threshold,
        }
    }
}

impl Default for AdDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Detector statistics
#[derive(Debug, Clone)]
pub struct DetectorStatistics {
    pub model_loaded: bool,
    pub url_patterns_count: usize,
    pub ad_domains_count: usize,
    pub confidence_threshold: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_detect_known_ad_domain() {
        let detector = AdDetector::new();
        let result = detector.detect_ad("https://doubleclick.net/ad").await;
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_detect_ad_pattern() {
        let detector = AdDetector::new();
        let result = detector.detect_ad("https://example.com/ads/banner.jpg").await;
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_feature_extraction() {
        let detector = AdDetector::new();
        let features = detector.extract_features("https://ads.example.com/banner?ad_id=123");
        assert!(features.has_ad_keywords);
        assert!(features.has_numbers);
    }

    #[test]
    fn test_confidence_threshold() {
        let mut detector = AdDetector::new();
        detector.set_confidence_threshold(0.8);
        assert_eq!(detector.get_confidence_threshold(), 0.8);

        detector.set_confidence_threshold(1.5);
        assert_eq!(detector.get_confidence_threshold(), 1.0);
    }
}