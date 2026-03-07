//! UI Components for VantisWeb Ad Blocker
//! 
//! This module provides user interface structures and data for:
//! - Ad blocker popup UI
//! - Settings page components
//! - Statistics dashboard
//! - Rule editor interface
//! - Notification components

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Popup UI state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopupState {
    /// Whether ad blocking is enabled for current site
    pub enabled: bool,
    /// Current page URL
    pub page_url: String,
    /// Page domain
    pub domain: String,
    /// Statistics for current page
    pub page_stats: PageBlockStats,
    /// Total statistics
    pub total_stats: TotalBlockStats,
    /// Quick toggle options
    pub quick_toggles: Vec<QuickToggle>,
    /// Recent blocked items
    pub recent_blocked: Vec<BlockedItem>,
    /// Whether page is whitelisted
    pub is_whitelisted: bool,
    /// Whitelist reason if applicable
    pub whitelist_reason: Option<String>,
}

/// Page-specific blocking statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageBlockStats {
    /// Ads blocked on this page
    pub ads_blocked: usize,
    /// Trackers blocked on this page
    pub trackers_blocked: usize,
    /// Scripts blocked on this page
    pub scripts_blocked: usize,
    /// Bandwidth saved (bytes)
    pub bandwidth_saved: u64,
    /// Time saved (milliseconds)
    pub time_saved_ms: u64,
    /// Threats blocked
    pub threats_blocked: usize,
}

/// Total blocking statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotalBlockStats {
    /// Total ads blocked
    pub total_ads_blocked: u64,
    /// Total trackers blocked
    pub total_trackers_blocked: u64,
    /// Total scripts blocked
    pub total_scripts_blocked: u64,
    /// Total bandwidth saved (bytes)
    pub total_bandwidth_saved: u64,
    /// Total time saved (seconds)
    pub total_time_saved_seconds: u64,
    /// Total threats blocked
    pub total_threats_blocked: u64,
    /// Since when statistics are tracked
    pub tracking_since: DateTime<Utc>,
}

/// Quick toggle option in popup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickToggle {
    /// Toggle ID
    pub id: String,
    /// Display label
    pub label: String,
    /// Icon name
    pub icon: String,
    /// Whether enabled
    pub enabled: bool,
    /// Category
    pub category: ToggleCategory,
    /// Tooltip text
    pub tooltip: String,
}

/// Toggle categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToggleCategory {
    /// Ad blocking
    Ads,
    /// Tracker blocking
    Trackers,
    /// Social widgets
    Social,
    /// Cookie notices
    Cookies,
    /// Malware protection
    Malware,
    /// Annoyances
    Annoyances,
    /// Custom
    Custom,
}

/// A blocked item shown in UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedItem {
    /// Item ID
    pub id: String,
    /// Blocked URL
    pub url: String,
    /// Resource type
    pub resource_type: String,
    /// Why it was blocked
    pub block_reason: String,
    /// Filter rule that matched
    pub filter_rule: String,
    /// Timestamp
    pub blocked_at: DateTime<Utc>,
    /// Domain
    pub domain: String,
    /// Whether can be unblocked
    pub can_unblock: bool,
    /// Size in bytes (if known)
    pub size: Option<u64>,
}

/// Settings page state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsState {
    /// General settings
    pub general: GeneralSettings,
    /// Filter lists settings
    pub filter_lists: FilterListsSettings,
    /// Custom rules settings
    pub custom_rules: CustomRulesSettings,
    /// Privacy settings
    pub privacy: PrivacySettings,
    /// Advanced settings
    pub advanced: AdvancedSettings,
    /// UI preferences
    pub ui_preferences: UIPreferences,
}

/// General settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    /// Ad blocking enabled
    pub ad_blocking_enabled: bool,
    /// Tracker blocking enabled
    pub tracker_blocking_enabled: bool,
    /// Malware protection enabled
    pub malware_protection_enabled: bool,
    /// Show blocked count badge
    pub show_badge: bool,
    /// Show context menu
    pub show_context_menu: bool,
    /// Auto-update filter lists
    pub auto_update_filters: bool,
    /// Update interval (hours)
    pub update_interval_hours: u32,
    /// Language
    pub language: String,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            ad_blocking_enabled: true,
            tracker_blocking_enabled: true,
            malware_protection_enabled: true,
            show_badge: true,
            show_context_menu: true,
            auto_update_filters: true,
            update_interval_hours: 24,
            language: "en".to_string(),
        }
    }
}

