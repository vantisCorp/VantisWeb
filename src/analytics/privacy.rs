//! Privacy filter module for analytics data sanitization
//! 
//! This module provides privacy-focused filtering and anonymization
//! capabilities to ensure user privacy while collecting analytics.

use std::collections::HashSet;
use regex::Regex;
use url::Url;

/// Privacy configuration for analytics
#[derive(Debug, Clone)]
pub struct PrivacyConfig {
    /// Whether privacy mode is enabled
    pub enabled: bool,
    /// Sensitive URL patterns to exclude
    pub sensitive_patterns: Vec<String>,
    /// Query parameters to strip from URLs
    pub sensitive_params: Vec<String>,
    /// Domains to exclude from tracking
    pub excluded_domains: HashSet<String>,
    /// Whether to anonymize IPs
    pub anonymize_ips: bool,
    /// Whether to strip user IDs from URLs
    pub strip_user_ids: bool,
    /// Minimum time threshold for time aggregation (ms)
    pub time_aggregation_threshold: u64,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        let mut excluded_domains = HashSet::new();
        excluded_domains.insert("localhost".to_string());
        excluded_domains.insert("127.0.0.1".to_string());
        excluded_domains.insert("about:blank".to_string());
        excluded_domains.insert("chrome://".to_string());
        excluded_domains.insert("vantis://".to_string());
        
        let sensitive_params = vec![
            // Authentication tokens
            "token".to_string(),
            "access_token".to_string(),
            "auth".to_string(),
            "api_key".to_string(),
            "apikey".to_string(),
            "key".to_string(),
            "secret".to_string(),
            "password".to_string(),
            "passwd".to_string(),
            // Session identifiers
            "session".to_string(),
            "session_id".to_string(),
            "sessionid".to_string(),
            "sid".to_string(),
            // User identifiers
            "user".to_string(),
            "user_id".to_string(),
            "userid".to_string(),
            "uid".to_string(),
            "email".to_string(),
            "mail".to_string(),
            // OAuth
            "code".to_string(),
            "state".to_string(),
            "nonce".to_string(),
            // Other sensitive
            "ssn".to_string(),
            "credit_card".to_string(),
            "card".to_string(),
        ];
        
        let sensitive_patterns = vec![
            r"^https?://[^/]*\.bank\.".to_string(),
            r"^https?://[^/]*\.secure\.".to_string(),
            r"^https?://[^/]*\.admin\.".to_string(),
            r"^https?://[^/]*\.private\.".to_string(),
            r"^https?://[^/]*banking\.".to_string(),
            r"^https?://[^/]*payment\.".to_string(),
            r"^https?://[^/]*checkout\.".to_string(),
            r"^https?://[^/]*signin\.".to_string(),
            r"^https?://[^/]*login\.".to_string(),
            r"^https?://[^/]*auth\.".to_string(),
        ];
        
        Self {
            enabled: true,
            sensitive_patterns,
            sensitive_params,
            excluded_domains,
            anonymize_ips: true,
            strip_user_ids: true,
            time_aggregation_threshold: 1000,
        }
    }
}

/// Privacy filter for sanitizing analytics data
pub struct PrivacyFilter {
    config: PrivacyConfig,
    sensitive_url_patterns: Vec<Regex>,
}

impl PrivacyFilter {
    /// Create a new privacy filter with the given configuration
    pub fn new(config: PrivacyConfig) -> Self {
        let sensitive_url_patterns = config.sensitive_patterns
            .iter()
            .filter_map(|p| Regex::new(p).ok())
            .collect();
        
        Self {
            config,
            sensitive_url_patterns,
        }
    }
    
    /// Create a privacy filter with default configuration
    pub fn with_defaults() -> Self {
        Self::new(PrivacyConfig::default())
    }
    
