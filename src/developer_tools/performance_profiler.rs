//! Performance Profiler
//! 
//! Captures and analyzes performance metrics with timeline visualization.

use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use anyhow::Result;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

use super::{
    DevToolsEvent, PerformanceMetric, PerformanceAnalysis,
};
use super::models::{PerformanceEntry, PerformanceEntryType};

/// Performance profiler
pub struct PerformanceProfiler {
    /// Performance entries
    entries: RwLock<VecDeque<PerformanceEntry>>,
    /// Active recordings
    recordings: RwLock<Vec<ProfileRecording>>,
    /// Metrics buffer
    metrics: RwLock<VecDeque<PerformanceMetric>>,
    /// Custom marks
    marks: RwLock<Vec<PerformanceMark>>,
    /// Custom measures
    measures: RwLock<Vec<PerformanceMeasure>>,
    /// Event sender
    event_sender: broadcast::Sender<DevToolsEvent>,
    /// Configuration
    config: RwLock<PerformanceConfig>,
    /// Profiling state
    is_profiling: RwLock<bool>,
}

/// Performance configuration
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Enable continuous profiling
    pub continuous_profiling: bool,
    /// Sample interval in ms
    pub sample_interval_ms: u32,
    /// Maximum entries to keep
    pub max_entries: usize,
    /// Enable memory metrics
    pub track_memory: bool,
    /// Enable layout metrics
    pub track_layout: bool,
    /// Enable paint metrics
    pub track_paint: bool,
    /// Long task threshold in ms
    pub long_task_threshold_ms: u32,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            continuous_profiling: true,
            sample_interval_ms: 100,
            max_entries: 100000,
            track_memory: true,
            track_layout: true,
            track_paint: true,
            long_task_threshold_ms: 50,
        }
    }
}

/// Profile recording
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileRecording {
    /// Recording ID
    pub id: String,
    /// Recording name
    pub name: String,
    /// Start time
    pub start_time: DateTime<Utc>,
    /// End time
    pub end_time: Option<DateTime<Utc>>,
    /// Duration in ms
    pub duration_ms: Option<u64>,
    /// Entries captured
    pub entry_count: usize,
    /// Is active
    pub is_active: bool,
}

/// Performance mark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMark {
    /// Mark name
    pub name: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Start time (relative to navigation)
    pub start_time: f64,
}

/// Performance measure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMeasure {
    /// Measure name
    pub name: String,
    /// Start mark
    pub start_mark: String,
    /// End mark
    pub end_mark: String,
    /// Duration in ms
    pub duration: f64,
    /// Start time
    pub start_time: f64,
}

impl PerformanceProfiler {
    /// Create a new performance profiler
    pub async fn new(event_sender: broadcast::Sender<DevToolsEvent>) -> Result<Self> {
        Ok(Self {
            entries: RwLock::new(VecDeque::new()),
            recordings: RwLock::new(Vec::new()),
            metrics: RwLock::new(VecDeque::new()),
            marks: RwLock::new(Vec::new()),
            measures: RwLock::new(Vec::new()),
            event_sender,
            config: RwLock::new(PerformanceConfig::default()),
            is_profiling: RwLock::new(false),
        })
    }
    
    /// Start profiling session
    pub async fn start_profiling(&self, name: &str) -> Result<String> {
        let mut is_profiling = self.is_profiling.write().await;
        
        if *is_profiling {
            return Err(anyhow::anyhow!("Already profiling"));
        }
        
        let recording = ProfileRecording {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            start_time: Utc::now(),
            end_time: None,
            duration_ms: None,
            entry_count: 0,
            is_active: true,
        };
        
        let id = recording.id.clone();
        
        let mut recordings = self.recordings.write().await;
        recordings.push(recording);
        
        *is_profiling = true;
        
        let _ = self.event_sender.send(DevToolsEvent::ProfileStarted(name.to_string()));
        
        tracing::info!("Performance profiling started: {}", name);
        Ok(id)
    }
    
    /// Stop profiling session
    pub async fn stop_profiling(&self, recording_id: &str) -> Result<ProfileRecording> {
        let mut is_profiling = self.is_profiling.write().await;
        *is_profiling = false;
        
        let mut recordings = self.recordings.write().await;
        
        if let Some(recording) = recordings.iter_mut().find(|r| r.id == recording_id) {
            recording.end_time = Some(Utc::now());
            recording.duration_ms = Some(
                (recording.end_time.unwrap() - recording.start_time)
                    .num_milliseconds() as u64
            );
            recording.is_active = false;
            
            let _ = self.event_sender.send(DevToolsEvent::ProfileCompleted(
                recording.name.clone(),
                recording.id.clone(),
            ));
            
            return Ok(recording.clone());
        }
        
        Err(anyhow::anyhow!("Recording not found"))
    }
    
