//! Lip Sync Processor - Viseme generation and lip-sync processing
//! 
//! This module provides lip-sync processing for video dubbing,
//! generating viseme sequences and timing data for realistic dubbing.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::models::*;
use super::mod::{LipSyncData, VisemeFrame};

/// Lip sync processor for generating viseme data
pub struct LipSyncProcessor {
    viseme_map: VisemeMap,
    settings: LipSyncSettings,
    stats: LipSyncStats,
}

impl LipSyncProcessor {
    /// Create a new lip sync processor
    pub fn new() -> Self {
        Self {
            viseme_map: VisemeMap::new(),
            settings: LipSyncSettings::default(),
            stats: LipSyncStats::default(),
        }
    }

    /// Generate lip sync data from video frames and audio
    pub fn generate(
        &mut self,
        video_frames: &[VideoFrame],
        audio_data: &[u8],
        timing: TranslationTiming,
    ) -> LipSyncData {
        let start = std::time::Instant::now();
        
        // Step 1: Extract phonemes from audio
        let phonemes = self.extract_phonemes(audio_data, &timing);
        
        // Step 2: Convert phonemes to visemes
        let viseme_sequence = self.phonemes_to_visemes(&phonemes);
        
        // Step 3: Align visemes with timing
        let aligned_visemes = self.align_visemes(viseme_sequence, &timing);
        
        // Step 4: Smooth transitions
        let smoothed = self.smooth_visemes(aligned_visemes);
        
        // Step 5: Generate confidence scores
        let confidence = self.calculate_confidence(&smoothed, &timing);
        
        // Step 6: Calculate timing offsets
        let timing_offsets = self.calculate_timing_offsets(&smoothed);
        
        self.stats.processing_count += 1;
        self.stats.total_frames_processed += video_frames.len() as u64;
        
        LipSyncData {
            visemes: smoothed,
            confidence,
            timing_offsets,
        }
    }

    /// Extract phonemes from audio
    fn extract_phonemes(&self, _audio_data: &[u8], timing: &TranslationTiming) -> Vec<Phoneme> {
        // In production, would use forced alignment model
        // Generate phonemes based on word timings
        let mut phonemes = Vec::new();
        
        for word_timing in &timing.word_timings {
            let word_phonemes = self.word_to_phonemes(&word_timing.word);
            let duration_per_phoneme = word_timing.duration_ms / word_phonemes.len() as u32;
            
            for (i, phoneme) in word_phonemes.iter().enumerate() {
                phonemes.push(Phoneme {
                    symbol: phoneme.clone(),
                    start_ms: word_timing.start_ms + (i as u32 * duration_per_phoneme),
                    duration_ms: duration_per_phoneme,
                    confidence: 0.9,
                });
            }
        }
        
        phonemes
    }

    /// Convert word to phonemes (simplified)
    fn word_to_phonemes(&self, word: &str) -> Vec<String> {
        // Simplified letter-to-phoneme mapping
        // In production, would use pronunciation dictionary or G2P model
        let word_lower = word.to_lowercase();
        let mut phonemes = Vec::new();
        
        let chars: Vec<char> = word_lower.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            // Check for digraphs
            if i + 1 < chars.len() {
                let digraph = format!("{}{}", chars[i], chars[i + 1]);
                if let Some(ph) = self.viseme_map.get_digraph_phoneme(&digraph) {
                    phonemes.push(ph.to_string());
                    i += 2;
                    continue;
                }
            }
            
            // Single character
            if let Some(ph) = self.viseme_map.get_char_phoneme(chars[i]) {
                phonemes.push(ph.to_string());
            }
            i += 1;
        }
        
        if phonemes.is_empty() {
            phonemes.push("sil".to_string()); // Silence
        }
        
