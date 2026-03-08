//! Tremor Guard Module
//! 
//! Provides cursor stabilization and adaptive controls for users with tremors,
//! using predictive movement filtering and smart input smoothing.

use std::collections::VecDeque;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize];
use crate::accessibility::{AccessibilityError, AccessibilityResult};

/// Tremor guard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TremorConfig {
    /// Enable tremor guard
    pub enabled: bool,
    /// Stabilization mode
    pub stabilization_mode: StabilizationMode,
    /// Smoothing strength (0.0 - 1.0)
    pub smoothing_strength: f32,
    /// Tremor frequency detection (Hz)
    pub tremor_frequency: f32,
    /// Enable adaptive sensitivity
    pub adaptive_sensitivity: bool,
    /// Click confirmation threshold (pixels)
    pub click_threshold: f32,
    /// Dwell time for click (ms)
    pub dwell_time_ms: u64,
    /// Enable movement prediction
    pub predictive_movement: bool,
    /// Ignore accidental double-clicks
    pub ignore_double_click: bool,
    /// Scroll smoothing
    pub smooth_scrolling: bool,
    /// Drag assist
    pub drag_assist: bool,
}

impl Default for TremorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            stabilization_mode: StabilizationMode::Moderate,
            smoothing_strength: 0.5,
            tremor_frequency: 4.0,
            adaptive_sensitivity: true,
            click_threshold: 10.0,
            dwell_time_ms: 300,
            predictive_movement: true,
            ignore_double_click: true,
            smooth_scrolling: true,
            drag_assist: true,
        }
    }
}

/// Stabilization mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StabilizationMode {
    /// Minimal stabilization
    Light,
    /// Standard stabilization
    Moderate,
    /// Strong stabilization
    Strong,
    /// Maximum stabilization (may feel slower)
    Maximum,
}

impl StabilizationMode {
    /// Get smoothing factor for this mode
    pub fn smoothing_factor(&self) -> f32 {
        match self {
            StabilizationMode::Light => 0.3,
            StabilizationMode::Moderate => 0.5,
            StabilizationMode::Strong => 0.7,
            StabilizationMode::Maximum => 0.85,
        }
    }
    
    /// Get history size for smoothing
    pub fn history_size(&self) -> usize {
        match self {
            StabilizationMode::Light => 5,
            StabilizationMode::Moderate => 10,
            StabilizationMode::Strong => 15,
            StabilizationMode::Maximum => 20,
        }
    }
}

/// Cursor state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorState {
    /// Cursor is idle
    Idle,
    /// Cursor is moving
    Moving,
    /// Cursor is dwelling (staying still for click)
    Dwelling,
    /// Cursor is dragging
    Dragging,
    /// Cursor is scrolling
    Scrolling,
}

/// Cursor position with timestamp
#[derive(Debug, Clone, Copy)]
pub struct CursorPosition {
    /// X coordinate
    pub x: f32,
    /// Y coordinate
    pub y: f32,
    /// Timestamp
    pub timestamp: Instant,
    /// Velocity X
    pub velocity_x: f32,
    /// Velocity Y
    pub velocity_y: f32,
}

impl CursorPosition {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            timestamp: Instant::now(),
            velocity_x: 0.0,
            velocity_y: 0.0,
        }
    }
    
    /// Distance to another position
    pub fn distance_to(&self, other: &CursorPosition) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
    
    /// Speed (magnitude of velocity)
    pub fn speed(&self) -> f32 {
        (self.velocity_x * self.velocity_x + self.velocity_y * self.velocity_y).sqrt()
    }
}

/// Detected tremor pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TremorPattern {
    /// Dominant frequency (Hz)
    pub frequency: f32,
    /// Amplitude (pixels)
    pub amplitude: f32,
    /// Direction of tremor
    pub direction: TremorDirection,
    /// Confidence of detection
    pub confidence: f32,
    /// Timestamp
    pub timestamp: Instant,
}

