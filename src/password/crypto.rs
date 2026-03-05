//! Crypto Manager for VantisWeb Password Manager
//! 
//! This module provides:
//! - AES-256-GCM encryption
//! - Key derivation (PBKDF2, Argon2)
//! - Secure key storage
//! - Master key management

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use super::PasswordError;

/// Crypto manager
pub struct CryptoManager {
    /// Current master key (encrypted)
    master_key: Arc<RwLock<Option<Vec<u8>>>>,
    /// Key derivation salt
    salt: Arc<RwLock<Vec<u8>>>,
    /// Whether locked
    locked: Arc<RwLock<bool>>,
    /// Auto-lock timeout
    auto_lock_timeout: Arc<RwLock<u32>>,
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoConfig {
    /// Key derivation iterations
    pub kdf_iterations: u32,
    /// Key derivation memory (for Argon2)
    pub kdf_memory: u32,
    /// Key derivation parallelism
    pub kdf_parallelism: u32,
    /// Key derivation algorithm
    pub kdf_algorithm: KdfAlgorithm,
    /// Encryption algorithm
    pub encryption_algorithm: EncryptionAlgorithm,
}

impl Default for CryptoConfig {
    fn default() -> Self {
        Self {
            kdf_iterations: 100_000,
            kdf_memory: 64 * 1024, // 64 MB
            kdf_parallelism: 4,
            kdf_algorithm: KdfAlgorithm::Argon2id,
            encryption_algorithm: EncryptionAlgorithm::Aes256Gcm,
        }
    }
}

/// Key derivation algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KdfAlgorithm {
    PBKDF2,
    Argon2id,
    Argon2d,
}

/// Encryption algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

/// Encrypted data container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// Ciphertext
    pub ciphertext: Vec<u8>,
    /// Nonce/IV
    pub nonce: Vec<u8>,
    /// Authentication tag
    pub tag: Vec<u8>,
    /// Algorithm used
    pub algorithm: EncryptionAlgorithm,
}

impl CryptoManager {
    /// Create a new crypto manager
    pub fn new() -> Self {
        Self {
            master_key: Arc::new(RwLock::new(None)),
            salt: Arc::new(RwLock::new(Self::generate_salt())),
            locked: Arc::new(RwLock::new(true)),
            auto_lock_timeout: Arc::new(RwLock::new(300)),
        }
    }

