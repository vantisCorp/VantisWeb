//! Suggestions Engine
//!
//! Provides smart suggestions for optimizing the user experience.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Suggestion category
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SuggestionCategory {
    /// Productivity suggestions
    Productivity,
    /// Security suggestions
    Security,
    /// Performance suggestions
    Performance,
    /// Privacy suggestions
    Privacy,
    /// Accessibility suggestions
    Accessibility,
    /// Custom suggestions
    Custom(String),
}

/// Suggestion action type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SuggestionAction {
    /// Apply setting
    ApplySetting { key: String, value: String },
    /// Install extension
    InstallExtension { id: String },
    /// Enable feature
    EnableFeature { feature: String },
    /// Open URL
    OpenUrl { url: String },
    /// Custom action
    Custom { action: String },
}

/// A smart suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    /// Unique ID
    pub id: String,
    /// Category
    pub category: SuggestionCategory,
    /// Title
    pub title: String,
    /// Description
    pub description: String,
    /// Action to take
    pub action: SuggestionAction,
    /// Action text for UI
    pub action_text: String,
    /// Priority (0.0 - 1.0)
    pub priority: f64,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Whether suggestion was applied
    pub applied: bool,
    /// Whether suggestion was dismissed
    pub dismissed: bool,
}

impl Suggestion {
    /// Create a new suggestion
    pub fn new(category: SuggestionCategory, title: String, description: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            category,
            title,
            description,
            action: SuggestionAction::Custom { action: "Apply".to_string() },
            action_text: "Apply".to_string(),
            priority: 0.5,
            created_at: Utc::now(),
            applied: false,
            dismissed: false,
        }
    }

    /// With action
    pub fn with_action(mut self, action: SuggestionAction, action_text: String) -> Self {
        self.action = action;
        self.action_text = action_text;
        self
    }

    /// With priority
    pub fn with_priority(mut self, priority: f64) -> Self {
        self.priority = priority.clamp(0.0, 1.0);
        self
    }

    /// Mark as applied
    pub fn apply(&mut self) {
        self.applied = true;
    }

    /// Mark as dismissed
    pub fn dismiss(&mut self) {
        self.dismissed = true;
    }
}

/// Suggestions engine
pub struct SuggestionEngine {
    /// Generated suggestions
    suggestions: Vec<Suggestion>,
    /// Configuration
    config: SuggestionConfig,
}

/// Configuration for suggestions engine
#[derive(Debug, Clone)]
struct SuggestionConfig {
    /// Maximum suggestions
    max_suggestions: usize,
    /// Minimum priority
    min_priority: f64,
}

impl Default for SuggestionConfig {
    fn default() -> Self {
        Self {
            max_suggestions: 5,
            min_priority: 0.6,
        }
    }
}

impl SuggestionEngine {
    /// Create a new suggestions engine
    pub fn new() -> Self {
        Self {
            suggestions: Vec::new(),
            config: SuggestionConfig::default(),
        }
    }

    /// Generate contextual suggestions
    pub fn generate(&mut self, context: &SuggestionContext) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        // Time-based suggestions
        suggestions.extend(self.generate_time_based_suggestions(context));

        // Activity-based suggestions
        suggestions.extend(self.generate_activity_based_suggestions(context));

        // Security suggestions
        suggestions.extend(self.generate_security_suggestions(context));

        // Performance suggestions
        suggestions.extend(self.generate_performance_suggestions(context));

