//! Mobile Authentication Manager
//! 
//! Handles authentication for mobile devices.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use anyhow::{Result, Context};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

use super::{MobileConfig, DeviceType};
use super::models::{Permission, BiometricType};

/// Mobile authentication manager
pub struct MobileAuthManager {
    /// Active sessions
    sessions: RwLock<HashMap<String, MobileSession>>,
    /// Auth tokens
    tokens: RwLock<HashMap<String, TokenInfo>>,
    /// Pending challenges
    challenges: RwLock<HashMap<String, AuthChallenge>>,
    /// Event sender
    event_sender: Option<broadcast::Sender<super::MobileEvent>>,
    /// Configuration
    config: RwLock<MobileConfig>,
}

/// Mobile session
#[derive(Debug, Clone)]
pub struct MobileSession {
    /// Session ID
    pub id: String,
    /// Device ID
    pub device_id: String,
    /// User ID (if authenticated)
    pub user_id: Option<String>,
    /// Permissions granted
    pub permissions: Vec<Permission>,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Expires at
    pub expires_at: DateTime<Utc>,
    /// Last activity
    pub last_activity: DateTime<Utc>,
    /// Is active
    pub is_active: bool,
    /// Biometric verified
    pub biometric_verified: bool,
}

/// Token information
#[derive(Debug, Clone)]
pub struct TokenInfo {
    /// Token value
    pub token: String,
    /// Device ID
    pub device_id: String,
    /// Token type
    pub token_type: TokenType,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Expires at
    pub expires_at: DateTime<Utc>,
    /// Is valid
    pub is_valid: bool,
}

/// Token types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    Device,
    Session,
    Refresh,
    Pairing,
}

/// Authentication challenge
#[derive(Debug, Clone)]
pub struct AuthChallenge {
    /// Challenge ID
    pub id: String,
    /// Device ID
    pub device_id: String,
    /// Challenge data
    pub challenge: String,
    /// Challenge type
    pub challenge_type: ChallengeType,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Expires at
    pub expires_at: DateTime<Utc>,
    /// Attempts remaining
    pub attempts_remaining: u32,
}

/// Challenge types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChallengeType {
    PairingCode,
    Biometric,
    Pin,
    Password,
    Totp,
}

/// Authentication configuration
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// Session duration in hours
    pub session_duration_hours: i64,
    /// Token expiration in days
    pub token_expiration_days: i64,
    /// Max login attempts
    pub max_login_attempts: u32,
    /// Lockout duration in minutes
    pub lockout_duration_minutes: i64,
    /// Require biometric for sensitive operations
    pub require_biometric_for_sensitive: bool,
    /// Allowed biometric types
    pub allowed_biometrics: Vec<BiometricType>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            session_duration_hours: 24,
            token_expiration_days: 30,
            max_login_attempts: 5,
            lockout_duration_minutes: 15,
            require_biometric_for_sensitive: true,
            allowed_biometrics: vec![
                BiometricType::Fingerprint,
                BiometricType::FaceId,
            ],
        }
    }
}

impl MobileAuthManager {
    /// Create a new auth manager
    pub async fn new(config: MobileConfig) -> Result<Self> {
        Ok(Self {
            sessions: RwLock::new(HashMap::new()),
            tokens: RwLock::new(HashMap::new()),
            challenges: RwLock::new(HashMap::new()),
            event_sender: None,
            config: RwLock::new(config),
        })
    }
    
    /// Create with event sender
    pub async fn with_event_sender(
        config: MobileConfig,
        event_sender: broadcast::Sender<super::MobileEvent>,
    ) -> Result<Self> {
        Ok(Self {
            sessions: RwLock::new(HashMap::new()),
            tokens: RwLock::new(HashMap::new()),
            challenges: RwLock::new(HashMap::new()),
            event_sender: Some(event_sender),
            config: RwLock::new(config),
        })
    }
    
    /// Create device token
    pub async fn create_device_token(&self, device_id: &str) -> Result<String> {
        let token = Self::generate_token();
        
        let token_info = TokenInfo {
            token: token.clone(),
            device_id: device_id.to_string(),
            token_type: TokenType::Device,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::days(365),
            is_valid: true,
        };
        
        let mut tokens = self.tokens.write().await;
        tokens.insert(token.clone(), token_info);
        
        tracing::info!("Device token created for: {}", device_id);
        Ok(token)
    }
    
    /// Validate device token
    pub async fn validate_device_token(&self, token: &str) -> Result<Option<String>> {
        let tokens = self.tokens.read().await;
        
        if let Some(info) = tokens.get(token) {
            if info.is_valid && info.expires_at > Utc::now() {
                return Ok(Some(info.device_id.clone()));
            }
        }
        
        Ok(None)
    }
    
