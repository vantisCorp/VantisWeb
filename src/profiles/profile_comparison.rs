use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use crate::profiles::ProfileConfig;

/// Profile comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileComparison {
    /// First profile ID
    pub profile_a_id: String,
    /// Second profile ID
    pub profile_b_id: String,
    /// Comparison timestamp
    pub timestamp: i64,
    /// Settings comparison
    pub settings_diff: SettingsDiff,
    /// Bookmarks comparison
    pub bookmarks_diff: BookmarksDiff,
    /// Extensions comparison
    pub extensions_diff: ExtensionsDiff,
    /// Security comparison
    pub security_diff: SecurityDiff,
    /// History comparison
    pub history_diff: HistoryDiff,
    /// Overall similarity score (0-100)
    pub similarity_score: f64,
}

/// Settings difference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsDiff {
    /// Settings only in profile A
    pub only_in_a: HashMap<String, serde_json::Value>,
    /// Settings only in profile B
    pub only_in_b: HashMap<String, serde_json::Value>,
    /// Settings with different values
    pub different: Vec<SettingDifference>,
    /// Settings with same values
    pub same: HashMap<String, serde_json::Value>,
}

/// Single setting difference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingDifference {
    pub key: String,
    pub value_a: serde_json::Value,
    pub value_b: serde_json::Value,
}

/// Bookmarks difference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarksDiff {
    /// Bookmarks only in profile A
    pub only_in_a: Vec<String>,
    /// Bookmarks only in profile B
    pub only_in_b: Vec<String>,
    /// Shared bookmarks
    pub shared: Vec<String>,
    /// Similar bookmarks (same domain)
    pub similar: Vec<SimilarBookmark>,
}

/// Similar bookmark (same domain, different path)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarBookmark {
    pub bookmark_a: String,
    pub bookmark_b: String,
    pub domain: String,
}

/// Extensions difference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionsDiff {
    /// Extensions only in profile A
    pub only_in_a: Vec<String>,
    /// Extensions only in profile B
    pub only_in_b: Vec<String>,
    /// Shared extensions
    pub shared: Vec<String>,
    /// Extension settings differences
    pub settings_diff: HashMap<String, SettingsDiff>,
}

/// Security difference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityDiff {
    /// Security level difference
    pub level_diff: Option<SecurityLevelDiff>,
    /// Authentication methods difference
    pub auth_diff: AuthDiff,
    /// Privacy settings difference
    pub privacy_diff: SettingsDiff,
}

/// Security level difference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityLevelDiff {
    pub level_a: String,
    pub level_b: String,
}

/// Authentication difference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthDiff {
    pub only_in_a: Vec<String>,
    pub only_in_b: Vec<String>,
    pub shared: Vec<String>,
}

/// History difference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryDiff {
    /// History entries only in profile A
    pub only_in_a: Vec<String>,
    /// History entries only in profile B
    pub only_in_b: Vec<String>,
    /// Shared history entries
    pub shared: Vec<String>,
    /// Common domains visited
    pub common_domains: Vec<DomainCount>,
}

/// Domain visit count
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainCount {
    pub domain: String,
    pub count_a: u32,
    pub count_b: u32,
}

/// Comparison options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonOptions {
    /// Include settings comparison
    pub include_settings: bool,
    /// Include bookmarks comparison
    pub include_bookmarks: bool,
    /// Include extensions comparison
    pub include_extensions: bool,
    /// Include security comparison
    pub include_security: bool,
    /// Include history comparison
    pub include_history: bool,
    /// Maximum history items to compare
    pub max_history_items: usize,
}

impl Default for ComparisonOptions {
    fn default() -> Self {
        Self {
            include_settings: true,
            include_bookmarks: true,
            include_extensions: true,
            include_security: true,
            include_history: true,
            max_history_items: 100,
        }
    }
}

/// Merge operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeOperation {
    /// Source profile ID
    pub source_id: String,
    /// Target profile ID
    pub target_id: String,
    /// Items to merge
    pub items: Vec<MergeItem>,
}

