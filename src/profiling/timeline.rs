//! Timeline Recorder
//! 
//! This module provides timeline-based event recording for performance analysis.
//! It captures browser events in chronological order to create detailed performance timelines.
//! 
//! # Features
//! - Event recording with timestamps
//! - Category-based event organization
//! - Timeline visualization data generation
//! - Event filtering and search
//! - Flame graph data generation

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// Event category for organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventCategory {
    Rendering,
    Scripting,
    Painting,
    Network,
    System,
    User,
    Idle,
    Other,
}

/// Event phase in the timeline
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventPhase {
    Begin,
    End,
    Instant,
    Complete,
}

/// Priority level for events
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Timeline event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub id: String,
    pub name: String,
    pub category: EventCategory,
    pub phase: EventPhase,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration: Option<std::time::Duration>,
    pub timestamp: DateTime<Utc>,
    pub thread_id: Option<String>,
    pub process_id: Option<u32>,
    pub priority: EventPriority,
    pub args: HashMap<String, String>,
    pub parent_id: Option<String>,
    pub children: Vec<String>,
}

impl TimelineEvent {
    /// Create a new timeline event
    pub fn new(
        name: String,
        category: EventCategory,
        phase: EventPhase,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        Self {
            id: id.clone(),
            name,
            category,
            phase,
            start_time: now,
            end_time: None,
            duration: None,
            timestamp: now,
            thread_id: None,
            process_id: None,
            priority: EventPriority::Normal,
            args: HashMap::new(),
            parent_id: None,
            children: Vec::new(),
        }
    }

    /// Mark event as complete
    pub fn complete(&mut self) {
        self.end_time = Some(Utc::now());
        self.duration = self.end_time.and_then(|end| {
            Some(end.signed_duration_since(self.start_time).to_std().ok()?)
        });
        self.phase = EventPhase::Complete;
    }

    /// Calculate duration in milliseconds
    pub fn duration_ms(&self) -> f64 {
        self.duration
            .map(|d| d.as_secs_f64() * 1000.0)
            .unwrap_or(0.0)
    }

    /// Check if event is complete
    pub fn is_complete(&self) -> bool {
        self.end_time.is_some() && self.duration.is_some()
    }

    /// Add an argument
    pub fn add_arg(&mut self, key: String, value: String) {
        self.args.insert(key, value);
    }

    /// Set parent event
    pub fn set_parent(&mut self, parent_id: String) {
        self.parent_id = Some(parent_id);
    }

    /// Add child event
    pub fn add_child(&mut self, child_id: String) {
        self.children.push(child_id);
    }
}

/// Frame information for paint events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameInfo {
    pub frame_number: u64,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration: std::time::Duration,
    pub fps: f64,
    pub dropped: bool,
}

/// Trace data for performance analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceData {
    pub events: Vec<TimelineEvent>,
    pub frames: Vec<FrameInfo>,
    pub metadata: TraceMetadata,
}

/// Metadata for trace data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceMetadata {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub total_duration: std::time::Duration,
    pub total_events: usize,
    pub threads: Vec<String>,
    pub categories: Vec<EventCategory>,
}

/// Timeline recorder configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineRecorderConfig {
    pub max_events: usize,
    pub record_frames: bool,
    pub auto_complete_events: bool,
    auto_complete_timeout_ms: u64,
    pub filter_categories: Vec<EventCategory>,
}

impl Default for TimelineRecorderConfig {
    fn default() -> Self {
        Self {
            max_events: 100000,
            record_frames: true,
            auto_complete_events: true,
            auto_complete_timeout_ms: 5000,
            filter_categories: vec![],
        }
    }
}

/// Main timeline recorder
pub struct TimelineRecorder {
    config: Arc<RwLock<TimelineRecorderConfig>>,
    events: Arc<RwLock<VecDeque<TimelineEvent>>>,
    event_map: Arc<RwLock<HashMap<String, usize>>>,
    frames: Arc<RwLock<Vec<FrameInfo>>>,
    current_frame: Arc<RwLock<Option<FrameInfo>>>,
    is_recording: Arc<RwLock<bool>>,
    start_time: Arc<RwLock<Option<DateTime<Utc>>>>,
}

