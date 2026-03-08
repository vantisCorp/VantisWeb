//! Bio-Sync - Health monitoring and wellness tracking
//! 
//! This module provides blue light reduction, circadian rhythm sync,
//! break reminders, and comprehensive health monitoring features.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local, Timelike, Utc};

/// Bio-Sync health monitor
pub struct BioSync {
    /// Blue light filter settings
    blue_light_filter: BlueLightFilter,
    /// Circadian rhythm settings
    circadian_settings: CircadianSettings,
    /// Break reminder system
    break_system: BreakReminderSystem,
    /// Health metrics tracker
    metrics: HealthMetricsTracker,
    /// Daily screen time log
    screen_time_log: Vec<ScreenTimeEntry>,
    /// Water intake log
    water_intake: Vec<WaterEntry>,
    /// Whether monitoring is active
    active: bool,
}

impl BioSync {
    /// Create a new Bio-Sync instance
    pub fn new() -> Self {
        Self {
            blue_light_filter: BlueLightFilter::new(),
            circadian_settings: CircadianSettings::new(),
            break_system: BreakReminderSystem::new(),
            metrics: HealthMetricsTracker::new(),
            screen_time_log: Vec::new(),
            water_intake: Vec::new(),
            active: true,
        }
    }

    // === Blue Light Filter ===

    /// Get current blue light filter settings
    pub fn get_blue_light_settings(&self) -> &BlueLightFilter {
        &self.blue_light_filter
    }

    /// Update blue light filter settings
    pub fn update_blue_light(&mut self, settings: BlueLightFilter) {
        self.blue_light_filter = settings;
    }

    /// Get recommended color temperature for current time
    pub fn get_recommended_color_temp(&self) -> u16 {
        let hour = Local::now().hour();
        self.blue_light_filter.get_recommended_temp(hour)
    }

    /// Check if blue light filter should be active
    pub fn should_filter_blue_light(&self) -> bool {
        let hour = Local::now().hour();
        self.blue_light_filter.should_be_active(hour)
    }

    // === Circadian Rhythm ===

    /// Get circadian rhythm settings
    pub fn get_circadian_settings(&self) -> &CircadianSettings {
        &self.circadian_settings
    }

    /// Update circadian rhythm settings
    pub fn update_circadian_settings(&mut self, settings: CircadianSettings) {
        self.circadian_settings = settings;
    }

    /// Get current chronotype recommendation
    pub fn get_chronotype_recommendation(&self) -> ChronotypeRecommendation {
        self.circadian_settings.get_recommendation()
    }

    /// Get optimal sleep time
    pub fn get_optimal_sleep_time(&self) -> DateTime<Local> {
        self.circadian_settings.calculate_optimal_sleep_time()
    }

    /// Get optimal wake time
    pub fn get_optimal_wake_time(&self) -> DateTime<Local> {
        self.circadian_settings.calculate_optimal_wake_time()
    }

    /// Check if it's within optimal activity hours
    pub fn is_optimal_activity_time(&self) -> bool {
        let hour = Local::now().hour();
        self.circadian_settings.is_optimal_activity_hour(hour)
    }

    // === Break Reminders ===

    /// Get break reminder settings
    pub fn get_break_settings(&self) -> &BreakReminderSettings {
        &self.break_system.settings
    }

    /// Update break reminder settings
    pub fn update_break_settings(&mut self, settings: BreakReminderSettings) {
        self.break_system.settings = settings;
    }

    /// Check if a break is due
    pub fn is_break_due(&self) -> bool {
        self.break_system.is_break_due()
    }

    /// Get next break time in minutes
    pub fn get_next_break_in(&self) -> u32 {
        self.break_system.minutes_until_next_break()
    }

    /// Record that a break was taken
    pub fn record_break(&mut self, break_type: BreakType, duration_minutes: u8) {
        self.break_system.record_break(break_type, duration_minutes);
    }

    /// Skip current break
    pub fn skip_break(&mut self) {
        self.break_system.skip_break();
    }

    /// Get break count for today
    pub fn get_break_count(&self) -> u32 {
        self.break_system.get_today_break_count()
    }

    /// Get break streak (consecutive days with proper breaks)
    pub fn get_break_streak(&self) -> u32 {
        self.break_system.get_streak()
    }

    // === Screen Time Tracking ===

