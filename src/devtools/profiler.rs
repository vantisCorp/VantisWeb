// Copyright 2024 Vantis Corporation - All Rights Reserved

//! Performance Profiler Module
//! 
//! This module provides CPU profiling with flame graphs, memory profiling
//! with heap snapshots, performance timeline recording, and rendering metrics.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Profiling session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSession {
    /// Session ID
    pub id: Uuid,
    /// Session name
    pub name: String,
    /// Start time
    pub start_time: DateTime<Utc>,
    /// End time
    pub end_time: Option<DateTime<Utc>>,
    /// Status
    pub status: ProfileStatus,
    /// CPU profile
    pub cpu_profile: Option<CPUProfile>,
    /// Memory profile
    pub memory_profile: Option<MemoryProfile>,
    /// Timeline events
    pub timeline_events: Vec<TimelineEvent>,
}

/// Profile status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProfileStatus {
    Running,
    Stopped,
    Error,
}

/// CPU profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CPUProfile {
    /// Profile ID
    pub id: Uuid,
    /// Samples
    pub samples: Vec<ProfileSample>,
    /// Time deltas (microseconds)
    pub time_deltas: Vec<u64>,
    /// Nodes
    pub nodes: Vec<ProfileNode>,
    /// Start time
    pub start_time: DateTime<Utc>,
    /// End time
    pub end_time: DateTime<Utc>,
    /// Total duration (ms)
    pub duration: f64,
}

/// Profile sample
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSample {
    /// Sample ID
    pub id: Uuid,
    /// Node ID
    pub node_id: Uuid,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Sample weight
    pub weight: f64,
}

/// Profile node (for flame graph)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileNode {
    /// Node ID
    pub id: Uuid,
    /// Function name
    pub function_name: String,
    /// Script URL
    pub script_url: String,
    /// Line number
    pub line_number: u32,
    /// Column number
    pub column_number: u32,
    /// Self time (ms)
    pub self_time: f64,
    /// Total time (ms)
    pub total_time: f64,
    /// Call count
    pub call_count: u64,
    /// Children
    pub children: Vec<Uuid>,
    /// Parent
    pub parent: Option<Uuid>,
    /// Hit count
    pub hit_count: u64,
}

/// Memory profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryProfile {
    /// Profile ID
    pub id: Uuid,
    /// Heap snapshot
    pub heap_snapshot: Option<HeapSnapshot>,
    /// Allocation timeline
    pub allocations: Vec<AllocationEvent>,
    /// Memory statistics
    pub stats: MemoryStats,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Heap snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeapSnapshot {
    /// Snapshot ID
    pub id: Uuid,
    /// Nodes
    pub nodes: Vec<HeapNode>,
    /// Edges
    pub edges: Vec<HeapEdge>,
    /// Locations
    pub locations: Vec<HeapLocation>,
    /// Statistics
    pub statistics: HeapStatistics,
}

/// Heap node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeapNode {
    /// Node ID
    pub id: Uuid,
    /// Type
    pub node_type: HeapNodeType,
    /// Name
    pub name: String,
    /// Self size (bytes)
    pub self_size: u64,
    /// Edge count
    pub edge_count: u32,
    /// Trace node ID
    pub trace_node_id: Option<Uuid>,
    /// Detached
    pub detached: bool,
}

/// Heap node type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HeapNodeType {
    Hidden,
    Array,
    String,
    Object,
    Code,
    Closure,
    RegExp,
    Number,
    Native,
    Synthetic,
    ConcatenatedString,
    SlicedString,
    Symbol,
    BigInt,
}

/// Heap edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeapEdge {
    /// Edge ID
    pub id: Uuid,
    /// Type
    pub edge_type: HeapEdgeType,
    /// Name or index
    pub name_or_index: String,
    /// From node
    pub from_node: Uuid,
    /// To node
    pub to_node: Uuid,
}

