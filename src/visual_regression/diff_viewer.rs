//! Interactive diff viewer for visual regression testing

use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};

use super::models::*;
use super::{Screenshot, Baseline, DiffResult, VRError};

/// Diff viewer for interactive comparison
pub struct DiffViewer {
    /// Output directory for viewer files
    output_dir: PathBuf,
    /// Viewer configuration
    config: ViewerConfig,
}

/// Viewer configuration
#[derive(Debug, Clone)]
pub struct ViewerConfig {
    /// Show side by side
    pub side_by_side: bool,
    /// Show overlay (slider)
    pub overlay: bool,
    /// Show heatmap
    pub heatmap: bool,
    /// Highlight diff regions
    pub highlight_regions: bool,
    /// Zoom level
    pub zoom: f32,
    /// Background color
    pub background_color: String,
}

impl Default for ViewerConfig {
    fn default() -> Self {
        Self {
            side_by_side: true,
            overlay: true,
            heatmap: true,
            highlight_regions: true,
            zoom: 1.0,
            background_color: "#1a1a1a".to_string(),
        }
    }
}

impl DiffViewer {
    /// Create a new diff viewer
    pub fn new(output_dir: PathBuf) -> Self {
        Self {
            output_dir,
            config: ViewerConfig::default(),
        }
    }
    
    /// Create viewer with configuration
    pub fn with_config(output_dir: PathBuf, config: ViewerConfig) -> Self {
        Self { output_dir, config }
    }
    
    /// Generate interactive HTML viewer
    pub async fn generate_viewer(
        &self,
        baseline: &Baseline,
        screenshot: &Screenshot,
        diff: &DiffResult,
    ) -> Result<PathBuf, VRError> {
        tokio::fs::create_dir_all(&self.output_dir).await?;
        
        let html = self.build_html(baseline, screenshot, diff)?;
        
        let filename = format!("diff_{}_{}.html", baseline.name, baseline.viewport);
        let path = self.output_dir.join(&filename);
        
        tokio::fs::write(&path, &html).await?;
        
        Ok(path)
    }
    
    /// Generate comparison report
    pub async fn generate_report(
        &self,
        results: &[TestResult],
    ) -> Result<PathBuf, VRError> {
        tokio::fs::create_dir_all(&self.output_dir).await?;
        
        let html = self.build_report_html(results)?;
        
        let path = self.output_dir.join("report.html");
        tokio::fs::write(&path, &html).await?;
        
        Ok(path)
    }
    