    /// Start screen time session
    pub fn start_session(&mut self, domain: Option<String>) {
        let entry = ScreenTimeEntry {
            start: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            end: 0,
            domain,
            category: None,
        };
        self.screen_time_log.push(entry);
    }

    /// End current screen time session
    pub fn end_session(&mut self) {
        if let Some(entry) = self.screen_time_log.last_mut() {
            if entry.end == 0 {
                entry.end = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
            }
        }
    }

    /// Get total screen time today in minutes
    pub fn get_screen_time(&self) -> u32 {
        let today_start = self.get_today_start();
        self.screen_time_log.iter()
            .filter(|e| e.start >= today_start)
            .map(|e| {
                let end = if e.end == 0 {
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                } else {
                    e.end
                };
                ((end - e.start) / 60) as u32
            })
            .sum()
    }

    /// Get screen time by category
    pub fn get_screen_time_by_category(&self) -> HashMap<String, u32> {
        let mut result = HashMap::new();
        let today_start = self.get_today_start();
        
        for entry in self.screen_time_log.iter().filter(|e| e.start >= today_start) {
            let category = entry.category.clone().unwrap_or_else(|| "Other".to_string());
            let end = if entry.end == 0 {
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            } else {
                entry.end
            };
            let minutes = ((end - entry.start) / 60) as u32;
            *result.entry(category).or_insert(0) += minutes;
        }
        
        result
    }

    // === Water Intake ===

    /// Log water intake
    pub fn log_water(&mut self, glasses: u8) {
        let entry = WaterEntry {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            glasses,
        };
        self.water_intake.push(entry);
    }

    /// Get total water intake today (glasses)
    pub fn get_water_intake(&self) -> u8 {
        let today_start = self.get_today_start();
        self.water_intake.iter()
            .filter(|e| e.timestamp >= today_start)
            .map(|e| e.glasses)
            .sum()
    }

    /// Check if hydration goal is met
    pub fn is_hydration_goal_met(&self) -> bool {
        self.get_water_intake() >= 8
    }

    /// Get hydration reminder message
    pub fn get_hydration_reminder(&self) -> Option<String> {
        let glasses = self.get_water_intake();
        if glasses < 8 {
            Some(format!(
                "You've had {} glasses of water today. Aim for 8 glasses!",
                glasses
            ))
        } else {
            None
        }
    }

    // === Health Metrics ===

    /// Record daily health metrics
    pub fn record_metrics(&mut self, metrics: DailyMetricsInput) {
        self.metrics.record(metrics);
    }

    /// Get health trends
    pub fn get_health_trends(&self, days: u8) -> HealthTrends {
        self.metrics.get_trends(days)
    }

    /// Get wellness score (0-100)
    pub fn get_wellness_score(&self) -> u8 {
        let mut score = 100u8;
        
        // Penalize for excessive screen time
        let screen_time = self.get_screen_time();
        if screen_time > 480 {
            score = score.saturating_sub(((screen_time - 480) / 60) as u8);
        }
        
        // Penalize for missing breaks
        let expected_breaks = screen_time / 30;
        let actual_breaks = self.get_break_count();
        if actual_breaks < expected_breaks {
            score = score.saturating_sub((expected_breaks - actual_breaks) as u8 * 2);
        }
        
        // Penalize for low hydration
        let water = self.get_water_intake();
        if water < 4 {
            score = score.saturating_sub((4 - water) * 3);
        }
        
        // Bonus for meeting hydration goal
        if water >= 8 {
            score = score.saturating_add(5);
        }
        
        score.min(100)
    }

    // === General ===

    /// Check if monitoring is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Toggle monitoring
    pub fn toggle(&mut self) {
        self.active = !self.active;
    }

    /// Set monitoring state
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    /// Get today's start timestamp
    fn get_today_start(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now - (now % 86400)
    }

    /// Get comprehensive health status
    pub fn get_health_status(&self) -> HealthStatus {
        HealthStatus {
            screen_time_minutes: self.get_screen_time(),
            breaks_taken: self.get_break_count(),
            water_glasses: self.get_water_intake(),
            wellness_score: self.get_wellness_score(),
            break_streak: self.get_break_streak(),
            is_break_due: self.is_break_due(),
            next_break_minutes: self.get_next_break_in(),
            blue_light_active: self.should_filter_blue_light(),
            optimal_activity_time: self.is_optimal_activity_time(),
            hydration_reminder: self.get_hydration_reminder(),
        }
    }
}

