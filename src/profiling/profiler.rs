/// # Performance Profiler Module
/// 
/// Measures page load times, rendering metrics, and Core Web Vitals.

use std::collections::HashMap;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use chrono::{DateTime, Utc};

use super::{Metric, MetricType, Result, ProfilingError};

/// Page timing metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageTimings {
    /// DNS lookup time
    pub dns_lookup: f64,
    /// TCP connection time
    pub tcp_connection: f64,
    /// TLS negotiation time
    pub tls_negotiation: f64,
    /// Time to first byte
    pub ttfb: f64,
    /// Content download time
    pub download_time: f64,
    /// DOM parsing time
    pub dom_parsing: f64,
    /// DOM content loaded
    pub dom_content_loaded: f64,
    /// Window load event
    pub load_event: f64,
    /// Total page load time
    pub total: f64,
}

/// Rendering metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderingMetrics {
    /// First paint
    pub first_paint: f64,
    /// First contentful paint
    pub first_contentful_paint: f64,
    /// Largest contentful paint
    pub largest_contentful_paint: f64,
    /// Speed index
    pub speed_index: f64,
    /// Time to interactive
    pub time_to_interactive: f64,
    /// Total blocking time
    pub total_blocking_time: f64,
    /// Cumulative layout shift
    pub cumulative_layout_shift: f64,
}

/// Performance profiler
pub struct PerformanceProfiler {
    /// Current page URL
    current_page: Arc<RwLock<Option<String>>>,
    /// Page timings
    timings: Arc<RwLock<PageTimings>>,
    /// Rendering metrics
    rendering: Arc<RwLock<RenderingMetrics>>,
    /// Custom metrics
    custom_metrics: Arc<RwLock<HashMap<String, f64>>>,
    /// Start time
    start_time: Arc<RwLock<Option<DateTime<Utc>>>>,
    /// Is profiling
    is_profiling: Arc<RwLock<bool>>,
}

impl PerformanceProfiler {
    /// Create a new performance profiler
    pub fn new() -> Self {
        Self {
            current_page: Arc::new(RwLock::new(None)),
            timings: Arc::new(RwLock::new(PageTimings {
                dns_lookup: 0.0,
                tcp_connection: 0.0,
                tls_negotiation: 0.0,
                ttfb: 0.0,
                download_time: 0.0,
                dom_parsing: 0.0,
                dom_content_loaded: 0.0,
                load_event: 0.0,
                total: 0.0,
            })),
            rendering: Arc::new(RwLock::new(RenderingMetrics {
                first_paint: 0.0,
                first_contentful_paint: 0.0,
                largest_contentful_paint: 0.0,
                speed_index: 0.0,
                time_to_interactive: 0.0,
                total_blocking_time: 0.0,
                cumulative_layout_shift: 0.0,
            })),
            custom_metrics: Arc::new(RwLock::new(HashMap::new())),
            start_time: Arc::new(RwLock::new(None)),
            is_profiling: Arc::new(RwLock::new(false)),
        }
    }

    /// Start profiling a page
    pub async fn start(&self, page_url: &str) -> Result<()> {
        *self.current_page.write().await = Some(page_url.to_string());
        *self.start_time.write().await = Some(Utc::now());
        *self.is_profiling.write().await = true;
        Ok(())
    }

    /// Stop profiling and return metrics
    pub async fn stop(&self) -> Result<Vec<Metric>> {
        *self.is_profiling.write().await = false;
        
        let page_url = self.current_page.read().await.clone().unwrap_or_default();
        let timings = self.timings.read().await;
        let rendering = self.rendering.read().await;
        
        let metrics = vec![
            Metric::new(MetricType::PageLoadTime, timings.total, "ms".to_string(), page_url.clone()),
            Metric::new(MetricType::FirstPaint, rendering.first_paint, "ms".to_string(), page_url.clone()),
            Metric::new(MetricType::FirstContentfulPaint, rendering.first_contentful_paint, "ms".to_string(), page_url.clone()),
            Metric::new(MetricType::LargestContentfulPaint, rendering.largest_contentful_paint, "ms".to_string(), page_url.clone()),
            Metric::new(MetricType::TimeToInteractive, rendering.time_to_interactive, "ms".to_string(), page_url.clone()),
            Metric::new(MetricType::TotalBlockingTime, rendering.total_blocking_time, "ms".to_string(), page_url.clone()),
            Metric::new(MetricType::CumulativeLayoutShift, rendering.cumulative_layout_shift, "".to_string(), page_url.clone()),
            Metric::new(MetricType::SpeedIndex, rendering.speed_index, "ms".to_string(), page_url),
        ];
        
        Ok(metrics)
    }

    /// Record a page timing event
    pub async fn record_timing(&self, event: TimingEvent, duration_ms: f64) -> Result<()> {
        let mut timings = self.timings.write().await;
        
        match event {
            TimingEvent::DnsLookup => timings.dns_lookup = duration_ms,
            TimingEvent::TcpConnection => timings.tcp_connection = duration_ms,
            TimingEvent::TlsNegotiation => timings.tls_negotiation = duration_ms,
            TimingEvent::Ttfb => timings.ttfb = duration_ms,
            TimingEvent::Download => timings.download_time = duration_ms,
            TimingEvent::DomParsing => timings.dom_parsing = duration_ms,
            TimingEvent::DomContentLoaded => timings.dom_content_loaded = duration_ms,
            TimingEvent::LoadEvent => timings.load_event = duration_ms,
        }
        
        Ok(())
    }

