//! Profile Security (Optimized)
//!
//! Security features for profiles including password protection, biometric authentication, and encryption.
//!
//! Optimizations:
//! - Pre-allocated collection capacities
//! - Reduced clones
//! - Optimized string operations

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Security level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SecurityLevel {
    /// No security
    None,
    /// Low security (password only)
    Low,
    /// Medium security (password + encryption)
    Medium,
    /// High security (password + encryption + biometric)
    High,
}

/// Authentication method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    /// Password authentication
    Password,
    /// Biometric authentication
    Biometric,
    /// Two-factor authentication
    TwoFactor,
}

/// Profile security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSecurity {
    /// Profile ID
    pub profile_id: String,
    /// Security level
    pub security_level: SecurityLevel,
    /// Authentication methods
    pub auth_methods: Vec<AuthMethod>,
    /// Password hash (if password auth is enabled)
    pub password_hash: Option<String>,
    /// Biometric data (if biometric auth is enabled)
    pub biometric_data: Option<String>,
    /// Encryption key (if encryption is enabled)
    pub encryption_key: Option<String>,
    /// Auto-lock timeout (in seconds)
    pub auto_lock_timeout: Option<u64>,
    /// Last authentication timestamp
    pub last_auth: Option<chrono::DateTime<chrono::Utc>>,
    /// Failed authentication attempts
    pub failed_attempts: u32,
    /// Maximum failed attempts before lockout
    pub max_failed_attempts: u32,
    /// Lockout duration (in seconds)
    pub lockout_duration: u64,
    /// Is locked
    pub is_locked: bool,
    /// Lockout expiry
    pub lockout_expiry: Option<chrono::DateTime<chrono::Utc>>,
}

/// Profile security manager (Optimized)
pub struct ProfileSecurityManager {
    /// Security configurations per profile
    securities: HashMap<String, ProfileSecurity>,
    /// Crypto engine
    crypto: crate::security::CryptoEngine,
}

impl ProfileSecurityManager {
    /// Creates a new profile security manager (Optimized)
    pub fn new() -> Result<Self> {
        Ok(Self {
            securities: HashMap::with_capacity(10),
            crypto: crate::security::CryptoEngine::new()?,
        })
    }

    /// Creates security configuration for a profile (Optimized)
    pub fn create_security(&mut self, profile_id: &str, security_level: SecurityLevel) -> Result<()> {
        let security = ProfileSecurity {
            profile_id: profile_id.to_string(),
            security_level: security_level.clone(),
            auth_methods: match security_level {
                SecurityLevel::None => Vec::with_capacity(0),
                SecurityLevel::Low => vec![AuthMethod::Password],
                SecurityLevel::Medium => vec![AuthMethod::Password],
                SecurityLevel::High => vec![AuthMethod::Password, AuthMethod::Biometric],
            },
            password_hash: None,
            biometric_data: None,
            encryption_key: None,
            auto_lock_timeout: Some(300), // 5 minutes
            last_auth: None,
            failed_attempts: 0,
            max_failed_attempts: 5,
            lockout_duration: 900, // 15 minutes
            is_locked: false,
            lockout_expiry: None,
        };

        self.securities.insert(profile_id.to_string(), security);
        Ok(())
    }

    /// Sets password for a profile
    pub fn set_password(&mut self, profile_id: &str, password: &str) -> Result<()> {
        let security = self.securities.get_mut(profile_id)
            .ok_or_else(|| anyhow!("Security not found for profile: {}", profile_id))?;

        // Hash password using BLAKE3
        let hash = self.crypto.hash(password.as_bytes());
        let hash_hex = hex::encode(hash);

        security.password_hash = Some(hash_hex);
        security.auth_methods.push(AuthMethod::Password);

        Ok(())
    }

    /// Verifies password for a profile
    pub fn verify_password(&self, profile_id: &str, password: &str) -> Result<bool> {
        let security = self.securities.get(profile_id)
            .ok_or_else(|| anyhow!("Security not found for profile: {}", profile_id))?;

        if let Some(stored_hash) = &security.password_hash {
            let hash = self.crypto.hash(password.as_bytes());
            let hash_hex = hex::encode(hash);

            Ok(hash_hex == *stored_hash)
        } else {
            Ok(false)
        }
    }