impl Default for BioSync {
    fn default() -> Self {
        Self::new()
    }
}

/// Blue light filter settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueLightFilter {
    /// Whether the filter is enabled
    pub enabled: bool,
    /// Start hour (24h format)
    pub start_hour: u8,
    /// End hour (24h format)
    pub end_hour: u8,
    /// Color temperature (K)
    pub color_temperature: u16,
    /// Auto-adjust based on sunset
    pub auto_sunset: bool,
    /// Intensity (0-100)
    pub intensity: u8,
}

impl BlueLightFilter {
    pub fn new() -> Self {
        Self {
            enabled: true,
            start_hour: 19,  // 7 PM
            end_hour: 7,     // 7 AM
            color_temperature: 3400, // Warmer color
            auto_sunset: true,
            intensity: 80,
        }
    }

    /// Get recommended color temperature for given hour
    pub fn get_recommended_temp(&self, hour: u8) -> u16 {
        if self.should_be_active(hour) {
            self.color_temperature
        } else {
            6500 // Standard daylight
        }
    }

    /// Check if filter should be active at given hour
    pub fn should_be_active(&self, hour: u8) -> bool {
        if !self.enabled {
            return false;
        }
        
        if self.start_hour > self.end_hour {
            // Spans midnight (e.g., 19:00 to 07:00)
            hour >= self.start_hour || hour < self.end_hour
        } else {
            // Same day (e.g., 12:00 to 18:00)
            hour >= self.start_hour && hour < self.end_hour
        }
    }
}

impl Default for BlueLightFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Circadian rhythm settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircadianSettings {
    /// User's chronotype (morning/evening preference)
    pub chronotype: Chronotype,
    /// Target sleep duration in hours
    pub sleep_duration: u8,
    /// Wake time preference (hour)
    pub preferred_wake_hour: u8,
    /// Sleep time preference (hour)
    pub preferred_sleep_hour: u8,
    /// Weekend adjustment
    pub weekend_shift_hours: i8,
}

impl CircadianSettings {
    pub fn new() -> Self {
        Self {
            chronotype: Chronotype::Intermediate,
            sleep_duration: 8,
            preferred_wake_hour: 7,
            preferred_sleep_hour: 23,
            weekend_shift_hours: 1,
        }
    }

    /// Get chronotype recommendation
    pub fn get_recommendation(&self) -> ChronotypeRecommendation {
        match self.chronotype {
            Chronotype::MorningLark => ChronotypeRecommendation {
                optimal_activity_start: 6,
                peak_performance: 10,
                wind_down_start: 20,
                description: "You're naturally inclined to wake early and be most productive in the morning.".to_string(),
            },
            Chronotype::Intermediate => ChronotypeRecommendation {
                optimal_activity_start: 8,
                peak_performance: 12,
                wind_down_start: 21,
                description: "You have a balanced sleep-wake preference.".to_string(),
            },
            Chronotype::NightOwl => ChronotypeRecommendation {
                optimal_activity_start: 10,
                peak_performance: 16,
                wind_down_start: 23,
                description: "You're naturally inclined to stay up late and be most productive in the evening.".to_string(),
            },
        }
    }

    /// Calculate optimal sleep time
    pub fn calculate_optimal_sleep_time(&self) -> DateTime<Local> {
        let now = Local::now();
        let target_hour = self.preferred_sleep_hour;
        
        let mut target = now.date_naive()
            .and_hms_opt(target_hour as u32, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap();
        
        if target < now {
            target = target + chrono::Duration::days(1);
        }
        
        target
    }

    /// Calculate optimal wake time
    pub fn calculate_optimal_wake_time(&self) -> DateTime<Local> {
        let now = Local::now();
        let target_hour = self.preferred_wake_hour;
        
        let mut target = now.date_naive()
            .and_hms_opt(target_hour as u32, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap();
        
        if target < now {
            target = target + chrono::Duration::days(1);
        }
        
        target
    }

    /// Check if given hour is optimal for activity
    pub fn is_optimal_activity_hour(&self, hour: u8) -> bool {
        let rec = self.get_recommendation();
        hour >= rec.optimal_activity_start && hour < rec.wind_down_start
    }
}

impl Default for CircadianSettings {
    fn default() -> Self {
        Self::new()
    }
}

/// Chronotype classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Chronotype {
    MorningLark,
    Intermediate,
    NightOwl,
}

/// Chronotype recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronotypeRecommendation {
    pub optimal_activity_start: u8,
    pub peak_performance: u8,
    pub wind_down_start: u8,
    pub description: String,
}

