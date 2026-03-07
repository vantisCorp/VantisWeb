use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadCategory {
    pub id: Uuid,
    pub name: String,
    pub extensions: Vec<String>,
    pub default_folder: PathBuf,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub rules: Vec<CategoryRule>,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryRule {
    pub field: RuleField,
    pub operator: RuleOperator,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleField {
    FileExtension,
    FileName,
    UrlPattern,
    MimeType,
    Size,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleOperator {
    Equals,
    Contains,
    StartsWith,
    EndsWith,
    Matches,
    GreaterThan,
    LessThan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadOrganizer {
    categories: Arc<RwLock<HashMap<Uuid, DownloadCategory>>>,
    default_category: Option<Uuid>,
    auto_organize: Arc<RwLock<bool>>,
}

impl DownloadOrganizer {
    pub fn new() -> Self {
        let mut categories = HashMap::new();
        
        // Add default categories
        let documents = DownloadCategory {
            id: Uuid::new_v4(),
            name: "Documents".to_string(),
            extensions: vec!["pdf".to_string(), "doc".to_string(), "docx".to_string(), 
                            "txt".to_string(), "xls".to_string(), "xlsx".to_string(),
                            "ppt".to_string(), "pptx".to_string()],
            default_folder: PathBuf::from("~/Downloads/Documents"),
            icon: Some("document".to_string()),
            color: Some("#3b82f6".to_string()),
            rules: Vec::new(),
            priority: 1,
        };
        
        let images = DownloadCategory {
            id: Uuid::new_v4(),
            name: "Images".to_string(),
            extensions: vec!["jpg".to_string(), "jpeg".to_string(), "png".to_string(),
                            "gif".to_string(), "svg".to_string(), "webp".to_string(),
                            "bmp".to_string(), "ico".to_string()],
            default_folder: PathBuf::from("~/Downloads/Images"),
            icon: Some("image".to_string()),
            color: Some("#22c55e".to_string()),
            rules: Vec::new(),
            priority: 1,
        };
        
        let videos = DownloadCategory {
            id: Uuid::new_v4(),
            name: "Videos".to_string(),
            extensions: vec!["mp4".to_string(), "avi".to_string(), "mkv".to_string(),
                            "mov".to_string(), "wmv".to_string(), "webm".to_string()],
            default_folder: PathBuf::from("~/Downloads/Videos"),
            icon: Some("video".to_string()),
            color: Some("#ef4444".to_string()),
            rules: Vec::new(),
            priority: 1,
        };
        
        let music = DownloadCategory {
            id: Uuid::new_v4(),
            name: "Music".to_string(),
            extensions: vec!["mp3".to_string(), "wav".to_string(), "flac".to_string(),
                            "aac".to_string(), "ogg".to_string(), "m4a".to_string()],
            default_folder: PathBuf::from("~/Downloads/Music"),
            icon: Some("music".to_string()),
            color: Some("#a855f7".to_string()),
            rules: Vec::new(),
            priority: 1,
        };
        
        let archives = DownloadCategory {
            id: Uuid::new_v4(),
            name: "Archives".to_string(),
            extensions: vec!["zip".to_string(), "rar".to_string(), "7z".to_string(),
                            "tar".to_string(), "gz".to_string(), "bz2".to_string()],
            default_folder: PathBuf::from("~/Downloads/Archives"),
            icon: Some("archive".to_string()),
            color: Some("#f97316".to_string()),
            rules: Vec::new(),
            priority: 1,
        };
        
        let software = DownloadCategory {
            id: Uuid::new_v4(),
            name: "Software".to_string(),
            extensions: vec!["exe".to_string(), "msi".to_string(), "dmg".to_string(),
                            "deb".to_string(), "rpm".to_string(), "AppImage".to_string()],
            default_folder: PathBuf::from("~/Downloads/Software"),
            icon: Some("application".to_string()),
            color: Some("#6366f1".to_string()),
            rules: Vec::new(),
            priority: 1,
        };
        
        let other = DownloadCategory {
            id: Uuid::new_v4(),
            name: "Other".to_string(),
            extensions: Vec::new(),
            default_folder: PathBuf::from("~/Downloads"),
            icon: Some("folder".to_string()),
            color: Some("#6b7280".to_string()),
            rules: Vec::new(),
            priority: 100,
        };
        
        let default_category = Some(other.id);
        
        categories.insert(documents.id, documents);
        categories.insert(images.id, images);
        categories.insert(videos.id, videos);
        categories.insert(music.id, music);
        categories.insert(archives.id, archives);
        categories.insert(software.id, software);
        categories.insert(other.id, other);
        
        Self {
            categories: Arc::new(RwLock::new(categories)),
            default_category,
            auto_organize: Arc::new(RwLock::new(true)),
        }
    }

    pub async fn get_category(&self, id: Uuid) -> Option<DownloadCategory> {
        self.categories.read().await.get(&id).cloned()
    }

    pub async fn get_all_categories(&self) -> Vec<DownloadCategory> {
        let categories = self.categories.read().await;
        let mut cats: Vec<_> = categories.values().cloned().collect();
        cats.sort_by(|a, b| a.priority.cmp(&b.priority));
        cats
    }

    pub async fn add_category(&self, category: DownloadCategory) {
        let mut categories = self.categories.write().await;
        categories.insert(category.id, category);
    }

    pub async fn update_category(&self, id: Uuid, category: DownloadCategory) -> Result<(), String> {
        let mut categories = self.categories.write().await;
        if categories.contains_key(&id) {
            categories.insert(id, category);
            return Ok(());
        }
        Err("Category not found".to_string())
    }

    pub async fn delete_category(&self, id: Uuid) -> Result<bool, String> {
        let mut categories = self.categories.write().await;
        if let Some(cat) = categories.get(&id) {
            if cat.name == "Other" {
                return Err("Cannot delete default category".to_string());
            }
        }
        Ok(categories.remove(&id).is_some())
    }

    pub async fn categorize_file(&self, filename: &str, url: &str, mime_type: Option<&str>) -> Option<DownloadCategory> {
        let categories = self.categories.read().await;
        
        // Get file extension
        let extension = filename.rsplit('.').next().map(|s| s.to_lowercase()).unwrap_or_default();
        
        // Check by extension first
        for category in categories.values() {
            if category.extensions.iter().any(|ext| ext.to_lowercase() == extension) {
                return Some(category.clone());
            }
        }
        
        // Check by rules
        for category in categories.values() {
            for rule in &category.rules {
                if self.matches_rule(rule, filename, url, mime_type) {
                    return Some(category.clone());
                }
            }
        }
        
        // Return default category
        categories.get(&self.default_category?).cloned()
    }

    fn matches_rule(&self, rule: &CategoryRule, filename: &str, url: &str, mime_type: Option<&str>) -> bool {
        let value = match rule.field {
            RuleField::FileExtension => filename.rsplit('.').next().unwrap_or("").to_lowercase(),
            RuleField::FileName => filename.to_lowercase(),
            RuleField::UrlPattern => url.to_lowercase(),
            RuleField::MimeType => mime_type.unwrap_or("").to_lowercase(),
            RuleField::Size => return false, // Size comparison would need file info
        };
        
        match rule.operator {
            RuleOperator::Equals => value.eq_ignore_ascii_case(&rule.value),
            RuleOperator::Contains => value.contains(&rule.value.to_lowercase()),
            RuleOperator::StartsWith => value.starts_with(&rule.value.to_lowercase()),
            RuleOperator::EndsWith => value.ends_with(&rule.value.to_lowercase()),
            RuleOperator::Matches => {
                // Simple glob-style matching
                if rule.value.starts_with('*') && rule.value.ends_with('*') {
                    let middle = &rule.value[1..rule.value.len()-1];
                    value.contains(middle)
                } else if rule.value.starts_with('*') {
                    value.ends_with(&rule.value[1..])
                } else if rule.value.ends_with('*') {
                    value.starts_with(&rule.value[..rule.value.len()-1])
                } else {
                    value == rule.value.to_lowercase()
                }
            }
            _ => false,
        }
    }

    pub async fn get_save_path(&self, filename: &str, url: &str, mime_type: Option<&str>) -> PathBuf {
        if let Some(category) = self.categorize_file(filename, url, mime_type).await {
            category.default_folder.join(filename)
        } else {
            PathBuf::from("~/Downloads").join(filename)
        }
    }

    pub async fn move_to_category(&self, filepath: &PathBuf, category_id: Uuid) -> Result<PathBuf, String> {
        let categories = self.categories.read().await;
        
        if let Some(category) = categories.get(&category_id) {
            let filename = filepath.file_name()
                .and_then(|n| n.to_str())
                .ok_or("Invalid filename")?;
            
            let new_path = category.default_folder.join(filename);
            
            // In production, would actually move the file
            // tokio::fs::rename(filepath, &new_path).await
            //     .map_err(|e| format!("Failed to move file: {}", e))?;
            
            return Ok(new_path);
        }
        
        Err("Category not found".to_string())
    }

    pub async fn set_auto_organize(&self, enabled: bool) {
        *self.auto_organize.write().await = enabled;
    }

    pub async fn is_auto_organize(&self) -> bool {
        *self.auto_organize.read().await
    }

    pub async fn create_category_folders(&self) -> Result<(), String> {
        let categories = self.categories.read().await;
        
        for category in categories.values() {
            tokio::fs::create_dir_all(&category.default_folder).await
                .map_err(|e| format!("Failed to create folder: {}", e))?;
        }
        
        Ok(())
    }

    pub async fn get_statistics(&self) -> OrganizerStats {
        let categories = self.categories.read().await;
        
        OrganizerStats {
            total_categories: categories.len(),
            auto_organize_enabled: *self.auto_organize.read().await,
        }
    }

    pub async fn export_rules(&self) -> Result<String, String> {
        let categories = self.categories.read().await;
        let data: Vec<_> = categories.values().cloned().collect();
        
        serde_json::to_string_pretty(&data)
            .map_err(|e| format!("Failed to serialize categories: {}", e))
    }

    pub async fn import_rules(&self, json: &str) -> Result<(), String> {
        let data: Vec<DownloadCategory> = serde_json::from_str(json)
            .map_err(|e| format!("Failed to deserialize categories: {}", e))?;
        
        let mut categories = self.categories.write().await;
        categories.clear();
        
        for category in data {
            categories.insert(category.id, category);
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizerStats {
    pub total_categories: usize,
    pub auto_organize_enabled: bool,
}

impl Default for DownloadOrganizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_organizer_creation() {
        let organizer = DownloadOrganizer::new();
        let categories = organizer.get_all_categories().await;
        
        assert!(!categories.is_empty());
        assert!(categories.iter().any(|c| c.name == "Documents"));
    }

    #[tokio::test]
    async fn test_categorize_by_extension() {
        let organizer = DownloadOrganizer::new();
        
        let category = organizer.categorize_file(
            "document.pdf",
            "https://example.com/document.pdf",
            None,
        ).await;
        
        assert!(category.is_some());
        assert_eq!(category.unwrap().name, "Documents");
    }

    #[tokio::test]
    async fn test_categorize_image() {
        let organizer = DownloadOrganizer::new();
        
        let category = organizer.categorize_file(
            "photo.jpg",
            "https://example.com/photo.jpg",
            None,
        ).await;
        
        assert!(category.is_some());
        assert_eq!(category.unwrap().name, "Images");
    }

    #[tokio::test]
    async fn test_categorize_video() {
        let organizer = DownloadOrganizer::new();
        
        let category = organizer.categorize_file(
            "video.mp4",
            "https://example.com/video.mp4",
            None,
        ).await;
        
        assert!(category.is_some());
        assert_eq!(category.unwrap().name, "Videos");
    }

    #[tokio::test]
    async fn test_categorize_unknown() {
        let organizer = DownloadOrganizer::new();
        
        let category = organizer.categorize_file(
            "unknown.xyz",
            "https://example.com/unknown.xyz",
            None,
        ).await;
        
        assert!(category.is_some());
        assert_eq!(category.unwrap().name, "Other");
    }

    #[tokio::test]
    async fn test_get_save_path() {
        let organizer = DownloadOrganizer::new();
        
        let path = organizer.get_save_path(
            "report.pdf",
            "https://example.com/report.pdf",
            None,
        ).await;
        
        assert!(path.to_str().unwrap().contains("Documents"));
    }

    #[tokio::test]
    async fn test_add_category() {
        let organizer = DownloadOrganizer::new();
        
        let custom = DownloadCategory {
            id: Uuid::new_v4(),
            name: "Custom".to_string(),
            extensions: vec!["xyz".to_string()],
            default_folder: PathBuf::from("~/Downloads/Custom"),
            icon: None,
            color: None,
            rules: Vec::new(),
            priority: 1,
        };
        
        organizer.add_category(custom.clone()).await;
        
        let category = organizer.get_category(custom.id).await;
        assert!(category.is_some());
    }

    #[tokio::test]
    async fn test_auto_organize() {
        let organizer = DownloadOrganizer::new();
        
        assert!(organizer.is_auto_organize().await);
        
        organizer.set_auto_organize(false).await;
        assert!(!organizer.is_auto_organize().await);
    }

    #[tokio::test]
    async fn test_export_import_rules() {
        let organizer = DownloadOrganizer::new();
        
        let json = organizer.export_rules().await.unwrap();
        
        let new_organizer = DownloadOrganizer::new();
        new_organizer.import_rules(&json).await.unwrap();
        
        let categories = new_organizer.get_all_categories().await;
        assert!(!categories.is_empty());
    }
}