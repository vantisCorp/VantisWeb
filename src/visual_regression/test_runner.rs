//! Test runner for visual regression tests

use std::path::PathBuf;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

use super::models::*;
use super::{VisualTest, Viewport, VisualRegressionConfig, VRError};

/// Test runner for executing visual regression tests
pub struct TestRunner {
    config: VisualRegressionConfig,
    /// Test registry
    tests: RwLock<HashMap<String, VisualTest>>,
    /// Suite registry
    suites: RwLock<HashMap<String, TestSuite>>,
}

/// Test suite definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite {
    /// Suite name
    pub name: String,
    /// Tests in suite
    pub tests: Vec<VisualTest>,
    /// Viewports to test
    pub viewports: Vec<Viewport>,
    /// Parallel execution
    pub parallel: bool,
    /// Max concurrent tests
    pub max_concurrent: usize,
    /// Retry count on failure
    pub retries: u32,
    /// Suite tags
    pub tags: Vec<String>,
}

impl TestSuite {
    /// Create a new test suite
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tests: Vec::new(),
            viewports: vec![
                Viewport::desktop(),
                Viewport::tablet(),
                Viewport::mobile(),
            ],
            parallel: true,
            max_concurrent: 4,
            retries: 0,
            tags: Vec::new(),
        }
    }
    
    /// Add a test to the suite
    pub fn add_test(mut self, test: VisualTest) -> Self {
        self.tests.push(test);
        self
    }
    
    /// Add viewport
    pub fn add_viewport(mut self, viewport: Viewport) -> Self {
        self.viewports.push(viewport);
        self
    }
    
    /// Set parallel execution
    pub fn parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }
    
    /// Set max concurrent tests
    pub fn max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent = max;
        self
    }
    
    /// Set retry count
    pub fn retries(mut self, retries: u32) -> Self {
        self.retries = retries;
        self
    }
    
    /// Add tag
    pub fn add_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }
}

