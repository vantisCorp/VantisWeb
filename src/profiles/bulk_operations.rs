use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::profiles::profile_config::ProfileConfig;

/// Bulk operation type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BulkOperationType {
    /// Delete multiple profiles
    Delete,
    /// Export multiple profiles
    Export,
    /// Apply settings to multiple profiles
    ApplySettings,
    /// Change security level for multiple profiles
    ChangeSecurity,
    /// Clone/duplicate multiple profiles
    Clone,
}

/// Bulk operation result for a single profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOperationResult {
    /// Profile ID
    pub profile_id: String,
    /// Profile name
    pub profile_name: String,
    /// Whether the operation succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Additional data (e.g., new profile ID for clone)
    pub data: Option<serde_json::Value>,
}

/// Bulk operation status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BulkOperationStatus {
    /// Operation is pending
    Pending,
    /// Operation is in progress
    InProgress,
    /// Operation completed successfully
    Completed,
    /// Operation completed with partial failures
    PartialSuccess,
    /// Operation failed
    Failed,
    /// Operation was cancelled
    Cancelled,
}

/// Bulk operation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOperationRequest {
    /// Unique operation ID
    pub operation_id: String,
    /// Operation type
    pub operation_type: BulkOperationType,
    /// Profile IDs to operate on
    pub profile_ids: Vec<String>,
    /// Operation parameters
    pub parameters: BulkOperationParameters,
    /// Timestamp when operation was created
    pub created_at: DateTime<Utc>,
}

/// Parameters for bulk operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOperationParameters {
    /// Settings to apply (for ApplySettings)
    pub settings: Option<HashMap<String, serde_json::Value>>,
    /// Security level (for ChangeSecurity)
    pub security_level: Option<String>,
    /// Clone options (for Clone)
    pub clone_options: Option<CloneOptions>,
    /// Export format (for Export)
    pub export_format: Option<String>,
    /// Export path (for Export)
    pub export_path: Option<String>,
    /// Include bookmarks in export
    pub include_bookmarks: bool,
    /// Include history in export
    pub include_history: bool,
}

impl Default for BulkOperationParameters {
    fn default() -> Self {
        Self {
            settings: None,
            security_level: None,
            clone_options: None,
            export_format: Some("json".to_string()),
            export_path: None,
            include_bookmarks: true,
            include_history: false,
        }
    }
}

/// Clone options for bulk operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneOptions {
    /// New name prefix/suffix
    pub name_pattern: String,
    /// Include bookmarks
    pub include_bookmarks: bool,
    /// Include history
    pub include_history: bool,
    /// Include settings
    pub include_settings: bool,
}

impl Default for CloneOptions {
    fn default() -> Self {
        Self {
            name_pattern: " (Copy)".to_string(),
            include_bookmarks: true,
            include_history: false,
            include_settings: true,
        }
    }
}

/// Bulk operation with progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOperation {
    /// Operation ID
    pub id: String,
    /// Operation type
    pub operation_type: BulkOperationType,
    /// Total profiles to process
    pub total_profiles: u32,
    /// Profiles processed so far
    pub processed_profiles: u32,
    /// Successful operations
    pub successful_count: u32,
    /// Failed operations
    pub failed_count: u32,
    /// Current status
    pub status: BulkOperationStatus,
    /// Results for each profile
    pub results: Vec<BulkOperationResult>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Started timestamp
    pub started_at: Option<DateTime<Utc>>,
    /// Completed timestamp
    pub completed_at: Option<DateTime<Utc>>,
    /// Error message (for failed operations)
    pub error: Option<String>,
}

impl BulkOperation {
    /// Calculate progress percentage
    pub fn progress(&self) -> f64 {
        if self.total_profiles == 0 {
            return 100.0;
        }
        (self.processed_profiles as f64 / self.total_profiles as f64) * 100.0
    }

