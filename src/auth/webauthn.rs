// VantisWeb Browser - WebAuthn Authentication
// Copyright (c) 2024 VantisCorp
// Hardware security key support via WebAuthn/FIDO2

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// WebAuthn configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAuthnConfig {
    /// Relying party ID (origin)
    pub rp_id: String,
    
    /// Relying party name
    pub rp_name: String,
    
    /// Origin
    pub origin: String,
    
    /// Require user verification
    pub user_verification: UserVerificationRequirement,
    
    /// Attestation conveyance preference
    pub attestation: AttestationConveyancePreference,
}

impl WebAuthnConfig {
    /// Create a new WebAuthn config
    pub fn new(rp_id: String, rp_name: String, origin: String) -> Self {
        Self {
            rp_id,
            rp_name,
            origin,
            user_verification: UserVerificationRequirement::Preferred,
            attestation: AttestationConveyancePreference::None,
        }
    }
}

/// User verification requirement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserVerificationRequirement {
    Required,
    Preferred,
    Discouraged,
}

/// Attestation conveyance preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttestationConveyancePreference {
    None,
    Indirect,
    Direct,
}

/// Security key registration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyCredentialCreationOptions {
    pub challenge: Vec<u8>,
    pub rp: RelyingParty,
    pub user: User,
    pub pub_key_cred_params: Vec<PublicKeyCredentialParameters>,
    pub timeout: u32,
    pub exclude_credentials: Vec<PublicKeyCredentialDescriptor>,
    pub authenticator_selection: AuthenticatorSelectionCriteria,
    pub attestation: AttestationConveyancePreference,
}

/// Relying party information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelyingParty {
    pub id: String,
    pub name: String,
}

/// User information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Vec<u8>,
    pub name: String,
    pub display_name: String,
}

/// Public key credential parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyCredentialParameters {
    pub type_: String,
    pub alg: i32,
}

/// Public key credential descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyCredentialDescriptor {
    pub type_: String,
    pub id: Vec<u8>,
    pub transports: Vec<String>,
}

/// Authenticator selection criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatorSelectionCriteria {
    pub authenticator_attachment: Option<AuthenticatorAttachment>,
    pub require_resident_key: bool,
    pub user_verification: UserVerificationRequirement,
}

/// Authenticator attachment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthenticatorAttachment {
    Platform,
    CrossPlatform,
}

/// Security key authentication options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyCredentialRequestOptions {
    pub challenge: Vec<u8>,
    pub timeout: u32,
    pub rp_id: String,
    pub allow_credentials: Vec<PublicKeyCredentialDescriptor>,
    pub user_verification: UserVerificationRequirement,
}

/// Security key credential
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityKey {
    pub id: String,
    pub name: String,
    pub credential_id: Vec<u8>,
    pub public_key: Vec<u8>,
    pub sign_count: u32,
    pub user_handle: Vec<u8>,
    pub transports: Vec<String>,
    pub backup_eligible: bool,
    pub backup_state: bool,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
}

impl SecurityKey {
    /// Create a new security key
    pub fn new(
        id: String,
        name: String,
        credential_id: Vec<u8>,
        public_key: Vec<u8>,
        user_handle: Vec<u8>,
    ) -> Self {
        Self {
            id,
            name,
            credential_id,
            public_key,
            sign_count: 0,
            user_handle,
            transports: vec!["usb".to_string(), "hybrid".to_string(), "internal".to_string()],
            backup_eligible: true,
            backup_state: false,
            created_at: Utc::now(),
            last_used: None,
        }
    }
    
    /// Increment sign counter
    pub fn increment_sign_count(&mut self) {
        self.sign_count += 1;
        self.last_used = Some(Utc::now());
    }
    
    /// Get time since last used
    pub fn time_since_last_use(&self) -> Option<chrono::Duration> {
        self.last_used.map(|t| Utc::now().signed_duration_since(t))
    }
}

/// WebAuthn registration response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResponse {
    pub id: String,
    pub raw_id: Vec<u8>,
    pub response: AuthenticatorAttestationResponse,
    pub type_: String,
}

