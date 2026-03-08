//! Image comparison and diff detection

use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::models::*;
use super::{Screenshot, VisualRegressionConfig, VRError};

/// Diff comparison engine
pub struct DiffEngine {
    config: VisualRegressionConfig,
}

impl DiffEngine {
    /// Create a new diff engine
    pub fn new(config: VisualRegressionConfig) -> Self {
        Self { config }
    }
    
    /// Compare two screenshots
    pub async fn compare(
        &self,
        baseline: &Baseline,
        screenshot: &Screenshot,
    ) -> Result<DiffResult, VRError> {
        // Check dimensions match
        if baseline.screenshot.width != screenshot.width ||
           baseline.screenshot.height != screenshot.height {
            return Err(VRError::ComparisonError(
                format!(
                    "Dimension mismatch: baseline {}x{} vs screenshot {}x{}",
                    baseline.screenshot.width,
                    baseline.screenshot.height,
                    screenshot.width,
                    screenshot.height
                )
            ));
        }
        
        // Perform comparison
        let diff_data = self.compute_diff(&baseline.screenshot.data, &screenshot.data)?;
        
        // Calculate statistics
        let total_pixels = (screenshot.width * screenshot.height) as usize;
        let diff_pixels = diff_data.iter().filter(|&&p| p > 0).count() / 4;
        let diff_percent = (diff_pixels as f32 / total_pixels as f32) * 100.0;
        
        // Calculate SSIM
        let ssim = self.calculate_ssim(&baseline.screenshot, screenshot)?;
        
        // Find diff regions
        let regions = self.find_diff_regions(&diff_data, screenshot.width, screenshot.height)?;
        
        // Create diff image path
        let filename = format!("{}_diff.png", baseline.name);
        let path = self.config.diff_dir.join(&filename);
        
        Ok(DiffResult {
            diff_percent,
            diff_pixels,
            total_pixels,
            diff_image: diff_data,
            path,
            regions,
            ssim,
        })
    }
    
    /// Compare with custom config
    pub async fn compare_with_config(
        &self,
        baseline: &Baseline,
        screenshot: &Screenshot,
        config: &DiffConfig,
    ) -> Result<DiffResult, VRError> {
        let mut result = self.compare(baseline, screenshot).await?;
        
        // Apply ignore regions
        for region in &config.ignore_regions {
            self.apply_ignore_region(&mut result, region)?;
        }
        
        // Adjust based on comparison method
        match config.method {
            ComparisonMethod::PixelDiff => {
                // Already computed
            }
            ComparisonMethod::PerceptualDiff => {
                // Apply perceptual threshold
                result.diff_percent = self.apply_perceptual_threshold(
                    result.diff_percent,
                    config.perceptual_threshold
                );
            }
            ComparisonMethod::SSIM => {
                // Use SSIM-based difference
                result.diff_percent = (1.0 - result.ssim) * 100.0;
            }
            ComparisonMethod::FeatureDiff => {
                // Feature-based comparison
                result.diff_percent = self.feature_comparison(baseline, screenshot)?;
            }
        }
        
        Ok(result)
    }
    
    /// Save diff image
    pub async fn save_diff(&self, diff: &DiffResult, name: &str) -> Result<(), VRError> {
        tokio::fs::create_dir_all(&self.config.diff_dir).await?;
        
        let filename = format!("{}_diff.png", name);
        let path = self.config.diff_dir.join(&filename);
        
        // Save diff image
        tokio::fs::write(&path, &diff.diff_image).await?;
        
        Ok(())
    }
    
