//! Performance Load Testing Module
//! 
//! Comprehensive performance testing and benchmarking for VantisWeb Browser
//! 
//! # Features
//! - Load testing with configurable scenarios
//! - Stress testing capabilities
//! - Memory and CPU profiling
//! - Network throttling simulation
//! - Real-time performance monitoring
//! - Benchmark suite runner

pub mod models;
pub mod load_tester;
pub mod stress_tester;
pub mod benchmark;
pub mod monitor;
pub mod profiler;
pub mod report;

use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

pub use models::*;
pub use load_tester::LoadTester;
pub use stress_tester::StressTester;
pub use benchmark::{BenchmarkRunner, BenchmarkSuite};
pub use monitor::PerformanceMonitor;
pub use profiler::{Profiler, ProfileResult};
pub use report::{ReportGenerator, PerformanceReport};

/// Performance Testing Manager
pub struct PerformanceManager {
    /// Load tester
    load_tester: Arc<LoadTester>,
    /// Stress tester
    stress_tester: Arc<StressTester>,
    /// Benchmark runner
    benchmark_runner: Arc<BenchmarkRunner>,
    /// Performance monitor
    monitor: Arc<RwLock<PerformanceMonitor>>,
    /// Profiler
    profiler: Arc<Profiler>,
    /// Configuration
    config: PerformanceConfig,
}

/// Performance testing configuration
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Maximum concurrent users for load testing
    pub max_concurrent_users: usize,
    /// Test duration in seconds
    pub default_duration_secs: u64,
    /// Ramp-up time in seconds
    pub ramp_up_secs: u64,
    /// Sampling interval in milliseconds
    pub sampling_interval_ms: u64,
    /// Enable memory profiling
    pub profile_memory: bool,
    /// Enable CPU profiling
    pub profile_cpu: bool,
    /// Output directory for reports
    pub output_dir: std::path::PathBuf,
    /// Threshold for warning (ms)
    pub warning_threshold_ms: f64,
    /// Threshold for error (ms)
    pub error_threshold_ms: f64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_concurrent_users: 1000,
            default_duration_secs: 60,
            ramp_up_secs: 10,
            sampling_interval_ms: 100,
            profile_memory: true,
            profile_cpu: true,
            output_dir: std::path::PathBuf::from("./performance-results"),
            warning_threshold_ms: 100.0,
            error_threshold_ms: 500.0,
        }
    }
}

impl PerformanceManager {
    /// Create a new performance manager
    pub fn new(config: PerformanceConfig) -> Self {
        let load_tester = Arc::new(LoadTester::new(config.clone()));
        let stress_tester = Arc::new(StressTester::new(config.clone()));
        let benchmark_runner = Arc::new(BenchmarkRunner::new(config.clone()));
        let monitor = Arc::new(RwLock::new(PerformanceMonitor::new(config.clone())));
        let profiler = Arc::new(Profiler::new(config.clone()));
        
        Self {
            load_tester,
            stress_tester,
            benchmark_runner,
            monitor,
            profiler,
            config,
        }
    }
    
    /// Run a load test
    pub async fn run_load_test(&self, scenario: LoadScenario) -> Result<LoadTestResult, PerfError> {
        // Start monitoring
        let monitor = self.monitor.clone();
        let monitor_handle = tokio::spawn(async move {
            let mut m = monitor.write().await;
            m.start_monitoring().await
        });
        
        // Run load test
        let result = self.load_tester.run(scenario).await?;
        
        // Stop monitoring
        monitor_handle.await.ok();
        
        Ok(result)
    }
    
    /// Run a stress test
    pub async fn run_stress_test(&self, scenario: StressScenario) -> Result<StressTestResult, PerfError> {
        self.stress_tester.run(scenario).await
    }
    
    /// Run benchmarks
    pub async fn run_benchmarks(&self, suite: BenchmarkSuite) -> Result<BenchmarkResult, PerfError> {
        self.benchmark_runner.run_suite(suite).await
    }
    
    /// Profile an operation
    pub async fn profile<F, T>(&self, name: &str, operation: F) -> Result<ProfileResult, PerfError>
    where
        F: std::future::Future<Output = T> + Send,
        T: Send,
    {
        self.profiler.profile(name, operation).await
    }
    
    /// Get current performance metrics
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        let monitor = self.monitor.read().await;
        monitor.current_metrics()
    }
    
    /// Generate performance report
    pub async fn generate_report(&self, results: &[TestExecution]) -> Result<std::path::PathBuf, PerfError> {
        let generator = ReportGenerator::new(self.config.output_dir.clone());
        generator.generate(results).await
    }
    
    /// Get the load tester
    pub fn load_tester(&self) -> &LoadTester {
        &self.load_tester
    }
    
    /// Get the stress tester
    pub fn stress_tester(&self) -> &StressTester {
        &self.stress_tester
    }
    
    /// Get the benchmark runner
    pub fn benchmark_runner(&self) -> &BenchmarkRunner {
        &self.benchmark_runner
    }
    
    /// Get the monitor
    pub async fn monitor(&self) -> tokio::sync::RwLockReadGuard<'_, PerformanceMonitor> {
        self.monitor.read().await
    }
}

/// Test execution record
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestExecution {
    pub test_id: String,
    pub test_type: TestType,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub duration_ms: u64,
    pub status: TestStatus,
    pub metrics: PerformanceMetrics,
    pub errors: Vec<String>,
}

/// Test types
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum TestType {
    LoadTest,
    StressTest,
    Benchmark,
    Profile,
}

/// Test status
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TestStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}