    /// Revoke device token
    pub async fn revoke_device_token(&self, token: &str) -> Result<()> {
        let mut tokens = self.tokens.write().await;
        
        if let Some(info) = tokens.get_mut(token) {
            info.is_valid = false;
            tracing::info!("Device token revoked for: {}", info.device_id);
        }
        
        Ok(())
    }
    
    /// Create session
    pub async fn create_session(
        &self,
        device_id: &str,
        permissions: Vec<Permission>,
    ) -> Result<MobileSession> {
        let config = self.config.read().await;
        let now = Utc::now();
        
        let session = MobileSession {
            id: Uuid::new_v4().to_string(),
            device_id: device_id.to_string(),
            user_id: None,
            permissions,
            created_at: now,
            expires_at: now + Duration::hours(24),
            last_activity: now,
            is_active: true,
            biometric_verified: false,
        };
        
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id.clone(), session.clone());
        
        tracing::info!("Session created: {} for device: {}", session.id, device_id);
        Ok(session)
    }
    
    /// Validate session
    pub async fn validate_session(&self, session_id: &str) -> Result<Option<MobileSession>> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            if session.is_active && session.expires_at > Utc::now() {
                session.last_activity = Utc::now();
                return Ok(Some(session.clone()));
            }
        }
        
        Ok(None)
    }
    
    /// End session
    pub async fn end_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.remove(session_id) {
            tracing::info!("Session ended: {} for device: {}", session_id, session.device_id);
        }
        
        Ok(())
    }
    
    /// Check permission
    pub async fn check_permission(
        &self,
        session_id: &str,
        permission: &Permission,
    ) -> Result<bool> {
        let sessions = self.sessions.read().await;
        
        if let Some(session) = sessions.get(session_id) {
            return Ok(session.permissions.contains(permission) ||
                session.permissions.contains(&Permission::FullAccess));
        }
        
        Ok(false)
    }
    
    /// Create auth challenge
    pub async fn create_challenge(
        &self,
        device_id: &str,
        challenge_type: ChallengeType,
    ) -> Result<AuthChallenge> {
        let challenge = AuthChallenge {
            id: Uuid::new_v4().to_string(),
            device_id: device_id.to_string(),
            challenge: Self::generate_challenge_code(),
            challenge_type,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::minutes(5),
            attempts_remaining: 3,
        };
        
        let mut challenges = self.challenges.write().await;
        challenges.insert(challenge.id.clone(), challenge.clone());
        
        Ok(challenge)
    }
    
    /// Validate challenge response
    pub async fn validate_challenge(
        &self,
        challenge_id: &str,
        response: &str,
    ) -> Result<bool> {
        let mut challenges = self.challenges.write().await;
        
        if let Some(challenge) = challenges.get_mut(challenge_id) {
            // Check expiration
            if challenge.expires_at < Utc::now() {
                return Ok(false);
            }
            
            // Check attempts
            if challenge.attempts_remaining == 0 {
                return Ok(false);
            }
            
            // Validate response
            let valid = challenge.challenge == response;
            
            if !valid {
                challenge.attempts_remaining -= 1;
            }
            
            if valid {
                challenges.remove(challenge_id);
            }
            
            return Ok(valid);
        }
        
        Ok(false)
    }
    
    /// Create pairing code
    pub async fn create_pairing_code(&self, device_id: &str) -> Result<String> {
        let code = Self::generate_pairing_code();
        
        let challenge = AuthChallenge {
            id: Uuid::new_v4().to_string(),
            device_id: device_id.to_string(),
            challenge: code.clone(),
            challenge_type: ChallengeType::PairingCode,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::minutes(5),
            attempts_remaining: 3,
        };
        
        let mut challenges = self.challenges.write().await;
        challenges.insert(challenge.id.clone(), challenge);
        
        Ok(code)
    }
    
    /// Validate pairing code
    pub async fn validate_pairing_code(&self, code: &str) -> Result<Option<String>> {
        let mut challenges = self.challenges.write().await;
        
        for (_, challenge) in challenges.iter_mut() {
            if challenge.challenge == code && 
               challenge.challenge_type == ChallengeType::PairingCode &&
               challenge.expires_at > Utc::now() {
                
                let device_id = challenge.device_id.clone();
                challenges.retain(|_, c| c.challenge != code);
                
                return Ok(Some(device_id));
            }
        }
        
        Ok(None)
    }
    
    /// Set biometric verified
    pub async fn set_biometric_verified(
        &self,
        session_id: &str,
        verified: bool,
    ) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            session.biometric_verified = verified;
        }
        
        Ok(())
    }
    
    /// Require biometric check
    pub async fn requires_biometric(&self, permission: &Permission) -> bool {
        matches!(
            permission,
            Permission::ViewPasswords | Permission::FullAccess
        )
    }
    
    /// Generate secure token
    fn generate_token() -> String {
        format!(
            "vweb_{}_{}",
            Uuid::new_v4(),
            Utc::now().timestamp()
        )
    }
    
    /// Generate pairing code (6 digits)
    fn generate_pairing_code() -> String {
        // Simple 6-digit code
        use std::time::{SystemTime, UNIX_EPOCH};
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("{:06}", ts % 1_000_000)
    }
    
    /// Generate challenge code
    fn generate_challenge_code() -> String {
        Self::generate_pairing_code()
    }
    
    /// Clean expired sessions
    pub async fn clean_expired_sessions(&self) -> Result<u32> {
        let mut sessions = self.sessions.write().await;
        let initial_count = sessions.len();
        
        sessions.retain(|_, session| {
            session.is_active && session.expires_at > Utc::now()
        });
        
        Ok((initial_count - sessions.len()) as u32)
    }
    
    /// Clean expired tokens
    pub async fn clean_expired_tokens(&self) -> Result<u32> {
        let mut tokens = self.tokens.write().await;
        let initial_count = tokens.len();
        
        tokens.retain(|_, token| {
            token.is_valid && token.expires_at > Utc::now()
        });
        
        Ok((initial_count - tokens.len()) as u32)
    }
    
    /// Clean expired challenges
    pub async fn clean_expired_challenges(&self) -> Result<u32> {
        let mut challenges = self.challenges.write().await;
        let initial_count = challenges.len();
        
        challenges.retain(|_, challenge| {
            challenge.expires_at > Utc::now()
        });
        
        Ok((initial_count - challenges.len()) as u32)
    }
    
    /// Get statistics
    pub async fn get_statistics(&self) -> Result<AuthStatistics> {
        let sessions = self.sessions.read().await;
        let tokens = self.tokens.read().await;
        let challenges = self.challenges.read().await;
        
        let active_sessions = sessions.values().filter(|s| s.is_active).count();
        let valid_tokens = tokens.values().filter(|t| t.is_valid).count();
        let pending_challenges = challenges.len();
        
        Ok(AuthStatistics {
            active_sessions: active_sessions as u64,
            valid_tokens: valid_tokens as u64,
            pending_challenges: pending_challenges as u64,
        })
    }
}

