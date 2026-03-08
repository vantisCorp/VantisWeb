//! Food and additive database
//! 
//! This module provides access to food nutritional data and
//! additive information databases.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::models::*;

/// Food database for nutritional information
pub struct FoodDatabase {
    /// Food items indexed by name
    foods: HashMap<String, NutritionInfo>,
    /// Additives indexed by E-number
    additives: HashMap<String, AdditiveInfo>,
    /// Common allergens
    allergens: Vec<String>,
}

impl FoodDatabase {
    /// Create a new food database with initial data
    pub fn new() -> Self {
        let mut db = Self {
            foods: HashMap::new(),
            additives: HashMap::new(),
            allergens: Vec::new(),
        };
        db.initialize_common_foods();
        db.initialize_additives();
        db.initialize_allergens();
        db
    }

    /// Initialize common food items
    fn initialize_common_foods(&mut self) {
        // Fruits
        self.add_food(NutritionInfo {
            name: "Apple".to_string(),
            serving_size: 182.0,
            calories: 95,
            protein: 0.5,
            carbs: 25.0,
            fat: 0.3,
            fiber: 4.4,
            sugar: 19.0,
            sodium: 2.0,
            cholesterol: 0.0,
            micronutrients: {
                let mut m = HashMap::new();
                m.insert("Vitamin C".to_string(), 8.4);
                m.insert("Potassium".to_string(), 195.0);
                m
            },
        });

        self.add_food(NutritionInfo {
            name: "Banana".to_string(),
            serving_size: 118.0,
            calories: 105,
            protein: 1.3,
            carbs: 27.0,
            fat: 0.4,
            fiber: 3.1,
            sugar: 14.0,
            sodium: 1.0,
            cholesterol: 0.0,
            micronutrients: {
                let mut m = HashMap::new();
                m.insert("Vitamin C".to_string(), 10.3);
                m.insert("Potassium".to_string(), 422.0);
                m.insert("Vitamin B6".to_string(), 0.4);
                m
            },
        });

        // Proteins
        self.add_food(NutritionInfo {
            name: "Chicken Breast".to_string(),
            serving_size: 100.0,
            calories: 165,
            protein: 31.0,
            carbs: 0.0,
            fat: 3.6,
            fiber: 0.0,
            sugar: 0.0,
            sodium: 74.0,
            cholesterol: 85.0,
            micronutrients: {
                let mut m = HashMap::new();
                m.insert("Vitamin B6".to_string(), 0.6);
                m.insert("Niacin".to_string(), 13.7);
                m.insert("Selenium".to_string(), 27.6);
                m
            },
        });

        self.add_food(NutritionInfo {
            name: "Salmon".to_string(),
            serving_size: 100.0,
            calories: 208,
            protein: 20.0,
            carbs: 0.0,
            fat: 13.0,
            fiber: 0.0,
            sugar: 0.0,
            sodium: 59.0,
            cholesterol: 55.0,
            micronutrients: {
                let mut m = HashMap::new();
                m.insert("Vitamin D".to_string(), 526.0);
                m.insert("Omega-3".to_string(), 2.3);
                m.insert("Selenium".to_string(), 36.5);
                m
            },
        });

        self.add_food(NutritionInfo {
            name: "Egg".to_string(),
            serving_size: 50.0,
            calories: 78,
            protein: 6.0,
            carbs: 0.6,
            fat: 5.0,
            fiber: 0.0,
            sugar: 0.6,
            sodium: 62.0,
            cholesterol: 187.0,
            micronutrients: {
                let mut m = HashMap::new();
                m.insert("Vitamin D".to_string(), 1.0);
                m.insert("Vitamin B12".to_string(), 0.6);
                m.insert("Choline".to_string(), 147.0);
                m
            },
        });

        // Grains
        self.add_food(NutritionInfo {
            name: "Brown Rice".to_string(),
            serving_size: 100.0,
            calories: 111,
            protein: 2.6,
            carbs: 23.0,
            fat: 0.9,
            fiber: 1.8,
            sugar: 0.4,
            sodium: 5.0,
            cholesterol: 0.0,
            micronutrients: {
                let mut m = HashMap::new();
                m.insert("Manganese".to_string(), 1.1);
                m.insert("Magnesium".to_string(), 44.0);
                m
            },
        });

        self.add_food(NutritionInfo {
            name: "Oatmeal".to_string(),
            serving_size: 40.0,
            calories: 154,
            protein: 5.0,
            carbs: 27.0,
            fat: 3.0,
            fiber: 4.0,
            sugar: 1.0,
            sodium: 2.0,
            cholesterol: 0.0,
            micronutrients: {
                let mut m = HashMap::new();
                m.insert("Fiber".to_string(), 4.0);
                m.insert("Iron".to_string(), 1.7);
                m
            },
        });

        // Vegetables
        self.add_food(NutritionInfo {
            name: "Broccoli".to_string(),
            serving_size: 91.0,
            calories: 31,
            protein: 2.5,
            carbs: 6.0,
            fat: 0.3,
            fiber: 2.4,
            sugar: 1.5,
            sodium: 30.0,
            cholesterol: 0.0,
            micronutrients: {
                let mut m = HashMap::new();
                m.insert("Vitamin C".to_string(), 81.2);
                m.insert("Vitamin K".to_string(), 92.5);
                m.insert("Folate".to_string(), 57.0);
                m
            },
        });

        self.add_food(NutritionInfo {
            name: "Spinach".to_string(),
            serving_size: 30.0,
            calories: 7,
            protein: 0.9,
            carbs: 1.1,
            fat: 0.1,
            fiber: 0.7,
            sugar: 0.1,
            sodium: 24.0,
            cholesterol: 0.0,
            micronutrients: {
                let mut m = HashMap::new();
                m.insert("Vitamin A".to_string(), 2813.0);
                m.insert("Iron".to_string(), 0.8);
                m.insert("Vitamin K".to_string(), 145.0);
                m
            },
        });

        // Dairy
        self.add_food(NutritionInfo {
            name: "Greek Yogurt".to_string(),
            serving_size: 170.0,
            calories: 100,
            protein: 17.0,
            carbs: 6.0,
            fat: 0.7,
            fiber: 0.0,
            sugar: 4.0,
            sodium: 65.0,
            cholesterol: 10.0,
            micronutrients: {
                let mut m = HashMap::new();
                m.insert("Calcium".to_string(), 150.0);
                m.insert("Probiotics".to_string(), 1.0);
                m
            },
        });

        // Snacks
        self.add_food(NutritionInfo {
            name: "Almonds".to_string(),
            serving_size: 28.0,
            calories: 164,
            protein: 6.0,
            carbs: 6.0,
            fat: 14.0,
            fiber: 3.5,
            sugar: 1.2,
            sodium: 0.0,
            cholesterol: 0.0,
            micronutrients: {
                let mut m = HashMap::new();
                m.insert("Vitamin E".to_string(), 7.3);
                m.insert("Magnesium".to_string(), 76.0);
                m
            },
        });

        // Beverages
        self.add_food(NutritionInfo {
            name: "Orange Juice".to_string(),
            serving_size: 240.0,
            calories: 110,
            protein: 2.0,
            carbs: 26.0,
            fat: 0.5,
            fiber: 0.5,
            sugar: 22.0,
            sodium: 5.0,
            cholesterol: 0.0,
            micronutrients: {
                let mut m = HashMap::new();
                m.insert("Vitamin C".to_string(), 124.0);
                m.insert("Folate".to_string(), 74.0);
                m
            },
        });
    }

