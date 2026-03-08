//! Data models for live dubbing module

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported language
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Language {
    /// ISO 639-1 code
    pub code: String,
    /// English name
    pub name: String,
    /// Native name
    pub native_name: String,
    /// RTL (right-to-left) flag
    pub rtl: bool,
    /// Speech synthesis available
    pub tts_available: bool,
    /// Translation quality score (0-100)
    pub translation_quality: u8,
}

impl Language {
    /// Get all supported languages
    pub fn all() -> Vec<Self> {
        vec![
            Self { code: "en".into(), name: "English".into(), native_name: "English".into(), rtl: false, tts_available: true, translation_quality: 100 },
            Self { code: "es".into(), name: "Spanish".into(), native_name: "Español".into(), rtl: false, tts_available: true, translation_quality: 98 },
            Self { code: "fr".into(), name: "French".into(), native_name: "Français".into(), rtl: false, tts_available: true, translation_quality: 98 },
            Self { code: "de".into(), name: "German".into(), native_name: "Deutsch".into(), rtl: false, tts_available: true, translation_quality: 97 },
            Self { code: "it".into(), name: "Italian".into(), native_name: "Italiano".into(), rtl: false, tts_available: true, translation_quality: 96 },
            Self { code: "pt".into(), name: "Portuguese".into(), native_name: "Português".into(), rtl: false, tts_available: true, translation_quality: 96 },
            Self { code: "ru".into(), name: "Russian".into(), native_name: "Русский".into(), rtl: false, tts_available: true, translation_quality: 95 },
            Self { code: "zh".into(), name: "Chinese".into(), native_name: "中文".into(), rtl: false, tts_available: true, translation_quality: 94 },
            Self { code: "ja".into(), name: "Japanese".into(), native_name: "日本語".into(), rtl: false, tts_available: true, translation_quality: 94 },
            Self { code: "ko".into(), name: "Korean".into(), native_name: "한국어".into(), rtl: false, tts_available: true, translation_quality: 93 },
            Self { code: "ar".into(), name: "Arabic".into(), native_name: "العربية".into(), rtl: true, tts_available: true, translation_quality: 92 },
            Self { code: "hi".into(), name: "Hindi".into(), native_name: "हिन्दी".into(), rtl: false, tts_available: true, translation_quality: 91 },
            Self { code: "tr".into(), name: "Turkish".into(), native_name: "Türkçe".into(), rtl: false, tts_available: true, translation_quality: 90 },
            Self { code: "pl".into(), name: "Polish".into(), native_name: "Polski".into(), rtl: false, tts_available: true, translation_quality: 90 },
            Self { code: "nl".into(), name: "Dutch".into(), native_name: "Nederlands".into(), rtl: false, tts_available: true, translation_quality: 92 },
            Self { code: "sv".into(), name: "Swedish".into(), native_name: "Svenska".into(), rtl: false, tts_available: true, translation_quality: 91 },
            Self { code: "da".into(), name: "Danish".into(), native_name: "Dansk".into(), rtl: false, tts_available: true, translation_quality: 90 },
            Self { code: "fi".into(), name: "Finnish".into(), native_name: "Suomi".into(), rtl: false, tts_available: true, translation_quality: 89 },
            Self { code: "no".into(), name: "Norwegian".into(), native_name: "Norsk".into(), rtl: false, tts_available: true, translation_quality: 90 },
            Self { code: "th".into(), name: "Thai".into(), native_name: "ไทย".into(), rtl: false, tts_available: true, translation_quality: 85 },
            Self { code: "vi".into(), name: "Vietnamese".into(), native_name: "Tiếng Việt".into(), rtl: false, tts_available: true, translation_quality: 86 },
            Self { code: "id".into(), name: "Indonesian".into(), native_name: "Bahasa Indonesia".into(), rtl: false, tts_available: true, translation_quality: 88 },
            Self { code: "ms".into(), name: "Malay".into(), native_name: "Bahasa Melayu".into(), rtl: false, tts_available: true, translation_quality: 87 },
            Self { code: "uk".into(), name: "Ukrainian".into(), native_name: "Українська".into(), rtl: false, tts_available: true, translation_quality: 89 },
            Self { code: "cs".into(), name: "Czech".into(), native_name: "Čeština".into(), rtl: false, tts_available: true, translation_quality: 88 },
            Self { code: "ro".into(), name: "Romanian".into(), native_name: "Română".into(), rtl: false, tts_available: true, translation_quality: 87 },
            Self { code: "hu".into(), name: "Hungarian".into(), native_name: "Magyar".into(), rtl: false, tts_available: true, translation_quality: 86 },
            Self { code: "el".into(), name: "Greek".into(), native_name: "Ελληνικά".into(), rtl: false, tts_available: true, translation_quality: 87 },
            Self { code: "he".into(), name: "Hebrew".into(), native_name: "עברית".into(), rtl: true, tts_available: true, translation_quality: 88 },
            Self { code: "bn".into(), name: "Bengali".into(), native_name: "বাংলা".into(), rtl: false, tts_available: true, translation_quality: 82 },
        ]
    }

