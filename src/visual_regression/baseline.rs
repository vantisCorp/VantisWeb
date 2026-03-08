//! Baseline management for visual regression testing

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use super::models::*;
use super::{Screenshot, VisualRegressionConfig, VRError};

/// Baseline manager
pub struct BaselineManager {
    config: VisualRegressionConfig,
    /// In-memory cache of baselines
    cache: HashMap<String, Baseline>,
    /// Baseline metadata index
    index: BaselineIndex,
}

/// Index of all baselines
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BaselineIndex {
    /// Baseline entries
    pub entries: HashMap<String, BaselineEntry>,
    /// Last updated
    pub last_updated: DateTime<Utc>,
}

/// Baseline entry in index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineEntry {
    /// Baseline name
    pub name: String,
    /// Viewport name
    pub viewport: String,
    /// File path
    pub path: PathBuf,
    /// Creation date
    pub created_at: DateTime<Utc>,
    /// Last modified
    pub modified_at: DateTime<Utc>,
    /// Version/hash
    pub version: String,
    /// Tags
    pub tags: Vec<String>,
    /// Approved by
    pub approved_by: Option<String>,
    /// Approval date
    pub approved_at: Option<DateTime<Utc>>,
}

impl BaselineManager {
    /// Create a new baseline manager
    pub fn new(config: VisualRegressionConfig) -> Self {
        Self {
            config,
            cache: HashMap::new(),
            index: BaselineIndex::default(),
        }
    }
    
    /// Load baseline for a test
    pub async fn load(&self, name: &str, viewport: &str) -> Result<Option<Baseline>, VRError> {
        let key = format!("{}_{}", name, viewport);
        
        // Check cache first
        if let Some(baseline) = self.cache.get(&key) {
            return Ok(Some(baseline.clone()));
        }
        
        // Load from disk
        let filename = format!("{}.png", key);
        let path = self.config.baseline_dir.join(&filename);
        
        if !path.exists() {
            return Ok(None);
        }
        
        // Load image
        let data = tokio::fs::read(&path).await?;
        
        // Load metadata
        let meta_path = path.with_extension("json");
        let metadata = if meta_path.exists() {
            let meta_data = tokio::fs::read_to_string(&meta_path).await?;
            serde_json::from_str(&meta_data).unwrap_or_default()
        } else {
            BaselineMetadata::default()
        };
        
        // Create baseline
        let baseline = Baseline {
            name: name.to_string(),
            viewport: viewport.to_string(),
            screenshot: Screenshot::new(
                metadata.width.unwrap_or(1920),
                metadata.height.unwrap_or(1080),
                data,
            ),
            path,
            created_at: metadata.created_at.unwrap_or_else(Utc::now),
            version: metadata.version.unwrap_or_default(),
        };
        
        Ok(Some(baseline))
    }
    
    /// Save a new baseline
    pub async fn save(
        &mut self,
        name: &str,
        viewport: &str,
        screenshot: &Screenshot,
    ) -> Result<Baseline, VRError> {
        // Ensure directory exists
        tokio::fs::create_dir_all(&self.config.baseline_dir).await?;
        
        let key = format!("{}_{}", name, viewport);
        let filename = format!("{}.png", key);
        let path = self.config.baseline_dir.join(&filename);
        
        // Save image
        let png_data = screenshot.to_png()?;
        tokio::fs::write(&path, &png_data).await?;
        
        // Calculate version hash
        let version = format!("{:x}", md5::compute(&png_data));
        
        // Save metadata
        let metadata = BaselineMetadata {
            width: Some(screenshot.width),
            height: Some(screenshot.height),
            created_at: Some(Utc::now()),
            version: Some(version.clone()),
            viewport: Some(viewport.to_string()),
        };
        
        let meta_path = path.with_extension("json");
        let meta_json = serde_json::to_string_pretty(&metadata)?;
        tokio::fs::write(&meta_path, &meta_json).await?;
        
        // Create baseline
        let baseline = Baseline {
            name: name.to_string(),
            viewport: viewport.to_string(),
            screenshot: screenshot.clone(),
            path,
            created_at: Utc::now(),
            version,
        };
        
        // Update cache
        self.cache.insert(key, baseline.clone());
        
        // Update index
        self.index.entries.insert(
            format!("{}_{}", name, viewport),
            BaselineEntry {
                name: name.to_string(),
                viewport: viewport.to_string(),
                path: baseline.path.clone(),
                created_at: baseline.created_at,
                modified_at: Utc::now(),
                version: baseline.version.clone(),
                tags: Vec::new(),
                approved_by: None,
                approved_at: None,
            },
        );
        self.index.last_updated = Utc::now();
        
        Ok(baseline)
    }
    
    /// Delete a baseline
    pub async fn delete(&mut self, name: &str, viewport: &str) -> Result<(), VRError> {
        let key = format!("{}_{}", name, viewport);
        let filename = format!("{}.png", key);
        let path = self.config.baseline_dir.join(&filename);
        
        if path.exists() {
            tokio::fs::remove_file(&path).await?;
        }
        
        // Remove metadata
        let meta_path = path.with_extension("json");
        if meta_path.exists() {
            tokio::fs::remove_file(&meta_path).await?;
        }
        
        // Remove from cache
        self.cache.remove(&key);
        
        // Remove from index
        self.index.entries.remove(&key);
        self.index.last_updated = Utc::now();
        
        Ok(())
    }
    