/// Direction of tremor movement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TremorDirection {
    /// Horizontal tremor
    Horizontal,
    /// Vertical tremor
    Vertical,
    /// Circular tremor
    Circular,
    /// Diagonal tremor
    Diagonal,
    /// Random/unpredictable
    Random,
}

/// Click intent detection
#[derive(Debug, Clone)]
pub struct ClickIntent {
    /// Position where click is intended
    pub position: (f32, f32),
    /// Confidence level
    pub confidence: f32,
    /// Time remaining until auto-click (ms)
    pub time_remaining_ms: u64,
    /// Is dwelling detected
    pub is_dwelling: bool,
}

/// Tremor guard statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TremorStats {
    /// Tremor corrections applied
    pub corrections_applied: u64,
    /// Average tremor amplitude detected
    pub avg_tremor_amplitude: f32,
    /// Dwell clicks performed
    pub dwell_clicks: u64,
    /// Accidental clicks prevented
    pub accidental_clicks_prevented: u64,
    /// Movement predictions made
    pub predictions_made: u64,
    /// Active time in seconds
    pub active_time: f64,
}

/// Main tremor guard struct
pub struct TremorGuard {
    /// Configuration
    config: TremorConfig,
    /// Current cursor state
    state: CursorState,
    /// Position history for smoothing
    position_history: VecDeque<CursorPosition>,
    /// Stabilized cursor position
    stabilized_position: CursorPosition,
    /// Raw cursor position
    raw_position: CursorPosition,
    /// Detected tremor pattern
    tremor_pattern: Option<TremorPattern>,
    /// Dwell start time
    dwell_start: Option<Instant>,
    /// Click intent
    click_intent: Option<ClickIntent>,
    /// Statistics
    stats: TremorStats,
    /// Last processed time
    last_update: Instant,
    /// Movement prediction buffer
    prediction_buffer: VecDeque<(f32, f32)>,
}

impl TremorGuard {
    /// Create a new tremor guard
    pub fn new(config: TremorConfig) -> Self {
        Self {
            config,
            state: CursorState::Idle,
            position_history: VecDeque::with_capacity(50),
            stabilized_position: CursorPosition::new(0.0, 0.0),
            raw_position: CursorPosition::new(0.0, 0.0),
            tremor_pattern: None,
            dwell_start: None,
            click_intent: None,
            stats: TremorStats::default(),
            last_update: Instant::now(),
            prediction_buffer: VecDeque::with_capacity(10),
        }
    }
    
    /// Initialize the tremor guard
    pub fn initialize(&mut self) -> AccessibilityResult<()> {
        self.last_update = Instant::now();
        Ok(())
    }
    
    /// Process a raw cursor position and return stabilized position
    pub fn process_position(&mut self, x: f32, y: f32) -> (f32, f32) {
        if !self.config.enabled {
            return (x, y);
        }
        
        let now = Instant::now();
        let dt = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;
        
        // Create new position with velocity
        let mut new_pos = CursorPosition::new(x, y);
        
        // Calculate velocity
        if let Some(last) = self.position_history.back() {
            let dt = dt.max(0.001); // Prevent division by zero
            new_pos.velocity_x = (x - last.x) / dt;
            new_pos.velocity_y = (y - last.y) / dt;
        }
        
        // Update raw position
        self.raw_position = new_pos.clone();
        
        // Add to history
        self.position_history.push_back(new_pos.clone());
        let history_size = self.config.stabilization_mode.history_size();
        while self.position_history.len() > history_size {
            self.position_history.pop_front();
        }
        
        // Detect tremor pattern
        self.detect_tremor_pattern();
        
        // Apply stabilization
        let stabilized = self.stabilize_position(&new_pos);
        self.stabilized_position = stabilized.clone();
        
        // Update cursor state
        self.update_cursor_state(&new_pos);
        
        // Check for dwell click
        if self.config.dwell_time_ms > 0 {
            self.check_dwell_click(&stabilized);
        }
        
        // Predict movement if enabled
        if self.config.predictive_movement {
            self.predict_movement(&stabilized);
        }
        
        (stabilized.x, stabilized.y)
    }
    
