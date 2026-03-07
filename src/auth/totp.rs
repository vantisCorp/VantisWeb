// VantisWeb Browser - TOTP Authentication
// Copyright (c) 2024 VantisCorp
// Time-based One-Time Password (TOTP) implementation

use base32::Alphabet;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// TOTP configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TOTPConfig {
    /// Time step in seconds (default: 30)
    pub time_step: u64,
    
    /// Number of digits in code (default: 6)
    pub digits: u8,
    
    /// HMAC algorithm (default: SHA1)
    pub algorithm: HMACAlgorithm,
    
    /// Clock skew tolerance (default: 1 step)
    pub skew: u64,
}

impl Default for TOTPConfig {
    fn default() -> Self {
        Self {
            time_step: 30,
            digits: 6,
            algorithm: HMACAlgorithm::SHA1,
            skew: 1,
        }
    }
}

/// HMAC algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HMACAlgorithm {
    SHA1,
    SHA256,
    SHA512,
}

/// TOTP secret
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TOTPSecret {
    /// Base32 encoded secret
    pub secret: String,
    
    /// Algorithm used
    pub algorithm: HMACAlgorithm,
    
    /// Number of digits
    pub digits: u8,
    
    /// Time step
    pub time_step: u64,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

impl TOTPSecret {
    /// Generate a new random secret
    pub fn generate() -> Self {
        Self::generate_with_config(TOTPConfig::default())
    }
    
    /// Generate a new secret with custom configuration
    pub fn generate_with_config(config: TOTPConfig) -> Self {
        use rand::Rng;
        const SECRET_SIZE: usize = 32;
        
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; SECRET_SIZE];
        rng.fill(&mut bytes);
        
        let secret = base32::encode(Alphabet::RFC4648 { padding: true }, &bytes);
        
        Self {
            secret,
            algorithm: config.algorithm,
            digits: config.digits,
            time_step: config.time_step,
            created_at: Utc::now(),
        }
    }
    
    /// Get the current TOTP code
    pub fn current_code(&self) -> Result<String, TOTPError> {
        self.generate_code(0)
    }
    
    /// Get TOTP code for a specific time offset
    pub fn generate_code(&self, time_offset: i64) -> Result<String, TOTPError> {
        let counter = self.get_counter(time_offset)?;
        self.generate_code_for_counter(counter)
    }
    
    /// Generate code for a specific counter
    fn generate_code_for_counter(&self, counter: u64) -> Result<String, TOTPError> {
        // Convert counter to 8-byte big-endian
        let counter_bytes = counter.to_be_bytes();
        
        // HMAC hash
        let hmac_result = match self.algorithm {
            HMACAlgorithm::SHA1 => {
                use hmac::{Hmac, Mac};
                type HmacSha1 = Hmac<sha1::Sha1>;
                
                let secret_bytes = base32::decode(Alphabet::RFC4648 { padding: true }, &self.secret)
                    .ok_or_else(|| TOTPError::InvalidSecret)?;
                
                let mut mac = HmacSha1::new_from_slice(&secret_bytes)
                    .map_err(|_| TOTPError::InvalidSecret)?;
                mac.update(&counter_bytes);
                mac.finalize().into_bytes().to_vec()
            }
            HMACAlgorithm::SHA256 => {
                use hmac::{Hmac, Mac};
                type HmacSha256 = Hmac<sha2::Sha256>;
                
                let secret_bytes = base32::decode(Alphabet::RFC4648 { padding: true }, &self.secret)
                    .ok_or_else(|| TOTPError::InvalidSecret)?;
                
                let mut mac = HmacSha256::new_from_slice(&secret_bytes)
                    .map_err(|_| TOTPError::InvalidSecret)?;
                mac.update(&counter_bytes);
                mac.finalize().into_bytes().to_vec()
            }
            HMACAlgorithm::SHA512 => {
                use hmac::{Hmac, Mac};
                type HmacSha512 = Hmac<sha2::Sha512>;
                
                let secret_bytes = base32::decode(Alphabet::RFC4648 { padding: true }, &self.secret)
                    .ok_or_else(|| TOTPError::InvalidSecret)?;
                
                let mut mac = HmacSha512::new_from_slice(&secret_bytes)
                    .map_err(|_| TOTPError::InvalidSecret)?;
                mac.update(&counter_bytes);
                mac.finalize().into_bytes().to_vec()
            }
        };
        
        // Dynamic truncation
        let offset = (hmac_result[hmac_result.len() - 1] & 0x0f) as usize;
        let binary = ((hmac_result[offset] & 0x7f) as u32) << 24
            | ((hmac_result[offset + 1] & 0xff) as u32) << 16
            | ((hmac_result[offset + 2] & 0xff) as u32) << 8
            | ((hmac_result[offset + 3] & 0xff) as u32);
        
        let code = binary % 10u32.pow(self.digits as u32);
        Ok(format!("{:0width$}", code, width = self.digits as usize))
    }
    
    /// Get the current counter value
    fn get_counter(&self, time_offset: i64) -> Result<u64, TOTPError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| TOTPError::InvalidTime)?
            .as_secs() as i64;
        
        Ok(((timestamp + time_offset) / self.time_step as i64) as u64)
    }
    
    /// Verify a TOTP code
    pub fn verify(&self, code: &str) -> Result<bool, TOTPError> {
        self.verify_with_skew(code, 1)
    }
    
    /// Verify a TOTP code with custom skew
    pub fn verify_with_skew(&self, code: &str, skew: u64) -> Result<bool, TOTPError> {
        for offset in -(skew as i64)..=(skew as i64) {
            let expected_code = self.generate_code(offset)?;
            if expected_code == code {
                return Ok(true);
            }
        }
        Ok(false)
    }
    
    /// Get the remaining time until next code
    pub fn remaining_time(&self) -> u64 {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        
        self.time_step - (timestamp % self.time_step)
    }
    
    /// Get the time elapsed in current code
    pub fn elapsed_time(&self) -> u64 {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        
        timestamp % self.time_step
    }
    
    /// Generate QR code URI for authenticator apps
    pub fn to_qr_uri(&self, account_name: &str, issuer: &str) -> String {
        format!(
            "otpauth://totp/{}:{}?secret={}&algorithm={}&digits={}&period={}&issuer={}",
            urlencoding::encode(issuer),
            urlencoding::encode(account_name),
            self.secret,
            self.algorithm.to_string().to_lowercase(),
            self.digits,
            self.time_step,
            urlencoding::encode(issuer)
        )
    }
}