    /// Initialize common food additives
    fn initialize_additives(&mut self) {
        // Colorants
        self.add_additive(AdditiveInfo {
            e_number: "E100".to_string(),
            name: "Curcumin".to_string(),
            category: AdditiveCategory::Colorant,
            safety: SafetyRating::Safe,
            description: "Natural yellow color from turmeric".to_string(),
            allergens: vec![],
            dietary_flags: vec![DietaryFlag::Vegetarian, DietaryFlag::Vegan],
            source: AdditiveSource::Natural,
        });

        self.add_additive(AdditiveInfo {
            e_number: "E102".to_string(),
            name: "Tartrazine".to_string(),
            category: AdditiveCategory::Colorant,
            safety: SafetyRating::Caution,
            description: "Synthetic yellow dye. May cause hyperactivity in children".to_string(),
            allergens: vec!["Aspirin sensitivity".to_string()],
            dietary_flags: vec![DietaryFlag::Vegetarian, DietaryFlag::Vegan],
            source: AdditiveSource::Synthetic,
        });

        self.add_additive(AdditiveInfo {
            e_number: "E129".to_string(),
            name: "Allura Red AC".to_string(),
            category: AdditiveCategory::Colorant,
            safety: SafetyRating::Caution,
            description: "Red food coloring. Banned in some European countries".to_string(),
            allergens: vec![],
            dietary_flags: vec![DietaryFlag::Vegetarian, DietaryFlag::Vegan],
            source: AdditiveSource::Synthetic,
        });

        // Preservatives
        self.add_additive(AdditiveInfo {
            e_number: "E200".to_string(),
            name: "Sorbic Acid".to_string(),
            category: AdditiveCategory::Preservative,
            safety: SafetyRating::Safe,
            description: "Natural preservative effective against mold".to_string(),
            allergens: vec![],
            dietary_flags: vec![DietaryFlag::Vegetarian, DietaryFlag::Vegan],
            source: AdditiveSource::Natural,
        });

        self.add_additive(AdditiveInfo {
            e_number: "E210".to_string(),
            name: "Benzoic Acid".to_string(),
            category: AdditiveCategory::Preservative,
            safety: SafetyRating::Caution,
            description: "May form benzene with vitamin C. Use with caution".to_string(),
            allergens: vec!["Asthma".to_string()],
            dietary_flags: vec![DietaryFlag::Vegetarian, DietaryFlag::Vegan],
            source: AdditiveSource::Both,
        });

        self.add_additive(AdditiveInfo {
            e_number: "E250".to_string(),
            name: "Sodium Nitrite".to_string(),
            category: AdditiveCategory::Preservative,
            safety: SafetyRating::Avoid,
            description: "Used in cured meats. Linked to cancer in high amounts".to_string(),
            allergens: vec![],
            dietary_flags: vec![DietaryFlag::Vegetarian, DietaryFlag::Vegan],
            source: AdditiveSource::Synthetic,
        });

        // Antioxidants
        self.add_additive(AdditiveInfo {
            e_number: "E300".to_string(),
            name: "Ascorbic Acid (Vitamin C)".to_string(),
            category: AdditiveCategory::Antioxidant,
            safety: SafetyRating::Safe,
            description: "Essential vitamin, safe and beneficial".to_string(),
            allergens: vec![],
            dietary_flags: vec![DietaryFlag::Vegetarian, DietaryFlag::Vegan],
            source: AdditiveSource::Both,
        });

        self.add_additive(AdditiveInfo {
            e_number: "E320".to_string(),
            name: "Butylated Hydroxyanisole (BHA)".to_string(),
            category: AdditiveCategory::Antioxidant,
            safety: SafetyRating::Avoid,
            description: "Potential carcinogen. Banned in some countries".to_string(),
            allergens: vec![],
            dietary_flags: vec![DietaryFlag::Vegetarian, DietaryFlag::Vegan],
            source: AdditiveSource::Synthetic,
        });

        self.add_additive(AdditiveInfo {
            e_number: "E321".to_string(),
            name: "Butylated Hydroxytoluene (BHT)".to_string(),
            category: AdditiveCategory::Antioxidant,
            safety: SafetyRating::Caution,
            description: "Controversial safety profile. Limited studies".to_string(),
            allergens: vec![],
            dietary_flags: vec![DietaryFlag::Vegetarian, DietaryFlag::Vegan],
            source: AdditiveSource::Synthetic,
        });

        // Emulsifiers
        self.add_additive(AdditiveInfo {
            e_number: "E322".to_string(),
            name: "Lecithin".to_string(),
            category: AdditiveCategory::Emulsifier,
            safety: SafetyRating::Safe,
            description: "Natural emulsifier, often from soy or sunflower".to_string(),
            allergens: vec!["Soy".to_string()],
            dietary_flags: vec![DietaryFlag::Vegetarian, DietaryFlag::Vegan],
            source: AdditiveSource::Natural,
        });

        self.add_additive(AdditiveInfo {
            e_number: "E471".to_string(),
            name: "Mono and Diglycerides".to_string(),
            category: AdditiveCategory::Emulsifier,
            safety: SafetyRating::Safe,
            description: "Common emulsifier, usually from vegetable oils".to_string(),
            allergens: vec![],
            dietary_flags: vec![DietaryFlag::Vegetarian],
            source: AdditiveSource::Both,
        });

        // Flavor Enhancers
        self.add_additive(AdditiveInfo {
            e_number: "E621".to_string(),
            name: "Monosodium Glutamate (MSG)".to_string(),
            category: AdditiveCategory::FlavorEnhancer,
            safety: SafetyRating::Caution,
            description: "Flavor enhancer. May cause reactions in sensitive individuals".to_string(),
            allergens: vec!["MSG sensitivity".to_string()],
            dietary_flags: vec![DietaryFlag::Vegetarian, DietaryFlag::Vegan],
            source: AdditiveSource::Both,
        });

        // Sweeteners
        self.add_additive(AdditiveInfo {
            e_number: "E950".to_string(),
            name: "Acesulfame K".to_string(),
            category: AdditiveCategory::Sweetener,
            safety: SafetyRating::Caution,
            description: "Artificial sweetener. Limited long-term studies".to_string(),
            allergens: vec![],
            dietary_flags: vec![DietaryFlag::Vegetarian, DietaryFlag::Vegan, DietaryFlag::Diabetic, DietaryFlag::Keto],
            source: AdditiveSource::Synthetic,
        });

        self.add_additive(AdditiveInfo {
            e_number: "E951".to_string(),
            name: "Aspartame".to_string(),
            category: AdditiveCategory::Sweetener,
            safety: SafetyRating::Caution,
            description: "Artificial sweetener. Not safe for PKU patients".to_string(),
            allergens: vec!["Phenylketonuria (PKU)".to_string()],
            dietary_flags: vec![DietaryFlag::Vegetarian, DietaryFlag::Vegan, DietaryFlag::Diabetic, DietaryFlag::Keto],
            source: AdditiveSource::Synthetic,
        });

        self.add_additive(AdditiveInfo {
            e_number: "E967".to_string(),
            name: "Xylitol".to_string(),
            category: AdditiveCategory::Sweetener,
            safety: SafetyRating::Safe,
            description: "Sugar alcohol. Safe for diabetics. Toxic to dogs!".to_string(),
            allergens: vec![],
            dietary_flags: vec![DietaryFlag::Vegetarian, DietaryFlag::Vegan, DietaryFlag::Diabetic, DietaryFlag::Keto],
            source: AdditiveSource::Natural,
        });

        // Stabilizers
        self.add_additive(AdditiveInfo {
            e_number: "E414".to_string(),
            name: "Gum Arabic".to_string(),
            category: AdditiveCategory::Stabilizer,
            safety: SafetyRating::Safe,
            description: "Natural gum from acacia trees".to_string(),
            allergens: vec![],
            dietary_flags: vec![DietaryFlag::Vegetarian, DietaryFlag::Vegan],
            source: AdditiveSource::Natural,
        });

        self.add_additive(AdditiveInfo {
            e_number: "E415".to_string(),
            name: "Xanthan Gum".to_string(),
            category: AdditiveCategory::Stabilizer,
            safety: SafetyRating::Safe,
            description: "Fermentation product. Safe thickener".to_string(),
            allergens: vec![],
            dietary_flags: vec![DietaryFlag::Vegetarian, DietaryFlag::Vegan, DietaryFlag::GlutenFree],
            source: AdditiveSource::Natural,
        });

        // Acidity Regulators
        self.add_additive(AdditiveInfo {
            e_number: "E330".to_string(),
            name: "Citric Acid".to_string(),
            category: AdditiveCategory::AcidityRegulator,
            safety: SafetyRating::Safe,
            description: "Natural acid from citrus fruits".to_string(),
            allergens: vec![],
            dietary_flags: vec![DietaryFlag::Vegetarian, DietaryFlag::Vegan],
            source: AdditiveSource::Both,
        });

        self.add_additive(AdditiveInfo {
            e_number: "E338".to_string(),
            name: "Phosphoric Acid".to_string(),
            category: AdditiveCategory::AcidityRegulator,
            safety: SafetyRating::Caution,
            description: "May affect bone density with excessive consumption".to_string(),
            allergens: vec![],
            dietary_flags: vec![DietaryFlag::Vegetarian, DietaryFlag::Vegan],
            source: AdditiveSource::Synthetic,
        });
    }

