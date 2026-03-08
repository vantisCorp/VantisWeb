//! Visual Calorie Counter - Photo-based food recognition and calorie tracking
//! 
//! This module provides AI-powered food recognition from images,
//! portion estimation, and comprehensive calorie tracking.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use super::models::*;
use super::database::FoodDatabase;

/// Visual Calorie Counter for photo-based food tracking
pub struct VisualCalorieCounter {
    database: FoodDatabase,
    food_log: Vec<FoodEntry>,
    target_calories: u32,
    target_protein: f32,
    target_carbs: f32,
    target_fat: f32,
    recognition_history: Vec<RecognitionResult>,
}

impl VisualCalorieCounter {
    /// Create a new Visual Calorie Counter
    pub fn new() -> Self {
        Self {
            database: FoodDatabase::new(),
            food_log: Vec::new(),
            target_calories: 2000,
            target_protein: 150.0,
            target_carbs: 250.0,
            target_fat: 65.0,
            recognition_history: Vec::new(),
        }
    }

    /// Set daily calorie target
    pub fn set_target_calories(&mut self, target: u32) {
        self.target_calories = target;
    }

    /// Set macro targets
    pub fn set_macro_targets(&mut self, protein: f32, carbs: f32, fat: f32) {
        self.target_protein = protein;
        self.target_carbs = carbs;
        self.target_fat = fat;
    }

    /// Analyze food image (simulated AI recognition)
    /// In a real implementation, this would use computer vision models
    pub fn analyze_image(&mut self, image_data: &[u8]) -> RecognitionResult {
        // Simulated AI recognition
        // In production, this would use TensorFlow/PyTorch models
        let recognized_foods = self.simulate_recognition(image_data);
        
        let result = RecognitionResult {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            foods: recognized_foods.clone(),
            total_calories: recognized_foods.iter().map(|f| f.estimated_calories).sum(),
            confidence: self.calculate_overall_confidence(&recognized_foods),
            image_hash: self.hash_image(image_data),
        };
        
        self.recognition_history.push(result.clone());
        result
    }

    /// Simulate food recognition (placeholder for AI model)
    fn simulate_recognition(&self, _image_data: &[u8]) -> Vec<RecognizedFood> {
        // In a real implementation, this would:
        // 1. Preprocess image (resize, normalize)
        // 2. Run through CNN model (e.g., FoodNet, YOLO)
        // 3. Post-process detections
        // 4. Estimate portion sizes
        
        // Return sample data for demonstration
        vec![
            RecognizedFood {
                name: "Chicken Breast".to_string(),
                estimated_calories: 165,
                estimated_portion: 1.0,
                confidence: 0.92,
                bounding_box: Some(BoundingBox {
                    x: 100,
                    y: 100,
                    width: 200,
                    height: 150,
                }),
                nutrition: self.database.get_food("Chicken Breast")
                    .cloned()
                    .unwrap_or_default(),
            }
        ]
    }

    /// Calculate overall confidence for recognition
    fn calculate_overall_confidence(&self, foods: &[RecognizedFood]) -> f32 {
        if foods.is_empty() {
            return 0.0;
        }
        
        let sum: f32 = foods.iter().map(|f| f.confidence).sum();
        sum / foods.len() as f32
    }