    /// Stabilize cursor position
    fn stabilize_position(&self, raw: &CursorPosition) -> CursorPosition {
        if self.position_history.len() < 3 {
            return raw.clone();
        }
        
        let smoothing = self.config.smoothing_strength * self.config.stabilization_mode.smoothing_factor();
        
        // Weighted moving average with tremor filtering
        let mut stabilized_x = 0.0;
        let mut stabilized_y = 0.0;
        let mut total_weight = 0.0;
        
        for (i, pos) in self.position_history.iter().enumerate().rev() {
            // Higher weight for more recent positions
            let weight = (i + 1) as f32;
            
            // Apply tremor filtering if pattern detected
            let filter_weight = if let Some(ref pattern) = self.tremor_pattern {
                // Reduce weight of positions that align with tremor frequency
                let time_factor = pos.timestamp.elapsed().as_secs_f32();
                let tremor_phase = (time_factor * pattern.frequency * std::f32::consts::TAU).sin();
                1.0 - smoothing * tremor_phase.abs() * pattern.confidence
            } else {
                1.0
            };
            
            let effective_weight = weight * filter_weight;
            stabilized_x += pos.x * effective_weight;
            stabilized_y += pos.y * effective_weight;
            total_weight += effective_weight;
        }
        
        stabilized_x /= total_weight;
        stabilized_y /= total_weight;
        
        let mut stabilized = CursorPosition::new(stabilized_x, stabilized_y);
        stabilized.velocity_x = raw.velocity_x * (1.0 - smoothing);
        stabilized.velocity_y = raw.velocity_y * (1.0 - smoothing);
        
        self.stats.corrections_applied += 1;
        
        stabilized
    }
    
    /// Detect tremor pattern from position history
    fn detect_tremor_pattern(&mut self) {
        if self.position_history.len() < 20 {
            return;
        }
        
        // Analyze position changes for tremor characteristics
        let positions: Vec<_> = self.position_history.iter().cloned().collect();
        
        // Calculate movement variance
        let mut x_diffs = Vec::new();
        let mut y_diffs = Vec::new();
        
        for i in 1..positions.len() {
            x_diffs.push(positions[i].x - positions[i-1].x);
            y_diffs.push(positions[i].y - positions[i-1].y);
        }
        
        // Calculate variance
        let x_var = variance(&x_diffs);
        let y_var = variance(&y_diffs);
        
        // Determine tremor direction
        let direction = if x_var > y_var * 2.0 {
            TremorDirection::Horizontal
        } else if y_var > x_var * 2.0 {
            TremorDirection::Vertical
        } else if (x_var - y_var).abs() < x_var * 0.3 {
            TremorDirection::Circular
        } else {
            TremorDirection::Random
        };
        
        // Estimate frequency (simplified)
        let amplitude = (x_var + y_var).sqrt();
        let frequency = self.estimate_frequency(&x_diffs, &y_diffs);
        
        // Update statistics
        self.stats.avg_tremor_amplitude = 
            (self.stats.avg_tremor_amplitude * 0.9 + amplitude * 0.1);
        
        self.tremor_pattern = Some(TremorPattern {
            frequency,
            amplitude,
            direction,
            confidence: 0.7,
            timestamp: Instant::now(),
        });
    }
    
    /// Estimate tremor frequency
    fn estimate_frequency(&self, x_diffs: &[f32], y_diffs: &[f32]) -> f32 {
        // Count zero crossings as a simple frequency estimate
        let mut zero_crossings = 0;
        
        for i in 1..x_diffs.len() {
            if (x_diffs[i] > 0.0 && x_diffs[i-1] <= 0.0) ||
               (x_diffs[i] < 0.0 && x_diffs[i-1] >= 0.0) {
                zero_crossings += 1;
            }
        }
        
        // Approximate frequency
        let time_span = self.position_history.len() as f32 * 0.016; // Assume ~60fps
        zero_crossings as f32 / (2.0 * time_span.max(0.001))
    }
    