/// Filter lists settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterListsSettings {
    /// Available filter lists
    pub available_lists: Vec<FilterListInfo>,
    /// Enabled list IDs
    pub enabled_lists: Vec<String>,
    /// Last update time
    pub last_update: Option<DateTime<Utc>>,
    /// Custom list URLs
    pub custom_lists: Vec<CustomFilterList>,
    /// Total rules count
    pub total_rules: usize,
}

/// Filter list information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterListInfo {
    /// List ID
    pub id: String,
    /// List name
    pub name: String,
    /// Description
    pub description: String,
    /// Source URL
    pub url: Option<String>,
    /// Category
    pub category: String,
    /// Language/region
    pub language: Option<String>,
    /// Number of rules
    pub rule_count: usize,
    /// Whether enabled
    pub enabled: bool,
    /// Last updated
    pub last_updated: Option<DateTime<Utc>>,
    /// Whether updating
    pub is_updating: bool,
}

/// Custom filter list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomFilterList {
    /// List URL
    pub url: String,
    /// Custom name
    pub name: String,
    /// Whether enabled
    pub enabled: bool,
    /// Last update status
    pub last_update_status: Option<String>,
}

/// Custom rules settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRulesSettings {
    /// User rules
    pub rules: Vec<UserRuleInfo>,
    /// Rule groups
    pub groups: Vec<RuleGroupInfo>,
    /// Available presets
    pub presets: Vec<PresetInfo>,
    /// Import/export enabled
    pub allow_import_export: bool,
}

/// User rule information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRuleInfo {
    /// Rule ID
    pub id: String,
    /// Rule text
    pub text: String,
    /// Rule type
    pub rule_type: String,
    /// Whether enabled
    pub enabled: bool,
    /// Description
    pub description: Option<String>,
    /// Hit count
    pub hits: usize,
    /// When created
    pub created: DateTime<Utc>,
}

/// Rule group information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleGroupInfo {
    /// Group ID
    pub id: String,
    /// Group name
    pub name: String,
    /// Number of rules
    pub rule_count: usize,
    /// Whether enabled
    pub enabled: bool,
    /// Group color
    pub color: Option<String>,
}

/// Preset information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetInfo {
    /// Preset ID
    pub id: String,
    /// Preset name
    pub name: String,
    /// Description
    pub description: String,
    /// Category
    pub category: String,
    /// Whether installed
    pub installed: bool,
}

/// Privacy settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    /// Block third-party cookies
    pub block_third_party_cookies: bool,
    /// Block fingerprinting
    pub block_fingerprinting: bool,
    /// Block crypto miners
    pub block_crypto_mining: bool,
    /// Send do-not-track header
    pub send_dnt: bool,
    /// Block social widgets
    pub block_social_widgets: bool,
    /// Hide referrer
    pub hide_referrer: bool,
    /// User agent spoofing
    pub spoof_user_agent: bool,
    /// Block WebRTC
    pub block_webrtc: bool,
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            block_third_party_cookies: true,
            block_fingerprinting: true,
            block_crypto_mining: true,
            send_dnt: true,
            block_social_widgets: false,
            hide_referrer: false,
            spoof_user_agent: false,
            block_webrtc: false,
        }
    }
}

/// Advanced settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedSettings {
    /// Enable debug logging
    pub debug_logging: bool,
    /// Use machine learning detection
    pub use_ml_detection: bool,
    /// Block strict mode
    pub strict_mode: bool,
    /// Ignore exceptions
    pub ignore_exceptions: bool,
    /// Use optimized filters
    pub optimized_filters: bool,
    /// Maximum blocked items per page
    pub max_blocked_items: usize,
    /// Cache size (MB)
    pub cache_size_mb: u32,
    /// Enable network inspection
    pub network_inspection: bool,
}

impl Default for AdvancedSettings {
    fn default() -> Self {
        Self {
            debug_logging: false,
            use_ml_detection: true,
            strict_mode: false,
            ignore_exceptions: false,
            optimized_filters: true,
            max_blocked_items: 1000,
            cache_size_mb: 50,
            network_inspection: true,
        }
    }
}

/// UI preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIPreferences {
    /// Theme (light/dark/system)
    pub theme: String,
    /// Compact mode
    pub compact_mode: bool,
    /// Show advanced options
    pub show_advanced: bool,
    /// Notification settings
    pub notifications: NotificationSettings,
    /// Animation enabled
    pub animations_enabled: bool,
}

impl Default for UIPreferences {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            compact_mode: false,
            show_advanced: false,
            notifications: NotificationSettings::default(),
            animations_enabled: true,
        }
    }
}

