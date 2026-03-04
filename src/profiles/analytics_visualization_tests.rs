//! Integration tests for analytics visualization module

#[cfg(test)]
mod integration_tests {
    use super::super::analytics_visualization::*;
    use super::super::analytics::*;
    use chrono::Utc;

    fn create_test_analytics() -> ProfileAnalytics {
        let mut daily_usage = HashMap::new();
        daily_usage.insert("2024-01-01".to_string(), DailyUsage {
            date: "2024-01-01".to_string(),
            time_spent: 3600, // 1 hour
            sessions: 2,
            websites_visited: 10,
        });
        daily_usage.insert("2024-01-02".to_string(), DailyUsage {
            date: "2024-01-02".to_string(),
            time_spent: 7200, // 2 hours
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
            url: "https://youtube.com".to_string(),
            visits: 10,
            time_spent: 2400,
            last_visit: Utc::now(),
        });
        top_websites.push(WebsiteUsage {
            url: "https://twitter.com".to_string(),
            visits: 15,
            time_spent: 900,
            last_visit: Utc::now(),
        });

        ProfileAnalytics {
            profile_id: "test-profile".to_string(),
            total_time: 10800,
            session_count: 5,
            first_session: Some(Utc::now()),
            last_session: Some(Utc::now()),
            daily_usage,
            top_websites,
            tab_stats: TabStatistics {
                total_opened: 50,
                average_per_session: 10.0,
                max_open: 15,
                current_open: 3,
            },
            performance: PerformanceMetrics {
                avg_page_load_time: 150,
                total_crashes: 0,
                memory_usage: 500,
                cpu_usage: 25.0,
            },
        }
    }

    #[test]
    fn test_complete_visualization_workflow() {
        let manager = VisualizationManager::new();
        let analytics = create_test_analytics();

        // Test usage trends
        let trends = manager.generate_usage_trends(&analytics, &DateRange::last_days(7));
        assert!(!trends.is_empty());

        // Test category distribution
        let categories = manager.generate_category_distribution(&analytics);
        assert!(!categories.is_empty());
        assert!(categories.iter().any(|c| c.name == "Development"));
        assert!(categories.iter().any(|c| c.name == "Entertainment"));
        assert!(categories.iter().any(|c| c.name == "Social Media"));

        // Test heatmap
        let heatmap = manager.generate_heatmap(&analytics);
        assert_eq!(heatmap.len(), 168); // 24 hours * 7 days
        assert!(heatmap.iter().any(|h| h.value > 0));

        // Test top categories
        let top_categories = manager.generate_top_categories(&analytics, 3);
        assert!(top_categories.len() <= 3);
    }

    #[test]
    fn test_profile_comparison() {
        let manager = VisualizationManager::new();
        let analytics1 = create_test_analytics();
        
        let mut analytics2 = create_test_analytics();
        analytics2.total_time = 21600; // 2x the time
        analytics2.session_count = 10;
        analytics2.profile_id = "test-profile-2".to_string();

        let comparison = manager.generate_comparison(&analytics1, &analytics2);
        
        assert_eq!(comparison.primary_profile, "test-profile");
        assert_eq!(comparison.secondary_profile, "test-profile-2");
        assert_eq!(comparison.metrics.total_time_diff, -10800);
        assert_eq!(comparison.metrics.session_count_diff, -5);
    }

    #[test]
    fn test_report_generation_and_export() {
        let manager = VisualizationManager::new();
        let analytics = create_test_analytics();
        let date_range = DateRange::last_days(7);

        // Generate report
        let report = manager.generate_report(&analytics, "Test Profile", date_range);
        
        assert_eq!(report.profile_name, "Test Profile");
        assert_eq!(report.profile_id, "test-profile");
        assert!(!report.top_websites.is_empty());
        assert!(!report.categories.is_empty());
        assert!(!report.trends.is_empty());
        assert_eq!(report.heatmap.len(), 168);

        // Test JSON export
        let json_result = manager.export_report(&report, ExportFormat::Json);
        assert!(json_result.is_ok());
        let json_data = json_result.unwrap();
        assert!(!json_data.is_empty());

        // Test CSV export
        let csv_result = manager.export_report(&report, ExportFormat::Csv);
        assert!(csv_result.is_ok());
        let csv_data = String::from_utf8(csv_result.unwrap()).unwrap();
        assert!(csv_data.contains("SUMMARY"));
        assert!(csv_data.contains("TOP WEBSITES"));

        // Test HTML export
        let html_result = manager.export_report(&report, ExportFormat::Html);
        assert!(html_result.is_ok());
        let html_data = String::from_utf8(html_result.unwrap()).unwrap();
        assert!(html_data.contains("<html>"));
        assert!(html_data.contains("VantisWeb Analytics Report"));
        assert!(html_data.contains("Test Profile"));
    }

    #[test]
    fn test_date_range_presets() {
        let last_7_days = DateRange::last_days(7);
        assert!(last_7_days.end > last_7_days.start);
        
        let current_week = DateRange::current_week();
        assert!(current_week.end >= current_week.start);
        
        let current_month = DateRange::current_month();
        assert!(current_month.end >= current_month.start);
    }

    #[test]
    fn test_custom_category_mappings() {
        let mut manager = VisualizationManager::new();
        
        // Add custom mapping
        manager.add_category_mapping("custom-site.com".to_string(), "Custom".to_string());
        
        assert_eq!(manager.get_category("https://custom-site.com"), "Custom");
        assert_eq!(manager.get_category("https://github.com"), "Development");
    }

    #[test]
    fn test_category_percentages() {
        let manager = VisualizationManager::new();
        let analytics = create_test_analytics();
        
        let categories = manager.generate_category_distribution(&analytics);
        let total_percentage: f64 = categories.iter().map(|c| c.percentage).sum();
        
        // Allow for small floating point errors
        assert!((total_percentage - 100.0).abs() < 0.1);
    }

    #[test]
    fn test_export_formats_all_valid() {
        let manager = VisualizationManager::new();
        let analytics = create_test_analytics();
        let report = manager.generate_report(&analytics, "Test", DateRange::last_days(7));

        let formats = vec![
            ExportFormat::Json,
            ExportFormat::Csv,
            ExportFormat::Html,
            ExportFormat::Pdf,
        ];

        for format in formats {
            let result = manager.export_report(&report, format.clone());
            assert!(result.is_ok(), "Export format {:?} should succeed", format);
            let data = result.unwrap();
            assert!(!data.is_empty(), "Export format {:?} should produce data", format);
        }
    }
}