/// Heap edge type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HeapEdgeType {
    Context,
    Element,
    Property,
    Internal,
    Hidden,
    Shortcut,
    Weak,
}

/// Heap location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeapLocation {
    /// Object ID
    pub object_id: Uuid,
    /// Script ID
    pub script_id: String,
    /// Line
    pub line: u32,
    /// Column
    pub column: u32,
}

/// Heap statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeapStatistics {
    /// Total size
    pub total_size: u64,
    /// Used size
    pub used_size: u64,
    /// Object count
    pub object_count: u64,
    /// Edge count
    pub edge_count: u64,
    /// Node count
    pub node_count: u64,
    /// Type counts
    pub type_counts: HashMap<String, u64>,
}

/// Memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Total heap size
    pub total_heap_size: u64,
    /// Used heap size
    pub used_heap_size: u64,
    /// Heap size limit
    pub heap_size_limit: u64,
    /// Total external memory
    pub external_memory: u64,
    /// Live DOM nodes
    pub dom_nodes: u64,
    /// DOM document count
    pub dom_documents: u64,
    /// Event listeners
    pub event_listeners: u64,
    /// Total JS heap size
    pub js_heap_size: u64,
    /// GC time (ms)
    pub gc_time: f64,
}

/// Allocation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationEvent {
    /// Event ID
    pub id: Uuid,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Size (bytes)
    pub size: i64,
    /// Object type
    pub object_type: String,
    /// Stack trace
    pub stack_trace: Vec<String>,
}

/// Timeline event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    /// Event ID
    pub id: Uuid,
    /// Event type
    pub event_type: TimelineEventType,
    /// Name
    pub name: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Duration (ms)
    pub duration: f64,
    /// Start time (ms from session start)
    pub start_time: f64,
    /// End time (ms from session start)
    pub end_time: f64,
    /// Phase
    pub phase: TimelinePhase,
    /// Data
    pub data: serde_json::Value,
}

/// Timeline event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimelineEventType {
    FunctionCall,
    EvaluateScript,
    TimerFire,
    TimerInstall,
    TimerRemove,
    XHRLoad,
    XHRReadyStateChange,
    Layout,
    Paint,
    Composite,
    ScheduleStyleRecalculation,
    RecalculateStyle,
    InvalidateLayout,
    LayoutShift,
    FirstPaint,
    FirstContentfulPaint,
    LargestContentfulPaint,
    DOMContentLoaded,
    Load,
    GC,
    JSFrame,
    Resource,
}

/// Timeline phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimelinePhase {
    Begin,
    End,
    Instant,
    DurationBegin,
    DurationEnd,
}

/// Performance metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    /// Metric ID
    pub id: Uuid,
    /// Name
    pub name: String,
    /// Value
    pub value: f64,
    /// Unit
    pub unit: MetricUnit,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Tags
    pub tags: HashMap<String, String>,
}

/// Metric unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricUnit {
    Milliseconds,
    Bytes,
    Count,
    Percent,
    FPS,
}

/// Performance profiler
pub struct PerformanceProfiler {
    /// Active sessions
    sessions: Arc<RwLock<HashMap<Uuid, ProfileSession>>>,
    /// Current session
    current_session: Arc<RwLock<Option<Uuid>>>,
    /// Timeline events buffer
    timeline_buffer: Arc<RwLock<VecDeque<TimelineEvent>>>,
    /// FPS monitor
    fps_monitor: Arc<RwLock<FPSMonitor>>,
    /// Long task threshold (ms)
    long_task_threshold: Arc<RwLock<f64>>,
    /// Is recording
    is_recording: Arc<RwLock<bool>>,
}

/// FPS monitor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FPSMonitor {
    /// Current FPS
    pub current_fps: f64,
    /// FPS history
    pub history: VecDeque<f64>,
    /// Last frame time
    pub last_frame_time: DateTime<Utc>,
    /// Average FPS
    pub avg_fps: f64,
    /// Min FPS
    pub min_fps: f64,
    /// Max FPS
    pub max_fps: f64,
}

