//! Language support module
//! 
//! Provides language detection, translation, and speech synthesis support.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::models::{Language, TranslationResult, WordTiming, WordAlignment, EmotionSegment, Emotion, TranslationTiming};

/// Language support manager
pub struct LanguageManager {
    supported_languages: Vec<Language>,
    detection_cache: HashMap<String, String>,
    translation_cache: HashMap<String, TranslationResult>,
}

impl LanguageManager {
    /// Create a new language manager
    pub fn new() -> Self {
        Self {
            supported_languages: Language::all(),
            detection_cache: HashMap::new(),
            translation_cache: HashMap::new(),
        }
    }

    /// Get all supported languages
    pub fn get_supported_languages(&self) -> &[Language] {
        &self.supported_languages
    }

    /// Check if a language is supported
    pub fn is_language_supported(&self, code: &str) -> bool {
        self.supported_languages.iter().any(|l| l.code == code)
    }

    /// Get language by code
    pub fn get_language(&self, code: &str) -> Option<&Language> {
        self.supported_languages.iter().find(|l| l.code == code)
    }

    /// Detect language from text
    pub fn detect_language(&mut self, text: &str) -> Option<String> {
        // Check cache first
        let cache_key = self.get_text_hash(text);
        if let Some(cached) = self.detection_cache.get(&cache_key) {
            return Some(cached.clone());
        }

        // Simplified language detection based on character patterns
        let detected = self.detect_from_patterns(text);
        
        if let Some(ref lang) = detected {
            self.detection_cache.insert(cache_key, lang.clone());
        }
        
        detected
    }

    /// Detect language from text patterns
    fn detect_from_patterns(&self, text: &str) -> Option<String> {
        let text_lower = text.to_lowercase();
        
        // Chinese characters
        if self.has_chinese_chars(&text_lower) {
            return Some("zh".to_string());
        }
        
        // Japanese characters
        if self.has_japanese_chars(&text_lower) {
            return Some("ja".to_string());
        }
        
        // Korean characters
        if self.has_korean_chars(&text_lower) {
            return Some("ko".to_string());
        }
        
        // Arabic characters
        if self.has_arabic_chars(&text_lower) {
            return Some("ar".to_string());
        }
        
        // Hebrew characters
        if self.has_hebrew_chars(&text_lower) {
            return Some("he".to_string());
        }
        
        // Thai characters
        if self.has_thai_chars(&text_lower) {
            return Some("th".to_string());
        }
        
        // Hindi characters
        if self.has_hindi_chars(&text_lower) {
            return Some("hi".to_string());
        }
        
        // Russian (Cyrillic) characters
        if self.has_cyrillic_chars(&text_lower) {
            return Some("ru".to_string());
        }
        
        // Greek characters
        if self.has_greek_chars(&text_lower) {
            return Some("el".to_string());
        }
        
        // For Latin-based languages, use common words
        self.detect_latin_language(&text_lower)
    }

    fn has_chinese_chars(&self, text: &str) -> bool {
        text.chars().any(|c| ('\u{4E00}'..='\u{9FFF}').contains(&c))
    }

    fn has_japanese_chars(&self, text: &str) -> bool {
        text.chars().any(|c| {
            ('\u{3040}'..='\u{309F}').contains(&c) || // Hiragana
            ('\u{30A0}'..='\u{30FF}').contains(&c)     // Katakana
        })
    }

    fn has_korean_chars(&self, text: &str) -> bool {
        text.chars().any(|c| ('\u{AC00}'..='\u{D7AF}').contains(&c))
    }

    fn has_arabic_chars(&self, text: &str) -> bool {
        text.chars().any(|c| ('\u{0600}'..='\u{06FF}').contains(&c))
    }

    fn has_hebrew_chars(&self, text: &str) -> bool {
        text.chars().any(|c| ('\u{0590}'..='\u{05FF}').contains(&c))
    }

    fn has_thai_chars(&self, text: &str) -> bool {
        text.chars().any(|c| ('\u{0E00}'..='\u{0E7F}').contains(&c))
    }

    fn has_hindi_chars(&self, text: &str) -> bool {
        text.chars().any(|c| ('\u{0900}'..='\u{097F}').contains(&c))
    }

    fn has_cyrillic_chars(&self, text: &str) -> bool {
        text.chars().any(|c| ('\u{0400}'..='\u{04FF}').contains(&c))
    }

    fn has_greek_chars(&self, text: &str) -> bool {
        text.chars().any(|c| ('\u{0370}'..='\u{03FF}').contains(&c))
    }