impl TestRunner {
    /// Create a new test runner
    pub fn new(config: VisualRegressionConfig) -> Self {
        Self {
            config,
            tests: RwLock::new(HashMap::new()),
            suites: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a test
    pub async fn register_test(&self, test: VisualTest) {
        let mut tests = self.tests.write().await;
        tests.insert(test.name.clone(), test);
    }
    
    /// Register a test suite
    pub async fn register_suite(&self, suite: TestSuite) {
        let mut suites = self.suites.write().await;
        suites.insert(suite.name.clone(), suite);
    }
    
    /// Run a single test
    pub async fn run_single(
        &self,
        test: &VisualTest,
        capture: &super::ScreenshotCapture,
        diff_engine: &super::DiffEngine,
        baselines: &mut super::BaselineManager,
    ) -> Result<TestResult, VRError> {
        // Capture screenshot
        let screenshot = capture.capture(test).await?;
        
        // Load baseline
        let baseline = baselines.load(&test.name, &test.viewport.name).await?;
        
        match baseline {
            Some(b) => {
                // Compare with baseline
                let diff = diff_engine.compare(&b, &screenshot).await?;
                
                let passed = diff.diff_percent <= self.config.max_diff_percent;
                
                if !passed {
                    diff_engine.save_diff(&diff, &test.name).await?;
                }
                
                Ok(TestResult {
                    test_name: test.name.clone(),
                    viewport: test.viewport.name.clone(),
                    passed,
                    diff_percent: diff.diff_percent,
                    diff_pixels: diff.diff_pixels,
                    baseline_path: Some(b.path),
                    screenshot_path: screenshot.path.clone(),
                    diff_path: if !passed { Some(diff.path) } else { None },
                })
            }
            None => {
                // Create new baseline
                baselines.save(&test.name, &test.viewport.name, &screenshot).await?;
                
                Ok(TestResult {
                    test_name: test.name.clone(),
                    viewport: test.viewport.name.clone(),
                    passed: true,
                    diff_percent: 0.0,
                    diff_pixels: 0,
                    baseline_path: None,
                    screenshot_path: screenshot.path.clone(),
                    diff_path: None,
                })
            }
        }
    }
    
    /// Run all tests in a suite
    pub async fn run_suite(
        &self,
        suite_name: &str,
        capture: &super::ScreenshotCapture,
        diff_engine: &super::DiffEngine,
        baselines: &mut super::BaselineManager,
    ) -> Result<SuiteRunResult, VRError> {
        let suites = self.suites.read().await;
        let suite = suites.get(suite_name)
            .ok_or_else(|| VRError::ConfigError(format!("Suite not found: {}", suite_name)))?
            .clone();
        drop(suites);
        
        let mut results = Vec::new();
        let start_time = std::time::Instant::now();
        
        if suite.parallel {
            results = self.run_parallel(&suite, capture, diff_engine, baselines).await?;
        } else {
            results = self.run_sequential(&suite, capture, diff_engine, baselines).await?;
        }
        
        let duration = start_time.elapsed();
        
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = results.len() - passed;
        
        Ok(SuiteRunResult {
            suite_name: suite.name,
            total_tests: results.len(),
            passed,
            failed,
            duration_ms: duration.as_millis() as u64,
            results,
        })
    }
    
    /// Run tests in parallel
    async fn run_parallel(
        &self,
        suite: &TestSuite,
        capture: &super::ScreenshotCapture,
        diff_engine: &super::DiffEngine,
        baselines: &mut super::BaselineManager,
    ) -> Result<Vec<TestResult>, VRError> {
        // For simplicity, we'll run sequentially here
        // In real implementation, would use tokio::task::JoinSet
        self.run_sequential(suite, capture, diff_engine, baselines).await
    }
    
    /// Run tests sequentially
    async fn run_sequential(
        &self,
        suite: &TestSuite,
        capture: &super::ScreenshotCapture,
        diff_engine: &super::DiffEngine,
        baselines: &mut super::BaselineManager,
    ) -> Result<Vec<TestResult>, VRError> {
        let mut results = Vec::new();
        
        for test in &suite.tests {
            for viewport in &suite.viewports {
                let test_with_viewport = VisualTest {
                    name: test.name.clone(),
                    url: test.url.clone(),
                    viewport: viewport.clone(),
                    selectors: test.selectors.clone(),
                    wait_conditions: test.wait_conditions.clone(),
                };
                
                let result = self.run_single(&test_with_viewport, capture, diff_engine, baselines).await?;
                results.push(result);
            }
        }
        
        Ok(results)
    }
    
    /// Run all registered tests
    pub async fn run_all(
        &self,
        capture: &super::ScreenshotCapture,
        diff_engine: &super::DiffEngine,
        baselines: &mut super::BaselineManager,
    ) -> Result<RunAllResult, VRError> {
        let tests = self.tests.read().await;
        let test_list: Vec<_> = tests.values().cloned().collect();
        drop(tests);
        
        let mut results = Vec::new();
        let start_time = std::time::Instant::now();
        
        for test in &test_list {
            let result = self.run_single(test, capture, diff_engine, baselines).await?;
            results.push(result);
        }
        
        let duration = start_time.elapsed();
        
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = results.len() - passed;
        
        Ok(RunAllResult {
            total_tests: results.len(),
            passed,
            failed,
            duration_ms: duration.as_millis() as u64,
            results,
        })
    }
    
    /// List all registered tests
    pub async fn list_tests(&self) -> Vec<String> {
        let tests = self.tests.read().await;
        tests.keys().cloned().collect()
    }
    
    /// List all registered suites
    pub async fn list_suites(&self) -> Vec<String> {
        let suites = self.suites.read().await;
        suites.keys().cloned().collect()
    }
    
    /// Load tests from configuration file
    pub async fn load_from_config(&self, path: &PathBuf) -> Result<usize, VRError> {
        let content = tokio::fs::read_to_string(path).await?;
        let config: TestConfig = serde_json::from_str(&content)
            .map_err(|e| VRError::ConfigError(format!("Failed to parse config: {}", e)))?;
        
        let mut count = 0;
        
        for test in config.tests {
            self.register_test(test).await;
            count += 1;
        }
        
        for suite in config.suites {
            self.register_suite(suite).await;
            count += suite.tests.len();
        }
        
        Ok(count)
    }
    
    /// Export tests to configuration file
    pub async fn export_config(&self, path: &PathBuf) -> Result<(), VRError> {
        let tests = self.tests.read().await;
        let suites = self.suites.read().await;
        
        let config = TestConfig {
            tests: tests.values().cloned().collect(),
            suites: suites.values().cloned().collect(),
        };
        
        let json = serde_json::to_string_pretty(&config)?;
        tokio::fs::write(path, &json).await?;
        
        Ok(())
    }
}

/// Suite run result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteRunResult {
    pub suite_name: String,
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub duration_ms: u64,
    pub results: Vec<TestResult>,
}

/// Run all result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunAllResult {
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub duration_ms: u64,
    pub results: Vec<TestResult>,
}