    /// Generate simple hash for image
    fn hash_image(&self, data: &[u8]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Confirm recognition and log food
    pub fn confirm_and_log(&mut self, recognition_id: &str, adjustments: Option<&[FoodAdjustment]>) -> Option<FoodEntry> {
        // Find the recognition result
        let recognition = self.recognition_history.iter()
            .find(|r| r.id == recognition_id)?;
        
        // Apply any adjustments
        let mut foods = recognition.foods.clone();
        if let Some(adj) = adjustments {
            for adjustment in adj {
                if let Some(food) = foods.iter_mut().find(|f| f.name == adjustment.food_name) {
                    food.estimated_portion = adjustment.new_portion;
                    food.estimated_calories = (food.nutrition.calories as f32 * adjustment.new_portion) as u32;
                }
            }
        }
        
        // Create food entry
        let total_calories: u32 = foods.iter().map(|f| f.estimated_calories).sum();
        let total_protein: f32 = foods.iter().map(|f| f.nutrition.protein * f.estimated_portion).sum();
        let total_carbs: f32 = foods.iter().map(|f| f.nutrition.carbs * f.estimated_portion).sum();
        let total_fat: f32 = foods.iter().map(|f| f.nutrition.fat * f.estimated_portion).sum();
        
        let entry = FoodEntry {
            id: uuid::Uuid::new_v4().to_string(),
            name: foods.iter().map(|f| f.name.as_str()).collect::<Vec<_>>().join(", "),
            nutrition: NutritionInfo {
                name: "Combined".to_string(),
                calories: total_calories,
                protein: total_protein,
                carbs: total_carbs,
                fat: total_fat,
                ..Default::default()
            },
            portion: 1.0,
            meal_type: self.infer_meal_type(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            source: EntrySource::Photo,
            confidence: Some(recognition.confidence),
        };
        
        self.food_log.push(entry.clone());
        Some(entry)
    }

    /// Infer meal type from current time
    fn infer_meal_type(&self) -> MealType {
        let hour = chrono::Local::now().hour();
        match hour {
            5..=10 => MealType::Breakfast,
            11..=14 => MealType::Lunch,
            17..=20 => MealType::Dinner,
            _ => MealType::Snack,
        }
    }

    /// Log food manually
    pub fn log_food_manual(&mut self, name: &str, portion: f32, meal_type: Option<MealType>) -> Option<FoodEntry> {
        let nutrition = self.database.get_food(name)?.clone();
        let adjusted_nutrition = NutritionInfo {
            name: nutrition.name.clone(),
            calories: (nutrition.calories as f32 * portion) as u32,
            protein: nutrition.protein * portion,
            carbs: nutrition.carbs * portion,
            fat: nutrition.fat * portion,
            fiber: nutrition.fiber * portion,
            sugar: nutrition.sugar * portion,
            ..nutrition
        };
        
        let entry = FoodEntry {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            nutrition: adjusted_nutrition,
            portion,
            meal_type: meal_type.unwrap_or_else(|| self.infer_meal_type()),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            source: EntrySource::Manual,
            confidence: None,
        };
        
        self.food_log.push(entry.clone());
        Some(entry)
    }

    /// Get daily calories consumed
    pub fn get_daily_calories(&self) -> u32 {
        let today_start = self.get_today_start();
        self.food_log.iter()
            .filter(|e| e.timestamp >= today_start)
            .map(|e| e.actual_calories())
            .sum()
    }

    /// Get remaining calories for today
    pub fn get_remaining_calories(&self) -> i32 {
        self.target_calories as i32 - self.get_daily_calories() as i32
    }

    /// Get daily protein consumed
    pub fn get_daily_protein(&self) -> f32 {
        let today_start = self.get_today_start();
        self.food_log.iter()
            .filter(|e| e.timestamp >= today_start)
            .map(|e| e.nutrition.protein * e.portion)
            .sum()
    }

    /// Get daily carbs consumed
    pub fn get_daily_carbs(&self) -> f32 {
        let today_start = self.get_today_start();
        self.food_log.iter()
            .filter(|e| e.timestamp >= today_start)
            .map(|e| e.nutrition.carbs * e.portion)
            .sum()
    }

    /// Get daily fat consumed
    pub fn get_daily_fat(&self) -> f32 {
        let today_start = self.get_today_start();
        self.food_log.iter()
            .filter(|e| e.timestamp >= today_start)
            .map(|e| e.nutrition.fat * e.portion)
            .sum()
    }

    /// Get target calories
    pub fn get_target_calories(&self) -> u32 {
        self.target_calories
    }

    /// Get today's start timestamp
    fn get_today_start(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now - (now % 86400)
    }

    /// Get today's food log
    pub fn get_today_log(&self) -> Vec<&FoodEntry> {
        let today_start = self.get_today_start();
        self.food_log.iter()
            .filter(|e| e.timestamp >= today_start)
            .collect()
    }

    /// Get macro breakdown for today
    pub fn get_macro_breakdown(&self) -> MacroBreakdown {
        let protein = self.get_daily_protein();
        let carbs = self.get_daily_carbs();
        let fat = self.get_daily_fat();
        let total = protein + carbs + fat;
        
        MacroBreakdown {
            protein_percentage: if total > 0.0 { (protein / total * 100.0) as u8 } else { 0 },
            carbs_percentage: if total > 0.0 { (carbs / total * 100.0) as u8 } else { 0 },
            fat_percentage: if total > 0.0 { (fat / total * 100.0) as u8 } else { 0 },
            protein_grams: protein,
            carbs_grams: carbs,
            fat_grams: fat,
        }
    }

    /// Get weekly summary
    pub fn get_weekly_summary(&self) -> WeeklySummary {
        let week_start = self.get_week_start();
        let week_entries: Vec<&FoodEntry> = self.food_log.iter()
            .filter(|e| e.timestamp >= week_start)
            .collect();
        
        let total_calories: u32 = week_entries.iter().map(|e| e.actual_calories()).sum();
        let days_with_logs = self.count_days_with_logs(&week_entries);
        
        WeeklySummary {
            total_calories,
            average_daily_calories: if days_with_logs > 0 { total_calories / days_with_logs as u32 } else { 0 },
            total_protein: week_entries.iter().map(|e| e.nutrition.protein * e.portion).sum(),
            total_carbs: week_entries.iter().map(|e| e.nutrition.carbs * e.portion).sum(),
            total_fat: week_entries.iter().map(|e| e.nutrition.fat * e.portion).sum(),
            days_logged: days_with_logs,
            meals_logged: week_entries.len() as u32,
        }
    }

    /// Get week start timestamp
    fn get_week_start(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let day_of_week = (now / 86400) % 7;
        now - (day_of_week * 86400)
    }

    /// Count unique days with food logs
    fn count_days_with_logs(&self, entries: &[&FoodEntry]) -> u32 {
        let mut days = std::collections::HashSet::new();
        for entry in entries {
            days.insert(entry.timestamp / 86400);
        }
        days.len() as u32
    }

    /// Delete food entry
    pub fn delete_entry(&mut self, entry_id: &str) -> bool {
        if let Some(pos) = self.food_log.iter().position(|e| e.id == entry_id) {
            self.food_log.remove(pos);
            true
        } else {
            false
        }
    }

    /// Search food database
    pub fn search_foods(&self, query: &str) -> Vec<&NutritionInfo> {
        self.database.search_foods(query)
    }

    /// Get food suggestions based on remaining calories
    pub fn get_suggestions(&self) -> Vec<FoodSuggestion> {
        let remaining = self.get_remaining_calories();
        if remaining <= 0 {
            return vec![];
        }
        
        let foods = self.database.search_foods("");
        foods.into_iter()
            .filter(|f| f.calories <= remaining as u32)
            .take(5)
            .map(|f| FoodSuggestion {
                name: f.name.clone(),
                calories: f.calories,
                protein: f.protein,
                reason: self.get_suggestion_reason(f),
            })
            .collect()
    }

    /// Get suggestion reason
    fn get_suggestion_reason(&self, nutrition: &NutritionInfo) -> String {
        let protein_remaining = self.target_protein - self.get_daily_protein();
        
        if nutrition.protein > 10.0 && protein_remaining > 0.0 {
            "High protein option".to_string()
        } else if nutrition.fiber > 5.0 {
            "Good fiber source".to_string()
        } else {
            "Fits your calorie goal".to_string()
        }
    }
}

impl Default for VisualCalorieCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// Recognition result from image analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognitionResult {
    /// Unique result ID
    pub id: String,
    /// Timestamp
    pub timestamp: u64,
    /// Recognized foods
    pub foods: Vec<RecognizedFood>,
    /// Total estimated calories
    pub total_calories: u32,
    /// Overall confidence (0.0-1.0)
    pub confidence: f32,
    /// Image hash for caching
    pub image_hash: String,
}

/// Recognized food item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognizedFood {
    /// Food name
    pub name: String,
    /// Estimated calories
    pub estimated_calories: u32,
    /// Estimated portion (multiplier)
    pub estimated_portion: f32,
    /// Recognition confidence (0.0-1.0)
    pub confidence: f32,
    /// Bounding box in image
    pub bounding_box: Option<BoundingBox>,
    /// Nutrition information
    pub nutrition: NutritionInfo,
}

