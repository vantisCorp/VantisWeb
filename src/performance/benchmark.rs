//! Benchmark runner

use std::sync::Arc;
use std::time::{Instant, Duration};
use tokio::sync::RwLock;

use super::models::*;
use super::{PerformanceConfig, PerfError};

/// Benchmark runner
pub struct BenchmarkRunner {
    config: PerformanceConfig,
    /// Registered benchmarks
    benchmarks: RwLock<Vec<Benchmark>>,
}

impl BenchmarkRunner {
    /// Create a new benchmark runner
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            config,
            benchmarks: RwLock::new(Vec::new()),
        }
    }
    
    /// Register a benchmark
    pub async fn register(&self, benchmark: Benchmark) {
        let mut benchmarks = self.benchmarks.write().await;
        benchmarks.push(benchmark);
    }
    
    /// Run a single benchmark
    pub async fn run<F>(&self, benchmark: &Benchmark, mut operation: F) -> Result<SingleBenchmarkResult, PerfError>
    where
        F: FnMut() -> (),
    {
        // Warmup phase
        for _ in 0..benchmark.warmup_iterations {
            operation();
        }
        
        // Measurement phase
        let mut times = Vec::with_capacity(benchmark.iterations);
        let mut memory_samples = Vec::new();
        
        for _ in 0..benchmark.iterations {
            let start = Instant::now();
            operation();
            let elapsed = start.elapsed();
            times.push(elapsed.as_nanos() as f64);
            
            // Sample memory if enabled
            if self.config.profile_memory {
                memory_samples.push(self.sample_memory());
            }
        }
        
        // Calculate statistics
        let result = self.calculate_benchmark_stats(benchmark, &times, &memory_samples);
        
        Ok(result)
    }
    
    /// Run an async benchmark
    pub async fn run_async<F, Fut>(&self, benchmark: &Benchmark, mut operation: F) -> Result<SingleBenchmarkResult, PerfError>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        // Warmup phase
        for _ in 0..benchmark.warmup_iterations {
            operation().await;
        }
        
        // Measurement phase
        let mut times = Vec::with_capacity(benchmark.iterations);
        let mut memory_samples = Vec::new();
        
        for _ in 0..benchmark.iterations {
            let start = Instant::now();
            operation().await;
            let elapsed = start.elapsed();
            times.push(elapsed.as_nanos() as f64);
            
            if self.config.profile_memory {
                memory_samples.push(self.sample_memory());
            }
        }
        
        let result = self.calculate_benchmark_stats(benchmark, &times, &memory_samples);
        
        Ok(result)
    }
    
    /// Run a benchmark suite
    pub async fn run_suite(&self, suite: BenchmarkSuite) -> Result<BenchmarkResult, PerfError> {
        let start_time = Instant::now();
        let mut results = Vec::new();
        
        for benchmark in &suite.benchmarks {
            // Run each benchmark
            let result = self.run_benchmark_default(benchmark).await?;
            results.push(result);
        }
        
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        Ok(BenchmarkResult {
            suite_name: suite.name,
            benchmarks: results,
            total_duration_ms: duration_ms,
            system_info: SystemInfo::default(),
        })
    }
    
    /// Run benchmark with default operation
    async fn run_benchmark_default(&self, benchmark: &Benchmark) -> Result<SingleBenchmarkResult, PerfError> {
        // Simulated benchmark run
        let mut times = Vec::new();
        
        for _ in 0..benchmark.iterations {
            let time = rand::random::<f64>() * 1_000_000.0; // Random time in ns
            times.push(time);
        }
        
        Ok(self.calculate_benchmark_stats(benchmark, &times, &[]))
    }
    
    /// Calculate benchmark statistics
    fn calculate_benchmark_stats(
        &self,
        benchmark: &Benchmark,
        times: &[f64],
        memory_samples: &[u64],
    ) -> SingleBenchmarkResult {
        let n = times.len();
        if n == 0 {
            return SingleBenchmarkResult {
                name: benchmark.name.clone(),
                mean_ns: 0.0,
                std_dev_ns: 0.0,
                min_ns: 0.0,
                max_ns: 0.0,
                median_ns: 0.0,
                percentiles: Percentiles {
                    p5: 0.0, p10: 0.0, p25: 0.0, p50: 0.0,
                    p75: 0.0, p90: 0.0, p95: 0.0, p99: 0.0,
                },
                iterations: 0,
                throughput: 0.0,
                memory_used: None,
            };
        }
        
        // Sort for percentiles
        let mut sorted = times.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        // Calculate mean
        let sum: f64 = sorted.iter().sum();
        let mean = sum / n as f64;
        
        // Calculate standard deviation
        let variance: f64 = sorted.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / n as f64;
        let std_dev = variance.sqrt();
        
        // Calculate percentiles
        let percentile = |p: f64| -> f64 {
            let idx = ((n as f64) * p / 100.0) as usize;
            sorted[idx.min(n - 1)]
        };
        
        // Calculate throughput (ops/sec)
        let throughput = if mean > 0.0 {
            1_000_000_000.0 / mean
        } else {
            0.0
        };
        
        // Memory used
        let memory_used = if !memory_samples.is_empty() {
            let max_memory = memory_samples.iter().max().copied();
            max_memory
        } else {
            None
        };
        
        SingleBenchmarkResult {
            name: benchmark.name.clone(),
            mean_ns: mean,
            std_dev_ns: std_dev,
            min_ns: sorted[0],
            max_ns: sorted[n - 1],
            median_ns: percentile(50.0),
            percentiles: Percentiles {
                p5: percentile(5.0),
                p10: percentile(10.0),
                p25: percentile(25.0),
                p50: percentile(50.0),
                p75: percentile(75.0),
                p90: percentile(90.0),
                p95: percentile(95.0),
                p99: percentile(99.0),
            },
            iterations: n,
            throughput,
            memory_used,
        }
    }
    
    /// Sample memory usage
    fn sample_memory(&self) -> u64 {
        // In real implementation, would use system APIs
        rand::random::<u64>() % 1_000_000 + 10_000
    }
    
    /// List registered benchmarks
    pub async fn list(&self) -> Vec<String> {
        let benchmarks = self.benchmarks.read().await;
        benchmarks.iter().map(|b| b.name.clone()).collect()
    }
    
    /// Clear registered benchmarks
    pub async fn clear(&self) {
        let mut benchmarks = self.benchmarks.write().await;
        benchmarks.clear();
    }
}