    /// Initialize common allergens list
    fn initialize_allergens(&mut self) {
        self.allergens = vec![
            "Milk".to_string(),
            "Eggs".to_string(),
            "Fish".to_string(),
            "Crustacean shellfish".to_string(),
            "Tree nuts".to_string(),
            "Peanuts".to_string(),
            "Wheat".to_string(),
            "Soybeans".to_string(),
            "Sesame".to_string(),
            "Sulfites".to_string(),
            "Mustard".to_string(),
            "Celery".to_string(),
            "Lupin".to_string(),
            "Molluscs".to_string(),
        ];
    }

    /// Add a food item to the database
    pub fn add_food(&mut self, nutrition: NutritionInfo) {
        self.foods.insert(nutrition.name.to_lowercase(), nutrition);
    }

    /// Add an additive to the database
    pub fn add_additive(&mut self, additive: AdditiveInfo) {
        self.additives.insert(additive.e_number.clone(), additive);
    }

    /// Look up food by name
    pub fn get_food(&self, name: &str) -> Option<&NutritionInfo> {
        self.foods.get(&name.to_lowercase())
    }

    /// Look up additive by E-number
    pub fn get_additive(&self, e_number: &str) -> Option<&AdditiveInfo> {
        self.additives.get(e_number)
    }

