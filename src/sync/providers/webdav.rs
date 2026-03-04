// VantisWeb Browser - WebDAV Sync Provider
// Copyright (c) 2024 VantisCorp
// Implementation of WebDAV sync functionality for self-hosted solutions

use super::{
    SyncProvider, SyncProviderType, SyncCredentials, SyncStatus, 
    RemoteFile, StorageInfo, FileChange, FileChangeType
};
use crate::sync::{SyncError, SyncResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// WebDAV sync provider for self-hosted and custom cloud solutions
#[derive(Debug)]
pub struct WebDAVProvider {
    server_url: String,
    username: Option<String>,
    password: Option<String>,
    api_key: Option<String>,
    root_path: String,
    connected: bool,
}

impl WebDAVProvider {
    /// Create a new WebDAV provider
    pub fn new(server_url: String, root_path: String) -> Self {
        Self {
            server_url,
            username: None,
            password: None,
            api_key: None,
            root_path,
            connected: false,
        }
    }
    
    /// Create with a standard Nextcloud server
    pub fn nextcloud(server_url: String, username: String) -> Self {
        Self {
            server_url,
            username: Some(username),
            password: None,
            api_key: None,
            root_path: "/remote.php/dav/files".to_string(),
            connected: false,
        }
    }
    
    /// Create with a standard ownCloud server
    pub fn owncloud(server_url: String, username: String) -> Self {
        Self {
            server_url,
            username: Some(username),
            password: None,
            api_key: None,
            root_path: "/remote.php/dav/files".to_string(),
            connected: false,
        }
    }
    
    /// Get the full URL for a path
    fn get_full_url(&self, path: &str) -> String {
        let base = self.server_url.trim_end_matches('/');
        let root = self.root_path.trim_start_matches('/');
        let path = path.trim_start_matches('/');
        
        if path.is_empty() {
            format!("{}/{}", base, root)
        } else {
            format!("{}/{}/{}", base, root, path)
        }
    }
    
    /// Make a PROPFIND request to list files
    async fn propfind(&self, path: &str, depth: u32) -> SyncResult<String> {
        let client = reqwest::Client::new();
        let url = self.get_full_url(path);
        
        let mut request = client
            .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), &url)
            .header("Depth", depth.to_string())
            .header("Content-Type", "application/xml; charset=utf-8");

        // Add authentication
        if let (Some(user), Some(pass)) = (&self.username, &self.password) {
            request = request.basic_auth(user, Some(pass));
        } else if let Some(key) = &self.api_key {
            request = request.bearer_auth(key);
        }
        
        // PROPFIND request body
        let body = r#"<?xml version="1.0" encoding="utf-8"?>
            <d:propfind xmlns:d="DAV:">
                <d:prop>
                    <d:displayname/>
                    <d:getlastmodified/>
                    <d:getcontentlength/>
                    <d:getetag/>
                    <d:resourcetype/>
                    <d:getcontenttype/>
                </d:prop>
            </d:propfind>"#;
        
        request = request.body(body.to_string());
        
        let response = request
            .send()
            .await
            .map_err(|e| SyncError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(SyncError::ProviderError(format!(
                "PROPFIND failed: {}", response.status()
            )));
        }
        
        response
            .text()
            .await
            .map_err(|e| SyncError::NetworkError(e.to_string()))
    }
    
    /// Make a MKCOL request to create a directory
    async fn mkcol(&self, path: &str) -> SyncResult<()> {
        let client = reqwest::Client::new();
        let url = self.get_full_url(path);
        
        let mut request = client
            .request(reqwest::Method::from_bytes(b"MKCOL").unwrap(), &url);

        if let (Some(user), Some(pass)) = (&self.username, &self.password) {
            request = request.basic_auth(user, Some(pass));
        } else if let Some(key) = &self.api_key {
            request = request.bearer_auth(key);
        }
        
        let response = request
            .send()
            .await
            .map_err(|e| SyncError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() && response.status() != 405 {
            // 405 means the directory already exists
            return Err(SyncError::ProviderError(format!(
                "MKCOL failed: {}", response.status()
            )));
        }
        
        Ok(())
    }
    
    /// Parse WebDAV XML response
    fn parse_propfind_response(&self, xml: &str, base_path: &str) -> Vec<RemoteFile> {
        let mut files = Vec::new();
        
        // Simple XML parsing for WebDAV response
        // In production, use a proper XML parser like quick-xml
        
        // Extract response elements
        let responses: Vec<&str> = xml.split("<d:response>")
            .skip(1)
            .collect();
        
        for response in responses {
            // Extract href
            let href = extract_xml_tag(response, "d:href")
                .or_else(|| extract_xml_tag(response, "href"))
                .unwrap_or_default();
            
            // Skip the collection itself
            if href.trim_end_matches('/') == base_path.trim_end_matches('/') {
                continue;
            }
            
            // Extract display name
            let name = extract_xml_tag(response, "d:displayname")
                .or_else(|| extract_xml_tag(response, "displayname"))
                .or_else(|| {
                    // Extract from href if displayname not available
                    href.trim_end_matches('/').split('/').last()
                        .map(|s| urlencoding::decode(s).unwrap_or_default().to_string())
                })
                .unwrap_or_else(|| "Unknown".to_string());
            
            // Extract content length
            let size: u64 = extract_xml_tag(response, "d:getcontentlength")
                .or_else(|| extract_xml_tag(response, "getcontentlength"))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            
            // Extract last modified
            let modified_str = extract_xml_tag(response, "d:getlastmodified")
                .or_else(|| extract_xml_tag(response, "getlastmodified"))
                .unwrap_or_else(|| Utc::now().to_rfc3339());
            
            let modified = DateTime::parse_from_rfc2822(&modified_str)
                .map(|dt| dt.with_timezone(&Utc))
                .or_else(|_| DateTime::parse_from_rfc3339(&modified_str)
                    .map(|dt| dt.with_timezone(&Utc)))
                .unwrap_or_else(|_| Utc::now());
            
            // Extract ETag as hash
            let hash = extract_xml_tag(response, "d:getetag")
                .or_else(|| extract_xml_tag(response, "getetag"))
                .unwrap_or_default()
                .trim_matches('"')
                .to_string();
            
            // Check if it's a directory
            let is_directory = response.contains("<d:collection/>") 
                || response.contains("<d:resourcetype><d:collection/></d:resourcetype>");
            
            if !is_directory {
                files.push(RemoteFile {
                    id: href.clone(),
                    name,
                    path: href.clone(),
                    size,
                    modified,
                    created: modified,
                    hash,
                    version: None,
                });
            }
        }
        
        files
    }
    
    /// Ensure the root directory exists
    pub async fn ensure_root_directory(&self) -> SyncResult<()> {
        self.mkcol(&self.root_path).await
    }
}