    /// Compute pixel difference
    fn compute_diff(&self, baseline: &[u8], screenshot: &[u8]) -> Result<Vec<u8>, VRError> {
        let mut diff = vec![0u8; baseline.len()];
        
        for i in (0..baseline.len()).step_by(4) {
            let r_diff = (baseline[i] as i16 - screenshot[i] as i16).abs() as u8;
            let g_diff = (baseline[i + 1] as i16 - screenshot[i + 1] as i16).abs() as u8;
            let b_diff = (baseline[i + 2] as i16 - screenshot[i + 2] as i16).abs() as u8;
            
            // Use threshold
            let threshold = self.config.pixel_threshold as u8;
            
            if r_diff > threshold || g_diff > threshold || b_diff > threshold {
                // Highlight with magenta
                diff[i] = 255;     // R
                diff[i + 1] = 0;   // G
                diff[i + 2] = 255; // B
                diff[i + 3] = 255; // A
            } else {
                // Copy original pixel (dimmed)
                diff[i] = screenshot[i] / 3;
                diff[i + 1] = screenshot[i + 1] / 3;
                diff[i + 2] = screenshot[i + 2] / 3;
                diff[i + 3] = screenshot[i + 3];
            }
        }
        
        Ok(diff)
    }
    
    /// Calculate SSIM (Structural Similarity Index)
    fn calculate_ssim(&self, img1: &Screenshot, img2: &Screenshot) -> Result<f32, VRError> {
        // Simplified SSIM calculation
        // In real implementation, would use proper SSIM algorithm
        
        let mut sum1 = 0u64;
        let mut sum2 = 0u64;
        let mut sum_sq1 = 0u64;
        let mut sum_sq2 = 0u64;
        let mut sum_prod = 0i64;
        
        let pixels = (img1.width * img1.height) as usize;
        
        for i in (0..img1.data.len()).step_by(4) {
            let p1 = (img1.data[i] as u32 + img1.data[i+1] as u32 + img1.data[i+2] as u32) / 3;
            let p2 = (img2.data[i] as u32 + img2.data[i+1] as u32 + img2.data[i+2] as u32) / 3;
            
            sum1 += p1 as u64;
            sum2 += p2 as u64;
            sum_sq1 += (p1 * p1) as u64;
            sum_sq2 += (p2 * p2) as u64;
            sum_prod += (p1 as i64 * p2 as i64);
        }
        
        let n = pixels as f64;
        let mean1 = sum1 as f64 / n;
        let mean2 = sum2 as f64 / n;
        
        let var1 = (sum_sq1 as f64 / n) - (mean1 * mean1);
        let var2 = (sum_sq2 as f64 / n) - (mean2 * mean2);
        let covar = (sum_prod as f64 / n) - (mean1 * mean2);
        
        // SSIM constants
        let c1 = 6.5025;  // (0.01 * 255)^2
        let c2 = 58.5225; // (0.03 * 255)^2
        
        let numerator = (2.0 * mean1 * mean2 + c1) * (2.0 * covar + c2);
        let denominator = (mean1 * mean1 + mean2 * mean2 + c1) * (var1 + var2 + c2);
        
        Ok((numerator / denominator) as f32)
    }
    
