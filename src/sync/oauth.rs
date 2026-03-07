// VantisWeb Browser - OAuth2 Authentication
// Copyright (c) 2024 VantisCorp
// OAuth2 authentication flow for cloud sync providers

use crate::sync::SyncError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// OAuth2 provider types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OAuth2Provider {
    Google,
    Dropbox,
    Microsoft,
    Apple,
}

impl OAuth2Provider {
    /// Get the authorization URL
    pub fn get_auth_url(
        &self,
        client_id: &str,
        redirect_uri: &str,
        state: &str,
        scopes: &[&str],
    ) -> String {
        match self {
            Self::Google => {
                format!(
                    "https://accounts.google.com/o/oauth2/v2/auth?\
                     client_id={}&\
                     redirect_uri={}&\
                     response_type=code&\
                     scope={}&\
                     access_type=offline&\
                     prompt=consent&\
                     state={}",
                    client_id,
                    urlencoding::encode(redirect_uri),
                    urlencoding::encode(&scopes.join(" ")),
                    state
                )
            }
            Self::Dropbox => {
                format!(
                    "https://www.dropbox.com/oauth2/authorize?\
                     client_id={}&\
                     redirect_uri={}&\
                     response_type=code&\
                     token_access_type=offline&\
                     state={}",
                    client_id,
                    urlencoding::encode(redirect_uri),
                    state
                )
            }
            Self::Microsoft => {
                format!(
                    "https://login.microsoftonline.com/common/oauth2/v2.0/authorize?\
                     client_id={}&\
                     redirect_uri={}&\
                     response_type=code&\
                     scope={}&\
                     state={}",
                    client_id,
                    urlencoding::encode(redirect_uri),
                    urlencoding::encode(&scopes.join(" ")),
                    state
                )
            }
            Self::Apple => {
                // Apple Sign In uses a different flow
                format!(
                    "https://appleid.apple.com/auth/authorize?\
                     client_id={}&\
                     redirect_uri={}&\
                     response_type=code&\
                     scope={}&\
                     state={}&\
                     response_mode=form_post",
                    client_id,
                    urlencoding::encode(redirect_uri),
                    urlencoding::encode(&scopes.join(" ")),
                    state
                )
            }
        }
    }
    
    /// Get the token exchange endpoint
    pub fn get_token_endpoint(&self) -> &str {
        match self {
            Self::Google => "https://oauth2.googleapis.com/token",
            Self::Dropbox => "https://api.dropboxapi.com/oauth2/token",
            Self::Microsoft => "https://login.microsoftonline.com/common/oauth2/v2.0/token",
            Self::Apple => "https://appleid.apple.com/auth/token",
        }
    }
    
    /// Get the token refresh endpoint
    pub fn get_refresh_endpoint(&self) -> &str {
        self.get_token_endpoint()
    }
    
    /// Get default scopes for this provider
    pub fn get_default_scopes(&self) -> Vec<&'static str> {
        match self {
            Self::Google => vec![
                "https://www.googleapis.com/auth/drive.appdata",
                "https://www.googleapis.com/auth/drive.file",
            ],
            Self::Dropbox => vec!["files.content.write", "files.content.read"],
            Self::Microsoft => vec!["User.Read", "Files.ReadWrite.AppFolder"],
            Self::Apple => vec!["name", "email"],
        }
    }
}

/// OAuth2 configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Config {
    pub provider: OAuth2Provider,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}

impl OAuth2Config {
    /// Create a new OAuth2 config
    pub fn new(
        provider: OAuth2Provider,
        client_id: String,
        client_secret: String,
        redirect_uri: String,
    ) -> Self {
        let scopes = provider
            .get_default_scopes()
            .iter()
            .map(|s| s.to_string())
            .collect();
        
        Self {
            provider,
            client_id,
            client_secret,
            redirect_uri,
            scopes,
        }
    }
    
    /// Generate an authorization URL
    pub fn generate_auth_url(&self, state: &str) -> String {
        self.provider.get_auth_url(
            &self.client_id,
            &self.redirect_uri,
            state,
            &self.scopes.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        )
    }
}

/// OAuth2 token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Token {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub token_type: String,
    pub scope: Option<String>,
}

impl OAuth2Token {
    /// Create a new token
    pub fn new(access_token: String, refresh_token: Option<String>, expires_in_seconds: i64) -> Self {
        Self {
            access_token,
            refresh_token,
            expires_at: Some(Utc::now() + chrono::Duration::seconds(expires_in_seconds)),
            token_type: "Bearer".to_string(),
            scope: None,
        }
    }
    
    /// Check if the token is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() >= expires_at - chrono::Duration::minutes(5) // 5 minute buffer
        } else {
            false
        }
    }
    
    /// Get the authorization header value
    pub fn get_auth_header(&self) -> Option<String> {
        Some(format!("{} {}", self.token_type, self.access_token))
    }
}