    /// Get language by code
    pub fn from_code(code: &str) -> Option<Self> {
        Self::all().into_iter().find(|l| l.code == code)
    }
}

/// Translation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResult {
    /// Original text
    pub original: String,
    /// Translated text
    pub translated: String,
    /// Source language
    pub source_language: String,
    /// Target language
    pub target_language: String,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Timing information
    pub timing: TranslationTiming,
    /// Word-level alignments
    pub alignments: Vec<WordAlignment>,
    /// Detected emotions
    pub emotions: Vec<EmotionSegment>,
}

/// Translation timing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationTiming {
    /// Start time in milliseconds
    pub start_ms: u32,
    /// End time in milliseconds
    pub end_ms: u32,
    /// Word timings
    pub word_timings: Vec<WordTiming>,
}

/// Word timing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordTiming {
    /// Word
    pub word: String,
    /// Start time in milliseconds
    pub start_ms: u32,
    /// Duration in milliseconds
    pub duration_ms: u32,
}

/// Word alignment between source and target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordAlignment {
    /// Source word index
    pub source_index: usize,
    /// Target word indices
    pub target_indices: Vec<usize>,
    /// Alignment confidence
    pub confidence: f32,
}

/// Emotion segment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionSegment {
    /// Emotion type
    pub emotion: Emotion,
    /// Start time in milliseconds
    pub start_ms: u32,
    /// Duration in milliseconds
    pub duration_ms: u32,
    /// Intensity (0.0-1.0)
    pub intensity: f32,
}

/// Emotion types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Emotion {
    Neutral,
    Happy,
    Sad,
    Angry,
    Fearful,
    Surprised,
    Disgusted,
    Excited,
    Calm,
}

/// Voice synthesis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisResult {
    /// Audio data
    pub audio: Vec<u8>,
    /// Sample rate
    pub sample_rate: u32,
    /// Duration in milliseconds
    pub duration_ms: u32,
    /// Voice ID used
    pub voice_id: String,
    /// Quality score
    pub quality_score: f32,
}

/// Voice profile for synthesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceProfile {
    /// Unique voice ID
    pub id: String,
    /// Voice name
    pub name: String,
    /// Language
    pub language: String,
    /// Gender
    pub gender: VoiceGender,
    /// Age category
    pub age: VoiceAge,
    /// Voice characteristics
    pub characteristics: VoiceCharacteristics,
    /// Preview audio URL
    pub preview_url: Option<String>,
    /// Is custom cloned voice
    pub is_custom: bool,
}

/// Voice gender
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceGender {
    Male,
    Female,
    Neutral,
}

/// Voice age category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceAge {
    Child,
    Young,
    Adult,
    Senior,
}

/// Voice characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceCharacteristics {
    /// Pitch (0.0-2.0, 1.0 = normal)
    pub pitch: f32,
    /// Speed (0.5-2.0, 1.0 = normal)
    pub speed: f32,
    /// Volume (0.0-1.0)
    pub volume: f32,
    /// Breathiness (0.0-1.0)
    pub breathiness: f32,
    /// Roughness (0.0-1.0)
    pub roughness: f32,
    /// Warmth (0.0-1.0)
    pub warmth: f32,
}

impl Default for VoiceCharacteristics {
    fn default() -> Self {
        Self {
            pitch: 1.0,
            speed: 1.0,
            volume: 0.8,
            breathiness: 0.0,
            roughness: 0.0,
            warmth: 0.5,
        }
    }
}

