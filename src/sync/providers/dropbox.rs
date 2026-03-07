// VantisWeb Browser - Dropbox Sync Provider
// Copyright (c) 2024 VantisCorp
// Implementation of Dropbox sync functionality

use super::{
    SyncProvider, SyncProviderType, SyncCredentials, SyncStatus, 
    RemoteFile, StorageInfo, FileChange, FileChangeType
};
use crate::sync::{SyncError, SyncResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Dropbox API configuration
const DROPBOX_API_BASE: &str = "https://api.dropboxapi.com/2";
const DROPBOX_CONTENT_BASE: &str = "https://content.dropboxapi.com/2";

/// Dropbox sync provider
#[derive(Debug)]
pub struct DropboxProvider {
    access_token: Option<String>,
    refresh_token: Option<String>,
    token_expires_at: Option<DateTime<Utc>>,
    app_key: String,
    app_secret: String,
    root_path: String,
}

impl DropboxProvider {
    /// Create a new Dropbox provider
    pub fn new(app_key: String, app_secret: String) -> Self {
        Self {
            access_token: None,
            refresh_token: None,
            token_expires_at: None,
            app_key,
            app_secret,
            root_path: "/VantisWeb".to_string(),
        }
    }
    
    /// Create with default OAuth credentials
    pub fn with_default_credentials() -> Self {
        Self::new(
            "vantisweb-dropbox-app".to_string(),
            "default-app-secret".to_string(),
        )
    }
    
    /// Get OAuth2 authorization URL
    pub fn get_authorization_url(&self, redirect_uri: &str, state: &str) -> String {
        format!(
            "https://www.dropbox.com/oauth2/authorize?\
             client_id={}&\
             redirect_uri={}&\
             response_type=code&\
             token_access_type=offline&\
             state={}",
            self.app_key,
            urlencoding::encode(redirect_uri),
            state
        )
    }
    
    /// Exchange authorization code for tokens
    pub async fn exchange_code(&mut self, code: &str, redirect_uri: &str) -> SyncResult<()> {
        let client = reqwest::Client::new();
        
        let response = client
            .post("https://api.dropboxapi.com/oauth2/token")
            .form(&[
                ("code", code),
                ("client_id", &self.app_key),
                ("client_secret", &self.app_secret),
                ("redirect_uri", redirect_uri),
                ("grant_type", "authorization_code"),
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
        
        let token_response: DropboxTokenResponse = response
            .json()
            .await
            .map_err(|e| SyncError::AuthenticationFailed(e.to_string()))?;
        
        self.access_token = Some(token_response.access_token);
        self.refresh_token = token_response.refresh_token;
        self.token_expires_at = Some(Utc::now() + chrono::Duration::seconds(token_response.expires_in));
        
        Ok(())
    }
    
    /// Make authenticated API request
    async fn api_request(
        &self,
        endpoint: &str,
        body: Option<&serde_json::Value>,
    ) -> SyncResult<reqwest::Response> {
        let token = self.access_token.as_ref()
            .ok_or(SyncError::AuthenticationFailed("Not authenticated".to_string()))?;
        
        let client = reqwest::Client::new();
        let url = format!("{}{}", DROPBOX_API_BASE, endpoint);
        
        let mut request = client
            .post(&url)
            .bearer_auth(token)
            .header("Content-Type", "application/json");
        
        if let Some(json_body) = body {
            request = request.json(json_body);
        }
        
        request
            .send()
            .await
            .map_err(|e| SyncError::NetworkError(e.to_string()))
    }
    
    /// Make authenticated content request
    async fn content_request(
        &self,
        endpoint: &str,
        path: &str,
        data: Option<&[u8]>,
    ) -> SyncResult<reqwest::Response> {
        let token = self.access_token.as_ref()
            .ok_or(SyncError::AuthenticationFailed("Not authenticated".to_string()))?;
        
        let client = reqwest::Client::new();
        let url = format!("{}{}", DROPBOX_CONTENT_BASE, endpoint);
        
        let args = serde_json::json!({ "path": path });
        
        let mut request = client
            .post(&url)
            .bearer_auth(token)
            .header("Dropbox-API-Arg", args.to_string());
        
        if let Some(content) = data {
            request = request.body(content.to_vec());
        }
        
        request
            .send()
            .await
            .map_err(|e| SyncError::NetworkError(e.to_string()))
    }
    
    /// Ensure root folder exists
    pub async fn ensure_root_folder(&self) -> SyncResult<()> {
        let body = serde_json::json!({
            "path": self.root_path,
            "autorename": false
        });
        
        let result = self.api_request("/files/create_folder_v2", Some(&body)).await;
        
        // Ignore error if folder already exists
        match result {
            Ok(_) => Ok(()),
            Err(SyncError::ProviderError(msg)) if msg.contains("conflict") => Ok(()),
            Err(e) => Err(e),
        }
    }
    
    /// Convert Dropbox metadata to RemoteFile
    fn convert_metadata(metadata: DropboxFileMetadata) -> RemoteFile {
        RemoteFile {
            id: metadata.id,
            name: metadata.name,
            path: metadata.path_display,
            size: metadata.size,
            modified: DateTime::parse_from_rfc3339(&metadata.client_modified)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            created: DateTime::parse_from_rfc3339(&metadata.client_modified)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            hash: metadata.content_hash.unwrap_or_default(),
            version: Some(metadata.rev),
        }
    }
}

#[async_trait]
impl SyncProvider for DropboxProvider {
    fn provider_type(&self) -> SyncProviderType {
        SyncProviderType::Dropbox
    }
    
    async fn authenticate(&mut self, credentials: &SyncCredentials) -> SyncResult<()> {
        match credentials {
            SyncCredentials::OAuth2 { access_token, refresh_token, expires_at } => {
                self.access_token = Some(access_token.clone());
                self.refresh_token = Some(refresh_token.clone());
                self.token_expires_at = Some(*expires_at);
                self.ensure_root_folder().await?;
                Ok(())
            }
            _ => Err(SyncError::AuthenticationFailed(
                "Dropbox requires OAuth2 credentials".to_string()
            )),
        }
    }
    
    fn is_authenticated(&self) -> bool {
        self.access_token.is_some()
    }
    
    async fn refresh_auth(&mut self) -> SyncResult<()> {
        let refresh_token = self.refresh_token.as_ref()
            .ok_or(SyncError::AuthenticationFailed("No refresh token".to_string()))?;
        
        let client = reqwest::Client::new();
        let response = client
            .post("https://api.dropboxapi.com/oauth2/token")
            .form(&[
                ("refresh_token", refresh_token.as_str()),
                ("client_id", self.app_key.as_str()),
                ("client_secret", self.app_secret.as_str()),
                ("grant_type", "refresh_token"),
            ])
            .send()
            .await
            .map_err(|e| SyncError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(SyncError::AuthenticationFailed("Token refresh failed".to_string()));
        }
        
        let token_response: DropboxTokenResponse = response
            .json()
            .await
            .map_err(|e| SyncError::AuthenticationFailed(e.to_string()))?;
        
        self.access_token = Some(token_response.access_token);
        self.token_expires_at = Some(Utc::now() + chrono::Duration::seconds(token_response.expires_in));
        
        Ok(())
    }
    
    async fn disconnect(&mut self) -> SyncResult<()> {
        // Revoke token
        if self.access_token.is_some() {
            let _ = self.api_request("/auth/token/revoke", None).await;
        }
        
        self.access_token = None;
        self.refresh_token = None;
        self.token_expires_at = None;
        Ok(())
    }
    
    async fn get_status(&self) -> SyncResult<SyncStatus> {
        let storage_info = self.get_storage_info().await?;
        
        Ok(SyncStatus {
            provider: SyncProviderType::Dropbox,
            connected: self.is_authenticated(),
            last_sync: None,
            last_sync_status: None,
            pending_changes: 0,
            storage_used: storage_info.used_bytes,
            storage_limit: storage_info.limit_bytes,
            sync_in_progress: false,
            error_count: 0,
            last_error: None,
        })
    }
    
    async fn list_remote_files(&self, path: &str) -> SyncResult<Vec<RemoteFile>> {
        let full_path = if path.is_empty() {
            self.root_path.clone()
        } else {
            format!("{}/{}", self.root_path, path)
        };
        
        let body = serde_json::json!({
            "path": full_path,
            "recursive": false,
            "include_deleted": false
        });
        
        let response = self.api_request("/files/list_folder", Some(&body)).await?;
        
        let list_result: DropboxListFolderResult = response
            .json()
            .await
            .map_err(|e| SyncError::ProviderError(e.to_string()))?;
        
        let files: Vec<RemoteFile> = list_result
            .entries
            .into_iter()
            .filter_map(|entry| {
                if entry.file_type == "file" {
                    Some(RemoteFile {
                        id: entry.id,
                        name: entry.name,
                        path: entry.path_display,
                        size: entry.size.unwrap_or(0),
                        modified: DateTime::parse_from_rfc3339(&entry.client_modified?)
                            .ok()?
                            .with_timezone(&Utc),
                        created: DateTime::parse_from_rfc3339(&entry.client_modified?)
                            .ok()?
                            .with_timezone(&Utc),
                        hash: entry.content_hash.unwrap_or_default(),
                        version: entry.rev,
                    })
                } else {
                    None
                }
            })
            .collect();
        
        Ok(files)
    }
    
    async fn upload_file(&self, _local_path: &str, remote_path: &str, data: &[u8]) -> SyncResult<RemoteFile> {
        let full_path = format!("{}/{}", self.root_path, remote_path);
        
        let response = self.content_request("/files/upload", &full_path, Some(data)).await?;
        
        if !response.status().is_success() {
            return Err(SyncError::ProviderError(format!(
                "Upload failed: {}", response.status()
            )));
        }
        
        let metadata: DropboxFileMetadata = response
            .json()
            .await
            .map_err(|e| SyncError::ProviderError(e.to_string()))?;
        
        Ok(Self::convert_metadata(metadata))
    }
    
    async fn download_file(&self, remote_id: &str) -> SyncResult<Vec<u8>> {
        // remote_id is the path for Dropbox
        let response = self.content_request("/files/download", remote_id, None).await?;
        
        let bytes = response
            .bytes()
            .await
            .map_err(|e| SyncError::NetworkError(e.to_string()))?;
        
        Ok(bytes.to_vec())
    }
    
    async fn delete_file(&self, remote_id: &str) -> SyncResult<()> {
        let body = serde_json::json!({
            "path": remote_id
        });
        
        let response = self.api_request("/files/delete_v2", Some(&body)).await?;
        
        if !response.status().is_success() {
            return Err(SyncError::ProviderError("Delete failed".to_string()));
        }
        
        Ok(())
    }
    
    async fn create_folder(&self, path: &str) -> SyncResult<RemoteFile> {
        let full_path = format!("{}/{}", self.root_path, path);
        
        let body = serde_json::json!({
            "path": full_path,
            "autorename": false
        });
        
        let response = self.api_request("/files/create_folder_v2", Some(&body)).await?;
        
        let result: DropboxCreateFolderResult = response
            .json()
            .await
            .map_err(|e| SyncError::ProviderError(e.to_string()))?;
        
        Ok(RemoteFile {
            id: result.metadata.id,
            name: result.metadata.name,
            path: result.metadata.path_display,
            size: 0,
            modified: Utc::now(),
            created: Utc::now(),
            hash: String::new(),
            version: None,
        })
    }
    
    async fn get_storage_info(&self) -> SyncResult<StorageInfo> {
        let response = self.api_request("/users/get_space_usage", None).await?;
        
        let usage: DropboxSpaceUsage = response
            .json()
            .await
            .map_err(|e| SyncError::ProviderError(e.to_string()))?;
        
        Ok(StorageInfo {
            used_bytes: usage.used,
            limit_bytes: Some(usage.allocation.individual.allocated),
            plan_name: usage.allocation.team.map(|t| t.name).flatten(),
        })
    }
    
    async fn subscribe_changes(&self) -> SyncResult<()> {
        // Dropbox doesn't have native push notifications
        // Would need to implement polling or webhook server
        Ok(())
    }
    
    async fn get_changes(&self, _since: DateTime<Utc>) -> SyncResult<Vec<FileChange>> {
        // Get cursor and list changes
        let body = serde_json::json!({
            "path": self.root_path,
            "recursive": true,
            "include_deleted": true
        });
        
        let response = self.api_request("/files/list_folder/get_latest_cursor", Some(&body)).await?;
        
        let _cursor_result: DropboxCursorResult = response
            .json()
            .await
            .map_err(|e| SyncError::ProviderError(e.to_string()))?;
        
        // Would continue to fetch changes using the cursor
        Ok(Vec::new())
    }
}

// API Response Types

#[derive(Debug, Deserialize)]
struct DropboxTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: i64,
    token_type: String,
}

#[derive(Debug, Deserialize)]
struct DropboxFileMetadata {
    id: String,
    name: String,
    path_display: String,
    size: u64,
    client_modified: String,
    server_modified: String,
    rev: String,
    content_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DropboxListFolderResult {
    entries: Vec<DropboxMetadataEntry>,
    cursor: String,
    has_more: bool,
}

#[derive(Debug, Deserialize)]
struct DropboxMetadataEntry {
    #[serde(rename = ".tag")]
    file_type: String,
    id: String,
    name: String,
    path_display: String,
    size: Option<u64>,
    client_modified: Option<String>,
    server_modified: Option<String>,
    rev: Option<String>,
    content_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DropboxCreateFolderResult {
    metadata: DropboxFolderMetadata,
}

#[derive(Debug, Deserialize)]
struct DropboxFolderMetadata {
    id: String,
    name: String,
    path_display: String,
}

#[derive(Debug, Deserialize)]
struct DropboxSpaceUsage {
    used: u64,
    allocation: DropboxSpaceAllocation,
}

#[derive(Debug, Deserialize)]
struct DropboxSpaceAllocation {
    #[serde(rename = ".tag")]
    allocation_type: String,
    individual: DropboxIndividualAllocation,
    team: Option<DropboxTeamAllocation>,
}

#[derive(Debug, Deserialize)]
struct DropboxIndividualAllocation {
    allocated: u64,
}

#[derive(Debug, Deserialize)]
struct DropboxTeamAllocation {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DropboxCursorResult {
    cursor: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_provider_creation() {
        let provider = DropboxProvider::with_default_credentials();
        assert_eq!(provider.provider_type(), SyncProviderType::Dropbox);
        assert!(!provider.is_authenticated());
    }
    
    #[test]
    fn test_authorization_url() {
        let provider = DropboxProvider::with_default_credentials();
        let url = provider.get_authorization_url("http://localhost/callback", "state123");
        
        assert!(url.contains("dropbox.com/oauth2/authorize"));
        assert!(url.contains("client_id="));
        assert!(url.contains("state=state123"));
    }
}