//! Unit tests for the analytics module

#[cfg(test)]
mod tests {
    use crate::analytics::*;

    #[test]
    fn test_analytics_config_default() {
        let config = AnalyticsConfig::default();
        assert!(config.enabled);
        assert!(config.track_browsing);
        assert!(config.track_performance);
        assert!(config.track_security);
        assert_eq!(config.retention_days, 90);
    }

    #[test]
    fn test_privacy_filter_creation() {
        let filter = privacy::PrivacyFilter::with_defaults();
        assert!(filter.should_track_url("https://example.com"));
        assert!(!filter.should_track_url("about:blank"));
        assert!(!filter.should_track_url("chrome://settings"));
    }

    #[test]
    fn test_privacy_filter_sensitive_urls() {
        let filter = privacy::PrivacyFilter::with_defaults();
        
        // Should block banking URLs
        assert!(!filter.should_track_url("https://bank.example.com/login"));
        assert!(!filter.should_track_url("https://secure.payment.com/checkout"));
        
        // Should allow normal URLs
        assert!(filter.should_track_url("https://example.com/page"));
    }

    #[test]
    fn test_privacy_filter_url_sanitization() {
        let filter = privacy::PrivacyFilter::with_defaults();
        
        let url = "https://example.com/page?token=secret&id=123&name=test";
        let sanitized = filter.sanitize_url(url);
        
        // Should remove sensitive params
        assert!(!sanitized.contains("token"));
        assert!(!sanitized.contains("secret"));
        
        // Should keep non-sensitive params
        assert!(sanitized.contains("id=123"));
        assert!(sanitized.contains("name=test"));
    }

    #[test]
    fn test_privacy_filter_domain_extraction() {
        let filter = privacy::PrivacyFilter::with_defaults();
        
        assert_eq!(
            filter.extract_domain("https://www.example.com/page"),
            Some("example.com".to_string())
        );
        
        assert_eq!(
            filter.extract_domain("https://sub.sub.example.com/page"),
            Some("example.com".to_string())
        );
        
        // Internal URLs should not be tracked
        assert_eq!(filter.extract_domain("about:blank"), None);
    }

    #[test]
    fn test_usage_metrics_creation() {
        let metrics = metrics::UsageMetrics::new();
        assert_eq!(metrics.total_visits, 0);
        assert_eq!(metrics.total_time_ms, 0);
        assert_eq!(metrics.unique_domains, 0);
    }

    #[test]
    fn test_performance_metrics_creation() {
        let metrics = metrics::PerformanceMetrics::new();
        assert_eq!(metrics.average_load_time_ms, 0);
        assert_eq!(metrics.slow_pages, 0);
    }

    #[test]
    fn test_security_metrics_creation() {
        let metrics = metrics::SecurityMetrics::new();
        assert_eq!(metrics.threats_blocked, 0);
        assert_eq!(metrics.https_upgrades, 0);
        assert_eq!(metrics.trackers_blocked, 0);
    }

    #[test]
    fn test_analytics_snapshot_creation() {
        let snapshot = metrics::AnalyticsSnapshot::new();
        assert_eq!(snapshot.usage.total_visits, 0);
        assert_eq!(snapshot.performance.average_load_time_ms, 0);
        assert_eq!(snapshot.security.threats_blocked, 0);
    }

    #[test]
    fn test_page_load_metrics() {
        let metrics = PageLoadMetrics::new("https://example.com".to_string());
        assert!(!metrics.is_slow());
        
        let mut slow_metrics = metrics.clone();
        slow_metrics.load_complete_ms = 5000;
        assert!(slow_metrics.is_slow());
    }

    #[test]
    fn test_security_event_severity() {
        let phishing = SecurityEvent::PhishingAttemptBlocked { 
            url: "https://evil.com".to_string() 
        };
        assert_eq!(phishing.severity(), SecuritySeverity::Critical);
        
        let cookie = SecurityEvent::CookieBlocked { 
            domain: "tracker.com".to_string() 
        };
        assert_eq!(cookie.severity(), SecuritySeverity::Low);
        
        let tracker = SecurityEvent::TrackingAttempt { 
            tracker: "google-analytics".to_string(),
            url: "https://example.com".to_string()
        };
        assert_eq!(tracker.severity(), SecuritySeverity::Medium);
    }

    #[test]
    fn test_security_event_description() {
        let event = SecurityEvent::HttpsUpgrade { 
            url: "http://example.com".to_string() 
        };
        assert!(event.description().contains("HTTPS"));
    }

    #[test]
    fn test_export_format() {
        let exporter = exporter::AnalyticsExporter::with_defaults();
        assert!(exporter.output_path.is_none());
    }

    #[test]
    fn test_time_range_presets() {
        let range = exporter::TimeRange::last_days(7);
        let duration = range.end - range.start;
        assert_eq!(duration.num_days(), 7);
        
        let today = exporter::TimeRange::today();
        assert!(today.start < today.end);
    }

    #[test]
    fn test_dashboard_config_default() {
        let config = dashboard::DashboardConfig::default();
        assert_eq!(config.top_domains_count, 10);
        assert!(config.show_performance);
        assert!(config.show_security);
        assert!(config.show_insights);
    }

    #[test]
    fn test_dashboard_builder() {
        let config = dashboard::DashboardConfig::default();
        let builder = dashboard::DashboardBuilder::new(config);
        let snapshot = metrics::AnalyticsSnapshot::new();
        let dashboard = builder.build(&snapshot);
        
        assert!(dashboard.summary.total_visits >= 0);
        assert!(dashboard.top_domains.is_empty());
    }

    #[test]
    fn test_storage_creation() {
        let storage = storage::AnalyticsStorage::new();
        assert!(storage.connection.is_none());
    }

    #[test]
    fn test_storage_with_path() {
        let storage = storage::AnalyticsStorage::with_path("/tmp/test.db");
        assert!(storage.connection.is_none());
    }
}