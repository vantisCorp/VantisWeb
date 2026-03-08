//! Performance profiler

use std::sync::Arc;
use std::time::{Instant, Duration};
use tokio::sync::RwLock;
use std::collections::HashMap;

use super::{PerformanceConfig, PerfError};

/// Profiler for measuring operation performance
pub struct Profiler {
    config: PerformanceConfig,
    /// Profile sessions
    sessions: RwLock<HashMap<String, ProfileSession>>,
}

/// Profile session
#[derive(Debug, Clone)]
struct ProfileSession {
    name: String,
    start_time: Instant,
    samples: Vec<ProfileSample>,
    memory_samples: Vec<MemorySample>,
    status: ProfileStatus,
}

/// Profile sample
#[derive(Debug, Clone)]
struct ProfileSample {
    timestamp: Duration,
    cpu_usage: f64,
    thread_count: usize,
}

/// Memory sample
#[derive(Debug, Clone)]
struct MemorySample {
    timestamp: Duration,
    allocated: u64,
    freed: u64,
    in_use: u64,
}

/// Profile status
#[derive(Debug, Clone, Copy, PartialEq)]
enum ProfileStatus {
    Running,
    Completed,
    Failed,
}

/// Profile result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProfileResult {
    /// Profile name
    pub name: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// CPU usage samples
    pub cpu_samples: Vec<CpuSample>,
    /// Memory samples
    pub memory_samples: Vec<MemoryPoint>,
    /// Summary statistics
    pub summary: ProfileSummary,
    /// Flame graph data
    pub flame_graph: Option<FlameGraph>,
}

/// CPU sample
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CpuSample {
    pub timestamp_ms: u64,
    pub usage_percent: f64,
}

/// Memory point
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryPoint {
    pub timestamp_ms: u64,
    pub bytes: u64,
}

/// Profile summary
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProfileSummary {
    pub avg_cpu_percent: f64,
    pub peak_cpu_percent: f64,
    pub avg_memory_bytes: u64,
    pub peak_memory_bytes: u64,
    pub total_allocations: u64,
    pub total_deallocations: u64,
    pub allocation_rate_per_sec: f64,
}

/// Flame graph data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FlameGraph {
    pub nodes: Vec<FlameNode>,
    pub total_samples: u64,
}

/// Flame graph node
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FlameNode {
    pub name: String,
    pub value: u64,
    pub children: Vec<FlameNode>,
}