    /// Check if operation is complete
    pub fn is_complete(&self) -> bool {
        matches!(self.status, BulkOperationStatus::Completed | BulkOperationStatus::PartialSuccess | BulkOperationStatus::Failed | BulkOperationStatus::Cancelled)
    }
}

/// Bulk operations manager
pub struct BulkOperationsManager;

impl BulkOperationsManager {
    /// Validate bulk delete operation
    pub fn validate_delete(profiles: &[ProfileConfig], profile_ids: &[String]) -> Result<(), String> {
        if profile_ids.is_empty() {
            return Err("No profiles selected for deletion".to_string());
        }

        // Check if trying to delete all profiles
        if profile_ids.len() >= profiles.len() {
            return Err("Cannot delete all profiles. At least one profile must remain.".to_string());
        }

        // Check if trying to delete active profile
        for profile in profiles {
            if profile_ids.contains(&profile.id) && profile.active {
                return Err(format!("Cannot delete active profile '{}'. Please switch to another profile first.", profile.name));
            }
        }

        Ok(())
    }

    /// Validate bulk export operation
    pub fn validate_export(profiles: &[ProfileConfig], profile_ids: &[String]) -> Result<(), String> {
        if profile_ids.is_empty() {
            return Err("No profiles selected for export".to_string());
        }

        // Verify all profiles exist
        let profile_set: std::collections::HashSet<&String> = profiles.iter().map(|p| &p.id).collect();
        for id in profile_ids {
            if !profile_set.contains(id) {
                return Err(format!("Profile with ID '{}' not found", id));
            }
        }

        Ok(())
    }

    /// Validate bulk clone operation
    pub fn validate_clone(profiles: &[ProfileConfig], profile_ids: &[String]) -> Result<(), String> {
        if profile_ids.is_empty() {
            return Err("No profiles selected for cloning".to_string());
        }

        // Verify all profiles exist
        let profile_set: std::collections::HashSet<&String> = profiles.iter().map(|p| &p.id).collect();
        for id in profile_ids {
            if !profile_set.contains(id) {
                return Err(format!("Profile with ID '{}' not found", id));
            }
        }

        Ok(())
    }

    /// Create bulk operation request
    pub fn create_request(
        operation_type: BulkOperationType,
        profile_ids: Vec<String>,
        parameters: BulkOperationParameters,
    ) -> BulkOperationRequest {
        BulkOperationRequest {
            operation_id: Uuid::new_v4().to_string(),
            operation_type,
            profile_ids,
            parameters,
            created_at: Utc::now(),
        }
    }

    /// Initialize bulk operation
    pub fn initialize_operation(request: &BulkOperationRequest, total_profiles: u32) -> BulkOperation {
        BulkOperation {
            id: request.operation_id.clone(),
            operation_type: request.operation_type.clone(),
            total_profiles,
            processed_profiles: 0,
            successful_count: 0,
            failed_count: 0,
            status: BulkOperationStatus::Pending,
            results: vec![],
            created_at: request.created_at,
            started_at: None,
            completed_at: None,
            error: None,
        }
    }

    /// Validate bulk apply settings operation
    pub fn validate_apply_settings(
        profiles: &[ProfileConfig],
        profile_ids: &[String],
        settings: &HashMap<String, serde_json::Value>,
    ) -> Result<(), String> {
        if profile_ids.is_empty() {
            return Err("No profiles selected for applying settings".to_string());
        }

        if settings.is_empty() {
            return Err("No settings provided to apply".to_string());
        }

        // Verify all profiles exist
        let profile_set: std::collections::HashSet<&String> = profiles.iter().map(|p| &p.id).collect();
        for id in profile_ids {
            if !profile_set.contains(id) {
                return Err(format!("Profile with ID '{}' not found", id));
            }
        }

        Ok(())
    }