    /// Add performance entry
    pub async fn add_entry(&self, entry: PerformanceEntry) -> Result<()> {
        let config = self.config.read().await;
        
        let mut entries = self.entries.write().await;
        
        // Enforce max entries
        while entries.len() >= config.max_entries {
            entries.pop_front();
        }
        
        entries.push_back(entry.clone());
        
        // Check for long task
        if entry.entry_type == PerformanceEntryType::LongTask && 
           entry.duration > config.long_task_threshold_ms as f64 {
            tracing::warn!("Long task detected: {:.2}ms", entry.duration);
        }
        
        // Emit event
        let _ = self.event_sender.send(DevToolsEvent::PerformanceMetric(
            entry.name.clone(),
            entry.duration,
        ));
        
        Ok(())
    }
    
    /// Add metric
    pub async fn add_metric(&self, name: &str, value: f64, unit: &str) -> Result<()> {
        let metric = PerformanceMetric {
            name: name.to_string(),
            value,
            unit: unit.to_string(),
            timestamp: Utc::now(),
        };
        
        let config = self.config.read().await;
        let mut metrics = self.metrics.write().await;
        
        while metrics.len() >= config.max_entries {
            metrics.pop_front();
        }
        
        metrics.push_back(metric);
        
        Ok(())
    }
    
    /// Create performance mark
    pub async fn mark(&self, name: &str, start_time: f64) -> Result<()> {
        let mark = PerformanceMark {
            name: name.to_string(),
            timestamp: Utc::now(),
            start_time,
        };
        
        let mut marks = self.marks.write().await;
        marks.push(mark);
        
        Ok(())
    }
    
    /// Create performance measure
    pub async fn measure(
        &self,
        name: &str,
        start_mark: &str,
        end_mark: &str,
    ) -> Result<PerformanceMeasure> {
        let marks = self.marks.read().await;
        
        let start = marks.iter().find(|m| m.name == start_mark)
            .ok_or_else(|| anyhow::anyhow!("Start mark not found"))?;
        
        let end = marks.iter().find(|m| m.name == end_mark)
            .ok_or_else(|| anyhow::anyhow!("End mark not found"))?;
        
        let duration = end.start_time - start.start_time;
        
        let measure = PerformanceMeasure {
            name: name.to_string(),
            start_mark: start_mark.to_string(),
            end_mark: end_mark.to_string(),
            duration,
            start_time: start.start_time,
        };
        
        let mut measures = self.measures.write().await;
        measures.push(measure.clone());
        
        Ok(measure)
    }
    
    /// Get all entries
    pub async fn get_entries(&self) -> Result<Vec<PerformanceEntry>> {
        let entries = self.entries.read().await;
        Ok(entries.iter().cloned().collect())
    }
    
    /// Get entries by type
    pub async fn get_entries_by_type(
        &self,
        entry_type: PerformanceEntryType,
    ) -> Result<Vec<PerformanceEntry>> {
        let entries = self.entries.read().await;
        Ok(entries.iter()
            .filter(|e| e.entry_type == entry_type)
            .cloned()
            .collect())
    }
    
    /// Get metrics
    pub async fn get_metrics(&self) -> Result<Vec<PerformanceMetric>> {
        let metrics = self.metrics.read().await;
        Ok(metrics.iter().cloned().collect())
    }
    
    /// Get marks
    pub async fn get_marks(&self) -> Result<Vec<PerformanceMark>> {
        let marks = self.marks.read().await;
        Ok(marks.clone())
    }
    
    /// Get measures
    pub async fn get_measures(&self) -> Result<Vec<PerformanceMeasure>> {
        let measures = self.measures.read().await;
        Ok(measures.clone())
    }
    
    /// Clear all data
    pub async fn clear(&self) -> Result<()> {
        let mut entries = self.entries.write().await;
        let mut metrics = self.metrics.write().await;
        let mut marks = self.marks.write().await;
        let mut measures = self.measures.write().await;
        
        entries.clear();
        metrics.clear();
        marks.clear();
        measures.clear();
        
        Ok(())
    }
    