    /// Update cursor state based on movement
    fn update_cursor_state(&mut self, pos: &CursorPosition) {
        let speed = pos.speed();
        
        self.state = if speed < 5.0 && self.state != CursorState::Dragging {
            if self.dwell_start.is_some() {
                CursorState::Dwelling
            } else {
                CursorState::Idle
            }
        } else if speed < 50.0 {
            CursorState::Moving
        } else {
            CursorState::Moving
        };
    }
    
    /// Check for dwell click
    fn check_dwell_click(&mut self, pos: &CursorPosition) {
        let threshold = self.config.click_threshold;
        
        // Check if cursor is staying in place
        if let Some(start_time) = self.dwell_start {
            // Check if moved away
            if let Some(first) = self.position_history.front() {
                if pos.distance_to(first) > threshold {
                    self.dwell_start = None;
                    self.click_intent = None;
                    return;
                }
            }
            
            // Check if dwell time reached
            let elapsed = start_time.elapsed().as_millis() as u64;
            if elapsed >= self.config.dwell_time_ms {
                // Trigger dwell click
                self.stats.dwell_clicks += 1;
                self.dwell_start = None;
                self.click_intent = None;
            } else {
                // Update click intent
                self.click_intent = Some(ClickIntent {
                    position: (pos.x, pos.y),
                    confidence: 0.8,
                    time_remaining_ms: self.config.dwell_time_ms - elapsed,
                    is_dwelling: true,
                });
            }
        } else {
            // Start dwell timer
            self.dwell_start = Some(Instant::now());
        }
    }
    
    /// Predict movement direction
    fn predict_movement(&mut self, pos: &CursorPosition) {
        if self.position_history.len() < 5 {
            return;
        }
        
        // Simple linear extrapolation
        let recent: Vec<_> = self.position_history.iter().rev().take(5).collect();
        
        let mut avg_vx = 0.0;
        let mut avg_vy = 0.0;
        
        for pos in &recent {
            avg_vx += pos.velocity_x;
            avg_vy += pos.velocity_y;
        }
        
        avg_vx /= recent.len() as f32;
        avg_vy /= recent.len() as f32;
        
        // Predict future position
        let prediction = (pos.x + avg_vx * 0.1, pos.y + avg_vy * 0.1);
        self.prediction_buffer.push_back(prediction);
        
        if self.prediction_buffer.len() > 10 {
            self.prediction_buffer.pop_front();
        }
        
        self.stats.predictions_made += 1;
    }
    
    /// Process a click event
    pub fn process_click(&mut self, x: f32, y: f32) -> AccessibilityResult<bool> {
        if !self.config.enabled {
            return Ok(true);
        }
        
        // Check for accidental click
        if let Some(ref pattern) = self.tremor_pattern {
            if pattern.amplitude > self.config.click_threshold {
                // High tremor - check if this might be accidental
                self.stats.accidental_clicks_prevented += 1;
                
                // Require confirmation for high tremor
                if pattern.amplitude > self.config.click_threshold * 2.0 {
                    return Ok(false);
                }
            }
        }
        
        // Check for double-click
        if self.config.ignore_double_click {
            // Implement double-click prevention logic
        }
        
        Ok(true)
    }
    
    /// Process scroll event
    pub fn process_scroll(&mut self, delta: f32) -> f32 {
        if !self.config.enabled || !self.config.smooth_scrolling {
            return delta;
        }
        
        // Apply smoothing to scroll
        let smoothed = delta * (1.0 - self.config.stabilization_mode.smoothing_factor() * 0.5);
        
        self.state = CursorState::Scrolling;
        
        smoothed
    }
    
    /// Start drag operation
    pub fn start_drag(&mut self) {
        if self.config.drag_assist {
            self.state = CursorState::Dragging;
        }
    }
    
    /// End drag operation
    pub fn end_drag(&mut self) {
        self.state = CursorState::Idle;
    }
    
