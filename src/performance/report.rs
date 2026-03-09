//! Performance report generator

use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};

use super::models::*;
use super::TestExecution;
use super::PerfError;

/// Report generator
pub struct ReportGenerator {
    output_dir: PathBuf,
}

impl ReportGenerator {
    /// Create a new report generator
    pub fn new(output_dir: PathBuf) -> Self {
        Self { output_dir }
    }
    
    /// Generate performance report
    pub async fn generate(&self, executions: &[TestExecution]) -> Result<PathBuf, PerfError> {
        tokio::fs::create_dir_all(&self.output_dir).await?;
        
        let report = self.build_report(executions);
        let html = self.render_html(&report);
        
        let path = self.output_dir.join("performance-report.html");
        tokio::fs::write(&path, &html).await?;
        
        Ok(path)
    }
    
    /// Generate JSON report
    pub async fn generate_json(&self, executions: &[TestExecution]) -> Result<PathBuf, PerfError> {
        tokio::fs::create_dir_all(&self.output_dir).await?;
        
        let report = self.build_report(executions);
        let json = serde_json::to_string_pretty(&report)?;
        
        let path = self.output_dir.join("performance-report.json");
        tokio::fs::write(&path, &json).await?;
        
        Ok(path)
    }
    
    /// Generate Markdown report
    pub async fn generate_markdown(&self, executions: &[TestExecution]) -> Result<PathBuf, PerfError> {
        tokio::fs::create_dir_all(&self.output_dir).await?;
        
        let report = self.build_report(executions);
        let md = self.render_markdown(&report);
        
        let path = self.output_dir.join("performance-report.md");
        tokio::fs::write(&path, &md).await?;
        
        Ok(path)
    }
    
    /// Build report data structure
    fn build_report(&self, executions: &[TestExecution]) -> PerformanceReport {
        let total = executions.len();
        let passed = executions.iter().filter(|e| e.status == super::TestStatus::Completed).count();
        let failed = total - passed;
        
        let mut load_tests = Vec::new();
        let mut stress_tests = Vec::new();
        let mut benchmarks = Vec::new();
        
        for execution in executions {
            match execution.test_type {
                super::TestType::LoadTest => load_tests.push(execution.clone()),
                super::TestType::StressTest => stress_tests.push(execution.clone()),
                super::TestType::Benchmark => benchmarks.push(execution.clone()),
                super::TestType::Profile => {}
            }
        }
        
        PerformanceReport {
            title: "Performance Test Report".to_string(),
            generated_at: chrono::Utc::now(),
            summary: ReportSummary {
                total_tests: total,
                passed,
                failed,
                duration_ms: executions.iter().map(|e| e.duration_ms).sum(),
            },
            load_tests,
            stress_tests,
            benchmarks,
        }
    }
    
