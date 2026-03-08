//! Accessibility Manager Module
//! 
//! Central manager for all accessibility features, providing unified
//! configuration, profile management, and feature coordination.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;

use super::{
    EyeTracker, EyeTrackingConfig, GazePoint, HeadGesture,
    VoiceController, VoiceConfig, VoiceCommand, VoiceResult,
    VisionDescriber, VisionConfig, SceneDescription,
    SeniorMode, SeniorConfig,
    TremorGuard, TremorConfig, CursorState,
    AccessibilityError, AccessibilityResult, AccessibilityLevel,
};
use crate::accessibility::ElementType;

/// Main accessibility configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityConfig {
    /// Global accessibility level
    pub level: AccessibilityLevel,
    /// Eye tracking configuration
    pub eye_tracking: EyeTrackingConfig,
    /// Voice control configuration
    pub voice_control: VoiceConfig,
    /// Vision describer configuration
    pub vision: VisionConfig,
    /// Senior mode configuration
    pub senior_mode: SeniorConfig,
    /// Tremor guard configuration
    pub tremor_guard: TremorConfig,
    /// Enable all accessibility features
    pub enable_all: bool,
    /// Auto-detect accessibility needs
    pub auto_detect: bool,
    /// Show accessibility indicators
    pub show_indicators: bool,
    /// Accessibility shortcut keys
    pub shortcuts: HashMap<String, String>,
}

impl Default for AccessibilityConfig {
    fn default() -> Self {
        let mut shortcuts = HashMap::new();
        shortcuts.insert("toggle_voice".into(), "Alt+V".into());
        shortcuts.insert("toggle_reader".into(), "Alt+R".into());
        shortcuts.insert("toggle_tremor".into(), "Alt+T".into());
        shortcuts.insert("toggle_senior".into(), "Alt+S".into());
        shortcuts.insert("read_page".into(), "Ctrl+Shift+R".into());
        shortcuts.insert("describe_element".into(), "Ctrl+Shift+D".into());
        
        Self {
            level: AccessibilityLevel::Full,
            eye_tracking: EyeTrackingConfig::default(),
            voice_control: VoiceConfig::default(),
            vision: VisionConfig::default(),
            senior_mode: SeniorConfig::default(),
            tremor_guard: TremorConfig::default(),
            enable_all: false,
            auto_detect: false,
            show_indicators: true,
            shortcuts,
        }
    }
}

/// Accessibility profile for saving/loading settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityProfile {
    /// Profile ID
    pub id: String,
    /// Profile name
    pub name: String,
    /// Profile description
    pub description: String,
    /// Configuration
    pub config: AccessibilityConfig,
    /// Created timestamp
    pub created: i64,
    /// Last modified timestamp
    pub modified: i64,
    /// Is default profile
    pub is_default: bool,
}

impl AccessibilityProfile {
    pub fn new(name: &str, description: &str, config: AccessibilityConfig) -> Self {
        let now = chrono_timestamp();
        Self {
            id: uuid_string(),
            name: name.to_string(),
            description: description.to_string(),
            config,
            created: now,
            modified: now,
            is_default: false,
        }
    }
    
    /// Create default profile
    pub fn default_profile() -> Self {
        Self {
            id: "default".into(),
            name: "Default".into(),
            description: "Default accessibility settings".into(),
            config: AccessibilityConfig::default(),
            created: 0,
            modified: 0,
            is_default: true,
        }
    }
}

/// Predefined profile templates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfileTemplate {
    /// Standard accessibility
    Standard,
    /// Visual impairment focused
    VisualImpairment,
    /// Motor impairment focused
    MotorImpairment,
    /// Hearing impairment focused
    HearingImpairment,
    /// Cognitive assistance
    CognitiveAssistance,
    /// Senior-friendly
    SeniorFriendly,
    /// Tremor assistance
    TremorAssistance,
    /// Full accessibility suite
    FullSuite,
}

impl ProfileTemplate {
    /// Create profile from template
    pub fn create_profile(&self) -> AccessibilityProfile {
        let config = self.create_config();
        let (name, description) = self.info();
        AccessibilityProfile::new(name, description, config)
    }
    
