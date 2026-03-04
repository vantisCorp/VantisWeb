//! Extension Sandbox Module
//!
//! Provides a sandboxed execution environment for browser extensions
//! to ensure security and isolate extension code from the main browser.

use anyhow::{Result, Error};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Sandbox for extension execution
pub struct ExtensionSandbox {
    config: SandboxConfig,
    active_sandboxes: Arc<RwLock<HashMap<String, SandboxInstance>>>,
}

/// Sandbox configuration
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Enable sandboxing
    pub enabled: bool,
    /// Memory limit in MB
    pub memory_limit_mb: u64,
    /// CPU time limit in seconds
    pub cpu_limit_seconds: u64,
    /// Network access allowed
    pub network_access: bool,
    /// File system access allowed
    pub file_access: bool,
    /// Allow access to browser APIs
    pub allow_browser_apis: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            memory_limit_mb: 128,
            cpu_limit_seconds: 30,
            network_access: true,
            file_access: false,
            allow_browser_apis: true,
        }
    }
}

/// Active sandbox instance
#[derive(Debug, Clone)]
struct SandboxInstance {
    extension_id: String,
    created_at: chrono::DateTime<chrono::Utc>,
    is_active: bool,
    resource_usage: ResourceUsage,
}

/// Resource usage tracking
#[derive(Debug, Clone, Default)]
pub struct ResourceUsage {
    pub memory_mb: f64,
    pub cpu_time_ms: u64,
    pub network_requests: u32,
    pub api_calls: u32,
}

/// Sandbox execution context
#[derive(Debug, Clone)]
pub struct SandboxContext {
    pub extension_id: String,
    pub permissions: Vec<String>,
    pub origin: String,
}

/// Sandbox execution result
#[derive(Debug, Clone)]
pub struct SandboxResult {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub resource_usage: ResourceUsage,
}

