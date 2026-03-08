//! Data models for visual regression testing

use std::path::PathBuf;
use serde::{Serialize, Deserialize};

/// Visual test definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualTest {
    /// Test name/identifier
    pub name: String,
    /// URL to capture
    pub url: String,
    /// Viewport configuration
    pub viewport: super::Viewport,
    /// CSS selectors to wait for before capture
    pub selectors: Vec<String>,
    /// Wait conditions
    pub wait_conditions: Vec<WaitCondition>,
}

impl VisualTest {
    /// Create a new visual test
    pub fn new(name: &str, url: &str, viewport: super::Viewport) -> Self {
        Self {
            name: name.to_string(),
            url: url.to_string(),
            viewport,
            selectors: Vec::new(),
            wait_conditions: Vec::new(),
        }
    }
    
    /// Add a selector to wait for
    pub fn with_selector(mut self, selector: &str) -> Self {
        self.selectors.push(selector.to_string());
        self
    }
    
    /// Add a wait condition
    pub fn with_wait_condition(mut self, condition: WaitCondition) -> Self {
        self.wait_conditions.push(condition);
        self
    }
}

/// Wait conditions before capture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WaitCondition {
    /// Wait for selector to appear
    Selector(String),
    /// Wait for specified milliseconds
    Timeout(u64),
    /// Wait for network idle
    NetworkIdle,
    /// Wait for page load
    PageLoad,
    /// Wait for custom JavaScript condition
    Script(String),
    /// Wait for element to be visible
    Visible(String),
    /// Wait for element to be hidden
    Hidden(String),
}

/// Screenshot data
#[derive(Debug, Clone)]
pub struct Screenshot {
    /// Image width
    pub width: u32,
    /// Image height
    pub height: u32,
    /// Raw pixel data (RGBA)
    pub data: Vec<u8>,
    /// File path
    pub path: PathBuf,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Screenshot {
    /// Create a new screenshot
    pub fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self {
            width,
            height,
            data,
            path: PathBuf::new(),
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// Get pixel at coordinates
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        
        let index = ((y * self.width + x) * 4) as usize;
        if index + 3 < self.data.len() {
            Some([
                self.data[index],
                self.data[index + 1],
                self.data[index + 2],
                self.data[index + 3],
            ])
        } else {
            None
        }
    }
    
    /// Set pixel at coordinates
    pub fn set_pixel(&mut self, x: u32, y: u32, rgba: [u8; 4]) {
        if x >= self.width || y >= self.height {
            return;
        }
        
        let index = ((y * self.width + x) * 4) as usize;
        if index + 3 < self.data.len() {
            self.data[index] = rgba[0];
            self.data[index + 1] = rgba[1];
            self.data[index + 2] = rgba[2];
            self.data[index + 3] = rgba[3];
        }
    }
    
    /// Convert to PNG bytes
    pub fn to_png(&self) -> Result<Vec<u8>, VRError> {
        // Simplified PNG encoding - in real implementation would use image crate
        Ok(self.data.clone())
    }
}

/// Baseline screenshot
#[derive(Debug, Clone)]
pub struct Baseline {
    /// Baseline name
    pub name: String,
    /// Viewport name
    pub viewport: String,
    /// Screenshot data
    pub screenshot: Screenshot,
    /// File path
    pub path: PathBuf,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Version/hash
    pub version: String,
}

impl Baseline {
    /// Create a new baseline
    pub fn new(name: &str, viewport: &str, screenshot: Screenshot) -> Self {
        let hash = format!("{:x}", md5::compute(&screenshot.data));
        Self {
            name: name.to_string(),
            viewport: viewport.to_string(),
            path: PathBuf::new(),
            created_at: chrono::Utc::now(),
            version: hash,
            screenshot,
        }
    }
}

/// Test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Test name
    pub test_name: String,
    /// Viewport name
    pub viewport: String,
    /// Whether test passed
    pub passed: bool,
    /// Percentage of different pixels
    pub diff_percent: f32,
    /// Number of different pixels
    pub diff_pixels: usize,
    /// Path to baseline image
    pub baseline_path: Option<PathBuf>,
    /// Path to current screenshot
    pub screenshot_path: PathBuf,
    /// Path to diff image (if failed)
    pub diff_path: Option<PathBuf>,
}

impl TestResult {
    /// Check if this is a new baseline
    pub fn is_new_baseline(&self) -> bool {
        self.baseline_path.is_none()
    }
}