/// Authenticator attestation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatorAttestationResponse {
    pub client_data_json: Vec<u8>,
    pub attestation_object: Vec<u8>,
}

/// WebAuthn authentication response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationResponse {
    pub id: String,
    pub raw_id: Vec<u8>,
    pub response: AuthenticatorAssertionResponse,
    pub type_: String,
}

/// Authenticator assertion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatorAssertionResponse {
    pub client_data_json: Vec<u8>,
    pub authenticator_data: Vec<u8>,
    pub signature: Vec<u8>,
    pub user_handle: Option<Vec<u8>>,
}

/// WebAuthn Manager
pub struct WebAuthnManager {
    config: WebAuthnConfig,
    registered_keys: Vec<SecurityKey>,
}

impl WebAuthnManager {
    /// Create a new WebAuthn manager
    pub fn new() -> Self {
        Self {
            config: WebAuthnConfig::new(
                "localhost".to_string(),
                "VantisWeb Browser".to_string(),
                "https://localhost".to_string(),
            ),
            registered_keys: Vec::new(),
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(config: WebAuthnConfig) -> Self {
        Self {
            config,
            registered_keys: Vec::new(),
        }
    }
    
    /// Generate registration challenge
    pub fn generate_registration_options(&self, username: &str) -> PublicKeyCredentialCreationOptions {
        let challenge = Self::generate_challenge();
        
        PublicKeyCredentialCreationOptions {
            challenge,
            rp: RelyingParty {
                id: self.config.rp_id.clone(),
                name: self.config.rp_name.clone(),
            },
            user: User {
                id: username.as_bytes().to_vec(),
                name: username.to_string(),
                display_name: username.to_string(),
            },
            pub_key_cred_params: vec![
                PublicKeyCredentialParameters {
                    type_: "public-key".to_string(),
                    alg: -7, // ES256
                },
                PublicKeyCredentialParameters {
                    type_: "public-key".to_string(),
                    alg: -257, // RS256
                },
            ],
            timeout: 60000,
            exclude_credentials: self
                .registered_keys
                .iter()
                .map(|key| PublicKeyCredentialDescriptor {
                    type_: "public-key".to_string(),
                    id: key.credential_id.clone(),
                    transports: key.transports.clone(),
                })
                .collect(),
            authenticator_selection: AuthenticatorSelectionCriteria {
                authenticator_attachment: None,
                require_resident_key: false,
                user_verification: self.config.user_verification,
            },
            attestation: self.config.attestation,
        }
    }
    
    /// Generate authentication challenge
    pub fn generate_authentication_options(&self, username: Option<&str>) -> PublicKeyCredentialRequestOptions {
        let challenge = Self::generate_challenge();
        
        let allow_credentials = if let Some(username) = username {
            self.registered_keys
                .iter()
                .filter(|key| {
                    // In real implementation, would match by user_handle
                    true
                })
                .map(|key| PublicKeyCredentialDescriptor {
                    type_: "public-key".to_string(),
                    id: key.credential_id.clone(),
                    transports: key.transports.clone(),
                })
                .collect()
        } else {
            self.registered_keys
                .iter()
                .map(|key| PublicKeyCredentialDescriptor {
                    type_: "public-key".to_string(),
                    id: key.credential_id.clone(),
                    transports: key.transports.clone(),
                })
                .collect()
        };
        
        PublicKeyCredentialRequestOptions {
            challenge,
            timeout: 60000,
            rp_id: self.config.rp_id.clone(),
            allow_credentials,
            user_verification: self.config.user_verification,
        }
    }
    
    /// Register a new security key
    pub async fn register_key(
        &mut self,
        response: RegistrationResponse,
        name: String,
        username: &str,
    ) -> Result<SecurityKey, WebAuthnError> {
        // In a real implementation, this would verify the attestation
        // and extract the public key
        
        let security_key = SecurityKey::new(
            uuid::Uuid::new_v4().to_string(),
            name,
            response.raw_id.clone(),
            vec![], // Public key would be extracted from response
            username.as_bytes().to_vec(),
        );
        
        self.registered_keys.push(security_key.clone());
        
        Ok(security_key)
    }
    
    /// Authenticate with a security key
    pub async fn authenticate(&mut self, response: AuthenticationResponse) -> Result<String, WebAuthnError> {
        // In a real implementation, this would verify the assertion signature
        
        // Find the matching key
        if let Some(key) = self.registered_keys
            .iter_mut()
            .find(|k| k.credential_id == response.raw_id)
        {
            key.increment_sign_count();
            Ok(key.id.clone())
        } else {
            Err(WebAuthnError::KeyNotFound)
        }
    }
    
    /// Get all registered keys
    pub fn get_keys(&self) -> &[SecurityKey] {
        &self.registered_keys
    }
    
    /// Remove a security key
    pub fn remove_key(&mut self, key_id: &str) -> Result<(), WebAuthnError> {
        let initial_len = self.registered_keys.len();
        self.registered_keys.retain(|k| k.id != key_id);
        
        if self.registered_keys.len() == initial_len {
            Err(WebAuthnError::KeyNotFound)
        } else {
            Ok(())
        }
    }
    
    /// Get a key by ID
    pub fn get_key(&self, key_id: &str) -> Option<&SecurityKey> {
        self.registered_keys.iter().find(|k| k.id == key_id)
    }
    
    /// Generate a random challenge
    fn generate_challenge() -> Vec<u8> {
        use rand::Rng;
        const CHALLENGE_SIZE: usize = 32;
        
        let mut rng = rand::thread_rng();
        let mut challenge = vec![0u8; CHALLENGE_SIZE];
        rng.fill(&mut challenge);
        challenge
    }
}

impl Default for WebAuthnManager {
    fn default() -> Self {
        Self::new()
    }
}

/// WebAuthn error types
#[derive(Debug, thiserror::Error)]
pub enum WebAuthnError {
    #[error("Invalid challenge")]
    InvalidChallenge,
    
    #[error("Invalid response")]
    InvalidResponse,
    
    #[error("Key not found")]
    KeyNotFound,
    
    #[error("Signature verification failed")]
    SignatureVerificationFailed,
    
    #[error("User verification failed")]
    UserVerificationFailed,
    
    #[error("Unsupported algorithm")]
    UnsupportedAlgorithm,
    
    #[error("Attestation verification failed")]
    AttestationVerificationFailed,
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_webauthn_manager_creation() {
        let manager = WebAuthnManager::new();
        assert_eq!(manager.config.rp_name, "VantisWeb Browser");
        assert!(manager.registered_keys.is_empty());
    }
    
    #[test]
    fn test_registration_options() {
        let manager = WebAuthnManager::new();
        let options = manager.generate_registration_options("testuser");
        
        assert_eq!(options.user.name, "testuser");
        assert_eq!(options.rp.name, "VantisWeb Browser");
        assert!(!options.challenge.is_empty());
        assert_eq!(options.pub_key_cred_params.len(), 2);
    }
    
    #[test]
    fn test_authentication_options() {
        let manager = WebAuthnManager::new();
        let options = manager.generate_authentication_options(None);
        
        assert!(!options.challenge.is_empty());
        assert_eq!(options.rp_id, "localhost");
    }
    
    #[test]
    fn test_security_key_creation() {
        let key = SecurityKey::new(
            "test-key".to_string(),
            "My YubiKey".to_string(),
            vec![1, 2, 3, 4],
            vec![5, 6, 7, 8],
            b"user".to_vec(),
        );
        
        assert_eq!(key.id, "test-key");
        assert_eq!(key.sign_count, 0);
        assert!(key.last_used.is_none());
    }
    
    #[test]
    fn test_sign_count_increment() {
        let mut key = SecurityKey::new(
            "test-key".to_string(),
            "My YubiKey".to_string(),
            vec![1, 2, 3, 4],
            vec![5, 6, 7, 8],
            b"user".to_vec(),
        );
        
        key.increment_sign_count();
        assert_eq!(key.sign_count, 1);
        assert!(key.last_used.is_some());
    }
}