impl ExtensionSandbox {
    /// Create a new sandbox manager
    pub fn new(config: SandboxConfig) -> Self {
        Self {
            config,
            active_sandboxes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a sandbox for an extension
    pub async fn create_sandbox(&self, extension_id: &str) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let instance = SandboxInstance {
            extension_id: extension_id.to_string(),
            created_at: chrono::Utc::now(),
            is_active: true,
            resource_usage: ResourceUsage::default(),
        };

        let mut sandboxes = self.active_sandboxes.write().await;
        sandboxes.insert(extension_id.to_string(), instance);

        Ok(())
    }

    /// Execute code in sandbox
    pub async fn execute(
        &self,
        extension_id: &str,
        code: &str,
        context: &SandboxContext,
    ) -> Result<SandboxResult> {
        let sandboxes = self.active_sandboxes.read().await;
        
        if !sandboxes.contains_key(extension_id) {
            return Err(Error::msg("Sandbox not found for extension"));
        }

        // In a real implementation, this would execute code in a sandboxed environment
        // For now, we simulate execution
        let result = SandboxResult {
            success: true,
            output: Some("Code executed successfully".to_string()),
            error: None,
            resource_usage: ResourceUsage {
                memory_mb: 1.0,
                cpu_time_ms: 10,
                network_requests: 0,
                api_calls: 1,
            },
        };

        Ok(result)
    }

    /// Terminate a sandbox
    pub async fn terminate_sandbox(&self, extension_id: &str) -> Result<()> {
        let mut sandboxes = self.active_sandboxes.write().await;
        if let Some(mut instance) = sandboxes.get_mut(extension_id) {
            instance.is_active = false;
        }
        Ok(())
    }

    /// Get resource usage for an extension
    pub async fn get_resource_usage(&self, extension_id: &str) -> Option<ResourceUsage> {
        let sandboxes = self.active_sandboxes.read().await;
        sandboxes.get(extension_id).map(|s| s.resource_usage.clone())
    }

    /// Terminate all sandboxes
    pub async fn terminate_all(&self) {
        let mut sandboxes = self.active_sandboxes.write().await;
        for instance in sandboxes.values_mut() {
            instance.is_active = false;
        }
    }

    /// Get active sandbox count
    pub async fn active_count(&self) -> usize {
        let sandboxes = self.active_sandboxes.read().await;
        sandboxes.values().filter(|s| s.is_active).count()
    }

    /// Check if sandbox exists and is active
    pub async fn is_active(&self, extension_id: &str) -> bool {
        let sandboxes = self.active_sandboxes.read().await;
        sandboxes
            .get(extension_id)
            .map(|s| s.is_active)
            .unwrap_or(false)
    }

    /// Validate code before execution
    pub fn validate_code(&self, code: &str) -> Result<()> {
        // Check for dangerous patterns
        let dangerous_patterns = [
            "eval(",
            "Function(",
            "setTimeout(",
            "setInterval(",
            "require(",
            "import(",
        ];

        for pattern in &dangerous_patterns {
            if code.contains(pattern) {
                return Err(Error::msg(format!(
                    "Dangerous pattern detected: {}",
                    pattern
                )));
            }
        }

        Ok(())
    }

    /// Sanitize input to prevent code injection
    pub fn sanitize_input(&self, input: &str) -> String {
        input
            .replace("<script>", "")
            .replace("</script>", "")
            .replace("javascript:", "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::default();
        assert!(config.enabled);
        assert_eq!(config.memory_limit_mb, 128);
    }

    #[test]
    fn test_validate_code_safe() {
        let sandbox = ExtensionSandbox::new(SandboxConfig::default());
        assert!(sandbox.validate_code("console.log('hello')").is_ok());
    }

    #[test]
    fn test_validate_code_dangerous() {
        let sandbox = ExtensionSandbox::new(SandboxConfig::default());
        assert!(sandbox.validate_code("eval(malicious)").is_err());
    }

    #[test]
    fn test_sanitize_input() {
        let sandbox = ExtensionSandbox::new(SandboxConfig::default());
        let sanitized = sandbox.sanitize_input("<script>alert('xss')</script>");
        assert!(!sanitized.contains("<script>"));
    }

    #[tokio::test]
    async fn test_create_sandbox() {
        let sandbox = ExtensionSandbox::new(SandboxConfig::default());
        sandbox.create_sandbox("ext1").await.unwrap();
        assert!(sandbox.is_active("ext1").await);
    }

    #[tokio::test]
    async fn test_terminate_sandbox() {
        let sandbox = ExtensionSandbox::new(SandboxConfig::default());
        sandbox.create_sandbox("ext1").await.unwrap();
        sandbox.terminate_sandbox("ext1").await.unwrap();
        assert!(!sandbox.is_active("ext1").await);
    }

    #[tokio::test]
    async fn test_active_count() {
        let sandbox = ExtensionSandbox::new(SandboxConfig::default());
        sandbox.create_sandbox("ext1").await.unwrap();
        sandbox.create_sandbox("ext2").await.unwrap();
        sandbox.create_sandbox("ext3").await.unwrap();
        
        sandbox.terminate_sandbox("ext2").await.unwrap();
        assert_eq!(sandbox.active_count().await, 2);
    }

    #[tokio::test]
    async fn test_execute_code() {
        let sandbox = ExtensionSandbox::new(SandboxConfig::default());
        sandbox.create_sandbox("ext1").await.unwrap();
        
        let context = SandboxContext {
            extension_id: "ext1".to_string(),
            permissions: vec!["storage".to_string()],
            origin: "https://example.com".to_string(),
        };
        
        let result = sandbox.execute("ext1", "console.log('test')", &context).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_terminate_all() {
        let sandbox = ExtensionSandbox::new(SandboxConfig::default());
        sandbox.create_sandbox("ext1").await.unwrap();
        sandbox.create_sandbox("ext2").await.unwrap();
        sandbox.create_sandbox("ext3").await.unwrap();
        
        sandbox.terminate_all().await;
        assert_eq!(sandbox.active_count().await, 0);
    }
}