/// Merge item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeItem {
    /// Item type
    pub item_type: MergeItemType,
    /// Item key/identifier
    pub key: String,
    /// Merge strategy
    pub strategy: MergeStrategy,
}

/// Merge item type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MergeItemType {
    Setting,
    Bookmark,
    Extension,
    SecuritySetting,
}

/// Merge strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MergeStrategy {
    /// Overwrite target with source
    Overwrite,
    /// Keep both (for lists)
    Append,
    /// Keep target value
    KeepTarget,
    /// Create new with custom value
    Custom(serde_json::Value),
}

/// Comparison report for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonReport {
    /// Report title
    pub title: String,
    /// Comparison timestamp
    pub timestamp: i64,
    /// Profile A name
    pub profile_a_name: String,
    /// Profile B name
    pub profile_b_name: String,
    /// Similarity score
    pub similarity_score: f64,
    /// Summary sections
    pub summary: Vec<ReportSection>,
    /// Detailed differences
    pub details: Vec<ReportDetail>,
}

/// Report section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSection {
    pub title: String,
    pub content: String,
}

/// Report detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportDetail {
    pub category: String,
    pub items: Vec<String>,
}

/// Profile comparison manager
pub struct ProfileComparisonManager;

impl ProfileComparisonManager {
    /// Compare two profiles
    pub fn compare(profile_a: &ProfileConfig, profile_b: &ProfileConfig, options: &ComparisonOptions) 
        -> ProfileComparison 
    {
        let settings_diff = if options.include_settings {
            Self::compare_settings(&profile_a.settings, &profile_b.settings)
        } else {
            SettingsDiff {
                only_in_a: HashMap::new(),
                only_in_b: HashMap::new(),
                different: vec![],
                same: HashMap::new(),
            }
        };

        let bookmarks_diff = if options.include_bookmarks {
            Self::compare_bookmarks(&profile_a.bookmarks, &profile_b.bookmarks)
        } else {
            BookmarksDiff {
                only_in_a: vec![],
                only_in_b: vec![],
                shared: vec![],
                similar: vec![],
            }
        };

        let extensions_diff = if options.include_extensions {
            Self::compare_extensions(&profile_a.extensions, &profile_b.extensions)
        } else {
            ExtensionsDiff {
                only_in_a: vec![],
                only_in_b: vec![],
                shared: vec![],
                settings_diff: HashMap::new(),
            }
        };

        let history_diff = if options.include_history {
            let history_a: Vec<String> = profile_a.history.iter()
                .take(options.max_history_items)
                .cloned()
                .collect();
            let history_b: Vec<String> = profile_b.history.iter()
                .take(options.max_history_items)
                .cloned()
                .collect();
            Self::compare_history(&history_a, &history_b)
        } else {
            HistoryDiff {
                only_in_a: vec![],
                only_in_b: vec![],
                shared: vec![],
                common_domains: vec![],
            }
        };

        let security_diff = if options.include_security {
            Self::compare_security(profile_a, profile_b)
        } else {
            SecurityDiff {
                level_diff: None,
                auth_diff: AuthDiff {
                    only_in_a: vec![],
                    only_in_b: vec![],
                    shared: vec![],
                },
                privacy_diff: SettingsDiff {
                    only_in_a: HashMap::new(),
                    only_in_b: HashMap::new(),
                    different: vec![],
                    same: HashMap::new(),
                },
            }
        };

        let similarity_score = Self::calculate_similarity(
            &settings_diff,
            &bookmarks_diff,
            &extensions_diff,
            &history_diff,
        );

        ProfileComparison {
            profile_a_id: profile_a.id.clone(),
            profile_b_id: profile_b.id.clone(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            settings_diff,
            bookmarks_diff,
            extensions_diff,
            security_diff,
            history_diff,
            similarity_score,
        }
    }

    /// Compare settings
    fn compare_settings(settings_a: &HashMap<String, serde_json::Value>, settings_b: &HashMap<String, serde_json::Value>) 
        -> SettingsDiff 
    {
        let keys_a: HashSet<&String> = settings_a.keys().collect();
        let keys_b: HashSet<&String> = settings_b.keys().collect();

        let only_in_a_keys: HashSet<&String> = keys_a.difference(&keys_b).cloned().collect();
        let only_in_b_keys: HashSet<&String> = keys_b.difference(&keys_a).cloned().collect();
        let common_keys: HashSet<&String> = keys_a.intersection(&keys_b).cloned().collect();

        let mut only_in_a = HashMap::new();
        for key in only_in_a_keys {
            only_in_a.insert(key.clone(), settings_a[key].clone());
        }

        let mut only_in_b = HashMap::new();
        for key in only_in_b_keys {
            only_in_b.insert(key.clone(), settings_b[key].clone());
        }

        let mut different = vec![];
        let mut same = HashMap::new();

        for key in common_keys {
            if settings_a[key] != settings_b[key] {
                different.push(SettingDifference {
                    key: key.clone(),
                    value_a: settings_a[key].clone(),
                    value_b: settings_b[key].clone(),
                });
            } else {
                same.insert(key.clone(), settings_a[key].clone());
            }
        }

        SettingsDiff {
            only_in_a,
            only_in_b,
            different,
            same,
        }
    }

    /// Compare bookmarks
    fn compare_bookmarks(bookmarks_a: &[String], bookmarks_b: &[String]) -> BookmarksDiff {
        let set_a: HashSet<&String> = bookmarks_a.iter().collect();
        let set_b: HashSet<&String> = bookmarks_b.iter().collect();

        let only_in_a: Vec<String> = set_a.difference(&set_b).map(|s| (*s).clone()).collect();
        let only_in_b: Vec<String> = set_b.difference(&set_a).map(|s| (*s).clone()).collect();
        let shared: Vec<String> = set_a.intersection(&set_b).map(|s| (*s).clone()).collect();

        // Find similar bookmarks (same domain)
        let mut similar = vec![];
        let domains_a: HashMap<String, &String> = bookmarks_a.iter()
            .filter_map(|b| {
                if let Ok(url) = url::Url::parse(b) {
                    if let Some(domain) = url.host_str() {
                        return Some((domain.to_string(), b));
                    }
                }
                None
            })
            .collect();

        for bookmark_b in bookmarks_b {
            if let Ok(url) = url::Url::parse(bookmark_b) {
                if let Some(domain) = url.host_str() {
                    if let Some(bookmark_a) = domains_a.get(domain) {
                        if bookmark_a != &bookmark_b {
                            similar.push(SimilarBookmark {
                                bookmark_a: (*bookmark_a).clone(),
                                bookmark_b: bookmark_b.clone(),
                                domain: domain.to_string(),
                            });
                        }
                    }
                }
            }
        }

        BookmarksDiff {
            only_in_a,
            only_in_b,
            shared,
            similar,
        }
    }

    /// Compare extensions
    fn compare_extensions(extensions_a: &[String], extensions_b: &[String]) -> ExtensionsDiff {
        let set_a: HashSet<&String> = extensions_a.iter().collect();
        let set_b: HashSet<&String> = extensions_b.iter().collect();

        let only_in_a: Vec<String> = set_a.difference(&set_b).map(|s| (*s).clone()).collect();
        let only_in_b: Vec<String> = set_b.difference(&set_a).map(|s| (*s).clone()).collect();
        let shared: Vec<String> = set_a.intersection(&set_b).map(|s| (*s).clone()).collect();

        ExtensionsDiff {
            only_in_a,
            only_in_b,
            shared,
            settings_diff: HashMap::new(), // Would need extension-specific settings
        }
    }

    /// Compare history
    fn compare_history(history_a: &[String], history_b: &[String]) -> HistoryDiff {
        let set_a: HashSet<&String> = history_a.iter().collect();
        let set_b: HashSet<&String> = history_b.iter().collect();

        let only_in_a: Vec<String> = set_a.difference(&set_b).map(|s| (*s).clone()).collect();
        let only_in_b: Vec<String> = set_b.difference(&set_a).map(|s| (*s).clone()).collect();
        let shared: Vec<String> = set_a.intersection(&set_b).map(|s| (*s).clone()).collect();

        // Calculate common domains
        let mut domain_counts_a: HashMap<String, u32> = HashMap::new();
        let mut domain_counts_b: HashMap<String, u32> = HashMap::new();

        for url in history_a {
            if let Ok(parsed) = url::Url::parse(url) {
                if let Some(domain) = parsed.host_str() {
                    *domain_counts_a.entry(domain.to_string()).or_insert(0) += 1;
                }
            }
        }

        for url in history_b {
            if let Ok(parsed) = url::Url::parse(url) {
                if let Some(domain) = parsed.host_str() {
                    *domain_counts_b.entry(domain.to_string()).or_insert(0) += 1;
                }
            }
        }

        let mut common_domains: Vec<DomainCount> = domain_counts_a.iter()
            .filter_map(|(domain, count_a)| {
                domain_counts_b.get(domain).map(|count_b| DomainCount {
                    domain: domain.clone(),
                    count_a: *count_a,
                    count_b: *count_b,
                })
            })
            .collect();

        common_domains.sort_by(|a, b| (b.count_a + b.count_b).cmp(&(a.count_a + a.count_b)));

        HistoryDiff {
            only_in_a,
            only_in_b,
            shared,
            common_domains,
        }
    }

    /// Compare security settings
    fn compare_security(profile_a: &ProfileConfig, profile_b: &ProfileConfig) -> SecurityDiff {
        // Extract security-related settings
        let security_keys = ["security_level", "auth_method", "encryption", "tracking_protection"];
        
        let sec_settings_a: HashMap<String, serde_json::Value> = profile_a.settings.iter()
            .filter(|(k, _)| security_keys.contains(&k.as_str()))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let sec_settings_b: HashMap<String, serde_json::Value> = profile_b.settings.iter()
            .filter(|(k, _)| security_keys.contains(&k.as_str()))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let privacy_diff = Self::compare_settings(&sec_settings_a, &sec_settings_b);

        SecurityDiff {
            level_diff: None, // Would need actual security level data
            auth_diff: AuthDiff {
                only_in_a: vec![],
                only_in_b: vec![],
                shared: vec![],
            },
            privacy_diff,
        }
    }

    /// Calculate similarity score
    fn calculate_similarity(
        settings_diff: &SettingsDiff,
        bookmarks_diff: &BookmarksDiff,
        extensions_diff: &ExtensionsDiff,
        history_diff: &HistoryDiff,
    ) -> f64 {
        let mut total_items = 0;
        let mut matching_items = 0;

        // Settings similarity
        let settings_total = settings_diff.only_in_a.len() 
            + settings_diff.only_in_b.len() 
            + settings_diff.different.len() 
            + settings_diff.same.len();
        if settings_total > 0 {
            total_items += settings_total;
            matching_items += settings_diff.same.len();
        }

        // Bookmarks similarity
        let bookmarks_total = bookmarks_diff.only_in_a.len() 
            + bookmarks_diff.only_in_b.len() 
            + bookmarks_diff.shared.len();
        if bookmarks_total > 0 {
            total_items += bookmarks_total;
            matching_items += bookmarks_diff.shared.len();
        }

        // Extensions similarity
        let extensions_total = extensions_diff.only_in_a.len() 
            + extensions_diff.only_in_b.len() 
            + extensions_diff.shared.len();
        if extensions_total > 0 {
            total_items += extensions_total;
            matching_items += extensions_diff.shared.len();
        }

        // History similarity
        let history_total = history_diff.only_in_a.len() 
            + history_diff.only_in_b.len() 
            + history_diff.shared.len();
        if history_total > 0 {
            total_items += history_total;
            matching_items += history_diff.shared.len();
        }

        if total_items == 0 {
            100.0
        } else {
            (matching_items as f64 / total_items as f64) * 100.0
        }
    }

    /// Generate comparison report
    pub fn generate_report(profile_a: &ProfileConfig, profile_b: &ProfileConfig, comparison: &ProfileComparison) 
        -> ComparisonReport 
    {
        let mut summary = vec![];
        let mut details = vec![];

        // Overall summary
        summary.push(ReportSection {
            title: "Similarity Score".to_string(),
            content: format!("{:.1}% similar", comparison.similarity_score),
        });

        // Settings summary
        if !comparison.settings_diff.different.is_empty() {
            summary.push(ReportSection {
                title: "Settings Differences".to_string(),
                content: format!("{} settings differ between profiles", comparison.settings_diff.different.len()),
            });
        }

        // Bookmarks summary
        summary.push(ReportSection {
            title: "Bookmarks".to_string(),
            content: format!(
                "{} shared, {} unique to '{}', {} unique to '{}'",
                comparison.bookmarks_diff.shared.len(),
                comparison.bookmarks_diff.only_in_a.len(),
                profile_a.name,
                comparison.bookmarks_diff.only_in_b.len(),
                profile_b.name,
            ),
        });

        // Extensions summary
        summary.push(ReportSection {
            title: "Extensions".to_string(),
            content: format!(
                "{} shared extensions, {} unique extensions",
                comparison.extensions_diff.shared.len(),
                comparison.extensions_diff.only_in_a.len() + comparison.extensions_diff.only_in_b.len(),
            ),
        });

        // Details
        if !comparison.settings_diff.different.is_empty() {
            details.push(ReportDetail {
                category: "Settings Differences".to_string(),
                items: comparison.settings_diff.different.iter()
                    .map(|d| format!("{}: '{}' vs '{}'", d.key, d.value_a, d.value_b))
                    .collect(),
            });
        }

        if !comparison.bookmarks_diff.only_in_a.is_empty() {
            details.push(ReportDetail {
                category: format!("Bookmarks only in '{}'", profile_a.name),
                items: comparison.bookmarks_diff.only_in_a.clone(),
            });
        }

        if !comparison.bookmarks_diff.only_in_b.is_empty() {
            details.push(ReportDetail {
                category: format!("Bookmarks only in '{}'", profile_b.name),
                items: comparison.bookmarks_diff.only_in_b.clone(),
            });
        }

        ComparisonReport {
            title: format!("Profile Comparison: {} vs {}", profile_a.name, profile_b.name),
            timestamp: comparison.timestamp,
            profile_a_name: profile_a.name.clone(),
            profile_b_name: profile_b.name.clone(),
            similarity_score: comparison.similarity_score,
            summary,
            details,
        }
    }

    /// Export comparison report to JSON
    pub fn export_report_json(report: &ComparisonReport) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(report)
    }

