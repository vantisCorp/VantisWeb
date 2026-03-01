//! JavaScript Bridge
//!
//! JavaScript-to-Rust and Rust-to-JavaScript bridge:
//! - JavaScript context management
//! - Function registration system
//! - Callback system
//! - Promise handling

use anyhow::Result;
use log::{debug, info};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::core::kernel::VantisKernel;
use super::fetch::FetchApi;
use super::storage::StorageApi;
use super::console::ConsoleApi;
use super::event_loop::EventLoop;

/// JavaScript function type (async)
pub type JsFunction = Box<dyn Fn(Vec<serde_json::Value>) -> Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>> + Send + Sync>;

/// JavaScript context
pub struct JsContext {
    /// Registered JavaScript functions
    functions: Arc<RwLock<HashMap<String, JsFunction>>>,
    /// Registered JavaScript objects
    objects: Arc<RwLock<HashMap<String, HashMap<String, JsFunction>>>>,
}

impl JsContext {
    /// Create a new JavaScript context
    pub fn new() -> Self {
        info!("Initializing JavaScript Bridge Context...");
        
        Self {
            functions: Arc::new(RwLock::new(HashMap::new())),
            objects: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a JavaScript function
    pub async fn register_function(&self, name: String, func: JsFunction) {
        debug!("Registering JavaScript function: {}", name);
        self.functions.write().await.insert(name, func);
    }
    
    /// Register a JavaScript object with methods
    pub async fn register_object(&self, name: String, methods: HashMap<String, JsFunction>) {
        debug!("Registering JavaScript object: {}", name);
        self.objects.write().await.insert(name, methods);
    }
    
    /// Call a JavaScript function
    pub async fn call_function(&self, name: &str, args: Vec<serde_json::Value>) -> Result<serde_json::Value> {
        let functions = self.functions.read().await;
        
        match functions.get(name) {
            Some(func) => {
                debug!("Calling JavaScript function: {}", name);
                func(args).await
            }
            None => Err(anyhow::anyhow!("Function not found: {}", name)),
        }
    }
    
    /// Call a JavaScript object method
    pub async fn call_method(&self, object: &str, method: &str, args: Vec<serde_json::Value>) -> Result<serde_json::Value> {
        let objects = self.objects.read().await;
        
        match objects.get(object) {
            Some(methods) => {
                match methods.get(method) {
                    Some(func) => {
                        debug!("Calling JavaScript method: {}.{}", object, method);
                        func(args).await
                    }
                    None => Err(anyhow::anyhow!("Method not found: {}.{}", object, method)),
                }
            }
            None => Err(anyhow::anyhow!("Object not found: {}", object)),
        }
    }
}

/// JavaScript Bridge
#[derive(Clone)]
pub struct JsBridge {
    kernel: Arc<VantisKernel>,
    context: Arc<JsContext>,
    fetch_api: Arc<FetchApi>,
    storage_api: Arc<StorageApi>,
    console_api: Arc<ConsoleApi>,
    event_loop: Arc<EventLoop>,
}

impl JsBridge {
    /// Create a new JavaScript bridge
    pub fn new(
        kernel: Arc<VantisKernel>,
        fetch_api: Arc<FetchApi>,
        storage_api: Arc<StorageApi>,
        console_api: Arc<ConsoleApi>,
        event_loop: Arc<EventLoop>,
    ) -> Self {
        info!("Initializing JavaScript Bridge...");
        
        let context = Arc::new(JsContext::new());
        
        let bridge = Self {
            kernel: kernel.clone(),
            context: context.clone(),
            fetch_api,
            storage_api,
            console_api,
            event_loop,
        };
        
        // Register Web APIs
        let bridge_clone = bridge.clone();
        tokio::spawn(async move {
            bridge_clone.register_fetch_api().await;
            bridge_clone.register_storage_api().await;
            bridge_clone.register_console_api().await;
            bridge_clone.register_event_loop().await;
        });
        
        bridge
    }
    
    /// Get the JavaScript context
    pub fn get_context(&self) -> Arc<JsContext> {
        self.context.clone()
    }
    
    /// Register Fetch API
    async fn register_fetch_api(&self) {
        debug!("Registering Fetch API...");
        
        // Register fetch function
        let fetch_api = self.fetch_api.clone();
        self.context.register_function(
            "fetch".to_string(),
            Box::new(move |args: Vec<serde_json::Value>| {
                let fetch_api = fetch_api.clone();
                Box::pin(async move {
                    // Parse arguments
                    if args.is_empty() {
                        return Err(anyhow::anyhow!("fetch requires URL argument"));
                    }
                    
                    let url = args[0].as_str().ok_or_else(|| anyhow::anyhow!("URL must be a string"))?;
                    
                    // Create HTTP request
                    let request = crate::engine::fetch::HttpRequest {
                        url: url.to_string(),
                        method: crate::engine::fetch::HttpMethod::GET,
                        headers: crate::engine::fetch::HttpHeaders::new(),
                        body: crate::engine::fetch::RequestBody::None,
                        follow_redirects: true,
                        timeout: 30000,
                        user_agent: "VantisWeb/0.1.0".to_string(),
                    };
                    
                    // Execute request
                    let response = fetch_api.fetch(request).await?;
                    
                    // Convert response to JSON
                    Ok(serde_json::json!({
                        "status": response.status,
                        "statusText": response.status_text,
                        "headers": response.headers.all(),
                        "body": response.body,
                    }))
                }) as Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>>
            }),
        ).await;
    }
    
    /// Register Storage API
    async fn register_storage_api(&self) {
        debug!("Registering Storage API...");
        
        let storage_api = self.storage_api.clone();
        
        // Register localStorage object
        let mut local_storage_methods = HashMap::new();
        
        // localStorage.getItem
        let storage_api_clone = storage_api.clone();
        local_storage_methods.insert(
            "getItem".to_string(),
            Box::new(move |args: Vec<serde_json::Value>| {
                let storage_api = storage_api_clone.clone();
                Box::pin(async move {
                    if args.is_empty() {
                        return Err(anyhow::anyhow!("getItem requires key argument"));
                    }
                    
                    let key = args[0].as_str().ok_or_else(|| anyhow::anyhow!("key must be a string"))?;
                    let value = storage_api.get_item(crate::engine::storage::StorageType::Local, key.to_string()).await;
                    
                    Ok(serde_json::json!(value))
                }) as Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>>
            }) as JsFunction,
        );
        
        // localStorage.setItem
        let storage_api_clone = storage_api.clone();
        local_storage_methods.insert(
            "setItem".to_string(),
            Box::new(move |args: Vec<serde_json::Value>| {
                let storage_api = storage_api_clone.clone();
                Box::pin(async move {
                    if args.len() < 2 {
                        return Err(anyhow::anyhow!("setItem requires key and value arguments"));
                    }
                    
                    let key = args[0].as_str().ok_or_else(|| anyhow::anyhow!("key must be a string"))?;
                    let value = args[1].as_str().ok_or_else(|| anyhow::anyhow!("value must be a string"))?;
                    
                    storage_api.set_item(crate::engine::storage::StorageType::Local, key.to_string(), value.to_string()).await?;
                    
                    Ok(serde_json::json!(null))
                }) as Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>>
            }) as JsFunction,
        );
        
        // localStorage.removeItem
        let storage_api_clone = storage_api.clone();
        local_storage_methods.insert(
            "removeItem".to_string(),
            Box::new(move |args: Vec<serde_json::Value>| {
                let storage_api = storage_api_clone.clone();
                Box::pin(async move {
                    if args.is_empty() {
                        return Err(anyhow::anyhow!("removeItem requires key argument"));
                    }
                    
                    let key = args[0].as_str().ok_or_else(|| anyhow::anyhow!("key must be a string"))?;
                    storage_api.remove_item(crate::engine::storage::StorageType::Local, key.to_string()).await?;
                    
                    Ok(serde_json::json!(null))
                }) as Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>>
            }) as JsFunction,
        );
        
        // localStorage.clear
        let storage_api_clone = storage_api.clone();
        local_storage_methods.insert(
            "clear".to_string(),
            Box::new(move |_args| {
                let storage_api = storage_api_clone.clone();
                Box::pin(async move {
                    storage_api.clear(crate::engine::storage::StorageType::Local).await?;
                    Ok(serde_json::json!(null))
                }) as Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>> as Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>>
            }) as JsFunction,
        );
        
        self.context.register_object("localStorage".to_string(), local_storage_methods).await;
        
        // Register sessionStorage object (similar implementation)
        let mut session_storage_methods = HashMap::new();
        
        // sessionStorage.getItem
        let storage_api_clone = storage_api.clone();
        session_storage_methods.insert(
            "getItem".to_string(),
            Box::new(move |args: Vec<serde_json::Value>| {
                let storage_api = storage_api_clone.clone();
                Box::pin(async move {
                    if args.is_empty() {
                        return Err(anyhow::anyhow!("getItem requires key argument"));
                    }
                    
                    let key = args[0].as_str().ok_or_else(|| anyhow::anyhow!("key must be a string"))?;
                    let value = storage_api.get_item(crate::engine::storage::StorageType::Session, key.to_string()).await;
                    
                    Ok(serde_json::json!(value))
                }) as Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>>
            }) as JsFunction,
        );
        
        // sessionStorage.setItem
        let storage_api_clone = storage_api.clone();
        session_storage_methods.insert(
            "setItem".to_string(),
            Box::new(move |args: Vec<serde_json::Value>| {
                let storage_api = storage_api_clone.clone();
                Box::pin(async move {
                    if args.len() < 2 {
                        return Err(anyhow::anyhow!("setItem requires key and value arguments"));
                    }
                    
                    let key = args[0].as_str().ok_or_else(|| anyhow::anyhow!("key must be a string"))?;
                    let value = args[1].as_str().ok_or_else(|| anyhow::anyhow!("value must be a string"))?;
                    
                    storage_api.set_item(crate::engine::storage::StorageType::Session, key.to_string(), value.to_string()).await?;
                    
                    Ok(serde_json::json!(null))
                }) as Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>>
            }) as JsFunction,
        );
        
        // sessionStorage.removeItem
        let storage_api_clone = storage_api.clone();
        session_storage_methods.insert(
            "removeItem".to_string(),
            Box::new(move |args: Vec<serde_json::Value>| {
                let storage_api = storage_api_clone.clone();
                Box::pin(async move {
                    if args.is_empty() {
                        return Err(anyhow::anyhow!("removeItem requires key argument"));
                    }
                    
                    let key = args[0].as_str().ok_or_else(|| anyhow::anyhow!("key must be a string"))?;
                    storage_api.remove_item(crate::engine::storage::StorageType::Session, key.to_string()).await?;
                    
                    Ok(serde_json::json!(null))
                }) as Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>>
            }) as JsFunction,
        );
        
        // sessionStorage.clear
        let storage_api_clone = storage_api.clone();
        session_storage_methods.insert(
            "clear".to_string(),
            Box::new(move |_args| {
                let storage_api = storage_api_clone.clone();
                Box::pin(async move {
                    storage_api.clear(crate::engine::storage::StorageType::Session).await?;
                    Ok(serde_json::json!(null))
                }) as Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>> as Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>>
            }) as JsFunction,
        );
        
        self.context.register_object("sessionStorage".to_string(), session_storage_methods).await;
    }
    
    /// Register Console API
    async fn register_console_api(&self) {
        debug!("Registering Console API...");
        
        let console_api = self.console_api.clone();
        
        let mut console_methods = HashMap::new();
        
        // console.log
        let console_api_clone = console_api.clone();
        console_methods.insert(
            "log".to_string(),
            Box::new(move |args: Vec<serde_json::Value>| {
                let console_api = console_api_clone.clone();
                Box::pin(async move {
                    let message = args.iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(" ");
                    
                    console_api.log(crate::engine::console::LogLevel::Info, message, vec![]).await;
                    Ok(serde_json::json!(null))
                }) as Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>> as Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>>
            }) as JsFunction,
        );
        
        // console.warn
        let console_api_clone = console_api.clone();
        console_methods.insert(
            "warn".to_string(),
            Box::new(move |args: Vec<serde_json::Value>| {
                let console_api = console_api_clone.clone();
                Box::pin(async move {
                    let message = args.iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(" ");
                    
                    console_api.log(crate::engine::console::LogLevel::Warn, message, vec![]).await;
                    Ok(serde_json::json!(null))
                }) as Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>> as Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>>
            }) as JsFunction,
        );
        
        // console.error
        let console_api_clone = console_api.clone();
        console_methods.insert(
            "error".to_string(),
            Box::new(move |args: Vec<serde_json::Value>| {
                let console_api = console_api_clone.clone();
                Box::pin(async move {
                    let message = args.iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(" ");
                    
                    console_api.log(crate::engine::console::LogLevel::Error, message, vec![]).await;
                    Ok(serde_json::json!(null))
                }) as Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>> as Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>>
            }) as JsFunction,
        );
        
        // console.info
        let console_api_clone = console_api.clone();
        console_methods.insert(
            "info".to_string(),
            Box::new(move |args: Vec<serde_json::Value>| {
                let console_api = console_api_clone.clone();
                Box::pin(async move {
                    let message = args.iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(" ");
                    
                    console_api.log(crate::engine::console::LogLevel::Info, message, vec![]).await;
                    Ok(serde_json::json!(null))
                }) as Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>> as Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>>
            }) as JsFunction,
        );
        
        // console.debug
        let console_api_clone = console_api.clone();
        console_methods.insert(
            "debug".to_string(),
            Box::new(move |args: Vec<serde_json::Value>| {
                let console_api = console_api_clone.clone();
                Box::pin(async move {
                    let message = args.iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(" ");
                    
                    console_api.log(crate::engine::console::LogLevel::Debug, message, vec![]).await;
                    Ok(serde_json::json!(null))
                }) as Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>> as Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>>
            }) as JsFunction,
        );
        
        self.context.register_object("console".to_string(), console_methods).await;
    }
    
    /// Register Event Loop
    async fn register_event_loop(&self) {
        debug!("Registering Event Loop...");
        
        let event_loop = self.event_loop.clone();
        
        // setTimeout
        let event_loop_clone = event_loop.clone();
        self.context.register_function(
            "setTimeout".to_string(),
            Box::new(move |args: Vec<serde_json::Value>| {
                let event_loop = event_loop_clone.clone();
                Box::pin(async move {
                    if args.is_empty() {
                        return Err(anyhow::anyhow!("setTimeout requires callback argument"));
                    }
                    
                    let delay_ms = if args.len() > 1 {
                        args[1].as_u64().unwrap_or(0)
                    } else {
                        0
                    };
                    
                    // Create a timeout task
                    let task_name = format!("setTimeout_{}", uuid::Uuid::new_v4());
                    let duration = std::time::Duration::from_millis(delay_ms);
                    
                    let task_id = event_loop.set_timeout(task_name, duration).await?;
                    
                    Ok(serde_json::json!(task_id))
                }) as Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>>
            }),
        ).await;
        
        // setInterval
        let event_loop_clone = event_loop.clone();
        self.context.register_function(
            "setInterval".to_string(),
            Box::new(move |args: Vec<serde_json::Value>| {
                let event_loop = event_loop_clone.clone();
                Box::pin(async move {
                    if args.is_empty() {
                        return Err(anyhow::anyhow!("setInterval requires callback argument"));
                    }
                    
                    let interval_ms = if args.len() > 1 {
                        args[1].as_u64().unwrap_or(0)
                    } else {
                        0
                    };
                    
                    // Create an interval task
                    let task_name = format!("setInterval_{}", uuid::Uuid::new_v4());
                    let duration = std::time::Duration::from_millis(interval_ms);
                    
                    let task_id = event_loop.set_interval(task_name, duration).await?;
                    
                    Ok(serde_json::json!(task_id))
                }) as Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>>
            }),
        ).await;
        
        // clearTimeout
        let event_loop_clone = event_loop.clone();
        self.context.register_function(
            "clearTimeout".to_string(),
            Box::new(move |args: Vec<serde_json::Value>| {
                let event_loop = event_loop_clone.clone();
                Box::pin(async move {
                    if args.is_empty() {
                        return Err(anyhow::anyhow!("clearTimeout requires task_id argument"));
                    }
                    
                    let task_id = args[0].as_str().ok_or_else(|| anyhow::anyhow!("task_id must be a string"))?;
                    event_loop.cancel_timer(task_id.to_string()).await?;
                    
                    Ok(serde_json::json!(null))
                }) as Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>>
            }),
        ).await;
        
        // clearInterval
        let event_loop_clone = event_loop.clone();
        self.context.register_function(
            "clearInterval".to_string(),
            Box::new(move |args: Vec<serde_json::Value>| {
                let event_loop = event_loop_clone.clone();
                Box::pin(async move {
                    if args.is_empty() {
                        return Err(anyhow::anyhow!("clearInterval requires task_id argument"));
                    }
                    
                    let task_id = args[0].as_str().ok_or_else(|| anyhow::anyhow!("task_id must be a string"))?;
                    event_loop.cancel_timer(task_id.to_string()).await?;
                    
                    Ok(serde_json::json!(null))
                }) as Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>>
            }),
        ).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_js_context_creation() {
        let context = JsContext::new();
        assert!(Arc::strong_count(&context.functions) > 0);
    }

    #[tokio::test]
    async fn test_register_function() {
        let context = JsContext::new();
        
        context.register_function(
            "test".to_string(),
            Box::new(|_args| Ok(serde_json::json!("test result"))),
        ).await;
        
        let result = context.call_function("test", vec![]).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), serde_json::json!("test result"));
    }

    #[tokio::test]
    async fn test_register_object() {
        let context = JsContext::new();
        
        let mut methods = HashMap::new();
        methods.insert(
            "testMethod".to_string(),
            Box::new(|_args| Ok(serde_json::json!("method result"))) as JsFunction,
        );
        
        context.register_object("testObject".to_string(), methods).await;
        
        let result = context.call_method("testObject", "testMethod", vec![]).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), serde_json::json!("method result"));
    }
}