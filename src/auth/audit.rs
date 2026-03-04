// VantisWeb Browser - Security Audit Log
// Copyright (c) 2024 VantisCorp
// Security audit logging for tracking authentication and security events

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

/// Audit entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub user_id: String,
    pub session_id: Option<String>,
    pub ip_address: String,
    pub user_agent: String,
    pub device_id: Option<String>,
    pub success: bool,
    pub details: AuditDetails,
    pub severity: AuditSeverity,
}

/// Audit event types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    // Authentication events
    Login,
    Logout,
    LoginFailed,
    PasswordChanged,
    PasswordResetRequested,
    PasswordResetCompleted,
    
    // MFA events
    MFAEnabled,
    MFADisabled,
    MFAVerified,
    MFAFailed,
    TOTPSetup,
    TOTPVerified,
    TOTPFailed,
    WebAuthnRegistered,
    WebAuthnRemoved,
    WebAuthnVerified,
    WebAuthnFailed,
    
    // Session events
    SessionCreated,
    SessionExtended,
    SessionRevoked,
    SessionExpired,
    
    // Security events
    SecurityKeyAdded,
    SecurityKeyRemoved,
    BiometricEnabled,
    BiometricDisabled,
    
    // Account events
    AccountCreated,
    AccountDeleted,
    AccountLocked,
    AccountUnlocked,
    AccountRecoveryInitiated,
    AccountRecoveryCompleted,
    
    // Profile events
    ProfileAccessed,
    ProfileModified,
    ProfileDeleted,
    ProfileExported,
    
    // Sensitive operations
    SensitiveOperation,
    SettingsChanged,
    PermissionsModified,
    
    // Sync events
    SyncStarted,
    SyncCompleted,
    SyncFailed,
    
    // Unknown
    Unknown,
}

/// Audit details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditDetails {
    pub message: String,
    pub additional_data: Option<serde_json::Value>,
    pub failure_reason: Option<String>,
}

impl AuditDetails {
    pub fn new(message: String) -> Self {
        Self {
            message,
            additional_data: None,
            failure_reason: None,
        }
    }
    
    pub fn with_failure(mut self, reason: String) -> Self {
        self.failure_reason = Some(reason);
        self
    }
    
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.additional_data = Some(data);
        self
    }
}

/// Audit severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Audit log configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Maximum number of entries to keep
    pub max_entries: usize,
    
    /// Enable detailed logging
    pub detailed_logging: bool,
    
    /// Log IP addresses
    pub log_ip_addresses: bool,
    
    /// Log user agents
    pub log_user_agents: bool,
    
    /// Retention period in days
    pub retention_days: u32,
    
    /// Enable encryption for sensitive data
    pub encrypt_sensitive_data: bool,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            max_entries: 10000,
            detailed_logging: true,
            log_ip_addresses: true,
            log_user_agents: true,
            retention_days: 90,
            encrypt_sensitive_data: true,
        }
    }
}

