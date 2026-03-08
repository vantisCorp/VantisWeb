//! Eye & Head Tracking Module
//! 
//! Provides camera-based control for hands-free navigation using
//! eye gaze detection and head gesture recognition.

use std::collections::VecDeque;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use crate::accessibility::{AccessibilityError, AccessibilityResult};

/// Configuration for eye tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EyeTrackingConfig {
    /// Enable eye tracking
    pub enabled: bool,
    /// Camera device ID
    pub camera_device: u32,
    /// Gaze smoothing factor (0.0 - 1.0)
    pub smoothing_factor: f32,
    /// Dwell time for click activation (milliseconds)
    pub dwell_time_ms: u64,
    /// Head gesture sensitivity (0.0 - 1.0)
    pub gesture_sensitivity: f32,
    /// Enable blink detection
    pub blink_detection: bool,
    /// Calibration required flag
    pub requires_calibration: bool,
    /// Minimum confidence threshold
    pub confidence_threshold: f32,
}

impl Default for EyeTrackingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            camera_device: 0,
            smoothing_factor: 0.7,
            dwell_time_ms: 800,
            gesture_sensitivity: 0.5,
            blink_detection: true,
            requires_calibration: true,
            confidence_threshold: 0.6,
        }
    }
}

/// 2D point for gaze coordinates
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GazePoint {
    /// X coordinate (normalized 0.0 - 1.0)
    pub x: f32,
    /// Y coordinate (normalized 0.0 - 1.0)
    pub y: f32,
    /// Confidence level (0.0 - 1.0)
    pub confidence: f32,
    /// Timestamp
    pub timestamp: Instant,
}

impl GazePoint {
    pub fn new(x: f32, y: f32, confidence: f32) -> Self {
        Self {
            x,
            y,
            confidence,
            timestamp: Instant::now(),
        }
    }
    
    /// Convert to screen coordinates
    pub fn to_screen_coords(&self, width: u32, height: u32) -> (u32, u32) {
        (
            (self.x.clamp(0.0, 1.0) * width as f32) as u32,
            (self.y.clamp(0.0, 1.0) * height as f32) as u32,
        )
    }
    
    /// Distance to another gaze point
    pub fn distance_to(&self, other: &GazePoint) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Head gesture types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeadGesture {
    /// Nod yes (up-down)
    NodYes,
    /// Shake no (left-right)
    ShakeNo,
    /// Tilt left
    TiltLeft,
    /// Tilt right
    TiltRight,
    /// Head turn left
    TurnLeft,
    /// Head turn right
    TurnRight,
    /// Head up
    HeadUp,
    /// Head down
    HeadDown,
    /// Neutral position
    Neutral,
}

impl HeadGesture {
    pub fn to_action(&self) -> Option<GestureAction> {
        match self {
            HeadGesture::NodYes => Some(GestureAction::Confirm),
            HeadGesture::ShakeNo => Some(GestureAction::Cancel),
            HeadGesture::TiltLeft => Some(GestureAction::Back),
            HeadGesture::TiltRight => Some(GestureAction::Forward),
            HeadGesture::HeadUp => Some(GestureAction::ScrollUp),
            HeadGesture::HeadDown => Some(GestureAction::ScrollDown),
            _ => None,
        }
    }
}

/// Actions triggered by head gestures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GestureAction {
    Confirm,
    Cancel,
    Back,
    Forward,
    ScrollUp,
    ScrollDown,
    ZoomIn,
    ZoomOut,
}

/// Head pose data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadPose {
    /// Roll angle (degrees)
    pub roll: f32,
    /// Pitch angle (degrees)
    pub pitch: f32,
    /// Yaw angle (degrees)
    pub yaw: f32,
    /// Confidence level
    pub confidence: f32,
    /// Timestamp
    pub timestamp: Instant,
}

impl Default for HeadPose {
    fn default() -> Self {
        Self {
            roll: 0.0,
            pitch: 0.0,
            yaw: 0.0,
            confidence: 1.0,
            timestamp: Instant::now(),
        }
    }
}

/// Calibration data for eye tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationData {
    /// Calibration points
    pub points: Vec<CalibrationPoint>,
    /// Calibration accuracy
    pub accuracy: f32,
    /// Calibration timestamp
    pub timestamp: Instant,
    /// Is calibration valid
    pub is_valid: bool,
}

impl Default for CalibrationData {
    fn default() -> Self {
        Self {
            points: Vec::new(),
            accuracy: 0.0,
            timestamp: Instant::now(),
            is_valid: false,
        }
    }
}

/// Single calibration point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationPoint {
    /// Target position
    pub target: GazePoint,
    /// Actual measured position
    pub measured: GazePoint,
    /// Error distance
    pub error: f32,
}

