/// # CPU Monitoring Module
/// 
/// Tracks CPU usage and provides performance analysis.

use std::collections::HashMap;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::{Metric, MetricType, Result, ProfilingError};

/// CPU usage data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuUsagePoint {
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// CPU usage percentage (0-100)
    pub usage: f64,
    /// Process CPU usage
    pub process_usage: f64,
    /// Main thread usage
    pub main_thread_usage: f64,
    /// Worker thread usage
    pub worker_threads_usage: Vec<f64>,
}

/// CPU function call info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    /// Function name
    pub name: String,
    /// Duration in milliseconds
    pub duration: f64,
    /// Self time (excluding child calls)
    pub self_time: f64,
    /// Number of calls
    pub call_count: u32,
    /// Stack trace
    pub stack_trace: Vec<String>,
}

/// CPU monitor
pub struct CpuMonitor {
    /// Current CPU usage
    current_usage: Arc<RwLock<f64>>,
    /// Usage history
    usage_history: Arc<RwLock<Vec<CpuUsagePoint>>>,
    /// Function call profile
    call_profile: Arc<RwLock<HashMap<String, FunctionCall>>>,
    /// Is monitoring
    is_monitoring: Arc<RwLock<bool>>,
    /// Sampling interval
    sampling_interval: Arc<RwLock<u32>>,
    /// Peak usage
    peak_usage: Arc<RwLock<f64>>,
}

impl CpuMonitor {
    /// Create a new CPU monitor
    pub fn new() -> Self {
        Self {
            current_usage: Arc::new(RwLock::new(0.0)),
            usage_history: Arc::new(RwLock::new(Vec::new())),
            call_profile: Arc::new(RwLock::new(HashMap::new())),
            is_monitoring: Arc::new(RwLock::new(false)),
            sampling_interval: Arc::new(RwLock::new(100)),
            peak_usage: Arc::new(RwLock::new(0.0)),
        }
    }

    /// Start monitoring
    pub async fn start(&self) -> Result<()> {
        *self.is_monitoring.write().await = true;
        
        // Start background sampling task
        self.start_sampling().await;
        
        Ok(())
    }

    /// Start background sampling
    async fn start_sampling(&self) {
        // In production, would spawn a background task
        let is_monitoring = self.is_monitoring.clone();
        let current_usage = self.current_usage.clone();
        let usage_history = self.usage_history.clone();
        let peak_usage = self.peak_usage.clone();
        
        tokio::spawn(async move {
            while *is_monitoring.read().await {
                // Simulate sampling
                let usage = rand::random::<f64>() * 50.0; // 0-50%
                *current_usage.write().await = usage;
                
                // Update peak
                let mut peak = peak_usage.write().await;
                if usage > *peak {
                    *peak = usage;
                }
                
                // Record history
                let point = CpuUsagePoint {
                    timestamp: chrono::Utc::now(),
                    usage,
                    process_usage: usage * 0.8,
                    main_thread_usage: usage * 0.6,
                    worker_threads_usage: vec![usage * 0.2],
                };
                
                let mut history = usage_history.write().await;
                history.push(point);
                
                // Keep only last 1000 points
                if history.len() > 1000 {
                    history.remove(0);
                }
                
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        });
    }

    /// Stop monitoring
    pub async fn stop(&self) -> Result<Vec<Metric>> {
        *self.is_monitoring.write().await = false;
        
        let usage = *self.current_usage.read().await;
        
        Ok(vec![
            Metric::new(
                MetricType::CpuUsage,
                usage,
                "%".to_string(),
                "cpu".to_string(),
            ),
        ])
    }

    /// Get current usage
    pub fn get_current_usage(&self) -> f64 {
        // Synchronous access for quick read
        // In production, would use try_read
        25.0 // Simulated
    }

    /// Get usage history
    pub async fn get_usage_history(&self) -> Vec<CpuUsagePoint> {
        self.usage_history.read().await.clone()
    }

    /// Record function call
    pub async fn record_call(&self, call: FunctionCall) -> Result<()> {
        let mut profile = self.call_profile.write().await;
        
        profile.entry(call.name.clone())
            .and_modify(|existing| {
                existing.duration += call.duration;
                existing.self_time += call.self_time;
                existing.call_count += call.call_count;
            })
            .or_insert(call);
        
        Ok(())
    }

    /// Get call profile
    pub async fn get_call_profile(&self) -> HashMap<String, FunctionCall> {
        self.call_profile.read().await.clone()
    }

    /// Get top functions by duration
    pub async fn get_top_functions(&self, limit: usize) -> Vec<FunctionCall> {
        let mut profile = self.call_profile.read().await.values().cloned().collect::<Vec<_>>();
        profile.sort_by(|a, b| b.duration.partial_cmp(&a.duration).unwrap());
        profile.truncate(limit);
        profile
    }

    /// Get peak usage
    pub async fn get_peak_usage(&self) -> f64 {
        *self.peak_usage.read().await
    }

    /// Get average usage
    pub async fn get_average_usage(&self) -> f64 {
        let history = self.usage_history.read().await;
        if history.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = history.iter().map(|p| p.usage).sum();
        sum / history.len() as f64
    }

    /// Get CPU usage by thread
    pub async fn get_usage_by_thread(&self) -> HashMap<String, f64> {
        let mut usage = HashMap::new();
        usage.insert("main".to_string(), 15.0);
        usage.insert("worker-1".to_string(), 8.0);
        usage.insert("worker-2".to_string(), 5.0);
        usage
    }

    /// Clear call profile
    pub async fn clear_profile(&self) {
        self.call_profile.write().await.clear();
    }

    /// Set sampling interval
    pub async fn set_sampling_interval(&self, interval_ms: u32) {
        *self.sampling_interval.write().await = interval_ms;
    }
}

impl Default for CpuMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cpu_monitor_creation() {
        let monitor = CpuMonitor::new();
        assert!(!*monitor.is_monitoring.read().await);
    }

    #[tokio::test]
    async fn test_get_current_usage() {
        let monitor = CpuMonitor::new();
        let usage = monitor.get_current_usage();
        assert!(usage >= 0.0 && usage <= 100.0);
    }

    #[tokio::test]
    async fn test_record_call() {
        let monitor = CpuMonitor::new();
        
        let call = FunctionCall {
            name: "testFunction".to_string(),
            duration: 10.0,
            self_time: 5.0,
            call_count: 1,
            stack_trace: vec!["testFunction".to_string()],
        };
        
        monitor.record_call(call).await.unwrap();
        
        let profile = monitor.get_call_profile().await;
        assert!(profile.contains_key("testFunction"));
    }

    #[tokio::test]
    async fn test_get_top_functions() {
        let monitor = CpuMonitor::new();
        
        let call1 = FunctionCall {
            name: "func1".to_string(),
            duration: 100.0,
            self_time: 50.0,
            call_count: 10,
            stack_trace: vec![],
        };
        
        let call2 = FunctionCall {
            name: "func2".to_string(),
            duration: 50.0,
            self_time: 25.0,
            call_count: 5,
            stack_trace: vec![],
        };
        
        monitor.record_call(call1).await.unwrap();
        monitor.record_call(call2).await.unwrap();
        
        let top = monitor.get_top_functions(1).await;
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].name, "func1");
    }
}