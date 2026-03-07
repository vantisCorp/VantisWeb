use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc, TimeZone};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledDownload {
    pub id: Uuid,
    pub url: String,
    pub save_path: String,
    pub scheduled_time: DateTime<Utc>,
    pub repeat: RepeatOption,
    pub is_active: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RepeatOption {
    Once,
    Daily,
    Weekly,
    Monthly,
    Custom { interval_hours: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleRule {
    pub name: String,
    pub enabled: bool,
    pub conditions: Vec<ScheduleCondition>,
    pub actions: Vec<ScheduleAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScheduleCondition {
    TimeOfDay { hour: u8, minute: u8 },
    DayOfWeek { days: Vec<u8> }, // 0 = Sunday
    DayOfMonth { days: Vec<u8> },
    NetworkAvailable,
    BatteryAbove { percentage: u8 },
    StorageAbove { mb: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScheduleAction {
    StartAllPending,
    StartByCategory { category: String },
    LimitSpeed { bytes_per_second: f64 },
    PauseAll,
}

pub struct DownloadScheduler {
    scheduled_downloads: Arc<RwLock<HashMap<Uuid, ScheduledDownload>>>,
    rules: Arc<RwLock<Vec<ScheduleRule>>>,
    is_running: Arc<RwLock<bool>>,
}

impl DownloadScheduler {
    pub fn new() -> Self {
        Self {
            scheduled_downloads: Arc::new(RwLock::new(HashMap::new())),
            rules: Arc::new(RwLock::new(Vec::new())),
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn schedule_download(
        &self,
        url: String,
        save_path: String,
        scheduled_time: DateTime<Utc>,
    ) -> ScheduledDownload {
        self.schedule_download_with_options(
            url,
            save_path,
            scheduled_time,
            RepeatOption::Once,
        ).await
    }

    pub async fn schedule_download_with_options(
        &self,
        url: String,
        save_path: String,
        scheduled_time: DateTime<Utc>,
        repeat: RepeatOption,
    ) -> ScheduledDownload {
        let next_run = self.calculate_next_run(&scheduled_time, &repeat);
        
        let scheduled = ScheduledDownload {
            id: Uuid::new_v4(),
            url,
            save_path,
            scheduled_time,
            repeat,
            is_active: true,
            last_run: None,
            next_run,
            created_at: Utc::now(),
            metadata: HashMap::new(),
        };
        
        let id = scheduled.id;
        let mut downloads = self.scheduled_downloads.write().await;
        downloads.insert(id, scheduled.clone());
        
        scheduled
    }

    pub async fn get_scheduled_download(&self, id: Uuid) -> Option<ScheduledDownload> {
        self.scheduled_downloads.read().await.get(&id).cloned()
    }

    pub async fn get_all_scheduled_downloads(&self) -> Vec<ScheduledDownload> {
        self.scheduled_downloads.read().await.values().cloned().collect()
    }

    pub async fn get_active_scheduled(&self) -> Vec<ScheduledDownload> {
        self.scheduled_downloads.read().await.values()
            .filter(|s| s.is_active)
            .cloned()
            .collect()
    }

    pub async fn get_due_downloads(&self) -> Vec<ScheduledDownload> {
        let now = Utc::now();
        self.scheduled_downloads.read().await.values()
            .filter(|s| {
                s.is_active && s.next_run.map_or(false, |nr| nr <= now)
            })
            .cloned()
            .collect()
    }

    pub async fn update_scheduled(
        &self,
        id: Uuid,
        scheduled_time: Option<DateTime<Utc>>,
        repeat: Option<RepeatOption>,
    ) -> Result<ScheduledDownload, String> {
        let mut downloads = self.scheduled_downloads.write().await;
        if let Some(scheduled) = downloads.get_mut(&id) {
            if let Some(time) = scheduled_time {
                scheduled.scheduled_time = time;
            }
            if let Some(r) = repeat {
                scheduled.repeat = r;
            }
            scheduled.next_run = self.calculate_next_run(&scheduled.scheduled_time, &scheduled.repeat);
            
            return Ok(scheduled.clone());
        }
        Err("Scheduled download not found".to_string())
    }

    pub async fn activate_scheduled(&self, id: Uuid) -> Result<(), String> {
        let mut downloads = self.scheduled_downloads.write().await;
        if let Some(scheduled) = downloads.get_mut(&id) {
            scheduled.is_active = true;
            scheduled.next_run = self.calculate_next_run(&scheduled.scheduled_time, &scheduled.repeat);
            return Ok(());
        }
        Err("Scheduled download not found".to_string())
    }

    pub async fn deactivate_scheduled(&self, id: Uuid) -> Result<(), String> {
        let mut downloads = self.scheduled_downloads.write().await;
        if let Some(scheduled) = downloads.get_mut(&id) {
            scheduled.is_active = false;
            return Ok(());
        }
        Err("Scheduled download not found".to_string())
    }

    pub async fn delete_scheduled(&self, id: Uuid) -> Result<bool, String> {
        let mut downloads = self.scheduled_downloads.write().await;
        Ok(downloads.remove(&id).is_some())
    }

    pub async fn mark_completed(&self, id: Uuid) -> Result<ScheduledDownload, String> {
        let mut downloads = self.scheduled_downloads.write().await;
        if let Some(scheduled) = downloads.get_mut(&id) {
            scheduled.last_run = Some(Utc::now());
            
            // Calculate next run for repeating downloads
            if !matches!(scheduled.repeat, RepeatOption::Once) {
                scheduled.next_run = self.calculate_next_run(&scheduled.scheduled_time, &scheduled.repeat);
            } else {
                scheduled.is_active = false;
                scheduled.next_run = None;
            }
            
            return Ok(scheduled.clone());
        }
        Err("Scheduled download not found".to_string())
    }

    fn calculate_next_run(&self, scheduled_time: &DateTime<Utc>, repeat: &RepeatOption) -> Option<DateTime<Utc>> {
        let now = Utc::now();
        
        match repeat {
            RepeatOption::Once => {
                if scheduled_time > &now {
                    Some(*scheduled_time)
                } else {
                    None
                }
            }
            RepeatOption::Daily => {
                let mut next = scheduled_time.clone();
                while next <= now {
                    next = next + chrono::Duration::days(1);
                }
                Some(next)
            }
            RepeatOption::Weekly => {
                let mut next = scheduled_time.clone();
                while next <= now {
                    next = next + chrono::Duration::weeks(1);
                }
                Some(next)
            }
            RepeatOption::Monthly => {
                let mut next = scheduled_time.clone();
                while next <= now {
                    next = next + chrono::Duration::days(30);
                }
                Some(next)
            }
            RepeatOption::Custom { interval_hours } => {
                let mut next = scheduled_time.clone();
                while next <= now {
                    next = next + chrono::Duration::hours(*interval_hours as i64);
                }
                Some(next)
            }
        }
    }

    pub async fn add_rule(&self, rule: ScheduleRule) {
        self.rules.write().await.push(rule);
    }

    pub async fn get_rules(&self) -> Vec<ScheduleRule> {
        self.rules.read().await.clone()
    }

    pub async fn update_rule(&self, index: usize, rule: ScheduleRule) -> Result<(), String> {
        let mut rules = self.rules.write().await;
        if index < rules.len() {
            rules[index] = rule;
            return Ok(());
        }
        Err("Rule not found".to_string())
    }

    pub async fn delete_rule(&self, index: usize) -> Result<(), String> {
        let mut rules = self.rules.write().await;
        if index < rules.len() {
            rules.remove(index);
            return Ok(());
        }
        Err("Rule not found".to_string())
    }

    pub async fn start(&self) {
        *self.is_running.write().await = true;
    }

    pub async fn stop(&self) {
        *self.is_running.write().await = false;
    }

    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }

    pub async fn get_statistics(&self) -> SchedulerStats {
        let downloads = self.scheduled_downloads.read().await;
        let rules = self.rules.read().await;
        
        SchedulerStats {
            total_scheduled: downloads.len(),
            active_scheduled: downloads.values().filter(|s| s.is_active).count(),
            total_rules: rules.len(),
            enabled_rules: rules.iter().filter(|r| r.enabled).count(),
        }
    }

    pub async fn save(&self) -> Result<String, String> {
        let downloads = self.scheduled_downloads.read().await;
        let rules = self.rules.read().await;
        
        let data = ScheduleData {
            scheduled_downloads: downloads.values().cloned().collect(),
            rules: rules.clone(),
        };
        
        serde_json::to_string_pretty(&data)
            .map_err(|e| format!("Failed to serialize: {}", e))
    }

    pub async fn load(&self, json: &str) -> Result<(), String> {
        let data: ScheduleData = serde_json::from_str(json)
            .map_err(|e| format!("Failed to deserialize: {}", e))?;
        
        let mut downloads = self.scheduled_downloads.write().await;
        downloads.clear();
        for scheduled in data.scheduled_downloads {
            downloads.insert(scheduled.id, scheduled);
        }
        
        let mut rules = self.rules.write().await;
        *rules = data.rules;
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScheduleData {
    scheduled_downloads: Vec<ScheduledDownload>,
    rules: Vec<ScheduleRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStats {
    pub total_scheduled: usize,
    pub active_scheduled: usize,
    pub total_rules: usize,
    pub enabled_rules: usize,
}

impl Default for DownloadScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let scheduler = DownloadScheduler::new();
        let stats = scheduler.get_statistics().await;
        assert_eq!(stats.total_scheduled, 0);
    }

    #[tokio::test]
    async fn test_schedule_download() {
        let scheduler = DownloadScheduler::new();
        
        let future_time = Utc::now() + chrono::Duration::hours(1);
        let scheduled = scheduler.schedule_download(
            "https://example.com/file.zip".to_string(),
            "/tmp/file.zip".to_string(),
            future_time,
        ).await;
        
        assert!(scheduled.is_active);
        assert!(scheduled.next_run.is_some());
    }

    #[tokio::test]
    async fn test_get_due_downloads() {
        let scheduler = DownloadScheduler::new();
        
        // Schedule for past - should be due now
        let past_time = Utc::now() - chrono::Duration::hours(1);
        scheduler.schedule_download(
            "https://example.com/file.zip".to_string(),
            "/tmp/file.zip".to_string(),
            past_time,
        ).await;
        
        let due = scheduler.get_due_downloads().await;
        assert_eq!(due.len(), 1);
    }

    #[tokio::test]
    async fn test_deactivate_scheduled() {
        let scheduler = DownloadScheduler::new();
        
        let future_time = Utc::now() + chrono::Duration::hours(1);
        let scheduled = scheduler.schedule_download(
            "https://example.com/file.zip".to_string(),
            "/tmp/file.zip".to_string(),
            future_time,
        ).await;
        
        scheduler.deactivate_scheduled(scheduled.id).await.unwrap();
        
        let updated = scheduler.get_scheduled_download(scheduled.id).await.unwrap();
        assert!(!updated.is_active);
    }

    #[tokio::test]
    async fn test_repeating_schedule() {
        let scheduler = DownloadScheduler::new();
        
        let past_time = Utc::now() - chrono::Duration::days(1);
        let scheduled = scheduler.schedule_download_with_options(
            "https://example.com/file.zip".to_string(),
            "/tmp/file.zip".to_string(),
            past_time,
            RepeatOption::Daily,
        ).await;
        
        // Should calculate next run for tomorrow
        assert!(scheduled.next_run.is_some());
        let next_run = scheduled.next_run.unwrap();
        assert!(next_run > Utc::now());
    }

    #[tokio::test]
    async fn test_add_rule() {
        let scheduler = DownloadScheduler::new();
        
        let rule = ScheduleRule {
            name: "Night Downloads".to_string(),
            enabled: true,
            conditions: vec![ScheduleCondition::TimeOfDay { hour: 2, minute: 0 }],
            actions: vec![ScheduleAction::StartAllPending],
        };
        
        scheduler.add_rule(rule).await;
        
        let rules = scheduler.get_rules().await;
        assert_eq!(rules.len(), 1);
    }

    #[tokio::test]
    async fn test_save_load() {
        let scheduler = DownloadScheduler::new();
        
        let future_time = Utc::now() + chrono::Duration::hours(1);
        scheduler.schedule_download(
            "https://example.com/file.zip".to_string(),
            "/tmp/file.zip".to_string(),
            future_time,
        ).await;
        
        let json = scheduler.save().await.unwrap();
        
        let new_scheduler = DownloadScheduler::new();
        new_scheduler.load(&json).await.unwrap();
        
        let downloads = new_scheduler.get_all_scheduled_downloads().await;
        assert_eq!(downloads.len(), 1);
    }
}