    /// Record navigation timing
    pub async fn record_navigation(&self, timing: NavigationTiming) -> Result<()> {
        // Add key navigation metrics
        self.add_metric("DNS Time", timing.dns_time_ms as f64, "ms").await?;
        self.add_metric("Connect Time", timing.connect_time_ms as f64, "ms").await?;
        self.add_metric("TTFB", timing.ttfb_ms as f64, "ms").await?;
        self.add_metric("DOM Content Loaded", timing.dom_content_loaded_ms as f64, "ms").await?;
        self.add_metric("Load Complete", timing.load_complete_ms as f64, "ms").await?;
        
        // Add entries
        if timing.ttfb_ms > 0 {
            self.add_entry(PerformanceEntry {
                id: Uuid::new_v4().to_string(),
                entry_type: PerformanceEntryType::Navigation,
                name: "navigation".to_string(),
                start_time: 0.0,
                duration: timing.load_complete_ms as f64,
                timestamp: Utc::now(),
                data: serde_json::to_value(&timing)?,
            }).await?;
        }
        
        Ok(())
    }
    
    /// Record paint timing
    pub async fn record_paint(&self, paint_type: &str, timestamp_ms: f64) -> Result<()> {
        self.add_entry(PerformanceEntry {
            id: Uuid::new_v4().to_string(),
            entry_type: PerformanceEntryType::Paint,
            name: format!("paint-{}", paint_type),
            start_time: 0.0,
            duration: timestamp_ms,
            timestamp: Utc::now(),
            data: serde_json::json!({ "paintType": paint_type }),
        }).await?;
        
        // Add metric
        self.add_metric(&format!("First {} Paint", paint_type), timestamp_ms, "ms").await?;
        
        Ok(())
    }
    
    /// Record long task
    pub async fn record_long_task(
        &self,
        start_time: f64,
        duration: f64,
        source: &str,
    ) -> Result<()> {
        self.add_entry(PerformanceEntry {
            id: Uuid::new_v4().to_string(),
            entry_type: PerformanceEntryType::LongTask,
            name: "longtask".to_string(),
            start_time,
            duration,
            timestamp: Utc::now(),
            data: serde_json::json!({ "source": source }),
        }).await?;
        
        Ok(())
    }
    
    /// Record layout shift
    pub async fn record_layout_shift(
        &self,
        value: f64,
        sources: Vec<LayoutShiftSource>,
    ) -> Result<()> {
        self.add_entry(PerformanceEntry {
            id: Uuid::new_v4().to_string(),
            entry_type: PerformanceEntryType::LayoutShift,
            name: "layout-shift".to_string(),
            start_time: 0.0,
            duration: 0.0,
            timestamp: Utc::now(),
            data: serde_json::json!({
                "value": value,
                "sources": sources
            }),
        }).await?;
        
        Ok(())
    }
    
    /// Analyze performance
    pub async fn analyze(&self) -> Result<PerformanceAnalysis> {
        let entries = self.entries.read().await;
        let metrics = self.metrics.read().await;
        
        // Count long tasks
        let long_tasks = entries.iter()
            .filter(|e| e.entry_type == PerformanceEntryType::LongTask)
            .count() as u32;
        
        // Count layout shifts
        let layout_shifts = entries.iter()
            .filter(|e| e.entry_type == PerformanceEntryType::LayoutShift)
            .count() as u32;
        
        // Get average frame time
        let frame_times: Vec<f64> = entries.iter()
            .filter(|e| e.entry_type == PerformanceEntryType::LongTask)
            .map(|e| e.duration)
            .collect();
        
        let avg_frame_time = if !frame_times.is_empty() {
            frame_times.iter().sum::<f64>() / frame_times.len() as f64
        } else {
            16.67 // Default 60fps
        };
        
        // Get paint metrics
        let fcp_ms = metrics.iter()
            .find(|m| m.name.contains("First Contentful Paint") || m.name == "paint-first-contentful-paint")
            .map(|m| m.value)
            .unwrap_or(0.0);
        
        let lcp_ms = metrics.iter()
            .find(|m| m.name.contains("Largest Contentful Paint"))
            .map(|m| m.value)
            .unwrap_or(0.0);
        
        // Calculate CLS
        let cls: f64 = entries.iter()
            .filter(|e| e.entry_type == PerformanceEntryType::LayoutShift)
            .map(|e| e.data.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0))
            .sum();
        
