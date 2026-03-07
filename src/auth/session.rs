// VantisWeb Browser - Session Management
// Copyright (c) 2024 VantisCorp
// Session management for authenticated users

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub device_id: String,
    pub device_name: String,
    pub device_type: DeviceType,
    pub ip_address: String,
    pub user_agent: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub is_active: bool,
    pub auth_methods: Vec<String>,
    pub location: Option<Location>,
}

/// Device type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    Desktop,
    Mobile,
    Tablet,
    Browser,
    Other,
}

/// Location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub country: String,
    pub city: Option<String>,
    pub region: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

impl Session {
    /// Create a new session
    pub fn new(
        user_id: String,
        device_id: String,
        device_name: String,
        device_type: DeviceType,
        ip_address: String,
        user_agent: String,
        duration_minutes: u32,
        auth_methods: Vec<String>,
    ) -> Self {
        let now = Utc::now();
        
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            device_id,
            device_name,
            device_type,
            ip_address,
            user_agent,
            created_at: now,
            expires_at: now + Duration::minutes(duration_minutes as i64),
            last_activity: now,
            is_active: true,
            auth_methods,
            location: None,
        }
    }
    
    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
    
    /// Check if session is valid
    pub fn is_valid(&self) -> bool {
        self.is_active && !self.is_expired()
    }
    
    /// Get time until expiration
    pub fn time_until_expiration(&self) -> Duration {
        self.expires_at.signed_duration_since(Utc::now())
    }
    
    /// Get session age
    pub fn age(&self) -> Duration {
        Utc::now().signed_duration_since(self.created_at)
    }
    
    /// Get time since last activity
    pub fn time_since_activity(&self) -> Duration {
        Utc::now().signed_duration_since(self.last_activity)
    }
    
    /// Extend session expiration
    pub fn extend(&mut self, duration_minutes: u32) {
        self.expires_at = Utc::now() + Duration::minutes(duration_minutes as i64);
    }
    
    /// Update last activity
    pub fn update_activity(&mut self) {
        self.last_activity = Utc::now();
    }
    
    /// Revoke session
    pub fn revoke(&mut self) {
        self.is_active = false;
    }
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Default session duration in minutes
    pub default_duration_minutes: u32,
    
    /// Maximum number of active sessions
    pub max_sessions: usize,
    
    /// Session cleanup interval in minutes
    pub cleanup_interval_minutes: u32,
    
    /// Require re-authentication for sensitive operations
    pub require_reauth_for_sensitive: bool,
    
    /// Enable device tracking
    pub enable_device_tracking: bool,
    
    /// Session timeout after inactivity
    pub inactivity_timeout_minutes: u32,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            default_duration_minutes: 30,
            max_sessions: 5,
            cleanup_interval_minutes: 60,
            require_reauth_for_sensitive: true,
            enable_device_tracking: true,
            inactivity_timeout_minutes: 15,
        }
    }
}