impl HMACAlgorithm {
    fn to_string(&self) -> &'static str {
        match self {
            Self::SHA1 => "SHA1",
            Self::SHA256 => "SHA256",
            Self::SHA512 => "SHA512",
        }
    }
}

/// TOTP Manager
pub struct TOTPManager {
    config: TOTPConfig,
}

impl TOTPManager {
    /// Create a new TOTP manager
    pub fn new() -> Self {
        Self {
            config: TOTPConfig::default(),
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(config: TOTPConfig) -> Self {
        Self { config }
    }
    
    /// Generate a new TOTP secret
    pub fn generate_secret(&self) -> TOTPSecret {
        TOTPSecret::generate_with_config(self.config)
    }
    
    /// Verify a TOTP code against a secret
    pub fn verify(&self, secret: &TOTPSecret, code: &str) -> Result<bool, TOTPError> {
        secret.verify_with_skew(code, self.config.skew)
    }
    
    /// Get current code for a secret
    pub fn current_code(&self, secret: &TOTPSecret) -> Result<String, TOTPError> {
        secret.current_code()
    }
    
    /// Generate backup codes
    pub fn generate_backup_codes(count: usize) -> Vec<String> {
        use rand::Rng;
        
        (0..count)
            .map(|_| {
                (0..8)
                    .map(|_| {
                        let chars = "0123456789ABCDEF";
                        let idx = rand::thread_rng().gen_range(0..chars.len());
                        chars.chars().nth(idx).unwrap()
                    })
                    .collect::<String>()
            })
            .collect()
    }
}

impl Default for TOTPManager {
    fn default() -> Self {
        Self::new()
    }
}

/// TOTP error types
#[derive(Debug, thiserror::Error)]
pub enum TOTPError {
    #[error("Invalid secret")]
    InvalidSecret,
    
    #[error("Invalid time")]
    InvalidTime,
    
    #[error("Invalid code format")]
    InvalidCodeFormat,
    
    #[error("HMAC error: {0}")]
    HMAC(String),
    
    #[error("Code expired")]
    CodeExpired,
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_secret_generation() {
        let secret = TOTPSecret::generate();
        assert!(!secret.secret.is_empty());
        assert_eq!(secret.digits, 6);
        assert_eq!(secret.time_step, 30);
    }
    
    #[test]
    fn test_code_generation() {
        let secret = TOTPSecret::generate();
        let code = secret.current_code().unwrap();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }
    
    #[test]
    fn test_code_verification() {
        let secret = TOTPSecret::generate();
        let code = secret.current_code().unwrap();
        let verified = secret.verify(&code).unwrap();
        assert!(verified);
    }
    
    #[test]
    fn test_remaining_time() {
        let secret = TOTPSecret::generate();
        let remaining = secret.remaining_time();
        assert!(remaining <= 30);
    }
    
    #[test]
    fn test_qr_uri() {
        let secret = TOTPSecret::generate();
        let uri = secret.to_qr_uri("user@example.com", "VantisWeb");
        assert!(uri.starts_with("otpauth://totp/"));
        assert!(uri.contains("VantisWeb"));
        assert!(uri.contains("user@example.com"));
    }
    
    #[test]
    fn test_backup_codes() {
        let codes = TOTPManager::generate_backup_codes(10);
        assert_eq!(codes.len(), 10);
        assert!(codes.iter().all(|c| c.len() == 8));
    }
}