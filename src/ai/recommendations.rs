//! Profile Recommendations Engine
//!
//! AI-powered recommendations for profile settings, templates, and features.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::pattern_analyzer::{UsagePattern, PatternType, PatternMetrics};

/// Recommendation type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecommendationType {
    /// Profile setting optimization
    ProfileSetting {
        setting_key: String,
        suggested_value: String,
        current_value: Option<String>,
    },
    /// Template recommendation
    Template {
        template_id: String,
        template_name: String,
    },
    /// Extension suggestion
    Extension {
        extension_id: String,
        extension_name: String,
        reason: String,
    },
    /// Security recommendation
    Security {
        level: String,
        reason: String,
    },
    /// Performance tip
    Performance {
        tip: String,
        impact: String,
    },
    /// Productivity tip
    Productivity {
        tip: String,
        benefit: String,
    },
    /// Custom recommendation
    Custom {
        title: String,
        description: String,
        action: String,
    },
}

/// Recommendation priority
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl RecommendationPriority {
    pub fn to_weight(&self) -> f64 {
        match self {
            RecommendationPriority::Low => 0.25,
            RecommendationPriority::Medium => 0.5,
            RecommendationPriority::High => 0.75,
            RecommendationPriority::Critical => 1.0,
        }
    }
}

/// A single recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Unique ID
    pub id: String,
    /// Recommendation type
    pub recommendation_type: RecommendationType,
    /// Priority level
    pub priority: RecommendationPriority,
    /// Title
    pub title: String,
    /// Description
    pub description: String,
    /// Reason for recommendation
    pub reason: String,
    /// Expected benefit
    pub benefit: String,
    /// Action to take
    pub action_text: String,
    /// Action type (accept, dismiss, learn_more)
    pub action_type: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Expiration timestamp (if applicable)
    pub expires_at: Option<DateTime<Utc>>,
    /// Whether recommendation was accepted
    pub accepted: bool,
    /// Whether recommendation was dismissed
    pub dismissed: bool,
    /// Related patterns
    pub related_patterns: Vec<PatternType>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
}

impl Recommendation {
    /// Create a new recommendation
    pub fn new(
        recommendation_type: RecommendationType,
        title: String,
        description: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            recommendation_type,
            priority: RecommendationPriority::Medium,
            title,
            description,
            reason: String::new(),
            benefit: String::new(),
            action_text: "Apply".to_string(),
            action_type: "accept".to_string(),
            created_at: Utc::now(),
            expires_at: None,
            accepted: false,
            dismissed: false,
            related_patterns: Vec::new(),
            confidence: 0.8,
        }
    }

    /// Set priority
    pub fn with_priority(mut self, priority: RecommendationPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set reason
    pub fn with_reason(mut self, reason: String) -> Self {
        self.reason = reason;
        self
    }

    /// Set benefit
    pub fn with_benefit(mut self, benefit: String) -> Self {
        self.benefit = benefit;
        self
    }

    /// Set action
    pub fn with_action(mut self, action_text: String, action_type: String) -> Self {
        self.action_text = action_text;
        self.action_type = action_type;
        self
    }

    /// Set confidence
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Mark as accepted
    pub fn accept(&mut self) {
        self.accepted = true;
    }

    /// Mark as dismissed
    pub fn dismiss(&mut self) {
        self.dismissed = true;
    }
}

/// Recommendation engine
pub struct RecommendationEngine {
    /// Generated recommendations
    recommendations: HashMap<String, Vec<Recommendation>>,
    /// Configuration
    config: RecommendationConfig,
}

/// Configuration for recommendation engine
#[derive(Debug, Clone)]
struct RecommendationConfig {
    /// Maximum recommendations per profile
    max_recommendations: usize,
    /// Minimum confidence threshold
    min_confidence: f64,
    /// Enable auto-suggestions
    auto_suggest: bool,
}

impl Default for RecommendationConfig {
    fn default() -> Self {
        Self {
            max_recommendations: 10,
            min_confidence: 0.6,
            auto_suggest: true,
        }
    }
}

impl RecommendationEngine {
    /// Create a new recommendation engine
    pub fn new() -> Self {
        Self {
            recommendations: HashMap::new(),
            config: RecommendationConfig::default(),
        }
    }