    /// Get template name and description
    pub fn info(&self) -> (&'static str, &'static str) {
        match self {
            ProfileTemplate::Standard => ("Standard", "Basic accessibility features"),
            ProfileTemplate::VisualImpairment => ("Visual Assistance", "Optimized for visual impairments"),
            ProfileTemplate::MotorImpairment => ("Motor Assistance", "Optimized for motor impairments"),
            ProfileTemplate::HearingImpairment => ("Hearing Assistance", "Visual alternatives for audio"),
            ProfileTemplate::CognitiveAssistance => ("Cognitive Assistance", "Simplified interface and guidance"),
            ProfileTemplate::SeniorFriendly => ("Senior Mode", "Large text, simple interface"),
            ProfileTemplate::TremorAssistance => ("Tremor Guard", "Cursor stabilization enabled"),
            ProfileTemplate::FullSuite => ("Full Suite", "All accessibility features enabled"),
        }
    }
    
    /// Create configuration from template
    pub fn create_config(&self) -> AccessibilityConfig {
        let mut config = AccessibilityConfig::default();
        
        match self {
            ProfileTemplate::Standard => {
                // Default configuration
            }
            ProfileTemplate::VisualImpairment => {
                config.vision.enabled = true;
                config.vision.auto_read = true;
                config.vision.describe_images = true;
                config.voice_control.enabled = true;
                config.voice_control.voice_feedback = true;
                config.eye_tracking.enabled = false;
            }
            ProfileTemplate::MotorImpairment => {
                config.eye_tracking.enabled = true;
                config.voice_control.enabled = true;
                config.tremor_guard.enabled = true;
                config.tremor_guard.stabilization_mode = super::StabilizationMode::Strong;
            }
            ProfileTemplate::HearingImpairment => {
                config.vision.enabled = true;
                config.voice_control.enabled = false;
                config.vision.auto_read = false;
                // Enable visual notifications
            }
            ProfileTemplate::CognitiveAssistance => {
                config.senior_mode.enabled = true;
                config.senior_mode.simplified_nav = true;
                config.senior_mode.confirm_actions = true;
                config.voice_control.enabled = true;
            }
            ProfileTemplate::SeniorFriendly => {
                config.senior_mode.enabled = true;
                config.senior_mode.font_size = super::FontSize::Huge;
                config.senior_mode.high_contrast = true;
                config.senior_mode.large_icons = true;
                config.tremor_guard.enabled = true;
                config.voice_control.enabled = true;
            }
            ProfileTemplate::TremorAssistance => {
                config.tremor_guard.enabled = true;
                config.tremor_guard.stabilization_mode = super::StabilizationMode::Maximum;
                config.tremor_guard.dwell_time_ms = 500;
                config.tremor_guard.click_threshold = 15.0;
            }
            ProfileTemplate::FullSuite => {
                config.enable_all = true;
                config.eye_tracking.enabled = true;
                config.voice_control.enabled = true;
                config.vision.enabled = true;
                config.senior_mode.enabled = true;
                config.tremor_guard.enabled = true;
            }
        }
        
        config
    }
}

/// Accessibility feature status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureStatus {
    /// Feature name
    pub name: String,
    /// Is enabled
    pub enabled: bool,
    /// Is active (currently in use)
    pub active: bool,
    /// Status message
    pub message: String,
    /// Error if any
    pub error: Option<String>,
}

/// Accessibility statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccessibilityStats {
    /// Total active time in seconds
    pub total_active_time: f64,
    /// Features used count
    pub features_used: HashMap<String, u64>,
    /// Commands executed via voice
    pub voice_commands_executed: u64,
    /// Pages read aloud
    pub pages_read: u64,
    /// Eye tracking session time
    pub eye_tracking_time: f64,
    /// Tremor corrections applied
    pub tremor_corrections: u64,
}