        Ok(PerformanceAnalysis {
            long_tasks,
            avg_frame_time_ms: avg_frame_time,
            layout_shifts,
            fcp_ms,
            lcp_ms,
            cls,
        })
    }
    
    /// Get timeline data
    pub async fn get_timeline(&self, start_ms: f64, end_ms: f64) -> Result<TimelineData> {
        let entries = self.entries.read().await;
        
        let filtered: Vec<PerformanceEntry> = entries.iter()
            .filter(|e| e.start_time >= start_ms && e.start_time <= end_ms)
            .cloned()
            .collect();
        
        let tracks = self.group_entries_to_tracks(&filtered);
        
        Ok(TimelineData {
            start_ms,
            end_ms,
            tracks,
            total_entries: filtered.len(),
        })
    }
    
    /// Group entries into timeline tracks
    fn group_entries_to_tracks(&self, entries: &[PerformanceEntry]) -> Vec<TimelineTrack> {
        let mut tracks: Vec<TimelineTrack> = Vec::new();
        
        // Group by entry type
        let mut type_entries: std::collections::HashMap<String, Vec<&PerformanceEntry>> = 
            std::collections::HashMap::new();
        
        for entry in entries {
            let type_name = format!("{:?}", entry.entry_type);
            type_entries.entry(type_name).or_default().push(entry);
        }
        
        for (type_name, entries) in type_entries {
            let events: Vec<TimelineEvent> = entries.iter()
                .map(|e| TimelineEvent {
                    id: e.id.clone(),
                    name: e.name.clone(),
                    start_ms: e.start_time,
                    duration_ms: e.duration,
                    data: e.data.clone(),
                })
                .collect();
            
            tracks.push(TimelineTrack {
                name: type_name,
                events,
            });
        }
        
        tracks
    }
    
    /// Export profile
    pub async fn export_profile(&self, recording_id: &str) -> Result<serde_json::Value> {
        let recordings = self.recordings.read().await;
        let entries = self.entries.read().await;
        
        let recording = recordings.iter()
            .find(|r| r.id == recording_id)
            .ok_or_else(|| anyhow::anyhow!("Recording not found"))?;
        
        let profile_entries: Vec<&PerformanceEntry> = entries.iter()
            .filter(|e| {
                let entry_time = e.timestamp;
                entry_time >= recording.start_time && 
                recording.end_time.map_or(true, |end| entry_time <= end)
            })
            .collect();
        
        Ok(serde_json::json!({
            "recording": recording,
            "entries": profile_entries,
            "analysis": self.analyze().await?
        }))
    }
}

/// Navigation timing data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationTiming {
    pub dns_time_ms: u64,
    pub connect_time_ms: u64,
    pub ssl_time_ms: u64,
    pub ttfb_ms: u64,
    pub dom_content_loaded_ms: u64,
    pub load_complete_ms: u64,
    pub dom_interactive_ms: u64,
    pub redirect_time_ms: u64,
    pub app_cache_time_ms: u64,
    pub request_time_ms: u64,
    pub response_time_ms: u64,
    pub processing_time_ms: u64,
}

/// Layout shift source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutShiftSource {
    pub node_id: u64,
    pub current_rect: LayoutRect,
    pub previous_rect: LayoutRect,
}

/// Layout rectangle
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LayoutRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Timeline data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineData {
    pub start_ms: f64,
    pub end_ms: f64,
    pub tracks: Vec<TimelineTrack>,
    pub total_entries: usize,
}

/// Timeline track
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineTrack {
    pub name: String,
    pub events: Vec<TimelineEvent>,
}

/// Timeline event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub id: String,
    pub name: String,
    pub start_ms: f64,
    pub duration_ms: f64,
    pub data: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_performance_profiler_creation() {
        let (tx, _rx) = broadcast::channel(16);
        let profiler = PerformanceProfiler::new(tx).await.unwrap();
        
        let entries = profiler.get_entries().await.unwrap();
        assert!(entries.is_empty());
    }
    
    #[tokio::test]
    async fn test_profiling_session() {
        let (tx, _rx) = broadcast::channel(16);
        let profiler = PerformanceProfiler::new(tx).await.unwrap();
        
        let id = profiler.start_profiling("test").await.unwrap();
        assert!(!id.is_empty());
        
        let recording = profiler.stop_profiling(&id).await.unwrap();
        assert!(!recording.is_active);
    }
    
    #[tokio::test]
    async fn test_add_metric() {
        let (tx, _rx) = broadcast::channel(16);
        let profiler = PerformanceProfiler::new(tx).await.unwrap();
        
        profiler.add_metric("FPS", 60.0, "fps").await.unwrap();
        
        let metrics = profiler.get_metrics().await.unwrap();
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].name, "FPS");
    }
}