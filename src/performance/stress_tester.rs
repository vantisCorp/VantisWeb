//! Stress testing engine

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use tokio::sync::RwLock;

use super::models::*;
use super::{PerformanceConfig, PerfError};

/// Stress testing engine
pub struct StressTester {
    config: PerformanceConfig,
    running: AtomicBool,
    current_load: AtomicUsize,
}

impl StressTester {
    /// Create a new stress tester
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            config,
            running: AtomicBool::new(false),
            current_load: AtomicUsize::new(0),
        }
    }
    
    /// Run a stress test scenario
    pub async fn run(&self, scenario: StressScenario) -> Result<StressTestResult, PerfError> {
        self.running.store(true, Ordering::SeqCst);
        
        let start_time = std::time::Instant::now();
        let mut step_results = Vec::new();
        let mut breaking_point = None;
        let mut max_sustained_load = 0;
        
        let mut current_load = scenario.start_load;
        
        while current_load <= scenario.max_load && self.running.load(Ordering::SeqCst) {
            self.current_load.store(current_load, Ordering::SeqCst);
            
            // Run step
            let step_result = self.run_step(&scenario, current_load).await?;
            
            let passed = step_result.error_rate < scenario.failure_threshold;
            
            if !passed && breaking_point.is_none() {
                breaking_point = Some(current_load);
            }
            
            if passed {
                max_sustained_load = current_load;
            }
            
            step_results.push(step_result.clone());
            
            // Check if we should continue
            if !passed {
                // System is failing, stop stress test
                break;
            }
            
            current_load += scenario.load_step;
        }
        
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        self.running.store(false, Ordering::SeqCst);
        
        Ok(StressTestResult {
            scenario_name: scenario.name,
            breaking_point,
            steps_completed: step_results.len(),
            step_results,
            max_sustained_load,
            duration_ms,
        })
    }
    
    /// Run a single step at a given load level
    async fn run_step(&self, scenario: &StressScenario, load: usize) -> Result<StepResult, PerfError> {
        let start = std::time::Instant::now();
        let duration = std::time::Duration::from_secs(scenario.step_duration_secs);
        
        let mut response_times = Vec::new();
        let mut errors = 0usize;
        let mut requests = 0usize;
        
        // Simulate load
        while start.elapsed() < duration && self.running.load(Ordering::SeqCst) {
            // Spawn concurrent requests
            let results = self.simulate_concurrent_requests(load, scenario.target.clone()).await;
            
            for result in results {
                requests += 1;
                response_times.push(result.response_time_ms);
                if !result.success {
                    errors += 1;
                }
            }
            
            // Small delay between batches
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        
        let step_duration_secs = scenario.step_duration_secs as f64;
        let avg_response_time = if !response_times.is_empty() {
            response_times.iter().sum::<f64>() / response_times.len() as f64
        } else {
            0.0
        };
        
        let error_rate = if requests > 0 {
            errors as f64 / requests as f64
        } else {
            0.0
        };
        
        let throughput = requests as f64 / step_duration_secs;
        
        Ok(StepResult {
            load_level: load,
            avg_response_time_ms: avg_response_time,
            error_rate,
            throughput,
            passed: error_rate < scenario.failure_threshold,
        })
    }
    
    /// Simulate concurrent requests
    async fn simulate_concurrent_requests(&self, count: usize, _target: String) -> Vec<RequestOutcome> {
        let mut results = Vec::new();
        
        for _ in 0..count {
            // Simulate individual request
            let response_time = self.simulate_stress_response().await;
            let success = rand::random::<f64>() > self.calculate_failure_rate(count);
            
            results.push(RequestOutcome {
                response_time_ms: response_time,
                success,
            });
        }
        
        results
    }
    
    /// Simulate response under stress
    async fn simulate_stress_response(&self) -> f64 {
        // Higher load = higher response time
        let load = self.current_load.load(Ordering::SeqCst);
        let base_time = 10.0;
        let load_factor = load as f64 / 100.0;
        let random_variance = rand::random::<f64>() * 50.0;
        
        base_time + (load_factor * 5.0) + random_variance
    }
    
    /// Calculate failure rate based on load
    fn calculate_failure_rate(&self, load: usize) -> f64 {
        // Exponential increase in failure rate as load increases
        let base_rate = 0.001;
        let load_factor = (load as f64 / 500.0).powi(2);
        (base_rate + load_factor * 0.1).min(1.0)
    }
    
    /// Stop running test
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
    
    /// Check if test is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
    
    /// Get current load level
    pub fn current_load(&self) -> usize {
        self.current_load.load(Ordering::SeqCst)
    }
}

/// Request outcome
#[derive(Debug, Clone)]
struct RequestOutcome {
    response_time_ms: f64,
    success: bool,
}

/// Stress test builder
pub struct StressTestBuilder {
    scenario: StressScenario,
}

impl StressTestBuilder {
    /// Create a new builder
    pub fn new(name: &str, target: &str) -> Self {
        Self {
            scenario: StressScenario::new(name, target),
        }
    }
    
    /// Set starting load
    pub fn start_load(mut self, load: usize) -> Self {
        self.scenario.start_load = load;
        self
    }
    
    /// Set maximum load
    pub fn max_load(mut self, load: usize) -> Self {
        self.scenario.max_load = load;
        self
    }
    
    /// Set load increment step
    pub fn load_step(mut self, step: usize) -> Self {
        self.scenario.load_step = step;
        self
    }
    
    /// Set step duration
    pub fn step_duration(mut self, secs: u64) -> Self {
        self.scenario.step_duration_secs = secs;
        self
    }
    
    /// Set failure threshold
    pub fn failure_threshold(mut self, threshold: f64) -> Self {
        self.scenario.failure_threshold = threshold;
        self
    }
    
    /// Build the scenario
    pub fn build(self) -> StressScenario {
        self.scenario
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_stress_test_basic() {
        let config = PerformanceConfig::default();
        let tester = StressTester::new(config);
        
        let scenario = StressTestBuilder::new("test", "http://example.com")
            .start_load(10)
            .max_load(50)
            .load_step(20)
            .step_duration(1)
            .build();
        
        let result = tester.run(scenario).await.unwrap();
        
        assert!(result.steps_completed > 0);
        assert!(result.max_sustained_load > 0);
    }
}