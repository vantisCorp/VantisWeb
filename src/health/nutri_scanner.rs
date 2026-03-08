//! Nutri-Scanner AI - Recipe analysis and nutritional information
//! 
//! This module provides AI-powered recipe analysis, calorie counting,
//! nutritional information extraction, and dietary recommendations.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::models::*;
use super::database::FoodDatabase;

/// Nutri-Scanner AI for recipe and nutrition analysis
pub struct NutriScanner {
    database: FoodDatabase,
    profile: HealthProfile,
    meal_history: Vec<FoodEntry>,
    recipes: HashMap<String, Recipe>,
}

impl NutriScanner {
    /// Create a new Nutri-Scanner instance
    pub fn new() -> Self {
        Self {
            database: FoodDatabase::new(),
            profile: HealthProfile::default(),
            meal_history: Vec::new(),
            recipes: HashMap::new(),
        }
    }

    /// Set user health profile
    pub fn set_profile(&mut self, profile: HealthProfile) {
        self.profile = profile;
    }

    /// Get user health profile
    pub fn get_profile(&self) -> &HealthProfile {
        &self.profile
    }

    /// Analyze a recipe and calculate nutritional information
    pub fn analyze_recipe(&self, ingredients: &[Ingredient]) -> RecipeAnalysis {
        let mut total_nutrition = NutritionInfo::new("Total");
        
        for ingredient in ingredients {
            total_nutrition.calories += ingredient.nutrition.calories;
            total_nutrition.protein += ingredient.nutrition.protein;
            total_nutrition.carbs += ingredient.nutrition.carbs;
            total_nutrition.fat += ingredient.nutrition.fat;
            total_nutrition.fiber += ingredient.nutrition.fiber;
            total_nutrition.sugar += ingredient.nutrition.sugar;
            total_nutrition.sodium += ingredient.nutrition.sodium;
        }

        let health_rating = self.calculate_health_rating(&total_nutrition);
        let recommendations = self.generate_recommendations(&total_nutrition);
        
        RecipeAnalysis {
            total_nutrition,
            health_rating,
            recommendations,
            allergen_warnings: self.check_allergens(ingredients),
            dietary_compliance: self.check_dietary_compliance(ingredients),
        }
    }

    /// Calculate health rating for a recipe (0-100)
    fn calculate_health_rating(&self, nutrition: &NutritionInfo) -> u8 {
        let mut score = 100u8;
        
        // Penalize high sodium
        if nutrition.sodium > 600.0 {
            score = score.saturating_sub(10);
        }
        
        // Penalize high sugar
        if nutrition.sugar > 15.0 {
            score = score.saturating_sub(10);
        }
        
        // Penalize high cholesterol
        if nutrition.cholesterol > 100.0 {
            score = score.saturating_sub(5);
        }
        
        // Bonus for high fiber
        if nutrition.fiber > 10.0 {
            score = score.saturating_add(10);
        }
        
        // Bonus for high protein
        if nutrition.protein > 20.0 {
            score = score.saturating_add(5);
        }
        
        // Bonus for balanced macros
        let total_macros = nutrition.protein + nutrition.carbs + nutrition.fat;
        if total_macros > 0.0 {
            let protein_ratio = nutrition.protein / total_macros;
            let carb_ratio = nutrition.carbs / total_macros;
            let fat_ratio = nutrition.fat / total_macros;
            
            // Ideal: ~30% protein, ~45% carbs, ~25% fat
            if protein_ratio > 0.2 && carb_ratio < 0.6 && fat_ratio < 0.35 {
                score = score.saturating_add(5);
            }
        }
        
        score.min(100)
    }

    /// Generate dietary recommendations based on nutrition
    fn generate_recommendations(&self, nutrition: &NutritionInfo) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if nutrition.sodium > 2300.0 {
            recommendations.push(
                "High sodium content. Consider reducing salt or using herbs for flavor.".to_string()
            );
        }
        
        if nutrition.sugar > 25.0 {
            recommendations.push(
                "High sugar content. Consider natural sweeteners or reducing sweet ingredients.".to_string()
            );
        }
        
        if nutrition.fiber < 5.0 {
            recommendations.push(
                "Low fiber. Consider adding vegetables, whole grains, or legumes.".to_string()
            );
        }
        
        if nutrition.protein < 10.0 {
            recommendations.push(
                "Low protein. Consider adding lean protein sources like chicken, fish, or legumes.".to_string()
            );
        }
        
        if nutrition.fat > 30.0 {
            recommendations.push(
                "High fat content. Consider using healthier cooking methods or reducing oils.".to_string()
            );
        }
        