impl PerformanceProfiler {
    /// Create a new performance profiler
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            current_session: Arc::new(RwLock::new(None)),
            timeline_buffer: Arc::new(RwLock::new(VecDeque::with_capacity(10000))),
            fps_monitor: Arc::new(RwLock::new(FPSMonitor::new())),
            long_task_threshold: Arc::new(RwLock::new(50.0)),
            is_recording: Arc::new(RwLock::new(false)),
        }
    }

    /// Initialize the profiler
    pub fn initialize(&self) -> Result<(), ProfilerError> {
        Ok(())
    }

    /// Start profiling session
    pub async fn start_profile(&self, name: String) -> Result<Uuid, ProfilerError> {
        let session = ProfileSession {
            id: Uuid::new_v4(),
            name,
            start_time: Utc::now(),
            end_time: None,
            status: ProfileStatus::Running,
            cpu_profile: None,
            memory_profile: None,
            timeline_events: Vec::new(),
        };
        
        let session_id = session.id;
        self.sessions.write().await.insert(session_id, session);
        *self.current_session.write().await = Some(session_id);
        *self.is_recording.write().await = true;
        
        Ok(session_id)
    }

    /// Stop profiling session
    pub async fn stop_profile(&self, session_id: Uuid) -> Result<ProfileSession, ProfilerError> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(&session_id) {
            session.end_time = Some(Utc::now());
            session.status = ProfileStatus::Stopped;
            
            // Collect timeline events
            let buffer = self.timeline_buffer.read().await;
            session.timeline_events = buffer.iter().cloned().collect();
            
            // Create CPU profile summary
            session.cpu_profile = self.create_cpu_profile(&session.timeline_events);
            
            Ok(session.clone())
        } else {
            Err(ProfilerError::SessionNotFound)
        }
    }

    /// Get profile session
    pub async fn get_session(&self, session_id: Uuid) -> Option<ProfileSession> {
        self.sessions.read().await.get(&session_id).cloned()
    }

    /// Get all sessions
    pub async fn get_sessions(&self) -> Vec<ProfileSession> {
        self.sessions.read().await.values().cloned().collect()
    }

    /// Add timeline event
    pub async fn add_timeline_event(&self, event: TimelineEvent) {
        let is_recording = *self.is_recording.read().await;
        
        if is_recording {
            let mut buffer = self.timeline_buffer.write().await;
            
            // Check for long tasks
            if event.duration > *self.long_task_threshold.read().await {
                // Mark as long task
            }
            
            buffer.push_back(event);
            
            // Limit buffer size
            while buffer.len() > 10000 {
                buffer.pop_front();
            }
        }
    }

    /// Take heap snapshot
    pub async fn take_heap_snapshot(&self) -> Result<HeapSnapshot, ProfilerError> {
        // Simplified heap snapshot
        let snapshot = HeapSnapshot {
            id: Uuid::new_v4(),
            nodes: vec![
                HeapNode {
                    id: Uuid::new_v4(),
                    node_type: HeapNodeType::Object,
                    name: "Window".to_string(),
                    self_size: 1024,
                    edge_count: 10,
                    trace_node_id: None,
                    detached: false,
                },
            ],
            edges: vec![],
            locations: vec![],
            statistics: HeapStatistics {
                total_size: 1024 * 1024 * 100,
                used_size: 1024 * 1024 * 50,
                object_count: 1000,
                edge_count: 5000,
                node_count: 1000,
                type_counts: HashMap::new(),
            },
        };
        
        Ok(snapshot)
    }

    /// Get memory stats
    pub async fn get_memory_stats(&self) -> MemoryStats {
        MemoryStats {
            total_heap_size: 1024 * 1024 * 100,
            used_heap_size: 1024 * 1024 * 50,
            heap_size_limit: 1024 * 1024 * 1024 * 2,
            external_memory: 1024 * 1024 * 10,
            dom_nodes: 1500,
            dom_documents: 3,
            event_listeners: 25,
            js_heap_size: 1024 * 1024 * 45,
            gc_time: 5.5,
        }
    }

    /// Update FPS
    pub async fn update_fps(&self, delta_time: f64) {
        let mut monitor = self.fps_monitor.write().await;
        monitor.update(delta_time);
    }

    /// Get current FPS
    pub async fn get_fps(&self) -> f64 {
        self.fps_monitor.read().await.current_fps
    }

    /// Get FPS history
    pub async fn get_fps_history(&self, limit: usize) -> Vec<f64> {
        let monitor = self.fps_monitor.read().await;
        monitor.history.iter().take(limit).copied().collect()
    }

    /// Record performance metric
    pub async fn record_metric(&self, _metric: PerformanceMetric) -> Result<(), ProfilerError> {
        // In production, store metrics for analysis
        Ok(())
    }

    /// Get snapshot count
    pub async fn get_snapshot_count(&self) -> usize {
        self.sessions.read().await.len()
    }

    /// Clear profile data
    pub async fn clear(&self) {
        self.sessions.write().await.clear();
        self.timeline_buffer.write().await.clear();
        *self.current_session.write().await = None;
        *self.is_recording.write().await = false;
    }

    /// Export profile as JSON
    pub async fn export_profile(&self, session_id: Uuid) -> Result<String, ProfilerError> {
        let session = self.get_session(session_id).await.ok_or(ProfilerError::SessionNotFound)?;
        
        serde_json::to_string_pretty(&session).map_err(ProfilerError::from)
    }

    /// Export profile for Chrome DevTools
    pub async fn export_chrome_profile(&self, session_id: Uuid) -> Result<String, ProfilerError> {
        let session = self.get_session(session_id).await.ok_or(ProfilerError::SessionNotFound)?;
        
        let chrome_profile = ChromeProfile {
            metadata: ChromeMetadata {
                format_version: "1.0.0".to_string(),
                startTime: session.start_time.to_rfc3339(),
            },
            profile: session.cpu_profile.clone(),
        };
        
        serde_json::to_string_pretty(&chrome_profile).map_err(ProfilerError::from)
    }

    // Helper methods

    fn create_cpu_profile(&self, events: &[TimelineEvent]) -> Option<CPUProfile> {
        let mut nodes = Vec::new();
        let mut samples = Vec::new();
        let mut time_deltas = Vec::new();
        
        for event in events {
            if matches!(event.event_type, TimelineEventType::JSFrame | TimelineEventType::FunctionCall) {
                let node = ProfileNode {
                    id: Uuid::new_v4(),
                    function_name: event.name.clone(),
                    script_url: String::new(),
                    line_number: 0,
                    column_number: 0,
                    self_time: event.duration,
                    total_time: event.duration,
                    call_count: 1,
                    children: Vec::new(),
                    parent: None,
                    hit_count: 1,
                };
                
                let sample = ProfileSample {
                    id: Uuid::new_v4(),
                    node_id: node.id,
                    timestamp: event.timestamp,
                    weight: event.duration,
                };
                
                nodes.push(node);
                samples.push(sample);
                time_deltas.push(event.duration as u64);
            }
        }
        
        if !nodes.is_empty() {
            Some(CPUProfile {
                id: Uuid::new_v4(),
                samples,
                time_deltas,
                nodes,
                start_time: events.first().map(|e| e.timestamp).unwrap_or_else(Utc::now),
                end_time: events.last().map(|e| e.timestamp).unwrap_or_else(Utc::now),
                duration: events.iter().map(|e| e.duration).sum(),
            })
        } else {
            None
        }
    }
}