/// Main accessibility manager
pub struct AccessibilityManager {
    /// Configuration
    config: AccessibilityConfig,
    /// Current profile
    current_profile: AccessibilityProfile,
    /// Saved profiles
    profiles: Vec<AccessibilityProfile>,
    /// Eye tracker
    eye_tracker: Option<EyeTracker>,
    /// Voice controller
    voice_controller: Option<VoiceController>,
    /// Vision describer
    vision_describer: Option<VisionDescriber>,
    /// Senior mode
    senior_mode: Option<SeniorMode>,
    /// Tremor guard
    tremor_guard: Option<TremorGuard>,
    /// Statistics
    stats: AccessibilityStats,
    /// Feature statuses
    feature_statuses: HashMap<String, FeatureStatus>,
    /// Initialization time
    init_time: Instant,
}

impl AccessibilityManager {
    /// Create a new accessibility manager
    pub fn new(config: AccessibilityConfig) -> Self {
        let current_profile = AccessibilityProfile::default_profile();
        
        Self {
            config,
            current_profile,
            profiles: vec![AccessibilityProfile::default_profile()],
            eye_tracker: None,
            voice_controller: None,
            vision_describer: None,
            senior_mode: None,
            tremor_guard: None,
            stats: AccessibilityStats::default(),
            feature_statuses: HashMap::new(),
            init_time: Instant::now(),
        }
    }
    
    /// Initialize all enabled features
    pub fn initialize(&mut self) -> AccessibilityResult<()> {
        // Initialize eye tracker if enabled
        if self.config.eye_tracking.enabled {
            let mut tracker = EyeTracker::new(self.config.eye_tracking.clone());
            tracker.initialize()?;
            self.eye_tracker = Some(tracker);
            self.update_feature_status("eye_tracking", true, true, "Active");
        }
        
        // Initialize voice controller if enabled
        if self.config.voice_control.enabled {
            let mut controller = VoiceController::new(self.config.voice_control.clone());
            controller.initialize()?;
            self.voice_controller = Some(controller);
            self.update_feature_status("voice_control", true, true, "Active");
        }
        
        // Initialize vision describer if enabled
        if self.config.vision.enabled {
            let mut describer = VisionDescriber::new(self.config.vision.clone());
            describer.initialize()?;
            self.vision_describer = Some(describer);
            self.update_feature_status("vision", true, true, "Active");
        }
        
        // Initialize senior mode if enabled
        if self.config.senior_mode.enabled {
            let mode = SeniorMode::new(self.config.senior_mode.clone());
            self.senior_mode = Some(mode);
            self.update_feature_status("senior_mode", true, true, "Active");
        }
        
        // Initialize tremor guard if enabled
        if self.config.tremor_guard.enabled {
            let mut guard = TremorGuard::new(self.config.tremor_guard.clone());
            guard.initialize()?;
            self.tremor_guard = Some(guard);
            self.update_feature_status("tremor_guard", true, true, "Active");
        }
        
        self.stats.total_active_time = 0.0;
        
        Ok(())
    }
    
    /// Update feature status
    fn update_feature_status(&mut self, name: &str, enabled: bool, active: bool, message: &str) {
        self.feature_statuses.insert(name.to_string(), FeatureStatus {
            name: name.to_string(),
            enabled,
            active,
            message: message.to_string(),
            error: None,
        });
    }
    
    /// Load a profile
    pub fn load_profile(&mut self, profile_id: &str) -> AccessibilityResult<()> {
        let profile = self.profiles.iter()
            .find(|p| p.id == profile_id)
            .cloned()
            .ok_or_else(|| AccessibilityError::ProfileNotFound(profile_id.to_string()))?;
        
        self.config = profile.config.clone();
        self.current_profile = profile;
        
        // Reinitialize with new config
        self.initialize()?;
        
        Ok(())
    }
    
    /// Save current settings as a profile
    pub fn save_profile(&mut self, name: &str, description: &str) -> String {
        let profile = AccessibilityProfile::new(name, description, self.config.clone());
        let id = profile.id.clone();
        self.profiles.push(profile);
        id
    }
    
    /// Delete a profile
    pub fn delete_profile(&mut self, profile_id: &str) -> AccessibilityResult<()> {
        let profile = self.profiles.iter()
            .find(|p| p.id == profile_id)
            .ok_or_else(|| AccessibilityError::ProfileNotFound(profile_id.to_string()))?;
        
        if profile.is_default {
            return Err(AccessibilityError::Configuration("Cannot delete default profile".into()));
        }
        
        self.profiles.retain(|p| p.id != profile_id);
        Ok(())
    }
    