/// Bounding box for detected food
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Food adjustment for confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodAdjustment {
    pub food_name: String,
    pub new_portion: f32,
}

/// Macro breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroBreakdown {
    pub protein_percentage: u8,
    pub carbs_percentage: u8,
    pub fat_percentage: u8,
    pub protein_grams: f32,
    pub carbs_grams: f32,
    pub fat_grams: f32,
}

/// Weekly summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklySummary {
    pub total_calories: u32,
    pub average_daily_calories: u32,
    pub total_protein: f32,
    pub total_carbs: f32,
    pub total_fat: f32,
    pub days_logged: u32,
    pub meals_logged: u32,
}

/// Food suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodSuggestion {
    pub name: String,
    pub calories: u32,
    pub protein: f32,
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_creation() {
        let counter = VisualCalorieCounter::new();
        assert_eq!(counter.get_target_calories(), 2000);
    }

    #[test]
    fn test_manual_logging() {
        let mut counter = VisualCalorieCounter::new();
        let entry = counter.log_food_manual("Apple", 1.0, Some(MealType::Snack));
        
        assert!(entry.is_some());
        assert_eq!(counter.get_daily_calories(), 95);
    }

    #[test]
    fn test_remaining_calories() {
        let mut counter = VisualCalorieCounter::new();
        counter.set_target_calories(2000);
        counter.log_food_manual("Apple", 1.0, None);
        
        let remaining = counter.get_remaining_calories();
        assert_eq!(remaining, 1905);
    }

    #[test]
    fn test_macro_breakdown() {
        let mut counter = VisualCalorieCounter::new();
        counter.log_food_manual("Chicken Breast", 1.0, None);
        
        let breakdown = counter.get_macro_breakdown();
        assert!(breakdown.protein_percentage > 50);
    }
}