    /// Export comparison report to HTML
    pub fn export_report_html(report: &ComparisonReport) -> String {
        format!(
            r##"<!DOCTYPE html>
<html>
<head>
    <title>{title}</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #1a1a1a; color: #fff; padding: 2rem; }}
        .container {{ max-width: 1200px; margin: 0 auto; }}
        h1 {{ color: #dc2626; }}
        h2 {{ border-bottom: 1px solid #333; padding-bottom: 0.5rem; margin-top: 2rem; }}
        .summary {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 1rem; margin: 1rem 0; }}
        .summary-card {{ background: #2a2a2a; padding: 1rem; border-radius: 8px; }}
        .summary-card h3 {{ margin: 0 0 0.5rem 0; color: #888; font-size: 0.9rem; }}
        .summary-card p {{ margin: 0; font-size: 1.2rem; }}
        .detail {{ background: #2a2a2a; padding: 1rem; border-radius: 8px; margin: 1rem 0; }}
        .detail h3 {{ margin: 0 0 1rem 0; }}
        .detail ul {{ margin: 0; padding-left: 1.5rem; }}
        .detail li {{ margin: 0.5rem 0; color: #aaa; }}
        .score {{ font-size: 3rem; font-weight: bold; color: #dc2626; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>{title}</h1>
        <p>Generated on {timestamp}</p>
        
        <div class="summary">
            <div class="summary-card">
                <h3>Similarity Score</h3>
                <p class="score">{similarity:.1}%</p>
            </div>
            <div class="summary-card">
                <h3>Profile A</h3>
                <p>{profile_a}</p>
            </div>
            <div class="summary-card">
                <h3>Profile B</h3>
                <p>{profile_b}</p>
            </div>
        </div>

        <h2>Summary</h2>
        <div class="summary">
            {summary_cards}
        </div>

        <h2>Details</h2>
        {detail_sections}
    </div>
</body>
</html>"##,
            title = report.title,
            timestamp = chrono::DateTime::from_timestamp_millis(report.timestamp)
                .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
            similarity = report.similarity_score,
            profile_a = report.profile_a_name,
            profile_b = report.profile_b_name,
            summary_cards = report.summary.iter().map(|s| format!(
                r##"<div class="summary-card"><h3>{}</h3><p>{}</p></div>"##,
                s.title, s.content
            )).collect::<Vec<_>>().join("\n"),
            detail_sections = report.details.iter().map(|d| format!(
                r##"<div class="detail"><h3>{}</h3><ul>{}</ul></div>"##,
                d.category,
                d.items.iter().map(|i| format!("<li>{}</li>", i)).collect::<Vec<_>>().join("\n")
            )).collect::<Vec<_>>().join("\n"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_profile(name: &str, id: &str) -> ProfileConfig {
        ProfileConfig {
            id: id.to_string(),
            name: name.to_string(),
            profile_type: crate::profiles::ProfileType::Custom("Test".to_string()),
            icon: None,
            color: None,
            active: false,
            order: 0,
            settings: HashMap::new(),
            bookmarks: vec!["https://example.com".to_string()],
            history: vec![],
            extensions: vec![],
            theme: None,
            created_at: 0,
            last_used_at: 0,
        }
    }

    #[test]
    fn test_compare_profiles() {
        let profile_a = create_test_profile("Profile A", "a");
        let profile_b = create_test_profile("Profile B", "b");

        let comparison = ProfileComparisonManager::compare(
            &profile_a,
            &profile_b,
            &ComparisonOptions::default(),
        );

        assert_eq!(comparison.profile_a_id, "a");
        assert_eq!(comparison.profile_b_id, "b");
    }

    #[test]
    fn test_similarity_calculation() {
        let settings_diff = SettingsDiff {
            only_in_a: HashMap::new(),
            only_in_b: HashMap::new(),
            different: vec![],
            same: HashMap::new(),
        };

        let bookmarks_diff = BookmarksDiff {
            only_in_a: vec![],
            only_in_b: vec![],
            shared: vec!["a".to_string(), "b".to_string()],
            similar: vec![],
        };

        let extensions_diff = ExtensionsDiff {
            only_in_a: vec![],
            only_in_b: vec![],
            shared: vec!["ext1".to_string()],
            settings_diff: HashMap::new(),
        };

        let history_diff = HistoryDiff {
            only_in_a: vec![],
            only_in_b: vec![],
            shared: vec![],
            common_domains: vec![],
        };

        let score = ProfileComparisonManager::calculate_similarity(
            &settings_diff,
            &bookmarks_diff,
            &extensions_diff,
            &history_diff,
        );

        assert_eq!(score, 100.0);
    }

    #[test]
    fn test_generate_report() {
        let profile_a = create_test_profile("Profile A", "a");
        let profile_b = create_test_profile("Profile B", "b");

        let comparison = ProfileComparisonManager::compare(
            &profile_a,
            &profile_b,
            &ComparisonOptions::default(),
        );

        let report = ProfileComparisonManager::generate_report(&profile_a, &profile_b, &comparison);

        assert!(report.title.contains("Profile A"));
        assert!(report.title.contains("Profile B"));
    }
}