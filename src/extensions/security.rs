// VantisWeb Browser - Advanced Extension System
// Security and Permission Management Module

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use url::Url;

/// Security manager for extensions
#[derive(Clone, Debug)]
pub struct SecurityManager {
    permission_validator: Arc<RwLock<PermissionValidator>>,
    threat_detector: Arc<RwLock<ThreatDetector>>,
    csp_manager: Arc<RwLock<ContentSecurityPolicyManager>>,
    audit_log: Arc<RwLock<Vec<SecurityAuditEntry>>>,
    security_policies: Arc<RwLock<HashMap<Uuid, ExtensionSecurityPolicy>>>,
    blocked_permissions: Arc<RwLock<HashSet<String>>>,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new() -> Self {
        Self {
            permission_validator: Arc::new(RwLock::new(PermissionValidator::new())),
            threat_detector: Arc::new(RwLock::new(ThreatDetector::new())),
            csp_manager: Arc::new(RwLock::new(ContentSecurityPolicyManager::new())),
            audit_log: Arc::new(RwLock::new(Vec::new())),
            security_policies: Arc::new(RwLock::new(HashMap::new())),
            blocked_permissions: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Validate extension permissions
    pub async fn validate_permissions(
        &self,
        extension_id: Uuid,
        manifest: &serde_json::Value,
    ) -> Result<PermissionValidationResult, SecurityError> {
        let mut validator = self.permission_validator.write().await;
        let result = validator.validate(manifest)?;
        
        // Log audit entry
        self.log_audit(
            extension_id,
            AuditAction::PermissionValidation,
            result.is_valid,
        ).await;
        
        Ok(result)
    }

    /// Check if permission is granted for extension
    pub async fn has_permission(
        &self,
        extension_id: Uuid,
        permission: &str,
    ) -> bool {
        let blocked = self.blocked_permissions.read().await;
        if blocked.contains(permission) {
            return false;
        }
        
        let policies = self.security_policies.read().await;
        if let Some(policy) = policies.get(&extension_id) {
            policy.granted_permissions.contains(permission)
        } else {
            false
        }
    }

    /// Grant permission to extension
    pub async fn grant_permission(
        &self,
        extension_id: Uuid,
        permission: &str,
    ) -> Result<(), SecurityError> {
        let mut policies = self.security_policies.write().await;
        let policy = policies.entry(extension_id)
            .or_insert_with(ExtensionSecurityPolicy::new);
        
        policy.granted_permissions.insert(permission.to_string());
        policy.permission_history.push(PermissionGrant {
            permission: permission.to_string(),
            granted_at: chrono::Utc::now().to_rfc3339(),
            source: GrantSource::UserApproval,
        });
        
        self.log_audit(extension_id, AuditAction::PermissionGrant, true).await;
        Ok(())
    }

    /// Revoke permission from extension
    pub async fn revoke_permission(
        &self,
        extension_id: Uuid,
        permission: &str,
    ) -> Result<(), SecurityError> {
        let mut policies = self.security_policies.write().await;
        if let Some(policy) = policies.get_mut(&extension_id) {
            policy.granted_permissions.remove(permission);
            self.log_audit(extension_id, AuditAction::PermissionRevoke, true).await;
        }
        Ok(())
    }

    /// Detect threats in extension code
    pub async fn detect_threats(
        &self,
        extension_id: Uuid,
        code: &str,
    ) -> Result<ThreatDetectionResult, SecurityError> {
        let mut detector = self.threat_detector.write().await;
        let result = detector.analyze(code)?;
        
        self.log_audit(
            extension_id,
            AuditAction::ThreatDetection,
            result.threats.is_empty(),
        ).await;
        
        Ok(result)
    }

    /// Generate CSP for extension
    pub async fn generate_csp(
        &self,
        extension_id: Uuid,
        manifest: &serde_json::Value,
    ) -> Result<String, SecurityError> {
        let manager = self.csp_manager.read().await;
        let csp = manager.generate(manifest)?;
        
        self.log_audit(extension_id, AuditAction::CSPGeneration, true).await;
        Ok(csp)
    }

    /// Validate URL access
    pub async fn validate_url_access(
        &self,
        extension_id: Uuid,
        url: &str,
        access_type: UrlAccessType,
    ) -> Result<bool, SecurityError> {
        let policies = self.security_policies.read().await;
        
        if let Some(policy) = policies.get(&extension_id) {
            let parsed_url = Url::parse(url)?;
            
            // Check host permissions
            for host_permission in &policy.host_permissions {
                if Self::url_matches_permission(&parsed_url, host_permission) {
                    self.log_audit(
                        extension_id,
                        AuditAction::UrlAccess,
                        true,
                    ).await;
                    return Ok(true);
                }
            }
            
            // Check if URL is blocked
            for blocked in &policy.blocked_urls {
                if Self::url_matches_permission(&parsed_url, blocked) {
                    self.log_audit(
                        extension_id,
                        AuditAction::UrlAccess,
                        false,
                    ).await;
                    return Ok(false);
                }
            }
        }
        
        Ok(false)
    }

    /// Check if URL matches permission pattern
    fn url_matches_permission(url: &Url, pattern: &str) -> bool {
        if pattern == "<all_urls>" {
            return true;
        }
        
        // Handle wildcard patterns
        if pattern.contains('*') {
            let host = url.host_str().unwrap_or("");
            let pattern_host = pattern
                .replace("*://", "")
                .replace("/*", "");
            
            if host == pattern_host || host.ends_with(&format!(".{}", pattern_host)) {
                return true;
            }
        }
        
        url.as_str().starts_with(pattern) || pattern.contains(url.host_str().unwrap_or(""))
    }

    /// Log security audit entry
    async fn log_audit(&self, extension_id: Uuid, action: AuditAction, success: bool) {
        let mut log = self.audit_log.write().await;
        log.push(SecurityAuditEntry {
            id: Uuid::new_v4(),
            extension_id,
            action,
            success,
            timestamp: chrono::Utc::now().to_rfc3339(),
            details: None,
        });
        
        // Keep log size manageable
        if log.len() > 10000 {
            log.remove(0);
        }
    }

    /// Get audit log for extension
    pub async fn get_audit_log(&self, extension_id: Uuid) -> Vec<SecurityAuditEntry> {
        let log = self.audit_log.read().await;
        log.iter()
            .filter(|entry| entry.extension_id == extension_id)
            .cloned()
            .collect()
    }

    /// Block a permission globally
    pub async fn block_permission(&self, permission: &str) {
        let mut blocked = self.blocked_permissions.write().await;
        blocked.insert(permission.to_string());
    }

    /// Unblock a permission
    pub async fn unblock_permission(&self, permission: &str) {
        let mut blocked = self.blocked_permissions.write().await;
        blocked.remove(permission);
    }

    /// Get security policy for extension
    pub async fn get_security_policy(&self, extension_id: Uuid) -> Option<ExtensionSecurityPolicy> {
        let policies = self.security_policies.read().await;
        policies.get(&extension_id).cloned()
    }

    /// Set security policy for extension
    pub async fn set_security_policy(
        &self,
        extension_id: Uuid,
        policy: ExtensionSecurityPolicy,
    ) {
        let mut policies = self.security_policies.write().await;
        policies.insert(extension_id, policy);
    }

    /// Check if extension is trusted
    pub async fn is_trusted(&self, extension_id: Uuid) -> bool {
        let policies = self.security_policies.read().await;
        policies.get(&extension_id)
            .map(|p| p.trust_level >= TrustLevel::Trusted)
            .unwrap_or(false)
    }

    /// Set extension trust level
    pub async fn set_trust_level(&self, extension_id: Uuid, level: TrustLevel) {
        let mut policies = self.security_policies.write().await;
        let policy = policies.entry(extension_id)
            .or_insert_with(ExtensionSecurityPolicy::new);
        policy.trust_level = level;
    }
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Permission validator
pub struct PermissionValidator {
    known_permissions: HashSet<String>,
    dangerous_permissions: HashSet<String>,
}

impl PermissionValidator {
    fn new() -> Self {
        let known_permissions = HashSet::from([
            "activeTab",
            "alarms",
            "background",
            "bookmarks",
            "browsingData",
            "clipboardRead",
            "clipboardWrite",
            "contentSettings",
            "contextMenus",
            "cookies",
            "debugger",
            "declarativeContent",
            "declarativeNetRequest",
            "declarativeNetRequestFeedback",
            "declarativeNetRequestWithHostAccess",
            "desktopCapture",
            "devtools",
            "dns",
            "documentScan",
            "downloads",
            "downloads.open",
            "downloads.ui",
            "enterprise.deviceAttributes",
            "enterprise.hardwarePlatform",
            "enterprise.networkingConfig",
            "enterprise.platformKeys",
            "experimental",
            "favicon",
            "fileBrowserHandler",
            "fileSystemProvider",
            "fontSettings",
            "gcm",
            "geolocation",
            "history",
            "identity",
            "identity.email",
            "idle",
            "idltest",
            "management",
            "mdns",
            "mediaGalleries",
            "metricsPrivate",
            "networking.config",
            "normandyPrivate",
            "notifications",
            "offscreen",
            "omnibox",
            "pageCapture",
            "passwordsPrivate",
            "pdfViewerPrivate",
            "permissions",
            "privacy",
            "printerProvider",
            "printing",
            "printingMetrics",
            "proxy",
            "readingList",
            "runtime",
            "scripting",
            "search",
            "searchProvider",
            "sessions",
            "signedInDevices",
            "storage",
            "sync",
            "system.cpu",
            "system.display",
            "system.memory",
            "system.storage",
            "tabCapture",
            "tabGroups",
            "tabs",
            "tabs.videoConverter",
            "topSites",
            "tts",
            "ttsEngine",
            "unlimitedStorage",
            "userScripts",
            "vpnProvider",
            "webNavigation",
            "webRequest",
            "webRequestAuthProvider",
        ]);

        let dangerous_permissions = HashSet::from([
            "debugger",
            "declarativeNetRequest",
            "downloads",
            "downloads.open",
            "experimental",
            "geolocation",
            "history",
            "management",
            "proxy",
            "tabs",
            "webRequest",
            "clipboardRead",
            "clipboardWrite",
            "background",
        ]);

        Self {
            known_permissions,
            dangerous_permissions,
        }
    }

    fn validate(&mut self, manifest: &serde_json::Value) -> Result<PermissionValidationResult, SecurityError> {
        let mut result = PermissionValidationResult {
            is_valid: true,
            unknown_permissions: Vec::new(),
            dangerous_permissions: Vec::new(),
            warnings: Vec::new(),
            required_consents: Vec::new(),
        };

        if let Some(permissions) = manifest.get("permissions").and_then(|v| v.as_array()) {
            for perm in permissions {
                if let Some(perm_str) = perm.as_str() {
                    if !self.known_permissions.contains(perm_str) {
                        result.unknown_permissions.push(perm_str.to_string());
                        result.warnings.push(format!("Unknown permission: {}", perm_str));
                    }
                    
                    if self.dangerous_permissions.contains(perm_str) {
                        result.dangerous_permissions.push(perm_str.to_string());
                        result.required_consents.push(format!(
                            "Extension requests access to: {}. This is a sensitive permission.",
                            perm_str
                        ));
                    }
                }
            }
        }

        // Check host permissions
        if let Some(host_permissions) = manifest.get("host_permissions").and_then(|v| v.as_array()) {
            for host in host_permissions {
                if let Some(host_str) = host.as_str() {
                    if host_str == "<all_urls>" {
                        result.warnings.push("Extension requests access to all websites".to_string());
                        result.required_consents.push(
                            "This extension can access all your data on all websites".to_string()
                        );
                    }
                }
            }
        }

        Ok(result)
    }
}

/// Threat detector for extension code
pub struct ThreatDetector {
    patterns: Vec<ThreatPattern>,
}

impl ThreatDetector {
    fn new() -> Self {
        let patterns = vec![
            ThreatPattern {
                id: "eval_usage",
                name: "Dynamic code execution",
                pattern: r"eval\s*\(",
                severity: ThreatSeverity::High,
                description: "Use of eval() can lead to code injection vulnerabilities",
            },
            ThreatPattern {
                id: "innerHTML_usage",
                name: "Unsafe HTML injection",
                pattern: r"\.innerHTML\s*=",
                severity: ThreatSeverity::Medium,
                description: "Direct innerHTML assignment can lead to XSS",
            },
            ThreatPattern {
                id: "document_write",
                name: "Document write",
                pattern: r"document\.write\s*\(",
                severity: ThreatSeverity::Medium,
                description: "document.write can lead to XSS and DOM manipulation",
            },
            ThreatPattern {
                id: "postmessage_wildcard",
                name: "Unsafe postMessage",
                pattern: r"postMessage\s*\([^,]*,\s*['&quot;]\*['&quot;]",
                severity: ThreatSeverity::High,
                description: "postMessage with wildcard origin is insecure",
            },
            ThreatPattern {
                id: "localStorage_sensitive",
                name: "Sensitive data in localStorage",
                pattern: r"localStorage\.(setItem|getItem)\s*\(\s*['&quot;](password|token|secret|key)",
                severity: ThreatSeverity::High,
                description: "Storing sensitive data in localStorage is insecure",
            },
            ThreatPattern {
                id: "xhr_credential",
                name: "XHR with credentials",
                pattern: r"withCredentials\s*=\s*true",
                severity: ThreatSeverity::Medium,
                description: "XHR with credentials enabled may expose cookies",
            },
            ThreatPattern {
                id: "external_resource",
                name: "External resource loading",
                pattern: r"(src|href)\s*=\s*['&quot;]https?://(?!chrom-extension|vantis-extension)",
                severity: ThreatSeverity::Low,
                description: "Loading external resources may leak data",
            },
        ];

        Self { patterns }
    }

    fn analyze(&mut self, code: &str) -> Result<ThreatDetectionResult, SecurityError> {
        let mut threats = Vec::new();
        let mut risk_score = 0;

        for pattern in &self.patterns {
            if let Ok(re) = regex::Regex::new(&pattern.pattern) {
                if re.is_match(code) {
                    threats.push(DetectedThreat {
                        pattern_id: pattern.id.to_string(),
                        name: pattern.name.to_string(),
                        severity: pattern.severity,
                        description: pattern.description.to_string(),
                        matched_code: Self::extract_match(code, &re),
                    });

                    risk_score += pattern.severity.score();
                }
            }
        }

        Ok(ThreatDetectionResult {
            threats,
            risk_score,
            is_malicious: risk_score >= 100,
        })
    }

    fn extract_match(code: &str, re: &regex::Regex) -> String {
        if let Some(m) = re.find(code) {
            let start = m.start().saturating_sub(20);
            let end = (m.end() + 20).min(code.len());
            code[start..end].to_string()
        } else {
            String::new()
        }
    }
}

/// Content Security Policy Manager
pub struct ContentSecurityPolicyManager {
    default_directives: HashMap<String, Vec<String>>,
}

impl ContentSecurityPolicyManager {
    fn new() -> Self {
        let mut default_directives = HashMap::new();
        default_directives.insert(
            "default-src".to_string(),
            vec!["'self'".to_string()],
        );
        default_directives.insert(
            "script-src".to_string(),
            vec!["'self'".to_string()],
        );
        default_directives.insert(
            "style-src".to_string(),
            vec!["'self'".to_string(), "'unsafe-inline'".to_string()],
        );
        default_directives.insert(
            "img-src".to_string(),
            vec!["'self'".to_string(), "data:".to_string(), "https:".to_string()],
        );
        default_directives.insert(
            "connect-src".to_string(),
            vec!["'self'".to_string()],
        );
        default_directives.insert(
            "font-src".to_string(),
            vec!["'self'".to_string()],
        );
        default_directives.insert(
            "object-src".to_string(),
            vec!["'none'".to_string()],
        );
        default_directives.insert(
            "media-src".to_string(),
            vec!["'self'".to_string()],
        );

        Self { default_directives }
    }

    fn generate(&self, manifest: &serde_json::Value) -> Result<String, SecurityError> {
        let mut directives = self.default_directives.clone();

        // Check for CSP in manifest
        if let Some(manifest_csp) = manifest
            .get("content_security_policy")
            .and_then(|v| v.get("extension_pages"))
            .and_then(|v| v.as_str())
        {
            // Parse and merge with default CSP
            Self::parse_and_merge_csp(manifest_csp, &mut directives);
        }

        // Generate CSP string
        let csp_parts: Vec<String> = directives
            .iter()
            .map(|(directive, sources)| format!("{} {}", directive, sources.join(" ")))
            .collect();

        Ok(csp_parts.join("; "))
    }

    fn parse_and_merge_csp(csp: &str, directives: &mut HashMap<String, Vec<String>>) {
        for part in csp.split(';') {
            let part = part.trim();
            if let Some(space_pos) = part.find(' ') {
                let directive = &part[..space_pos];
                let sources: Vec<String> = part[space_pos + 1..]
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();
                
                directives.insert(directive.to_string(), sources);
            }
        }
    }
}

/// Security error types
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Threat detected: {0}")]
    ThreatDetected(String),
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
    #[error("CSP violation: {0}")]
    CspViolation(String),
    #[error("Invalid permission: {0}")]
    InvalidPermission(String),
    #[error("Security policy violation: {0}")]
    PolicyViolation(String),
    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),
}

/// Permission validation result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PermissionValidationResult {
    pub is_valid: bool,
    pub unknown_permissions: Vec<String>,
    pub dangerous_permissions: Vec<String>,
    pub warnings: Vec<String>,
    pub required_consents: Vec<String>,
}

