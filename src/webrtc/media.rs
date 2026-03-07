//! # Media Streams Module
//!
//! Implements media stream capture and management for audio and video.

use anyhow::{Result, Error};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Media stream
///
/// Represents a collection of media tracks (audio and/or video).
///
/// # Examples
///
/// ```rust
/// use vantisweb::webrtc::media::MediaStream;
///
/// let stream = MediaStream::new("stream-0");
/// ```
#[derive(Debug, Clone)]
pub struct MediaStream {
    id: String,
    audio_tracks: Arc<RwLock<Vec<MediaTrack>>>,
    video_tracks: Arc<RwLock<Vec<MediaTrack>>>,
}

impl MediaStream {
    /// Create a new media stream
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            audio_tracks: Arc::new(RwLock::new(Vec::new())),
            video_tracks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add audio track
    pub async fn add_audio_track(&self, track: MediaTrack) {
        let mut tracks = self.audio_tracks.write().await;
        tracks.push(track);
    }

    /// Add video track
    pub async fn add_video_track(&self, track: MediaTrack) {
        let mut tracks = self.video_tracks.write().await;
        tracks.push(track);
    }

    /// Get stream ID
    pub fn get_id(&self) -> &str {
        &self.id
    }
}

/// Media track
///
/// Represents a single media track (audio or video).
///
/// # Examples
///
/// ```rust
/// use vantisweb::webrtc::media::MediaTrack;
///
/// let track = MediaTrack::new("audio", "audio-0");
/// ```
#[derive(Debug, Clone)]
pub struct MediaTrack {
    kind: String,
    id: String,
    enabled: Arc<RwLock<bool>>,
    muted: Arc<RwLock<bool>>,
}

impl MediaTrack {
    /// Create a new media track
    pub fn new(kind: &str, id: &str) -> Self {
        Self {
            kind: kind.to_string(),
            id: id.to_string(),
            enabled: Arc::new(RwLock::new(true)),
            muted: Arc::new(RwLock::new(false)),
        }
    }

    /// Get track kind
    pub fn get_kind(&self) -> &str {
        &self.kind
    }

    /// Get track ID
    pub fn get_id(&self) -> &str {
        &self.id
    }

    /// Enable track
    pub async fn enable(&self) {
        let mut enabled = self.enabled.write().await;
        *enabled = true;
    }

    /// Disable track
    pub async fn disable(&self) {
        let mut enabled = self.enabled.write().await;
        *enabled = false;
    }

    /// Mute track
    pub async fn mute(&self) {
        let mut muted = self.muted.write().await;
        *muted = true;
    }

    /// Unmute track
    pub async fn unmute(&self) {
        let mut muted = self.muted.write().await;
        *muted = false;
    }
}

/// Media constraints
///
/// Configuration for media capture.
#[derive(Debug, Clone, Default)]
pub struct MediaConstraints {
    pub audio: Option<AudioConstraints>,
    pub video: Option<VideoConstraints>,
}

/// Audio constraints
#[derive(Debug, Clone, Default)]
pub struct AudioConstraints {
    pub echo_cancellation: Option<bool>,
    pub noise_suppression: Option<bool>,
    pub auto_gain_control: Option<bool>,
}

/// Video constraints
#[derive(Debug, Clone, Default)]
pub struct VideoConstraints {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub frame_rate: Option<f64>,
    pub facing_mode: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_stream_create() {
        let stream = MediaStream::new("stream-0");
        assert_eq!(stream.get_id(), "stream-0");
    }

    #[test]
    fn test_media_track_create() {
        let track = MediaTrack::new("audio", "audio-0");
        assert_eq!(track.get_kind(), "audio");
        assert_eq!(track.get_id(), "audio-0");
    }

    #[tokio::test]
    async fn test_media_track_enable_disable() {
        let track = MediaTrack::new("audio", "audio-0");
        track.disable().await;
        track.enable().await;
    }
}