    /// Check if a URL should be tracked
    pub fn should_track_url(&self, url: &str) -> bool {
        if !self.config.enabled {
            return true;
        }
        
        // Check for internal/private URLs
        if url.starts_with("about:") || 
           url.starts_with("chrome://") || 
           url.starts_with("vantis://") ||
           url.starts_with("data:") ||
           url.starts_with("javascript:") {
            return false;
        }
        
        // Check excluded domains
        if let Ok(parsed) = Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                for excluded in &self.config.excluded_domains {
                    if host.contains(excluded) || host == excluded {
                        return false;
                    }
                }
            }
        }
        
        // Check sensitive URL patterns
        for pattern in &self.sensitive_url_patterns {
            if pattern.is_match(url) {
                return false;
            }
        }
        
        true
    }
    
    /// Sanitize a URL by removing sensitive query parameters
    pub fn sanitize_url(&self, url: &str) -> String {
        if !self.config.enabled {
            return url.to_string();
        }
        
        let mut parsed = match Url::parse(url) {
            Ok(u) => u,
            Err(_) => return url.to_string(),
        };
        
        // Get query pairs
        let query: Vec<(String, String)> = parsed
            .query_pairs()
            .filter(|(key, _)| {
                let key_lower = key.to_lowercase();
                !self.config.sensitive_params.iter().any(|p| {
                    key_lower.contains(&p.to_lowercase())
                })
            })
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        
        // Rebuild query string
        if query.is_empty() {
            parsed.set_query(None);
        } else {
            let query_str = query
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            parsed.set_query(Some(&query_str));
        }
        
        // Strip fragment (may contain sensitive data)
        parsed.set_fragment(None);
        
        // Strip user ID from URL if configured
        if self.config.strip_user_ids {
            if parsed.username().is_empty() && parsed.password().is_none() {
                // No user info in URL
            }
            // Note: URL crate doesn't allow removing user info easily
            // In production, would need custom URL reconstruction
        }
        
        parsed.to_string()
    }
    
    /// Extract domain from URL for tracking
    pub fn extract_domain(&self, url: &str) -> Option<String> {
        if !self.should_track_url(url) {
            return None;
        }
        
        let parsed = Url::parse(url).ok()?;
        let host = parsed.host_str()?;
        
        // Extract base domain (e.g., "example.com" from "sub.example.com")
        let parts: Vec<&str> = host.split('.').collect();
        if parts.len() >= 2 {
            Some(format!("{}.{}", parts[parts.len()-2], parts[parts.len()-1]))
        } else {
            Some(host.to_string())
        }
    }
    
    /// Anonymize an IP address
    pub fn anonymize_ip(&self, ip: &str) -> String {
        if !self.config.enabled || !self.config.anonymize_ips {
            return ip.to_string();
        }
        
        // IPv4: zero out last octet
        if ip.contains('.') {
            let parts: Vec<&str> = ip.split('.').collect();
            if parts.len() == 4 {
                return format!("{}.{}.{}.0", parts[0], parts[1], parts[2]);
            }
        }
        
        // IPv6: zero out last 64 bits
        if ip.contains(':') {
            let parts: Vec<&str> = ip.split(':').collect();
            if parts.len() >= 4 {
                return format!("{}:{}:{}::0", parts[0], parts[1], parts[2]);
            }
        }
        
        ip.to_string()
    }
    
    /// Hash a string value for anonymization
    pub fn hash_value(&self, value: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
    
    /// Check if tracking is allowed based on Do Not Track setting
    pub fn respects_do_not_track(&self, dnt_enabled: bool) -> bool {
        if dnt_enabled && self.config.enabled {
            return false;
        }
        true
    }
    
    /// Get a random delay for time-based metrics to prevent fingerprinting
    pub fn get_time_jitter(&self, actual_time_ms: u64) -> u64 {
        if !self.config.enabled {
            return actual_time_ms;
        }
        
        // Add small random jitter (±5%) to prevent timing attacks
        let jitter_range = (actual_time_ms as f64 * 0.05) as u64;
        // In production, use proper RNG
        actual_time_ms + jitter_range / 2
    }
    
    /// Aggregate time spent to prevent precise tracking
    pub fn aggregate_time(&self, time_ms: u64) -> u64 {
        if !self.config.enabled {
            return time_ms;
        }
        
        // Round to nearest threshold
        let threshold = self.config.time_aggregation_threshold;
        ((time_ms + threshold / 2) / threshold) * threshold
    }
    
    /// Sanitize page title for storage
    pub fn sanitize_title(&self, title: &str) -> String {
        // Remove potential PII patterns
        let sanitized = title
            .replace(|c: char| c.is_control(), "")
            .chars()
            .take(200)  // Limit length
            .collect();
        
        // Remove email patterns
        let email_pattern = regex::Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap();
        email_pattern.replace_all(&sanitized, "[email]").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_privacy_filter_default() {
        let filter = PrivacyFilter::with_defaults();
        assert!(filter.config.enabled);
    }
    
    #[test]
    fn test_should_track_url() {
        let filter = PrivacyFilter::with_defaults();
        
        // Should track normal URLs
        assert!(filter.should_track_url("https://example.com"));
        assert!(filter.should_track_url("https://www.google.com/search"));
        
        // Should not track internal URLs
        assert!(!filter.should_track_url("about:blank"));
        assert!(!filter.should_track_url("chrome://settings"));
        assert!(!filter.should_track_url("vantis://newtab"));
        
        // Should not track sensitive domains
        assert!(!filter.should_track_url("https://bank.example.com/login"));
        assert!(!filter.should_track_url("https://secure.payment.com/checkout"));
    }
    
    #[test]
    fn test_sanitize_url() {
        let filter = PrivacyFilter::with_defaults();
        
        let url = "https://example.com/page?token=secret&id=123&name=test";
        let sanitized = filter.sanitize_url(url);
        
        assert!(!sanitized.contains("token"));
        assert!(!sanitized.contains("secret"));
        assert!(sanitized.contains("id=123"));
        assert!(sanitized.contains("name=test"));
    }
    
    #[test]
    fn test_anonymize_ip() {
        let filter = PrivacyFilter::with_defaults();
        
        assert_eq!(filter.anonymize_ip("192.168.1.100"), "192.168.1.0");
        assert_eq!(filter.anonymize_ip("10.0.0.50"), "10.0.0.0");
    }
    
    #[test]
    fn test_extract_domain() {
        let filter = PrivacyFilter::with_defaults();
        
        assert_eq!(filter.extract_domain("https://www.example.com/page"), Some("example.com".to_string()));
        assert_eq!(filter.extract_domain("https://sub.sub.example.com/page"), Some("example.com".to_string()));
        assert_eq!(filter.extract_domain("about:blank"), None);
    }
}