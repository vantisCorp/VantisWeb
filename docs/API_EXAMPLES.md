# VantisWeb Browser - API Examples

This document provides practical examples for using the VantisWeb APIs.

---

## Table of Contents

1. [Core API Examples](#core-api-examples)
2. [Security API Examples](#security-api-examples)
3. [Web Engine API Examples](#web-engine-api-examples)
4. [Extensions API Examples](#extensions-api-examples)
5. [Profiles API Examples](#profiles-api-examples)
6. [UI API Examples](#ui-api-examples)

---

## Core API Examples

### Initializing the Kernel

```rust
use vantisweb::core::kernel::VantisKernel;

#[tokio::main]
async fn main() -> Result<()> {
    // Create kernel instance
    let kernel = VantisKernel::new().await?;
    
    println!("Kernel initialized successfully!");
    
    Ok(())
}
```

### Registering Modules

```rust
use vantisweb::core::kernel::VantisKernel;

async fn register_custom_module(kernel: &VantisKernel) -> Result<()> {
    // Register a custom module
    kernel.register_module("custom_module".to_string()).await?;
    
    println!("Custom module registered!");
    
    Ok(())
}
```

### Getting Kernel State

```rust
use vantisweb::core::kernel::VantisKernel;

async fn get_kernel_state(kernel: &VantisKernel) -> Result<()> {
    // Get kernel state
    let state = kernel.get_state().await?;
    
    println!("Kernel state: {:?}", state);
    
    Ok(())
}
```

### Using the Scheduler

```rust
use vantisweb::core::scheduler::VantisMicroScheduler;

async fn schedule_task(scheduler: &VantisMicroScheduler) -> Result<()> {
    // Schedule a task
    scheduler.schedule_task(async {
        println!("Task executed!");
    }).await?;
    
    Ok(())
}
```

---

## Security API Examples

### Hashing Passwords

```rust
use vantisweb::security::CryptoEngine;

fn hash_password(password: &str) -> Result<String> {
    let crypto = CryptoEngine::new()?;
    let hash = crypto.hash(password)?;
    
    println!("Password hashed: {}", hash);
    
    Ok(hash)
}
```

### Encrypting Data

```rust
use vantisweb::security::CryptoEngine;

fn encrypt_data(data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    let crypto = CryptoEngine::new()?;
    let encrypted = crypto.encrypt(data, key)?;
    
    println!("Data encrypted!");
    
    Ok(encrypted)
}
```

### Decrypting Data

```rust
use vantisweb::security::CryptoEngine;

fn decrypt_data(encrypted: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    let crypto = CryptoEngine::new()?;
    let decrypted = crypto.decrypt(encrypted, key)?;
    
    println!("Data decrypted!");
    
    Ok(decrypted)
}
```

### Using the Security Manager

```rust
use vantisweb::security::SecurityManager;

async fn check_security_status(manager: &SecurityManager) -> Result<()> {
    // Check security status
    let status = manager.get_status().await?;
    
    println!("Security status: {:?}", status);
    
    Ok(())
}
```

---

## Web Engine API Examples

### Creating a Web Renderer

```rust
use vantisweb::engine::web_renderer::WebRenderer;
use std::sync::Arc;

async fn create_web_renderer(kernel: Arc<VantisKernel>) -> Result<()> {
    // Create web renderer
    let renderer = WebRenderer::new(kernel)?;
    
    println!("Web renderer created!");
    
    Ok(())
}
```

### Loading a URL

```rust
use vantisweb::engine::web_renderer::WebRenderer;

async fn load_url(renderer: &WebRenderer, url: String) -> Result<()> {
    // Load URL
    renderer.load_url(url).await?;
    
    println!("URL loaded!");
    
    Ok(())
}
```

### Executing JavaScript

```rust
use vantisweb::engine::web_renderer::WebRenderer;

async fn execute_javascript(renderer: &WebRenderer, code: String) -> Result<String> {
    // Execute JavaScript
    let result = renderer.execute_javascript(code).await?;
    
    println!("JavaScript result: {}", result);
    
    Ok(result)
}
```

### Getting Page Title

```rust
use vantisweb::engine::web_renderer::WebRenderer;

async fn get_page_title(renderer: &WebRenderer) -> Result<Option<String>> {
    // Get page title
    let title = renderer.get_page_title().await;
    
    println!("Page title: {:?}", title);
    
    Ok(title)
}
```

### Navigation

```rust
use vantisweb::engine::web_renderer::WebRenderer;

async fn navigate(renderer: &WebRenderer) -> Result<()> {
    // Go back
    renderer.go_back().await?;
    
    // Go forward
    renderer.go_forward().await?;
    
    // Reload
    renderer.reload().await?;
    
    println!("Navigation complete!");
    
    Ok(())
}
```

---

## Extensions API Examples

### Creating an Extension Manager

```rust
use vantisweb::extensions::ExtensionManager;

async fn create_extension_manager() -> Result<()> {
    // Create extension manager
    let manager = ExtensionManager::new();
    
    println!("Extension manager created!");
    
    Ok(())
}
```

### Loading Extensions

```rust
use vantisweb::extensions::{ExtensionManager, ExtensionLoader};

async fn load_extensions(manager: &mut ExtensionManager) -> Result<()> {
    // Create extension loader
    let loader = ExtensionLoader::new("/path/to/extensions");
    
    // Load all extensions
    loader.load_all(manager).await?;
    
    println!("Extensions loaded!");
    
    Ok(())
}
```

### Enabling an Extension

```rust
use vantisweb::extensions::ExtensionManager;

async fn enable_extension(manager: &mut ExtensionManager, id: String) -> Result<()> {
    // Enable extension
    manager.enable_extension(&id).await?;
    
    println!("Extension enabled: {}", id);
    
    Ok(())
}
```

### Getting Extension Info

```rust
use vantisweb::extensions::ExtensionManager;

async fn get_extension_info(manager: &ExtensionManager, id: String) -> Result<()> {
    // Get extension info
    if let Some(info) = manager.get_extension(&id) {
        println!("Extension: {}", info.name);
        println!("Version: {}", info.version);
        println!("Description: {}", info.description);
    }
    
    Ok(())
}
```

### Using Extension APIs

```rust
use vantisweb::extensions::api::{browser::BrowserApi, storage::StorageApi};

async fn use_extension_apis() -> Result<()> {
    // Use browser API
    let browser_api = BrowserApi::new();
    browser_api.open_tab("https://example.com").await?;
    
    // Use storage API
    let storage_api = StorageApi::new();
    storage_api.set("key", "value").await?;
    let value = storage_api.get("key").await?;
    
    println!("Value: {:?}", value);
    
    Ok(())
}
```

---

## Profiles API Examples

### Creating a Profile Manager

```rust
use vantisweb::profiles::ProfileManager;
use std::sync::Arc;

async fn create_profile_manager(kernel: Arc<VantisKernel>) -> Result<()> {
    // Create profile manager
    let manager = ProfileManager::new(kernel).await?;
    
    println!("Profile manager created!");
    
    Ok(())
}
```

### Creating a Profile

```rust
use vantisweb::profiles::{ProfileManager, ProfileConfig, ProfileType};

async fn create_profile(manager: &ProfileManager, name: String) -> Result<()> {
    // Create profile
    let profile = ProfileConfig::new(name, ProfileType::Custom("Custom".to_string()));
    
    // Save profile
    manager.create_profile(profile).await?;
    
    println!("Profile created!");
    
    Ok(())
}
```

### Switching Profiles

```rust
use vantisweb::profiles::ProfileManager;

async fn switch_profile(manager: &ProfileManager, profile_id: String) -> Result<()> {
    // Switch profile
    manager.set_active_profile(&profile_id).await?;
    
    println!("Profile switched: {}", profile_id);
    
    Ok(())
}
```

### Using Templates

```rust
use vantisweb::profiles::TemplateManager;

async fn use_templates() -> Result<()> {
    // Create template manager
    let template_manager = TemplateManager::new();
    
    // Get template
    if let Some(template) = template_manager.get_template("Work") {
        println!("Template: {}", template.name);
        println!("Description: {}", template.description);
    }
    
    Ok(())
}
```

### Profile Synchronization

```rust
use vantisweb::profiles::ProfileSyncManager;

async fn sync_profiles(manager: &mut ProfileSyncManager) -> Result<()> {
    // Enable sync
    manager.enable(SyncProvider::Local)?;
    
    // Sync profiles
    let result = manager.sync()?;
    
    println!("Sync result: {:?}", result);
    
    Ok(())
}
```

### Profile Analytics

```rust
use vantisweb::profiles::AnalyticsManager;

async fn track_analytics(manager: &mut AnalyticsManager, profile_id: String) -> Result<()> {
    // Start session
    manager.start_session(&profile_id)?;
    
    // Record website visit
    manager.record_visit(&profile_id, "https://example.com")?;
    
    // Record tab opened
    manager.record_tab_opened(&profile_id)?;
    
    // End session
    manager.end_session(&profile_id)?;
    
    println!("Analytics tracked!");
    
    Ok(())
}
```

### Profile Security

```rust
use vantisweb::profiles::{ProfileSecurityManager, SecurityLevel};

async fn setup_profile_security(manager: &mut ProfileSecurityManager, profile_id: String) -> Result<()> {
    // Create security
    manager.create_security(&profile_id, SecurityLevel::High)?;
    
    // Set password
    manager.set_password(&profile_id, "my_password")?;
    
    // Verify password
    let valid = manager.verify_password(&profile_id, "my_password")?;
    
    println!("Password valid: {}", valid);
    
    Ok(())
}
```

---

## UI API Examples

### Creating VantisUI

```rust
use vantisweb::ui::VantisUI;
use std::sync::Arc;

async fn create_ui(kernel: Arc<VantisKernel>) -> Result<()> {
    // Create UI
    let mut ui = VantisUI::new(kernel).await?;
    
    println!("UI created!");
    
    Ok(())
}
```

### Running the UI

```rust
use vantisweb::ui::VantisUI;

async fn run_ui(ui: &mut VantisUI) -> Result<()> {
    // Run UI main loop
    ui.run().await?;
    
    println!("UI running!");
    
    Ok(())
}
```

### Creating Browser Window

```rust
use vantisweb::ui::browser::BrowserWindow;
use std::sync::Arc;

async fn create_browser_window(kernel: Arc<VantisKernel>) -> Result<()> {
    // Create browser window
    let window = BrowserWindow::new(
        "VantisWeb Browser".to_string(),
        1920,
        1080,
        kernel,
    ).await?;
    
    println!("Browser window created!");
    
    Ok(())
}
```

### Managing Tabs

```rust
use vantisweb::ui::browser::BrowserWindow;

async fn manage_tabs(window: &mut BrowserWindow) -> Result<()> {
    // Create tab
    window.create_tab("https://example.com".to_string()).await?;
    
    // Navigate
    window.navigate("https://github.com".to_string()).await?;
    
    // Go back
    window.go_back().await?;
    
    // Close tab
    window.close_tab().await?;
    
    println!("Tabs managed!");
    
    Ok(())
}
```

### Using Themes

```rust
use vantisweb::ui::theming::ThemeManager;

async fn use_themes() -> Result<()> {
    // Create theme manager
    let theme_manager = ThemeManager::new()?;
    
    // Get current theme
    let theme = theme_manager.current_theme();
    println!("Current theme: {}", theme.name);
    
    // Get CSS variables
    let css = theme_manager.to_css_variables();
    println!("CSS: {}", css);
    
    Ok(())
}
```

### Using UI Components

```rust
use vantisweb::ui::components::{Button, Input, UIComponent};

fn use_components() {
    // Create button
    let button = Button::new("my-button".to_string(), "Click Me".to_string());
    let button_html = button.render();
    println!("Button HTML: {}", button_html);
    
    // Create input
    let input = Input::new("my-input".to_string(), "Enter text".to_string());
    let input_html = input.render();
    println!("Input HTML: {}", input_html);
}
```

---

## Complete Example: Simple Browser

```rust
use vantisweb::core::kernel::VantisKernel;
use vantisweb::ui::VantisUI;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize kernel
    let kernel = Arc::new(VantisKernel::new().await?);
    
    // Create UI
    let mut ui = VantisUI::new(kernel.clone()).await?;
    
    // Run UI
    ui.run().await?;
    
    Ok(())
}
```

---

## Complete Example: Extension Manager

```rust
use vantisweb::extensions::{ExtensionManager, ExtensionLoader};

#[tokio::main]
async fn main() -> Result<()> {
    // Create extension manager
    let mut manager = ExtensionManager::new();
    
    // Create extension loader
    let loader = ExtensionLoader::new("/path/to/extensions");
    
    // Load all extensions
    loader.load_all(&mut manager).await?;
    
    // List all extensions
    let extensions = manager.get_all_extensions();
    for ext in extensions {
        println!("Extension: {} (v{})", ext.name, ext.version);
    }
    
    Ok(())
}
```

---

## Complete Example: Profile Manager

```rust
use vantisweb::profiles::{ProfileManager, ProfileConfig, ProfileType};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize kernel
    let kernel = Arc::new(VantisKernel::new().await?);
    
    // Create profile manager
    let manager = ProfileManager::new(kernel).await?;
    
    // Create profile
    let profile = ProfileConfig::new("My Profile".to_string(), ProfileType::Custom("Custom".to_string()));
    manager.create_profile(profile).await?;
    
    // List all profiles
    let profiles = manager.get_all_profiles().await;
    for profile in profiles {
        println!("Profile: {}", profile.name);
    }
    
    Ok(())
}
```

---

## Resources

- [API Reference](API_REFERENCE.md)
- [Developer Tutorial](DEVELOPER_TUTORIAL.md)
- [Testing Guide](TESTING_GUIDE.md)
- [Extensions Guide](EXTENSIONS.md)
- [Profiles Guide](../profiles/README.md)

---

**Last Updated:** March 2, 2025  
**Version:** 1.0.0