/// OAuth2 callback response
#[derive(Debug, Deserialize)]
pub struct OAuth2Callback {
    pub code: String,
    pub state: String,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

/// Token exchange request
#[derive(Debug, Serialize)]
struct TokenExchangeRequest {
    grant_type: String,
    code: String,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
}

/// Token refresh request
#[derive(Debug, Serialize)]
struct TokenRefreshRequest {
    grant_type: String,
    refresh_token: String,
    client_id: String,
    client_secret: String,
}

/// Token response
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: i64,
    token_type: String,
    scope: Option<String>,
}

/// OAuth2 client for handling authentication flows
pub struct OAuth2Client {
    config: OAuth2Config,
}

impl OAuth2Client {
    /// Create a new OAuth2 client
    pub fn new(config: OAuth2Config) -> Self {
        Self { config }
    }
    
    /// Exchange authorization code for tokens
    pub async fn exchange_code(&self, code: &str) -> Result<OAuth2Token, SyncError> {
        let client = reqwest::Client::new();
        let token_url = self.config.provider.get_token_endpoint();
        
        let request = TokenExchangeRequest {
            grant_type: "authorization_code".to_string(),
            code: code.to_string(),
            client_id: self.config.client_id.clone(),
            client_secret: self.config.client_secret.clone(),
            redirect_uri: self.config.redirect_uri.clone(),
        };
        
        let response = client
            .post(token_url)
            .form(&[
                ("code", &code.to_string()),
                ("client_id", &self.config.client_id),
                ("client_secret", &self.config.client_secret),
                ("redirect_uri", &self.config.redirect_uri),
                ("grant_type", &"authorization_code".to_string()),
            ])
            .send()
            .await
            .map_err(|e| SyncError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SyncError::AuthenticationFailed(format!(
                "Token exchange failed: {}", error_text
            )));
        }
        
        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| SyncError::AuthenticationFailed(e.to_string()))?;
        
        Ok(OAuth2Token {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
            expires_at: Some(Utc::now() + chrono::Duration::seconds(token_response.expires_in)),
            token_type: token_response.token_type,
            scope: token_response.scope,
        })
    }
    
    /// Refresh an expired token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<OAuth2Token, SyncError> {
        let client = reqwest::Client::new();
        let token_url = self.config.provider.get_refresh_endpoint();
        
        let response = client
            .post(token_url)
            .form(&[
                ("refresh_token", &refresh_token.to_string()),
                ("client_id", &self.config.client_id),
                ("client_secret", &self.config.client_secret),
                ("grant_type", &"refresh_token".to_string()),
            ])
            .send()
            .await
            .map_err(|e| SyncError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(SyncError::AuthenticationFailed(
                "Token refresh failed".to_string()
            ));
        }
        
        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| SyncError::AuthenticationFailed(e.to_string()))?;
        
        Ok(OAuth2Token {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
            expires_at: Some(Utc::now() + chrono::Duration::seconds(token_response.expires_in)),
            token_type: token_response.token_type,
            scope: token_response.scope,
        })
    }
    
    /// Revoke a token
    pub async fn revoke_token(&self, token: &str) -> Result<(), SyncError> {
        let client = reqwest::Client::new();
        
        let revoke_url = match self.config.provider {
            OAuth2Provider::Google => "https://oauth2.googleapis.com/revoke",
            _ => return Ok(()), // Other providers may not support revocation
        };
        
        let _ = client
            .post(revoke_url)
            .form(&[("token", token)])
            .send()
            .await;
        
        Ok(())
    }
    
    /// Get the OAuth2 config
    pub fn config(&self) -> &OAuth2Config {
        &self.config
    }
}

/// Generate a random state for OAuth2 flow
pub fn generate_state() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    
    (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_state() {
        let state1 = generate_state();
        let state2 = generate_state();
        
        assert_eq!(state1.len(), 32);
        assert_eq!(state2.len(), 32);
        assert_ne!(state1, state2);
    }
    
    #[test]
    fn test_token_expiration() {
        let token = OAuth2Token::new(
            "test_token".to_string(),
            Some("refresh_token".to_string()),
            3600, // 1 hour
        );
        
        assert!(!token.is_expired());
        
        let expired_token = OAuth2Token::new(
            "test_token".to_string(),
            Some("refresh_token".to_string()),
            -300, // 5 minutes ago
        );
        
        assert!(expired_token.is_expired());
    }
    
    #[test]
    fn test_auth_header() {
        let token = OAuth2Token::new(
            "test_token".to_string(),
            Some("refresh_token".to_string()),
            3600,
        );
        
        assert_eq!(token.get_auth_header(), Some("Bearer test_token".to_string()));
    }
}