    /// Record a rendering metric
    pub async fn record_rendering_metric(&self, metric: RenderingMetric, value: f64) -> Result<()> {
        let mut rendering = self.rendering.write().await;
        
        match metric {
            RenderingMetric::FirstPaint => rendering.first_paint = value,
            RenderingMetric::FirstContentfulPaint => rendering.first_contentful_paint = value,
            RenderingMetric::LargestContentfulPaint => rendering.largest_contentful_paint = value,
            RenderingMetric::SpeedIndex => rendering.speed_index = value,
            RenderingMetric::TimeToInteractive => rendering.time_to_interactive = value,
            RenderingMetric::TotalBlockingTime => rendering.total_blocking_time = value,
            RenderingMetric::CumulativeLayoutShift => rendering.cumulative_layout_shift = value,
        }
        
        Ok(())
    }

    /// Add custom metric
    pub async fn add_custom_metric(&self, name: &str, value: f64) -> Result<()> {
        self.custom_metrics.write().await.insert(name.to_string(), value);
        Ok(())
    }

    /// Get page timings
    pub async fn get_timings(&self) -> PageTimings {
        self.timings.read().await.clone()
    }

    /// Get rendering metrics
    pub async fn get_rendering_metrics(&self) -> RenderingMetrics {
        self.rendering.read().await.clone()
    }

    /// Mark a performance marker
    pub async fn mark(&self, name: &str) -> Result<()> {
        let now = Utc::now();
        if let Some(start) = *self.start_time.read().await {
            let elapsed = (now - start).num_milliseconds() as f64;
            self.add_custom_metric(name, elapsed).await?;
        }
        Ok(())
    }

    /// Measure between two marks
    pub async fn measure(&self, name: &str, start_mark: &str, end_mark: &str) -> Result<f64> {
        let metrics = self.custom_metrics.read().await;
        
        let start = metrics.get(start_mark)
            .ok_or_else(|| ProfilingError::ProfilingFailed(format!("Start mark not found: {}", start_mark)))?;
        let end = metrics.get(end_mark)
            .ok_or_else(|| ProfilingError::ProfilingFailed(format!("End mark not found: {}", end_mark)))?;
        
        let duration = end - start;
        drop(metrics);
        
        self.add_custom_metric(name, duration).await?;
        Ok(duration)
    }

    /// Get Navigation Timing API data
    pub async fn get_navigation_timing(&self) -> NavigationTiming {
        let timings = self.timings.read().await;
        
        NavigationTiming {
            dom_content_loaded_event_start: timings.dom_content_loaded,
            load_event_start: timings.load_event,
            dom_complete: timings.total,
            response_start: timings.ttfb,
            response_end: timings.ttfb + timings.download_time,
            dom_interactive: timings.dom_parsing,
        }
    }

    /// Get resource timings
    pub async fn get_resource_timings(&self) -> Vec<ResourceTiming> {
        // In production, would track all loaded resources
        Vec::new()
    }
}

impl Default for PerformanceProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Timing event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimingEvent {
    DnsLookup,
    TcpConnection,
    TlsNegotiation,
    Ttfb,
    Download,
    DomParsing,
    DomContentLoaded,
    LoadEvent,
}

/// Rendering metric types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderingMetric {
    FirstPaint,
    FirstContentfulPaint,
    LargestContentfulPaint,
    SpeedIndex,
    TimeToInteractive,
    TotalBlockingTime,
    CumulativeLayoutShift,
}

/// Navigation timing data (matches Navigation Timing API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationTiming {
    pub dom_content_loaded_event_start: f64,
    pub load_event_start: f64,
    pub dom_complete: f64,
    pub response_start: f64,
    pub response_end: f64,
    pub dom_interactive: f64,
}

/// Resource timing data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTiming {
    pub name: String,
    pub entry_type: String,
    pub start_time: f64,
    pub duration: f64,
    pub initiator_type: String,
    pub transfer_size: u64,
    pub encoded_body_size: u64,
    pub decoded_body_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_profiler_creation() {
        let profiler = PerformanceProfiler::new();
        assert!(!*profiler.is_profiling.read().await);
    }

    #[tokio::test]
    async fn test_start_stop() {
        let profiler = PerformanceProfiler::new();
        
        profiler.start("https://example.com").await.unwrap();
        assert!(*profiler.is_profiling.read().await);
        
        let metrics = profiler.stop().await.unwrap();
        assert!(!metrics.is_empty());
    }

    #[tokio::test]
    async fn test_record_timing() {
        let profiler = PerformanceProfiler::new();
        
        profiler.record_timing(TimingEvent::DnsLookup, 50.0).await.unwrap();
        profiler.record_timing(TimingEvent::Ttfb, 200.0).await.unwrap();
        
        let timings = profiler.get_timings().await;
        assert_eq!(timings.dns_lookup, 50.0);
        assert_eq!(timings.ttfb, 200.0);
    }
}