        phonemes
    }

    /// Convert phonemes to visemes
    fn phonemes_to_visemes(&self, phonemes: &[Phoneme]) -> Vec<VisemeWithTiming> {
        phonemes.iter().map(|p| {
            let viseme = self.viseme_map.phoneme_to_viseme(&p.symbol);
            VisemeWithTiming {
                viseme: viseme.to_string(),
                start_ms: p.start_ms,
                duration_ms: p.duration_ms,
                intensity: 1.0,
            }
        }).collect()
    }

    /// Align visemes with timing
    fn align_visemes(&self, visemes: Vec<VisemeWithTiming>, timing: &TranslationTiming) -> Vec<VisemeWithTiming> {
        // Adjust viseme timing to match overall timing
        if visemes.is_empty() {
            return visemes;
        }
        
        let last_viseme_end = visemes.last().map(|v| v.start_ms + v.duration_ms).unwrap_or(0);
        let target_end = timing.end_ms;
        
        if last_viseme_end == 0 || target_end == 0 {
            return visemes;
        }
        
        let scale = target_end as f32 / last_viseme_end as f32;
        
        visemes.into_iter().map(|mut v| {
            v.start_ms = (v.start_ms as f32 * scale) as u32;
            v.duration_ms = (v.duration_ms as f32 * scale) as u32;
            v
        }).collect()
    }

    /// Smooth viseme transitions
    fn smooth_visemes(&self, visemes: Vec<VisemeWithTiming>) -> Vec<VisemeFrame> {
        visemes.into_iter().map(|v| {
            VisemeFrame {
                viseme: v.viseme,
                intensity: v.intensity,
                duration_ms: v.duration_ms,
            }
        }).collect()
    }

    /// Calculate confidence scores
    fn calculate_confidence(&self, visemes: &[VisemeFrame], timing: &TranslationTiming) -> Vec<f32> {
        visemes.iter().enumerate().map(|(i, _)| {
            // Higher confidence for visemes with good timing alignment
            let base_confidence = 0.85;
            let timing_bonus = if timing.word_timings.len() > i {
                0.1
            } else {
                0.0
            };
            (base_confidence + timing_bonus).min(1.0)
        }).collect()
    }

    /// Calculate timing offsets
    fn calculate_timing_offsets(&self, visemes: &[VisemeFrame]) -> Vec<u32> {
        let mut offsets = Vec::new();
        let mut current = 0u32;
        
        for viseme in visemes {
            offsets.push(current);
            current += viseme.duration_ms;
        }
        
        offsets
    }

    /// Generate keyframes for animation
    pub fn generate_keyframes(&self, lip_sync_data: &LipSyncData) -> Vec<AnimationKeyframe> {
        let mut keyframes = Vec::new();
        let mut current_time = 0u32;
        
        for (i, viseme) in lip_sync_data.visemes.iter().enumerate() {
            let blend_shapes = self.viseme_to_blend_shapes(&viseme.viseme, viseme.intensity);
            
            keyframes.push(AnimationKeyframe {
                time_ms: current_time,
                blend_shapes,
                transition_ms: self.settings.transition_duration_ms,
            });
            
            current_time += viseme.duration_ms;
        }
        
        keyframes
    }

    /// Convert viseme to blend shape values
    fn viseme_to_blend_shapes(&self, viseme: &str, intensity: f32) -> HashMap<String, f32> {
        let mut shapes = HashMap::new();
        
        // Standard blend shape mapping for common visemes
        let viseme_shapes: HashMap<&str, &[(&str, f32)]> = [
            ("A", &[("jawOpen", 1.0), ("mouthFunnel", 0.3)]),
            ("E", &[("jawOpen", 0.5), ("mouthStretch", 0.7)]),
            ("I", &[("mouthStretch", 0.5), ("mouthSmile", 0.3)]),
            ("O", &[("jawOpen", 0.7), ("mouthFunnel", 0.8)]),
            ("U", &[("mouthFunnel", 1.0), ("jawOpen", 0.3)]),
            ("BMP", &[("mouthClose", 1.0), ("mouthPucker", 0.5)]),
            ("FV", &[("mouthPucker", 0.3), ("jawOpen", 0.1)]),
            ("TH", &[("tongueOut", 0.5), ("jawOpen", 0.2)]),
            ("L", &[("tongueOut", 0.3), ("mouthStretch", 0.2)]),
            ("CHSH", &[("mouthFunnel", 0.5), ("jawOpen", 0.3)]),
            ("sil", &[]),
        ].iter().cloned().collect();
        
        if let Some(shape_list) = viseme_shapes.get(viseme) {
            for (name, value) in shape_list {
                shapes.insert(name.to_string(), value * intensity);
            }
        }
        
        shapes
    }

    /// Get settings
    pub fn settings(&self) -> &LipSyncSettings {
        &self.settings
    }

    /// Update settings
    pub fn update_settings(&mut self, settings: LipSyncSettings) {
        self.settings = settings;
    }

    /// Get statistics
    pub fn stats(&self) -> &LipSyncStats {
        &self.stats
    }
}

