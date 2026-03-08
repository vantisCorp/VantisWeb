//! Data models for performance testing

use std::path::PathBuf;
use serde::{Serialize, Deserialize};

/// Load test scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadScenario {
    /// Scenario name
    pub name: String,
    /// Target URL or operation
    pub target: String,
    /// Number of virtual users
    pub virtual_users: usize,
    /// Test duration in seconds
    pub duration_secs: u64,
    /// Ramp-up time in seconds
    pub ramp_up_secs: u64,
    /// Request distribution
    pub distribution: RequestDistribution,
    /// Think time between requests (ms)
    pub think_time_ms: Option<u64>,
    /// Max requests per second
    pub max_rps: Option<u32>,
}

impl LoadScenario {
    /// Create a new load scenario
    pub fn new(name: &str, target: &str, virtual_users: usize) -> Self {
        Self {
            name: name.to_string(),
            target: target.to_string(),
            virtual_users,
            duration_secs: 60,
            ramp_up_secs: 10,
            distribution: RequestDistribution::Uniform,
            think_time_ms: Some(100),
            max_rps: None,
        }
    }
    
    /// Set duration
    pub fn with_duration(mut self, secs: u64) -> Self {
        self.duration_secs = secs;
        self
    }
    
    /// Set ramp-up
    pub fn with_ramp_up(mut self, secs: u64) -> Self {
        self.ramp_up_secs = secs;
        self
    }
}

/// Request distribution pattern
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RequestDistribution {
    /// Uniform distribution
    Uniform,
    /// Normal/Gaussian distribution
    Normal { mean: f64, std_dev: f64 },
    /// Poisson distribution
    Poisson { lambda: f64 },
    /// Custom intervals
    Custom,
}

/// Load test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadTestResult {
    /// Scenario name
    pub scenario_name: String,
    /// Total requests made
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Average response time (ms)
    pub avg_response_time_ms: f64,
    /// Min response time (ms)
    pub min_response_time_ms: f64,
    /// Max response time (ms)
    pub max_response_time_ms: f64,
    /// P50 response time (ms)
    pub p50_response_time_ms: f64,
    /// P90 response time (ms)
    pub p90_response_time_ms: f64,
    /// P95 response time (ms)
    pub p95_response_time_ms: f64,
    /// P99 response time (ms)
    pub p99_response_time_ms: f64,
    /// Requests per second
    pub requests_per_second: f64,
    /// Total data transferred (bytes)
    pub total_bytes: u64,
    /// Error rate
    pub error_rate: f64,
    /// Test duration (ms)
    pub duration_ms: u64,
    /// Timeline samples
    pub timeline: Vec<TimelineSample>,
}

/// Timeline sample for time-series data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineSample {
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Active users at this point
    pub active_users: usize,
    /// Response time at this point (ms)
    pub response_time_ms: f64,
    /// Requests per second
    pub rps: f64,
    /// Error rate
    pub error_rate: f64,
}

/// Stress test scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressScenario {
    /// Scenario name
    pub name: String,
    /// Target operation
    pub target: String,
    /// Starting load
    pub start_load: usize,
    /// Maximum load
    pub max_load: usize,
    /// Load increment step
    pub load_step: usize,
    /// Duration at each step (seconds)
    pub step_duration_secs: u64,
    /// Failure threshold (error rate)
    pub failure_threshold: f64,
}

impl StressScenario {
    /// Create a new stress scenario
    pub fn new(name: &str, target: &str) -> Self {
        Self {
            name: name.to_string(),
            target: target.to_string(),
            start_load: 10,
            max_load: 1000,
            load_step: 50,
            step_duration_secs: 30,
            failure_threshold: 0.1,
        }
    }
    
    /// Find breaking point
    pub fn breaking_point(&self) -> bool {
        true
    }
}

/// Stress test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTestResult {
    /// Scenario name
    pub scenario_name: String,
    /// Breaking point (VUs)
    pub breaking_point: Option<usize>,
    /// Steps completed
    pub steps_completed: usize,
    /// Results at each step
    pub step_results: Vec<StepResult>,
    /// Maximum sustained load
    pub max_sustained_load: usize,
    /// Total duration (ms)
    pub duration_ms: u64,
}

/// Result at a stress test step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// Load level (VUs)
    pub load_level: usize,
    /// Average response time (ms)
    pub avg_response_time_ms: f64,
    /// Error rate
    pub error_rate: f64,
    /// Throughput (RPS)
    pub throughput: f64,
    /// Whether this step passed threshold
    pub passed: bool,
}

/// Performance metrics snapshot
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage (bytes)
    pub memory_used: u64,
    /// Memory available (bytes)
    pub memory_available: u64,
    /// Memory usage percentage
    pub memory_usage_percent: f64,
    /// Number of threads
    pub thread_count: usize,
    /// Handle/file descriptor count
    pub handle_count: usize,
    /// Network bytes sent
    pub network_bytes_sent: u64,
    /// Network bytes received
    pub network_bytes_recv: u64,
    /// Disk reads (bytes)
    pub disk_read_bytes: u64,
    /// Disk writes (bytes)
    pub disk_write_bytes: u64,
    /// GC pause time (ms)
    pub gc_pause_ms: Option<f64>,
    /// Custom metrics
    pub custom: std::collections::HashMap<String, f64>,
}

impl PerformanceMetrics {
    /// Calculate memory pressure
    pub fn memory_pressure(&self) -> MemoryPressure {
        match self.memory_usage_percent {
            p if p > 90.0 => MemoryPressure::Critical,
            p if p > 75.0 => MemoryPressure::High,
            p if p > 50.0 => MemoryPressure::Medium,
            _ => MemoryPressure::Low,
        }
    }
}