    /// Generate recommendations based on patterns
    pub fn generate(&mut self, profile_id: &str, patterns: &[UsagePattern]) -> Result<Vec<Recommendation>> {
        log::info!("Generating recommendations for profile: {}", profile_id);

        let mut recommendations = Vec::new();

        // Generate recommendations based on each pattern
        for pattern in patterns {
            recommendations.extend(self.generate_for_pattern(pattern));
        }

        // Sort by priority and confidence
        recommendations.sort_by(|a, b| {
            let score_a = a.priority.to_weight() * a.confidence;
            let score_b = b.priority.to_weight() * b.confidence;
            score_b.partial_cmp(&score_a).unwrap()
        });

        // Limit recommendations
        recommendations.truncate(self.config.max_recommendations);

        // Filter by confidence
        recommendations.retain(|r| r.confidence >= self.config.min_confidence);

        // Cache recommendations
        self.recommendations.insert(profile_id.to_string(), recommendations.clone());

        log::info!("Generated {} recommendations for profile: {}", recommendations.len(), profile_id);
        Ok(recommendations)
    }

    /// Generate recommendations for a specific pattern
    fn generate_for_pattern(&self, pattern: &UsagePattern) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        match &pattern.pattern_type {
            PatternType::ExtendedSessions => {
                recommendations.push(
                    Recommendation::new(
                        RecommendationType::Performance {
                            tip: "Enable session reminders".to_string(),
                            impact: "Helps manage screen time and prevent eye strain".to_string(),
                        },
                        "Take Regular Breaks".to_string(),
                        "Your sessions tend to be long. Consider enabling break reminders.".to_string(),
                    )
                    .with_priority(RecommendationPriority::Medium)
                    .with_reason("Extended screen time can cause eye strain and fatigue".to_string())
                    .with_benefit("Improved health and productivity".to_string())
                    .with_confidence(pattern.confidence.to_score())
                );
            }
            PatternType::NightOwl => {
                recommendations.push(
                    Recommendation::new(
                        RecommendationType::ProfileSetting {
                            setting_key: "display.night_mode".to_string(),
                            suggested_value: "true".to_string(),
                            current_value: None,
                        },
                        "Enable Night Mode".to_string(),
                        "You're most active at night. Night mode can reduce eye strain.".to_string(),
                    )
                    .with_priority(RecommendationPriority::High)
                    .with_reason("Night-time browsing benefits from reduced blue light".to_string())
                    .with_benefit("Better sleep quality and reduced eye strain".to_string())
                    .with_confidence(pattern.confidence.to_score())
                );

                recommendations.push(
                    Recommendation::new(
                        RecommendationType::Performance {
                            tip: "Enable dark theme".to_string(),
                            impact: "Reduces eye strain in low-light conditions".to_string(),
                        },
                        "Switch to Dark Theme".to_string(),
                        "A dark theme is perfect for your night-time browsing habits.".to_string(),
                    )
                    .with_priority(RecommendationPriority::Medium)
                    .with_confidence(pattern.confidence.to_score() * 0.9)
                );
            }
            PatternType::EarlyBird => {
                recommendations.push(
                    Recommendation::new(
                        RecommendationType::Productivity {
                            tip: "Set up morning news digest".to_string(),
                            benefit: "Get the most out of your productive morning hours".to_string(),
                        },
                        "Morning Productivity Setup".to_string(),
                        "You're most active in the morning. Set up your daily briefing.".to_string(),
                    )
                    .with_priority(RecommendationPriority::Medium)
                    .with_confidence(pattern.confidence.to_score())
                );
            }
            PatternType::Developer => {
                recommendations.push(
                    Recommendation::new(
                        RecommendationType::Template {
                            template_id: "dev-power-user".to_string(),
                            template_name: "Developer Power User".to_string(),
                        },
                        "Use Developer Profile Template".to_string(),
                        "Your browsing suggests you're a developer. Try our developer template.".to_string(),
                    )
                    .with_priority(RecommendationPriority::High)
                    .with_reason("Optimized for development workflow".to_string())
                    .with_benefit("Faster development, better tools integration".to_string())
                    .with_confidence(pattern.confidence.to_score())
                );

                recommendations.push(
                    Recommendation::new(
                        RecommendationType::Extension {
                            extension_id: "github-enhanced".to_string(),
                            extension_name: "GitHub Enhanced".to_string(),
                            reason: "You frequently visit GitHub".to_string(),
                        },
                        "Install GitHub Enhancement Extension".to_string(),
                        "Enhance your GitHub experience with additional features.".to_string(),
                    )
                    .with_priority(RecommendationPriority::Medium)
                    .with_confidence(pattern.confidence.to_score() * 0.8)
                );

                recommendations.push(
                    Recommendation::new(
                        RecommendationType::ProfileSetting {
                            setting_key: "developer.tools_enabled".to_string(),
                            suggested_value: "true".to_string(),
                            current_value: None,
                        },
                        "Enable Developer Tools Quick Access".to_string(),
                        "Quick access to developer tools can speed up your workflow.".to_string(),
                    )
                    .with_priority(RecommendationPriority::Medium)
                    .with_confidence(pattern.confidence.to_score())
                );
            }
            PatternType::EntertainmentFocused => {
                recommendations.push(
                    Recommendation::new(
                        RecommendationType::Template {
                            template_id: "entertainment".to_string(),
                            template_name: "Entertainment".to_string(),
                        },
                        "Use Entertainment Profile Template".to_string(),
                        "An entertainment-optimized profile for your viewing habits.".to_string(),
                    )
                    .with_priority(RecommendationPriority::Medium)
                    .with_confidence(pattern.confidence.to_score())
                );

                recommendations.push(
                    Recommendation::new(
                        RecommendationType::Performance {
                            tip: "Enable video quality auto-adjustment".to_string(),
                            impact: "Smoother streaming experience".to_string(),
                        },
                        "Optimize Video Streaming".to_string(),
                        "Enable auto-quality adjustment for the best streaming experience.".to_string(),
                    )
                    .with_priority(RecommendationPriority::Low)
                    .with_confidence(pattern.confidence.to_score() * 0.7)
                );
            }
            PatternType::SocialButterfly => {
                recommendations.push(
                    Recommendation::new(
                        RecommendationType::Security {
                            level: "Enhanced".to_string(),
                            reason: "Social media accounts are high-value targets".to_string(),
                        },
                        "Enable Enhanced Security".to_string(),
                        "Social media users are common targets. Enable 2FA and privacy features.".to_string(),
                    )
                    .with_priority(RecommendationPriority::High)
                    .with_reason("Social media accounts often contain sensitive personal information".to_string())
                    .with_benefit("Better protection of your personal data".to_string())
                    .with_confidence(pattern.confidence.to_score())
                );

                recommendations.push(
                    Recommendation::new(
                        RecommendationType::Extension {
                            extension_id: "social-media-manager".to_string(),
                            extension_name: "Social Media Manager".to_string(),
                            reason: "Manage multiple social accounts efficiently".to_string(),
                        },
                        "Install Social Media Manager".to_string(),
                        "Manage your social media presence across platforms.".to_string(),
                    )
                    .with_priority(RecommendationPriority::Low)
                    .with_confidence(pattern.confidence.to_score() * 0.6)
                );
            }
            PatternType::Shopper => {
                recommendations.push(
                    Recommendation::new(
                        RecommendationType::Security {
                            level: "Strict".to_string(),
                            reason: "Shopping involves financial transactions".to_string(),
                        },
                        "Enable Strict Security Mode".to_string(),
                        "Protect your financial information while shopping online.".to_string(),
                    )
                    .with_priority(RecommendationPriority::High)
                    .with_reason("Financial data requires extra protection".to_string())
                    .with_benefit("Safer online transactions".to_string())
                    .with_confidence(pattern.confidence.to_score())
                );

                recommendations.push(
                    Recommendation::new(
                        RecommendationType::Extension {
                            extension_id: "price-tracker".to_string(),
                            extension_name: "Price Tracker".to_string(),
                            reason: "Track prices across shopping sites".to_string(),
                        },
                        "Install Price Tracker".to_string(),
                        "Automatically track prices and get alerts for the best deals.".to_string(),
                    )
                    .with_priority(RecommendationPriority::Medium)
                    .with_confidence(pattern.confidence.to_score() * 0.8)
                );
            }
            PatternType::TabHoarding => {
                recommendations.push(
                    Recommendation::new(
                        RecommendationType::Performance {
                            tip: "Enable tab sleeping".to_string(),
                            impact: "Reduce memory usage by up to 80%".to_string(),
                        },
                        "Enable Tab Sleeping".to_string(),
                        "Reduce memory usage by automatically sleeping inactive tabs.".to_string(),
                    )
                    .with_priority(RecommendationPriority::High)
                    .with_reason(format!("You average {} tabs per session", pattern.metrics.avg_tabs_per_session as u32))
                    .with_benefit("Significant memory savings and better performance".to_string())
                    .with_confidence(pattern.confidence.to_score())
                );

                recommendations.push(
                    Recommendation::new(
                        RecommendationType::Extension {
                            extension_id: "tab-manager".to_string(),
                            extension_name: "Tab Manager Pro".to_string(),
                            reason: "Organize and manage many tabs efficiently".to_string(),
                        },
                        "Install Tab Manager".to_string(),
                        "Better organize and manage your tabs with a dedicated manager.".to_string(),
                    )
                    .with_priority(RecommendationPriority::Medium)
                    .with_confidence(pattern.confidence.to_score() * 0.9)
                );
            }
            PatternType::TabMinimizer => {
                recommendations.push(
                    Recommendation::new(
                        RecommendationType::Productivity {
                            tip: "Your tab management is efficient".to_string(),
                            benefit: "Keep up the good habit".to_string(),
                        },
                        "Great Tab Management!".to_string(),
                        "You keep your browsing focused. Consider bookmarking important sites.".to_string(),
                    )
                    .with_priority(RecommendationPriority::Low)
                    .with_confidence(pattern.confidence.to_score() * 0.5)
                );
            }
            PatternType::Learner => {
                recommendations.push(
                    Recommendation::new(
                        RecommendationType::Template {
                            template_id: "education".to_string(),
                            template_name: "Education".to_string(),
                        },
                        "Use Education Profile Template".to_string(),
                        "An optimized profile for learning and research.".to_string(),
                    )
                    .with_priority(RecommendationPriority::Medium)
                    .with_confidence(pattern.confidence.to_score())
                );

                recommendations.push(
                    Recommendation::new(
                        RecommendationType::Extension {
                            extension_id: "note-taking".to_string(),
                            extension_name: "Quick Notes".to_string(),
                            reason: "Take notes while learning".to_string(),
                        },
                        "Install Note-Taking Extension".to_string(),
                        "Quickly capture insights while browsing educational content.".to_string(),
                    )
                    .with_priority(RecommendationPriority::Medium)
                    .with_confidence(pattern.confidence.to_score() * 0.8)
                );
            }
            PatternType::WorkOriented => {
                recommendations.push(
                    Recommendation::new(
                        RecommendationType::Template {
                            template_id: "work".to_string(),
                            template_name: "Work Profile".to_string(),
                        },
                        "Use Work Profile Template".to_string(),
                        "A productivity-optimized profile for work tasks.".to_string(),
                    )
                    .with_priority(RecommendationPriority::High)
                    .with_confidence(pattern.confidence.to_score())
                );

                recommendations.push(
                    Recommendation::new(
                        RecommendationType::Productivity {
                            tip: "Enable focus mode during work hours".to_string(),
                            benefit: "Block distractions during peak productivity".to_string(),
                        },
                        "Enable Focus Mode".to_string(),
                        "Automatically block distracting sites during work hours.".to_string(),
                    )
                    .with_priority(RecommendationPriority::High)
                    .with_confidence(pattern.confidence.to_score() * 0.9)
                );
            }
            PatternType::MicroSessions => {
                recommendations.push(
                    Recommendation::new(
                        RecommendationType::Productivity {
                            tip: "Enable quick access bookmarks bar".to_string(),
                            benefit: "Faster access to frequently used sites".to_string(),
                        },
                        "Show Bookmarks Bar".to_string(),
                        "With quick sessions, instant access to bookmarks helps.".to_string(),
                    )
                    .with_priority(RecommendationPriority::Medium)
                    .with_confidence(pattern.confidence.to_score() * 0.7)
                );
            }
            PatternType::FrequentVisits => {
                recommendations.push(
                    Recommendation::new(
                        RecommendationType::Productivity {
                            tip: "Pin frequently visited tabs".to_string(),
                            benefit: "Keep important sites always available".to_string(),
                        },
                        "Pin Frequent Sites".to_string(),
                        "Pin your most visited sites for instant access.".to_string(),
                    )
                    .with_priority(RecommendationPriority::Medium)
                    .with_confidence(pattern.confidence.to_score() * 0.8)
                );
            }
            PatternType::Custom(name) => {
                // Handle custom patterns
                recommendations.push(
                    Recommendation::new(
                        RecommendationType::Custom {
                            title: format!("Custom Recommendation for {}", name),
                            description: "Based on your custom usage pattern.".to_string(),
                            action: "Configure".to_string(),
                        },
                        format!("Custom: {}", name),
                        "A recommendation based on your unique browsing pattern.".to_string(),
                    )
                    .with_priority(RecommendationPriority::Low)
                    .with_confidence(0.5)
                );
            }
            PatternType::NewsReader => {
                recommendations.push(
                    Recommendation::new(
                        RecommendationType::Extension {
                            extension_id: "news-aggregator".to_string(),
                            extension_name: "News Aggregator".to_string(),
                            reason: "You frequently read news".to_string(),
                        },
                        "Install News Aggregator".to_string(),
                        "Get all your news in one place with an intelligent aggregator.".to_string(),
                    )
                    .with_priority(RecommendationPriority::Medium)
                    .with_confidence(pattern.confidence.to_score() * 0.8)
                );
            }
        }