    /// Build HTML for single diff viewer
    fn build_html(
        &self,
        baseline: &Baseline,
        screenshot: &Screenshot,
        diff: &DiffResult,
    ) -> Result<String, VRError> {
        let html = format!(r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Visual Diff: {} ({})</title>
    <style>
        :root {{
            --bg-color: {};
            --text-color: #ffffff;
            --accent-color: #dc143c;
            --panel-bg: #2a2a2a;
        }}
        
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        
        body {{
            background: var(--bg-color);
            color: var(--text-color);
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            min-height: 100vh;
            padding: 20px;
        }}
        
        .header {{
            text-align: center;
            margin-bottom: 30px;
        }}
        
        .header h1 {{
            font-size: 24px;
            margin-bottom: 10px;
        }}
        
        .stats {{
            display: flex;
            justify-content: center;
            gap: 30px;
            margin-bottom: 20px;
        }}
        
        .stat {{
            background: var(--panel-bg);
            padding: 15px 25px;
            border-radius: 8px;
            text-align: center;
        }}
        
        .stat-value {{
            font-size: 28px;
            font-weight: bold;
            color: {};
        }}
        
        .stat-label {{
            font-size: 12px;
            color: #888;
            text-transform: uppercase;
        }}
        
        .viewer-container {{
            display: flex;
            gap: 20px;
            justify-content: center;
            flex-wrap: wrap;
        }}
        
        .image-panel {{
            background: var(--panel-bg);
            border-radius: 8px;
            overflow: hidden;
        }}
        
        .image-panel h3 {{
            padding: 15px;
            background: rgba(0,0,0,0.3);
            font-size: 14px;
        }}
        
        .image-container {{
            position: relative;
            overflow: auto;
            max-height: 600px;
        }}
        
        .image-container img {{
            display: block;
            max-width: 100%;
        }}
        
        .overlay-container {{
            position: relative;
            overflow: hidden;
        }}
        
        .overlay-slider {{
            position: absolute;
            top: 0;
            left: 50%;
            width: 4px;
            height: 100%;
            background: var(--accent-color);
            cursor: ew-resize;
            z-index: 10;
        }}
        
        .overlay-slider::before {{
            content: '';
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            width: 40px;
            height: 40px;
            background: var(--accent-color);
            border-radius: 50%;
        }}
        
        .regions-overlay {{
            position: absolute;
            top: 0;
            left: 0;
            pointer-events: none;
        }}
        
        .region {{
            position: absolute;
            border: 2px solid var(--accent-color);
            background: rgba(220, 20, 60, 0.2);
        }}
        
        .tabs {{
            display: flex;
            gap: 10px;
            margin-bottom: 20px;
            justify-content: center;
        }}
        
        .tab {{
            padding: 10px 20px;
            background: var(--panel-bg);
            border: none;
            border-radius: 6px;
            color: var(--text-color);
            cursor: pointer;
            transition: all 0.2s;
        }}
        
        .tab:hover {{
            background: #3a3a3a;
        }}
        
        .tab.active {{
            background: var(--accent-color);
        }}
        
        .view {{
            display: none;
        }}
        
        .view.active {{
            display: block;
        }}
        
        .legend {{
            display: flex;
            gap: 20px;
            justify-content: center;
            margin-top: 20px;
            font-size: 14px;
        }}
        
        .legend-item {{
            display: flex;
            align-items: center;
            gap: 8px;
        }}
        
        .legend-color {{
            width: 20px;
            height: 20px;
            border-radius: 4px;
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1>Visual Diff: {}</h1>
        <p>Viewport: {}</p>
    </div>
    
    <div class="stats">
        <div class="stat">
            <div class="stat-value">{:.2}%</div>
            <div class="stat-label">Diff Percentage</div>
        </div>
        <div class="stat">
            <div class="stat-value">{}</div>
            <div class="stat-label">Diff Pixels</div>
        </div>
        <div class="stat">
            <div class="stat-value">{:.3}</div>
            <div class="stat-label">SSIM Score</div>
        </div>
        <div class="stat">
            <div class="stat-value">{}</div>
            <div class="stat-label">Diff Regions</div>
        </div>
    </div>
    
    <div class="tabs">
        <button class="tab active" onclick="showView('side-by-side')">Side by Side</button>
        <button class="tab" onclick="showView('overlay')">Overlay</button>
        <button class="tab" onclick="showView('diff')">Diff Only</button>
        <button class="tab" onclick="showView('heatmap')">Heatmap</button>
    </div>
    
    <div id="side-by-side" class="view active">
        <div class="viewer-container">
            <div class="image-panel">
                <h3>Baseline</h3>
                <div class="image-container">
                    <div style="width: {}px; height: {}px; background: #333; display: flex; align-items: center; justify-content: center;">Baseline Image</div>
                </div>
            </div>
            <div class="image-panel">
                <h3>Current</h3>
                <div class="image-container">
                    <div style="width: {}px; height: {}px; background: #333; display: flex; align-items: center; justify-content: center;">Current Image</div>
                </div>
            </div>
        </div>
    </div>
    
    <div id="overlay" class="view">
        <div class="viewer-container">
            <div class="image-panel">
                <h3>Overlay Comparison</h3>
                <div class="image-container overlay-container">
                    <div style="width: {}px; height: {}px; background: linear-gradient(90deg, #333 50%, #444 50%); display: flex; align-items: center; justify-content: center;">Drag slider to compare</div>
                    <div class="overlay-slider"></div>
                </div>
            </div>
        </div>
    </div>
    
    <div id="diff" class="view">
        <div class="viewer-container">
            <div class="image-panel">
                <h3>Difference</h3>
                <div class="image-container">
                    <div style="width: {}px; height: {}px; background: #000; display: flex; align-items: center; justify-content: center;">Diff Image (Magenta = Difference)</div>
                </div>
            </div>
        </div>
    </div>
    
    <div id="heatmap" class="view">
        <div class="viewer-container">
            <div class="image-panel">
                <h3>Diff Heatmap</h3>
                <div class="image-container">
                    <div style="width: {}px; height: {}px; background: linear-gradient(90deg, blue, green, yellow, red); display: flex; align-items: center; justify-content: center;">Heatmap View</div>
                </div>
            </div>
        </div>
    </div>
    
    <div class="legend">
        <div class="legend-item">
            <div class="legend-color" style="background: blue;"></div>
            <span>Low Diff</span>
        </div>
        <div class="legend-item">
            <div class="legend-color" style="background: green;"></div>
            <span>Medium-Low</span>
        </div>
        <div class="legend-item">
            <div class="legend-color" style="background: yellow;"></div>
            <span>Medium-High</span>
        </div>
        <div class="legend-item">
            <div class="legend-color" style="background: red;"></div>
            <span>High Diff</span>
        </div>
    </div>
    
    <script>
        function showView(viewId) {{
            document.querySelectorAll('.view').forEach(v => v.classList.remove('active'));
            document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
            document.getElementById(viewId).classList.add('active');
            event.target.classList.add('active');
        }}
    </script>
</body>
</html>"##,
            baseline.name,
            baseline.viewport,
            self.config.background_color,
            if diff.diff_percent > 5.0 { "#dc143c" } else { "#4caf50" },
            baseline.name,
            baseline.viewport,
            diff.diff_percent,
            diff.diff_pixels,
            diff.ssim,
            diff.regions.len(),
            baseline.screenshot.width,
            baseline.screenshot.height,
            screenshot.width,
            screenshot.height,
            screenshot.width,
            screenshot.height,
            screenshot.width,
            screenshot.height,
            screenshot.width,
            screenshot.height
        );
        
        Ok(html)
    }
    
    /// Build HTML for report
    fn build_report_html(&self, results: &[TestResult]) -> Result<String, VRError> {
        let total = results.len();
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = total - passed;
        
        let results_html: String = results.iter().map(|r| {
            format!(
                r##"<tr class="{}">
                    <td>{}</td>
                    <td>{}</td>
                    <td>{:.2}%</td>
                    <td>{}</td>
                </tr>"##,
                if r.passed { "passed" } else { "failed" },
                r.test_name,
                r.viewport,
                r.diff_percent,
                if r.passed { "✓ PASS" } else { "✗ FAIL" }
            )
        }).collect();
        
        let html = format!(r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Visual Regression Report</title>
    <style>
        body {{
            background: #1a1a1a;
            color: #fff;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            padding: 20px;
        }}
        .summary {{
            display: flex;
            gap: 20px;
            margin-bottom: 30px;
        }}
        .summary-card {{
            background: #2a2a2a;
            padding: 20px 30px;
            border-radius: 8px;
            text-align: center;
        }}
        .summary-card h3 {{
            font-size: 36px;
            margin-bottom: 5px;
        }}
        .summary-card p {{
            color: #888;
        }}
        .passed {{ color: #4caf50; }}
        .failed {{ color: #dc143c; }}
        table {{
            width: 100%;
            border-collapse: collapse;
            background: #2a2a2a;
            border-radius: 8px;
            overflow: hidden;
        }}
        th, td {{
            padding: 15px;
            text-align: left;
            border-bottom: 1px solid #3a3a3a;
        }}
        th {{
            background: rgba(0,0,0,0.3);
            text-transform: uppercase;
            font-size: 12px;
            color: #888;
        }}
        tr.passed {{ background: rgba(76, 175, 80, 0.1); }}
        tr.failed {{ background: rgba(220, 20, 60, 0.1); }}
    </style>
</head>
<body>
    <h1>Visual Regression Test Report</h1>
    
    <div class="summary">
        <div class="summary-card">
            <h3>{}</h3>
            <p>Total Tests</p>
        </div>
        <div class="summary-card passed">
            <h3>{}</h3>
            <p>Passed</p>
        </div>
        <div class="summary-card failed">
            <h3>{}</h3>
            <p>Failed</p>
        </div>
        <div class="summary-card">
            <h3>{:.1}%</h3>
            <p>Pass Rate</p>
        </div>
    </div>
    
    <table>
        <thead>
            <tr>
                <th>Test Name</th>
                <th>Viewport</th>
                <th>Diff %</th>
                <th>Status</th>
            </tr>
        </thead>
        <tbody>
            {}
        </tbody>
    </table>
</body>
</html>"##,
            total,
            passed,
            failed,
            if total > 0 { (passed as f32 / total as f32) * 100.0 } else { 0.0 },
            results_html
        );
        
        Ok(html)
    }
    
    /// Generate JSON report
    pub async fn generate_json_report(
        &self,
        results: &[TestResult],
    ) -> Result<PathBuf, VRError> {
        tokio::fs::create_dir_all(&self.output_dir).await?;
        
        let report = TestReport {
            title: "Visual Regression Report".to_string(),
            timestamp: chrono::Utc::now(),
            total_tests: results.len(),
            passed: results.iter().filter(|r| r.passed).count(),
            failed: results.iter().filter(|r| !r.passed).count(),
            new_baselines: results.iter().filter(|r| r.is_new_baseline()).count(),
            results: results.to_vec(),
            environment: EnvironmentInfo {
                os: std::env::consts::OS.to_string(),
                browser: "Chromium".to_string(),
                browser_version: "120.0".to_string(),
                viewport: "multiple".to_string(),
                timestamp: chrono::Utc::now(),
            },
        };
        
        let json = serde_json::to_string_pretty(&report)?;
        let path = self.output_dir.join("report.json");
        tokio::fs::write(&path, &json).await?;
        
        Ok(path)
    }
    
    /// Generate Markdown report
    pub async fn generate_markdown_report(
        &self,
        results: &[TestResult],
    ) -> Result<PathBuf, VRError> {
        tokio::fs::create_dir_all(&self.output_dir).await?;
        
        let total = results.len();
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = total - passed;
        
        let mut md = format!(
            r##"# Visual Regression Test Report

## Summary

| Metric | Value |
|--------|-------|
| Total Tests | {} |
| Passed | {} |
| Failed | {} |
| Pass Rate | {:.1}% |

## Results

| Test | Viewport | Diff % | Status |
|------|----------|--------|--------|
"##,
            total, passed, failed,
            if total > 0 { (passed as f32 / total as f32) * 100.0 } else { 0.0 }
        );
        
        for result in results {
            let status = if result.passed { "✅ PASS" } else { "❌ FAIL" };
            md.push_str(&format!(
                "| {} | {} | {:.2}% | {} |\n",
                result.test_name,
                result.viewport,
                result.diff_percent,
                status
            ));
        }
        
        let path = self.output_dir.join("report.md");
        tokio::fs::write(&path, &md).await?;
        
        Ok(path)
    }
}