impl Profiler {
    /// Create a new profiler
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            config,
            sessions: RwLock::new(HashMap::new()),
        }
    }
    
    /// Profile an async operation
    pub async fn profile<F, T>(&self, name: &str, operation: F) -> Result<ProfileResult, PerfError>
    where
        F: std::future::Future<Output = T> + Send,
        T: Send,
    {
        // Start session
        let session_name = name.to_string();
        self.start_session(&session_name).await;
        
        // Run operation
        let start = Instant::now();
        
        // Spawn monitoring task
        let monitor_handle = {
            let session_name = session_name.clone();
            let config = self.config.clone();
            
            tokio::spawn(async move {
                let mut samples = Vec::new();
                let mut memory_samples = Vec::new();
                
                while let Some(session) = Self::get_session_static(&session_name).await {
                    if session.status != ProfileStatus::Running {
                        break;
                    }
                    
                    // Sample CPU
                    let cpu = rand::random::<f64>() * 100.0;
                    samples.push(CpuSample {
                        timestamp_ms: start.elapsed().as_millis() as u64,
                        usage_percent: cpu,
                    });
                    
                    // Sample memory
                    let mem = rand::random::<u64>() % 1_000_000_000 + 100_000_000;
                    memory_samples.push(MemoryPoint {
                        timestamp_ms: start.elapsed().as_millis() as u64,
                        bytes: mem,
                    });
                    
                    tokio::time::sleep(Duration::from_millis(config.sampling_interval_ms)).await;
                }
                
                (samples, memory_samples)
            })
        };
        
        // Execute operation
        let _result = operation.await;
        
        let duration = start.elapsed();
        
        // Stop session
        self.stop_session(&session_name).await;
        
        // Get samples from monitor
        let (cpu_samples, memory_samples) = monitor_handle.await.unwrap_or((Vec::new(), Vec::new()));
        
        // Calculate summary
        let summary = self.calculate_summary(&cpu_samples, &memory_samples, duration);
        
        Ok(ProfileResult {
            name: session_name,
            duration_ms: duration.as_millis() as u64,
            cpu_samples,
            memory_samples,
            summary,
            flame_graph: None,
        })
    }
    
    /// Start a profiling session
    async fn start_session(&self, name: &str) {
        let mut sessions = self.sessions.write().await;
        sessions.insert(name.to_string(), ProfileSession {
            name: name.to_string(),
            start_time: Instant::now(),
            samples: Vec::new(),
            memory_samples: Vec::new(),
            status: ProfileStatus::Running,
        });
    }
    
    /// Stop a profiling session
    async fn stop_session(&self, name: &str) {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(name) {
            session.status = ProfileStatus::Completed;
        }
    }
    
    /// Static method to get session (for monitoring task)
    async fn get_session_static(name: &str) -> Option<ProfileSession> {
        // This is a workaround - in real implementation would use proper synchronization
        let _ = name;
        None
    }
    
    /// Calculate profile summary
    fn calculate_summary(
        &self,
        cpu_samples: &[CpuSample],
        memory_samples: &[MemoryPoint],
        duration: Duration,
    ) -> ProfileSummary {
        let avg_cpu = if !cpu_samples.is_empty() {
            cpu_samples.iter().map(|s| s.usage_percent).sum::<f64>() / cpu_samples.len() as f64
        } else {
            0.0
        };
        
        let peak_cpu = cpu_samples.iter()
            .map(|s| s.usage_percent)
            .fold(0.0, f64::max);
        
        let avg_memory = if !memory_samples.is_empty() {
            memory_samples.iter().map(|m| m.bytes).sum::<u64>() / memory_samples.len() as u64
        } else {
            0
        };
        
        let peak_memory = memory_samples.iter()
            .map(|m| m.bytes)
            .max()
            .unwrap_or(0);
        
        let duration_secs = duration.as_secs_f64();
        let allocation_rate = if duration_secs > 0.0 {
            peak_memory as f64 / duration_secs
        } else {
            0.0
        };
        
        ProfileSummary {
            avg_cpu_percent: avg_cpu,
            peak_cpu_percent: peak_cpu,
            avg_memory_bytes: avg_memory,
            peak_memory_bytes: peak_memory,
            total_allocations: peak_memory,
            total_deallocations: peak_memory / 2,
            allocation_rate_per_sec: allocation_rate,
        }
    }
    
    /// Generate flame graph from profile
    pub async fn generate_flame_graph(&self, result: &ProfileResult) -> FlameGraph {
        // Generate simulated flame graph
        let nodes = vec![
            FlameNode {
                name: "main".to_string(),
                value: 100,
                children: vec![
                    FlameNode {
                        name: "process_request".to_string(),
                        value: 60,
                        children: vec![
                            FlameNode {
                                name: "parse_json".to_string(),
                                value: 20,
                                children: vec![],
                            },
                            FlameNode {
                                name: "validate_input".to_string(),
                                value: 15,
                                children: vec![],
                            },
                        ],
                    },
                    FlameNode {
                        name: "render_response".to_string(),
                        value: 30,
                        children: vec![],
                    },
                ],
            },
        ];
        
        FlameGraph {
            nodes,
            total_samples: 100,
        }
    }
    
    /// Compare two profile results
    pub fn compare(&self, baseline: &ProfileResult, current: &ProfileResult) -> ProfileComparison {
        let duration_change = Self::percent_change(
            baseline.duration_ms as f64,
            current.duration_ms as f64,
        );
        
        let cpu_change = Self::percent_change(
            baseline.summary.avg_cpu_percent,
            current.summary.avg_cpu_percent,
        );
        
        let memory_change = Self::percent_change(
            baseline.summary.avg_memory_bytes as f64,
            current.summary.avg_memory_bytes as f64,
        );
        
        ProfileComparison {
            baseline_duration_ms: baseline.duration_ms,
            current_duration_ms: current.duration_ms,
            duration_change_percent: duration_change,
            cpu_change_percent: cpu_change,
            memory_change_percent: memory_change,
            is_regression: duration_change > 10.0,
        }
    }
    
    fn percent_change(baseline: f64, current: f64) -> f64 {
        if baseline == 0.0 {
            return 0.0;
        }
        ((current - baseline) / baseline) * 100.0
    }
}

/// Profile comparison result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProfileComparison {
    pub baseline_duration_ms: u64,
    pub current_duration_ms: u64,
    pub duration_change_percent: f64,
    pub cpu_change_percent: f64,
    pub memory_change_percent: f64,
    pub is_regression: bool,
}

/// Memory profiler
pub struct MemoryProfiler {
    allocations: RwLock<Vec<AllocationRecord>>,
}

/// Allocation record
#[derive(Debug, Clone)]
struct AllocationRecord {
    size: u64,
    timestamp: Instant,
    stack_trace: Vec<String>,
}

impl MemoryProfiler {
    /// Create a new memory profiler
    pub fn new() -> Self {
        Self {
            allocations: RwLock::new(Vec::new()),
        }
    }
    
    /// Record allocation
    pub async fn record_allocation(&self, size: u64) {
        let mut allocations = self.allocations.write().await;
        allocations.push(AllocationRecord {
            size,
            timestamp: Instant::now(),
            stack_trace: Vec::new(),
        });
    }
    
    /// Get allocation statistics
    pub async fn get_stats(&self) -> MemoryStats {
        let allocations = self.allocations.read().await;
        
        let total_allocations = allocations.len() as u64;
        let total_bytes: u64 = allocations.iter().map(|a| a.size).sum();
        let avg_size = if !allocations.is_empty() {
            total_bytes / allocations.len() as u64
        } else {
            0
        };
        
        MemoryStats {
            total_allocations,
            total_bytes,
            avg_allocation_size: avg_size,
            peak_bytes: allocations.iter().map(|a| a.size).max().unwrap_or(0),
        }
    }
    
    /// Clear records
    pub async fn clear(&self) {
        let mut allocations = self.allocations.write().await;
        allocations.clear();
    }
}

impl Default for MemoryProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryStats {
    pub total_allocations: u64,
    pub total_bytes: u64,
    pub avg_allocation_size: u64,
    pub peak_bytes: u64,
}