    /// Validate bulk change security operation
    pub fn validate_change_security(
        profiles: &[ProfileConfig],
        profile_ids: &[String],
        security_level: &str,
    ) -> Result<(), String> {
        if profile_ids.is_empty() {
            return Err("No profiles selected for changing security level".to_string());
        }

        if security_level.is_empty() {
            return Err("Security level not provided".to_string());
        }

        // Validate security level
        let valid_levels = ["none", "low", "medium", "high", "strict"];
        if !valid_levels.contains(&security_level.to_lowercase().as_str()) {
            return Err(format!("Invalid security level '{}'. Valid levels: {:?}", security_level, valid_levels));
        }

        // Verify all profiles exist
        let profile_set: std::collections::HashSet<&String> = profiles.iter().map(|p| &p.id).collect();
        for id in profile_ids {
            if !profile_set.contains(id) {
                return Err(format!("Profile with ID '{}' not found", id));
            }
        }

        Ok(())
    }

    /// Generate operation summary
    pub fn generate_summary(operation: &BulkOperation) -> String {
        let type_str = match operation.operation_type {
            BulkOperationType::Delete => "Delete",
            BulkOperationType::Export => "Export",
            BulkOperationType::ApplySettings => "Apply Settings",
            BulkOperationType::ChangeSecurity => "Change Security",
            BulkOperationType::Clone => "Clone",
        };

        match operation.status {
            BulkOperationStatus::Completed => {
                format!(
                    "{} operation completed successfully. Processed {}/{} profiles.",
                    type_str, operation.successful_count, operation.total_profiles
                )
            }
            BulkOperationStatus::PartialSuccess => {
                format!(
                    "{} operation partially completed. Success: {}, Failed: {}/{} profiles.",
                    type_str, operation.successful_count, operation.failed_count, operation.total_profiles
                )
            }
            BulkOperationStatus::Failed => {
                format!(
                    "{} operation failed. Error: {}",
                    type_str, operation.error.as_deref().unwrap_or("Unknown error")
                )
            }
            BulkOperationStatus::Cancelled => {
                format!(
                    "{} operation cancelled. Processed {}/{} profiles.",
                    type_str, operation.processed_profiles, operation.total_profiles
                )
            }
            _ => format!(
                "{} operation in progress. Progress: {:.1}%",
                type_str, operation.progress()
            ),
        }
    }
}

/// Confirmation dialog data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationDialog {
    /// Dialog title
    pub title: String,
    /// Dialog message
    pub message: String,
    /// Profile names affected
    pub profile_names: Vec<String>,
    /// Operation type
    pub operation_type: BulkOperationType,
    /// Warnings or additional info
    pub warnings: Vec<String>,
}

impl ConfirmationDialog {
    /// Create confirmation dialog for delete
    pub fn for_delete(profile_names: Vec<String>) -> Self {
        Self {
            title: "Confirm Bulk Delete".to_string(),
            message: format!("Are you sure you want to delete {} profile(s)?", profile_names.len()),
            profile_names,
            operation_type: BulkOperationType::Delete,
            warnings: vec!["This action cannot be undone.".to_string()],
        }
    }

    /// Create confirmation dialog for export
    pub fn for_export(profile_names: Vec<String>, include_history: bool) -> Self {
        let mut warnings = vec![];
        if include_history {
            warnings.push("Browsing history will be included in the export.".to_string());
        }

        Self {
            title: "Confirm Bulk Export".to_string(),
            message: format!("Export {} profile(s) to file?", profile_names.len()),
            profile_names,
            operation_type: BulkOperationType::Export,
            warnings,
        }
    }

    /// Create confirmation dialog for clone
    pub fn for_clone(profile_names: Vec<String>) -> Self {
        Self {
            title: "Confirm Bulk Clone".to_string(),
            message: format!("Clone {} profile(s)?", profile_names.len()),
            profile_names,
            operation_type: BulkOperationType::Clone,
            warnings: vec!["New profiles will be created with '(Copy)' suffix.".to_string()],
        }
    }

