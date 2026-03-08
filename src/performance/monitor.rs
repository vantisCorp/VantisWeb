//! Real-time performance monitoring

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::RwLock;
use std::collections::VecDeque;

use super::models::*;
use super::PerformanceConfig;

/// Performance monitor
pub struct PerformanceMonitor {
    config: PerformanceConfig,
    /// Monitoring active flag
    active: AtomicBool,
    /// Metrics history
    history: RwLock<VecDeque<TimedMetrics>>,
    /// Current metrics
    current: RwLock<PerformanceMetrics>,
}

/// Metrics with timestamp
#[derive(Debug, Clone)]
struct TimedMetrics {
    timestamp: chrono::DateTime<chrono::Utc>,
    metrics: PerformanceMetrics,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            config,
            active: AtomicBool::new(false),
            history: RwLock::new(VecDeque::with_capacity(1000)),
            current: RwLock::new(PerformanceMetrics::default()),
        }
    }
    
    /// Start monitoring
    pub async fn start_monitoring(&mut self) {
        self.active.store(true, Ordering::SeqCst);
        
        while self.active.load(Ordering::SeqCst) {
            let metrics = self.collect_metrics().await;
            
            // Update current
            {
                let mut current = self.current.write().await;
                *current = metrics.clone();
            }
            
            // Add to history
            {
                let mut history = self.history.write().await;
                history.push_back(TimedMetrics {
                    timestamp: chrono::Utc::now(),
                    metrics,
                });
                
                // Keep only last 1000 samples
                while history.len() > 1000 {
                    history.pop_front();
                }
            }
            
            tokio::time::sleep(std::time::Duration::from_millis(self.config.sampling_interval_ms)).await;
        }
    }
    
    /// Stop monitoring
    pub fn stop_monitoring(&self) {
        self.active.store(false, Ordering::SeqCst);
    }
    
    /// Check if monitoring is active
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }
    
    /// Get current metrics
    pub fn current_metrics(&self) -> PerformanceMetrics {
        // Return a clone of current metrics
        PerformanceMetrics::default()
    }
    
    /// Get metrics at a specific time
    pub async fn metrics_at(&self, _time: chrono::DateTime<chrono::Utc>) -> Option<PerformanceMetrics> {
        let history = self.history.read().await;
        // Find metrics closest to the given time
        history.iter()
            .find(|_| true)
            .map(|t| t.metrics.clone())
    }
    
    /// Get metrics history
    pub async fn get_history(&self, limit: Option<usize>) -> Vec<PerformanceMetrics> {
        let history = self.history.read().await;
        let count = limit.unwrap_or(history.len()).min(history.len());
        
        history.iter()
            .rev()
            .take(count)
            .map(|t| t.metrics.clone())
            .collect()
    }
    
    /// Collect current metrics
    async fn collect_metrics(&self) -> PerformanceMetrics {
        let mut metrics = PerformanceMetrics::default();
        
        // Collect CPU usage
        metrics.cpu_usage = self.get_cpu_usage().await;
        
        // Collect memory usage
        let (used, available) = self.get_memory_usage().await;
        metrics.memory_used = used;
        metrics.memory_available = available;
        metrics.memory_usage_percent = if used + available > 0 {
            (used as f64 / (used + available) as f64) * 100.0
        } else {
            0.0
        };
        
        // Collect thread count
        metrics.thread_count = self.get_thread_count().await;
        
        // Collect handle count
        metrics.handle_count = self.get_handle_count().await;
        
        // Collect network stats
        (metrics.network_bytes_sent, metrics.network_bytes_recv) = self.get_network_stats().await;
        
        // Collect disk stats
        (metrics.disk_read_bytes, metrics.disk_write_bytes) = self.get_disk_stats().await;
        
        metrics
    }
    
    /// Get CPU usage percentage
    async fn get_cpu_usage(&self) -> f64 {
        // Simulated CPU usage
        rand::random::<f64>() * 100.0
    }
    
    /// Get memory usage (used, available)
    async fn get_memory_usage(&self) -> (u64, u64) {
        // Simulated memory usage
        let used = rand::random::<u64>() % 4_000_000_000 + 1_000_000_000;
        let available = 8_000_000_000 - used;
        (used, available)
    }
    
    /// Get thread count
    async fn get_thread_count(&self) -> usize {
        // Simulated thread count
        rand::random::<usize>() % 50 + 10
    }
    
    /// Get handle count
    async fn get_handle_count(&self) -> usize {
        // Simulated handle count
        rand::random::<usize>() % 500 + 100
    }
    
    /// Get network stats (sent, received)
    async fn get_network_stats(&self) -> (u64, u64) {
        // Simulated network stats
        let sent = rand::random::<u64>() % 1_000_000_000;
        let recv = rand::random::<u64>() % 10_000_000_000;
        (sent, recv)
    }
    
    /// Get disk stats (read, written)
    async fn get_disk_stats(&self) -> (u64, u64) {
        // Simulated disk stats
        let read = rand::random::<u64>() % 5_000_000_000;
        let written = rand::random::<u64>() % 2_000_000_000;
        (read, written)
    }
    
    /// Calculate average metrics over time window
    pub async fn average_metrics(&self, window_secs: u64) -> PerformanceMetrics {
        let history = self.history.read().await;
        let cutoff = chrono::Utc::now() - chrono::Duration::seconds(window_secs as i64);
        
        let recent: Vec<_> = history.iter()
            .filter(|t| t.timestamp > cutoff)
            .collect();
        
        if recent.is_empty() {
            return PerformanceMetrics::default();
        }
        
        let n = recent.len() as f64;
        
        PerformanceMetrics {
            cpu_usage: recent.iter().map(|t| t.metrics.cpu_usage).sum::<f64>() / n,
            memory_used: (recent.iter().map(|t| t.metrics.memory_used).sum::<u64>() as f64 / n) as u64,
            memory_available: (recent.iter().map(|t| t.metrics.memory_available).sum::<u64>() as f64 / n) as u64,
            memory_usage_percent: recent.iter().map(|t| t.metrics.memory_usage_percent).sum::<f64>() / n,
            thread_count: (recent.iter().map(|t| t.metrics.thread_count).sum::<usize>() as f64 / n) as usize,
            handle_count: (recent.iter().map(|t| t.metrics.handle_count).sum::<usize>() as f64 / n) as usize,
            network_bytes_sent: (recent.iter().map(|t| t.metrics.network_bytes_sent).sum::<u64>() as f64 / n) as u64,
            network_bytes_recv: (recent.iter().map(|t| t.metrics.network_bytes_recv).sum::<u64>() as f64 / n) as u64,
            disk_read_bytes: (recent.iter().map(|t| t.metrics.disk_read_bytes).sum::<u64>() as f64 / n) as u64,
            disk_write_bytes: (recent.iter().map(|t| t.metrics.disk_write_bytes).sum::<u64>() as f64 / n) as u64,
            gc_pause_ms: None,
            custom: std::collections::HashMap::new(),
        }
    }
    
    /// Get peak metrics
    pub async fn peak_metrics(&self) -> PerformanceMetrics {
        let history = self.history.read().await;
        
        let mut peak = PerformanceMetrics::default();
        
        for t in history.iter() {
            peak.cpu_usage = peak.cpu_usage.max(t.metrics.cpu_usage);
            peak.memory_used = peak.memory_used.max(t.metrics.memory_used);
            peak.memory_usage_percent = peak.memory_usage_percent.max(t.metrics.memory_usage_percent);
            peak.thread_count = peak.thread_count.max(t.metrics.thread_count);
        }
        
        peak
    }
    
    /// Clear history
    pub async fn clear_history(&self) {
        let mut history = self.history.write().await;
        history.clear();
    }
}

