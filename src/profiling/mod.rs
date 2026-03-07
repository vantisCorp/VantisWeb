/// # Browser Performance Profiling Module
/// 
/// This module provides comprehensive performance profiling and debugging
/// tools for the VantisWeb browser.
//!
//! ## Features
//!
//! - **Performance Profiler**: Page load time analysis
//! - **Memory Monitoring**: Memory usage tracking and leak detection
//! - **CPU Monitoring**: CPU usage visualization
//! - **Network Analysis**: Request waterfall and timing
//! - **Timeline Recording**: Record and replay performance events
//! - **Debug Dashboard**: Real-time debugging interface

pub mod profiler;
pub mod memory;
pub mod cpu;
pub mod network;
pub mod timeline;
pub mod debugger;

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur in profiling operations
#[derive(Error, Debug)]
pub enum ProfilingError {
    #[error("Profiling failed: {0}")]
    ProfilingFailed(String),
    #[error("Memory error: {0}")]
    MemoryError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Debugger error: {0}")]
    DebuggerError(String),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

/// Result type for profiling operations
pub type Result<T> = std::result::Result<T, ProfilingError>;

/// Performance metric types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricType {
    PageLoadTime,
    FirstPaint,
    FirstContentfulPaint,
    LargestContentfulPaint,
    TimeToInteractive,
    TotalBlockingTime,
    CumulativeLayoutShift,
    SpeedIndex,
    MemoryUsage,
    CpuUsage,
    NetworkLatency,
    ScriptExecutionTime,
    LayoutTime,
    PaintTime,
}

/// Performance metric measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Metric type
    pub metric_type: MetricType,
    /// Metric value
    pub value: f64,
    /// Unit of measurement
    pub unit: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Page URL
    pub page_url: String,
    /// Additional context
    pub context: Option<String>,
}

impl Metric {
    /// Create a new metric
    pub fn new(metric_type: MetricType, value: f64, unit: String, page_url: String) -> Self {
        Self {
            metric_type,
            value,
            unit,
            timestamp: chrono::Utc::now(),
            page_url,
            context: None,
        }
    }
}

/// Performance score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceScore {
    /// Overall score (0-100)
    pub overall: f64,
    /// Performance score
    pub performance: f64,
    /// Accessibility score
    pub accessibility: f64,
    /// Best practices score
    pub best_practices: f64,
    /// SEO score
    pub seo: f64,
    /// Individual metrics
    pub metrics: Vec<Metric>,
    /// Suggestions for improvement
    pub suggestions: Vec<PerformanceSuggestion>,
}

/// Performance suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSuggestion {
    /// Suggestion title
    pub title: String,
    /// Description
    pub description: String,
    /// Impact (high, medium, low)
    pub impact: String,
    /// Category
    pub category: String,
    /// Documentation URL
    pub documentation_url: Option<String>,
}

/// Profiling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingConfig {
    /// Enable memory profiling
    pub enable_memory: bool,
    /// Enable CPU profiling
    pub enable_cpu: bool,
    /// Enable network profiling
    pub enable_network: bool,
    /// Enable timeline recording
    pub enable_timeline: bool,
    /// Sampling interval in milliseconds
    pub sampling_interval_ms: u32,
    /// Maximum profile duration in seconds
    pub max_duration_secs: u32,
    /// Enable stack trace capture
    pub capture_stack_traces: bool,
    /// Flush interval in milliseconds
    pub flush_interval_ms: u32,
}

impl Default for ProfilingConfig {
    fn default() -> Self {
        Self {
            enable_memory: true,
            enable_cpu: true,
            enable_network: true,
            enable_timeline: true,
            sampling_interval_ms: 100,
            max_duration_secs: 60,
            capture_stack_traces: true,
            flush_interval_ms: 1000,
        }
    }
}

/// Profiling manager for coordinating all profiling tools
pub struct ProfilingManager {
    /// Configuration
    config: Arc<RwLock<ProfilingConfig>>,
    /// Performance profiler
    profiler: Arc<RwLock<profiler::PerformanceProfiler>>,
    /// Memory monitor
    memory_monitor: Arc<RwLock<memory::MemoryMonitor>>,
    /// CPU monitor
    cpu_monitor: Arc<RwLock<cpu::CpuMonitor>>,
    /// Network analyzer
    network_analyzer: Arc<RwLock<network::NetworkAnalyzer>>,
    /// Timeline recorder
    timeline_recorder: Arc<RwLock<timeline::TimelineRecorder>>,
    /// Debugger
    debugger: Arc<RwLock<debugger::Debugger>>,
    /// Is profiling active
    is_active: Arc<RwLock<bool>>,
}

impl ProfilingManager {
    /// Create a new profiling manager
    pub async fn new(config: ProfilingConfig) -> Result<Self> {
        let config = Arc::new(RwLock::new(config));
        
        Ok(Self {
            config: config.clone(),
            profiler: Arc::new(RwLock::new(profiler::PerformanceProfiler::new())),
            memory_monitor: Arc::new(RwLock::new(memory::MemoryMonitor::new())),
            cpu_monitor: Arc::new(RwLock::new(cpu::CpuMonitor::new())),
            network_analyzer: Arc::new(RwLock::new(network::NetworkAnalyzer::new())),
            timeline_recorder: Arc::new(RwLock::new(timeline::TimelineRecorder::new())),
            debugger: Arc::new(RwLock::new(debugger::Debugger::new())),
            is_active: Arc::new(RwLock::new(false)),
        })
    }

