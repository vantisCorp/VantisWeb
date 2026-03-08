//! Ingred-X - Food additive detection and safety analysis
//! 
//! This module provides comprehensive food additive (E-number) detection,
//! safety ratings, allergen warnings, and product analysis.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::models::*;
use super::database::FoodDatabase;

/// Ingred-X - Food additive analyzer
pub struct IngredX {
    database: FoodDatabase,
    user_allergies: Vec<String>,
    dietary_restrictions: Vec<DietaryFlag>,
    warning_threshold: SafetyRating,
}

impl IngredX {
    /// Create a new Ingred-X instance
    pub fn new() -> Self {
        Self {
            database: FoodDatabase::new(),
            user_allergies: Vec::new(),
            dietary_restrictions: Vec::new(),
            warning_threshold: SafetyRating::Caution,
        }
    }

    /// Set user allergies
    pub fn set_allergies(&mut self, allergies: Vec<String>) {
        self.user_allergies = allergies;
    }

    /// Set dietary restrictions
    pub fn set_dietary_restrictions(&mut self, restrictions: Vec<DietaryFlag>) {
        self.dietary_restrictions = restrictions;
    }

    /// Set warning threshold (minimum safety level to show warnings)
    pub fn set_warning_threshold(&mut self, threshold: SafetyRating) {
        self.warning_threshold = threshold;
    }

    /// Analyze product ingredients string
    pub fn analyze_product(&self, ingredients_text: &str) -> ProductAnalysis {
        let detected_additives = self.extract_and_analyze_additives(ingredients_text);
        let safety_score = self.calculate_product_safety(&detected_additives);
        let warnings = self.generate_warnings(&detected_additives);
        let dietary_check = self.check_dietary_compliance(&detected_additives);
        
        ProductAnalysis {
            ingredients_text: ingredients_text.to_string(),
            additives: detected_additives.clone(),
            safety_score,
            warnings,
            dietary_compliance: dietary_check,
            recommendation: self.generate_recommendation(&detected_additives, safety_score),
        }
    }

    /// Extract and analyze E-numbers from ingredient text
    fn extract_and_analyze_additives(&self, text: &str) -> Vec<AdditiveAnalysis> {
        let mut analyses = Vec::new();
        let e_pattern = regex::Regex::new(r"(?i)\b(E\d{3,4}[a-k]?)\b").unwrap();
        
        for cap in e_pattern.captures_iter(text) {
            let e_number = cap[1].to_uppercase();
            if let Some(info) = self.database.get_additive(&e_number) {
                analyses.push(AdditiveAnalysis {
                    info: info.clone(),
                    warning_level: self.get_warning_level(info.safety),
                    user_relevant: self.is_relevant_to_user(info),
                });
            }
        }
        
        analyses
    }

    /// Get warning level based on threshold
    fn get_warning_level(&self, safety: SafetyRating) -> WarningLevel {
        match safety {
            SafetyRating::Safe => WarningLevel::None,
            SafetyRating::Caution => WarningLevel::Low,
            SafetyRating::Avoid => WarningLevel::Medium,
            SafetyRating::Banned => WarningLevel::High,
            SafetyRating::Unknown => WarningLevel::Unknown,
        }
    }

    /// Check if additive is relevant to user's allergies/restrictions
    fn is_relevant_to_user(&self, info: &AdditiveInfo) -> bool {
        // Check allergies
        for allergen in &self.user_allergies {
            if info.allergens.iter().any(|a| a.to_lowercase().contains(&allergen.to_lowercase())) {
                return true;
            }
        }
        
        // Check dietary restrictions
        for restriction in &self.dietary_restrictions {
            if !info.dietary_flags.contains(restriction) {
                return true;
            }
        }
        
        false
    }

    /// Calculate overall product safety score (0-100)
    fn calculate_product_safety(&self, additives: &[AdditiveAnalysis]) -> u8 {
        if additives.is_empty() {
            return 100;
        }
        
        let mut score = 100u8;
        
        for analysis in additives {
            match analysis.info.safety {
                SafetyRating::Banned => score = score.saturating_sub(30),
                SafetyRating::Avoid => score = score.saturating_sub(20),
                SafetyRating::Caution => score = score.saturating_sub(10),
                SafetyRating::Unknown => score = score.saturating_sub(5),
                SafetyRating::Safe => {},
            }
        }
        
        score
    }

