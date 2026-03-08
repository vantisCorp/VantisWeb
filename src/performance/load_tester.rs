//! Load testing engine

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use tokio::sync::RwLock;
use std::collections::VecDeque;

use super::models::*;
use super::{PerformanceConfig, PerfError};

/// Load testing engine
pub struct LoadTester {
    config: PerformanceConfig,
    /// Active test flag
    running: AtomicBool,
    /// Request counter
    request_counter: AtomicU64,
    /// Success counter
    success_counter: AtomicU64,
    /// Failure counter
    failure_counter: AtomicU64,
}

impl LoadTester {
    /// Create a new load tester
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            config,
            running: AtomicBool::new(false),
            request_counter: AtomicU64::new(0),
            success_counter: AtomicU64::new(0),
            failure_counter: AtomicU64::new(0),
        }
    }
    
    /// Run a load test scenario
    pub async fn run(&self, scenario: LoadScenario) -> Result<LoadTestResult, PerfError> {
        self.running.store(true, Ordering::SeqCst);
        self.request_counter.store(0, Ordering::SeqCst);
        self.success_counter.store(0, Ordering::SeqCst);
        self.failure_counter.store(0, Ordering::SeqCst);
        
        let start_time = std::time::Instant::now();
        let mut timeline = Vec::new();
        let mut response_times = Vec::new();
        let mut total_bytes = 0u64;
        
        // Ramp-up phase
        let ramp_up_users = self.calculate_ramp_up(&scenario);
        
        // Run virtual users
        let results = self.run_virtual_users(&scenario, ramp_up_users).await?;
        
        for result in results {
            response_times.push(result.response_time_ms);
            total_bytes += result.bytes_transferred;
            
            if result.success {
                self.success_counter.fetch_add(1, Ordering::SeqCst);
            } else {
                self.failure_counter.fetch_add(1, Ordering::SeqCst);
            }
            
            self.request_counter.fetch_add(1, Ordering::SeqCst);
        }
        
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        // Calculate statistics
        let total_requests = self.request_counter.load(Ordering::SeqCst);
        let successful_requests = self.success_counter.load(Ordering::SeqCst);
        let failed_requests = self.failure_counter.load(Ordering::SeqCst);
        
        let stats = self.calculate_statistics(&response_times);
        
        // Build timeline samples
        timeline = self.build_timeline(&response_times, scenario.duration_secs);
        
        self.running.store(false, Ordering::SeqCst);
        
        Ok(LoadTestResult {
            scenario_name: scenario.name,
            total_requests,
            successful_requests,
            failed_requests,
            avg_response_time_ms: stats.mean,
            min_response_time_ms: stats.min,
            max_response_time_ms: stats.max,
            p50_response_time_ms: stats.p50,
            p90_response_time_ms: stats.p90,
            p95_response_time_ms: stats.p95,
            p99_response_time_ms: stats.p99,
            requests_per_second: (total_requests as f64 / duration_ms as f64) * 1000.0,
            total_bytes,
            error_rate: if total_requests > 0 {
                failed_requests as f64 / total_requests as f64
            } else {
                0.0
            },
            duration_ms,
            timeline,
        })
    }
    
    /// Stop running test
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
    
    /// Check if test is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
    
    /// Get current request count
    pub fn request_count(&self) -> u64 {
        self.request_counter.load(Ordering::SeqCst)
    }
    
    /// Calculate ramp-up schedule
    fn calculate_ramp_up(&self, scenario: &LoadScenario) -> Vec<(u64, usize)> {
        let mut schedule = Vec::new();
        let ramp_up_secs = scenario.ramp_up_secs;
        let total_users = scenario.virtual_users;
        let interval_ms = 100u64; // 100ms intervals
        
        if ramp_up_secs == 0 {
            schedule.push((0, total_users));
            return schedule;
        }
        
        let steps = (ramp_up_secs * 1000) / interval_ms;
        let users_per_step = total_users as f64 / steps as f64;
        
        for i in 0..steps {
            let users = ((i + 1) as f64 * users_per_step).min(total_users as f64) as usize;
            schedule.push((i * interval_ms, users));
        }
        
        schedule
    }
    
    /// Run virtual users
    async fn run_virtual_users(
        &self,
        scenario: &LoadScenario,
        ramp_up_schedule: Vec<(u64, usize)>,
    ) -> Result<Vec<RequestResult>, PerfError> {
        let results = Arc::new(RwLock::new(Vec::new()));
        let duration = std::time::Duration::from_secs(scenario.duration_secs);
        let start = std::time::Instant::now();
        
        // Simulate virtual users
        let mut all_results = Vec::new();
        
        for (delay_ms, users) in ramp_up_schedule {
            if start.elapsed() >= duration || !self.running.load(Ordering::SeqCst) {
                break;
            }
            
            // Wait for ramp-up
            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
            
            // Simulate user requests
            for _ in 0..users {
                if start.elapsed() >= duration {
                    break;
                }
                
                let result = self.simulate_request(scenario).await;
                all_results.push(result);
                
                // Think time
                if let Some(think_time) = scenario.think_time_ms {
                    tokio::time::sleep(std::time::Duration::from_millis(think_time)).await;
                }
            }
        }
        
        // Continue for remaining duration
        while start.elapsed() < duration && self.running.load(Ordering::SeqCst) {
            for _ in 0..scenario.virtual_users {
                if start.elapsed() >= duration {
                    break;
                }
                
                let result = self.simulate_request(scenario).await;
                all_results.push(result);
                
                if let Some(think_time) = scenario.think_time_ms {
                    tokio::time::sleep(std::time::Duration::from_millis(think_time)).await;
                }
            }
        }
        
        Ok(all_results)
    }
    
    /// Simulate a single request
    async fn simulate_request(&self, scenario: &LoadScenario) -> RequestResult {
        let start = std::time::Instant::now();
        
        // Simulate network latency
        let latency = self.simulate_latency().await;
        
        // Simulate processing
        tokio::time::sleep(std::time::Duration::from_millis(latency)).await;
        
        let response_time_ms = start.elapsed().as_millis() as f64;
        
        // Simulate random failures based on load
        let success = rand::random::<f64>() > 0.01; // 1% base failure rate
        
        RequestResult {
            response_time_ms,
            success,
            status_code: if success { 200 } else { 500 },
            bytes_transferred: rand::random::<u64>() % 10000 + 500,
            error: if success { None } else { Some("Internal Server Error".to_string()) },
        }
    }
    
    /// Simulate network latency
    async fn simulate_latency(&self) -> u64 {
        // Simulate variable latency
        let base_latency = 10u64;
        let variance = rand::random::<u64>() % 50;
        base_latency + variance
    }
    
    /// Calculate statistics from response times
    fn calculate_statistics(&self, times: &[f64]) -> ResponseStats {
        if times.is_empty() {
            return ResponseStats::default();
        }
        
        let mut sorted = times.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let n = sorted.len();
        let sum: f64 = sorted.iter().sum();
        let mean = sum / n as f64;
        
        let variance: f64 = sorted.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / n as f64;
        let _std_dev = variance.sqrt();
        
        ResponseStats {
            mean,
            min: sorted[0],
            max: sorted[n - 1],
            p50: sorted[(n as f64 * 0.50) as usize],
            p90: sorted[(n as f64 * 0.90) as usize],
            p95: sorted[(n as f64 * 0.95) as usize],
            p99: sorted[(n as f64 * 0.99) as usize],
        }
    }
    
    /// Build timeline samples
    fn build_timeline(&self, times: &[f64], duration_secs: u64) -> Vec<TimelineSample> {
        let mut timeline = Vec::new();
        let sample_interval = std::time::Duration::from_millis(self.config.sampling_interval_ms);
        let total_samples = (duration_secs * 1000) / self.config.sampling_interval_ms;
        
        let times_per_sample = if total_samples > 0 {
            times.len() / total_samples as usize
        } else {
            1
        };
        
        for i in 0..total_samples as usize {
            let start_idx = i * times_per_sample;
            let end_idx = ((i + 1) * times_per_sample).min(times.len());
            
            let sample_times = &times[start_idx..end_idx];
            let avg_time = if !sample_times.is_empty() {
                sample_times.iter().sum::<f64>() / sample_times.len() as f64
            } else {
                0.0
            };
            
            timeline.push(TimelineSample {
                timestamp: chrono::Utc::now() + chrono::Duration::milliseconds(i as i64 * 100),
                active_users: 0,
                response_time_ms: avg_time,
                rps: if duration_secs > 0 {
                    times.len() as f64 / duration_secs as f64
                } else {
                    0.0
                },
                error_rate: 0.0,
            });
        }
        
        timeline
    }
}