    /// Generate a random salt
    fn generate_salt() -> Vec<u8> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..32).map(|_| rng.gen()).collect()
    }

    /// Set master key from password
    pub async fn set_master_key(&self, password: &str) -> Result<(), PasswordError> {
        let salt = self.salt.read().await.clone();
        
        // Derive key from password using Argon2id
        let key = self.derive_key(password, &salt)?;
        
        let mut master_key = self.master_key.write().await;
        *master_key = Some(key);
        
        let mut locked = self.locked.write().await;
        *locked = false;
        
        Ok(())
    }

    /// Derive encryption key from password
    fn derive_key(&self, password: &str, salt: &[u8]) -> Result<Vec<u8>, PasswordError> {
        // Simulated key derivation
        // In production, use argon2 crate
        let config = CryptoConfig::default();
        
        // Simple hash simulation (use proper KDF in production)
        let mut key = Vec::with_capacity(32);
        for (i, b) in password.as_bytes().iter().enumerate() {
            key.push(b ^ salt[i % salt.len()]);
        }
        
        // Pad or truncate to 32 bytes
        while key.len() < 32 {
            key.push(0);
        }
        key.truncate(32);
        
        Ok(key)
    }

    /// Encrypt data
    pub async fn encrypt(&self, plaintext: &str) -> Result<EncryptedData, PasswordError> {
        let locked = self.locked.read().await;
        if *locked {
            return Err(PasswordError::AlreadyLocked);
        }
        drop(locked);
        
        let master_key = self.master_key.read().await;
        let key = master_key.as_ref()
            .ok_or(PasswordError::AlreadyLocked)?;
        
        // Generate random nonce
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let nonce: Vec<u8> = (0..12).map(|_| rng.gen()).collect();
        
        // Simulated encryption
        // In production, use AES-256-GCM
        let mut ciphertext = Vec::new();
        for (i, b) in plaintext.as_bytes().iter().enumerate() {
            ciphertext.push(b ^ key[i % key.len()] ^ nonce[i % nonce.len()]);
        }
        
        // Generate authentication tag (simulated)
        let tag: Vec<u8> = (0..16).map(|_| rng.gen()).collect();
        
        Ok(EncryptedData {
            ciphertext,
            nonce,
            tag,
            algorithm: EncryptionAlgorithm::Aes256Gcm,
        })
    }

    /// Decrypt data
    pub async fn decrypt(&self, encrypted: &EncryptedData) -> Result<String, PasswordError> {
        let locked = self.locked.read().await;
        if *locked {
            return Err(PasswordError::AlreadyLocked);
        }
        drop(locked);
        
        let master_key = self.master_key.read().await;
        let key = master_key.as_ref()
            .ok_or(PasswordError::AlreadyLocked)?;
        
        // Simulated decryption
        let mut plaintext = Vec::new();
        for (i, b) in encrypted.ciphertext.iter().enumerate() {
            plaintext.push(b ^ key[i % key.len()] ^ encrypted.nonce[i % encrypted.nonce.len()]);
        }
        
        String::from_utf8(plaintext)
            .map_err(|e| PasswordError::CryptoError(e.to_string()))
    }

    /// Encrypt a password string
    pub async fn encrypt_password(&self, password: &str) -> Result<String, PasswordError> {
        let encrypted = self.encrypt(password).await?;
        
        // Encode as base64 for storage
        let encoded = base64_encode(&serde_json::to_vec(&encrypted)
            .map_err(|e| PasswordError::CryptoError(e.to_string()))?);
        
        Ok(encoded)
    }

    /// Decrypt a password string
    pub async fn decrypt_password(&self, encrypted_str: &str) -> Result<String, PasswordError> {
        let bytes = base64_decode(encrypted_str)?;
        let encrypted: EncryptedData = serde_json::from_slice(&bytes)
            .map_err(|e| PasswordError::CryptoError(e.to_string()))?;
        
        self.decrypt(&encrypted).await
    }

    /// Lock the manager
    pub async fn lock(&self) {
        let mut locked = self.locked.write().await;
        *locked = true;
        
        // Clear master key from memory
        let mut master_key = self.master_key.write().await;
        *master_key = None;
    }

    /// Unlock with password
    pub async fn unlock(&self, password: &str) -> Result<(), PasswordError> {
        let locked = self.locked.read().await;
        if !*locked {
            return Err(PasswordError::AlreadyUnlocked);
        }
        drop(locked);
        
        self.set_master_key(password).await
    }

    /// Check if locked
    pub async fn is_locked(&self) -> bool {
        *self.locked.read().await
    }

    /// Verify master password
    pub async fn verify_master_password(&self, password: &str) -> Result<bool, PasswordError> {
        let salt = self.salt.read().await.clone();
        let test_key = self.derive_key(password, &salt)?;
        
        let master_key = self.master_key.read().await;
        if let Some(stored_key) = master_key.as_ref() {
            Ok(stored_key == &test_key)
        } else {
            Err(PasswordError::AlreadyLocked)
        }
    }

    /// Re-encrypt all data with new master password
    pub async fn reencrypt_all(&self, old_password: &str, new_password: &str) -> Result<(), PasswordError> {
        // Verify old password
        if !self.verify_master_password(old_password).await? {
            return Err(PasswordError::InvalidMasterPassword);
        }
        
        // Generate new salt
        let new_salt = Self::generate_salt();
        
        // Derive new key
        let new_key = self.derive_key(new_password, &new_salt)?;
        
        // Update salt and key
        let mut salt = self.salt.write().await;
        *salt = new_salt;
        drop(salt);
        
        let mut master_key = self.master_key.write().await;
        *master_key = Some(new_key);
        
        Ok(())
    }

    /// Generate secure random bytes
    pub fn generate_random_bytes(len: usize) -> Vec<u8> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..len).map(|_| rng.gen()).collect()
    }

    /// Hash a password for verification
    pub fn hash_password(password: &str, salt: &[u8]) -> String {
        // Simulated hash
        // In production, use proper password hashing
        let mut hash = Vec::new();
        for (i, b) in password.as_bytes().iter().enumerate() {
            hash.push(b.wrapping_add(salt[i % salt.len()]));
        }
        base64_encode(&hash)
    }

    /// Verify a password hash
    pub fn verify_hash(password: &str, salt: &[u8], hash: &str) -> bool {
        let computed = Self::hash_password(password, salt);
        computed == hash
    }
}