    /// Generate warnings for detected additives
    fn generate_warnings(&self, additives: &[AdditiveAnalysis]) -> Vec<AdditiveWarning> {
        let mut warnings = Vec::new();
        
        for analysis in additives {
            if analysis.warning_level as u8 >= self.warning_threshold as u8 {
                warnings.push(AdditiveWarning {
                    e_number: analysis.info.e_number.clone(),
                    name: analysis.info.name.clone(),
                    message: self.create_warning_message(&analysis.info),
                    severity: analysis.info.safety,
                });
            }
            
            // Check for allergen-related warnings
            for allergen in &self.user_allergies {
                if analysis.info.allergens.iter().any(|a| a.to_lowercase().contains(&allergen.to_lowercase())) {
                    warnings.push(AdditiveWarning {
                        e_number: analysis.info.e_number.clone(),
                        name: analysis.info.name.clone(),
                        message: format!("May trigger your {} sensitivity", allergen),
                        severity: SafetyRating::Avoid,
                    });
                }
            }
        }
        
        warnings
    }

    /// Create warning message for additive
    fn create_warning_message(&self, info: &AdditiveInfo) -> String {
        match info.safety {
            SafetyRating::Banned => format!(
                "{} is banned in some countries. {}: {}",
                info.e_number, info.name, info.description
            ),
            SafetyRating::Avoid => format!(
                "{} ({}) should be avoided. {}",
                info.e_number, info.name, info.description
            ),
            SafetyRating::Caution => format!(
                "{} ({}) requires caution. {}",
                info.e_number, info.name, info.description
            ),
            SafetyRating::Unknown => format!(
                "{} ({}) has unknown safety profile.",
                info.e_number, info.name
            ),
            SafetyRating::Safe => format!(
                "{} ({}) is considered safe.",
                info.e_number, info.name
            ),
        }
    }

    /// Check dietary compliance
    fn check_dietary_compliance(&self, additives: &[AdditiveAnalysis]) -> HashMap<DietaryFlag, bool> {
        let mut compliance = HashMap::new();
        
        for flag in &self.dietary_restrictions {
            let is_compliant = additives.iter().all(|a| a.info.dietary_flags.contains(flag));
            compliance.insert(*flag, is_compliant);
        }
        
        compliance
    }

    /// Generate overall recommendation
    fn generate_recommendation(&self, additives: &[AdditiveAnalysis], safety_score: u8) -> Recommendation {
        if safety_score >= 80 {
            Recommendation::Safe
        } else if safety_score >= 60 {
            Recommendation::Moderate
        } else if safety_score >= 40 {
            Recommendation::Caution
        } else {
            Recommendation::Avoid
        }
    }

    /// Get detailed information about a specific additive
    pub fn get_additive_info(&self, e_number: &str) -> Option<&AdditiveInfo> {
        self.database.get_additive(e_number)
    }

    /// Analyze barcode (simulated - would connect to product database)
    pub fn analyze_barcode(&self, barcode: &str) -> BarcodeAnalysis {
        // In a real implementation, this would query a product database
        // For now, return a placeholder
        BarcodeAnalysis {
            barcode: barcode.to_string(),
            product_name: "Unknown Product".to_string(),
            brand: None,
            analysis: None,
            found: false,
        }
    }

    /// Get additives by category
    pub fn get_additives_by_category(&self, category: AdditiveCategory) -> Vec<&AdditiveInfo> {
        self.database.additives.values()
            .filter(|a| a.category == category)
            .collect()
    }

    /// Get all safe additives
    pub fn get_safe_additives(&self) -> Vec<&AdditiveInfo> {
        self.database.get_additives_by_safety(SafetyRating::Safe)
    }

    /// Get all additives to avoid
    pub fn get_additives_to_avoid(&self) -> Vec<&AdditiveInfo> {
        self.database.additives.values()
            .filter(|a| a.safety == SafetyRating::Avoid || a.safety == SafetyRating::Banned)
            .collect()
    }