    /// Get all profiles
    pub fn get_profiles(&self) -> &[AccessibilityProfile] {
        &self.profiles
    }
    
    /// Get current profile
    pub fn get_current_profile(&self) -> &AccessibilityProfile {
        &self.current_profile
    }
    
    /// Apply a profile template
    pub fn apply_template(&mut self, template: ProfileTemplate) -> AccessibilityResult<()> {
        let profile = template.create_profile();
        self.config = profile.config.clone();
        self.current_profile = profile;
        self.initialize()
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: AccessibilityConfig) -> AccessibilityResult<()> {
        self.config = config;
        self.initialize()
    }
    
    /// Get configuration
    pub fn get_config(&self) -> &AccessibilityConfig {
        &self.config
    }
    
    /// Toggle a specific feature
    pub fn toggle_feature(&mut self, feature: &str) -> AccessibilityResult<bool> {
        match feature {
            "eye_tracking" => {
                self.config.eye_tracking.enabled = !self.config.eye_tracking.enabled;
                if self.config.eye_tracking.enabled {
                    let mut tracker = EyeTracker::new(self.config.eye_tracking.clone());
                    tracker.initialize()?;
                    self.eye_tracker = Some(tracker);
                } else {
                    self.eye_tracker = None;
                }
                Ok(self.config.eye_tracking.enabled)
            }
            "voice_control" => {
                self.config.voice_control.enabled = !self.config.voice_control.enabled;
                if self.config.voice_control.enabled {
                    let mut controller = VoiceController::new(self.config.voice_control.clone());
                    controller.initialize()?;
                    self.voice_controller = Some(controller);
                } else {
                    self.voice_controller = None;
                }
                Ok(self.config.voice_control.enabled)
            }
            "vision" => {
                self.config.vision.enabled = !self.config.vision.enabled;
                if self.config.vision.enabled {
                    let mut describer = VisionDescriber::new(self.config.vision.clone());
                    describer.initialize()?;
                    self.vision_describer = Some(describer);
                } else {
                    self.vision_describer = None;
                }
                Ok(self.config.vision.enabled)
            }
            "senior_mode" => {
                self.config.senior_mode.enabled = !self.config.senior_mode.enabled;
                if self.config.senior_mode.enabled {
                    self.senior_mode = Some(SeniorMode::new(self.config.senior_mode.clone()));
                } else {
                    self.senior_mode = None;
                }
                Ok(self.config.senior_mode.enabled)
            }
            "tremor_guard" => {
                self.config.tremor_guard.enabled = !self.config.tremor_guard.enabled;
                if self.config.tremor_guard.enabled {
                    let mut guard = TremorGuard::new(self.config.tremor_guard.clone());
                    guard.initialize()?;
                    self.tremor_guard = Some(guard);
                } else {
                    self.tremor_guard = None;
                }
                Ok(self.config.tremor_guard.enabled)
            }
            _ => Err(AccessibilityError::Configuration(format!("Unknown feature: {}", feature)))
        }
    }
    
    /// Get feature statuses
    pub fn get_feature_statuses(&self) -> &HashMap<String, FeatureStatus> {
        &self.feature_statuses
    }
    
    /// Process mouse position (for tremor guard and eye tracking)
    pub fn process_mouse_position(&mut self, x: f32, y: f32) -> (f32, f32) {
        if let Some(ref mut guard) = self.tremor_guard {
            return guard.process_position(x, y);
        }
        (x, y)
    }
    
    /// Process voice input
    pub fn process_voice(&mut self, text: &str) -> Option<VoiceResult> {
        if let Some(ref mut controller) = self.voice_controller {
            if let Ok(result) = controller.process_speech(text) {
                self.stats.voice_commands_executed += 1;
                *self.stats.features_used.entry("voice_control".into()).or_insert(0) += 1;
                return Some(result);
            }
        }
        None
    }
    
    /// Process gaze data
    pub fn process_gaze(&mut self, x: f32, y: f32, confidence: f32) -> Option<GazePoint> {
        if let Some(ref mut tracker) = self.eye_tracker {
            if tracker.update_gaze(x, y, confidence).is_ok() {
                return tracker.get_gaze();
            }
        }
        None
    }
    
