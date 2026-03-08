//! Data models for health module
//! 
//! This module contains all the data structures used across
//! the health and wellness features.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Nutritional information for a food item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NutritionInfo {
    /// Food item name
    pub name: String,
    /// Serving size in grams
    pub serving_size: f32,
    /// Calories per serving
    pub calories: u32,
    /// Protein in grams
    pub protein: f32,
    /// Carbohydrates in grams
    pub carbs: f32,
    /// Fat in grams
    pub fat: f32,
    /// Fiber in grams
    pub fiber: f32,
    /// Sugar in grams
    pub sugar: f32,
    /// Sodium in milligrams
    pub sodium: f32,
    /// Cholesterol in milligrams
    pub cholesterol: f32,
    /// Vitamins and minerals
    pub micronutrients: HashMap<String, f32>,
}

impl Default for NutritionInfo {
    fn default() -> Self {
        Self {
            name: String::new(),
            serving_size: 100.0,
            calories: 0,
            protein: 0.0,
            carbs: 0.0,
            fat: 0.0,
            fiber: 0.0,
            sugar: 0.0,
            sodium: 0.0,
            cholesterol: 0.0,
            micronutrients: HashMap::new(),
        }
    }
}

impl NutritionInfo {
    /// Create new nutrition info
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Calculate calories from macros
    pub fn calculate_calories(&self) -> u32 {
        (self.protein * 4.0 + self.carbs * 4.0 + self.fat * 9.0) as u32
    }
}

/// Food additive (E-number) information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdditiveInfo {
    /// E-number (e.g., "E330")
    pub e_number: String,
    /// Common name
    pub name: String,
    /// Category of additive
    pub category: AdditiveCategory,
    /// Safety rating
    pub safety: SafetyRating,
    /// Description
    pub description: String,
    /// Possible allergens
    pub allergens: Vec<String>,
    /// Dietary restrictions
    pub dietary_flags: Vec<DietaryFlag>,
    /// Source (natural/synthetic)
    pub source: AdditiveSource,
}

/// Additive category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdditiveCategory {
    /// Colorants
    Colorant,
    /// Preservatives
    Preservative,
    /// Antioxidants
    Antioxidant,
    /// Emulsifiers
    Emulsifier,
    /// Stabilizers
    Stabilizer,
    /// Sweeteners
    Sweetener,
    /// FlavorEnhancer
    FlavorEnhancer,
    /// Acidity regulators
    AcidityRegulator,
    /// Thickeners
    Thickener,
    /// Other
    Other,
}

/// Safety rating for additives
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafetyRating {
    /// Safe for consumption
    Safe,
    /// Generally safe, some concerns
    Caution,
    /// Avoid if possible
    Avoid,
    /// Banned in some countries
    Banned,
    /// Unknown safety
    Unknown,
}

impl SafetyRating {
    /// Get display color for UI
    pub fn color(&self) -> &str {
        match self {
            SafetyRating::Safe => "#4CAF50",
            SafetyRating::Caution => "#FFC107",
            SafetyRating::Avoid => "#FF9800",
            SafetyRating::Banned => "#F44336",
            SafetyRating::Unknown => "#9E9E9E",
        }
    }
}

/// Dietary flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DietaryFlag {
    /// Suitable for vegetarians
    Vegetarian,
    /// Suitable for vegans
    Vegan,
    /// Gluten-free
    GlutenFree,
    /// Dairy-free
    DairyFree,
    /// Nut-free
    NutFree,
    /// Kosher
    Kosher,
    /// Halal
    Halal,
    /// Low FODMAP
    LowFodmap,
    /// Keto-friendly
    Keto,
    /// Diabetic-friendly
    Diabetic,
}

/// Additive source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdditiveSource {
    /// Natural source
    Natural,
    /// Synthetic source
    Synthetic,
    /// Both natural and synthetic
    Both,
}

/// Food item for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodEntry {
    /// Unique entry ID
    pub id: String,
    /// Food name
    pub name: String,
    /// Nutrition information
    pub nutrition: NutritionInfo,
    /// Portion multiplier
    pub portion: f32,
    /// Meal type
    pub meal_type: MealType,
    /// Timestamp
    pub timestamp: u64,
    /// Source (photo, manual, barcode)
    pub source: EntrySource,
    /// Confidence score (for AI-detected foods)
    pub confidence: Option<f32>,
}

impl FoodEntry {
    /// Create new food entry
    pub fn new(name: impl Into<String>, nutrition: NutritionInfo) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            nutrition,
            portion: 1.0,
            meal_type: MealType::Snack,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            source: EntrySource::Manual,
            confidence: None,
        }
    }

    /// Get actual calories based on portion
    pub fn actual_calories(&self) -> u32 {
        (self.nutrition.calories as f32 * self.portion) as u32
    }
}

/// Meal type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MealType {
    Breakfast,
    Lunch,
    Dinner,
    Snack,
    Brunch,
    Supper,
}

/// Entry source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntrySource {
    Manual,
    Photo,
    Barcode,
    Voice,
    Recipe,
}