    /// Quick scan mode for fast ingredient list parsing
    pub fn quick_scan(&self, ingredients: &str) -> QuickScanResult {
        let additives = self.extract_and_analyze_additives(ingredients);
        let dangerous_count = additives.iter()
            .filter(|a| matches!(a.info.safety, SafetyRating::Avoid | SafetyRating::Banned))
            .count();
        let caution_count = additives.iter()
            .filter(|a| a.info.safety == SafetyRating::Caution)
            .count();
        
        QuickScanResult {
            total_additives: additives.len(),
            safe_count: additives.len() - dangerous_count - caution_count,
            caution_count,
            dangerous_count,
            status: if dangerous_count > 0 {
                ScanStatus::Dangerous
            } else if caution_count > 0 {
                ScanStatus::Caution
            } else {
                ScanStatus::Safe
            },
        }
    }
}

impl Default for IngredX {
    fn default() -> Self {
        Self::new()
    }
}

/// Product analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductAnalysis {
    /// Original ingredients text
    pub ingredients_text: String,
    /// Detected additives with analysis
    pub additives: Vec<AdditiveAnalysis>,
    /// Overall safety score (0-100)
    pub safety_score: u8,
    /// Generated warnings
    pub warnings: Vec<AdditiveWarning>,
    /// Dietary compliance check
    pub dietary_compliance: HashMap<DietaryFlag, bool>,
    /// Overall recommendation
    pub recommendation: Recommendation,
}

/// Individual additive analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdditiveAnalysis {
    /// Additive information
    pub info: AdditiveInfo,
    /// Warning level
    pub warning_level: WarningLevel,
    /// Whether relevant to user
    pub user_relevant: bool,
}

/// Warning level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum WarningLevel {
    None = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Unknown = 4,
}

/// Additive warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdditiveWarning {
    /// E-number
    pub e_number: String,
    /// Additive name
    pub name: String,
    /// Warning message
    pub message: String,
    /// Severity
    pub severity: SafetyRating,
}

/// Overall recommendation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Recommendation {
    Safe,
    Moderate,
    Caution,
    Avoid,
}

/// Barcode analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarcodeAnalysis {
    /// Barcode number
    pub barcode: String,
    /// Product name if found
    pub product_name: String,
    /// Brand if found
    pub brand: Option<String>,
    /// Product analysis if found
    pub analysis: Option<ProductAnalysis>,
    /// Whether product was found in database
    pub found: bool,
}

/// Quick scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickScanResult {
    /// Total additives found
    pub total_additives: usize,
    /// Number of safe additives
    pub safe_count: usize,
    /// Number requiring caution
    pub caution_count: usize,
    /// Number to avoid
    pub dangerous_count: usize,
    /// Overall scan status
    pub status: ScanStatus,
}

/// Scan status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScanStatus {
    Safe,
    Caution,
    Dangerous,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ingred_x_creation() {
        let ingred = IngredX::new();
        assert!(ingred.get_additive_info("E330").is_some());
    }

    #[test]
    fn test_product_analysis() {
        let ingred = IngredX::new();
        let analysis = ingred.analyze_product("Ingredients: Water, Sugar, E330, E102, E250");
        
        assert!(!analysis.additives.is_empty());
        assert!(analysis.safety_score < 100);
    }

    #[test]
    fn test_quick_scan() {
        let ingred = IngredX::new();
        let result = ingred.quick_scan("E300, E320, E102");
        
        assert!(result.total_additives >= 2);
    }

    #[test]
    fn test_allergen_detection() {
        let mut ingred = IngredX::new();
        ingred.set_allergies(vec!["Aspirin sensitivity".to_string()]);
        
        let analysis = ingred.analyze_product("E102, E300");
        let has_allergen_warning = analysis.warnings.iter()
            .any(|w| w.message.contains("Aspirin"));
        
        assert!(has_allergen_warning);
    }

    #[test]
    fn test_dietary_compliance() {
        let mut ingred = IngredX::new();
        ingred.set_dietary_restrictions(vec![DietaryFlag::Vegan]);
        
        let analysis = ingred.analyze_product("E322, E471");
        assert!(analysis.dietary_compliance.contains_key(&DietaryFlag::Vegan));
    }
}