/// Break reminder system
#[derive(Debug, Clone)]
pub struct BreakReminderSystem {
    pub settings: BreakReminderSettings,
    breaks_today: Vec<BreakRecord>,
    last_break_time: Option<u64>,
    skip_count: u8,
    streak_data: HashMap<String, u32>,
}

impl BreakReminderSystem {
    pub fn new() -> Self {
        Self {
            settings: BreakReminderSettings::new(),
            breaks_today: Vec::new(),
            last_break_time: None,
            skip_count: 0,
            streak_data: HashMap::new(),
        }
    }

    /// Check if a break is due
    pub fn is_break_due(&self) -> bool {
        if !self.settings.enabled {
            return false;
        }
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let minutes_since_last = match self.last_break_time {
            Some(last) => ((now - last) / 60) as u32,
            None => return true,
        };
        
        minutes_since_last >= self.settings.interval_minutes as u32
    }

    /// Get minutes until next break
    pub fn minutes_until_next_break(&self) -> u32 {
        if !self.settings.enabled {
            return u32::MAX;
        }
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let minutes_since_last = match self.last_break_time {
            Some(last) => ((now - last) / 60) as u32,
            None => return 0,
        };
        
        let remaining = self.settings.interval_minutes as u32 - minutes_since_last;
        remaining.max(0)
    }

    /// Record a break
    pub fn record_break(&mut self, break_type: BreakType, duration: u8) {
        let record = BreakRecord {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            break_type,
            duration,
        };
        
        self.breaks_today.push(record);
        self.last_break_time = Some(SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs());
        self.skip_count = 0;
    }

    /// Skip current break
    pub fn skip_break(&mut self) {
        self.skip_count += 1;
        self.last_break_time = Some(SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs());
        
        // Auto-disable if too many skips
        if self.skip_count >= 3 {
            self.settings.enabled = false;
        }
    }

    /// Get today's break count
    pub fn get_today_break_count(&self) -> u32 {
        self.breaks_today.len() as u32
    }

    /// Get streak
    pub fn get_streak(&self) -> u32 {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        *self.streak_data.get(&today).unwrap_or(&0)
    }
}

impl Default for BreakReminderSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Break reminder settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakReminderSettings {
    pub enabled: bool,
    pub interval_minutes: u16,
    pub break_duration_minutes: u8,
    pub break_types: Vec<BreakType>,
    pub sound_enabled: bool,
    pub strict_mode: bool,
}

impl BreakReminderSettings {
    pub fn new() -> Self {
        Self {
            enabled: true,
            interval_minutes: 30,
            break_duration_minutes: 5,
            break_types: vec![BreakType::EyeRest, BreakType::Stretch, BreakType::Hydration],
            sound_enabled: true,
            strict_mode: false,
        }
    }
}

impl Default for BreakReminderSettings {
    fn default() -> Self {
        Self::new()
    }
}

/// Break record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakRecord {
    pub timestamp: u64,
    pub break_type: BreakType,
    pub duration: u8,
}

/// Water intake entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaterEntry {
    pub timestamp: u64,
    pub glasses: u8,
}

/// Health metrics tracker
#[derive(Debug, Clone, Default)]
pub struct HealthMetricsTracker {
    records: Vec<DailyMetrics>,
}

impl HealthMetricsTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, metrics: DailyMetricsInput) {
        let record = DailyMetrics {
            date: chrono::Local::now().format("%Y-%m-%d").to_string(),
            screen_time: metrics.screen_time,
            breaks_taken: metrics.breaks_taken,
            water_intake: metrics.water_intake,
            steps: metrics.steps,
            sleep_hours: metrics.sleep_hours,
            heart_rate_avg: metrics.heart_rate_avg,
            stress_level: metrics.stress_level,
            energy_level: metrics.energy_level,
        };
        