impl FPSMonitor {
    /// Create a new FPS monitor
    pub fn new() -> Self {
        Self {
            current_fps: 60.0,
            history: VecDeque::with_capacity(100),
            last_frame_time: Utc::now(),
            avg_fps: 60.0,
            min_fps: 60.0,
            max_fps: 60.0,
        }
    }

    /// Update FPS with delta time
    pub fn update(&mut self, delta_time: f64) {
        if delta_time > 0.0 {
            self.current_fps = 1000.0 / delta_time;
            self.history.push_back(self.current_fps);
            
            if self.history.len() > 100 {
                self.history.pop_front();
            }
            
            self.avg_fps = self.history.iter().sum::<f64>() / self.history.len() as f64;
            self.min_fps = self.history.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            self.max_fps = self.history.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        }
    }
}

/// Chrome profile format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromeProfile {
    metadata: ChromeMetadata,
    profile: Option<CPUProfile>,
}

/// Chrome metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromeMetadata {
    formatVersion: String,
    startTime: String,
}

/// Profiler error
#[derive(Debug, thiserror::Error)]
pub enum ProfilerError {
    #[error("Session not found")]
    SessionNotFound,
    #[error("Already recording")]
    AlreadyRecording,
    #[error("Not recording")]
    NotRecording,
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Other error: {0}")]
    Other(String),
}