/// Voice settings for synthesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceSettings {
    /// Stability (0.0-1.0)
    pub stability: f32,
    /// Similarity boost (0.0-1.0)
    pub similarity_boost: f32,
    /// Style (0.0-1.0)
    pub style: f32,
    /// Use speaker boost
    pub use_speaker_boost: bool,
}

impl Default for VoiceSettings {
    fn default() -> Self {
        Self {
            stability: 0.5,
            similarity_boost: 0.75,
            style: 0.0,
            use_speaker_boost: true,
        }
    }
}

/// Audio processing config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Sample rate
    pub sample_rate: u32,
    /// Channels
    pub channels: u16,
    /// Bit depth
    pub bit_depth: u16,
    /// Enable noise reduction
    pub noise_reduction: bool,
    /// Enable normalization
    pub normalization: bool,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            channels: 1,
            bit_depth: 16,
            noise_reduction: true,
            normalization: true,
        }
    }
}

/// Cluster node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterNode {
    /// Node ID
    pub id: String,
    /// Node name
    pub name: String,
    /// Node type
    pub node_type: NodeType,
    /// Processing capacity (operations per second)
    pub capacity: u32,
    /// Current load (0-100)
    pub load: u8,
    /// Status
    pub status: NodeStatus,
    /// Latency in milliseconds
    pub latency_ms: u32,
    /// Supported features
    pub features: Vec<String>,
}

/// Node type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    /// Local device
    Local,
    /// Remote server
    Remote,
    /// Edge device
    Edge,
    /// Cloud instance
    Cloud,
}

/// Node status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    Online,
    Offline,
    Busy,
    Maintenance,
}

/// Processing task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingTask {
    /// Task ID
    pub id: String,
    /// Task type
    pub task_type: TaskType,
    /// Priority (1-10)
    pub priority: u8,
    /// Input data reference
    pub input_ref: String,
    /// Output data reference
    pub output_ref: Option<String>,
    /// Status
    pub status: TaskStatus,
    /// Progress (0-100)
    pub progress: u8,
    /// Created timestamp
    pub created: u64,
    /// Completed timestamp
    pub completed: Option<u64>,
    /// Error message
    pub error: Option<String>,
}

/// Task type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    Translation,
    VoiceSynthesis,
    LipSync,
    AudioProcessing,
    VideoProcessing,
    FullDubbing,
}

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Dubbing presets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DubbingPreset {
    /// Preset name
    pub name: String,
    /// Description
    pub description: String,
    /// Settings
    pub settings: HashMap<String, String>,
    /// Quality level
    pub quality: super::ProcessingQuality,
    /// Estimated processing time multiplier
    pub time_multiplier: f32,
}

impl DubbingPreset {
    /// Get all presets
    pub fn all() -> Vec<Self> {
        vec![
            Self {
                name: "Fast".into(),
                description: "Quick processing with good quality".into(),
                settings: HashMap::new(),
                quality: super::ProcessingQuality::Medium,
                time_multiplier: 0.5,
            },
            Self {
                name: "Balanced".into(),
                description: "Optimal balance between speed and quality".into(),
                settings: HashMap::new(),
                quality: super::ProcessingQuality::High,
                time_multiplier: 1.0,
            },
            Self {
                name: "Quality".into(),
                description: "Maximum quality processing".into(),
                settings: HashMap::new(),
                quality: super::ProcessingQuality::Ultra,
                time_multiplier: 2.0,
            },
            Self {
                name: "Live".into(),
                description: "Real-time processing for live streams".into(),
                settings: HashMap::new(),
                quality: super::ProcessingQuality::Low,
                time_multiplier: 0.25,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_list() {
        let languages = Language::all();
        assert!(languages.len() >= 30);
    }

    #[test]
    fn test_language_from_code() {
        let en = Language::from_code("en");
        assert!(en.is_some());
        assert_eq!(en.unwrap().name, "English");
    }

    #[test]
    fn test_rtl_languages() {
        let languages = Language::all();
        let rtl_count = languages.iter().filter(|l| l.rtl).count();
        assert!(rtl_count >= 2); // Arabic and Hebrew at minimum
    }

    #[test]
    fn test_voice_characteristics_default() {
        let vc = VoiceCharacteristics::default();
        assert!((vc.pitch - 1.0).abs() < 0.01);
        assert!((vc.speed - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_dubbing_presets() {
        let presets = DubbingPreset::all();
        assert!(presets.len() >= 4);
    }
}