impl TimelineRecorder {
    /// Create a new timeline recorder
    pub fn new(config: TimelineRecorderConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            events: Arc::new(RwLock::new(VecDeque::new())),
            event_map: Arc::new(RwLock::new(HashMap::new())),
            frames: Arc::new(RwLock::new(Vec::new())),
            current_frame: Arc::new(RwLock::new(None)),
            is_recording: Arc::new(RwLock::new(false)),
            start_time: Arc::new(RwLock::new(None)),
        }
    }

    /// Start recording timeline events
    pub async fn start_recording(&self) {
        *self.is_recording.write().await = true;
        *self.start_time.write().await = Some(Utc::now());
        self.events.write().await.clear();
        self.frames.write().await.clear();
        self.event_map.write().await.clear();
    }

    /// Stop recording timeline events
    pub async fn stop_recording(&self) {
        *self.is_recording.write().await = false;
        self.complete_pending_events().await;
    }

    /// Check if currently recording
    pub async fn is_recording(&self) -> bool {
        *self.is_recording.read().await
    }

    /// Record a timeline event
    pub async fn record_event(&self, event: TimelineEvent) -> Result<(), String> {
        if !self.is_recording().await {
            return Ok(());
        }

        let config = self.config.read().await;

        // Check category filter
        if !config.filter_categories.is_empty()
            && !config.filter_categories.contains(&event.category) {
            return Ok(());
        }

        let mut events = self.events.write().await;
        let mut event_map = self.event_map.write().await;

        // Check max events limit
        if events.len() >= config.max_events {
            events.pop_front();
            // Clean up map entries for removed events
            event_map.retain(|_, idx| *idx < events.len());
        }

        // Add parent-child relationships
        if let Some(ref parent_id) = event.parent_id {
            if let Some(&parent_idx) = event_map.get(parent_id) {
                if parent_idx < events.len() {
                    let mut parent = events.get_mut(parent_idx).unwrap();
                    parent.add_child(event.id.clone());
                }
            }
        }

        // Add event
        let event_index = events.len();
        event_map.insert(event.id.clone(), event_index);
        events.push_back(event);

        Ok(())
    }

    /// Begin an event
    pub async fn begin_event(
        &self,
        name: String,
        category: EventCategory,
        thread_id: Option<String>,
    ) -> String {
        let mut event = TimelineEvent::new(name, category, EventPhase::Begin);
        event.thread_id = thread_id;

        let id = event.id.clone();
        let _ = self.record_event(event).await;
        id
    }

    /// End an event
    pub async fn end_event(&self, id: &str) -> Result<(), String> {
        let mut events = self.events.write().await;
        let event_map = self.event_map.read().await;

        if let Some(&idx) = event_map.get(id) {
            if idx < events.len() {
                let event = events.get_mut(idx).unwrap();
                event.end_time = Some(Utc::now());
                event.duration = event.end_time.and_then(|end| {
                    Some(end.signed_duration_since(event.start_time).to_std().ok()?)
                });
                event.phase = EventPhase::Complete;

                // Record corresponding end event
                let end_event = TimelineEvent {
                    id: Uuid::new_v4().to_string(),
                    name: format!("{} (end)", event.name),
                    category: event.category,
                    phase: EventPhase::End,
                    start_time: event.end_time.unwrap(),
                    end_time: None,
                    duration: None,
                    timestamp: event.end_time.unwrap(),
                    thread_id: event.thread_id.clone(),
                    process_id: event.process_id,
                    priority: event.priority,
                    args: HashMap::new(),
                    parent_id: Some(id.clone()),
                    children: Vec::new(),
                };

                drop(events);
                drop(event_map);

                let _ = self.record_event(end_event).await;

                Ok(())
            } else {
                Err(format!("Event index out of bounds: {}", idx))
            }
        } else {
            Err(format!("Event not found: {}", id))
        }
    }

    /// Record an instant event
    pub async fn record_instant(
        &self,
        name: String,
        category: EventCategory,
        args: HashMap<String, String>,
    ) {
        let mut event = TimelineEvent::new(name, category, EventPhase::Instant);
        event.args = args;
        let _ = self.record_event(event).await;
    }

    /// Start a frame
    pub async fn start_frame(&self, frame_number: u64) {
        let config = self.config.read().await;
        if !config.record_frames {
            return;
        }

        let frame_info = FrameInfo {
            frame_number,
            start_time: Utc::now(),
            end_time: Utc::now(),
            duration: std::time::Duration::from_nanos(0),
            fps: 0.0,
            dropped: false,
        };

        *self.current_frame.write().await = Some(frame_info);
    }

    /// End the current frame
    pub async fn end_frame(&self) {
        let config = self.config.read().await;
        if !config.record_frames {
            return;
        }

        if let Some(mut frame_info) = self.current_frame.write().await.take() {
            frame_info.end_time = Utc::now();
            frame_info.duration = frame_info.end_time
                .signed_duration_since(frame_info.start_time)
                .to_std()
                .unwrap_or_default();

            frame_info.fps = if frame_info.duration.as_secs_f64() > 0.0 {
                1.0 / frame_info.duration.as_secs_f64()
            } else {
                0.0
            };

            frame_info.dropped = frame_info.fps < 30.0;

            self.frames.write().await.push(frame_info);
        }
    }

    /// Complete all pending events
    async fn complete_pending_events(&self) {
        let config = self.config.read().await;
        if !config.auto_complete_events {
            return;
        }

        let mut events = self.events.write().await;
        let now = Utc::now();

        for event in events.iter_mut() {
            if !event.is_complete() {
                event.end_time = Some(now);
                event.duration = event.end_time.and_then(|end| {
                    Some(end.signed_duration_since(event.start_time).to_std().ok()?)
                });
                event.phase = EventPhase::Complete;
            }
        }
    }

    /// Get all events
    pub async fn get_events(&self) -> Vec<TimelineEvent> {
        self.events.read().await.iter().cloned().collect()
    }

    /// Get events filtered by category
    pub async fn get_events_by_category(&self, category: EventCategory) -> Vec<TimelineEvent> {
        self.events.read().await
            .iter()
            .filter(|e| e.category == category)
            .cloned()
            .collect()
    }

    /// Get events in time range
    pub async fn get_events_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<TimelineEvent> {
        self.events.read().await
            .iter()
            .filter(|e| e.start_time >= start && e.start_time <= end)
            .cloned()
            .collect()
    }

    /// Get events by name pattern
    pub async fn find_events(&self, pattern: &str) -> Vec<TimelineEvent> {
        self.events.read().await
            .iter()
            .filter(|e| e.name.contains(pattern))
            .cloned()
            .collect()
    }

    /// Get all frames
    pub async fn get_frames(&self) -> Vec<FrameInfo> {
        self.frames.read().await.clone()
    }

    /// Get frame rate statistics
    pub async fn get_frame_stats(&self) -> FrameStats {
        let frames = self.frames.read().await;

        if frames.is_empty() {
            return FrameStats::default();
        }

        let total_frames = frames.len();
        let dropped_frames = frames.iter().filter(|f| f.dropped).count();

        let fps_values: Vec<f64> = frames.iter()
            .map(|f| f.fps)
            .filter(|&fps| fps > 0.0)
            .collect();

        let avg_fps = if !fps_values.is_empty() {
            fps_values.iter().sum::<f64>() / fps_values.len() as f64
        } else {
            0.0
        };

        let min_fps = fps_values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_fps = fps_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        FrameStats {
            total_frames,
            dropped_frames,
            avg_fps,
            min_fps,
            max_fps,
        }
    }

    /// Generate flame graph data
    pub async fn generate_flame_graph(&self) -> FlameGraph {
        let events = self.events.read().await;

        let nodes: Vec<FlameGraphNode> = events.iter()
            .filter(|e| e.phase == EventPhase::Complete || e.phase == EventPhase::Begin)
            .map(|e| {
                FlameGraphNode {
                    name: e.name.clone(),
                    category: e.category,
                    start: e.start_time.timestamp_millis() as f64,
                    duration: e.duration_ms(),
                    depth: self.calculate_depth(e, &events),
                }
            })
            .collect();

        FlameGraph { nodes }
    }

    fn calculate_depth(&self, event: &TimelineEvent, events: &VecDeque<TimelineEvent>) -> u32 {
        let mut depth = 0;
        let mut current = event.parent_id.clone();

        while let Some(parent_id) = current {
            if let Some(parent) = events.iter().find(|e| e.id == parent_id) {
                depth += 1;
                current = parent.parent_id.clone();
            } else {
                break;
            }
        }

        depth
    }

    /// Generate complete trace data
    pub async fn generate_trace(&self) -> TraceData {
        let events = self.events.read().await;
        let frames = self.frames.read().await;
        let start_time = *self.start_time.read().await;

        let (start, end, threads, categories) = if !events.is_empty() {
            let start = events.front().map(|e| e.start_time).unwrap_or_else(|| Utc::now());
            let end = events.back().map(|e| e.end_time.unwrap_or(e.timestamp)).unwrap_or_else(|| Utc::now());
            
            let mut thread_set: std::collections::HashSet<String> = std::collections::HashSet::new();
            let mut cat_set: std::collections::HashSet<EventCategory> = std::collections::HashSet::new();
            
            for event in events.iter() {
                if let Some(ref thread_id) = event.thread_id {
                    thread_set.insert(thread_id.clone());
                }
                cat_set.insert(event.category);
            }

            (start, end, thread_set.into_iter().collect(), cat_set.into_iter().collect())
        } else {
            (Utc::now(), Utc::now(), vec![], vec![])
        };

        let total_duration = end.signed_duration_since(start).to_std().unwrap_or_default();

        let metadata = TraceMetadata {
            start_time: start,
            end_time: end,
            total_duration,
            total_events: events.len(),
            threads,
            categories,
        };

        TraceData {
            events: events.iter().cloned().collect(),
            frames: frames.clone(),
            metadata,
        }
    }

    /// Clear all recorded data
    pub async fn clear(&self) {
        self.events.write().await.clear();
        self.frames.write().await.clear();
        self.event_map.write().await.clear();
        *self.start_time.write().await = None;
    }

    /// Export to JSON
    pub async fn export_json(&self) -> Result<String, serde_json::Error> {
        let trace = self.generate_trace().await;
        serde_json::to_string_pretty(&trace)
    }
}