/// Recipe for nutritional analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    /// Recipe name
    pub name: String,
    /// List of ingredients with amounts
    pub ingredients: Vec<Ingredient>,
    /// Number of servings
    pub servings: u32,
    /// Cooking instructions
    pub instructions: Vec<String>,
    /// Total nutrition (calculated)
    pub total_nutrition: NutritionInfo,
    /// Per-serving nutrition
    pub per_serving: NutritionInfo,
}

/// Ingredient in a recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ingredient {
    /// Ingredient name
    pub name: String,
    /// Amount
    pub amount: f32,
    /// Unit (g, ml, cups, etc.)
    pub unit: String,
    /// Nutrition info for this amount
    pub nutrition: NutritionInfo,
}

/// Allergen information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllergenInfo {
    /// Allergen name
    pub name: String,
    /// Severity level
    pub severity: AllergenSeverity,
    /// Symptoms
    pub symptoms: Vec<String>,
}

/// Allergen severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AllergenSeverity {
    Mild,
    Moderate,
    Severe,
    LifeThreatening,
}

/// Health profile for user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthProfile {
    /// User's age
    pub age: u8,
    /// User's gender
    pub gender: Gender,
    /// Height in centimeters
    pub height_cm: u16,
    /// Weight in kilograms
    pub weight_kg: f32,
    /// Activity level
    pub activity_level: ActivityLevel,
    /// Dietary restrictions
    pub dietary_restrictions: Vec<DietaryFlag>,
    /// Allergies
    pub allergies: Vec<String>,
    /// Daily calorie target
    pub calorie_target: u32,
    /// Health goals
    pub goals: Vec<HealthGoal>,
}

impl Default for HealthProfile {
    fn default() -> Self {
        Self {
            age: 30,
            gender: Gender::NotSpecified,
            height_cm: 170,
            weight_kg: 70.0,
            activity_level: ActivityLevel::Moderate,
            dietary_restrictions: Vec::new(),
            allergies: Vec::new(),
            calorie_target: 2000,
            goals: Vec::new(),
        }
    }
}

/// Gender
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
    Other,
    NotSpecified,
}

/// Activity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityLevel {
    Sedentary,
    Light,
    Moderate,
    Active,
    VeryActive,
}

impl ActivityLevel {
    /// Get activity multiplier for TDEE calculation
    pub fn multiplier(&self) -> f32 {
        match self {
            ActivityLevel::Sedentary => 1.2,
            ActivityLevel::Light => 1.375,
            ActivityLevel::Moderate => 1.55,
            ActivityLevel::Active => 1.725,
            ActivityLevel::VeryActive => 1.9,
        }
    }
}

/// Health goal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthGoal {
    LoseWeight,
    MaintainWeight,
    GainWeight,
    BuildMuscle,
    ImproveEnergy,
    BetterSleep,
    ReduceStress,
    ImproveFocus,
}

/// Screen time entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenTimeEntry {
    /// Start timestamp
    pub start: u64,
    /// End timestamp
    pub end: u64,
    /// Domain or app
    pub domain: Option<String>,
    /// Category
    pub category: Option<String>,
}

/// Break reminder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakReminder {
    /// Reminder ID
    pub id: String,
    /// Break type
    pub break_type: BreakType,
    /// Scheduled time
    pub scheduled_time: u64,
    /// Was the break taken
    pub taken: bool,
    /// Duration in minutes
    pub duration_minutes: u8,
}

/// Type of break
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BreakType {
    /// Eye rest break
    EyeRest,
    /// Stretch break
    Stretch,
    /// Walking break
    Walk,
    /// Hydration break
    Hydration,
    /// Mindfulness break
    Mindfulness,
}

/// Daily health metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DailyMetrics {
    /// Date (YYYY-MM-DD)
    pub date: String,
    /// Total screen time in minutes
    pub screen_time: u32,
    /// Number of breaks taken
    pub breaks_taken: u32,
    /// Water intake (glasses)
    pub water_intake: u8,
    /// Steps (if available)
    pub steps: Option<u32>,
    /// Sleep hours
    pub sleep_hours: Option<f32>,
    /// Heart rate average
    pub heart_rate_avg: Option<u8>,
    /// Stress level (1-10)
    pub stress_level: Option<u8>,
    /// Energy level (1-10)
    pub energy_level: Option<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nutrition_info_calories() {
        let mut nutrition = NutritionInfo::new("Test Food");
        nutrition.protein = 10.0;
        nutrition.carbs = 20.0;
        nutrition.fat = 5.0;
        
        let calculated = nutrition.calculate_calories();
        assert_eq!(calculated, 165); // 10*4 + 20*4 + 5*9
    }

    #[test]
    fn test_safety_rating_colors() {
        assert_eq!(SafetyRating::Safe.color(), "#4CAF50");
        assert_eq!(SafetyRating::Banned.color(), "#F44336");
    }

    #[test]
    fn test_activity_level_multiplier() {
        assert!((ActivityLevel::Sedentary.multiplier() - 1.2).abs() < 0.01);
        assert!((ActivityLevel::VeryActive.multiplier() - 1.9).abs() < 0.01);
    }
}