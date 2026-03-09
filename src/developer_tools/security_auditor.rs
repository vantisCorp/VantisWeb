//! Security Auditor for vulnerability detection
//! 
//! Provides comprehensive security analysis including:
//! - Content Security Policy validation
//! - Mixed content detection
//! - Certificate analysis
//! - XSS vulnerability scanning
//! - CORS misconfiguration detection

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use url::Url;

/// Security issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Severity {
    /// Informational
    Info,
    /// Low risk
    Low,
    /// Medium risk
    Medium,
    /// High risk
    High,
    /// Critical risk
    Critical,
}

impl Default for Severity {
    fn default() -> Self {
        Severity::Info
    }
}

/// Security issue category
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecurityCategory {
    /// Content Security Policy issues
    CSP,
    /// Mixed content (HTTP/HTTPS)
    MixedContent,
    /// TLS/Certificate issues
    Certificate,
    /// Cross-Origin issues
    CORS,
    /// Cross-Site Scripting
    XSS,
    /// Cross-Site Request Forgery
    CSRF,
    /// Information disclosure
    InfoDisclosure,
    /// Insecure practices
    InsecurePractices,
    /// Authentication issues
    Authentication,
    /// Authorization issues
    Authorization,
    /// Data validation
    DataValidation,
    /// Cookie security
    CookieSecurity,
    /// Header security
    HeaderSecurity,
}

/// Security issue detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIssue {
    /// Unique issue ID
    pub id: String,
    /// Issue title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Severity level
    pub severity: Severity,
    /// Category
    pub category: SecurityCategory,
    /// URL where issue was found
    pub url: Option<String>,
    /// Line number if applicable
    pub line: Option<usize>,
    /// Column number if applicable
    pub column: Option<usize>,
    /// Evidence/snippet showing the issue
    pub evidence: Option<String>,
    /// Recommendation for fixing
    pub recommendation: String,
    /// References for more info
    pub references: Vec<String>,
    /// CWE ID if applicable
    pub cwe_id: Option<String>,
    /// OWASP category if applicable
    pub owasp_category: Option<String>,
}

/// CSP Directive status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CSPDirective {
    /// Directive name
    pub name: String,
    /// Directive values
    pub values: Vec<String>,
    /// Whether it's using unsafe-* 
    pub has_unsafe: bool,
    /// Whether it allows data: URIs
    pub allows_data_uri: bool,
    /// Whether it allows blob: URIs
    pub allows_blob_uri: bool,
    /// Whether it has 'none'
    pub is_none: bool,
    /// Whether it has '*'
    pub has_wildcard: bool,
}

/// Content Security Policy analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CSPAnalysis {
    /// Whether CSP is present
    pub has_csp: bool,
    /// Whether CSP is enforced (not just report-only)
    pub is_enforced: bool,
    /// Parsed directives
    pub directives: HashMap<String, CSPDirective>,
    /// Issues found in CSP
    pub issues: Vec<SecurityIssue>,
    /// Overall CSP score (0-100)
    pub score: u8,
}

/// Certificate information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateInfo {
    /// Subject name
    pub subject: String,
    /// Issuer name
    pub issuer: String,
    /// Serial number
    pub serial_number: String,
    /// Valid from (ISO 8601)
    pub valid_from: String,
    /// Valid until (ISO 8601)
    pub valid_until: String,
    /// Whether certificate is valid
    pub is_valid: bool,
    /// Whether certificate is expired
    pub is_expired: bool,
    /// Whether certificate is self-signed
    pub is_self_signed: bool,
    /// Whether certificate uses SHA-1
    pub uses_sha1: bool,
    /// Signature algorithm
    pub signature_algorithm: String,
    /// Key size in bits
    pub key_size: u32,
    /// SAN (Subject Alternative Names)
    pub san: Vec<String>,
}

/// Mixed content item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixedContent {
    /// Resource URL
    pub url: String,
    /// Type of resource
    pub resource_type: String,
    /// Whether it's blockable
    pub is_blockable: bool,
    /// Line in HTML
    pub line: Option<usize>,
    /// Source HTML snippet
    pub source: Option<String>,
}

/// CORS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CORSConfig {
    /// Access-Control-Allow-Origin
    pub allow_origin: Option<String>,
    /// Access-Control-Allow-Methods
    pub allow_methods: Vec<String>,
    /// Access-Control-Allow-Headers
    pub allow_headers: Vec<String>,
    /// Access-Control-Allow-Credentials
    pub allow_credentials: bool,
    /// Access-Control-Max-Age
    pub max_age: Option<u32>,
    /// Issues with CORS config
    pub issues: Vec<SecurityIssue>,
}

