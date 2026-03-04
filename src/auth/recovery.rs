// VantisWeb Browser - Account Recovery
// Copyright (c) 2024 VantisCorp
// Account recovery options including emergency access codes

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Recovery code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryCode {
    pub id: String,
    pub code: String,
    pub used: bool,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl RecoveryCode {
    /// Generate a new recovery code
    pub fn generate() -> Self {
        use rand::Rng;
        const CODE_LENGTH: usize = 10;
        const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
        
        let mut rng = rand::thread_rng();
        let code: String = (0..CODE_LENGTH)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            code,
            used: false,
            used_at: None,
            created_at: Utc::now(),
            expires_at: None,
        }
    }
    
    /// Generate with expiration
    pub fn generate_with_expiration(duration_days: u32) -> Self {
        let mut code = Self::generate();
        code.expires_at = Some(Utc::now() + chrono::Duration::days(duration_days as i64));
        code
    }
    
    /// Check if code is valid
    pub fn is_valid(&self) -> bool {
        !self.used
            && self
                .expires_at
                .map(|exp| Utc::now() < exp)
                .unwrap_or(true)
    }
    
    /// Mark code as used
    pub fn use_code(&mut self) {
        self.used = true;
        self.used_at = Some(Utc::now());
    }
}

/// Recovery option types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryOption {
    RecoveryCodes,
    SecurityQuestions,
    TrustedContact,
    Biometric,
    AdminReset,
}

/// Security question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityQuestion {
    pub id: String,
    pub question: String,
    pub answer_hash: String,
    pub created_at: DateTime<Utc>,
}

impl SecurityQuestion {
    /// Create a new security question
    pub fn new(question: String, answer: String) -> Self {
        let answer_hash = Self::hash_answer(&answer);
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            question,
            answer_hash,
            created_at: Utc::now(),
        }
    }
    
    /// Hash an answer
    fn hash_answer(answer: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let normalized = answer.to_lowercase().trim().to_string();
        let mut hasher = DefaultHasher::new();
        normalized.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
    
    /// Verify an answer
    pub fn verify(&self, answer: &str) -> bool {
        Self::hash_answer(answer) == self.answer_hash
    }
}

/// Trusted contact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedContact {
    pub id: String,
    pub email: String,
    pub name: String,
    pub relationship: String,
    pub verified: bool,
    pub verification_code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
}

impl TrustedContact {
    /// Create a new trusted contact
    pub fn new(email: String, name: String, relationship: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            email,
            name,
            relationship,
            verified: false,
            verification_code: None,
            created_at: Utc::now(),
            last_used: None,
        }
    }
    
    /// Generate verification code
    pub fn generate_verification_code(&mut self) -> String {
        use rand::Rng;
        const CODE_LENGTH: usize = 6;
        
        let code: String = (0..CODE_LENGTH)
            .map(|_| rand::thread_rng().gen_range(0..10).to_string())
            .collect();
        
        self.verification_code = Some(code.clone());
        code
    }
    
    /// Verify contact
    pub fn verify(&mut self, code: &str) -> bool {
        if self.verification_code.as_deref() == Some(code) {
            self.verified = true;
            self.verification_code = None;
            true
        } else {
            false
        }
    }
}

/// Recovery manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryConfig {
    /// Number of recovery codes to generate
    pub num_recovery_codes: usize,
    
    /// Recovery code expiration in days (None for no expiration)
    pub code_expiration_days: Option<u32>,
    
    /// Number of security questions required
    pub num_security_questions: usize,
    
    /// Maximum trusted contacts
    pub max_trusted_contacts: usize,
    
    /// Trusted contact verification timeout in hours
    pub verification_timeout_hours: u32,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            num_recovery_codes: 10,
            code_expiration_days: None,
            num_security_questions: 3,
            max_trusted_contacts: 3,
            verification_timeout_hours: 72,
        }
    }
}

/// Recovery manager
pub struct RecoveryManager {
    config: RecoveryConfig,
    recovery_codes: HashMap<String, Vec<RecoveryCode>>,
    security_questions: HashMap<String, Vec<SecurityQuestion>>,
    trusted_contacts: HashMap<String, Vec<TrustedContact>>,
}