/// Threat detection result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThreatDetectionResult {
    pub threats: Vec<DetectedThreat>,
    pub risk_score: u32,
    pub is_malicious: bool,
}

/// Detected threat
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DetectedThreat {
    pub pattern_id: String,
    pub name: String,
    pub severity: ThreatSeverity,
    pub description: String,
    pub matched_code: String,
}

/// Threat severity levels
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreatSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl ThreatSeverity {
    pub fn score(&self) -> u32 {
        match self {
            Self::Low => 10,
            Self::Medium => 25,
            Self::High => 50,
            Self::Critical => 100,
        }
    }
}

/// Threat pattern
pub struct ThreatPattern {
    pub id: &'static str,
    pub name: &'static str,
    pub pattern: &'static str,
    pub severity: ThreatSeverity,
    pub description: &'static str,
}

/// Extension security policy
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ExtensionSecurityPolicy {
    pub granted_permissions: HashSet<String>,
    pub host_permissions: Vec<String>,
    pub blocked_urls: Vec<String>,
    pub trust_level: TrustLevel,
    pub permission_history: Vec<PermissionGrant>,
    pub last_audit: Option<String>,
}

impl ExtensionSecurityPolicy {
    fn new() -> Self {
        Self {
            granted_permissions: HashSet::new(),
            host_permissions: Vec::new(),
            blocked_urls: Vec::new(),
            trust_level: TrustLevel::Untrusted,
            permission_history: Vec::new(),
            last_audit: None,
        }
    }
}

