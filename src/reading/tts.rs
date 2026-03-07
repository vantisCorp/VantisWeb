//! Text-to-Speech Module
//! 
//! This module provides text-to-speech capabilities for articles,
//! allowing users to listen to content while reading or on the go.
//! 
//! # Features
//! - Text-to-speech conversion
//! - Voice selection
//! - Speed and pitch control
//! - Reading position synchronization
//! - Playback controls

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Text-to-speech engine
pub struct TextToSpeech {
    config: Arc<RwLock<TTSConfig>>,
    is_playing: Arc<RwLock<bool>>,
    current_position: Arc<RwLock<usize>>,
    current_article_id: Arc<RwLock<Option<String>>>,
}

/// TTS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTSConfig {
    pub enabled: bool,
    pub voice: Option<String>,
    pub rate: f32,
    pub pitch: f32,
    pub volume: f32,
    pub auto_scroll: bool,
    pub highlight_current_sentence: bool,
}

impl Default for TTSConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            voice: None,
            rate: 1.0,
            pitch: 1.0,
            volume: 1.0,
            auto_scroll: true,
            highlight_current_sentence: true,
        }
    }
}

impl TTSConfig {
    /// Create preset for slow reading
    pub fn slow() -> Self {
        Self {
            enabled: true,
            voice: None,
            rate: 0.75,
            pitch: 1.0,
            volume: 1.0,
            auto_scroll: true,
            highlight_current_sentence: true,
        }
    }

    /// Create preset for fast reading
    pub fn fast() -> Self {
        Self {
            enabled: true,
            voice: None,
            rate: 1.5,
            pitch: 1.0,
            volume: 1.0,
            auto_scroll: true,
            highlight_current_sentence: false,
        }
    }

    /// Create preset for accessibility
    pub fn accessibility() -> Self {
        Self {
            enabled: true,
            voice: None,
            rate: 0.9,
            pitch: 1.0,
            volume: 1.0,
            auto_scroll: true,
            highlight_current_sentence: true,
        }
    }
}

/// Voice information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Voice {
    pub name: String,
    pub language: String,
    pub gender: VoiceGender,
    pub is_default: bool,
    pub is_local: bool,
}

/// Voice gender
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceGender {
    Male,
    Female,
    Neutral,
}

/// Playback state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
    Loading,
}