/// Authentication statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStatistics {
    pub active_sessions: u64,
    pub valid_tokens: u64,
    pub pending_challenges: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_device_token() {
        let config = MobileConfig::default();
        let manager = MobileAuthManager::new(config).await.unwrap();
        
        let token = manager.create_device_token("device-1").await.unwrap();
        assert!(token.starts_with("vweb_"));
        
        let valid = manager.validate_device_token(&token).await.unwrap();
        assert_eq!(valid, Some("device-1".to_string()));
    }
    
    #[tokio::test]
    async fn test_create_session() {
        let config = MobileConfig::default();
        let manager = MobileAuthManager::new(config).await.unwrap();
        
        let session = manager.create_session(
            "device-1",
            vec![Permission::ViewTabs, Permission::ManageTabs],
        ).await.unwrap();
        
        assert!(session.is_active);
        assert_eq!(session.device_id, "device-1");
    }
    
    #[tokio::test]
    async fn test_permission_check() {
        let config = MobileConfig::default();
        let manager = MobileAuthManager::new(config).await.unwrap();
        
        let session = manager.create_session(
            "device-1",
            vec![Permission::ViewTabs],
        ).await.unwrap();
        
        let has_view = manager.check_permission(&session.id, &Permission::ViewTabs).await.unwrap();
        assert!(has_view);
        
        let has_passwords = manager.check_permission(&session.id, &Permission::ViewPasswords).await.unwrap();
        assert!(!has_passwords);
    }
    
    #[tokio::test]
    async fn test_pairing_code() {
        let config = MobileConfig::default();
        let manager = MobileAuthManager::new(config).await.unwrap();
        
        let code = manager.create_pairing_code("device-1").await.unwrap();
        assert_eq!(code.len(), 6);
        
        let device = manager.validate_pairing_code(&code).await.unwrap();
        assert_eq!(device, Some("device-1".to_string()));
    }
}