impl Default for LipSyncProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Viseme mapping
pub struct VisemeMap {
    /// Phoneme to viseme mapping
    phoneme_to_viseme_map: HashMap<String, String>,
    /// Character to phoneme mapping
    char_to_phoneme_map: HashMap<char, &'static str>,
    /// Digraph to phoneme mapping
    digraph_map: HashMap<String, &'static str>,
}

impl VisemeMap {
    pub fn new() -> Self {
        let mut phoneme_map = HashMap::new();
        
        // Standard Preston Blair visemes (10 viseme set)
        let mappings = [
            // Consonants
            ("p", "BMP"), ("b", "BMP"), ("m", "BMP"),
            ("f", "FV"), ("v", "FV"),
            ("θ", "TH"), ("ð", "TH"),
            ("t", "T"), ("d", "T"), ("n", "T"), ("s", "T"), ("z", "T"),
            ("l", "L"), ("r", "L"),
            ("ʧ", "CHSH"), ("ʃ", "CHSH"), ("ʤ", "CHSH"), ("ʒ", "CHSH"),
            ("k", "KG"), ("g", "KG"), ("ŋ", "KG"),
            ("h", "CHSH"),
            ("w", "WQ"), ("ʍ", "WQ"),
            ("j", "CHSH"),
            // Vowels
            ("ɑ", "A"), ("æ", "A"), ("ʌ", "A"),
            ("ɛ", "E"), ("e", "E"),
            ("ɪ", "I"), ("i", "I"),
            ("ɔ", "O"), ("ɒ", "O"), ("o", "O"),
            ("ʊ", "U"), ("u", "U"),
            ("ə", "E"), ("ɪə", "I"), ("eə", "E"), ("ʊə", "U"),
            ("aɪ", "A"), ("aʊ", "A"), ("ɔɪ", "O"),
            // Silence
            ("sil", "sil"), ("sp", "sil"),
        ];
        
        for (phoneme, viseme) in mappings {
            phoneme_map.insert(phoneme.to_string(), viseme.to_string());
        }
        
        let mut char_map = HashMap::new();
        let char_phonemes: [(char, &str); 26] = [
            ('a', "æ"), ('b', "b"), ('c', "k"), ('d', "d"), ('e', "ɛ"),
            ('f', "f"), ('g', "g"), ('h', "h"), ('i', "ɪ"), ('j', "j"),
            ('k', "k"), ('l', "l"), ('m', "m"), ('n', "n"), ('o', "o"),
            ('p', "p"), ('q', "k"), ('r', "r"), ('s', "s"), ('t', "t"),
            ('u', "ʊ"), ('v', "v"), ('w', "w"), ('x', "k"), ('y', "j"),
            ('z', "z"),
        ];
        
        for (c, p) in char_phonemes {
            char_map.insert(c, p);
        }
        
        let mut digraphs = HashMap::new();
        let digraph_list: [(&str, &str); 20] = [
            ("th", "θ"), ("sh", "ʃ"), ("ch", "ʧ"), ("ng", "ŋ"),
            ("ph", "f"), ("gh", "g"), ("ck", "k"), ("wh", "ʍ"),
            ("ti", "ʃ"), ("si", "ʃ"), ("ea", "i"), ("ee", "i"),
            ("oo", "u"), ("ou", "aʊ"), ("ai", "e"), ("ay", "e"),
            ("oi", "ɔɪ"), ("oy", "ɔɪ"), ("ar", "ɑ"), ("or", "ɔ"),
        ];
        
        for (d, p) in digraph_list {
            digraphs.insert(d.to_string(), p);
        }
        
        Self {
            phoneme_to_viseme_map: phoneme_map,
            char_to_phoneme_map: char_map,
            digraph_map: digraphs,
        }
    }