        // Add related patterns to each recommendation
        for rec in &mut recommendations {
            rec.related_patterns.push(pattern.pattern_type.clone());
        }

        recommendations
    }

    /// Get recommendations for a profile
    pub fn get_recommendations(&self, profile_id: &str) -> Option<&Vec<Recommendation>> {
        self.recommendations.get(profile_id)
    }

    /// Accept a recommendation
    pub fn accept_recommendation(&mut self, profile_id: &str, recommendation_id: &str) -> Result<()> {
        if let Some(recommendations) = self.recommendations.get_mut(profile_id) {
            if let Some(rec) = recommendations.iter_mut().find(|r| r.id == recommendation_id) {
                rec.accept();
                log::info!("Accepted recommendation: {} for profile: {}", recommendation_id, profile_id);
                return Ok(());
            }
        }
        Err(anyhow::anyhow!("Recommendation not found: {}", recommendation_id))
    }

    /// Dismiss a recommendation
    pub fn dismiss_recommendation(&mut self, profile_id: &str, recommendation_id: &str) -> Result<()> {
        if let Some(recommendations) = self.recommendations.get_mut(profile_id) {
            if let Some(rec) = recommendations.iter_mut().find(|r| r.id == recommendation_id) {
                rec.dismiss();
                log::info!("Dismissed recommendation: {} for profile: {}", recommendation_id, profile_id);
                return Ok(());
            }
        }
        Err(anyhow::anyhow!("Recommendation not found: {}", recommendation_id))
    }

    /// Clear recommendations for a profile
    pub fn clear_recommendations(&mut self, profile_id: &str) {
        self.recommendations.remove(profile_id);
    }

    /// Get active (not accepted/dismissed) recommendations
    pub fn get_active_recommendations(&self, profile_id: &str) -> Vec<&Recommendation> {
        self.recommendations
            .get(profile_id)
            .map(|recs| recs.iter().filter(|r| !r.accepted && !r.dismissed).collect())
            .unwrap_or_default()
    }
}