    /// Create confirmation dialog for apply settings
    pub fn for_apply_settings(profile_names: Vec<String>, settings_count: usize) -> Self {
        Self {
            title: "Confirm Apply Settings".to_string(),
            message: format!("Apply {} setting(s) to {} profile(s)?", settings_count, profile_names.len()),
            profile_names,
            operation_type: BulkOperationType::ApplySettings,
            warnings: vec!["This will override existing settings.".to_string()],
        }
    }

    /// Create confirmation dialog for change security
    pub fn for_change_security(profile_names: Vec<String>, security_level: &str) -> Self {
        Self {
            title: "Confirm Security Change".to_string(),
            message: format!("Set security level to '{}' for {} profile(s)?", security_level, profile_names.len()),
            profile_names,
            operation_type: BulkOperationType::ChangeSecurity,
            warnings: vec!["This may affect security settings.".to_string()],
        }
    }
}

/// Progress update for bulk operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    /// Operation ID
    pub operation_id: String,
    /// Current profile being processed
    pub current_profile_name: String,
    /// Profiles processed so far
    pub processed: u32,
    /// Total profiles
    pub total: u32,
    /// Percentage complete
    pub percentage: f64,
    /// Current status message
    pub status_message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_profile(name: &str, id: &str, active: bool) -> ProfileConfig {
        ProfileConfig {
            id: id.to_string(),
            name: name.to_string(),
            profile_type: crate::profiles::ProfileType::Custom("Test".to_string()),
            icon: None,
            color: None,
            active,
            order: 0,
            settings: HashMap::new(),
            bookmarks: vec![],
            history: vec![],
            extensions: vec![],
            theme: None,
            created_at: 0,
            last_used_at: 0,
        }
    }

    #[test]
    fn test_validate_delete_success() {
        let profiles = vec![
            create_test_profile("Profile 1", "1", false),
            create_test_profile("Profile 2", "2", false),
            create_test_profile("Active", "3", true),
        ];

        let result = BulkOperationsManager::validate_delete(&profiles, &["1".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_delete_all_profiles() {
        let profiles = vec![
            create_test_profile("Profile 1", "1", false),
            create_test_profile("Profile 2", "2", false),
        ];

        let result = BulkOperationsManager::validate_delete(&profiles, &["1".to_string(), "2".to_string()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot delete all profiles"));
    }

    #[test]
    fn test_validate_delete_active_profile() {
        let profiles = vec![
            create_test_profile("Profile 1", "1", false),
            create_test_profile("Active", "3", true),
        ];

        let result = BulkOperationsManager::validate_delete(&profiles, &["3".to_string()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot delete active profile"));
    }

    #[test]
    fn test_bulk_operation_progress() {
        let mut operation = BulkOperation {
            id: "test".to_string(),
            operation_type: BulkOperationType::Delete,
            total_profiles: 10,
            processed_profiles: 5,
            successful_count: 5,
            failed_count: 0,
            status: BulkOperationStatus::InProgress,
            results: vec![],
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            error: None,
        };

        assert_eq!(operation.progress(), 50.0);
        assert!(!operation.is_complete());
    }

    #[test]
    fn test_confirmation_dialog_delete() {
        let dialog = ConfirmationDialog::for_delete(vec!["Profile 1".to_string(), "Profile 2".to_string()]);
        
        assert_eq!(dialog.title, "Confirm Bulk Delete");
        assert_eq!(dialog.operation_type, BulkOperationType::Delete);
        assert_eq!(dialog.profile_names.len(), 2);
        assert!(dialog.warnings.iter().any(|w| w.contains("cannot be undone")));
    }

    #[test]
    fn test_create_request() {
        let request = BulkOperationsManager::create_request(
            BulkOperationType::Clone,
            vec!["1".to_string(), "2".to_string()],
            BulkOperationParameters::default(),
        );

        assert_eq!(request.operation_type, BulkOperationType::Clone);
        assert_eq!(request.profile_ids.len(), 2);
        assert!(!request.operation_id.is_empty());
    }
}