    /// Get phoneme for character
    pub fn get_char_phoneme(&self, c: char) -> Option<&'static str> {
        self.char_to_phoneme_map.get(&c).copied()
    }

    /// Get phoneme for digraph
    pub fn get_digraph_phoneme(&self, digraph: &str) -> Option<&'static str> {
        self.digraph_map.get(digraph).copied()
    }

    /// Convert phoneme to viseme
    pub fn phoneme_to_viseme(&self, phoneme: &str) -> &str {
        self.phoneme_to_viseme_map.get(phoneme)
            .map(|s| s.as_str())
            .unwrap_or("sil")
    }
}

impl Default for VisemeMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Phoneme with timing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phoneme {
    /// IPA symbol
    pub symbol: String,
    /// Start time in milliseconds
    pub start_ms: u32,
    /// Duration in milliseconds
    pub duration_ms: u32,
    /// Confidence
    pub confidence: f32,
}

/// Viseme with timing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisemeWithTiming {
    /// Viseme identifier
    pub viseme: String,
    /// Start time in milliseconds
    pub start_ms: u32,
    /// Duration in milliseconds
    pub duration_ms: u32,
    /// Intensity
    pub intensity: f32,
}

/// Lip sync settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LipSyncSettings {
    /// Transition duration between visemes
    pub transition_duration_ms: u32,
    /// Minimum viseme duration
    pub min_viseme_duration_ms: u32,
    /// Maximum viseme duration
    pub max_viseme_duration_ms: u32,
    /// Smoothing factor
    pub smoothing: f32,
    /// Enable coarticulation
    pub coarticulation: bool,
}

impl Default for LipSyncSettings {
    fn default() -> Self {
        Self {
            transition_duration_ms: 30,
            min_viseme_duration_ms: 20,
            max_viseme_duration_ms: 200,
            smoothing: 0.3,
            coarticulation: true,
        }
    }
}

/// Lip sync statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LipSyncStats {
    /// Processing count
    pub processing_count: u64,
    /// Total frames processed
    pub total_frames_processed: u64,
    /// Average processing time
    pub avg_processing_time_ms: f32,
}

/// Animation keyframe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationKeyframe {
    /// Time in milliseconds
    pub time_ms: u32,
    /// Blend shape values
    pub blend_shapes: HashMap<String, f32>,
    /// Transition duration
    pub transition_ms: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_creation() {
        let processor = LipSyncProcessor::new();
        assert!(processor.settings().transition_duration_ms > 0);
    }

    #[test]
    fn test_viseme_map() {
        let map = VisemeMap::new();
        
        assert_eq!(map.phoneme_to_viseme("p"), "BMP");
        assert_eq!(map.phoneme_to_viseme("f"), "FV");
        assert_eq!(map.phoneme_to_viseme("ɑ"), "A");
    }

    #[test]
    fn test_char_to_phoneme() {
        let map = VisemeMap::new();
        
        assert_eq!(map.get_char_phoneme('a'), Some("æ"));
        assert_eq!(map.get_char_phoneme('b'), Some("b"));
    }

    #[test]
    fn test_digraph_phoneme() {
        let map = VisemeMap::new();
        
        assert_eq!(map.get_digraph_phoneme("th"), Some("θ"));
        assert_eq!(map.get_digraph_phoneme("sh"), Some("ʃ"));
    }

    #[test]
    fn test_blend_shapes() {
        let processor = LipSyncProcessor::new();
        let shapes = processor.viseme_to_blend_shapes("A", 1.0);
        
        assert!(shapes.contains_key("jawOpen"));
        assert!((shapes.get("jawOpen").unwrap() - 1.0).abs() < 0.01);
    }
}