        if recommendations.is_empty() {
            recommendations.push("Great balance of nutrients!".to_string());
        }
        
        recommendations
    }

    /// Check for allergens in ingredients
    fn check_allergens(&self, ingredients: &[Ingredient]) -> Vec<AllergenWarning> {
        let user_allergens = &self.profile.allergies;
        let mut warnings = Vec::new();
        
        for ingredient in ingredients {
            for allergen in user_allergens {
                if ingredient.name.to_lowercase().contains(&allergen.to_lowercase()) {
                    warnings.push(AllergenWarning {
                        ingredient: ingredient.name.clone(),
                        allergen: allergen.clone(),
                        severity: AllergenSeverity::Moderate,
                    });
                }
            }
        }
        
        warnings
    }

    /// Check dietary compliance
    fn check_dietary_compliance(&self, ingredients: &[Ingredient]) -> HashMap<DietaryFlag, bool> {
        let mut compliance = HashMap::new();
        
        for flag in &self.profile.dietary_restrictions {
            let compliant = match flag {
                DietaryFlag::Vegan => ingredients.iter().all(|i| {
                    let name = i.name.to_lowercase();
                    !name.contains("meat") && !name.contains("fish") && 
                    !name.contains("egg") && !name.contains("milk") &&
                    !name.contains("cheese") && !name.contains("butter")
                }),
                DietaryFlag::Vegetarian => ingredients.iter().all(|i| {
                    let name = i.name.to_lowercase();
                    !name.contains("meat") && !name.contains("fish")
                }),
                DietaryFlag::GlutenFree => ingredients.iter().all(|i| {
                    let name = i.name.to_lowercase();
                    !name.contains("wheat") && !name.contains("flour") &&
                    !name.contains("barley") && !name.contains("rye")
                }),
                DietaryFlag::DairyFree => ingredients.iter().all(|i| {
                    let name = i.name.to_lowercase();
                    !name.contains("milk") && !name.contains("cheese") &&
                    !name.contains("butter") && !name.contains("cream")
                }),
                DietaryFlag::Keto => {
                    let total_carbs: f32 = ingredients.iter()
                        .map(|i| i.nutrition.carbs)
                        .sum();
                    total_carbs < 20.0
                },
                _ => true,
            };
            compliance.insert(*flag, compliant);
        }
        
        compliance
    }

    /// Log a meal
    pub fn log_meal(&mut self, entry: FoodEntry) {
        self.meal_history.push(entry);
    }

    /// Get today's nutrition summary
    pub fn get_today_nutrition(&self) -> NutritionSummary {
        let today = self.get_today_timestamp();
        let today_entries: Vec<&FoodEntry> = self.meal_history
            .iter()
            .filter(|e| e.timestamp >= today)
            .collect();
        
        let mut summary = NutritionSummary::default();
        
        for entry in &today_entries {
            summary.calories += entry.actual_calories();
            summary.protein += entry.nutrition.protein * entry.portion;
            summary.carbs += entry.nutrition.carbs * entry.portion;
            summary.fat += entry.nutrition.fat * entry.portion;
            summary.fiber += entry.nutrition.fiber * entry.portion;
            summary.sugar += entry.nutrition.sugar * entry.portion;
        }
        
        summary.remaining_calories = self.profile.calorie_target as i32 - summary.calories as i32;
        summary
    }

    /// Get today's start timestamp
    fn get_today_timestamp(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        // Get midnight today
        let seconds_per_day = 86400;
        now - (now % seconds_per_day)
    }

    /// Get food from database
    pub fn get_food_info(&self, name: &str) -> Option<&NutritionInfo> {
        self.database.get_food(name)
    }

    /// Search foods in database
    pub fn search_foods(&self, query: &str) -> Vec<&NutritionInfo> {
        self.database.search_foods(query)
    }

    /// Calculate recommended daily calories
    pub fn calculate_tdee(&self) -> u32 {
        let profile = &self.profile;
        
        // Mifflin-St Jeor Equation
        let bmr = match profile.gender {
            Gender::Male => {
                10.0 * profile.weight_kg + 6.25 * profile.height_cm as f32 - 5.0 * profile.age as f32 + 5.0
            }
            Gender::Female => {
                10.0 * profile.weight_kg + 6.25 * profile.height_cm as f32 - 5.0 * profile.age as f32 - 161.0
            }
            _ => {
                // Average of male and female
                10.0 * profile.weight_kg + 6.25 * profile.height_cm as f32 - 5.0 * profile.age as f32 - 78.0
            }
        };
        
        (bmr * profile.activity_level.multiplier()) as u32
    }

    /// Get meal suggestions based on remaining calories
    pub fn get_meal_suggestions(&self, max_calories: u32) -> Vec<MealSuggestion> {
        let mut suggestions = Vec::new();
        
        // Get some healthy options from database
        let healthy_foods = vec!["Greek Yogurt", "Chicken Breast", "Broccoli", "Spinach", "Salmon", "Oatmeal"];
        
        for food_name in healthy_foods {
            if let Some(nutrition) = self.database.get_food(food_name) {
                if nutrition.calories <= max_calories {
                    suggestions.push(MealSuggestion {
                        name: nutrition.name.clone(),
                        calories: nutrition.calories,
                        protein: nutrition.protein,
                        meal_type: self.suggest_meal_type(nutrition),
                    });
                }
            }
        }
        
        suggestions
    }

    /// Suggest meal type based on nutrition profile
    fn suggest_meal_type(&self, nutrition: &NutritionInfo) -> MealType {
        if nutrition.protein > 15.0 {
            MealType::Breakfast // High protein breakfast
        } else if nutrition.carbs > 20.0 {
            MealType::Lunch // Carb-focused for energy
        } else if nutrition.calories < 200 {
            MealType::Snack
        } else {
            MealType::Dinner
        }
    }
}

