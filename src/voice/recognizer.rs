//! Voice Recognizer Module
//!
//! Handles speech recognition using Web Speech API or local engines.

use anyhow::{Result, Error};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Voice recognizer for speech-to-text
pub struct VoiceRecognizer {
    config: RecognizerConfig,
    is_listening: Arc<RwLock<bool>>,
    language: Arc<RwLock<String>>,
}

/// Recognizer configuration
#[derive(Debug, Clone)]
pub struct RecognizerConfig {
    /// Speech recognition engine
    pub engine: RecognitionEngine,
    /// Continuous listening mode
    pub continuous: bool,
    /// Interim results
    pub interim_results: bool,
    /// Maximum alternatives
    pub max_alternatives: u32,
    /// Enable auto-start
    pub auto_start: bool,
}

impl Default for RecognizerConfig {
    fn default() -> Self {
        Self {
            engine: RecognitionEngine::WebSpeech,
            continuous: false,
            interim_results: true,
            max_alternatives: 3,
            auto_start: false,
        }
    }
}

/// Recognition engine type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecognitionEngine {
    WebSpeech,
    Whisper,
    PocketSphinx,
}

/// Recognition result
#[derive(Debug, Clone)]
pub struct RecognitionResult {
    /// Transcribed text
    pub text: String,
    /// Confidence score (0-1)
    pub confidence: f32,
    /// Alternative transcriptions
    pub alternatives: Vec<RecognitionAlternative>,
    /// Whether this is an interim result
    pub is_final: bool,
}

/// Alternative transcription
#[derive(Debug, Clone)]
pub struct RecognitionAlternative {
    /// Alternative text
    pub text: String,
    /// Confidence score
    pub confidence: f32,
}

/// Recognition event
#[derive(Debug, Clone)]
pub enum RecognitionEvent {
    /// Started listening
    Started,
    /// Result received
    Result(RecognitionResult),
    /// Speech detected
    SpeechStart,
    /// Speech ended
    SpeechEnd,
    /// Error occurred
    Error(String),
    /// Stopped listening
    Stopped,
}

impl VoiceRecognizer {
    /// Create a new voice recognizer
    pub fn new(config: RecognizerConfig) -> Self {
        Self {
            config,
            is_listening: Arc::new(RwLock::new(false)),
            language: Arc::new(RwLock::new("en-US".to_string())),
        }
    }

    /// Start listening for speech
    pub async fn start(&self) -> Result<()> {
        let mut listening = self.is_listening.write().await;
        *listening = true;
        Ok(())
    }

    /// Stop listening
    pub async fn stop(&self) -> Result<()> {
        let mut listening = self.is_listening.write().await;
        *listening = false;
        Ok(())
    }

    /// Check if currently listening
    pub async fn is_listening(&self) -> bool {
        *self.is_listening.read().await
    }

    /// Set language for recognition
    pub async fn set_language(&self, language: &str) -> Result<()> {
        let mut lang = self.language.write().await;
        *lang = language.to_string();
        Ok(())
    }

    /// Get current language
    pub async fn get_language(&self) -> String {
        self.language.read().await.clone()
    }

    /// Recognize from audio data (for file-based input)
    pub async fn recognize_from_audio(&self, audio_data: &[u8]) -> Result<RecognitionResult> {
        // In a real implementation, this would process audio data
        // For now, return a mock result
        Ok(RecognitionResult {
            text: "hello world".to_string(),
            confidence: 0.95,
            alternatives: vec![],
            is_final: true,
        })
    }

    /// Get available languages
    pub fn available_languages(&self) -> Vec<Language> {
        vec![
            Language {
                code: "en-US".to_string(),
                name: "English (US)".to_string(),
            },
            Language {
                code: "en-GB".to_string(),
                name: "English (UK)".to_string(),
            },
            Language {
                code: "es-ES".to_string(),
                name: "Spanish (Spain)".to_string(),
            },
            Language {
                code: "fr-FR".to_string(),
                name: "French (France)".to_string(),
            },
            Language {
                code: "de-DE".to_string(),
                name: "German (Germany)".to_string(),
            },
            Language {
                code: "zh-CN".to_string(),
                name: "Chinese (Simplified)".to_string(),
            },
            Language {
                code: "ja-JP".to_string(),
                name: "Japanese (Japan)".to_string(),
            },
            Language {
                code: "ko-KR".to_string(),
                name: "Korean (South Korea)".to_string(),
            },
            Language {
                code: "pl-PL".to_string(),
                name: "Polish (Poland)".to_string(),
            },
            Language {
                code: "ru-RU".to_string(),
                name: "Russian (Russia)".to_string(),
            },
        ]
    }
}

/// Language information
#[derive(Debug, Clone)]
pub struct Language {
    pub code: String,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recognizer_config_default() {
        let config = RecognizerConfig::default();
        assert_eq!(config.engine, RecognitionEngine::WebSpeech);
        assert!(!config.continuous);
        assert!(config.interim_results);
    }

    #[test]
    fn test_recognition_result() {
        let result = RecognitionResult {
            text: "test".to_string(),
            confidence: 0.9,
            alternatives: vec![],
            is_final: true,
        };
        assert_eq!(result.text, "test");
        assert!(result.is_final);
    }

    #[tokio::test]
    async fn test_start_stop() {
        let recognizer = VoiceRecognizer::new(RecognizerConfig::default());
        assert!(!recognizer.is_listening().await);
        
        recognizer.start().await.unwrap();
        assert!(recognizer.is_listening().await);
        
        recognizer.stop().await.unwrap();
        assert!(!recognizer.is_listening().await);
    }

    #[tokio::test]
    async fn test_set_language() {
        let recognizer = VoiceRecognizer::new(RecognizerConfig::default());
        recognizer.set_language("es-ES").await.unwrap();
        assert_eq!(recognizer.get_language().await, "es-ES");
    }

    #[test]
    fn test_available_languages() {
        let recognizer = VoiceRecognizer::new(RecognizerConfig::default());
        let languages = recognizer.available_languages();
        assert!(!languages.is_empty());
        assert!(languages.iter().any(|l| l.code == "en-US"));
    }
}