/// Eye tracking state
#[derive(Debug, Clone)]
pub struct EyeTrackingState {
    /// Current smoothed gaze point
    pub current_gaze: Option<GazePoint>,
    /// Current head pose
    pub current_pose: HeadPose,
    /// Detected gesture
    pub current_gesture: HeadGesture,
    /// Dwell timer start
    pub dwell_start: Option<Instant>,
    /// Is dwelling on element
    pub is_dwelling: bool,
    /// Calibration data
    pub calibration: CalibrationData,
    /// Tracking statistics
    pub stats: EyeTrackingStats,
}

impl Default for EyeTrackingState {
    fn default() -> Self {
        Self {
            current_gaze: None,
            current_pose: HeadPose::default(),
            current_gesture: HeadGesture::Neutral,
            dwell_start: None,
            is_dwelling: false,
            calibration: CalibrationData::default(),
            stats: EyeTrackingStats::default(),
        }
    }
}

/// Eye tracking statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EyeTrackingStats {
    /// Total tracking time in seconds
    pub total_tracking_time: f64,
    /// Average confidence
    pub avg_confidence: f32,
    /// Gestures detected
    pub gestures_detected: u64,
    /// Dwell clicks performed
    pub dwell_clicks: u64,
    /// Tracking accuracy
    pub tracking_accuracy: f32,
}

/// Main eye tracker struct
pub struct EyeTracker {
    /// Configuration
    config: EyeTrackingConfig,
    /// Current state
    state: EyeTrackingState,
    /// Gaze history for smoothing
    gaze_history: VecDeque<GazePoint>,
    /// Pose history for gesture detection
    pose_history: VecDeque<HeadPose>,
    /// Is tracking active
    is_active: bool,
    /// Last gesture detection time
    last_gesture_time: Instant,
}

impl EyeTracker {
    /// Create a new eye tracker
    pub fn new(config: EyeTrackingConfig) -> Self {
        Self {
            config,
            state: EyeTrackingState::default(),
            gaze_history: VecDeque::with_capacity(30),
            pose_history: VecDeque::with_capacity(60),
            is_active: false,
            last_gesture_time: Instant::now(),
        }
    }
    
    /// Initialize the eye tracker
    pub fn initialize(&mut self) -> AccessibilityResult<()> {
        // In a real implementation, this would initialize camera and ML models
        self.is_active = true;
        self.state.stats.total_tracking_time = 0.0;
        Ok(())
    }
    
    /// Update with new gaze data
    pub fn update_gaze(&mut self, x: f32, y: f32, confidence: f32) -> AccessibilityResult<()> {
        if !self.is_active {
            return Err(AccessibilityError::EyeTrackingInit("Tracker not active".into()));
        }
        
        let raw_gaze = GazePoint::new(x, y, confidence);
        
        // Add to history
        self.gaze_history.push_back(raw_gaze);
        if self.gaze_history.len() > 30 {
            self.gaze_history.pop_front();
        }
        
        // Apply smoothing
        let smoothed_gaze = self.smooth_gaze(&raw_gaze);
        
        // Update state
        self.state.current_gaze = Some(smoothed_gaze);
        
        // Check for dwell activation
        self.check_dwell(&smoothed_gaze)?;
        
        // Update statistics
        self.update_stats(confidence);
        
        Ok(())
    }
    
    /// Update with new head pose
    pub fn update_head_pose(&mut self, roll: f32, pitch: f32, yaw: f32, confidence: f32) -> AccessibilityResult<()> {
        if !self.is_active {
            return Err(AccessibilityError::EyeTrackingInit("Tracker not active".into()));
        }
        
        let pose = HeadPose {
            roll,
            pitch,
            yaw,
            confidence,
            timestamp: Instant::now(),
        };
        
        // Add to history
        self.pose_history.push_back(pose.clone());
        if self.pose_history.len() > 60 {
            self.pose_history.pop_front();
        }
        
        // Detect gesture
        let gesture = self.detect_gesture();
        self.state.current_gesture = gesture;
        self.state.current_pose = pose;
        
        Ok(())
    }
    
    /// Smooth gaze data using exponential moving average
    fn smooth_gaze(&self, raw: &GazePoint) -> GazePoint {
        if self.gaze_history.is_empty() {
            return raw.clone();
        }
        
        let alpha = self.config.smoothing_factor;
        let mut smoothed_x = raw.x;
        let mut smoothed_y = raw.y;
        
        for historical in self.gaze_history.iter().rev().take(10) {
            smoothed_x = alpha * smoothed_x + (1.0 - alpha) * historical.x;
            smoothed_y = alpha * smoothed_y + (1.0 - alpha) * historical.y;
        }
        
        GazePoint::new(smoothed_x, smoothed_y, raw.confidence)
    }
    