impl Default for RecommendationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::pattern_analyzer::PatternConfidence;

    fn create_test_pattern() -> UsagePattern {
        UsagePattern {
            pattern_type: PatternType::Developer,
            confidence: PatternConfidence::High,
            description: "Test pattern".to_string(),
            detected_at: Utc::now(),
            metrics: PatternMetrics {
                avg_session_length: 1800,
                sessions_per_day: 5,
                peak_hour: 10,
                dominant_category: "Development".to_string(),
                avg_tabs_per_session: 12.0,
                website_diversity: 0.3,
            },
        }
    }

    #[test]
    fn test_recommendation_engine_creation() {
        let engine = RecommendationEngine::new();
        assert!(engine.recommendations.is_empty());
    }

    #[test]
    fn test_generate_recommendations() {
        let mut engine = RecommendationEngine::new();
        let patterns = vec![create_test_pattern()];

        let recommendations = engine.generate("test", &patterns).unwrap();
        assert!(!recommendations.is_empty());
    }

    #[test]
    fn test_accept_recommendation() {
        let mut engine = RecommendationEngine::new();
        let patterns = vec![create_test_pattern()];

        engine.generate("test", &patterns).unwrap();
        let recs = engine.get_recommendations("test").unwrap();
        let rec_id = recs[0].id.clone();

        engine.accept_recommendation("test", &rec_id).unwrap();

        let recs = engine.get_recommendations("test").unwrap();
        assert!(recs.iter().find(|r| r.id == rec_id).unwrap().accepted);
    }

    #[test]
    fn test_dismiss_recommendation() {
        let mut engine = RecommendationEngine::new();
        let patterns = vec![create_test_pattern()];

        engine.generate("test", &patterns).unwrap();
        let recs = engine.get_recommendations("test").unwrap();
        let rec_id = recs[0].id.clone();

        engine.dismiss_recommendation("test", &rec_id).unwrap();

        let recs = engine.get_recommendations("test").unwrap();
        assert!(recs.iter().find(|r| r.id == rec_id).unwrap().dismissed);
    }

    #[test]
    fn test_active_recommendations() {
        let mut engine = RecommendationEngine::new();
        let patterns = vec![create_test_pattern()];

        engine.generate("test", &patterns).unwrap();
        let active = engine.get_active_recommendations("test");
        assert!(!active.is_empty());
    }
}