/// Helper function to extract XML tag content
fn extract_xml_tag(xml: &str, tag: &str) -> Option<String> {
    let start_tag = format!("<{}>", tag);
    let end_tag = format!("</{}>", tag);
    
    if let Some(start) = xml.find(&start_tag) {
        let start = start + start_tag.len();
        if let Some(end) = xml[start..].find(&end_tag) {
            return Some(xml[start..start + end].to_string());
        }
    }
    None
}

#[async_trait]
impl SyncProvider for WebDAVProvider {
    fn provider_type(&self) -> SyncProviderType {
        SyncProviderType::WebDAV
    }
    
    async fn authenticate(&mut self, credentials: &SyncCredentials) -> SyncResult<()> {
        match credentials {
            SyncCredentials::BasicAuth { username, password } => {
                self.username = Some(username.clone());
                self.password = Some(password.clone());
            }
            SyncCredentials::ApiKey { key } => {
                self.api_key = Some(key.clone());
            }
            _ => return Err(SyncError::AuthenticationFailed(
                "WebDAV requires BasicAuth or ApiKey credentials".to_string()
            )),
        }
        
        // Verify connection by attempting to list root
        self.ensure_root_directory().await?;
        
        match self.propfind("", 0).await {
            Ok(_) => {
                self.connected = true;
                Ok(())
            }
            Err(e) => {
                self.connected = false;
                Err(e)
            }
        }
    }
    