/// Test configuration file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub tests: Vec<VisualTest>,
    pub suites: Vec<TestSuite>,
}

/// CI/CD integration
pub struct CIIntegration {
    config: CIConfig,
}

/// CI configuration
#[derive(Debug, Clone)]
pub struct CIConfig {
    /// Fail on diff
    pub fail_on_diff: bool,
    /// Update baselines automatically
    pub update_baselines: bool,
    /// Generate report
    pub generate_report: bool,
    /// Report format
    pub report_format: ReportFormat,
    /// Output directory
    pub output_dir: PathBuf,
    /// Notify on failure
    pub notify_on_failure: bool,
    /// Webhook URL for notifications
    pub webhook_url: Option<String>,
}

impl Default for CIConfig {
    fn default() -> Self {
        Self {
            fail_on_diff: true,
            update_baselines: false,
            generate_report: true,
            report_format: ReportFormat::HTML,
            output_dir: PathBuf::from("./visual-regression-output"),
            notify_on_failure: false,
            webhook_url: None,
        }
    }
}

impl CIIntegration {
    /// Create new CI integration
    pub fn new(config: CIConfig) -> Self {
        Self { config }
    }
    
    /// Generate CI report
    pub async fn generate_report(
        &self,
        results: &[TestResult],
    ) -> Result<PathBuf, VRError> {
        tokio::fs::create_dir_all(&self.config.output_dir).await?;
        
        let path = match self.config.report_format {
            ReportFormat::HTML => {
                let viewer = super::DiffViewer::new(self.config.output_dir.clone());
                viewer.generate_report(results).await?
            }
            ReportFormat::JSON => {
                let viewer = super::DiffViewer::new(self.config.output_dir.clone());
                viewer.generate_json_report(results).await?
            }
            ReportFormat::Markdown => {
                let viewer = super::DiffViewer::new(self.config.output_dir.clone());
                viewer.generate_markdown_report(results).await?
            }
            ReportFormat::JUnit => {
                self.generate_junit_report(results).await?
            }
        };
        
        Ok(path)
    }
    
    /// Generate JUnit XML report
    async fn generate_junit_report(&self, results: &[TestResult]) -> Result<PathBuf, VRError> {
        let total = results.len();
        let failures = results.iter().filter(|r| !r.passed).count();
        
        let mut xml = format!(
            r##"<?xml version="1.0" encoding="UTF-8"?>
<testsuite name="visual-regression" tests="{}" failures="{}">
"##,
            total, failures
        );
        
        for result in results {
            xml.push_str(&format!(
                r##"  <testcase name="{}_{}" classname="visual-regression">
{},
    </testcase>
"##,
                result.test_name,
                result.viewport,
                if result.passed {
                    String::new()
                } else {
                    format!(
                        "    <failure message=&quot;Diff: {:.2}%&quot;>Diff percentage: {:.2}%</failure>\n",
                        result.diff_percent,
                        result.diff_percent
                    )
                }
            ));
        }
        
        xml.push_str("</testsuite>\n");
        
        let path = self.config.output_dir.join("junit.xml");
        tokio::fs::write(&path, &xml).await?;
        
        Ok(path)
    }
    
    /// Send notification
    pub async fn send_notification(
        &self,
        results: &[TestResult],
    ) -> Result<(), VRError> {
        if let Some(webhook_url) = &self.config.webhook_url {
            let passed = results.iter().filter(|r| r.passed).count();
            let failed = results.len() - passed;
            
            let payload = serde_json::json!({
                "text": format!(
                    "Visual Regression Tests: {} passed, {} failed",
                    passed, failed
                ),
                "results": results.iter().map(|r| serde_json::json!({
                    "test": r.test_name,
                    "viewport": r.viewport,
                    "passed": r.passed,
                    "diff_percent": r.diff_percent
                })).collect::<Vec<_>>()
            });
            
            let client = reqwest::Client::new();
            client.post(webhook_url)
                .json(&payload)
                .send()
                .await
                .map_err(|e| VRError::ConfigError(format!("Webhook failed: {}", e)))?;
        }
        
        Ok(())
    }
}

/// Report format
#[derive(Debug, Clone, Copy)]
pub enum ReportFormat {
    HTML,
    JSON,
    JUnit,
    Markdown,
}