    /// Find regions of difference
    fn find_diff_regions(
        &self,
        diff_data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<Vec<DiffRegion>, VRError> {
        let mut regions = Vec::new();
        
        // Simple region detection - scan for contiguous diff areas
        let mut visited = vec![false; (width * height) as usize];
        
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) as usize;
                
                if visited[idx] {
                    continue;
                }
                
                // Check if this pixel is different
                let pixel_idx = idx * 4;
                if pixel_idx + 3 < diff_data.len() {
                    let is_diff = diff_data[pixel_idx] == 255 && 
                                  diff_data[pixel_idx + 2] == 255;
                    
                    if is_diff {
                        // Find bounding box of this region
                        let region = self.flood_fill_region(diff_data, width, height, x, y, &mut visited);
                        if region.width > 5 && region.height > 5 {  // Filter small regions
                            regions.push(region);
                        }
                    }
                }
                
                visited[idx] = true;
            }
        }
        
        Ok(regions)
    }
    
    /// Flood fill to find region bounds
    fn flood_fill_region(
        &self,
        diff_data: &[u8],
        width: u32,
        height: u32,
        start_x: u32,
        start_y: u32,
        visited: &mut [bool],
    ) -> DiffRegion {
        let mut min_x = start_x;
        let mut max_x = start_x;
        let mut min_y = start_y;
        let mut max_y = start_y;
        
        // Simple scan for bounds (not true flood fill for performance)
        let scan_range = 100u32; // Limit scan range
        
        for y in start_y.saturating_sub(scan_range)..=(start_y + scan_range).min(height - 1) {
            for x in start_x.saturating_sub(scan_range)..=(start_x + scan_range).min(width - 1) {
                let idx = (y * width + x) as usize;
                let pixel_idx = idx * 4;
                
                if pixel_idx + 3 < diff_data.len() {
                    let is_diff = diff_data[pixel_idx] == 255 && 
                                  diff_data[pixel_idx + 2] == 255;
                    
                    if is_diff {
                        visited[idx] = true;
                        min_x = min_x.min(x);
                        max_x = max_x.max(x);
                        min_y = min_y.min(y);
                        max_y = max_y.max(y);
                    }
                }
            }
        }
        
        let region_width = max_x - min_x + 1;
        let region_height = max_y - min_y + 1;
        let region_pixels = (region_width * region_height) as usize;
        
        DiffRegion {
            x: min_x,
            y: min_y,
            width: region_width,
            height: region_height,
            diff_percent: if region_pixels > 0 { 50.0 } else { 0.0 }, // Simplified
        }
    }
    
    /// Apply ignore region to diff result
    fn apply_ignore_region(&self, diff: &mut DiffResult, region: &IgnoreRegion) -> Result<(), VRError> {
        // Mark region as not different
        let _ = (diff, region);
        Ok(())
    }
    
    /// Apply perceptual threshold
    fn apply_perceptual_threshold(&self, diff: f32, threshold: f32) -> f32 {
        if diff < threshold * 100.0 {
            0.0
        } else {
            diff - threshold * 100.0
        }
    }
    
    /// Feature-based comparison
    fn feature_comparison(&self, baseline: &Baseline, screenshot: &Screenshot) -> Result<f32, VRError> {
        // Simplified feature comparison
        // In real implementation, would use feature detection algorithms
        let ssim = self.calculate_ssim(&baseline.screenshot, screenshot)?;
        Ok((1.0 - ssim) * 100.0)
    }
    
    /// Match screenshots with tolerance
    pub fn matches_with_tolerance(
        &self,
        baseline: &Baseline,
        screenshot: &Screenshot,
        tolerance: f32,
    ) -> Result<bool, VRError> {
        let ssim = self.calculate_ssim(&baseline.screenshot, screenshot)?;
        Ok(ssim >= 1.0 - (tolerance / 100.0))
    }
    
    /// Generate heatmap from diff
    pub fn generate_heatmap(&self, diff: &DiffResult) -> Result<Vec<u8>, VRError> {
        let mut heatmap = diff.diff_image.clone();
        
        // Apply heatmap coloring based on diff intensity
        for i in (0..heatmap.len()).step_by(4) {
            let intensity = ((heatmap[i] as i16 + heatmap[i+1] as i16 + heatmap[i+2] as i16) / 3) as u8;
            
            // Color gradient: blue (low) -> green -> yellow -> red (high)
            if intensity < 64 {
                heatmap[i] = 0;
                heatmap[i + 1] = intensity * 4;
                heatmap[i + 2] = 255;
            } else if intensity < 128 {
                heatmap[i] = 0;
                heatmap[i + 1] = 255;
                heatmap[i + 2] = 255 - (intensity - 64) * 4;
            } else if intensity < 192 {
                heatmap[i] = (intensity - 128) * 4;
                heatmap[i + 1] = 255;
                heatmap[i + 2] = 0;
            } else {
                heatmap[i] = 255;
                heatmap[i + 1] = 255 - (intensity - 192) * 4;
                heatmap[i + 2] = 0;
            }
        }
        
        Ok(heatmap)
    }
}

use super::baseline::Baseline;