    fn is_authenticated(&self) -> bool {
        self.connected
    }
    
    async fn refresh_auth(&mut self) -> SyncResult<()> {
        // WebDAV doesn't have token refresh - just verify connection
        self.connected = self.propfind("", 0).await.is_ok();
        Ok(())
    }
    
    async fn disconnect(&mut self) -> SyncResult<()> {
        self.username = None;
        self.password = None;
        self.api_key = None;
        self.connected = false;
        Ok(())
    }
    
    async fn get_status(&self) -> SyncResult<SyncStatus> {
        Ok(SyncStatus {
            provider: SyncProviderType::WebDAV,
            connected: self.connected,
            last_sync: None,
            last_sync_status: None,
            pending_changes: 0,
            storage_used: 0,
            storage_limit: None, // WebDAV doesn't provide quota info
            sync_in_progress: false,
            error_count: 0,
            last_error: None,
        })
    }
    
    async fn list_remote_files(&self, path: &str) -> SyncResult<Vec<RemoteFile>> {
        let xml = self.propfind(path, 1).await?;
        let full_path = self.get_full_url(path);
        Ok(self.parse_propfind_response(&xml, &full_path))
    }
    
    async fn upload_file(&self, _local_path: &str, remote_path: &str, data: &[u8]) -> SyncResult<RemoteFile> {
        let client = reqwest::Client::new();
        let url = self.get_full_url(remote_path);
        
        let mut request = client
            .put(&url)
            .header("Content-Type", "application/octet-stream")
            .body(data.to_vec());

        if let (Some(user), Some(pass)) = (&self.username, &self.password) {
            request = request.basic_auth(user, Some(pass));
        } else if let Some(key) = &self.api_key {
            request = request.bearer_auth(key);
        }
        
        let response = request
            .send()
            .await
            .map_err(|e| SyncError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() && response.status() != 204 {
            return Err(SyncError::ProviderError(format!(
                "Upload failed: {}", response.status()
            )));
        }
        
        let etag = response
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.trim_matches('"').to_string())
            .unwrap_or_default();
        
        Ok(RemoteFile {
            id: url.clone(),
            name: remote_path.split('/').last().unwrap_or(remote_path).to_string(),
            path: url,
            size: data.len() as u64,
            modified: Utc::now(),
            created: Utc::now(),
            hash: etag,
            version: None,
        })
    }
    
    async fn download_file(&self, remote_id: &str) -> SyncResult<Vec<u8>> {
        let client = reqwest::Client::new();
        // remote_id is already the full URL for WebDAV
        let url = if remote_id.starts_with("http") {
            remote_id.to_string()
        } else {
            self.get_full_url(remote_id)
        };
        
        let mut request = client.get(&url);

        if let (Some(user), Some(pass)) = (&self.username, &self.password) {
            request = request.basic_auth(user, Some(pass));
        } else if let Some(key) = &self.api_key {
            request = request.bearer_auth(key);
        }
        
        let response = request
            .send()
            .await
            .map_err(|e| SyncError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(SyncError::ProviderError(format!(
                "Download failed: {}", response.status()
            )));
        }
        
        let bytes = response
            .bytes()
            .await
            .map_err(|e| SyncError::NetworkError(e.to_string()))?;
        
        Ok(bytes.to_vec())
    }
    