/// Cookie security info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieSecurity {
    /// Cookie name
    pub name: String,
    /// Whether HttpOnly is set
    pub http_only: bool,
    /// Whether Secure flag is set
    pub secure: bool,
    /// SameSite attribute
    pub same_site: Option<String>,
    /// Domain
    pub domain: Option<String>,
    /// Path
    pub path: Option<String>,
    /// Issues with this cookie
    pub issues: Vec<SecurityIssue>,
}

/// Security audit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAuditorConfig {
    /// Enable CSP analysis
    pub analyze_csp: bool,
    /// Enable mixed content detection
    pub detect_mixed_content: bool,
    /// Enable certificate checking
    pub check_certificates: bool,
    /// Enable XSS scanning
    pub scan_xss: bool,
    /// Enable CORS analysis
    pub analyze_cors: bool,
    /// Enable cookie analysis
    pub analyze_cookies: bool,
    /// Minimum severity to report
    pub min_severity: Severity,
    /// Maximum issues to collect per category
    pub max_issues_per_category: usize,
}

impl Default for SecurityAuditorConfig {
    fn default() -> Self {
        Self {
            analyze_csp: true,
            detect_mixed_content: true,
            check_certificates: true,
            scan_xss: true,
            analyze_cors: true,
            analyze_cookies: true,
            min_severity: Severity::Info,
            max_issues_per_category: 100,
        }
    }
}

/// Security Auditor
pub struct SecurityAuditor {
    /// Configuration
    config: SecurityAuditorConfig,
    /// Detected issues
    issues: Arc<RwLock<Vec<SecurityIssue>>>,
    /// CSP analysis cache
    csp_cache: Arc<RwLock<Option<CSPAnalysis>>>,
    /// Certificate info cache
    cert_cache: Arc<RwLock<Option<CertificateInfo>>>,
}

impl SecurityAuditor {
    /// Create a new security auditor
    pub fn new(config: SecurityAuditorConfig) -> Self {
        Self {
            config,
            issues: Arc::new(RwLock::new(Vec::new())),
            csp_cache: Arc::new(RwLock::new(None)),
            cert_cache: Arc::new(RwLock::new(None)),
        }
    }

    /// Run full security audit
    pub async fn run_audit(&self, url: &str, html: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();

        // Analyze CSP
        if self.config.analyze_csp {
            let csp_issues = self.analyze_csp(html).await;
            issues.extend(csp_issues);
        }

        // Detect mixed content
        if self.config.detect_mixed_content {
            let mixed_issues = self.detect_mixed_content(url, html).await;
            issues.extend(mixed_issues);
        }

        // XSS scanning
        if self.config.scan_xss {
            let xss_issues = self.scan_xss(html).await;
            issues.extend(xss_issues);
        }

        // Filter by minimum severity
        issues.retain(|issue| issue.severity >= self.config.min_severity);

        // Store issues
        let mut stored = self.issues.write().await;
        stored.extend(issues.clone());

        issues
    }

    /// Analyze Content Security Policy
    pub async fn analyze_csp(&self, html: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();
        
        // Check for CSP meta tag
        let has_csp_meta = html.contains("Content-Security-Policy") || 
                          html.contains("content-security-policy");
        
        // In real implementation, would parse and validate CSP directives
        if !has_csp_meta {
            issues.push(SecurityIssue {
                id: "CSP-001".to_string(),
                title: "Missing Content Security Policy".to_string(),
                description: "No Content Security Policy header or meta tag found. CSP helps prevent XSS attacks.".to_string(),
                severity: Severity::Medium,
                category: SecurityCategory::CSP,
                url: None,
                line: None,
                column: None,
                evidence: None,
                recommendation: "Add a Content-Security-Policy header with appropriate directives.".to_string(),
                references: vec![
                    "https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP".to_string()
                ],
                cwe_id: Some("CWE-1021".to_string()),
                owasp_category: Some("A05:2021 - Security Misconfiguration".to_string()),
            });
        }

        // Check for unsafe-inline
        if html.contains("'unsafe-inline'") {
            issues.push(SecurityIssue {
                id: "CSP-002".to_string(),
                title: "CSP allows unsafe-inline scripts".to_string(),
                description: "The Content Security Policy contains 'unsafe-inline' which significantly weakens XSS protection.".to_string(),
                severity: Severity::High,
                category: SecurityCategory::CSP,
                url: None,
                line: None,
                column: None,
                evidence: Some("'unsafe-inline' found in CSP".to_string()),
                recommendation: "Use nonces or hashes instead of 'unsafe-inline' for script execution.".to_string(),
                references: vec![
                    "https://content-security-policy.com/".to_string()
                ],
                cwe_id: Some("CWE-1021".to_string()),
                owasp_category: Some("A05:2021 - Security Misconfiguration".to_string()),
            });
        }

        // Check for unsafe-eval
        if html.contains("'unsafe-eval'") {
            issues.push(SecurityIssue {
                id: "CSP-003".to_string(),
                title: "CSP allows unsafe-eval".to_string(),
                description: "The Content Security Policy contains 'unsafe-eval' which allows code injection via eval().".to_string(),
                severity: Severity::Medium,
                category: SecurityCategory::CSP,
                url: None,
                line: None,
                column: None,
                evidence: Some("'unsafe-eval' found in CSP".to_string()),
                recommendation: "Remove 'unsafe-eval' and refactor code to avoid dynamic code evaluation.".to_string(),
                references: vec![],
                cwe_id: None,
                owasp_category: Some("A05:2021 - Security Misconfiguration".to_string()),
            });
        }

        issues
    }