/// Memory pressure levels
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MemoryPressure {
    Low,
    Medium,
    High,
    Critical,
}

/// Benchmark definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Benchmark {
    /// Benchmark name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Number of iterations
    pub iterations: usize,
    /// Warm-up iterations
    pub warmup_iterations: usize,
    /// Measurement type
    pub measurement: MeasurementType,
    /// Tags
    pub tags: Vec<String>,
}

impl Benchmark {
    /// Create a new benchmark
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            iterations: 1000,
            warmup_iterations: 100,
            measurement: MeasurementType::Time,
            tags: Vec::new(),
        }
    }
}

/// Measurement types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MeasurementType {
    /// Wall-clock time
    Time,
    /// CPU cycles
    CpuCycles,
    /// Instructions retired
    Instructions,
    /// Cache misses
    CacheMisses,
    /// Memory allocated
    MemoryAllocated,
    /// Throughput
    Throughput,
}

/// Benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Suite name
    pub suite_name: String,
    /// Individual benchmark results
    pub benchmarks: Vec<SingleBenchmarkResult>,
    /// Total duration (ms)
    pub total_duration_ms: u64,
    /// System info
    pub system_info: SystemInfo,
}

/// Single benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleBenchmarkResult {
    /// Benchmark name
    pub name: String,
    /// Mean time (ns)
    pub mean_ns: f64,
    /// Standard deviation (ns)
    pub std_dev_ns: f64,
    /// Min time (ns)
    pub min_ns: f64,
    /// Max time (ns)
    pub max_ns: f64,
    /// Median (ns)
    pub median_ns: f64,
    /// Percentiles
    pub percentiles: Percentiles,
    /// Iterations run
    pub iterations: usize,
    /// Throughput (ops/sec)
    pub throughput: f64,
    /// Memory used (bytes)
    pub memory_used: Option<u64>,
}

/// Percentile values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Percentiles {
    pub p5: f64,
    pub p10: f64,
    pub p25: f64,
    pub p50: f64,
    pub p75: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
}

/// System information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub cpu: String,
    pub cpu_cores: usize,
    pub memory_total: u64,
    pub rust_version: String,
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            cpu: "Unknown".to_string(),
            cpu_cores: num_cpus::get(),
            memory_total: 0,
            rust_version: env!("RUST_VERSION").to_string(),
        }
    }
}

/// Network condition simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkCondition {
    /// Latency in ms
    pub latency_ms: u64,
    /// Jitter in ms
    pub jitter_ms: u64,
    /// Packet loss percentage
    pub packet_loss_percent: f64,
    /// Bandwidth limit (kbps)
    pub bandwidth_kbps: Option<u32>,
}

impl NetworkCondition {
    /// Fast 4G
    pub fn fast_4g() -> Self {
        Self {
            latency_ms: 20,
            jitter_ms: 5,
            packet_loss_percent: 0.0,
            bandwidth_kbps: Some(10000),
        }
    }
    
    /// Slow 3G
    pub fn slow_3g() -> Self {
        Self {
            latency_ms: 300,
            jitter_ms: 100,
            packet_loss_percent: 0.1,
            bandwidth_kbps: Some(400),
        }
    }
    
    /// Offline
    pub fn offline() -> Self {
        Self {
            latency_ms: u64::MAX,
            jitter_ms: 0,
            packet_loss_percent: 100.0,
            bandwidth_kbps: Some(0),
        }
    }
}

/// Performance error
#[derive(Debug, thiserror::Error)]
pub enum PerfError {
    #[error("Load test failed: {0}")]
    LoadTestFailed(String),
    
    #[error("Stress test failed: {0}")]
    StressTestFailed(String),
    
    #[error("Benchmark failed: {0}")]
    BenchmarkFailed(String),
    
    #[error("Profiling failed: {0}")]
    ProfilingFailed(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Timeout exceeded")]
    Timeout,
    
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Performance threshold
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThreshold {
    /// Metric name
    pub metric: String,
    /// Warning threshold
    pub warning: f64,
    /// Error threshold
    pub error: f64,
    /// Comparison operator
    pub operator: ComparisonOperator,
}

/// Comparison operators
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ComparisonOperator {
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Equal,
}

impl PerformanceThreshold {
    /// Check if value exceeds threshold
    pub fn check(&self, value: f64) -> ThresholdStatus {
        let exceeds_warning = match self.operator {
            ComparisonOperator::LessThan => value >= self.warning,
            ComparisonOperator::LessThanOrEqual => value > self.warning,
            ComparisonOperator::GreaterThan => value <= self.warning,
            ComparisonOperator::GreaterThanOrEqual => value < self.warning,
            ComparisonOperator::Equal => (value - self.warning).abs() > f64::EPSILON,
        };
        
        let exceeds_error = match self.operator {
            ComparisonOperator::LessThan => value >= self.error,
            ComparisonOperator::LessThanOrEqual => value > self.error,
            ComparisonOperator::GreaterThan => value <= self.error,
            ComparisonOperator::GreaterThanOrEqual => value < self.error,
            ComparisonOperator::Equal => (value - self.error).abs() > f64::EPSILON,
        };
        
        if exceeds_error {
            ThresholdStatus::Error
        } else if exceeds_warning {
            ThresholdStatus::Warning
        } else {
            ThresholdStatus::Ok
        }
    }
}

/// Threshold status
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ThresholdStatus {
    Ok,
    Warning,
    Error,
}