    async fn delete_file(&self, remote_id: &str) -> SyncResult<()> {
        let client = reqwest::Client::new();
        let url = if remote_id.starts_with("http") {
            remote_id.to_string()
        } else {
            self.get_full_url(remote_id)
        };
        
        let mut request = client.delete(&url);

        if let (Some(user), Some(pass)) = (&self.username, &self.password) {
            request = request.basic_auth(user, Some(pass));
        } else if let Some(key) = &self.api_key {
            request = request.bearer_auth(key);
        }
        
        let response = request
            .send()
            .await
            .map_err(|e| SyncError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() && response.status() != 204 {
            return Err(SyncError::ProviderError(format!(
                "Delete failed: {}", response.status()
            )));
        }
        
        Ok(())
    }
    
    async fn create_folder(&self, path: &str) -> SyncResult<RemoteFile> {
        self.mkcol(path).await?;
        
        Ok(RemoteFile {
            id: self.get_full_url(path),
            name: path.split('/').last().unwrap_or(path).to_string(),
            path: self.get_full_url(path),
            size: 0,
            modified: Utc::now(),
            created: Utc::now(),
            hash: String::new(),
            version: None,
        })
    }
    
    async fn get_storage_info(&self) -> SyncResult<StorageInfo> {
        // WebDAV doesn't provide storage quota info in standard implementations
        // Some servers like Nextcloud do provide it via custom extensions
        Ok(StorageInfo {
            used_bytes: 0,
            limit_bytes: None,
            plan_name: None,
        })
    }
    
    async fn subscribe_changes(&self) -> SyncResult<()> {
        // WebDAV doesn't have native push notifications
        // Would need to implement polling
        Ok(())
    }
    
    async fn get_changes(&self, _since: DateTime<Utc>) -> SyncResult<Vec<FileChange>> {
        // Would need to implement change tracking via ETags
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_provider_creation() {
        let provider = WebDAVProvider::new(
            "https://cloud.example.com".to_string(),
            "/webdav/VantisWeb".to_string(),
        );
        assert_eq!(provider.provider_type(), SyncProviderType::WebDAV);
        assert!(!provider.is_authenticated());
    }
    
    #[test]
    fn test_url_building() {
        let provider = WebDAVProvider::new(
            "https://cloud.example.com/".to_string(),
            "/webdav/VantisWeb/".to_string(),
        );
        
        let url = provider.get_full_url("profile.json");
        assert_eq!(url, "https://cloud.example.com/webdav/VantisWeb/profile.json");
    }
    
    #[test]
    fn test_nextcloud_preset() {
        let provider = WebDAVProvider::nextcloud(
            "https://nextcloud.example.com".to_string(),
            "user123".to_string(),
        );
        
        assert_eq!(provider.username, Some("user123".to_string()));
        assert!(provider.root_path.contains("remote.php/dav/files"));
    }
    
    #[test]
    fn test_xml_parsing() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
            <d:multistatus xmlns:d="DAV:">
                <d:response>
                    <d:href>/webdav/VantisWeb/</d:href>
                    <d:propstat>
                        <d:prop>
                            <d:displayname>VantisWeb</d:displayname>
                            <d:resourcetype><d:collection/></d:resourcetype>
                        </d:prop>
                        <d:status>HTTP/1.1 200 OK</d:status>
                    </d:propstat>
                </d:response>
                <d:response>
                    <d:href>/webdav/VantisWeb/profile.json</d:href>
                    <d:propstat>
                        <d:prop>
                            <d:displayname>profile.json</d:displayname>
                            <d:getcontentlength>1024</d:getcontentlength>
                            <d:getlastmodified>Mon, 01 Jan 2024 12:00:00 GMT</d:getlastmodified>
                            <d:getetag>"abc123"</d:getetag>
                        </d:prop>
                        <d:status>HTTP/1.1 200 OK</d:status>
                    </d:propstat>
                </d:response>
            </d:multistatus>"#;
        
        let provider = WebDAVProvider::new(
            "https://cloud.example.com".to_string(),
            "/webdav/VantisWeb".to_string(),
        );
        
        let files = provider.parse_propfind_response(xml, "/webdav/VantisWeb/");
        
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].name, "profile.json");
        assert_eq!(files[0].size, 1024);
        assert_eq!(files[0].hash, "abc123");
    }
}