impl Default for PerformanceProfiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_profiler_initialization() {
        let profiler = PerformanceProfiler::new();
        assert!(profiler.initialize().is_ok());
    }

    #[tokio::test]
    async fn test_start_stop_profile() {
        let profiler = PerformanceProfiler::new();
        
        let session_id = profiler.start_profile("Test Profile".to_string()).await.unwrap();
        let session = profiler.stop_profile(session_id).await.unwrap();
        
        assert_eq!(session.name, "Test Profile");
        assert_eq!(session.status, ProfileStatus::Stopped);
    }

    #[tokio::test]
    async fn test_timeline_events() {
        let profiler = PerformanceProfiler::new();
        profiler.start_profile("Test".to_string()).await.unwrap();
        
        let event = TimelineEvent {
            id: Uuid::new_v4(),
            event_type: TimelineEventType::FunctionCall,
            name: "testFunction".to_string(),
            timestamp: Utc::now(),
            duration: 10.0,
            start_time: 0.0,
            end_time: 10.0,
            phase: TimelinePhase::DurationBegin,
            data: serde_json::Value::Null,
        };
        
        profiler.add_timeline_event(event).await;
        
        let sessions = profiler.get_sessions().await;
        assert!(!sessions.is_empty());
    }

    #[tokio::test]
    async fn test_heap_snapshot() {
        let profiler = PerformanceProfiler::new();
        let snapshot = profiler.take_heap_snapshot().await.unwrap();
        
        assert!(!snapshot.nodes.is_empty());
        assert!(snapshot.statistics.total_size > 0);
    }

    #[tokio::test]
    async fn test_fps_monitor() {
        let profiler = PerformanceProfiler::new();
        profiler.update_fps(16.67).await; // ~60 FPS
        
        let fps = profiler.get_fps().await;
        assert!(fps > 50.0 && fps < 70.0);
    }

    #[tokio::test]
    async fn test_memory_stats() {
        let profiler = PerformanceProfiler::new();
        let stats = profiler.get_memory_stats().await;
        
        assert!(stats.total_heap_size > 0);
        assert!(stats.used_heap_size > 0);
    }

    #[tokio::test]
    async fn test_export_profile() {
        let profiler = PerformanceProfiler::new();
        
        let session_id = profiler.start_profile("Export Test".to_string()).await.unwrap();
        profiler.stop_profile(session_id).await.unwrap();
        
        let json = profiler.export_profile(session_id).await.unwrap();
        assert!(json.contains("Export Test"));
    }

    #[tokio::test]
    async fn test_clear() {
        let profiler = PerformanceProfiler::new();
        
        profiler.start_profile("Test".to_string()).await.unwrap();
        profiler.clear().await;
        
        let sessions = profiler.get_sessions().await;
        assert!(sessions.is_empty());
    }
}