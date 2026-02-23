//! Downloads Module
//! 
//! Download management:
//! - File downloads
//! - Download queue
//! - Progress tracking
//! - Resume support

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Download status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DownloadStatus {
    Pending,
    Downloading { progress: f32 },
    Paused,
    Completed,
    Failed { error: String },
}

/// Download entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Download {
    pub id: String,
    pub url: String,
    pub filename: String,
    pub save_path: PathBuf,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub status: DownloadStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Download manager
pub struct DownloadManager {
    downloads: Vec<Download>,
    download_dir: PathBuf,
}

impl DownloadManager {
    /// Create a new download manager
    pub fn new(download_dir: PathBuf) -> Result<Self> {
        info!("Initializing Download Manager...");
        
        std::fs::create_dir_all(&amp;download_dir)?;
        
        Ok(Self {
            downloads: Vec::new(),
            download_dir,
        })
    }
    
    /// Start a download
    pub async fn start_download(&amp;mut self, url: String) -> Result<String> {
        info!("Starting download: {}", url);
        
        // Extract filename from URL
        let filename = self.extract_filename(&amp;url)?;
        let save_path = self.download_dir.join(&amp;filename);
        
        let download = Download {
            id: uuid::Uuid::new_v4().to_string(),
            url: url.clone(),
            filename: filename.clone(),
            save_path,
            total_bytes: 0,
            downloaded_bytes: 0,
            status: DownloadStatus::Pending,
            created_at: Utc::now(),
            completed_at: None,
        };
        
        let download_id = download.id.clone();
        self.downloads.push(download);
        
        // Start download in background
        self.download_file(download_id.clone()).await?;
        
        info!("Download started: {} (ID: {})", url, download_id);
        
        Ok(download_id)
    }
    
    /// Extract filename from URL
    fn extract_filename(&amp;self, url: &amp;str) -> Result<String> {
        let url_parts: Vec<&amp;str> = url.split('/').collect();
        let filename = url_parts.last().unwrap_or("download");
        
        if filename.is_empty() {
            Ok("download".to_string())
        } else {
            Ok(filename.to_string())
        }
    }
    
    /// Download file
    async fn download_file(&amp;mut self, download_id: String) -> Result<()> {
        // Update status
        if let Some(download) = self.downloads.iter_mut().find(|d| d.id == download_id) {
            download.status = DownloadStatus::Downloading { progress: 0.0 };
        }
        
        // In production: Use reqwest to download file
        // For MVP: Simulate download
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Update status to completed
        if let Some(download) = self.downloads.iter_mut().find(|d| d.id == download_id) {
            download.downloaded_bytes = download.total_bytes;
            download.status = DownloadStatus::Completed;
            download.completed_at = Some(Utc::now());
        }
        
        info!("Download completed: {}", download_id);
        
        Ok(())
    }
    
    /// Pause download
    pub async fn pause_download(&amp;mut self, download_id: String) -> Result<()> {
        info!("Pausing download: {}", download_id);
        
        if let Some(download) = self.downloads.iter_mut().find(|d| d.id == download_id) {
            download.status = DownloadStatus::Paused;
        }
        
        Ok(())
    }
    
    /// Resume download
    pub async fn resume_download(&amp;mut self, download_id: String) -> Result<()> {
        info!("Resuming download: {}", download_id);
        
        if let Some(download) = self.downloads.iter_mut().find(|d| d.id == download_id) {
            download.status = DownloadStatus::Downloading { progress: download.downloaded_bytes as f32 / download.total_bytes.max(1) as f32 };
            self.download_file(download_id).await?;
        }
        
        Ok(())
    }
    
    /// Cancel download
    pub async fn cancel_download(&amp;mut self, download_id: String) -> Result<()> {
        info!("Cancelling download: {}", download_id);
        
        self.downloads.retain(|d| d.id != download_id);
        
        Ok(())
    }
    
    /// Get all downloads
    pub fn get_all(&amp;self) -> &amp;[Download] {
        &amp;self.downloads
    }
    
    /// Get download by ID
    pub fn get(&amp;self, download_id: &amp;str) -> Option<&amp;Download> {
        self.downloads.iter().find(|d| d.id == download_id)
    }
    
    /// Clear completed downloads
    pub async fn clear_completed(&amp;mut self) {
        info!("Clearing completed downloads");
        self.downloads.retain(|d| !matches!(d.status, DownloadStatus::Completed));
    }
}