impl TextToSpeech {
    /// Create a new TTS engine
    pub fn new(config: TTSConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            is_playing: Arc::new(RwLock::new(false)),
            current_position: Arc::new(RwLock::new(0)),
            current_article_id: Arc::new(RwLock::new(None)),
        }
    }

    /// Get available voices
    pub fn get_available_voices(&self) -> Vec<Voice> {
        // In a real implementation, this would query the browser's TTS API
        vec![
            Voice {
                name: "English (US) - Female".to_string(),
                language: "en-US".to_string(),
                gender: VoiceGender::Female,
                is_default: true,
                is_local: true,
            },
            Voice {
                name: "English (US) - Male".to_string(),
                language: "en-US".to_string(),
                gender: VoiceGender::Male,
                is_default: false,
                is_local: true,
            },
            Voice {
                name: "English (UK) - Female".to_string(),
                language: "en-GB".to_string(),
                gender: VoiceGender::Female,
                is_default: false,
                is_local: true,
            },
        ]
    }

    /// Start reading an article
    pub async fn start_reading(&self, article_id: &str, text: &str) -> Result<TTSError> {
        *self.current_article_id.write().await = Some(article_id.to_string());
        *self.current_position.write().await = 0;
        *self.is_playing.write().await = true;

        // In a real implementation, this would call the browser's TTS API
        // For now, we'll simulate the interface
        Ok(())
    }

    /// Stop reading
    pub async fn stop(&self) {
        *self.is_playing.write().await = false;
        *self.current_position.write().await = 0;
    }

    /// Pause reading
    pub async fn pause(&self) {
        *self.is_playing.write().await = false;
    }

    /// Resume reading
    pub async fn resume(&self) {
        *self.is_playing.write().await = true;
    }

    /// Check if currently playing
    pub async fn is_playing(&self) -> bool {
        *self.is_playing.read().await
    }

    /// Get current playback state
    pub async fn get_playback_state(&self) -> PlaybackState {
        if *self.is_playing.read().await {
            PlaybackState::Playing
        } else if *self.current_article_id.read().await.is_some() {
            PlaybackState::Paused
        } else {
            PlaybackState::Stopped
        }
    }

    /// Set reading position
    pub async fn set_position(&self, position: usize) {
        *self.current_position.write().await = position;
    }

    /// Get current reading position
    pub async fn get_position(&self) -> usize {
        *self.current_position.read().await
    }

    /// Get current article ID
    pub async fn get_current_article_id(&self) -> Option<String> {
        self.current_article_id.read().await.clone()
    }

    /// Update TTS configuration
    pub async fn update_config(&self, config: TTSConfig) {
        *self.config.write().await = config;
    }

    /// Get current configuration
    pub async fn get_config(&self) -> TTSConfig {
        self.config.read().await.clone()
    }

    /// Set voice
    pub async fn set_voice(&self, voice: String) {
        let mut config = self.config.write().await;
        config.voice = Some(voice);
    }

    /// Set playback rate
    pub async fn set_rate(&self, rate: f32) {
        let mut config = self.config.write().await;
        config.rate = rate.clamp(0.5, 2.0);
    }

    /// Set pitch
    pub async fn set_pitch(&self, pitch: f32) {
        let mut config = self.config.write().await;
        config.pitch = pitch.clamp(0.5, 2.0);
    }

    /// Set volume
    pub async fn set_volume(&self, volume: f32) {
        let mut config = self.config.write().await;
        config.volume = volume.clamp(0.0, 1.0);
    }

    /// Get estimated reading time for text
    pub fn estimate_reading_time(&self, text: &str) -> u32 {
        let word_count = text.split_whitespace().count() as f32;
        let rate = if let Ok(config) = self.config.try_read() {
            config.rate
        } else {
            1.0
        };

        // Average speaking rate: 150 words per minute
        let base_rate = 150.0 * rate;
        ((word_count / base_rate) * 60.0).ceil() as u32
    }

    /// Split text into sentences for TTS
    pub fn split_into_sentences(&self, text: &str) -> Vec<String> {
        let mut sentences = Vec::new();
        let mut current_sentence = String::new();
        let mut chars = text.chars().peekable();

        while let Some(c) = chars.next() {
            current_sentence.push(c);

            // Check for sentence boundaries
            if c == '.' || c == '!' || c == '?' {
                // Check if next char is whitespace or end of string
                if let Some(&next_char) = chars.peek() {
                    if next_char.is_whitespace() {
                        // Add current sentence
                        let sentence = current_sentence.trim().to_string();
                        if !sentence.is_empty() {
                            sentences.push(sentence);
                        }
                        current_sentence.clear();
                    }
                } else {
                    // End of string
                    let sentence = current_sentence.trim().to_string();
                    if !sentence.is_empty() {
                        sentences.push(sentence);
                    }
                }
            }
        }

        // Add any remaining text
        let sentence = current_sentence.trim().to_string();
        if !sentence.is_empty() {
            sentences.push(sentence);
        }

        sentences
    }

    /// Get reading progress percentage
    pub async fn get_progress(&self) -> f64 {
        // In a real implementation, this would track actual position
        0.0
    }

    /// Jump to next sentence
    pub async fn next_sentence(&self) {
        // In a real implementation, this would skip to the next sentence
    }

    /// Jump to previous sentence
    pub async fn previous_sentence(&self) {
        // In a real implementation, this would skip to the previous sentence
    }

    /// Rewind by specified seconds
    pub async fn rewind(&self, seconds: u32) {
        // In a real implementation, this would rewind playback
    }

    /// Fast forward by specified seconds
    pub async fn fast_forward(&self, seconds: u32) {
        // In a real implementation, this would fast forward playback
    }

    /// Get reading statistics
    pub async fn get_statistics(&self) -> TTSStatistics {
        TTSStatistics {
            total_articles_read: 0,
            total_reading_time_minutes: 0,
            average_reading_rate: self.config.read().await.rate,
            most_used_voice: self.config.read().await.voice.clone(),
        }
    }
}

impl Default for TextToSpeech {
    fn default() -> Self {
        Self::new(TTSConfig::default())
    }
}

/// TTS error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTSError {
    pub code: String,
    pub message: String,
}

impl TTSError {
    /// Create a new TTS error
    pub fn new(code: String, message: String) -> Self {
        Self { code, message }
    }
}

impl Result<()> for TTSError {
    type Error = TTSError;

    fn from(value: TTSError) -> Self::Error {
        value
    }
}

/// TTS statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTSStatistics {
    pub total_articles_read: usize,
    pub total_reading_time_minutes: u32,
    pub average_reading_rate: f32,
    pub most_used_voice: Option<String>,
}

/// Custom Result type for TTS operations
pub type Result<T> = std::result::Result<T, TTSError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_into_sentences() {
        let tts = TextToSpeech::new(TTSConfig::default());
        
        let text = "This is a sentence. This is another sentence! And a third one?";
        let sentences = tts.split_into_sentences(text);
        
        assert_eq!(sentences.len(), 3);
        assert!(sentences[0].contains("sentence"));
        assert!(sentences[1].contains("another"));
        assert!(sentences[2].contains("third"));
    }

    #[test]
    fn test_estimate_reading_time() {
        let tts = TextToSpeech::new(TTSConfig::default());
        let text = "one two three four five";
        
        // 5 words at 150 wpm = 2 seconds
        let time = tts.estimate_reading_time(text);
        assert!(time < 10);
    }

    #[test]
    fn test_tts_config_presets() {
        let slow = TTSConfig::slow();
        assert!(slow.rate < 1.0);

        let fast = TTSConfig::fast();
        assert!(fast.rate > 1.0);
    }
}