    /// Sets biometric data for a profile
    pub fn set_biometric(&mut self, profile_id: &str, biometric_data: &str) -> Result<()> {
        let security = self.securities.get_mut(profile_id)
            .ok_or_else(|| anyhow!("Security not found for profile: {}", profile_id))?;

        // Encrypt biometric data
        let encrypted = self.crypto.encrypt(biometric_data.as_bytes())?;
        let encrypted_hex = hex::encode(encrypted);

        security.biometric_data = Some(encrypted_hex);
        security.auth_methods.push(AuthMethod::Biometric);

        Ok(())
    }

    /// Verifies biometric data for a profile
    pub fn verify_biometric(&self, profile_id: &str, biometric_data: &str) -> Result<bool> {
        let security = self.securities.get(profile_id)
            .ok_or_else(|| anyhow!("Security not found for profile: {}", profile_id))?;

        if let Some(stored_data) = &security.biometric_data {
            // Decrypt stored data
            let encrypted = hex::decode(stored_data)?;
            let decrypted = self.crypto.decrypt(&encrypted)?;
            let stored_str = String::from_utf8(decrypted)?;

            Ok(stored_str == biometric_data)
        } else {
            Ok(false)
        }
    }

    /// Authenticates a profile
    pub fn authenticate(&mut self, profile_id: &str, auth_method: &AuthMethod, credential: &str) -> Result<bool> {
        let security = self.securities.get_mut(profile_id)
            .ok_or_else(|| anyhow!("Security not found for profile: {}", profile_id))?;

        // Check if locked
        if security.is_locked {
            if let Some(expiry) = security.lockout_expiry {
                if chrono::Utc::now() < expiry {
                    return Err(anyhow!("Profile is locked due to too many failed attempts"));
                } else {
                    // Lockout expired
                    security.is_locked = false;
                    security.failed_attempts = 0;
                }
            }
        }

        let verified = match auth_method {
            AuthMethod::Password => self.verify_password(profile_id, credential)?,
            AuthMethod::Biometric => self.verify_biometric(profile_id, credential)?,
            AuthMethod::TwoFactor => {
                // For 2FA, verify password first
                if self.verify_password(profile_id, credential)? {
                    // TODO: Implement second factor verification
                    true
                } else {
                    false
                }
            }
        };

        if verified {
            security.last_auth = Some(chrono::Utc::now());
            security.failed_attempts = 0;
            Ok(true)
        } else {
            security.failed_attempts += 1;
            // Check if should lock
            if security.failed_attempts >= security.max_failed_attempts {
                security.is_locked = true;
                security.lockout_expiry = Some(chrono::Utc::now() + chrono::Duration::seconds(security.lockout_duration as i64));
            }

            Ok(false)
        }
    }

    /// Encrypts profile data
    pub fn encrypt_profile_data(&self, profile_id: &str, data: &[u8]) -> Result<Vec<u8>> {
        let security = self.securities.get(profile_id)
            .ok_or_else(|| anyhow!("Security not found for profile: {}", profile_id))?;

        if security.security_level == SecurityLevel::None {
            return Ok(data.to_vec());
        }

        // Use encryption key if available, otherwise use password hash
        let key = if let Some(encryption_key) = &security.encryption_key {
            hex::decode(encryption_key)?
        } else if let Some(password_hash) = &security.password_hash {
            password_hash.as_bytes().to_vec()
        } else {
            return Err(anyhow!("No encryption key available"));
        };

        // TODO: Implement proper encryption with key
        // For now, use the crypto engine
        self.crypto.encrypt(data)
    }

    /// Decrypts profile data
    pub fn decrypt_profile_data(&self, profile_id: &str, encrypted_data: &[u8]) -> Result<Vec<u8>> {
        let security = self.securities.get(profile_id)
            .ok_or_else(|| anyhow!("Security not found for profile: {}", profile_id))?;

        if security.security_level == SecurityLevel::None {
            return Ok(encrypted_data.to_vec());
        }

        // TODO: Implement proper decryption with key
        // For now, use the crypto engine
        self.crypto.decrypt(encrypted_data)
    }

    /// Checks if a profile is locked
    pub fn is_locked(&self, profile_id: &str) -> Result<bool> {
        let security = self.securities.get(profile_id)
            .ok_or_else(|| anyhow!("Security not found for profile: {}", profile_id))?;

        // Check if lockout has expired
        if security.is_locked {
            if let Some(expiry) = security.lockout_expiry {
                if chrono::Utc::now() >= expiry {
                    return Ok(false);
                }
            }
        }

        Ok(security.is_locked)
    }