    /// Detect mixed content on HTTPS pages
    pub async fn detect_mixed_content(&self, page_url: &str, html: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();
        
        // Check if page is HTTPS
        let is_https = page_url.starts_with("https://");
        if !is_https {
            return issues;
        }

        // Find HTTP resources
        let patterns = [
            ("src=&quot;http://", "script"),
            ("href=&quot;http://", "link"),
            ("action=&quot;http://", "form"),
            ("poster=&quot;http://", "video"),
            ("data=&quot;http://", "object"),
        ];

        for (pattern, resource_type) in patterns {
            if let Some(pos) = html.find(pattern) {
                // Extract the URL
                let start = pos + pattern.len() - 7; // Position of http://
                if let Some(end) = html[start..].find('"') {
                    let url = &html[start..start + end];
                    
                    issues.push(SecurityIssue {
                        id: format!("MIXED-{}", resource_type.to_uppercase()),
                        title: format!("Mixed content: {} loaded over HTTP", resource_type),
                        description: format!("Resource {} is loaded over HTTP on an HTTPS page.", url),
                        severity: if resource_type == "script" { Severity::High } else { Severity::Medium },
                        category: SecurityCategory::MixedContent,
                        url: Some(url.to_string()),
                        line: None,
                        column: None,
                        evidence: Some(format!("{}=&quot;{}&quot;", &pattern[..pattern.len()-8], url)),
                        recommendation: "Load all resources over HTTPS or use protocol-relative URLs.".to_string(),
                        references: vec![
                            "https://developer.mozilla.org/en-US/docs/Web/Security/Mixed_content".to_string()
                        ],
                        cwe_id: Some("CWE-319".to_string()),
                        owasp_category: Some("A02:2021 - Cryptographic Failures".to_string()),
                    });
                }
            }
        }

        issues
    }

    /// Scan for potential XSS vulnerabilities
    pub async fn scan_xss(&self, html: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();

        // Check for inline event handlers
        let event_handlers = [
            "onclick", "onerror", "onload", "onmouseover", "onfocus",
            "onblur", "onsubmit", "onkeydown", "onkeyup", "onchange",
        ];

        for handler in event_handlers {
            let pattern = format!("{}=", handler);
            if html.contains(&pattern) {
                // Only flag if it contains user input pattern
                issues.push(SecurityIssue {
                    id: format!("XSS-{}", handler.to_uppercase()),
                    title: format!("Inline event handler: {}", handler),
                    description: format!("Inline event handler '{}' detected. If user input is used here, it could lead to XSS.", handler),
                    severity: Severity::Low,
                    category: SecurityCategory::XSS,
                    url: None,
                    line: None,
                    column: None,
                    evidence: Some(format!("{}=&quot;...&quot;", handler)),
                    recommendation: "Use addEventListener() instead of inline event handlers.".to_string(),
                    references: vec![
                        "https://owasp.org/www-community/attacks/xss/".to_string()
                    ],
                    cwe_id: Some("CWE-79".to_string()),
                    owasp_category: Some("A03:2021 - Injection".to_string()),
                });
            }
        }

        // Check for javascript: URLs
        if html.contains("javascript:") {
            issues.push(SecurityIssue {
                id: "XSS-JAVASCRIPT".to_string(),
                title: "JavaScript URL found".to_string(),
                description: "A javascript: URL was found which could be an XSS vector if user-controlled.".to_string(),
                severity: Severity::Medium,
                category: SecurityCategory::XSS,
                url: None,
                line: None,
                column: None,
                evidence: Some("javascript: URL".to_string()),
                recommendation: "Avoid javascript: URLs. Use event handlers or proper navigation instead.".to_string(),
                references: vec![],
                cwe_id: Some("CWE-79".to_string()),
                owasp_category: Some("A03:2021 - Injection".to_string()),
            });
        }

        // Check for innerHTML usage
        if html.contains("innerHTML") {
            issues.push(SecurityIssue {
                id: "XSS-INNERHTML".to_string(),
                title: "innerHTML usage detected".to_string(),
                description: "innerHTML was found. If used with user input, it can lead to XSS.".to_string(),
                severity: Severity::Medium,
                category: SecurityCategory::XSS,
                url: None,
                line: None,
                column: None,
                evidence: Some("innerHTML".to_string()),
                recommendation: "Use textContent instead of innerHTML when displaying user data.".to_string(),
                references: vec![],
                cwe_id: Some("CWE-79".to_string()),
                owasp_category: Some("A03:2021 - Injection".to_string()),
            });
        }

        issues
    }