impl RecoveryManager {
    /// Create a new recovery manager
    pub fn new() -> Self {
        Self::with_config(RecoveryConfig::default())
    }
    
    /// Create with custom configuration
    pub fn with_config(config: RecoveryConfig) -> Self {
        Self {
            config,
            recovery_codes: HashMap::new(),
            security_questions: HashMap::new(),
            trusted_contacts: HashMap::new(),
        }
    }
    
    /// Generate recovery codes for a user
    pub fn generate_recovery_codes(&mut self, user_id: &str) -> Vec<RecoveryCode> {
        let codes: Vec<RecoveryCode> = (0..self.config.num_recovery_codes)
            .map(|_| {
                if let Some(exp_days) = self.config.code_expiration_days {
                    RecoveryCode::generate_with_expiration(exp_days)
                } else {
                    RecoveryCode::generate()
                }
            })
            .collect();
        
        self.recovery_codes.insert(user_id.to_string(), codes.clone());
        codes
    }
    
    /// Get recovery codes for a user
    pub fn get_recovery_codes(&self, user_id: &str) -> Vec<RecoveryCode> {
        self.recovery_codes.get(user_id).cloned().unwrap_or_default()
    }
    
    /// Validate a recovery code
    pub fn validate_recovery_code(&mut self, user_id: &str, code: &str) -> Result<bool, RecoveryError> {
        let codes = self.recovery_codes.get_mut(user_id)
            .ok_or(RecoveryError::NoRecoveryCodes)?;
        
        for recovery_code in codes.iter_mut() {
            if recovery_code.code == code {
                if recovery_code.is_valid() {
                    recovery_code.use_code();
                    return Ok(true);
                } else {
                    return Err(RecoveryError::CodeUsed);
                }
            }
        }
        
        Err(RecoveryError::InvalidCode)
    }
    
    /// Set security questions for a user
    pub fn set_security_questions(
        &mut self,
        user_id: &str,
        questions: Vec<(String, String)>,
    ) -> Result<(), RecoveryError> {
        if questions.len() != self.config.num_security_questions {
            return Err(RecoveryError::InvalidQuestionCount);
        }
        
        let security_questions: Vec<SecurityQuestion> = questions
            .into_iter()
            .map(|(q, a)| SecurityQuestion::new(q, a))
            .collect();
        
        self.security_questions.insert(user_id.to_string(), security_questions);
        Ok(())
    }
    
    /// Get security questions for a user
    pub fn get_security_questions(&self, user_id: &str) -> Vec<String> {
        self.security_questions
            .get(user_id)
            .map(|qs| qs.iter().map(|q| q.question.clone()).collect())
            .unwrap_or_default()
    }
    
    /// Verify security question answers
    pub fn verify_security_questions(
        &self,
        user_id: &str,
        answers: &[(String, String)],
    ) -> Result<bool, RecoveryError> {
        let questions = self.security_questions.get(user_id)
            .ok_or(RecoveryError::NoSecurityQuestions)?;
        
        let mut correct_count = 0;
        
        for (question_id, answer) in answers {
            if let Some(question) = questions.iter().find(|q| &q.id == question_id) {
                if question.verify(answer) {
                    correct_count += 1;
                }
            }
        }
        
        if correct_count >= self.config.num_security_questions {
            Ok(true)
        } else {
            Err(RecoveryError::IncorrectAnswers)
        }
    }
    
    /// Add a trusted contact
    pub fn add_trusted_contact(
        &mut self,
        user_id: &str,
        email: String,
        name: String,
        relationship: String,
    ) -> Result<TrustedContact, RecoveryError> {
        let contacts = self.trusted_contacts.entry(user_id.to_string())
            .or_insert_with(Vec::new);
        
        if contacts.len() >= self.config.max_trusted_contacts {
            return Err(RecoveryError::MaxContactsReached);
        }
        
        let contact = TrustedContact::new(email, name, relationship);
        contacts.push(contact.clone());
        
        Ok(contact)
    }
    
