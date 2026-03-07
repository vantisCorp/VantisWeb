use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadHistoryEntry {
    pub id: Uuid,
    pub url: String,
    pub file_name: String,
    pub file_size: u64,
    pub save_path: PathBuf,
    pub download_time: DateTime<Utc>,
    pub completion_time: Option<DateTime<Utc>>,
    pub duration_seconds: Option<u64>,
    pub status: DownloadStatus,
    pub category: Option<String>,
    pub checksum: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadStatus {
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryQuery {
    pub url: Option<String>,
    pub file_name: Option<String>,
    pub category: Option<String>,
    pub status: Option<DownloadStatus>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryStats {
    pub total_downloads: usize,
    pub completed_downloads: usize,
    pub failed_downloads: usize,
    pub cancelled_downloads: usize,
    pub total_downloaded_bytes: u64,
    pub total_bandwidth_used: u64,
    pub average_speed: f64,
    pub fastest_download: Option<SpeedRecord>,
    pub slowest_download: Option<SpeedRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedRecord {
    pub url: String,
    pub file_name: String,
    pub speed: f64, // bytes per second
    pub download_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthUsage {
    pub date: DateTime<Utc>,
    pub bytes_downloaded: u64,
    pub downloads_count: usize,
}

pub struct DownloadHistory {
    entries: Arc<RwLock<HashMap<Uuid, DownloadHistoryEntry>>>,
    url_index: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
    category_index: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
}

impl DownloadHistory {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            url_index: Arc::new(RwLock::new(HashMap::new())),
            category_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_entry(&self, entry: DownloadHistoryEntry) {
        let id = entry.id;
        let url = entry.url.clone();
        let category = entry.category.clone();
        
        // Add to main storage
        let mut entries = self.entries.write().await;
        entries.insert(id, entry.clone());
        drop(entries);
        
        // Update URL index
        let mut url_idx = self.url_index.write().await;
        url_idx.entry(url).or_insert_with(Vec::new).push(id);
        drop(url_idx);
        
        // Update category index
        if let Some(cat) = category {
            let mut cat_idx = self.category_index.write().await;
            cat_idx.entry(cat).or_insert_with(Vec::new).push(id);
        }
    }

    pub async fn get_entry(&self, id: Uuid) -> Option<DownloadHistoryEntry> {
        self.entries.read().await.get(&id).cloned()
    }

    pub async fn get_all_entries(&self) -> Vec<DownloadHistoryEntry> {
        let entries = self.entries.read().await;
        let mut list: Vec<_> = entries.values().cloned().collect();
        list.sort_by(|a, b| b.download_time.cmp(&a.download_time));
        list
    }

    pub async fn query(&self, query: HistoryQuery) -> Vec<DownloadHistoryEntry> {
        let entries = self.entries.read().await;
        let mut results = Vec::new();
        
        for entry in entries.values() {
            if query.url.as_ref().map_or(true, |u| entry.url.contains(u))
                && query.file_name.as_ref().map_or(true, |f| entry.file_name.contains(f))
                && query.category.as_ref().map_or(true, |c| entry.category.as_ref().map_or(false, |cat| cat == c))
                && query.status.as_ref().map_or(true, |s| *s == entry.status)
                && query.start_date.map_or(true, |sd| entry.download_time >= sd)
                && query.end_date.map_or(true, |ed| entry.download_time <= ed)
                && query.min_size.map_or(true, |min| entry.file_size >= min)
                && query.max_size.map_or(true, |max| entry.file_size <= max)
            {
                results.push(entry.clone());
            }
        }
        
        results.sort_by(|a, b| b.download_time.cmp(&a.download_time));
        results
    }

    pub async fn get_by_url(&self, url: &str) -> Vec<DownloadHistoryEntry> {
        let url_idx = self.url_index.read().await;
        if let Some(ids) = url_idx.get(url) {
            let entries = self.entries.read().await;
            ids.iter()
                .filter_map(|id| entries.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    pub async fn get_by_category(&self, category: &str) -> Vec<DownloadHistoryEntry> {
        let cat_idx = self.category_index.read().await;
        if let Some(ids) = cat_idx.get(category) {
            let entries = self.entries.read().await;
            ids.iter()
                .filter_map(|id| entries.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    pub async fn get_completed(&self) -> Vec<DownloadHistoryEntry> {
        let entries = self.entries.read().await;
        entries.values()
            .filter(|e| matches!(e.status, DownloadStatus::Completed))
            .cloned()
            .collect()
    }

    pub async fn get_failed(&self) -> Vec<DownloadHistoryEntry> {
        let entries = self.entries.read().await;
        entries.values()
            .filter(|e| matches!(e.status, DownloadStatus::Failed))
            .cloned()
            .collect()
    }

    pub async fn get_recent(&self, limit: usize) -> Vec<DownloadHistoryEntry> {
        let entries = self.entries.read().await;
        let mut recent: Vec<_> = entries.values().cloned().collect();
        recent.sort_by(|a, b| b.download_time.cmp(&a.download_time));
        recent.truncate(limit);
        recent
    }

    pub async fn delete_entry(&self, id: Uuid) -> Result<bool, String> {
        let entries = self.entries.read().await;
        
        if let Some(entry) = entries.get(&id) {
            let url = entry.url.clone();
            let category = entry.category.clone();
            drop(entries);
            
            let mut entries = self.entries.write().await;
            entries.remove(&id);
            
            // Update URL index
            let mut url_idx = self.url_index.write().await;
            if let Some(ids) = url_idx.get_mut(&url) {
                ids.retain(|&i| i != id);
                if ids.is_empty() {
                    url_idx.remove(&url);
                }
            }
            
            // Update category index
            if let Some(cat) = category {
                let mut cat_idx = self.category_index.write().await;
                if let Some(ids) = cat_idx.get_mut(&cat) {
                    ids.retain(|&i| i != id);
                    if ids.is_empty() {
                        cat_idx.remove(&cat);
                    }
                }
            }
            
            return Ok(true);
        }
        
        Ok(false)
    }

    pub async fn clear_old_entries(&self, days_old: i64) -> Result<usize, String> {
        let cutoff = Utc::now() - chrono::Duration::days(days_old);
        let entries = self.entries.read().await;
        
        let ids_to_delete: Vec<_> = entries.values()
            .filter(|e| e.download_time < cutoff)
            .map(|e| e.id)
            .collect();
        drop(entries);
        
        let mut deleted = 0;
        for id in ids_to_delete {
            if self.delete_entry(id).await.unwrap() {
                deleted += 1;
            }
        }
        
        Ok(deleted)
    }

    pub async fn clear_all(&self) -> Result<(), String> {
        self.entries.write().await.clear();
        self.url_index.write().await.clear();
        self.category_index.write().await.clear();
        Ok(())
    }

    pub async fn get_statistics(&self) -> HistoryStats {
        let entries = self.entries.read().await;
        
        let total_downloads = entries.len();
        let completed_downloads = entries.values()
            .filter(|e| matches!(e.status, DownloadStatus::Completed))
            .count();
        let failed_downloads = entries.values()
            .filter(|e| matches!(e.status, DownloadStatus::Failed))
            .count();
        let cancelled_downloads = entries.values()
            .filter(|e| matches!(e.status, DownloadStatus::Cancelled))
            .count();
        
        let total_downloaded_bytes: u64 = entries.values()
            .filter(|e| matches!(e.status, DownloadStatus::Completed))
            .map(|e| e.file_size)
            .sum();
        
        let mut fastest_download: Option<SpeedRecord> = None;
        let mut slowest_download: Option<SpeedRecord> = None;
        let mut total_speed = 0.0;
        let mut speed_count = 0;
        
        for entry in entries.values() {
            if let (Some(completed), Some(duration)) = (entry.completion_time, entry.duration_seconds) {
                let speed = entry.file_size as f64 / duration as f64;
                total_speed += speed;
                speed_count += 1;
                
                if fastest_download.is_none() || speed > fastest_download.as_ref().unwrap().speed {
                    fastest_download = Some(SpeedRecord {
                        url: entry.url.clone(),
                        file_name: entry.file_name.clone(),
                        speed,
                        download_time: entry.download_time,
                    });
                }
                
                if slowest_download.is_none() || speed < slowest_download.as_ref().unwrap().speed {
                    slowest_download = Some(SpeedRecord {
                        url: entry.url.clone(),
                        file_name: entry.file_name.clone(),
                        speed,
                        download_time: entry.download_time,
                    });
                }
            }
        }
        
        let average_speed = if speed_count > 0 {
            total_speed / speed_count as f64
        } else {
            0.0
        };
        
        HistoryStats {
            total_downloads,
            completed_downloads,
            failed_downloads,
            cancelled_downloads,
            total_downloaded_bytes,
            total_bandwidth_used: total_downloaded_bytes,
            average_speed,
            fastest_download,
            slowest_download,
        }
    }

    pub async fn get_bandwidth_usage(&self, days: i64) -> Vec<BandwidthUsage> {
        let entries = self.entries.read().await;
        let start_date = Utc::now() - chrono::Duration::days(days);
        
        let mut daily_usage: HashMap<DateTime<Utc>, (u64, usize)> = HashMap::new();
        
        for entry in entries.values() {
            if entry.download_time >= start_date && matches!(entry.status, DownloadStatus::Completed) {
                let date = entry.download_time.date_naive().and_hms(0, 0, 0);
                let datetime = DateTime::from_utc(date, Utc);
                
                let entry = daily_usage.entry(datetime).or_insert((0, 0));
                entry.0 += entry.file_size;
                entry.1 += 1;
            }
        }
        
        let mut usage: Vec<_> = daily_usage.into_iter()
            .map(|(date, (bytes, count))| BandwidthUsage {
                date,
                bytes_downloaded: bytes,
                downloads_count: count,
            })
            .collect();
        
        usage.sort_by(|a, b| a.date.cmp(&b.date));
        usage
    }

    pub async fn export_to_csv(&self) -> Result<String, String> {
        let entries = self.get_all_entries().await;
        
        let mut csv = String::from("URL,FileName,FileSize,DownloadTime,Status,Category\n");
        
        for entry in entries {
            let status_str = match entry.status {
                DownloadStatus::Completed => "Completed",
                DownloadStatus::Failed => "Failed",
                DownloadStatus::Cancelled => "Cancelled",
            };
            
            csv.push_str(&format!(
                "{},{},{},{},{},{}\n",
                entry.url,
                entry.file_name,
                entry.file_size,
                entry.download_time.to_rfc3339(),
                status_str,
                entry.category.unwrap_or_default()
            ));
        }
        
        Ok(csv)
    }

    pub async fn save(&self) -> Result<String, String> {
        let entries = self.entries.read().await;
        let data: Vec<_> = entries.values().cloned().collect();
        
        serde_json::to_string_pretty(&data)
            .map_err(|e| format!("Failed to serialize history: {}", e))
    }

    pub async fn load(&self, json: &str) -> Result<(), String> {
        let data: Vec<DownloadHistoryEntry> = serde_json::from_str(json)
            .map_err(|e| format!("Failed to deserialize history: {}", e))?;
        
        // Clear existing
        self.clear_all().await.unwrap();
        
        // Add new entries
        for entry in data {
            self.add_entry(entry).await;
        }
        
        Ok(())
    }
}

impl Default for DownloadHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_history_creation() {
        let history = DownloadHistory::new();
        let stats = history.get_statistics().await;
        assert_eq!(stats.total_downloads, 0);
    }

    #[tokio::test]
    async fn test_add_entry() {
        let history = DownloadHistory::new();
        
        let entry = DownloadHistoryEntry {
            id: Uuid::new_v4(),
            url: "https://example.com/file.zip".to_string(),
            file_name: "file.zip".to_string(),
            file_size: 1024 * 1024,
            save_path: PathBuf::from("/tmp/file.zip"),
            download_time: Utc::now(),
            completion_time: Some(Utc::now()),
            duration_seconds: Some(10),
            status: DownloadStatus::Completed,
            category: Some("Archives".to_string()),
            checksum: None,
            error_message: None,
        };
        
        history.add_entry(entry).await;
        
        let stats = history.get_statistics().await;
        assert_eq!(stats.total_downloads, 1);
    }

    #[tokio::test]
    async fn test_query_history() {
        let history = DownloadHistory::new();
        
        let entry = DownloadHistoryEntry {
            id: Uuid::new_v4(),
            url: "https://example.com/file.pdf".to_string(),
            file_name: "file.pdf".to_string(),
            file_size: 1024 * 1024,
            save_path: PathBuf::from("/tmp/file.pdf"),
            download_time: Utc::now(),
            completion_time: Some(Utc::now()),
            duration_seconds: Some(10),
            status: DownloadStatus::Completed,
            category: Some("Documents".to_string()),
            checksum: None,
            error_message: None,
        };
        
        history.add_entry(entry).await;
        
        let query = HistoryQuery {
            category: Some("Documents".to_string()),
            ..Default::default()
        };
        
        let results = history.query(query).await;
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_delete_entry() {
        let history = DownloadHistory::new();
        
        let entry = DownloadHistoryEntry {
            id: Uuid::new_v4(),
            url: "https://example.com/file.zip".to_string(),
            file_name: "file.zip".to_string(),
            file_size: 1024 * 1024,
            save_path: PathBuf::from("/tmp/file.zip"),
            download_time: Utc::now(),
            completion_time: Some(Utc::now()),
            duration_seconds: Some(10),
            status: DownloadStatus::Completed,
            category: None,
            checksum: None,
            error_message: None,
        };
        
        history.add_entry(entry.clone()).await;
        history.delete_entry(entry.id).await.unwrap();
        
        let stats = history.get_statistics().await;
        assert_eq!(stats.total_downloads, 0);
    }

    #[tokio::test]
    async fn test_get_bandwidth_usage() {
        let history = DownloadHistory::new();
        
        let entry = DownloadHistoryEntry {
            id: Uuid::new_v4(),
            url: "https://example.com/file.zip".to_string(),
            file_name: "file.zip".to_string(),
            file_size: 10 * 1024 * 1024, // 10 MB
            save_path: PathBuf::from("/tmp/file.zip"),
            download_time: Utc::now(),
            completion_time: Some(Utc::now()),
            duration_seconds: Some(10),
            status: DownloadStatus::Completed,
            category: None,
            checksum: None,
            error_message: None,
        };
        
        history.add_entry(entry).await;
        
        let usage = history.get_bandwidth_usage(7).await;
        assert!(!usage.is_empty());
    }

    #[tokio::test]
    async fn test_save_load() {
        let history = DownloadHistory::new();
        
        let entry = DownloadHistoryEntry {
            id: Uuid::new_v4(),
            url: "https://example.com/file.zip".to_string(),
            file_name: "file.zip".to_string(),
            file_size: 1024 * 1024,
            save_path: PathBuf::from("/tmp/file.zip"),
            download_time: Utc::now(),
            completion_time: Some(Utc::now()),
            duration_seconds: Some(10),
            status: DownloadStatus::Completed,
            category: None,
            checksum: None,
            error_message: None,
        };
        
        history.add_entry(entry).await;
        
        let json = history.save().await.unwrap();
        
        let new_history = DownloadHistory::new();
        new_history.load(&json).await.unwrap();
        
        let stats = new_history.get_statistics().await;
        assert_eq!(stats.total_downloads, 1);
    }
}

impl Default for HistoryQuery {
    fn default() -> Self {
        Self {
            url: None,
            file_name: None,
            category: None,
            status: None,
            start_date: None,
            end_date: None,
            min_size: None,
            max_size: None,
        }
    }
}