    fn detect_latin_language(&self, text: &str) -> Option<String> {
        // Common word patterns for language detection
        let patterns: HashMap<&str, &str> = [
            ("the", "en"), ("is", "en"), ("and", "en"), ("that", "en"),
            ("el", "es"), ("la", "es"), ("que", "es"), ("y", "es"),
            ("le", "fr"), ("la", "fr"), ("et", "fr"), ("que", "fr"),
            ("der", "de"), ("die", "de"), ("und", "de"), ("das", "de"),
            ("il", "it"), ("la", "it"), ("che", "it"), ("e", "it"),
            ("o", "pt"), ("a", "pt"), ("que", "pt"), ("e", "pt"),
            ("de", "nl"), ("het", "nl"), ("een", "nl"), ("en", "nl"),
            ("och", "sv"), ("att", "sv"), ("det", "sv"), ("är", "sv"),
            ("og", "da"), ("det", "da"), ("er", "da"), ("en", "da"),
            ("ja", "fi"), ("on", "fi"), "ei", "fi"), ("että", "fi"),
            ("i", "pl"), ("nie", "pl"), ("to", "pl"), ("na", "pl"),
            ("și", "ro"), ("de", "ro"), ("la", "ro"), ("nu", "ro"),
            ("a", "cs"), ("je", "cs"), ("na", "cs"), ("ne", "cs"),
            ("és", "hu"), ("a", "hu"), ("nem", "hu"), ("hogy", "hu"),
            ("ve", "tr"), ("bir", "tr"), ("bu", "tr"), ("ve", "tr"),
            ("і", "uk"), ("на", "uk"), ("не", "uk"), ("що", "uk"),
        ].iter().cloned().collect();

        let words: Vec<&str> = text.split_whitespace().take(20).collect();
        let mut scores: HashMap<String, u32> = HashMap::new();

        for word in &words {
            if let Some(lang) = patterns.get(word) {
                *scores.entry(lang.to_string()).or_insert(0) += 1;
            }
        }

        scores.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(lang, _)| lang)
    }

    /// Get text hash for caching
    fn get_text_hash(&self, text: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Get language pair translation quality
    pub fn get_translation_quality(&self, source: &str, target: &str) -> u8 {
        let source_lang = self.get_language(source);
        let target_lang = self.get_language(target);
        
        match (source_lang, target_lang) {
            (Some(s), Some(t)) => {
                // Quality is the average of both languages' quality
                ((s.translation_quality as u16 + t.translation_quality as u16) / 2) as u8
            }
            _ => 50, // Default quality for unknown languages
        }
    }

    /// Check if text needs translation
    pub fn needs_translation(&self, text: &str, target_lang: &str) -> bool {
        if let Some(detected) = self.detect_language(text) {
            detected != target_lang
        } else {
            true // Unknown language, assume translation needed
        }
    }

    /// Get language direction (LTR or RTL)
    pub fn get_text_direction(&self, lang_code: &str) -> TextDirection {
        if let Some(lang) = self.get_language(lang_code) {
            if lang.rtl {
                TextDirection::RTL
            } else {
                TextDirection::LTR
            }
        } else {
            TextDirection::LTR
        }
    }

    /// Clear caches
    pub fn clear_caches(&mut self) {
        self.detection_cache.clear();
        self.translation_cache.clear();
    }
}

impl Default for LanguageManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Text direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextDirection {
    LTR,
    RTL,
}

/// Language statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LanguageStats {
    pub detection_count: u64,
    pub translation_count: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_manager_creation() {
        let manager = LanguageManager::new();
        assert!(manager.get_supported_languages().len() >= 30);
    }

    #[test]
    fn test_language_detection_english() {
        let mut manager = LanguageManager::new();
        let detected = manager.detect_language("The quick brown fox jumps over the lazy dog");
        assert_eq!(detected, Some("en".to_string()));
    }

    #[test]
    fn test_language_detection_chinese() {
        let mut manager = LanguageManager::new();
        let detected = manager.detect_language("你好世界");
        assert_eq!(detected, Some("zh".to_string()));
    }

    #[test]
    fn test_language_detection_japanese() {
        let mut manager = LanguageManager::new();
        let detected = manager.detect_language("こんにちは世界");
        assert_eq!(detected, Some("ja".to_string()));
    }

    #[test]
    fn test_language_detection_russian() {
        let mut manager = LanguageManager::new();
        let detected = manager.detect_language("Привет мир");
        assert_eq!(detected, Some("ru".to_string()));
    }

    #[test]
    fn test_rtl_detection() {
        let manager = LanguageManager::new();
        assert_eq!(manager.get_text_direction("ar"), TextDirection::RTL);
        assert_eq!(manager.get_text_direction("en"), TextDirection::LTR);
    }
}