    /// Get trusted contacts for a user
    pub fn get_trusted_contacts(&self, user_id: &str) -> Vec<TrustedContact> {
        self.trusted_contacts.get(user_id).cloned().unwrap_or_default()
    }
    
    /// Verify a trusted contact
    pub fn verify_trusted_contact(
        &mut self,
        user_id: &str,
        contact_id: &str,
        code: &str,
    ) -> Result<bool, RecoveryError> {
        let contacts = self.trusted_contacts.get_mut(user_id)
            .ok_or(RecoveryError::ContactNotFound)?;
        
        if let Some(contact) = contacts.iter_mut().find(|c| c.id == contact_id) {
            Ok(contact.verify(code))
        } else {
            Err(RecoveryError::ContactNotFound)
        }
    }
    
    /// Remove a trusted contact
    pub fn remove_trusted_contact(
        &mut self,
        user_id: &str,
        contact_id: &str,
    ) -> Result<(), RecoveryError> {
        let contacts = self.trusted_contacts.get_mut(user_id)
            .ok_or(RecoveryError::ContactNotFound)?;
        
        let initial_len = contacts.len();
        contacts.retain(|c| c.id != contact_id);
        
        if contacts.len() == initial_len {
            Err(RecoveryError::ContactNotFound)
        } else {
            Ok(())
        }
    }
    
    /// Check if user has recovery options set up
    pub fn has_recovery_options(&self, user_id: &str) -> bool {
        self.recovery_codes.contains_key(user_id)
            || self.security_questions.contains_key(user_id)
            || self.trusted_contacts.contains_key(user_id)
    }
    
    /// Get available recovery options for a user
    pub fn get_available_options(&self, user_id: &str) -> Vec<RecoveryOption> {
        let mut options = Vec::new();
        
        if self.recovery_codes.get(user_id).map(|c| c.iter().any(|c| c.is_valid())).unwrap_or(false) {
            options.push(RecoveryOption::RecoveryCodes);
        }
        
        if self.security_questions.contains_key(user_id) {
            options.push(RecoveryOption::SecurityQuestions);
        }
        
        if self.trusted_contacts.get(user_id).map(|c| c.iter().any(|tc| tc.verified)).unwrap_or(false) {
            options.push(RecoveryOption::TrustedContact);
        }
        
        options
    }
}

impl Default for RecoveryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Recovery error types
#[derive(Debug, thiserror::Error)]
pub enum RecoveryError {
    #[error("Invalid recovery code")]
    InvalidCode,
    
    #[error("Recovery code already used")]
    CodeUsed,
    
    #[error("No recovery codes found")]
    NoRecoveryCodes,
    
    #[error("Invalid question count")]
    InvalidQuestionCount,
    
    #[error("No security questions found")]
    NoSecurityQuestions,
    
    #[error("Incorrect answers")]
    IncorrectAnswers,
    
    #[error("Maximum contacts reached")]
    MaxContactsReached,
    
    #[error("Contact not found")]
    ContactNotFound,
    
    #[error("Verification failed")]
    VerificationFailed,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_recovery_code_generation() {
        let code = RecoveryCode::generate();
        assert_eq!(code.code.len(), 10);
        assert!(code.is_valid());
        assert!(!code.used);
    }
    
    #[test]
    fn test_recovery_code_usage() {
        let mut code = RecoveryCode::generate();
        assert!(code.is_valid());
        
        code.use_code();
        assert!(code.used);
        assert!(!code.is_valid());
    }
    
    #[test]
    fn test_security_question() {
        let question = SecurityQuestion::new(
            "What is your pet's name?".to_string(),
            "Fluffy".to_string(),
        );
        
        assert!(question.verify("Fluffy"));
        assert!(question.verify("fluffy")); // Case insensitive
        assert!(question.verify("  fluffy  ")); // Trimmed
    }
    
    #[test]
    fn test_recovery_manager() {
        let mut manager = RecoveryManager::new();
        
        let codes = manager.generate_recovery_codes("user123");
        assert_eq!(codes.len(), 10);
        
        let valid_code = &codes[0].code;
        let result = manager.validate_recovery_code("user123", valid_code);
        assert!(result.is_ok());
    }
}