/// Notification settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    /// Show threat notifications
    pub show_threat_notifications: bool,
    /// Show update notifications
    pub show_update_notifications: bool,
    /// Show stats notifications
    pub show_stats_notifications: bool,
    /// Sound enabled
    pub sound_enabled: bool,
    /// Notification position
    pub position: NotificationPosition,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            show_threat_notifications: true,
            show_update_notifications: true,
            show_stats_notifications: false,
            sound_enabled: false,
            position: NotificationPosition::BottomRight,
        }
    }
}

/// Notification position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Dashboard statistics for UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    /// Date range
    pub date_range: DateRange,
    /// Daily statistics
    pub daily_stats: Vec<DailyStats>,
    /// Top blocked domains
    pub top_domains: Vec<TopDomainInfo>,
    /// Blocking by type
    pub by_type: HashMap<String, u64>,
    /// Time saved total
    pub total_time_saved: String,
    /// Bandwidth saved total
    pub total_bandwidth_saved: String,
    /// Average blocked per day
    pub avg_blocked_per_day: f64,
}

/// Date range for statistics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DateRange {
    Last24Hours,
    Last7Days,
    Last30Days,
    Last90Days,
    AllTime,
}

/// Daily statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStats {
    /// Date
    pub date: String,
    /// Total blocked
    pub blocked: u64,
    /// Ads blocked
    pub ads: u64,
    /// Trackers blocked
    pub trackers: u64,
    /// Threats blocked
    pub threats: u64,
    /// Bandwidth saved
    pub bandwidth: u64,
}

/// Top blocked domain info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopDomainInfo {
    /// Domain name
    pub domain: String,
    /// Number of blocks
    pub blocked_count: u64,
    /// Category
    pub category: String,
    /// Percentage of total
    pub percentage: f64,
}

/// Notification to display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Notification ID
    pub id: String,
    /// Notification type
    pub notification_type: NotificationType,
    /// Title
    pub title: String,
    /// Message
    pub message: String,
    /// Icon
    pub icon: String,
    /// Action button text
    pub action_text: Option<String>,
    /// Action URL
    pub action_url: Option<String>,
    /// Dismissible
    pub dismissible: bool,
    /// Timestamp
    pub created_at: DateTime<Utc>,
    /// Expires at
    pub expires_at: Option<DateTime<Utc>>,
}

/// Notification types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationType {
    /// Threat detected
    Threat,
    /// Update available
    Update,
    /// Statistics milestone
    Milestone,
    /// Warning
    Warning,
    /// Info
    Info,
    /// Success
    Success,
}

/// Rule editor state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEditorState {
    /// Current rule text
    pub rule_text: String,
    /// Validation result
    pub validation: ValidationResult,
    /// Rule preview
    pub preview: Option<RulePreview>,
    /// Suggestions
    pub suggestions: Vec<RuleSuggestion>,
    /// Editor mode
    pub editor_mode: EditorMode,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Is valid
    pub is_valid: bool,
    /// Error message if invalid
    pub error: Option<String>,
    /// Warnings
    pub warnings: Vec<String>,
    /// Rule type detected
    pub detected_type: Option<String>,
}

/// Rule preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulePreview {
    /// Example URLs that would match
    pub matching_examples: Vec<String>,
    /// Example URLs that wouldn't match
    pub non_matching_examples: Vec<String>,
    /// Affected domains
    pub affected_domains: Vec<String>,
}

/// Rule suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSuggestion {
    /// Suggestion text
    pub text: String,
    /// Description
    pub description: String,
    /// Suggestion type
    pub suggestion_type: SuggestionType,
}

/// Suggestion types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuggestionType {
    AutoComplete,
    Documentation,
    Example,
    Optimization,
}

/// Editor mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorMode {
    Simple,
    Advanced,
    Expert,
}

/// UI Manager for Ad Blocker
pub struct UIManager {
    /// Current popup state
    popup_state: PopupState,
    /// Current settings state
    settings_state: SettingsState,
    /// Notifications queue
    notifications: Vec<Notification>,
}

impl UIManager {
    /// Create a new UI manager
    pub fn new() -> Self {
        Self {
            popup_state: PopupState::default(),
            settings_state: SettingsState::default(),
            notifications: Vec::new(),
        }
    }

    /// Get popup state
    pub fn get_popup_state(&self) -> &PopupState {
        &self.popup_state
    }

    /// Update popup state
    pub fn update_popup_state<F>(&mut self, f: F)
    where
        F: FnOnce(&mut PopupState),
    {
        f(&mut self.popup_state);
    }