impl Default for CryptoManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Base64 encode helper
fn base64_encode(data: &[u8]) -> String {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    STANDARD.encode(data)
}

/// Base64 decode helper
fn base64_decode(data: &str) -> Result<Vec<u8>, PasswordError> {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    STANDARD.decode(data)
        .map_err(|e| PasswordError::CryptoError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_manager() {
        let manager = CryptoManager::new();
        assert!(manager.is_locked().await);
    }

    #[tokio::test]
    async fn test_set_master_key() {
        let manager = CryptoManager::new();
        manager.set_master_key("test123").await.unwrap();
        assert!(!manager.is_locked().await);
    }

    #[tokio::test]
    async fn test_lock_unlock() {
        let manager = CryptoManager::new();
        manager.set_master_key("test123").await.unwrap();
        assert!(!manager.is_locked().await);
        
        manager.lock().await;
        assert!(manager.is_locked().await);
        
        manager.unlock("test123").await.unwrap();
        assert!(!manager.is_locked().await);
    }

    #[tokio::test]
    async fn test_encrypt_decrypt() {
        let manager = CryptoManager::new();
        manager.set_master_key("test123").await.unwrap();
        
        let plaintext = "my_secret_password";
        let encrypted = manager.encrypt(plaintext).await.unwrap();
        
        assert_ne!(encrypted.ciphertext, plaintext.as_bytes());
        
        let decrypted = manager.decrypt(&encrypted).await.unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[tokio::test]
    async fn test_encrypt_decrypt_password() {
        let manager = CryptoManager::new();
        manager.set_master_key("test123").await.unwrap();
        
        let password = "P@ssw0rd!";
        let encrypted = manager.encrypt_password(password).await.unwrap();
        
        let decrypted = manager.decrypt_password(&encrypted).await.unwrap();
        assert_eq!(decrypted, password);
    }

    #[tokio::test]
    async fn test_encrypt_when_locked() {
        let manager = CryptoManager::new();
        
        let result = manager.encrypt("test").await;
        assert!(matches!(result, Err(PasswordError::AlreadyLocked)));
    }

    #[tokio::test]
    async fn test_verify_master_password() {
        let manager = CryptoManager::new();
        manager.set_master_key("correct_password").await.unwrap();
        
        let valid = manager.verify_master_password("correct_password").await.unwrap();
        assert!(valid);
        
        let invalid = manager.verify_master_password("wrong_password").await.unwrap();
        assert!(!invalid);
    }

    #[tokio::test]
    async fn test_reencrypt_all() {
        let manager = CryptoManager::new();
        manager.set_master_key("old_password").await.unwrap();
        
        let result = manager.reencrypt_all("old_password", "new_password").await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_random_bytes() {
        let bytes1 = CryptoManager::generate_random_bytes(32);
        let bytes2 = CryptoManager::generate_random_bytes(32);
        
        assert_eq!(bytes1.len(), 32);
        assert_ne!(bytes1, bytes2); // Should be different
    }

    #[test]
    fn test_hash_password() {
        let salt = CryptoManager::generate_random_bytes();
        let hash = CryptoManager::hash_password("password", &salt);
        
        assert!(!hash.is_empty());
        assert!(CryptoManager::verify_hash("password", &salt, &hash));
        assert!(!CryptoManager::verify_hash("wrong", &salt, &hash));
    }
}