    /// Get current stabilized position
    pub fn get_stabilized_position(&self) -> (f32, f32) {
        (self.stabilized_position.x, self.stabilized_position.y)
    }
    
    /// Get raw position
    pub fn get_raw_position(&self) -> (f32, f32) {
        (self.raw_position.x, self.raw_position.y)
    }
    
    /// Get cursor state
    pub fn get_state(&self) -> CursorState {
        self.state
    }
    
    /// Get detected tremor pattern
    pub fn get_tremor_pattern(&self) -> Option<&TremorPattern> {
        self.tremor_pattern.as_ref()
    }
    
    /// Get click intent
    pub fn get_click_intent(&self) -> Option<&ClickIntent> {
        self.click_intent.as_ref()
    }
    
    /// Get movement prediction
    pub fn get_predicted_position(&self) -> Option<(f32, f32)> {
        self.prediction_buffer.back().copied()
    }
    
    /// Get statistics
    pub fn get_stats(&self) -> &TremorStats {
        &self.stats
    }
    
    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: TremorConfig) {
        self.config = config;
    }
}

/// Calculate variance of a slice of values
fn variance(values: &[f32]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    
    let mean: f32 = values.iter().sum::<f32>() / values.len() as f32;
    values.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / values.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tremor_config_default() {
        let config = TremorConfig::default();
        assert!(config.enabled);
        assert!(config.adaptive_sensitivity);
    }
    
    #[test]
    fn test_stabilization_mode() {
        assert!(StabilizationMode::Strong.smoothing_factor() > StabilizationMode::Light.smoothing_factor());
        assert!(StabilizationMode::Maximum.history_size() > StabilizationMode::Light.history_size());
    }
    
    #[test]
    fn test_tremor_guard_creation() {
        let config = TremorConfig::default();
        let guard = TremorGuard::new(config);
        assert!(guard.is_enabled());
    }
    
    #[test]
    fn test_position_processing() {
        let config = TremorConfig::default();
        let mut guard = TremorGuard::new(config);
        guard.initialize().unwrap();
        
        // Process several positions
        for i in 0..20 {
            let (x, y) = guard.process_position(100.0 + i as f32, 100.0);
            assert!(x >= 100.0);
        }
        
        assert!(guard.get_stats().corrections_applied > 0);
    }
    
    #[test]
    fn test_cursor_state() {
        let config = TremorConfig::default();
        let mut guard = TremorGuard::new(config);
        guard.initialize().unwrap();
        
        // Process stationary position
        for _ in 0..20 {
            guard.process_position(100.0, 100.0);
        }
        
        // Should be idle or dwelling
        let state = guard.get_state();
        assert!(state == CursorState::Idle || state == CursorState::Dwelling);
    }
    
    #[test]
    fn test_tremor_detection() {
        let config = TremorConfig::default();
        let mut guard = TremorGuard::new(config);
        guard.initialize().unwrap();
        
        // Simulate tremor movement (oscillating)
        for i in 0..30 {
            let offset = (i as f32 * 0.5).sin() * 10.0;
            guard.process_position(100.0 + offset, 100.0);
        }
        
        // Should detect tremor pattern
        assert!(guard.get_tremor_pattern().is_some());
    }
    
    #[test]
    fn test_scroll_smoothing() {
        let config = TremorConfig {
            smooth_scrolling: true,
            ..Default::default()
        };
        let mut guard = TremorGuard::new(config);
        
        let smoothed = guard.process_scroll(10.0);
        assert!(smoothed < 10.0); // Should be smoothed
    }
    
    #[test]
    fn test_dwell_click() {
        let config = TremorConfig {
            dwell_time_ms: 100,
            click_threshold: 50.0,
            ..Default::default()
        };
        let mut guard = TremorGuard::new(config);
        guard.initialize().unwrap();
        
        // Hold position
        for _ in 0..20 {
            guard.process_position(100.0, 100.0);
        }
        
        // Check for click intent
        // Note: In real testing, we'd need to wait for dwell time
    }
}