/// Flame graph node for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlameGraphNode {
    pub name: String,
    pub category: EventCategory,
    pub start: f64,
    pub duration: f64,
    pub depth: u32,
}

/// Flame graph data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlameGraph {
    pub nodes: Vec<FlameGraphNode>,
}

/// Frame statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameStats {
    pub total_frames: usize,
    pub dropped_frames: usize,
    pub avg_fps: f64,
    pub min_fps: f64,
    pub max_fps: f64,
}

impl Default for FrameStats {
    fn default() -> Self {
        Self {
            total_frames: 0,
            dropped_frames: 0,
            avg_fps: 0.0,
            min_fps: 0.0,
            max_fps: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_timeline_recorder_basic() {
        let recorder = TimelineRecorder::new(TimelineRecorderConfig::default());
        recorder.start_recording().await;

        let event_id = recorder.begin_event(
            "Test Event".to_string(),
            EventCategory::Rendering,
            Some("main".to_string()),
        ).await;

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        recorder.end_event(&event_id).await.unwrap();
        recorder.stop_recording().await;

        let events = recorder.get_events().await;
        assert!(!events.is_empty());
    }

    #[test]
    fn test_event_creation() {
        let event = TimelineEvent::new("Test".to_string(), EventCategory::Scripting, EventPhase::Begin);
        assert_eq!(event.name, "Test");
        assert_eq!(event.category, EventCategory::Scripting);
        assert!(!event.is_complete());
    }
}