    /// Render HTML report
    fn render_html(&self, report: &PerformanceReport) -> String {
        format!(r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
        :root {{
            --bg-color: #1a1a1a;
            --text-color: #ffffff;
            --accent-color: #dc143c;
            --panel-bg: #2a2a2a;
            --success: #4caf50;
            --warning: #ff9800;
            --error: #dc143c;
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
            padding: 20px;
        }}
        
        .header {{
            text-align: center;
            margin-bottom: 30px;
        }}
        
        .header h1 {{
            font-size: 28px;
            margin-bottom: 10px;
        }}
        
        .header .timestamp {{
            color: #888;
        }}
        
        .summary {{
            display: flex;
            gap: 20px;
            justify-content: center;
            margin-bottom: 40px;
        }}
        
        .summary-card {{
            background: var(--panel-bg);
            padding: 20px 30px;
            border-radius: 8px;
            text-align: center;
            min-width: 150px;
        }}
        
        .summary-card h3 {{
            font-size: 32px;
            margin-bottom: 5px;
        }}
        
        .summary-card p {{
            color: #888;
            font-size: 12px;
            text-transform: uppercase;
        }}
        
        .summary-card.passed h3 {{ color: var(--success); }}
        .summary-card.failed h3 {{ color: var(--error); }}
        
        .section {{
            margin-bottom: 30px;
        }}
        
        .section h2 {{
            margin-bottom: 15px;
            padding-bottom: 10px;
            border-bottom: 1px solid #333;
        }}
        
        table {{
            width: 100%;
            border-collapse: collapse;
            background: var(--panel-bg);
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
        
        .status-passed {{ color: var(--success); }}
        .status-failed {{ color: var(--error); }}
        
        .metrics {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 15px;
        }}
        
        .metric {{
            background: var(--panel-bg);
            padding: 15px;
            border-radius: 8px;
        }}
        
        .metric-label {{
            font-size: 12px;
            color: #888;
            margin-bottom: 5px;
        }}
        
        .metric-value {{
            font-size: 24px;
            font-weight: bold;
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1>{title}</h1>
        <p class="timestamp">Generated: {timestamp}</p>
    </div>
    
    <div class="summary">
        <div class="summary-card">
            <h3>{total_tests}</h3>
            <p>Total Tests</p>
        </div>
        <div class="summary-card passed">
            <h3>{passed}</h3>
            <p>Passed</p>
        </div>
        <div class="summary-card failed">
            <h3>{failed}</h3>
            <p>Failed</p>
        </div>
        <div class="summary-card">
            <h3>{duration}</h3>
            <p>Duration</p>
        </div>
    </div>
    
    <div class="section">
        <h2>Load Tests</h2>
        <table>
            <thead>
                <tr>
                    <th>Test Name</th>
                    <th>Status</th>
                    <th>Duration</th>
                    <th>Requests</th>
                    <th>Avg Response</th>
                    <th>Error Rate</th>
                </tr>
            </thead>
            <tbody>
                {load_test_rows}
            </tbody>
        </table>
    </div>
    
    <div class="section">
        <h2>Stress Tests</h2>
        <table>
            <thead>
                <tr>
                    <th>Test Name</th>
                    <th>Breaking Point</th>
                    <th>Max Sustained</th>
                    <th>Status</th>
                </tr>
            </thead>
            <tbody>
                {stress_test_rows}
            </tbody>
        </table>
    </div>
    
    <div class="section">
        <h2>Benchmarks</h2>
        <table>
            <thead>
                <tr>
                    <th>Benchmark</th>
                    <th>Mean (ns)</th>
                    <th>Std Dev</th>
                    <th>Throughput</th>
                </tr>
            </thead>
            <tbody>
                {benchmark_rows}
            </tbody>
        </table>
    </div>
</body>
</html>"##,
            title = report.title,
            timestamp = report.generated_at.format("%Y-%m-%d %H:%M:%S UTC"),
            total_tests = report.summary.total_tests,
            passed = report.summary.passed,
            failed = report.summary.failed,
            duration = format!("{}ms", report.summary.duration_ms),
            load_test_rows = self.render_load_test_rows(&report.load_tests),
            stress_test_rows = self.render_stress_test_rows(&report.stress_tests),
            benchmark_rows = self.render_benchmark_rows(&report.benchmarks),
        )
    }
    
    fn render_load_test_rows(&self, tests: &[TestExecution]) -> String {
        tests.iter().map(|t| {
            format!(
                r##"<tr>
                    <td>{}</td>
                    <td class="status-{}">{}</td>
                    <td>{}ms</td>
                    <td>-</td>
                    <td>-</td>
                    <td>-</td>
                </tr>"##,
                t.test_id,
                if t.status == super::TestStatus::Completed { "passed" } else { "failed" },
                if t.status == super::TestStatus::Completed { "✓ Pass" } else { "✗ Fail" },
                t.duration_ms
            )
        }).collect::<Vec<_>>().join("\n")
    }
    
    fn render_stress_test_rows(&self, tests: &[TestExecution]) -> String {
        tests.iter().map(|t| {
            format!(
                r##"<tr>
                    <td>{}</td>
                    <td>-</td>
                    <td>-</td>
                    <td class="status-{}">{}</td>
                </tr>"##,
                t.test_id,
                if t.status == super::TestStatus::Completed { "passed" } else { "failed" },
                if t.status == super::TestStatus::Completed { "✓ Pass" } else { "✗ Fail" }
            )
        }).collect::<Vec<_>>().join("\n")
    }
    
    fn render_benchmark_rows(&self, tests: &[TestExecution]) -> String {
        tests.iter().map(|t| {
            format!(
                r##"<tr>
                    <td>{}</td>
                    <td>-</td>
                    <td>-</td>
                    <td>-</td>
                </tr>"##,
                t.test_id
            )
        }).collect::<Vec<_>>().join("\n")
    }
    
    /// Render Markdown report
    fn render_markdown(&self, report: &PerformanceReport) -> String {
        format!(
            r##"# {title}

Generated: {timestamp}

## Summary

| Metric | Value |
|--------|-------|
| Total Tests | {total_tests} |
| Passed | {passed} |
| Failed | {failed} |
| Duration | {duration} |

## Load Tests

| Test Name | Status | Duration |
|-----------|--------|----------|
{load_tests}

## Stress Tests

| Test Name | Status |
|-----------|--------|
{stress_tests}

## Benchmarks

| Benchmark | Status |
|-----------|--------|
{benchmarks}
"##,
            title = report.title,
            timestamp = report.generated_at.format("%Y-%m-%d %H:%M:%S UTC"),
            total_tests = report.summary.total_tests,
            passed = report.summary.passed,
            failed = report.summary.failed,
            duration = format!("{}ms", report.summary.duration_ms),
            load_tests = report.load_tests.iter()
                .map(|t| format!("| {} | {} | {}ms |", t.test_id, 
                    if t.status == super::TestStatus::Completed { "✓" } else { "✗" },
                    t.duration_ms))
                .collect::<Vec<_>>()
                .join("\n"),
            stress_tests = report.stress_tests.iter()
                .map(|t| format!("| {} | {} |", t.test_id,
                    if t.status == super::TestStatus::Completed { "✓" } else { "✗" }))
                .collect::<Vec<_>>()
                .join("\n"),
            benchmarks = report.benchmarks.iter()
                .map(|t| format!("| {} | {} |", t.test_id,
                    if t.status == super::TestStatus::Completed { "✓" } else { "✗" }))
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }
}

/// Performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub title: String,
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub summary: ReportSummary,
    pub load_tests: Vec<TestExecution>,
    pub stress_tests: Vec<TestExecution>,
    pub benchmarks: Vec<TestExecution>,
}

/// Report summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub duration_ms: u64,
}

/// Report configuration
#[derive(Debug, Clone)]
pub struct ReportConfig {
    pub include_charts: bool,
    pub include_flame_graphs: bool,
    pub include_memory_analysis: bool,
    pub compare_baseline: Option<PathBuf>,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            include_charts: true,
            include_flame_graphs: false,
            include_memory_analysis: true,
            compare_baseline: None,
        }
    }
}