/// Session manager
pub struct SessionManager {
    config: SessionConfig,
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    user_sessions: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self::with_config(SessionConfig::default())
    }
    
    /// Create with custom configuration
    pub fn with_config(config: SessionConfig) -> Self {
        Self {
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            user_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Create a new session
    pub async fn create_session(
        &self,
        user_id: String,
        device_id: String,
        device_name: String,
        device_type: DeviceType,
        ip_address: String,
        user_agent: String,
        auth_methods: Vec<String>,
    ) -> Result<Session, SessionError> {
        // Check max sessions
        let user_sessions = self.user_sessions.read().unwrap();
        if let Some(session_ids) = user_sessions.get(&user_id) {
            if session_ids.len() >= self.config.max_sessions {
                drop(user_sessions);
                // Revoke oldest session
                self.revoke_oldest_session(&user_id).await?;
            }
        }
        drop(user_sessions);
        
        // Create session
        let session = Session::new(
            user_id.clone(),
            device_id,
            device_name,
            device_type,
            ip_address,
            user_agent,
            self.config.default_duration_minutes,
            auth_methods,
        );
        
        // Store session
        let mut sessions = self.sessions.write().unwrap();
        let mut user_sessions = self.user_sessions.write().unwrap();
        
        sessions.insert(session.id.clone(), session.clone());
        user_sessions
            .entry(user_id.clone())
            .or_insert_with(Vec::new)
            .push(session.id.clone());
        
        Ok(session)
    }
    
    /// Get a session by ID
    pub async fn get_session(&self, session_id: &str) -> Option<Session> {
        let sessions = self.sessions.read().unwrap();
        sessions.get(session_id).cloned()
    }
    
    /// Validate a session
    pub async fn validate_session(&self, session_id: &str) -> Result<Session, SessionError> {
        let session = self.get_session(session_id).await
            .ok_or(SessionError::NotFound)?;
        
        if !session.is_valid() {
            return Err(SessionError::Expired);
        }
        
        Ok(session)
    }
    
    /// Extend a session
    pub async fn extend_session(&self, session_id: &str, duration_minutes: u32) -> Result<(), SessionError> {
        let mut sessions = self.sessions.write().unwrap();
        
        if let Some(session) = sessions.get_mut(session_id) {
            if !session.is_valid() {
                return Err(SessionError::Expired);
            }
            session.extend(duration_minutes);
            Ok(())
        } else {
            Err(SessionError::NotFound)
        }
    }
    
    /// Update session activity
    pub async fn update_activity(&self, session_id: &str) -> Result<(), SessionError> {
        let mut sessions = self.sessions.write().unwrap();
        
        if let Some(session) = sessions.get_mut(session_id) {
            if !session.is_valid() {
                return Err(SessionError::Expired);
            }
            session.update_activity();
            Ok(())
        } else {
            Err(SessionError::NotFound)
        }
    }
    
    /// Revoke a session
    pub async fn revoke_session(&self, session_id: &str) -> Result<(), SessionError> {
        let mut sessions = self.sessions.write().unwrap();
        let mut user_sessions = self.user_sessions.write().unwrap();
        
        if let Some(session) = sessions.get_mut(session_id) {
            let user_id = session.user_id.clone();
            session.revoke();
            
            // Remove from user sessions
            if let Some(session_ids) = user_sessions.get_mut(&user_id) {
                session_ids.retain(|id| id != session_id);
            }
            
            Ok(())
        } else {
            Err(SessionError::NotFound)
        }
    }
    
    /// Revoke oldest session for a user
    pub async fn revoke_oldest_session(&self, user_id: &str) -> Result<(), SessionError> {
        let sessions = self.sessions.read().unwrap();
        let user_sessions = self.user_sessions.read().unwrap();
        
        if let Some(session_ids) = user_sessions.get(user_id) {
            if let Some(oldest_id) = session_ids.first() {
                drop(sessions);
                drop(user_sessions);
                return self.revoke_session(oldest_id).await;
            }
        }
        
        Err(SessionError::NotFound)
    }
    
    /// Revoke all sessions for a user
    pub async fn revoke_all_sessions(&self, user_id: &str) -> Result<(), SessionError> {
        let user_sessions = self.user_sessions.read().unwrap();
        
        if let Some(session_ids) = user_sessions.get(user_id) {
            let ids = session_ids.clone();
            drop(user_sessions);
            
            for session_id in ids {
                let _ = self.revoke_session(&session_id).await;
            }
            
            Ok(())
        } else {
            Err(SessionError::NotFound)
        }
    }
    
    /// Get all active sessions for a user
    pub async fn get_user_sessions(&self, user_id: &str) -> Vec<Session> {
        let sessions = self.sessions.read().unwrap();
        let user_sessions = self.user_sessions.read().unwrap();
        
        if let Some(session_ids) = user_sessions.get(user_id) {
            session_ids
                .iter()
                .filter_map(|id| sessions.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get all sessions
    pub async fn get_all_sessions(&self) -> Vec<Session> {
        let sessions = self.sessions.read().unwrap();
        sessions.values().cloned().collect()
    }
    
    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) -> usize {
        let mut sessions = self.sessions.write().unwrap();
        let mut user_sessions = self.user_sessions.write().unwrap();
        
        let mut expired_ids = Vec::new();
        
        for (id, session) in sessions.iter() {
            if session.is_expired() {
                expired_ids.push(id.clone());
            }
        }
        
        for id in &expired_ids {
            if let Some(session) = sessions.remove(id) {
                if let Some(session_ids) = user_sessions.get_mut(&session.user_id) {
                    session_ids.retain(|s_id| s_id != id);
                    if session_ids.is_empty() {
                        user_sessions.remove(&session.user_id);
                    }
                }
            }
        }
        
        expired_ids.len()
    }
    
    /// Get active session count
    pub async fn active_count(&self) -> usize {
        let sessions = self.sessions.read().unwrap();
        sessions.values().filter(|s| s.is_valid()).count()
    }
    
    /// Get session count for a user
    pub async fn user_session_count(&self, user_id: &str) -> usize {
        let sessions = self.sessions.read().unwrap();
        let user_sessions = self.user_sessions.read().unwrap();
        
        if let Some(session_ids) = user_sessions.get(user_id) {
            session_ids
                .iter()
                .filter(|id| {
                    sessions.get(*id).map(|s| s.is_valid()).unwrap_or(false)
                })
                .count()
        } else {
            0
        }
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Session error types
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Session not found")]
    NotFound,
    
    #[error("Session expired")]
    Expired,
    
    #[error("Session revoked")]
    Revoked,
    
    #[error("Maximum sessions reached")]
    MaxSessionsReached,
    
    #[error("Invalid session data")]
    InvalidData,
    
    #[error("Database error: {0}")]
    Database(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_session_creation() {
        let session = Session::new(
            "user123".to_string(),
            "device123".to_string(),
            "My Laptop".to_string(),
            DeviceType::Desktop,
            "192.168.1.1".to_string(),
            "Mozilla/5.0".to_string(),
            30,
            vec!["password".to_string(), "totp".to_string()],
        );
        
        assert!(session.is_valid());
        assert!(!session.is_expired());
    }
    
    #[tokio::test]
    async fn test_session_expiration() {
        let mut session = Session::new(
            "user123".to_string(),
            "device123".to_string(),
            "My Laptop".to_string(),
            DeviceType::Desktop,
            "192.168.1.1".to_string(),
            "Mozilla/5.0".to_string(),
            1, // 1 minute duration
            vec!["password".to_string()],
        );
        
        session.expires_at = Utc::now() - Duration::minutes(1);
        assert!(session.is_expired());
    }
    
    #[tokio::test]
    async fn test_session_manager() {
        let manager = SessionManager::new();
        
        let session = manager.create_session(
            "user123".to_string(),
            "device123".to_string(),
            "My Laptop".to_string(),
            DeviceType::Desktop,
            "192.168.1.1".to_string(),
            "Mozilla/5.0".to_string(),
            vec!["password".to_string()],
        ).await.unwrap();
        
        assert!(!session.id.is_empty());
        
        let retrieved = manager.get_session(&session.id).await;
        assert!(retrieved.is_some());
    }
}