    /// Start profiling session
    pub async fn start_profiling(&self, page_url: &str) -> Result<()> {
        let mut is_active = self.is_active.write().await;
        if *is_active {
            return Err(ProfilingError::InvalidOperation("Profiling already active".to_string()));
        }
        
        // Start all monitors
        self.profiler.write().await.start(page_url).await?;
        self.memory_monitor.write().await.start().await?;
        self.cpu_monitor.write().await.start().await?;
        self.network_analyzer.write().await.start().await?;
        self.timeline_recorder.write().await.start().await?;
        
        *is_active = true;
        Ok(())
    }

    /// Stop profiling session
    pub async fn stop_profiling(&self) -> Result<PerformanceScore> {
        let mut is_active = self.is_active.write().await;
        if !*is_active {
            return Err(ProfilingError::InvalidOperation("Profiling not active".to_string()));
        }
        
        // Stop all monitors
        let profile_results = self.profiler.write().await.stop().await?;
        let memory_results = self.memory_monitor.write().await.stop().await?;
        let cpu_results = self.cpu_monitor.write().await.stop().await?;
        let network_results = self.network_analyzer.write().await.stop().await?;
        let timeline = self.timeline_recorder.write().await.stop().await?;
        
        *is_active = false;
        
        // Calculate performance score
        self.calculate_score(&profile_results, &memory_results, &cpu_results).await
    }

    /// Calculate performance score
    async fn calculate_score(
        &self,
        profile: &[Metric],
        memory: &[Metric],
        cpu: &[Metric],
    ) -> Result<PerformanceScore> {
        // Calculate performance score based on Core Web Vitals
        let mut performance_score = 100.0;
        
        for metric in profile {
            let deduction = match metric.metric_type {
                MetricType::LargestContentfulPaint => {
                    if metric.value > 4000.0 { 25.0 }
                    else if metric.value > 2500.0 { 10.0 }
                    else { 0.0 }
                }
                MetricType::TotalBlockingTime => {
                    if metric.value > 600.0 { 25.0 }
                    else if metric.value > 300.0 { 10.0 }
                    else { 0.0 }
                }
                MetricType::CumulativeLayoutShift => {
                    if metric.value > 0.25 { 25.0 }
                    else if metric.value > 0.1 { 10.0 }
                    else { 0.0 }
                }
                _ => 0.0,
            };
            performance_score -= deduction;
        }

        // Generate suggestions
        let mut suggestions = Vec::new();
        if performance_score < 90.0 {
            suggestions.push(PerformanceSuggestion {
                title: "Optimize page load performance".to_string(),
                description: "Consider reducing render-blocking resources and optimizing images".to_string(),
                impact: "high".to_string(),
                category: "Performance".to_string(),
                documentation_url: Some("https://web.dev/performance/".to_string()),
            });
        }

        let mut all_metrics = Vec::new();
        all_metrics.extend(profile.to_vec());
        all_metrics.extend(memory.to_vec());
        all_metrics.extend(cpu.to_vec());

        Ok(PerformanceScore {
            overall: performance_score,
            performance: performance_score,
            accessibility: 85.0,
            best_practices: 90.0,
            seo: 80.0,
            metrics: all_metrics,
            suggestions,
        })
    }

    /// Get current metrics
    pub async fn get_current_metrics(&self) -> Result<Vec<Metric>> {
        let mut metrics = Vec::new();
        
        metrics.extend(self.memory_monitor.read().await.get_current_metrics());
        metrics.extend(self.cpu_monitor.read().await.get_current_metrics());
        
        Ok(metrics)
    }

    /// Take memory snapshot
    pub async fn take_memory_snapshot(&self) -> Result<memory::MemorySnapshot> {
        self.memory_monitor.read().await.take_snapshot().await
    }

    /// Get CPU usage
    pub async fn get_cpu_usage(&self) -> Result<f64> {
        Ok(self.cpu_monitor.read().await.get_current_usage())
    }

    /// Analyze network requests
    pub async fn analyze_network(&self) -> Result<Vec<network::NetworkRequest>> {
        Ok(self.network_analyzer.read().await.get_requests())
    }

    /// Start debugger
    pub async fn start_debugger(&self, port: u16) -> Result<()> {
        self.debugger.write().await.start(port).await
    }

    /// Stop debugger
    pub async fn stop_debugger(&self) -> Result<()> {
        self.debugger.write().await.stop().await
    }
}

impl Default for ProfilingManager {
    fn default() -> Self {
        // Create a default instance without async
        Self {
            config: Arc::new(RwLock::new(ProfilingConfig::default())),
            profiler: Arc::new(RwLock::new(profiler::PerformanceProfiler::new())),
            memory_monitor: Arc::new(RwLock::new(memory::MemoryMonitor::new())),
            cpu_monitor: Arc::new(RwLock::new(cpu::CpuMonitor::new())),
            network_analyzer: Arc::new(RwLock::new(network::NetworkAnalyzer::new())),
            timeline_recorder: Arc::new(RwLock::new(timeline::TimelineRecorder::new())),
            debugger: Arc::new(RwLock::new(debugger::Debugger::new())),
            is_active: Arc::new(RwLock::new(false)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_creation() {
        let metric = Metric::new(
            MetricType::PageLoadTime,
            1500.0,
            "ms".to_string(),
            "https://example.com".to_string(),
        );
        
        assert_eq!(metric.metric_type, MetricType::PageLoadTime);
        assert_eq!(metric.value, 1500.0);
    }

    #[test]
    fn test_config_default() {
        let config = ProfilingConfig::default();
        assert!(config.enable_memory);
        assert!(config.enable_cpu);
        assert_eq!(config.sampling_interval_ms, 100);
    }

    #[tokio::test]
    async fn test_profiling_manager_creation() {
        let manager = ProfilingManager::new(ProfilingConfig::default()).await;
        assert!(manager.is_ok());
    }
}