/// Security audit log
pub struct AuditLog {
    config: AuditConfig,
    entries: Arc<RwLock<Vec<AuditEntry>>>,
    user_events: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl AuditLog {
    /// Create a new audit log
    pub fn new() -> Self {
        Self::with_config(AuditConfig::default())
    }
    
    /// Create with custom configuration
    pub fn with_config(config: AuditConfig) -> Self {
        Self {
            config,
            entries: Arc::new(RwLock::new(Vec::new())),
            user_events: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Log an audit event
    pub fn log_event(
        &self,
        event_type: AuditEventType,
        user_id: String,
        ip_address: String,
        user_agent: String,
        success: bool,
        details: AuditDetails,
    ) -> Result<(), AuditError> {
        let entry = AuditEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type,
            session_id: None,
            ip_address,
            user_agent,
            device_id: None,
            success,
            severity: self.determine_severity(&event_type, success),
            details,
        };
        
        let mut entries = self.entries.write().unwrap();
        let mut user_events = self.user_events.write().unwrap();
        
        // Enforce max entries
        if entries.len() >= self.config.max_entries {
            entries.remove(0);
        }
        
        entries.push(entry.clone());
        user_events
            .entry(user_id.clone())
            .or_insert_with(Vec::new)
            .push(entry.id.clone());
        
        Ok(())
    }
    
    /// Get all audit entries
    pub fn get_entries(&self) -> Vec<AuditEntry> {
        let entries = self.entries.read().unwrap();
        entries.clone()
    }
    
    /// Get entries for a specific user
    pub fn get_user_entries(&self, user_id: &str) -> Vec<AuditEntry> {
        let entries = self.entries.read().unwrap();
        let user_events = self.user_events.read().unwrap();
        
        if let Some(entry_ids) = user_events.get(user_id) {
            entry_ids
                .iter()
                .filter_map(|id| entries.iter().find(|e| &e.id == id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get entries by event type
    pub fn get_entries_by_type(&self, event_type: AuditEventType) -> Vec<AuditEntry> {
        let entries = self.entries.read().unwrap();
        entries
            .iter()
            .filter(|e| e.event_type == event_type)
            .cloned()
            .collect()
    }
    
    /// Get entries by severity
    pub fn get_entries_by_severity(&self, severity: AuditSeverity) -> Vec<AuditEntry> {
        let entries = self.entries.read().unwrap();
        entries
            .iter()
            .filter(|e| e.severity == severity)
            .cloned()
            .collect()
    }
    
    /// Get entries within a date range
    pub fn get_entries_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<AuditEntry> {
        let entries = self.entries.read().unwrap();
        entries
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .cloned()
            .collect()
    }
    
    /// Get failed login attempts
    pub fn get_failed_logins(&self, user_id: Option<&str>, since: DateTime<Utc>) -> Vec<AuditEntry> {
        let entries = self.entries.read().unwrap();
        
        entries
            .iter()
            .filter(|e| {
                e.event_type == AuditEventType::LoginFailed
                    && e.timestamp >= since
                    && user_id.map(|uid| uid == e.user_id).unwrap_or(true)
            })
            .cloned()
            .collect()
    }
    
    /// Get security events (Critical and Error severity)
    pub fn get_security_events(&self, user_id: Option<&str>) -> Vec<AuditEntry> {
        let entries = self.entries.read().unwrap();
        
        entries
            .iter()
            .filter(|e| {
                (e.severity == AuditSeverity::Critical || e.severity == AuditSeverity::Error)
                    && user_id.map(|uid| uid == e.user_id).unwrap_or(true)
            })
            .cloned()
            .collect()
    }
    
    /// Count events for a user
    pub fn count_user_events(&self, user_id: &str, event_type: AuditEventType) -> usize {
        let entries = self.entries.read().unwrap();
        entries
            .iter()
            .filter(|e| e.user_id == user_id && e.event_type == event_type)
            .count()
    }
    
    /// Get recent events
    pub fn get_recent_events(&self, count: usize) -> Vec<AuditEntry> {
        let entries = self.entries.read().unwrap();
        let len = entries.len();
        entries[len.saturating_sub(count)..].to_vec()
    }
    
    /// Clear all audit entries
    pub fn clear_all(&self) {
        let mut entries = self.entries.write().unwrap();
        let mut user_events = self.user_events.write().unwrap();
        
        entries.clear();
        user_events.clear();
    }
    
    /// Clear entries older than retention period
    pub fn clear_old_entries(&self) -> usize {
        let cutoff = Utc::now() - chrono::Duration::days(self.config.retention_days as i64);
        
        let mut entries = self.entries.write().unwrap();
        let mut user_events = self.user_events.write().unwrap();
        
        let initial_len = entries.len();
        entries.retain(|e| e.timestamp > cutoff);
        
        // Rebuild user events
        *user_events = HashMap::new();
        for entry in entries.iter() {
            user_events
                .entry(entry.user_id.clone())
                .or_insert_with(Vec::new)
                .push(entry.id.clone());
        }
        
        initial_len - entries.len()
    }
    
    /// Determine severity based on event type and success
    fn determine_severity(&self, event_type: &AuditEventType, success: bool) -> AuditSeverity {
        if !success {
            return match event_type {
                AuditEventType::LoginFailed
                | AuditEventType::MFAFailed
                | AuditEventType::TOTPFailed
                | AuditEventType::WebAuthnFailed
                | AuditEventType::AccountLocked => AuditSeverity::Warning,
                AuditEventType::SyncFailed | AuditEventType::SensitiveOperation => AuditSeverity::Error,
                _ => AuditSeverity::Warning,
            };
        }
        
        match event_type {
            AuditEventType::Login
            | AuditEventType::Logout
            | AuditEventType::SessionCreated
            | AuditEventType::SessionExpired
            | AuditEventType::ProfileAccessed => AuditSeverity::Info,
            
            AuditEventType::PasswordChanged
            | AuditEventType::MFAEnabled
            | AuditEventType::MFADisabled
            | AuditEventType::TOTPSetup
            | AuditEventType::WebAuthnRegistered
            | AuditEventType::WebAuthnRemoved
            | AuditEventType::AccountLocked
            | AuditEventType::AccountUnlocked => AuditSeverity::Warning,
            
            AuditEventType::AccountDeleted
            | AuditEventType::ProfileDeleted
            | AuditEventType::SyncFailed
            | AuditEventType::SensitiveOperation => AuditSeverity::Error,
            
            _ => AuditSeverity::Info,
        }
    }
    
    /// Export audit log to JSON
    pub fn export_to_json(&self) -> Result<String, AuditError> {
        let entries = self.entries.read().unwrap();
        serde_json::to_string_pretty(&*entries)
            .map_err(|e| AuditError::ExportFailed(e.to_string()))
    }
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new()
    }
}

/// Audit error types
#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("Export failed: {0}")]
    ExportFailed(String),
    
    #[error("Invalid configuration")]
    InvalidConfig,
    
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("Encryption error: {0}")]
    Encryption(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_audit_log() {
        let log = AuditLog::new();
        
        let details = AuditDetails::new("Test login".to_string());
        log.log_event(
            AuditEventType::Login,
            "user123".to_string(),
            "192.168.1.1".to_string(),
            "Mozilla/5.0".to_string(),
            true,
            details,
        ).unwrap();
        
        let entries = log.get_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].user_id, "user123");
    }
    
    #[test]
    fn test_user_entries() {
        let log = AuditLog::new();
        
        log.log_event(
            AuditEventType::Login,
            "user123".to_string(),
            "192.168.1.1".to_string(),
            "Mozilla/5.0".to_string(),
            true,
            AuditDetails::new("Test login".to_string()),
        ).unwrap();
        
        log.log_event(
            AuditEventType::Login,
            "user456".to_string(),
            "192.168.1.2".to_string(),
            "Chrome/1.0".to_string(),
            true,
            AuditDetails::new("Test login".to_string()),
        ).unwrap();
        
        let user_entries = log.get_user_entries("user123");
        assert_eq!(user_entries.len(), 1);
    }
    
    #[test]
    fn test_severity_determination() {
        let log = AuditLog::new();
        
        let failed_details = AuditDetails::new("Failed login".to_string())
            .with_failure("Invalid password".to_string());
        
        log.log_event(
            AuditEventType::LoginFailed,
            "user123".to_string(),
            "192.168.1.1".to_string(),
            "Mozilla/5.0".to_string(),
            false,
            failed_details,
        ).unwrap();
        
        let entries = log.get_entries();
        assert_eq!(entries[0].severity, AuditSeverity::Warning);
    }
}