    /// Get settings state
    pub fn get_settings_state(&self) -> &SettingsState {
        &self.settings_state
    }

    /// Update settings state
    pub fn update_settings_state<F>(&mut self, f: F)
    where
        F: FnOnce(&mut SettingsState),
    {
        f(&mut self.settings_state);
    }

    /// Add notification
    pub fn add_notification(&mut self, notification: Notification) {
        self.notifications.push(notification);
        // Keep only last 10 notifications
        if self.notifications.len() > 10 {
            self.notifications.remove(0);
        }
    }

    /// Get notifications
    pub fn get_notifications(&self) -> &[Notification] {
        &self.notifications
    }

    /// Dismiss notification
    pub fn dismiss_notification(&mut self, id: &str) -> bool {
        if let Some(pos) = self.notifications.iter().position(|n| n.id == id) {
            self.notifications.remove(pos);
            true
        } else {
            false
        }
    }

    /// Create blocked item from data
    pub fn create_blocked_item(
        url: String,
        resource_type: String,
        block_reason: String,
        filter_rule: String,
        domain: String,
        size: Option<u64>,
    ) -> BlockedItem {
        BlockedItem {
            id: format!("block_{}", Utc::now().timestamp_millis()),
            url,
            resource_type,
            block_reason,
            filter_rule,
            blocked_at: Utc::now(),
            domain,
            can_unblock: true,
            size,
        }
    }

    /// Create notification for threat
    pub fn create_threat_notification(title: String, message: String) -> Notification {
        Notification {
            id: format!("notif_{}", Utc::now().timestamp_millis()),
            notification_type: NotificationType::Threat,
            title,
            message,
            icon: "shield-alert".to_string(),
            action_text: Some("View Details".to_string()),
            action_url: Some("#/settings/security".to_string()),
            dismissible: true,
            created_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
        }
    }

    /// Create notification for milestone
    pub fn create_milestone_notification(milestone: u64) -> Notification {
        Notification {
            id: format!("milestone_{}", milestone),
            notification_type: NotificationType::Milestone,
            title: "Milestone Reached!".to_string(),
            message: format!("You've blocked {} ads and trackers!", milestone),
            icon: "trophy".to_string(),
            action_text: Some("View Stats".to_string()),
            action_url: Some("#/dashboard".to_string()),
            dismissible: true,
            created_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::hours(24)),
        }
    }

    /// Export settings to JSON
    pub fn export_settings(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.settings_state)
    }

    /// Import settings from JSON
    pub fn import_settings(&mut self, json: &str) -> Result<(), serde_json::Error> {
        self.settings_state = serde_json::from_str(json)?;
        Ok(())
    }
}

impl Default for PopupState {
    fn default() -> Self {
        Self {
            enabled: true,
            page_url: String::new(),
            domain: String::new(),
            page_stats: PageBlockStats::default(),
            total_stats: TotalBlockStats::default(),
            quick_toggles: vec![
                QuickToggle {
                    id: "ads".to_string(),
                    label: "Block Ads".to_string(),
                    icon: "block".to_string(),
                    enabled: true,
                    category: ToggleCategory::Ads,
                    tooltip: "Block advertisements on this page".to_string(),
                },
                QuickToggle {
                    id: "trackers".to_string(),
                    label: "Block Trackers".to_string(),
                    icon: "visibility_off".to_string(),
                    enabled: true,
                    category: ToggleCategory::Trackers,
                    tooltip: "Block tracking scripts".to_string(),
                },
                QuickToggle {
                    id: "social".to_string(),
                    label: "Block Social Widgets".to_string(),
                    icon: "share_off".to_string(),
                    enabled: false,
                    category: ToggleCategory::Social,
                    tooltip: "Block social media buttons".to_string(),
                },
            ],
            recent_blocked: Vec::new(),
            is_whitelisted: false,
            whitelist_reason: None,
        }
    }
}

impl Default for PageBlockStats {
    fn default() -> Self {
        Self {
            ads_blocked: 0,
            trackers_blocked: 0,
            scripts_blocked: 0,
            bandwidth_saved: 0,
            time_saved_ms: 0,
            threats_blocked: 0,
        }
    }
}