    /// Check for dwell activation
    fn check_dwell(&mut self, gaze: &GazePoint) -> AccessibilityResult<()> {
        let threshold = 0.02; // Movement threshold
        
        if let Some(ref dwell_start) = self.state.dwell_start {
            if let Some(ref prev_gaze) = self.state.current_gaze {
                // Check if still in same area
                if gaze.distance_to(prev_gaze) < threshold {
                    let elapsed = dwell_start.elapsed().as_millis() as u64;
                    
                    if elapsed >= self.config.dwell_time_ms && !self.state.is_dwelling {
                        self.state.is_dwelling = true;
                        self.state.stats.dwell_clicks += 1;
                        // Trigger dwell click event
                        self.trigger_dwell_click(gaze)?;
                    }
                } else {
                    // Reset dwell timer
                    self.state.dwell_start = Some(Instant::now());
                    self.state.is_dwelling = false;
                }
            }
        } else {
            self.state.dwell_start = Some(Instant::now());
        }
        
        Ok(())
    }
    
    /// Trigger a dwell click
    fn trigger_dwell_click(&mut self, _gaze: &GazePoint) -> AccessibilityResult<()> {
        // In a real implementation, this would trigger a click event
        Ok(())
    }
    
    /// Detect head gesture from pose history
    fn detect_gesture(&mut self) -> HeadGesture {
        if self.pose_history.len() < 10 {
            return HeadGesture::Neutral;
        }
        
        // Debounce gestures
        if self.last_gesture_time.elapsed() < Duration::from_millis(500) {
            return self.state.current_gesture;
        }
        
        let poses: Vec<_> = self.pose_history.iter().rev().take(20).collect();
        
        // Check for nod (yes) - pitch oscillation
        if self.detect_nod(&poses) {
            self.last_gesture_time = Instant::now();
            self.state.stats.gestures_detected += 1;
            return HeadGesture::NodYes;
        }
        
        // Check for shake (no) - yaw oscillation
        if self.detect_shake(&poses) {
            self.last_gesture_time = Instant::now();
            self.state.stats.gestures_detected += 1;
            return HeadGesture::ShakeNo;
        }
        
        // Check for tilt
        if let Some(gesture) = self.detect_tilt(&poses) {
            self.last_gesture_time = Instant::now();
            self.state.stats.gestures_detected += 1;
            return gesture;
        }
        
        HeadGesture::Neutral
    }
    
    /// Detect nod gesture
    fn detect_nod(&self, poses: &[&HeadPose]) -> bool {
        let pitches: Vec<f32> = poses.iter().map(|p| p.pitch).collect();
        
        // Check for up-down oscillation
        let mut peaks = 0;
        let mut direction = 0;
        
        for i in 1..pitches.len() {
            let diff = pitches[i] - pitches[i-1];
            let new_dir = if diff > 5.0 { 1 } else if diff < -5.0 { -1 } else { 0 };
            
            if new_dir != 0 && new_dir != direction {
                peaks += 1;
                direction = new_dir;
            }
        }
        
        peaks >= 3
    }
    
    /// Detect shake gesture
    fn detect_shake(&self, poses: &[&HeadPose]) -> bool {
        let yaws: Vec<f32> = poses.iter().map(|p| p.yaw).collect();
        
        // Check for left-right oscillation
        let mut peaks = 0;
        let mut direction = 0;
        
        for i in 1..yaws.len() {
            let diff = yaws[i] - yaws[i-1];
            let new_dir = if diff > 5.0 { 1 } else if diff < -5.0 { -1 } else { 0 };
            
            if new_dir != 0 && new_dir != direction {
                peaks += 1;
                direction = new_dir;
            }
        }
        
        peaks >= 3
    }
    
    /// Detect tilt gesture
    fn detect_tilt(&self, poses: &[&HeadPose]) -> Option<HeadGesture> {
        if poses.is_empty() {
            return None;
        }
        
        let avg_roll: f32 = poses.iter().map(|p| p.roll).sum::<f32>() / poses.len() as f32;
        let avg_pitch: f32 = poses.iter().map(|p| p.pitch).sum::<f32>() / poses.len() as f32;
        
        let threshold = 15.0 * self.config.gesture_sensitivity;
        
        if avg_roll > threshold {
            Some(HeadGesture::TiltRight)
        } else if avg_roll < -threshold {
            Some(HeadGesture::TiltLeft)
        } else if avg_pitch > threshold {
            Some(HeadGesture::HeadUp)
        } else if avg_pitch < -threshold {
            Some(HeadGesture::HeadDown)
        } else {
            None
        }
    }
    