    /// Analyze cookie security
    pub async fn analyze_cookies(&self, cookies: &[(&str, &str, bool, bool)]) -> Vec<CookieSecurity> {
        let mut results = Vec::new();

        for (name, _value, http_only, secure) in cookies {
            let mut cookie_issues = Vec::new();

            if !*secure {
                cookie_issues.push(SecurityIssue {
                    id: "COOKIE-001".to_string(),
                    title: "Cookie without Secure flag".to_string(),
                    description: format!("Cookie '{}' is transmitted over unencrypted connections.", name),
                    severity: Severity::Medium,
                    category: SecurityCategory::CookieSecurity,
                    url: None,
                    line: None,
                    column: None,
                    evidence: None,
                    recommendation: "Set the Secure flag on all sensitive cookies.".to_string(),
                    references: vec![],
                    cwe_id: Some("CWE-614".to_string()),
                    owasp_category: Some("A05:2021 - Security Misconfiguration".to_string()),
                });
            }

            if !*http_only {
                cookie_issues.push(SecurityIssue {
                    id: "COOKIE-002".to_string(),
                    title: "Cookie without HttpOnly flag".to_string(),
                    description: format!("Cookie '{}' is accessible to JavaScript, making it vulnerable to XSS theft.", name),
                    severity: Severity::Medium,
                    category: SecurityCategory::CookieSecurity,
                    url: None,
                    line: None,
                    column: None,
                    evidence: None,
                    recommendation: "Set the HttpOnly flag on session cookies.".to_string(),
                    references: vec![],
                    cwe_id: Some("CWE-1004".to_string()),
                    owasp_category: Some("A05:2021 - Security Misconfiguration".to_string()),
                });
            }

            results.push(CookieSecurity {
                name: name.to_string(),
                http_only: *http_only,
                secure: *secure,
                same_site: None,
                domain: None,
                path: None,
                issues: cookie_issues,
            });
        }

        results
    }

    /// Get all issues
    pub async fn get_issues(&self) -> Vec<SecurityIssue> {
        let issues = self.issues.read().await;
        issues.clone()
    }

    /// Clear all issues
    pub async fn clear_issues(&self) {
        let mut issues = self.issues.write().await;
        issues.clear();
    }

    /// Get security score (0-100)
    pub async fn get_security_score(&self) -> u8 {
        let issues = self.issues.read().await;
        
        let mut score: i32 = 100;
        
        for issue in issues.iter() {
            let deduction = match issue.severity {
                Severity::Critical => 25,
                Severity::High => 15,
                Severity::Medium => 8,
                Severity::Low => 3,
                Severity::Info => 0,
            };
            score = (score - deduction).max(0);
        }
        
        score as u8
    }
}

impl PartialEq for Severity {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Severity::Info, Severity::Info)
                | (Severity::Low, Severity::Low)
                | (Severity::Medium, Severity::Medium)
                | (Severity::High, Severity::High)
                | (Severity::Critical, Severity::Critical)
        )
    }
}

impl PartialOrd for Severity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let order = |s: &Severity| match s {
            Severity::Info => 0,
            Severity::Low => 1,
            Severity::Medium => 2,
            Severity::High => 3,
            Severity::Critical => 4,
        };
        order(self).partial_cmp(&order(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_no_csp_detection() {
        let auditor = SecurityAuditor::new(SecurityAuditorConfig::default());
        let html = "<html><body>Hello</body></html>";
        let issues = auditor.analyze_csp(html).await;
        
        assert!(!issues.is_empty());
        assert_eq!(issues[0].id, "CSP-001");
    }

    #[tokio::test]
    async fn test_unsafe_inline_detection() {
        let auditor = SecurityAuditor::new(SecurityAuditorConfig::default());
        let html = r##"<meta http-equiv="Content-Security-Policy" content="script-src 'unsafe-inline'">"##;
        let issues = auditor.analyze_csp(html).await;
        
        assert!(issues.iter().any(|i| i.id == "CSP-002"));
    }

    #[tokio::test]
    async fn test_security_score() {
        let auditor = SecurityAuditor::new(SecurityAuditorConfig::default());
        let score = auditor.get_security_score().await;
        assert_eq!(score, 100);
    }
}