impl Default for TotalBlockStats {
    fn default() -> Self {
        Self {
            total_ads_blocked: 0,
            total_trackers_blocked: 0,
            total_scripts_blocked: 0,
            total_bandwidth_saved: 0,
            total_time_saved_seconds: 0,
            total_threats_blocked: 0,
            tracking_since: Utc::now(),
        }
    }
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            general: GeneralSettings::default(),
            filter_lists: FilterListsSettings::default(),
            custom_rules: CustomRulesSettings::default(),
            privacy: PrivacySettings::default(),
            advanced: AdvancedSettings::default(),
            ui_preferences: UIPreferences::default(),
        }
    }
}

impl Default for FilterListsSettings {
    fn default() -> Self {
        Self {
            available_lists: vec![
                FilterListInfo {
                    id: "easylist".to_string(),
                    name: "EasyList".to_string(),
                    description: "Primary ad blocking list".to_string(),
                    url: Some("https://easylist.to/easylist/easylist.txt".to_string()),
                    category: "Ads".to_string(),
                    language: None,
                    rule_count: 70000,
                    enabled: true,
                    last_updated: Some(Utc::now()),
                    is_updating: false,
                },
                FilterListInfo {
                    id: "easyprivacy".to_string(),
                    name: "EasyPrivacy".to_string(),
                    description: "Privacy protection list".to_string(),
                    url: Some("https://easylist.to/easylist/easyprivacy.txt".to_string()),
                    category: "Privacy".to_string(),
                    language: None,
                    rule_count: 20000,
                    enabled: true,
                    last_updated: Some(Utc::now()),
                    is_updating: false,
                },
            ],
            enabled_lists: vec!["easylist".to_string(), "easyprivacy".to_string()],
            last_update: Some(Utc::now()),
            custom_lists: Vec::new(),
            total_rules: 90000,
        }
    }
}

impl Default for CustomRulesSettings {
    fn default() -> Self {
        Self {
            rules: Vec::new(),
            groups: Vec::new(),
            presets: vec![
                PresetInfo {
                    id: "privacy-basic".to_string(),
                    name: "Basic Privacy".to_string(),
                    description: "Essential privacy rules".to_string(),
                    category: "Privacy".to_string(),
                    installed: false,
                },
            ],
            allow_import_export: true,
        }
    }
}

impl Default for UIManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_ui_manager() {
        let manager = UIManager::new();
        assert!(manager.get_popup_state().enabled);
    }

    #[test]
    fn test_update_popup_state() {
        let mut manager = UIManager::new();
        manager.update_popup_state(|state| {
            state.page_stats.ads_blocked = 10;
        });
        
        assert_eq!(manager.get_popup_state().page_stats.ads_blocked, 10);
    }

    #[test]
    fn test_create_blocked_item() {
        let item = UIManager::create_blocked_item(
            "https://ads.com/banner.js".to_string(),
            "script".to_string(),
            "Ad detected".to_string(),
            "||ads.com^".to_string(),
            "ads.com".to_string(),
            Some(1024),
        );
        
        assert_eq!(item.url, "https://ads.com/banner.js");
        assert_eq!(item.resource_type, "script");
        assert!(item.can_unblock);
    }

    #[test]
    fn test_add_notification() {
        let mut manager = UIManager::new();
        let notif = UIManager::create_threat_notification(
            "Threat Detected".to_string(),
            "Malicious script blocked".to_string(),
        );
        
        manager.add_notification(notif);
        assert_eq!(manager.get_notifications().len(), 1);
    }

    #[test]
    fn test_dismiss_notification() {
        let mut manager = UIManager::new();
        let notif = UIManager::create_threat_notification(
            "Threat".to_string(),
            "Blocked".to_string(),
        );
        let id = notif.id.clone();
        
        manager.add_notification(notif);
        assert_eq!(manager.get_notifications().len(), 1);
        
        manager.dismiss_notification(&id);
        assert_eq!(manager.get_notifications().len(), 0);
    }

    #[test]
    fn test_default_settings() {
        let settings = SettingsState::default();
        assert!(settings.general.ad_blocking_enabled);
        assert!(settings.privacy.block_fingerprinting);
    }

    #[test]
    fn test_export_import_settings() {
        let mut manager = UIManager::new();
        manager.update_settings_state(|s| {
            s.general.language = "de".to_string();
        });
        
        let exported = manager.export_settings().unwrap();
        
        let mut new_manager = UIManager::new();
        new_manager.import_settings(&exported).unwrap();
        
        assert_eq!(new_manager.get_settings_state().general.language, "de");
    }

    #[test]
    fn test_milestone_notification() {
        let notif = UIManager::create_milestone_notification(10000);
        assert_eq!(notif.notification_type, NotificationType::Milestone);
        assert!(notif.message.contains("10000"));
    }
}