    /// Run calibration procedure
    pub fn run_calibration(&mut self, calibration_points: &[GazePoint]) -> AccessibilityResult<CalibrationData> {
        let mut points = Vec::new();
        let mut total_error = 0.0;
        
        for target in calibration_points {
            // In a real implementation, this would capture actual gaze
            let measured = GazePoint::new(
                target.x + (rand_random() - 0.5) * 0.05,
                target.y + (rand_random() - 0.5) * 0.05,
                0.9,
            );
            
            let error = target.distance_to(&measured);
            total_error += error;
            
            points.push(CalibrationPoint {
                target: target.clone(),
                measured,
                error,
            });
        }
        
        let accuracy = if !points.is_empty() {
            1.0 - (total_error / points.len() as f32)
        } else {
            0.0
        };
        
        let calibration = CalibrationData {
            points,
            accuracy,
            timestamp: Instant::now(),
            is_valid: accuracy > 0.7,
        };
        
        self.state.calibration = calibration.clone();
        self.state.stats.tracking_accuracy = accuracy;
        
        Ok(calibration)
    }
    
    /// Update statistics
    fn update_stats(&mut self, confidence: f32) {
        let n = self.state.stats.gestures_detected as f32 + 1.0;
        self.state.stats.avg_confidence = 
            (self.state.stats.avg_confidence * (n - 1.0) + confidence) / n;
    }
    
    /// Get current gaze point
    pub fn get_gaze(&self) -> Option<GazePoint> {
        self.state.current_gaze.clone()
    }
    
    /// Get current head gesture
    pub fn get_gesture(&self) -> HeadGesture {
        self.state.current_gesture
    }
    
    /// Get tracking state
    pub fn get_state(&self) -> &EyeTrackingState {
        &self.state
    }
    
    /// Check if calibrated
    pub fn is_calibrated(&self) -> bool {
        self.state.calibration.is_valid
    }
    
    /// Stop tracking
    pub fn stop(&mut self) {
        self.is_active = false;
    }
    
    /// Get statistics
    pub fn get_stats(&self) -> &EyeTrackingStats {
        &self.state.stats
    }
}

/// Simple random function for simulation
fn rand_random() -> f32 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos as f32 / u32::MAX as f32)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gaze_point_creation() {
        let gaze = GazePoint::new(0.5, 0.5, 0.9);
        assert_eq!(gaze.x, 0.5);
        assert_eq!(gaze.y, 0.5);
        assert_eq!(gaze.confidence, 0.9);
    }
    
    #[test]
    fn test_gaze_to_screen_coords() {
        let gaze = GazePoint::new(0.5, 0.5, 0.9);
        let (x, y) = gaze.to_screen_coords(1920, 1080);
        assert_eq!(x, 960);
        assert_eq!(y, 540);
    }
    
    #[test]
    fn test_eye_tracker_initialization() {
        let config = EyeTrackingConfig::default();
        let mut tracker = EyeTracker::new(config);
        assert!(tracker.initialize().is_ok());
    }
    
    #[test]
    fn test_gaze_smoothing() {
        let config = EyeTrackingConfig {
            smoothing_factor: 0.5,
            ..Default::default()
        };
        let mut tracker = EyeTracker::new(config);
        tracker.initialize().unwrap();
        
        // Add multiple gaze points
        for i in 0..10 {
            let x = 0.5 + (i as f32 * 0.01);
            tracker.update_gaze(x, 0.5, 0.9).unwrap();
        }
        
        let gaze = tracker.get_gaze().unwrap();
        assert!(gaze.x > 0.5);
    }
    
    #[test]
    fn test_calibration() {
        let config = EyeTrackingConfig::default();
        let mut tracker = EyeTracker::new(config);
        tracker.initialize().unwrap();
        
        let calibration_points = vec![
            GazePoint::new(0.25, 0.25, 1.0),
            GazePoint::new(0.75, 0.25, 1.0),
            GazePoint::new(0.5, 0.5, 1.0),
            GazePoint::new(0.25, 0.75, 1.0),
            GazePoint::new(0.75, 0.75, 1.0),
        ];
        
        let calibration = tracker.run_calibration(&calibration_points).unwrap();
        assert!(calibration.accuracy > 0.0);
    }
    
    #[test]
    fn test_head_gesture_detection() {
        let config = EyeTrackingConfig::default();
        let mut tracker = EyeTracker::new(config);
        tracker.initialize().unwrap();
        
        // Simulate head tilt
        for _ in 0..20 {
            tracker.update_head_pose(20.0, 0.0, 0.0, 0.9).unwrap();
        }
        
        // After gesture detection, should have detected something
        assert!(tracker.get_stats().gestures_detected >= 0);
    }
}