        self.records.push(record);
    }

    pub fn get_trends(&self, days: u8) -> HealthTrends {
        let recent: Vec<_> = self.records.iter()
            .rev()
            .take(days as usize)
            .collect();
        
        if recent.is_empty() {
            return HealthTrends::default();
        }
        
        let avg_screen_time = recent.iter()
            .map(|r| r.screen_time)
            .sum::<u32>() / recent.len() as u32;
        
        let avg_breaks = recent.iter()
            .map(|r| r.breaks_taken)
            .sum::<u32>() / recent.len() as u32;
        
        let avg_water = recent.iter()
            .map(|r| r.water_intake as u32)
            .sum::<u32>() / recent.len() as u32;
        
        let avg_sleep = recent.iter()
            .filter_map(|r| r.sleep_hours)
            .sum::<f32>() / recent.iter().filter(|r| r.sleep_hours.is_some()).count() as f32;
        
        HealthTrends {
            average_screen_time: avg_screen_time,
            average_breaks_per_day: avg_breaks,
            average_water_intake: avg_water as u8,
            average_sleep_hours: avg_sleep,
            trend_direction: self.calculate_trend_direction(),
        }
    }

    fn calculate_trend_direction(&self) -> TrendDirection {
        if self.records.len() < 2 {
            return TrendDirection::Stable;
        }
        
        let recent: Vec<_> = self.records.iter().rev().take(7).collect();
        if recent.len() < 2 {
            return TrendDirection::Stable;
        }
        
        let recent_avg = recent.iter().take(3).map(|r| r.screen_time).sum::<u32>() / 3;
        let older_avg = recent.iter().skip(3).map(|r| r.screen_time).sum::<u32>() / (recent.len() - 3).max(1) as u32;
        
        if recent_avg < older_avg {
            TrendDirection::Improving
        } else if recent_avg > older_avg {
            TrendDirection::Declining
        } else {
            TrendDirection::Stable
        }
    }
}

/// Daily metrics input
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DailyMetricsInput {
    pub screen_time: u32,
    pub breaks_taken: u32,
    pub water_intake: u8,
    pub steps: Option<u32>,
    pub sleep_hours: Option<f32>,
    pub heart_rate_avg: Option<u8>,
    pub stress_level: Option<u8>,
    pub energy_level: Option<u8>,
}

/// Health trends
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HealthTrends {
    pub average_screen_time: u32,
    pub average_breaks_per_day: u32,
    pub average_water_intake: u8,
    pub average_sleep_hours: f32,
    pub trend_direction: TrendDirection,
}

/// Trend direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Stable,
    Declining,
}

impl Default for TrendDirection {
    fn default() -> Self {
        Self::Stable
    }
}

/// Comprehensive health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub screen_time_minutes: u32,
    pub breaks_taken: u32,
    pub water_glasses: u8,
    pub wellness_score: u8,
    pub break_streak: u32,
    pub is_break_due: bool,
    pub next_break_minutes: u32,
    pub blue_light_active: bool,
    pub optimal_activity_time: bool,
    pub hydration_reminder: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bio_sync_creation() {
        let bio = BioSync::new();
        assert!(bio.is_active());
    }

    #[test]
    fn test_blue_light_filter() {
        let filter = BlueLightFilter::new();
        
        // During night hours
        assert!(filter.should_be_active(22));
        assert!(filter.should_be_active(2));
        
        // During day hours
        assert!(!filter.should_be_active(12));
    }

    #[test]
    fn test_water_tracking() {
        let mut bio = BioSync::new();
        bio.log_water(3);
        bio.log_water(2);
        
        assert_eq!(bio.get_water_intake(), 5);
    }

    #[test]
    fn test_break_system() {
        let mut bio = BioSync::new();
        assert!(bio.is_break_due()); // First break always due
        
        bio.record_break(BreakType::EyeRest, 5);
        assert_eq!(bio.get_break_count(), 1);
    }

    #[test]
    fn test_wellness_score() {
        let mut bio = BioSync::new();
        bio.log_water(8);
        bio.record_break(BreakType::Stretch, 5);
        
        let score = bio.get_wellness_score();
        assert!(score > 50);
    }

    #[test]
    fn test_chronotype_recommendation() {
        let settings = CircadianSettings::new();
        let rec = settings.get_recommendation();
        
        assert!(rec.optimal_activity_start < 12);
        assert!(rec.wind_down_start > 18);
    }
}