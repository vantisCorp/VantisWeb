//! Vantis Vitality - Health & Wellness Module
//! 
//! This module provides comprehensive health and wellness features:
//! - Nutri-Scanner AI: Recipe analysis and nutritional information
//! - Ingred-X: Food additive detection and safety ratings
//! - Visual Calorie Counter: Photo-based food recognition
//! - Bio-Sync: Health monitoring and wellness tracking

pub mod nutri_scanner;
pub mod ingred_x;
pub mod visual_calorie;
pub mod bio_sync;
pub mod database;
pub mod models;

pub use nutri_scanner::NutriScanner;
pub use ingred_x::IngredX;
pub use visual_calorie::VisualCalorieCounter;
pub use bio_sync::BioSync;

use serde::{Deserialize, Serialize};

/// Main health module manager
pub struct HealthModule {
    nutri_scanner: NutriScanner,
    ingred_x: IngredX,
    visual_calorie: VisualCalorieCounter,
    bio_sync: BioSync,
    enabled: bool,
}

impl HealthModule {
    /// Create a new health module instance
    pub fn new() -> Self {
        Self {
            nutri_scanner: NutriScanner::new(),
            ingred_x: IngredX::new(),
            visual_calorie: VisualCalorieCounter::new(),
            bio_sync: BioSync::new(),
            enabled: true,
        }
    }

    /// Get reference to Nutri-Scanner
    pub fn nutri_scanner(&self) -> &NutriScanner {
        &self.nutri_scanner
    }

    /// Get mutable reference to Nutri-Scanner
    pub fn nutri_scanner_mut(&mut self) -> &mut NutriScanner {
        &mut self.nutri_scanner
    }

    /// Get reference to Ingred-X
    pub fn ingred_x(&self) -> &IngredX {
        &self.ingred_x
    }

    /// Get mutable reference to Ingred-X
    pub fn ingred_x_mut(&mut self) -> &mut IngredX {
        &mut self.ingred_x
    }

    /// Get reference to Visual Calorie Counter
    pub fn visual_calorie(&self) -> &VisualCalorieCounter {
        &self.visual_calorie
    }

    /// Get mutable reference to Visual Calorie Counter
    pub fn visual_calorie_mut(&mut self) -> &mut VisualCalorieCounter {
        &mut self.visual_calorie
    }

    /// Get reference to Bio-Sync
    pub fn bio_sync(&self) -> &BioSync {
        &self.bio_sync
    }

    /// Get mutable reference to Bio-Sync
    pub fn bio_sync_mut(&mut self) -> &mut BioSync {
        &mut self.bio_sync
    }

    /// Check if health module is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enable or disable the health module
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Get daily health summary
    pub fn get_daily_summary(&self) -> DailyHealthSummary {
        DailyHealthSummary {
            calories_consumed: self.visual_calorie.get_daily_calories(),
            calories_remaining: self.visual_calorie.get_remaining_calories(),
            protein_intake: self.visual_calorie.get_daily_protein(),
            carbs_intake: self.visual_calorie.get_daily_carbs(),
            fat_intake: self.visual_calorie.get_daily_fat(),
            water_glasses: self.bio_sync.get_water_intake(),
            screen_time_minutes: self.bio_sync.get_screen_time(),
            break_count: self.bio_sync.get_break_count(),
            health_score: self.calculate_health_score(),
        }
    }

    /// Calculate overall health score (0-100)
    fn calculate_health_score(&self) -> u8 {
        let mut score = 100u8;
        
        // Deduct for excessive screen time
        let screen_time = self.bio_sync.get_screen_time();
        if screen_time > 480 {
            score = score.saturating_sub(((screen_time - 480) / 60) as u8);
        }
        
        // Deduct for missing breaks
        let expected_breaks = screen_time / 30;
        let actual_breaks = self.bio_sync.get_break_count();
        if actual_breaks < expected_breaks {
            score = score.saturating_sub((expected_breaks - actual_breaks) as u8 * 2);
        }
        
        // Bonus for meeting calorie goals
        let calories = self.visual_calorie.get_daily_calories();
        let target = self.visual_calorie.get_target_calories();
        if calories > 0 && (calories as i32 - target as i32).abs() < 200 {
            score = score.saturating_add(5);
        }
        
        score.min(100)
    }
}

impl Default for HealthModule {
    fn default() -> Self {
        Self::new()
    }
}

/// Daily health summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyHealthSummary {
    /// Total calories consumed today
    pub calories_consumed: u32,
    /// Calories remaining for daily goal
    pub calories_remaining: i32,
    /// Protein intake in grams
    pub protein_intake: f32,
    /// Carbohydrate intake in grams
    pub carbs_intake: f32,
    /// Fat intake in grams
    pub fat_intake: f32,
    /// Number of water glasses consumed
    pub water_glasses: u8,
    /// Screen time in minutes
    pub screen_time_minutes: u32,
    /// Number of breaks taken
    pub break_count: u32,
    /// Overall health score (0-100)
    pub health_score: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_module_creation() {
        let module = HealthModule::new();
        assert!(module.is_enabled());
    }

    #[test]
    fn test_daily_summary() {
        let module = HealthModule::new();
        let summary = module.get_daily_summary();
        assert_eq!(summary.calories_consumed, 0);
        assert!(summary.health_score <= 100);
    }
}