    /// Search foods by partial name
    pub fn search_foods(&self, query: &str) -> Vec<&NutritionInfo> {
        let query = query.to_lowercase();
        self.foods
            .values()
            .filter(|food| food.name.to_lowercase().contains(&query))
            .collect()
    }

    /// Get all allergens
    pub fn get_allergens(&self) -> &[String] {
        &self.allergens
    }

    /// Check if food contains allergens
    pub fn check_allergens(&self, food_name: &str, user_allergens: &[String]) -> Vec<String> {
        let food_lower = food_name.to_lowercase();
        user_allergens
            .iter()
            .filter(|allergen| {
                let allergen_lower = allergen.to_lowercase();
                food_lower.contains(&allergen_lower)
            })
            .cloned()
            .collect()
    }

    /// Get additives by safety rating
    pub fn get_additives_by_safety(&self, safety: SafetyRating) -> Vec<&AdditiveInfo> {
        self.additives
            .values()
            .filter(|a| a.safety == safety)
            .collect()
    }

    /// Parse ingredient list and extract E-numbers
    pub fn extract_e_numbers(&self, ingredients: &str) -> Vec<&AdditiveInfo> {
        let e_pattern = regex::Regex::new(r"E\d{3,4}[a-k]?").unwrap();
        e_pattern
            .find_iter(ingredients)
            .filter_map(|m| self.additives.get(m.as_str()))
            .collect()
    }
}

impl Default for FoodDatabase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creation() {
        let db = FoodDatabase::new();
        assert!(db.get_food("apple").is_some());
        assert!(db.get_additive("E330").is_some());
    }

    #[test]
    fn test_food_search() {
        let db = FoodDatabase::new();
        let results = db.search_foods("chicken");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_additive_safety() {
        let db = FoodDatabase::new();
        let safe = db.get_additives_by_safety(SafetyRating::Safe);
        assert!(!safe.is_empty());
    }
}