//! Visual Regression Testing Module
//! 
//! Automated visual testing and screenshot comparison for web applications
//! 
//! # Features
//! - Screenshot capture and baseline management
//! - Pixel-perfect diff detection
//! - Perceptual diff (pDiff) algorithms
//! - Multi-viewport testing (desktop, tablet, mobile)
//! - CI/CD integration support
//! - Interactive diff viewer

pub mod models;
pub mod capture;
pub mod comparison;
pub mod baseline;
pub mod diff_viewer;
pub mod test_runner;

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

pub use models::*;
pub use capture::ScreenshotCapture;
pub use comparison::{DiffEngine, DiffResult, DiffConfig};
pub use baseline::{BaselineManager, BaselineStore};
pub use diff_viewer::DiffViewer;
pub use test_runner::{TestRunner, TestSuite, TestResult};

/// Visual Regression Testing Manager
pub struct VisualRegressionManager {
    /// Screenshot capture engine
    capture: Arc<ScreenshotCapture>,
    /// Diff comparison engine
    diff_engine: Arc<DiffEngine>,
    /// Baseline storage manager
    baselines: Arc<RwLock<BaselineManager>>,
    /// Test runner
    runner: Arc<TestRunner>,
    /// Configuration
    config: VisualRegressionConfig,
}

/// Configuration for visual regression testing
#[derive(Debug, Clone)]
pub struct VisualRegressionConfig {
    /// Root directory for screenshots
    pub screenshot_dir: PathBuf,
    /// Baseline directory
    pub baseline_dir: PathBuf,
    /// Diff output directory
    pub diff_dir: PathBuf,
    /// Threshold for pixel difference (0.0 - 1.0)
    pub pixel_threshold: f32,
    /// Threshold for perceptual difference (0.0 - 1.0)
    pub perceptual_threshold: f32,
    /// Enable anti-aliasing detection
    pub detect_antialiasing: bool,
    /// Viewports to test
    pub viewports: Vec<Viewport>,
    /// Capture delay in milliseconds
    pub capture_delay_ms: u64,
    /// Maximum diff percentage before failure
    pub max_diff_percent: f32,
}

impl Default for VisualRegressionConfig {
    fn default() -> Self {
        Self {
            screenshot_dir: PathBuf::from("./screenshots"),
            baseline_dir: PathBuf::from("./baselines"),
            diff_dir: PathBuf::from("./diffs"),
            pixel_threshold: 0.0,
            perceptual_threshold: 0.05,
            detect_antialiasing: true,
            viewports: vec![
                Viewport::desktop(),
                Viewport::tablet(),
                Viewport::mobile(),
            ],
            capture_delay_ms: 100,
            max_diff_percent: 1.0,
        }
    }
}

/// Viewport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewport {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub device_scale_factor: f32,
    pub user_agent: Option<String>,
}

impl Viewport {
    /// Desktop viewport (1920x1080)
    pub fn desktop() -> Self {
        Self {
            name: "desktop".to_string(),
            width: 1920,
            height: 1080,
            device_scale_factor: 1.0,
            user_agent: None,
        }
    }
    
    /// Tablet viewport (1024x768)
    pub fn tablet() -> Self {
        Self {
            name: "tablet".to_string(),
            width: 1024,
            height: 768,
            device_scale_factor: 2.0,
            user_agent: Some("Mozilla/5.0 (iPad; CPU OS 14_0 like Mac OS X)".to_string()),
        }
    }
    
    /// Mobile viewport (375x667)
    pub fn mobile() -> Self {
        Self {
            name: "mobile".to_string(),
            width: 375,
            height: 667,
            device_scale_factor: 3.0,
            user_agent: Some("Mozilla/5.0 (iPhone; CPU iPhone OS 14_0 like Mac OS X)".to_string()),
        }
    }
    
    /// Custom viewport
    pub fn custom(name: &str, width: u32, height: u32) -> Self {
        Self {
            name: name.to_string(),
            width,
            height,
            device_scale_factor: 1.0,
            user_agent: None,
        }
    }
}

impl VisualRegressionManager {
    /// Create a new visual regression manager
    pub fn new(config: VisualRegressionConfig) -> Self {
        let capture = Arc::new(ScreenshotCapture::new(config.clone()));
        let diff_engine = Arc::new(DiffEngine::new(config.clone()));
        let baselines = Arc::new(RwLock::new(BaselineManager::new(config.clone())));
        let runner = Arc::new(TestRunner::new(config.clone()));
        
        Self {
            capture,
            diff_engine,
            baselines,
            runner,
            config,
        }
    }
    
    /// Run visual regression test for a single scenario
    pub async fn run_test(&self, test: VisualTest) -> Result<TestResult, VRError> {
        // Capture screenshot
        let screenshot = self.capture.capture(&test).await?;
        
        // Load baseline
        let baselines = self.baselines.read().await;
        let baseline = baselines.load(&test.name, &test.viewport.name).await?;
        
        match baseline {
            Some(baseline) => {
                // Compare with baseline
                let diff = self.diff_engine.compare(&baseline, &screenshot).await?;
                
                // Determine if test passed
                let passed = diff.diff_percent <= self.config.max_diff_percent;
                
                // Save diff if failed
                if !passed {
                    self.diff_engine.save_diff(&diff, &test.name).await?;
                }
                
                Ok(TestResult {
                    test_name: test.name,
                    viewport: test.viewport.name,
                    passed,
                    diff_percent: diff.diff_percent,
                    diff_pixels: diff.diff_pixels,
                    baseline_path: Some(baseline.path),
                    screenshot_path: screenshot.path,
                    diff_path: if !passed { Some(diff.path) } else { None },
                })
            }
            None => {
                // No baseline exists - create it
                drop(baselines);
                let mut baselines = self.baselines.write().await;
                baselines.save(&test.name, &test.viewport.name, &screenshot).await?;
                
                Ok(TestResult {
                    test_name: test.name,
                    viewport: test.viewport.name,
                    passed: true, // New baseline counts as passed
                    diff_percent: 0.0,
                    diff_pixels: 0,
                    baseline_path: None,
                    screenshot_path: screenshot.path,
                    diff_path: None,
                })
            }
        }
    }
    
    /// Run a full test suite
    pub async fn run_suite(&self, suite: TestSuite) -> Result<SuiteResult, VRError> {
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
                
                let result = self.run_test(test_with_viewport).await?;
                results.push(result);
            }
        }
        
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = results.len() - passed;
        
        Ok(SuiteResult {
            suite_name: suite.name,
            total_tests: results.len(),
            passed,
            failed,
            results,
        })
    }
    
    /// Update baseline for a test
    pub async fn update_baseline(&self, test: VisualTest) -> Result<(), VRError> {
        let screenshot = self.capture.capture(&test).await?;
        
        let mut baselines = self.baselines.write().await;
        baselines.save(&test.name, &test.viewport.name, &screenshot).await?;
        
        Ok(())
    }
    
    /// Get the capture engine
    pub fn capture(&self) -> &ScreenshotCapture {
        &self.capture
    }
    
    /// Get the diff engine
    pub fn diff_engine(&self) -> &DiffEngine {
        &self.diff_engine
    }
    
    /// Get the baseline manager
    pub async fn baselines(&self) -> tokio::sync::RwLockReadGuard<'_, BaselineManager> {
        self.baselines.read().await
    }
}

/// Suite result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteResult {
    pub suite_name: String,
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub results: Vec<TestResult>,
}

use serde::{Serialize, Deserialize};
use crate::visual_regression::capture::Screenshot;
use crate::visual_regression::baseline::Baseline;