/// Benchmark suite
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkSuite {
    /// Suite name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Benchmarks in suite
    pub benchmarks: Vec<Benchmark>,
    /// Tags
    pub tags: Vec<String>,
}

impl BenchmarkSuite {
    /// Create a new benchmark suite
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            benchmarks: Vec::new(),
            tags: Vec::new(),
        }
    }
    
    /// Add a benchmark
    pub fn add(mut self, benchmark: Benchmark) -> Self {
        self.benchmarks.push(benchmark);
        self
    }
    
    /// Add description
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }
    
    /// Add tag
    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }
}

/// Benchmark builder
pub struct BenchmarkBuilder {
    benchmark: Benchmark,
}

impl BenchmarkBuilder {
    /// Create a new builder
    pub fn new(name: &str) -> Self {
        Self {
            benchmark: Benchmark::new(name),
        }
    }
    
    /// Set description
    pub fn description(mut self, desc: &str) -> Self {
        self.benchmark.description = Some(desc.to_string());
        self
    }
    
    /// Set iterations
    pub fn iterations(mut self, count: usize) -> Self {
        self.benchmark.iterations = count;
        self
    }
    
    /// Set warmup iterations
    pub fn warmup(mut self, count: usize) -> Self {
        self.benchmark.warmup_iterations = count;
        self
    }
    
    /// Set measurement type
    pub fn measurement(mut self, mtype: MeasurementType) -> Self {
        self.benchmark.measurement = mtype;
        self
    }
    
    /// Add tag
    pub fn tag(mut self, tag: &str) -> Self {
        self.benchmark.tags.push(tag.to_string());
        self
    }
    
    /// Build the benchmark
    pub fn build(self) -> Benchmark {
        self.benchmark
    }
}

/// Comparison result for benchmarks
#[derive(Debug, Clone)]
pub struct BenchmarkComparison {
    pub baseline: SingleBenchmarkResult,
    pub current: SingleBenchmarkResult,
    pub mean_change_percent: f64,
    pub median_change_percent: f64,
    pub throughput_change_percent: f64,
    pub regression: bool,
}

impl BenchmarkComparison {
    /// Compare two benchmark results
    pub fn compare(baseline: &SingleBenchmarkResult, current: &SingleBenchmarkResult) -> Self {
        let mean_change = Self::percent_change(baseline.mean_ns, current.mean_ns);
        let median_change = Self::percent_change(baseline.median_ns, current.median_ns);
        let throughput_change = Self::percent_change(baseline.throughput, current.throughput);
        
        // Consider it a regression if mean time increased by more than 10%
        let regression = mean_change > 10.0;
        
        Self {
            baseline: baseline.clone(),
            current: current.clone(),
            mean_change_percent: mean_change,
            median_change_percent: median_change,
            throughput_change_percent: throughput_change,
            regression,
        }
    }
    
    fn percent_change(baseline: f64, current: f64) -> f64 {
        if baseline == 0.0 {
            return 0.0;
        }
        ((current - baseline) / baseline) * 100.0
    }
}