/// Diff result
#[derive(Debug, Clone)]
pub struct DiffResult {
    /// Difference percentage
    pub diff_percent: f32,
    /// Total different pixels
    pub diff_pixels: usize,
    /// Total pixels compared
    pub total_pixels: usize,
    /// Diff image data
    pub diff_image: Vec<u8>,
    /// Diff image path
    pub path: PathBuf,
    /// Mismatched regions
    pub regions: Vec<DiffRegion>,
    /// SSIM score (structural similarity)
    pub ssim: f32,
}

impl DiffResult {
    /// Check if difference exceeds threshold
    pub fn exceeds_threshold(&self, threshold: f32) -> bool {
        self.diff_percent > threshold
    }
}

/// A region of difference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffRegion {
    /// X coordinate
    pub x: u32,
    /// Y coordinate
    pub y: u32,
    /// Width
    pub width: u32,
    /// Height
    pub height: u32,
    /// Difference percentage within region
    pub diff_percent: f32,
}

/// Visual regression error
#[derive(Debug, thiserror::Error)]
pub enum VRError {
    #[error("Capture error: {0}")]
    CaptureError(String),
    
    #[error("Comparison error: {0}")]
    ComparisonError(String),
    
    #[error("Baseline not found: {0}")]
    BaselineNotFound(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Image error: {0}")]
    ImageError(String),
    
    #[error("Invalid viewport: {0}")]
    InvalidViewport(String),
    
    #[error("Test timeout: {0}")]
    Timeout(String),
    
    #[error("Navigation error: {0}")]
    NavigationError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Test scenario definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestScenario {
    /// Scenario name
    pub name: String,
    /// URL to test
    pub url: String,
    /// Viewports to test
    pub viewports: Vec<String>,
    /// Selectors to wait for
    pub selectors: Vec<String>,
    /// Actions to perform before capture
    pub actions: Vec<TestAction>,
    /// Skip conditions
    pub skip_if: Vec<SkipCondition>,
}

/// Actions to perform before screenshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestAction {
    /// Click on element
    Click { selector: String },
    /// Type text into element
    Type { selector: String, text: String },
    /// Scroll to position
    Scroll { x: u32, y: u32 },
    /// Scroll to element
    ScrollTo { selector: String },
    /// Wait for duration
    Wait { ms: u64 },
    /// Hover over element
    Hover { selector: String },
    /// Focus element
    Focus { selector: String },
    /// Execute JavaScript
    ExecuteScript { script: String },
    /// Set viewport size
    SetViewport { width: u32, height: u32 },
    /// Emulate device
    EmulateDevice { name: String },
}

/// Conditions to skip test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkipCondition {
    /// Skip on specific browser
    Browser(String),
    /// Skip on specific OS
    OS(String),
    /// Skip if environment variable is set
    EnvVar(String),
    /// Skip if URL returns specific status
    HttpStatus(u16),
    /// Custom skip condition
    Custom(String),
}

/// Comparison method
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ComparisonMethod {
    /// Pixel-by-pixel comparison
    PixelDiff,
    /// Perceptual difference
    PerceptualDiff,
    /// Structural similarity (SSIM)
    SSIM,
    /// Feature-based comparison
    FeatureDiff,
}

/// Diff configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffConfig {
    /// Comparison method
    pub method: ComparisonMethod,
    /// Pixel threshold (0-255)
    pub pixel_threshold: u8,
    /// Perceptual threshold (0.0-1.0)
    pub perceptual_threshold: f32,
    /// SSIM threshold (0.0-1.0)
    pub ssim_threshold: f32,
    /// Enable anti-aliasing detection
    pub detect_antialiasing: bool,
    /// Ignore regions
    pub ignore_regions: Vec<IgnoreRegion>,
    /// Highlight color for diffs
    pub highlight_color: [u8; 4],
}

impl Default for DiffConfig {
    fn default() -> Self {
        Self {
            method: ComparisonMethod::PerceptualDiff,
            pixel_threshold: 0,
            perceptual_threshold: 0.05,
            ssim_threshold: 0.95,
            detect_antialiasing: true,
            ignore_regions: Vec::new(),
            highlight_color: [255, 0, 255, 255], // Magenta
        }
    }
}

/// Region to ignore during comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnoreRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub reason: Option<String>,
}

/// Report format
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ReportFormat {
    HTML,
    JSON,
    JUnit,
    Markdown,
}

/// Test report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestReport {
    /// Report title
    pub title: String,
    /// Generation timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Total tests
    pub total_tests: usize,
    /// Passed tests
    pub passed: usize,
    /// Failed tests
    pub failed: usize,
    /// New baselines
    pub new_baselines: usize,
    /// Test results
    pub results: Vec<TestResult>,
    /// Environment info
    pub environment: EnvironmentInfo,
}

/// Environment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    pub os: String,
    pub browser: String,
    pub browser_version: String,
    pub viewport: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}