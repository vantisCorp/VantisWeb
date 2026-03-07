use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadChunk {
    pub id: Uuid,
    pub download_id: Uuid,
    pub start_byte: u64,
    pub end_byte: u64,
    pub downloaded: u64,
    pub speed: f64,
    pub status: ChunkStatus,
    pub retry_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChunkStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccelerationConfig {
    pub max_chunks: usize,
    pub chunk_size: u64,
    pub min_chunk_size: u64,
    pub retry_attempts: u32,
    pub timeout_seconds: u64,
    pub enable_mirrors: bool,
    pub max_mirrors: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorServer {
    pub url: String,
    pub priority: u8,
    pub is_active: bool,
    pub latency_ms: u32,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccelerationStats {
    pub download_id: Uuid,
    pub chunks: Vec<DownloadChunk>,
    pub total_chunks: usize,
    pub completed_chunks: usize,
    pub active_chunks: usize,
    pub failed_chunks: usize,
    pub average_speed: f64,
    pub time_elapsed_seconds: u64,
    pub estimated_time_remaining: u64,
}

pub struct DownloadAccelerator {
    active_downloads: Arc<RwLock<HashMap<Uuid, Vec<DownloadChunk>>>>,
    mirrors: Arc<RwLock<HashMap<Uuid, Vec<MirrorServer>>>>,
    config: Arc<RwLock<AccelerationConfig>>,
}

impl DownloadAccelerator {
    pub fn new(config: AccelerationConfig) -> Self {
        Self {
            active_downloads: Arc::new(RwLock::new(HashMap::new())),
            mirrors: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(RwLock::new(config)),
        }
    }

    pub async fn start_accelerated_download(
        &self,
        download_id: Uuid,
        url: String,
        file_size: u64,
    ) -> Result<Vec<DownloadChunk>, String> {
        let config = self.config.read().await;
        let chunk_size = config.chunk_size.max(config.min_chunk_size);
        let num_chunks = ((file_size + chunk_size - 1) / chunk_size) as usize;
        let actual_chunks = num_chunks.min(config.max_chunks);
        drop(config);
        
        let mut chunks = Vec::new();
        let chunk_size = file_size / actual_chunks as u64;
        let remainder = file_size % actual_chunks as u64;
        
        let mut start = 0u64;
        for i in 0..actual_chunks {
            let size = chunk_size + if i == 0 { remainder } else { 0 };
            let end = start + size - 1;
            
            let chunk = DownloadChunk {
                id: Uuid::new_v4(),
                download_id,
                start_byte: start,
                end_byte: end,
                downloaded: 0,
                speed: 0.0,
                status: ChunkStatus::Pending,
                retry_count: 0,
            };
            
            chunks.push(chunk);
            start = end + 1;
        }
        
        let mut downloads = self.active_downloads.write().await;
        downloads.insert(download_id, chunks.clone());
        
        Ok(chunks)
    }

    pub async fn update_chunk_progress(
        &self,
        chunk_id: Uuid,
        downloaded: u64,
        speed: f64,
    ) -> Result<(), String> {
        let mut downloads = self.active_downloads.write().await;
        
        for chunks in downloads.values_mut() {
            if let Some(chunk) = chunks.iter_mut().find(|c| c.id == chunk_id) {
                chunk.downloaded = downloaded;
                chunk.speed = speed;
                return Ok(());
            }
        }
        
        Err("Chunk not found".to_string())
    }

    pub async fn mark_chunk_completed(&self, chunk_id: Uuid) -> Result<(), String> {
        let mut downloads = self.active_downloads.write().await;
        
        for chunks in downloads.iter_mut() {
            if let Some(chunk) = chunks.1.iter_mut().find(|c| c.id == chunk_id) {
                chunk.status = ChunkStatus::Completed;
                chunk.downloaded = chunk.end_byte - chunk.start_byte + 1;
                return Ok(());
            }
        }
        
        Err("Chunk not found".to_string())
    }

    pub async fn mark_chunk_failed(&self, chunk_id: Uuid) -> Result<bool, String> {
        let mut downloads = self.active_downloads.write().await;
        let config = self.config.read().await;
        
        for chunks in downloads.iter_mut() {
            if let Some(chunk) = chunks.1.iter_mut().find(|c| c.id == chunk_id) {
                chunk.retry_count += 1;
                
                if chunk.retry_count < config.retry_attempts {
                    chunk.status = ChunkStatus::Pending;
                    return Ok(true); // Should retry
                } else {
                    chunk.status = ChunkStatus::Failed;
                    return Ok(false); // Should not retry
                }
            }
        }
        
        Err("Chunk not found".to_string())
    }

    pub async fn get_download_chunks(&self, download_id: Uuid) -> Vec<DownloadChunk> {
        let downloads = self.active_downloads.read().await;
        downloads.get(&download_id).cloned().unwrap_or_default()
    }

    pub async fn get_acceleration_stats(&self, download_id: Uuid) -> AccelerationStats {
        let downloads = self.active_downloads.read().await;
        
        if let Some(chunks) = downloads.get(&download_id) {
            let completed = chunks.iter().filter(|c| c.status == ChunkStatus::Completed).count();
            let active = chunks.iter().filter(|c| c.status == ChunkStatus::InProgress).count();
            let failed = chunks.iter().filter(|c| c.status == ChunkStatus::Failed).count();
            
            let total_speed: f64 = chunks.iter()
                .filter(|c| c.status == ChunkStatus::InProgress)
                .map(|c| c.speed)
                .sum();
            
            let average_speed = if active > 0 {
                total_speed / active as f64
            } else {
                0.0
            };
            
            let total_downloaded: u64 = chunks.iter().map(|c| c.downloaded).sum();
            let total_size: u64 = chunks.iter().map(|c| c.end_byte - c.start_byte + 1).sum();
            let progress = total_downloaded as f64 / total_size as f64;
            
            let estimated_time_remaining = if average_speed > 0.0 {
                let remaining = total_size.saturating_sub(total_downloaded);
                (remaining as f64 / average_speed) as u64
            } else {
                u64::MAX
            };
            
            AccelerationStats {
                download_id,
                chunks: chunks.clone(),
                total_chunks: chunks.len(),
                completed_chunks: completed,
                active_chunks: active,
                failed_chunks: failed,
                average_speed,
                time_elapsed_seconds: 0,
                estimated_time_remaining,
            }
        } else {
            AccelerationStats {
                download_id,
                chunks: Vec::new(),
                total_chunks: 0,
                completed_chunks: 0,
                active_chunks: 0,
                failed_chunks: 0,
                average_speed: 0.0,
                time_elapsed_seconds: 0,
                estimated_time_remaining: 0,
            }
        }
    }

    pub async fn add_mirror(&self, download_id: Uuid, mirror: MirrorServer) {
        let mut mirrors = self.mirrors.write().await;
        mirrors.entry(download_id).or_insert_with(Vec::new).push(mirror);
    }

    pub async fn get_mirrors(&self, download_id: Uuid) -> Vec<MirrorServer> {
        let mirrors = self.mirrors.read().await;
        mirrors.get(&download_id).cloned().unwrap_or_default()
    }

    pub async fn get_best_mirror(&self, download_id: Uuid) -> Option<MirrorServer> {
        let mirrors = self.mirrors.read().await;
        
        mirrors.get(&download_id)
            .and_then(|m| {
                m.iter()
                    .filter(|m| m.is_active)
                    .min_by(|a, b| {
                        a.latency_ms.cmp(&b.latency_ms)
                            .then_with(|| b.success_rate.partial_cmp(&a.success_rate).unwrap())
                    })
                    .cloned()
            })
    }

    pub async fn update_mirror_status(
        &self,
        download_id: Uuid,
        url: String,
        latency_ms: u32,
        success: bool,
    ) {
        let mut mirrors = self.mirrors.write().await;
        
        if let Some(download_mirrors) = mirrors.get_mut(&download_id) {
            if let Some(mirror) = download_mirrors.iter_mut().find(|m| m.url == url) {
                mirror.latency_ms = latency_ms;
                let total_requests = 1.0;
                let success_rate = if success {
                    (mirror.success_rate * total_requests + 1.0) / (total_requests + 1.0)
                } else {
                    (mirror.success_rate * total_requests) / (total_requests + 1.0)
                };
                mirror.success_rate = success_rate;
            }
        }
    }

    pub async fn remove_download(&self, download_id: Uuid) {
        let mut downloads = self.active_downloads.write().await;
        downloads.remove(&download_id);
        
        let mut mirrors = self.mirrors.write().await;
        mirrors.remove(&download_id);
    }

    pub async fn update_config(&self, config: AccelerationConfig) {
        *self.config.write().await = config;
    }

    pub async fn get_config(&self) -> AccelerationConfig {
        self.config.read().await.clone()
    }

    pub async fn get_optimal_chunk_count(&self, file_size: u64, bandwidth_mbps: f64) -> usize {
        let config = self.config.read().await;
        
        // Calculate optimal chunk count based on file size and bandwidth
        let min_size_for_multi_chunk = 10 * 1024 * 1024; // 10 MB
        let chunks_per_mbps = bandwidth_mbps.ceil() as usize;
        
        if file_size < min_size_for_multi_chunk {
            1
        } else {
            (chunks_per_mbps.min(config.max_chunks)).max(2)
        }
    }

    pub async fn get_active_downloads(&self) -> Vec<Uuid> {
        self.active_downloads.read().await.keys().cloned().collect()
    }
}

impl Default for AccelerationConfig {
    fn default() -> Self {
        Self {
            max_chunks: 8,
            chunk_size: 5 * 1024 * 1024, // 5 MB
            min_chunk_size: 1024 * 1024, // 1 MB
            retry_attempts: 3,
            timeout_seconds: 30,
            enable_mirrors: true,
            max_mirrors: 5,
        }
    }
}

impl Default for DownloadAccelerator {
    fn default() -> Self {
        Self::new(AccelerationConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_accelerator_creation() {
        let config = AccelerationConfig::default();
        let accelerator = DownloadAccelerator::new(config);
        
        let active = accelerator.get_active_downloads().await;
        assert_eq!(active.len(), 0);
    }

    #[tokio::test]
    async fn test_start_accelerated_download() {
        let accelerator = DownloadAccelerator::default();
        let download_id = Uuid::new_v4();
        
        let chunks = accelerator.start_accelerated_download(
            download_id,
            "https://example.com/file.zip".to_string(),
            100 * 1024 * 1024, // 100 MB
        ).await.unwrap();
        
        assert!(!chunks.is_empty());
        assert_eq!(chunks[0].download_id, download_id);
    }

    #[tokio::test]
    async fn test_update_chunk_progress() {
        let accelerator = DownloadAccelerator::default();
        let download_id = Uuid::new_v4();
        
        let mut chunks = accelerator.start_accelerated_download(
            download_id,
            "https://example.com/file.zip".to_string(),
            10 * 1024 * 1024, // 10 MB
        ).await.unwrap();
        
        let chunk_id = chunks[0].id;
        accelerator.update_chunk_progress(chunk_id, 1024 * 1024, 1024.0 * 1024.0).await.unwrap();
        
        let updated_chunks = accelerator.get_download_chunks(download_id).await;
        assert_eq!(updated_chunks[0].downloaded, 1024 * 1024);
    }

    #[tokio::test]
    async fn test_mark_chunk_completed() {
        let accelerator = DownloadAccelerator::default();
        let download_id = Uuid::new_v4();
        
        let chunks = accelerator.start_accelerated_download(
            download_id,
            "https://example.com/file.zip".to_string(),
            10 * 1024 * 1024,
        ).await.unwrap();
        
        let chunk_id = chunks[0].id;
        accelerator.mark_chunk_completed(chunk_id).await.unwrap();
        
        let updated_chunks = accelerator.get_download_chunks(download_id).await;
        assert_eq!(updated_chunks[0].status, ChunkStatus::Completed);
    }

    #[tokio::test]
    async fn test_acceleration_stats() {
        let accelerator = DownloadAccelerator::default();
        let download_id = Uuid::new_v4();
        
        accelerator.start_accelerated_download(
            download_id,
            "https://example.com/file.zip".to_string(),
            10 * 1024 * 1024,
        ).await.unwrap();
        
        let stats = accelerator.get_acceleration_stats(download_id).await;
        assert_eq!(stats.download_id, download_id);
        assert!(stats.total_chunks > 0);
    }

    #[tokio::test]
    async fn test_add_mirror() {
        let accelerator = DownloadAccelerator::default();
        let download_id = Uuid::new_v4();
        
        let mirror = MirrorServer {
            url: "https://mirror1.example.com/file.zip".to_string(),
            priority: 1,
            is_active: true,
            latency_ms: 50,
            success_rate: 0.95,
        };
        
        accelerator.add_mirror(download_id, mirror).await;
        
        let mirrors = accelerator.get_mirrors(download_id).await;
        assert_eq!(mirrors.len(), 1);
    }

    #[tokio::test]
    async fn test_get_best_mirror() {
        let accelerator = DownloadAccelerator::default();
        let download_id = Uuid::new_v4();
        
        let mirror1 = MirrorServer {
            url: "https://mirror1.example.com/file.zip".to_string(),
            priority: 1,
            is_active: true,
            latency_ms: 50,
            success_rate: 0.95,
        };
        
        let mirror2 = MirrorServer {
            url: "https://mirror2.example.com/file.zip".to_string(),
            priority: 2,
            is_active: true,
            latency_ms: 30,
            success_rate: 0.90,
        };
        
        accelerator.add_mirror(download_id, mirror1).await;
        accelerator.add_mirror(download_id, mirror2).await;
        
        let best = accelerator.get_best_mirror(download_id).await;
        assert!(best.is_some());
        assert_eq!(best.unwrap().latency_ms, 30); // Lower latency wins
    }

    #[tokio::test]
    async fn test_remove_download() {
        let accelerator = DownloadAccelerator::default();
        let download_id = Uuid::new_v4();
        
        accelerator.start_accelerated_download(
            download_id,
            "https://example.com/file.zip".to_string(),
            10 * 1024 * 1024,
        ).await.unwrap();
        
        accelerator.remove_download(download_id).await;
        
        let chunks = accelerator.get_download_chunks(download_id).await;
        assert_eq!(chunks.len(), 0);
    }

    #[tokio::test]
    async fn test_optimal_chunk_count() {
        let accelerator = DownloadAccelerator::default();
        
        // Small file - single chunk
        let chunks1 = accelerator.get_optimal_chunk_count(5 * 1024 * 1024, 100.0).await;
        assert_eq!(chunks1, 1);
        
        // Large file - multiple chunks
        let chunks2 = accelerator.get_optimal_chunk_count(100 * 1024 * 1024, 100.0).await;
        assert!(chunks2 > 1);
    }
}