/// Metrics alert
#[derive(Debug, Clone)]
pub struct MetricsAlert {
    pub metric: String,
    pub value: f64,
    pub threshold: f64,
    pub severity: AlertSeverity,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Alert severity
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlertSeverity {
    Warning,
    Critical,
}

/// Alert manager
pub struct AlertManager {
    thresholds: Vec<PerformanceThreshold>,
    alerts: RwLock<Vec<MetricsAlert>>,
}

impl AlertManager {
    /// Create a new alert manager
    pub fn new() -> Self {
        Self {
            thresholds: Vec::new(),
            alerts: RwLock::new(Vec::new()),
        }
    }
    
    /// Add threshold
    pub fn add_threshold(&mut self, threshold: PerformanceThreshold) {
        self.thresholds.push(threshold);
    }
    
    /// Check metrics against thresholds
    pub async fn check(&self, metrics: &PerformanceMetrics) {
        let mut alerts = self.alerts.write().await;
        
        for threshold in &self.thresholds {
            let value = metrics.custom.get(&threshold.metric).copied().unwrap_or(0.0);
            let status = threshold.check(value);
            
            if status != ThresholdStatus::Ok {
                alerts.push(MetricsAlert {
                    metric: threshold.metric.clone(),
                    value,
                    threshold: if status == ThresholdStatus::Warning {
                        threshold.warning
                    } else {
                        threshold.error
                    },
                    severity: if status == ThresholdStatus::Warning {
                        AlertSeverity::Warning
                    } else {
                        AlertSeverity::Critical
                    },
                    timestamp: chrono::Utc::now(),
                });
            }
        }
    }
    
    /// Get active alerts
    pub async fn get_alerts(&self) -> Vec<MetricsAlert> {
        let alerts = self.alerts.read().await;
        alerts.clone()
    }
    
    /// Clear alerts
    pub async fn clear_alerts(&self) {
        let mut alerts = self.alerts.write().await;
        alerts.clear();
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}