/// Trust levels for extensions
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TrustLevel {
    #[default]
    Untrusted,
    Low,
    Medium,
    Trusted,
    Verified,
}

/// Permission grant record
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PermissionGrant {
    pub permission: String,
    pub granted_at: String,
    pub source: GrantSource,
}

/// Source of permission grant
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GrantSource {
    Manifest,
    UserApproval,
    AdminPolicy,
    Temporary,
}

/// URL access types
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UrlAccessType {
    Read,
    Write,
    Modify,
    Full,
}

/// Audit action types
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditAction {
    PermissionValidation,
    PermissionGrant,
    PermissionRevoke,
    ThreatDetection,
    UrlAccess,
    CSPGeneration,
    SecurityAlert,
}

/// Security audit entry
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SecurityAuditEntry {
    pub id: Uuid,
    pub extension_id: Uuid,
    pub action: AuditAction,
    pub success: bool,
    pub timestamp: String,
    pub details: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_security_manager_creation() {
        let manager = SecurityManager::new();
        assert!(manager.get_security_policy(Uuid::new_v4()).await.is_none());
    }

    #[tokio::test]
    async fn test_permission_validation() {
        let manager = SecurityManager::new();
        let extension_id = Uuid::new_v4();
        
        let manifest = json!({
            "name": "Test Extension",
            "version": "1.0",
            "permissions": ["tabs", "storage", "bookmarks"]
        });
        
        let result = manager.validate_permissions(extension_id, &manifest).await;
        assert!(result.is_ok());
        
        let validation = result.unwrap();
        assert!(validation.is_valid);
        assert!(validation.dangerous_permissions.contains(&"tabs".to_string()));
    }

    #[tokio::test]
    async fn test_permission_grant_revoke() {
        let manager = SecurityManager::new();
        let extension_id = Uuid::new_v4();
        
        manager.grant_permission(extension_id, "tabs").await.unwrap();
        assert!(manager.has_permission(extension_id, "tabs").await);
        
        manager.revoke_permission(extension_id, "tabs").await.unwrap();
        assert!(!manager.has_permission(extension_id, "tabs").await);
    }

    #[tokio::test]
    async fn test_threat_detection() {
        let manager = SecurityManager::new();
        let extension_id = Uuid::new_v4();
        
        let malicious_code = r##"
            function malicious() {
                eval("alert('xss')");
                document.innerHTML = "<script>steal()</script>";
            }
        "##;
        
        let result = manager.detect_threats(extension_id, malicious_code).await;
        assert!(result.is_ok());
        
        let detection = result.unwrap();
        assert!(!detection.threats.is_empty());
    }

    #[tokio::test]
    async fn test_csp_generation() {
        let manager = SecurityManager::new();
        
        let manifest = json!({
            "name": "Test",
            "version": "1.0"
        });
        
        let csp = manager.generate_csp(Uuid::new_v4(), &manifest).await;
        assert!(csp.is_ok());
        
        let csp_str = csp.unwrap();
        assert!(csp_str.contains("default-src"));
        assert!(csp_str.contains("script-src"));
    }

    #[tokio::test]
    async fn test_url_access_validation() {
        let manager = SecurityManager::new();
        let extension_id = Uuid::new_v4();
        
        let result = manager.validate_url_access(
            extension_id,
            "https://example.com",
            UrlAccessType::Read,
        ).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_trust_levels() {
        let manager = SecurityManager::new();
        let extension_id = Uuid::new_v4();
        
        assert!(!manager.is_trusted(extension_id).await);
        
        manager.set_trust_level(extension_id, TrustLevel::Trusted).await;
        assert!(manager.is_trusted(extension_id).await);
    }

    #[tokio::test]
    async fn test_audit_log() {
        let manager = SecurityManager::new();
        let extension_id = Uuid::new_v4();
        
        manager.grant_permission(extension_id, "tabs").await.unwrap();
        
        let log = manager.get_audit_log(extension_id).await;
        assert!(!log.is_empty());
    }

    #[tokio::test]
    async fn test_blocked_permissions() {
        let manager = SecurityManager::new();
        manager.block_permission("tabs").await;
        
        let extension_id = Uuid::new_v4();
        
        // Should not be able to grant blocked permission
        manager.grant_permission(extension_id, "tabs").await.unwrap();
        assert!(!manager.has_permission(extension_id, "tabs").await);
    }
}