        // Sort and filter
        suggestions.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());
        suggestions.retain(|s| s.priority >= self.config.min_priority);
        suggestions.truncate(self.config.max_suggestions);

        self.suggestions = suggestions.clone();
        suggestions
    }

    /// Generate time-based suggestions
    fn generate_time_based_suggestions(&self, context: &SuggestionContext) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        let hour = context.current_time.hour();

        // Morning (6-11)
        if hour >= 6 && hour < 11 {
            suggestions.push(
                Suggestion::new(
                    SuggestionCategory::Productivity,
                    "Morning Briefing".to_string(),
                    "Start your day with a personalized news and email briefing.".to_string(),
                )
                .with_action(SuggestionAction::OpenUrl { url: "vantis://briefing".to_string() }, "View Briefing".to_string())
                .with_priority(0.7)
            );
        }

        // Evening (18-22)
        if hour >= 18 && hour < 22 {
            suggestions.push(
                Suggestion::new(
                    SuggestionCategory::Productivity,
                    "Evening Review".to_string(),
                    "Review your browsing activity and accomplishments for the day.".to_string(),
                )
                .with_action(SuggestionAction::OpenUrl { url: "vantis://analytics".to_string() }, "View Analytics".to_string())
                .with_priority(0.6)
            );
        }

        // Late night (22+)
        if hour >= 22 {
            suggestions.push(
                Suggestion::new(
                    SuggestionCategory::Productivity,
                    "Night Mode Reminder".to_string(),
                    "Consider enabling night mode for better eye health during late-night browsing.".to_string(),
                )
                .with_action(SuggestionAction::ApplySetting { key: "display.night_mode".to_string(), value: "true".to_string() }, "Enable Night Mode".to_string())
                .with_priority(0.8)
            );
        }

        suggestions
    }

    /// Generate activity-based suggestions
    fn generate_activity_based_suggestions(&self, context: &SuggestionContext) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        // High tab count
        if context.active_tabs > 20 {
            suggestions.push(
                Suggestion::new(
                    SuggestionCategory::Performance,
                    "Reduce Tab Clutter".to_string(),
                    format!("You have {} tabs open. Consider using tab groups or closing unused tabs.", context.active_tabs),
                )
                .with_action(SuggestionAction::EnableFeature { feature: "tab_sleeping".to_string() }, "Enable Tab Sleeping".to_string())
                .with_priority(0.8)
            );
        }

        // Long session
        if context.session_duration_minutes > 60 {
            suggestions.push(
                Suggestion::new(
                    SuggestionCategory::Productivity,
                    "Take a Break".to_string(),
                    "You've been browsing for over an hour. Take a short break to rest your eyes.".to_string(),
                )
                .with_action(SuggestionAction::Custom { action: "break_reminder".to_string() }, "Set Break Reminder".to_string())
                .with_priority(0.7)
            );
        }

        // High memory usage
        if context.memory_usage_mb > 1000 {
            suggestions.push(
                Suggestion::new(
                    SuggestionCategory::Performance,
                    "High Memory Usage".to_string(),
                    format!("Memory usage is at {} MB. Consider closing some tabs.", context.memory_usage_mb),
                )
                .with_action(SuggestionAction::EnableFeature { feature: "automatic_memory_cleanup".to_string() }, "Enable Auto Cleanup".to_string())
                .with_priority(0.85)
            );
        }

        suggestions
    }

    /// Generate security suggestions
    fn generate_security_suggestions(&self, context: &SuggestionContext) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        // Suggest password manager if many logins
        if context.websites_requiring_login > 5 {
            suggestions.push(
                Suggestion::new(
                    SuggestionCategory::Security,
                    "Use a Password Manager".to_string(),
                    "You visit many sites requiring login. Consider using a password manager for better security.".to_string(),
                )
                .with_action(SuggestionAction::InstallExtension { id: "password-manager".to_string() }, "Install Password Manager".to_string())
                .with_priority(0.75)
            );
        }

        // Suggest 2FA for banking/shopping
        if context.visited_banking || context.visited_shopping {
            suggestions.push(
                Suggestion::new(
                    SuggestionCategory::Security,
                    "Enable Two-Factor Authentication".to_string(),
                    "Your banking and shopping accounts should have 2FA enabled for extra security.".to_string(),
                )
                .with_action(SuggestionAction::OpenUrl { url: "vantis://security/2fa".to_string() }, "Learn About 2FA".to_string())
                .with_priority(0.9)
            );
        }

        suggestions
    }

    /// Generate performance suggestions
    fn generate_performance_suggestions(&self, context: &SuggestionContext) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        // Suggest ad blocker
        if context.slow_page_loads > 5 {
            suggestions.push(
                Suggestion::new(
                    SuggestionCategory::Performance,
                    "Block Ads for Faster Browsing".to_string(),
                    "Ads can slow down page loads. Consider installing an ad blocker.".to_string(),
                )
                .with_action(SuggestionAction::InstallExtension { id: "ad-blocker".to_string() }, "Install Ad Blocker".to_string())
                .with_priority(0.8)
            );
        }

        // Suggest image compression
        if context.low_bandwidth {
            suggestions.push(
                Suggestion::new(
                    SuggestionCategory::Performance,
                    "Enable Data Saver Mode".to_string(),
                    "Reduce data usage and improve speed on slow connections.".to_string(),
                )
                .with_action(SuggestionAction::ApplySetting { key: "performance.data_saver".to_string(), value: "true".to_string() }, "Enable Data Saver".to_string())
                .with_priority(0.85)
            );
        }

        suggestions
    }

    /// Get all suggestions
    pub fn get_suggestions(&self) -> &[Suggestion] {
        &self.suggestions
    }

    /// Apply a suggestion
    pub fn apply_suggestion(&mut self, suggestion_id: &str) -> Result<()> {
        if let Some(suggestion) = self.suggestions.iter_mut().find(|s| s.id == suggestion_id) {
            suggestion.apply();
            log::info!("Applied suggestion: {}", suggestion_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Suggestion not found: {}", suggestion_id))
        }
    }

    /// Dismiss a suggestion
    pub fn dismiss_suggestion(&mut self, suggestion_id: &str) -> Result<()> {
        if let Some(suggestion) = self.suggestions.iter_mut().find(|s| s.id == suggestion_id) {
            suggestion.dismiss();
            log::info!("Dismissed suggestion: {}", suggestion_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Suggestion not found: {}", suggestion_id))
        }
    }

    /// Clear all suggestions
    pub fn clear(&mut self) {
        self.suggestions.clear();
    }
}

/// Context for generating suggestions
#[derive(Debug, Clone)]
pub struct SuggestionContext {
    /// Current time
    pub current_time: DateTime<Utc>,
    /// Active tab count
    pub active_tabs: u32,
    /// Session duration in minutes
    pub session_duration_minutes: u32,
    /// Memory usage in MB
    pub memory_usage_mb: u32,
    /// Number of websites requiring login
    pub websites_requiring_login: u32,
    /// Whether visited banking sites
    pub visited_banking: bool,
    /// Whether visited shopping sites
    pub visited_shopping: bool,
    /// Number of slow page loads
    pub slow_page_loads: u32,
    /// Whether on low bandwidth connection
    pub low_bandwidth: bool,
}

impl Default for SuggestionContext {
    fn default() -> Self {
        Self {
            current_time: Utc::now(),
            active_tabs: 5,
            session_duration_minutes: 10,
            memory_usage_mb: 300,
            websites_requiring_login: 0,
            visited_banking: false,
            visited_shopping: false,
            slow_page_loads: 0,
            low_bandwidth: false,
        }
    }
}

impl Default for SuggestionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggestion_engine_creation() {
        let engine = SuggestionEngine::new();
        assert!(engine.suggestions.is_empty());
    }

    #[test]
    fn test_generate_suggestions() {
        let mut engine = SuggestionEngine::new();
        let context = SuggestionContext {
            active_tabs: 25,
            session_duration_minutes: 70,
            memory_usage_mb: 1200,
            ..Default::default()
        };

        let suggestions = engine.generate(&context);
        assert!(!suggestions.is_empty());
    }

    #[test]
    fn test_apply_suggestion() {
        let mut engine = SuggestionEngine::new();
        let context = SuggestionContext::default();

        engine.generate(&context);
        if !engine.suggestions.is_empty() {
            let suggestion_id = engine.suggestions[0].id.clone();
            engine.apply_suggestion(&suggestion_id).unwrap();
            assert!(engine.suggestions[0].applied);
        }
    }

    #[test]
    fn test_dismiss_suggestion() {
        let mut engine = SuggestionEngine::new();
        let context = SuggestionContext::default();

        engine.generate(&context);
        if !engine.suggestions.is_empty() {
            let suggestion_id = engine.suggestions[0].id.clone();
            engine.dismiss_suggestion(&suggestion_id).unwrap();
            assert!(engine.suggestions[0].dismissed);
        }
    }
}