/// Request result
#[derive(Debug, Clone)]
struct RequestResult {
    response_time_ms: f64,
    success: bool,
    status_code: u16,
    bytes_transferred: u64,
    error: Option<String>,
}

/// Response statistics
#[derive(Debug, Clone, Default)]
struct ResponseStats {
    mean: f64,
    min: f64,
    max: f64,
    p50: f64,
    p90: f64,
    p95: f64,
    p99: f64,
}

/// Load test builder
pub struct LoadTestBuilder {
    scenario: LoadScenario,
}

impl LoadTestBuilder {
    /// Create a new builder
    pub fn new(name: &str, target: &str) -> Self {
        Self {
            scenario: LoadScenario::new(name, target, 100),
        }
    }
    
    /// Set virtual users
    pub fn users(mut self, count: usize) -> Self {
        self.scenario.virtual_users = count;
        self
    }
    
    /// Set duration
    pub fn duration(mut self, secs: u64) -> Self {
        self.scenario.duration_secs = secs;
        self
    }
    
    /// Set ramp-up
    pub fn ramp_up(mut self, secs: u64) -> Self {
        self.scenario.ramp_up_secs = secs;
        self
    }
    
    /// Set think time
    pub fn think_time(mut self, ms: u64) -> Self {
        self.scenario.think_time_ms = Some(ms);
        self
    }
    
    /// Set max RPS
    pub fn max_rps(mut self, rps: u32) -> Self {
        self.scenario.max_rps = Some(rps);
        self
    }
    
    /// Build the scenario
    pub fn build(self) -> LoadScenario {
        self.scenario
    }
}