impl Default for NutriScanner {
    fn default() -> Self {
        Self::new()
    }
}

/// Recipe analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeAnalysis {
    /// Total nutrition for the recipe
    pub total_nutrition: NutritionInfo,
    /// Health rating (0-100)
    pub health_rating: u8,
    /// Dietary recommendations
    pub recommendations: Vec<String>,
    /// Allergen warnings
    pub allergen_warnings: Vec<AllergenWarning>,
    /// Dietary compliance check
    pub dietary_compliance: HashMap<DietaryFlag, bool>,
}

/// Allergen warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllergenWarning {
    /// Ingredient containing allergen
    pub ingredient: String,
    /// The allergen
    pub allergen: String,
    /// Severity of the warning
    pub severity: AllergenSeverity,
}

/// Daily nutrition summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NutritionSummary {
    /// Total calories consumed
    pub calories: u32,
    /// Protein in grams
    pub protein: f32,
    /// Carbs in grams
    pub carbs: f32,
    /// Fat in grams
    pub fat: f32,
    /// Fiber in grams
    pub fiber: f32,
    /// Sugar in grams
    pub sugar: f32,
    /// Remaining calories for the day
    pub remaining_calories: i32,
}

/// Meal suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MealSuggestion {
    /// Food name
    pub name: String,
    /// Calories
    pub calories: u32,
    /// Protein content
    pub protein: f32,
    /// Suggested meal type
    pub meal_type: MealType,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nutri_scanner_creation() {
        let scanner = NutriScanner::new();
        assert!(scanner.get_food_info("apple").is_some());
    }

    #[test]
    fn test_recipe_analysis() {
        let scanner = NutriScanner::new();
        let ingredients = vec![
            Ingredient {
                name: "Chicken Breast".to_string(),
                amount: 100.0,
                unit: "g".to_string(),
                nutrition: NutritionInfo {
                    name: "Chicken Breast".to_string(),
                    calories: 165,
                    protein: 31.0,
                    carbs: 0.0,
                    fat: 3.6,
                    ..Default::default()
                },
            },
        ];
        
        let analysis = scanner.analyze_recipe(&ingredients);
        assert_eq!(analysis.total_nutrition.calories, 165);
        assert!(analysis.health_rating > 70);
    }

    #[test]
    fn test_tdee_calculation() {
        let mut scanner = NutriScanner::new();
        let mut profile = HealthProfile::default();
        profile.gender = Gender::Male;
        profile.age = 30;
        profile.weight_kg = 80.0;
        profile.height_cm = 180;
        profile.activity_level = ActivityLevel::Moderate;
        scanner.set_profile(profile);
        
        let tdee = scanner.calculate_tdee();
        assert!(tdee > 1500 && tdee < 4000);
    }

    #[test]
    fn test_meal_logging() {
        let mut scanner = NutriScanner::new();
        let nutrition = scanner.get_food_info("apple").unwrap().clone();
        let entry = FoodEntry::new("Apple", nutrition);
        
        scanner.log_meal(entry);
        let summary = scanner.get_today_nutrition();
        assert_eq!(summary.calories, 95);
    }
}