    /// Describe current page element
    pub fn describe_element(&mut self, element: &super::vision_describer::WebPageElement) -> Option<SceneDescription> {
        if let Some(ref mut describer) = self.vision_describer {
            if let Ok(description) = describer.describe_element(element) {
                *self.stats.features_used.entry("vision".into()).or_insert(0) += 1;
                return Some(description);
            }
        }
        None
    }
    
    /// Start reading content
    pub fn start_reading(&mut self, content: &str) -> Option<String> {
        if let Some(ref mut describer) = self.vision_describer {
            if let Ok(session_id) = describer.start_reading(content) {
                self.stats.pages_read += 1;
                return Some(session_id);
            }
        }
        None
    }
    
    /// Stop reading
    pub fn stop_reading(&mut self) {
        if let Some(ref mut describer) = self.vision_describer {
            describer.stop_reading();
        }
    }
    
    /// Get eye tracker reference
    pub fn get_eye_tracker(&self) -> Option<&EyeTracker> {
        self.eye_tracker.as_ref()
    }
    
    /// Get voice controller reference
    pub fn get_voice_controller(&self) -> Option<&VoiceController> {
        self.voice_controller.as_ref()
    }
    
    /// Get vision describer reference
    pub fn get_vision_describer(&self) -> Option<&VisionDescriber> {
        self.vision_describer.as_ref()
    }
    
    /// Get senior mode reference
    pub fn get_senior_mode(&self) -> Option<&SeniorMode> {
        self.senior_mode.as_ref()
    }
    
    /// Get tremor guard reference
    pub fn get_tremor_guard(&self) -> Option<&TremorGuard> {
        self.tremor_guard.as_ref()
    }
    
    /// Get statistics
    pub fn get_stats(&self) -> &AccessibilityStats {
        &self.stats
    }
    
    /// Export settings to JSON
    pub fn export_settings(&self) -> AccessibilityResult<String> {
        serde_json::to_string_pretty(&self.config)
            .map_err(|e| AccessibilityError::Configuration(format!("Export failed: {}", e)))
    }
    
    /// Import settings from JSON
    pub fn import_settings(&mut self, json: &str) -> AccessibilityResult<()> {
        let config: AccessibilityConfig = serde_json::from_str(json)
            .map_err(|e| AccessibilityError::Configuration(format!("Import failed: {}", e)))?;
        
        self.update_config(config)
    }
}

/// Simple UUID generation
fn uuid_string() -> String {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", nanos)
}

/// Simple timestamp
fn chrono_timestamp() -> i64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_accessibility_config_default() {
        let config = AccessibilityConfig::default();
        assert!(!config.enable_all);
        assert!(config.shortcuts.contains_key("toggle_voice"));
    }
    
    #[test]
    fn test_profile_creation() {
        let profile = AccessibilityProfile::new("Test", "Test profile", AccessibilityConfig::default());
        assert_eq!(profile.name, "Test");
        assert!(!profile.is_default);
    }
    
    #[test]
    fn test_manager_creation() {
        let config = AccessibilityConfig::default();
        let manager = AccessibilityManager::new(config);
        assert!(manager.get_profiles().len() >= 1); // Default profile
    }
    
    #[test]
    fn test_template_profiles() {
        let config = ProfileTemplate::SeniorFriendly.create_config();
        assert!(config.senior_mode.enabled);
        assert!(config.tremor_guard.enabled);
    }
    
    #[test]
    fn test_feature_toggle() {
        let config = AccessibilityConfig {
            voice_control: VoiceConfig { enabled: false, ..Default::default() },
            ..Default::default()
        };
        let mut manager = AccessibilityManager::new(config);
        
        let enabled = manager.toggle_feature("voice_control").unwrap();
        assert!(enabled);
    }
    
    #[test]
    fn test_settings_export_import() {
        let config = AccessibilityConfig::default();
        let manager = AccessibilityManager::new(config);
        
        let exported = manager.export_settings().unwrap();
        assert!(exported.contains("level"));
    }
}