    /// Unlocks a profile
    pub fn unlock(&mut self, profile_id: &str) -> Result<()> {
        let security = self.securities.get_mut(profile_id)
            .ok_or_else(|| anyhow!("Security not found for profile: {}", profile_id))?;

        security.is_locked = false;
        security.failed_attempts = 0;
        security.lockout_expiry = None;
        Ok(())
    }

    /// Sets auto-lock timeout
    pub fn set_auto_lock_timeout(&mut self, profile_id: &str, timeout_seconds: u64) -> Result<()> {
        let security = self.securities.get_mut(profile_id)
            .ok_or_else(|| anyhow!("Security not found for profile: {}", profile_id))?;

        security.auto_lock_timeout = Some(timeout_seconds);
        Ok(())
    }

    /// Checks if auto-lock should trigger
    pub fn should_auto_lock(&self, profile_id: &str) -> Result<bool> {
        let security = self.securities.get(profile_id)
            .ok_or_else(|| anyhow!("Security not found for profile: {}", profile_id))?;

        if let Some(timeout) = security.auto_lock_timeout {
            if let Some(last_auth) = security.last_auth {
                let elapsed = (chrono::Utc::now() - last_auth).num_seconds() as u64;
                return Ok(elapsed >= timeout);
            }
        }

        Ok(false)
    }

    /// Gets security configuration for a profile (Optimized - returns reference)
    pub fn get_security(&self, profile_id: &str) -> Option<&ProfileSecurity> {
        self.securities.get(profile_id)
    }

    /// Removes security for a profile
    pub fn remove_security(&mut self, profile_id: &str) -> Result<()> {
        self.securities.remove(profile_id)
            .ok_or_else(|| anyhow!("Security not found for profile: {}", profile_id))?;

        Ok(())
    }

    /// Exports security configuration
    pub fn export(&self, profile_id: &str) -> Result<String> {
        let security = self.get_security(profile_id)
            .ok_or_else(|| anyhow!("Security not found for profile: {}", profile_id))?;

        // Don't export sensitive data
        let export = serde_json::json!({
            "profile_id": security.profile_id,
            "security_level": security.security_level,
            "auth_methods": security.auth_methods,
            "auto_lock_timeout": security.auto_lock_timeout,
            "max_failed_attempts": security.max_failed_attempts,
            "lockout_duration": security.lockout_duration,
        });

        serde_json::to_string_pretty(&export)
            .map_err(|e| anyhow!("Failed to export security: {}", e))
    }
}

impl Default for ProfileSecurityManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| panic!("Failed to create security manager"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_manager_creation() {
        let manager = ProfileSecurityManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_create_security() {
        let mut manager = ProfileSecurityManager::new().unwrap();
        manager.create_security("test-profile", SecurityLevel::Medium).unwrap();

        let security = manager.get_security("test-profile");
        assert!(security.is_some());
        assert_eq!(security.unwrap().security_level, SecurityLevel::Medium);
    }

    #[test]
    fn test_set_verify_password() {
        let mut manager = ProfileSecurityManager::new().unwrap();
        manager.create_security("test-profile", SecurityLevel::Low).unwrap();

        manager.set_password("test-profile", "password123").unwrap();
        let verified = manager.verify_password("test-profile", "password123").unwrap();

        assert!(verified);
    }

    #[test]
    fn test_authenticate() {
        let mut manager = ProfileSecurityManager::new().unwrap();
        manager.create_security("test-profile", SecurityLevel::Low).unwrap();

        manager.set_password("test-profile", "password123").unwrap();

        let result = manager.authenticate("test-profile", &AuthMethod::Password, "password123").unwrap();
        assert!(result);

        let result = manager.authenticate("test-profile", &AuthMethod::Password, "wrongpassword").unwrap();
        assert!(!result);
    }

    #[test]
    fn test_failed_attempts_lockout() {
        let mut manager = ProfileSecurityManager::new().unwrap();
        manager.create_security("test-profile", SecurityLevel::Low).unwrap();

        manager.set_password("test-profile", "password123").unwrap();
        // Fail authentication multiple times
        for _ in 0..5 {
            manager.authenticate("test-profile", &AuthMethod::Password, "wrongpassword").unwrap();
        }

        let is_locked = manager.is_locked("test-profile").unwrap();
        assert!(is_locked);
    }
}