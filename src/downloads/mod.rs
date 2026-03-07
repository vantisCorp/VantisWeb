use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadStatus {
    Pending,
    Started,
    InProgress,
    Paused,
    Completed,
    Failed,
    Cancelled,
    Retrying,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Download {
    pub id: Uuid,
    pub url: String,
    pub file_name: String,
    pub save_path: PathBuf,
    pub total_size: u64,
    pub downloaded_size: u64,
    pub speed: f64, // bytes per second
    pub status: DownloadStatus,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub category: Option<String>,
    pub mime_type: Option<String>,
    pub error_message: Option<String>,
    pub checksum: Option<String>,
    pub is_virus_scanned: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadUpdate {
    pub progress: Option<f64>,
    pub downloaded_size: Option<u64>,
    pub speed: Option<f64>,
    pub status: Option<DownloadStatus>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadStats {
    pub total_downloads: usize,
    pub active_downloads: usize,
    pub completed_downloads: usize,
    pub failed_downloads: usize,
    pub total_downloaded_bytes: u64,
    pub total_bandwidth: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum DownloadPriority {
    Low,
    Normal,
    High,
    Urgent,
}

pub struct DownloadManager {
    downloads: Arc<RwLock<HashMap<Uuid, Download>>>,
    queue: Arc<RwLock<Vec<Uuid>>>,
    max_concurrent: usize,
    speed_limit: Arc<RwLock<Option<f64>>>, // bytes per second
}

impl DownloadManager {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            downloads: Arc::new(RwLock::new(HashMap::new())),
            queue: Arc::new(RwLock::new(Vec::new())),
            max_concurrent,
            speed_limit: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn add_download(&self, url: String, save_path: PathBuf) -> Result<Download, String> {
        self.add_download_with_options(
            url,
            save_path,
            None,
            3,
            DownloadPriority::Normal,
        ).await
    }

    pub async fn add_download_with_options(
        &self,
        url: String,
        save_path: PathBuf,
        category: Option<String>,
        max_retries: u32,
        priority: DownloadPriority,
    ) -> Result<Download, String> {
        let file_name = extract_filename(&url, &save_path);
        
        let now = Utc::now();
        let download = Download {
            id: Uuid::new_v4(),
            url,
            file_name,
            save_path,
            total_size: 0,
            downloaded_size: 0,
            speed: 0.0,
            status: DownloadStatus::Pending,
            start_time: None,
            end_time: None,
            retry_count: 0,
            max_retries,
            category,
            mime_type: None,
            error_message: None,
            checksum: None,
            is_virus_scanned: false,
            created_at: now,
            updated_at: now,
        };

        let id = download.id;
        
        // Add to downloads
        let mut downloads = self.downloads.write().await;
        downloads.insert(id, download.clone());
        drop(downloads);
        
        // Add to queue
        match priority {
            DownloadPriority::Urgent => {
                let mut queue = self.queue.write().await;
                queue.insert(0, id);
            }
            DownloadPriority::High => {
                let mut queue = self.queue.write().await;
                queue.insert(1.min(queue.len()), id);
            }
            DownloadPriority::Normal => {
                let mut queue = self.queue.write().await;
                queue.push(id);
            }
            DownloadPriority::Low => {
                let mut queue = self.queue.write().await;
                queue.push(id);
            }
        }
        
        Ok(download)
    }

    pub async fn get_download(&self, id: Uuid) -> Option<Download> {
        let downloads = self.downloads.read().await;
        downloads.get(&id).cloned()
    }

    pub async fn get_all_downloads(&self) -> Vec<Download> {
        let downloads = self.downloads.read().await;
        downloads.values().cloned().collect()
    }

    pub async fn get_active_downloads(&self) -> Vec<Download> {
        let downloads = self.downloads.read().await;
        downloads.values()
            .filter(|d| matches!(d.status, DownloadStatus::Started | DownloadStatus::InProgress))
            .cloned()
            .collect()
    }

    pub async fn get_completed_downloads(&self) -> Vec<Download> {
        let downloads = self.downloads.read().await;
        downloads.values()
            .filter(|d| matches!(d.status, DownloadStatus::Completed))
            .cloned()
            .collect()
    }

    pub async fn get_failed_downloads(&self) -> Vec<Download> {
        let downloads = self.downloads.read().await;
        downloads.values()
            .filter(|d| matches!(d.status, DownloadStatus::Failed))
            .cloned()
            .collect()
    }

    pub async fn start_download(&self, id: Uuid) -> Result<(), String> {
        let mut downloads = self.downloads.write().await;
        if let Some(download) = downloads.get_mut(&id) {
            if download.status == DownloadStatus::Pending || download.status == DownloadStatus::Paused {
                download.status = DownloadStatus::Started;
                download.start_time = Some(download.start_time.unwrap_or_else(Utc::now));
                download.updated_at = Utc::now();
                return Ok(());
            }
            return Err("Download cannot be started in current state".to_string());
        }
        Err("Download not found".to_string())
    }

    pub async fn pause_download(&self, id: Uuid) -> Result<(), String> {
        let mut downloads = self.downloads.write().await;
        if let Some(download) = downloads.get_mut(&id) {
            if matches!(download.status, DownloadStatus::Started | DownloadStatus::InProgress) {
                download.status = DownloadStatus::Paused;
                download.updated_at = Utc::now();
                return Ok(());
            }
            return Err("Download cannot be paused in current state".to_string());
        }
        Err("Download not found".to_string())
    }

    pub async fn resume_download(&self, id: Uuid) -> Result<(), String> {
        let mut downloads = self.downloads.write().await;
        if let Some(download) = downloads.get_mut(&id) {
            if download.status == DownloadStatus::Paused {
                download.status = DownloadStatus::Started;
                download.updated_at = Utc::now();
                return Ok(());
            }
            return Err("Download cannot be resumed in current state".to_string());
        }
        Err("Download not found".to_string())
    }

    pub async fn cancel_download(&self, id: Uuid) -> Result<(), String> {
        let mut downloads = self.downloads.write().await;
        if let Some(download) = downloads.get_mut(&id) {
            download.status = DownloadStatus::Cancelled;
            download.end_time = Some(Utc::now());
            download.updated_at = Utc::now();
            
            // Remove from queue
            let mut queue = self.queue.write().await;
            queue.retain(|&q| q != id);
            
            return Ok(());
        }
        Err("Download not found".to_string())
    }

    pub async fn retry_download(&self, id: Uuid) -> Result<(), String> {
        let mut downloads = self.downloads.write().await;
        if let Some(download) = downloads.get_mut(&id) {
            if download.status == DownloadStatus::Failed && download.retry_count < download.max_retries {
                download.status = DownloadStatus::Retrying;
                download.retry_count += 1;
                download.error_message = None;
                download.updated_at = Utc::now();
                return Ok(());
            }
            return Err("Download cannot be retried in current state".to_string());
        }
        Err("Download not found".to_string())
    }

    pub async fn update_download_progress(&self, id: Uuid, update: DownloadUpdate) -> Result<(), String> {
        let mut downloads = self.downloads.write().await;
        if let Some(download) = downloads.get_mut(&id) {
            if let Some(progress) = update.progress {
                if let Some(total) = download.total_size {
                    download.downloaded_size = (total as f64 * progress / 100.0) as u64;
                }
            }
            if let Some(downloaded_size) = update.downloaded_size {
                download.downloaded_size = downloaded_size;
            }
            if let Some(speed) = update.speed {
                download.speed = speed;
            }
            if let Some(status) = update.status {
                download.status = status;
                if status == DownloadStatus::Completed {
                    download.end_time = Some(Utc::now());
                } else if status == DownloadStatus::Failed {
                    download.end_time = Some(Utc::now());
                }
            }
            if let Some(error) = update.error_message {
                download.error_message = Some(error);
            }
            download.updated_at = Utc::now();
            return Ok(());
        }
        Err("Download not found".to_string())
    }

    pub async fn set_speed_limit(&self, limit: Option<f64>) {
        *self.speed_limit.write().await = limit;
    }

    pub async fn get_speed_limit(&self) -> Option<f64> {
        *self.speed_limit.read().await
    }

    pub async fn get_queue(&self) -> Vec<Download> {
        let queue = self.queue.read().await;
        let downloads = self.downloads.read().await;
        
        let mut result = Vec::new();
        for id in queue.iter() {
            if let Some(download) = downloads.get(id) {
                result.push(download.clone());
            }
        }
        
        result
    }

    pub async fn get_queue_position(&self, id: Uuid) -> Option<usize> {
        let queue = self.queue.read().await;
        queue.iter().position(|&q| q == id)
    }

    pub async fn move_in_queue(&self, id: Uuid, new_position: usize) -> Result<(), String> {
        let mut queue = self.queue.write().await;
        if let Some(pos) = queue.iter().position(|&q| q == id) {
            queue.remove(pos);
            queue.insert(new_position.min(queue.len()), id);
            return Ok(());
        }
        Err("Download not in queue".to_string())
    }

    pub async fn remove_from_queue(&self, id: Uuid) -> Result<(), String> {
        let mut queue = self.queue.write().await;
        if queue.iter().any(|&q| q == id) {
            queue.retain(|&q| q != id);
            return Ok(());
        }
        Err("Download not in queue".to_string())
    }

    pub async fn clear_queue(&self) {
        self.queue.write().await.clear();
    }

    pub async fn get_statistics(&self) -> DownloadStats {
        let downloads = self.downloads.read().await;
        let total_downloads = downloads.len();
        
        let active_downloads = downloads.values()
            .filter(|d| matches!(d.status, DownloadStatus::Started | DownloadStatus::InProgress))
            .count();
        
        let completed_downloads = downloads.values()
            .filter(|d| matches!(d.status, DownloadStatus::Completed))
            .count();
        
        let failed_downloads = downloads.values()
            .filter(|d| matches!(d.status, DownloadStatus::Failed))
            .count();
        
        let total_downloaded_bytes: u64 = downloads.values()
            .filter(|d| matches!(d.status, DownloadStatus::Completed))
            .map(|d| d.downloaded_size)
            .sum();
        
        let total_bandwidth: f64 = downloads.values()
            .filter(|d| matches!(d.status, DownloadStatus::Started | DownloadStatus::InProgress))
            .map(|d| d.speed)
            .sum();
        
        DownloadStats {
            total_downloads,
            active_downloads,
            completed_downloads,
            failed_downloads,
            total_downloaded_bytes,
            total_bandwidth,
        }
    }

    pub async fn cleanup_old_downloads(&self, days_old: i64) -> Result<usize, String> {
        let cutoff = Utc::now() - chrono::Duration::days(days_old);
        let mut downloads = self.downloads.write().await;
        
        let initial_count = downloads.len();
        downloads.retain(|_, d| {
            d.created_at > cutoff || matches!(d.status, DownloadStatus::Started | DownloadStatus::InProgress)
        });
        
        let removed = initial_count - downloads.len();
        Ok(removed)
    }

    pub async fn delete_download(&self, id: Uuid) -> Result<bool, String> {
        let mut downloads = self.downloads.write().await;
        if downloads.remove(&id).is_some() {
            let mut queue = self.queue.write().await;
            queue.retain(|&q| q != id);
            return Ok(true);
        }
        Ok(false)
    }

    pub async fn delete_file(&self, id: Uuid) -> Result<(), String> {
        let downloads = self.downloads.read().await;
        if let Some(download) = downloads.get(&id) {
            tokio::fs::remove_file(&download.save_path).await
                .map_err(|e| format!("Failed to delete file: {}", e))?;
            return Ok(());
        }
        Err("Download not found".to_string())
    }

    pub async fn delete_download_and_file(&self, id: Uuid) -> Result<bool, String> {
        self.delete_file(id).await?;
        self.delete_download(id).await
    }
}

fn extract_filename(url: &str, save_path: &PathBuf) -> String {
    if let Some(filename) = url.split('/').last() {
        if !filename.is_empty() && !filename.contains('?') {
            return filename.to_string();
        }
    }
    
    // Fallback to timestamp-based filename
    format!("download_{}", Utc::now().timestamp())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_download_manager_creation() {
        let manager = DownloadManager::new(3);
        let stats = manager.get_statistics().await;
        assert_eq!(stats.total_downloads, 0);
    }

    #[tokio::test]
    async fn test_add_download() {
        let manager = DownloadManager::new(3);
        let download = manager.add_download(
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/tmp/file.zip"),
        ).await.unwrap();
        
        assert_eq!(download.url, "https://example.com/file.zip");
        assert_eq!(download.status, DownloadStatus::Pending);
    }

    #[tokio::test]
    async fn test_start_download() {
        let manager = DownloadManager::new(3);
        let download = manager.add_download(
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/tmp/file.zip"),
        ).await.unwrap();
        
        manager.start_download(download.id).await.unwrap();
        
        let updated = manager.get_download(download.id).await.unwrap();
        assert_eq!(updated.status, DownloadStatus::Started);
        assert!(updated.start_time.is_some());
    }

    #[tokio::test]
    async fn test_pause_resume_download() {
        let manager = DownloadManager::new(3);
        let download = manager.add_download(
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/tmp/file.zip"),
        ).await.unwrap();
        
        manager.start_download(download.id).await.unwrap();
        manager.pause_download(download.id).await.unwrap();
        
        let paused = manager.get_download(download.id).await.unwrap();
        assert_eq!(paused.status, DownloadStatus::Paused);
        
        manager.resume_download(download.id).await.unwrap();
        
        let resumed = manager.get_download(download.id).await.unwrap();
        assert_eq!(resumed.status, DownloadStatus::Started);
    }

    #[tokio::test]
    async fn test_cancel_download() {
        let manager = DownloadManager::new(3);
        let download = manager.add_download(
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/tmp/file.zip"),
        ).await.unwrap();
        
        manager.cancel_download(download.id).await.unwrap();
        
        let cancelled = manager.get_download(download.id).await.unwrap();
        assert_eq!(cancelled.status, DownloadStatus::Cancelled);
        assert!(cancelled.end_time.is_some());
    }

    #[tokio::test]
    async fn test_update_progress() {
        let manager = DownloadManager::new(3);
        let mut download = manager.add_download(
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/tmp/file.zip"),
        ).await.unwrap();
        
        download.total_size = 1000;
        
        manager.update_download_progress(download.id, DownloadUpdate {
            progress: Some(50.0),
            speed: Some(1000.0),
            ..Default::default()
        }).await.unwrap();
        
        let updated = manager.get_download(download.id).await.unwrap();
        assert_eq!(updated.downloaded_size, 500);
        assert_eq!(updated.speed, 1000.0);
    }

    #[tokio::test]
    async fn test_queue_management() {
        let manager = DownloadManager::new(3);
        
        let dl1 = manager.add_download(
            "https://example.com/file1.zip".to_string(),
            PathBuf::from("/tmp/file1.zip"),
        ).await.unwrap();
        
        let dl2 = manager.add_download(
            "https://example.com/file2.zip".to_string(),
            PathBuf::from("/tmp/file2.zip"),
        ).await.unwrap();
        
        let queue = manager.get_queue().await;
        assert_eq!(queue.len(), 2);
    }
}

impl Default for DownloadUpdate {
    fn default() -> Self {
        Self {
            progress: None,
            downloaded_size: None,
            speed: None,
            status: None,
            error_message: None,
        }
    }
}