    /// Approve a baseline
    pub async fn approve(
        &mut self,
        name: &str,
        viewport: &str,
        approved_by: &str,
    ) -> Result<(), VRError> {
        let key = format!("{}_{}", name, viewport);
        
        if let Some(entry) = self.index.entries.get_mut(&key) {
            entry.approved_by = Some(approved_by.to_string());
            entry.approved_at = Some(Utc::now());
            entry.modified_at = Utc::now();
        }
        
        // Save updated index
        self.save_index().await?;
        
        Ok(())
    }
    
    /// List all baselines
    pub async fn list(&self) -> Result<Vec<BaselineEntry>, VRError> {
        Ok(self.index.entries.values().cloned().collect())
    }
    
    /// List baselines for a specific test
    pub async fn list_for_test(&self, name: &str) -> Result<Vec<BaselineEntry>, VRError> {
        Ok(self.index.entries
            .values()
            .filter(|e| e.name == name)
            .cloned()
            .collect())
    }
    
    /// Get baseline statistics
    pub async fn stats(&self) -> BaselineStats {
        let total = self.index.entries.len();
        let approved = self.index.entries
            .values()
            .filter(|e| e.approved_at.is_some())
            .count();
        
        let viewports: std::collections::HashSet<_> = self.index.entries
            .values()
            .map(|e| e.viewport.clone())
            .collect();
        
        BaselineStats {
            total_baselines: total,
            approved_baselines: approved,
            pending_baselines: total - approved,
            unique_viewports: viewports.len(),
        }
    }
    
    /// Load index from disk
    pub async fn load_index(&mut self) -> Result<(), VRError> {
        let index_path = self.config.baseline_dir.join("index.json");
        
        if index_path.exists() {
            let data = tokio::fs::read_to_string(&index_path).await?;
            self.index = serde_json::from_str(&data).unwrap_or_default();
        }
        
        Ok(())
    }
    
    /// Save index to disk
    pub async fn save_index(&self) -> Result<(), VRError> {
        let index_path = self.config.baseline_dir.join("index.json");
        let data = serde_json::to_string_pretty(&self.index)?;
        tokio::fs::write(&index_path, &data).await?;
        Ok(())
    }
    
    /// Compare with existing baseline
    pub async fn compare_hash(
        &self,
        name: &str,
        viewport: &str,
        screenshot: &Screenshot,
    ) -> Result<Option<bool>, VRError> {
        let baseline = self.load(name, viewport).await?;
        
        match baseline {
            Some(b) => {
                let new_hash = format!("{:x}", md5::compute(&screenshot.data));
                Ok(Some(b.version == new_hash))
            }
            None => Ok(None),
        }
    }
    
    /// Migrate baselines from old format
    pub async fn migrate(&mut self, from_dir: &Path) -> Result<usize, VRError> {
        let mut migrated = 0;
        
        if !from_dir.exists() {
            return Ok(0);
        }
        
        let mut entries = tokio::fs::read_dir(from_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.extension().map(|e| e == "png").unwrap_or(false) {
                if let Some(stem) = path.file_stem() {
                    let name = stem.to_string_lossy();
                    
                    // Parse name_viewport format
                    let parts: Vec<&str> = name.split('_').collect();
                    if parts.len() >= 2 {
                        let viewport = parts.last().unwrap();
                        let test_name = parts[..parts.len() - 1].join("_");
                        
                        // Load and save
                        let data = tokio::fs::read(&path).await?;
                        // Assume standard dimensions
                        let screenshot = Screenshot::new(1920, 1080, data);
                        
                        self.save(&test_name, viewport, &screenshot).await?;
                        migrated += 1;
                    }
                }
            }
        }
        
        Ok(migrated)
    }
}

/// Baseline metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BaselineMetadata {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub created_at: Option<DateTime<Utc>>,
    pub version: Option<String>,
    pub viewport: Option<String>,
}

/// Baseline statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineStats {
    pub total_baselines: usize,
    pub approved_baselines: usize,
    pub pending_baselines: usize,
    pub unique_viewports: usize,
}

/// Baseline store interface
pub struct BaselineStore {
    manager: Arc<RwLock<BaselineManager>>,
}

impl BaselineStore {
    /// Create a new baseline store
    pub fn new(manager: BaselineManager) -> Self {
        Self {
            manager: Arc::new(RwLock::new(manager)),
        }
    }
    
    /// Get baseline
    pub async fn get(&self, name: &str, viewport: &str) -> Result<Option<Baseline>, VRError> {
        let manager = self.manager.read().await;
        manager.load(name, viewport).await
    }
    
    /// Set baseline
    pub async fn set(&self, name: &str, viewport: &str, screenshot: &Screenshot) -> Result<Baseline, VRError> {
        let mut manager = self.manager.write().await;
        manager.save(name, viewport, screenshot).await
    }
    
    /// Remove baseline
    pub async fn remove(&self, name: &str, viewport: &str) -> Result<(), VRError> {
        let mut manager = self.manager.write().await;
        manager.delete(name, viewport).await
    }
}