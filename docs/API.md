# VantisWeb Browser - API Documentation

## Table of Contents
1. [Core Module](#core-module)
2. [Security Module](#security-module)
3. [UI Module](#ui-module)
4. [Examples](#examples)

---

## Core Module

### VantisKernel

Central orchestration system for VantisWeb Browser.

#### Methods

```rust
pub async fn new() -> Result<VantisKernel>
```
Creates a new Vantis Kernel instance and initializes all subsystems.

**Returns**: `Result<VantisKernel>` - Kernel instance or error

**Example**:
```rust
use vanisweb::core::kernel::VantisKernel;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let kernel = VantisKernel::new().await?;
    Ok(())
}
```

---

```rust
pub async fn get_state(&self) -> KernelState
```
Gets the current kernel state.

**Returns**: `KernelState` - Current state information

**Example**:
```rust
let state = kernel.get_state().await;
println!("Uptime: {} seconds", state.uptime_seconds);
```

---

```rust
pub async fn register_module(&self, name: String) -> Result<()>
```
Registers a new module with the kernel.

**Parameters**:
- `name`: `String` - Module name

**Returns**: `Result<()>` - Success or error

**Example**:
```rust
kernel.register_module("web_engine".to_string()).await?;
```

---

### MicroScheduler

Advanced CPU thread management with priority-based scheduling.

#### Methods

```rust
pub async fn new(config: Arc<VantisConfig>) -> Result<Self>
```
Creates a new micro-scheduler instance.

**Parameters**:
- `config`: `Arc<VantisConfig>` - Configuration reference

**Returns**: `Result<MicroScheduler>` - Scheduler instance or error

---

```rust
pub async fn schedule_task<F>(&self, name: String, priority: TaskPriority, func: F) -> Result<()>
```
Schedules a task for execution.

**Parameters**:
- `name`: `String` - Task name
- `priority`: `TaskPriority` - Task priority (Critical, High, Normal, Low)
- `func`: `F` - Function to execute

**Returns**: `Result<()>` - Success or error

**Example**:
```rust
scheduler.schedule_task(
    "render_ui".to_string(),
    TaskPriority::Critical,
    || {
        println!("Rendering UI...");
    }
).await?;
```

---

### StorageManager

Advanced data persistence system with encryption support.

#### Methods

```rust
pub async fn store(&self, key: &str, value: &[u8]) -> Result<()>
```
Stores a value with encryption.

**Parameters**:
- `key`: `&str` - Storage key
- `value`: `&[u8]` - Value to store

**Returns**: `Result<()>` - Success or error

**Example**:
```rust
storage.store("user_token", b"secret_token_123").await?;
```

---

```rust
pub async fn retrieve(&self, key: &str) -> Result<Option<Vec<u8>>>
```
Retrieves a value by key.

**Parameters**:
- `key`: `&str` - Storage key

**Returns**: `Result<Option<Vec<u8>>>` - Value or None if not found

**Example**:
```rust
if let Some(value) = storage.retrieve("user_token").await? {
    println!("Retrieved: {:?}", value);
}
```

---

### HistoryManager

Browsing history management.

#### Methods

```rust
pub fn add_entry(&mut self, url: String, title: String) -> Result<()>
```
Adds a history entry.

**Parameters**:
- `url`: `String` - Page URL
- `title`: `String` - Page title

**Returns**: `Result<()>` - Success or error

**Example**:
```rust
history.add_entry(
    "https://example.com".to_string(),
    "Example Site".to_string()
)?;
```

---

```rust
pub fn search(&self, query: &str) -> Vec<HistoryEntry>
```
Searches history by query.

**Parameters**:
- `query`: `&str` - Search query

**Returns**: `Vec<HistoryEntry>` - Matching entries

**Example**:
```rust
let results = history.search("example");
for entry in results {
    println!("Found: {} - {}", entry.title, entry.url);
}
```

---

### BookmarkManager

Bookmark management with folder support.

#### Methods

```rust
pub fn add(&mut self, url: String, title: String, folder: Option<String>) -> Result<()>
```
Adds a bookmark.

**Parameters**:
- `url`: `String` - Page URL
- `title`: `String` - Page title
- `folder`: `Option<String>` - Folder name (defaults to "General")

**Returns**: `Result<()>` - Success or error

**Example**:
```rust
bookmarks.add(
    "https://example.com".to_string(),
    "Example Site".to_string(),
    Some("Reading".to_string())
)?;
```

---

```rust
pub fn get_by_folder(&self, folder: &str) -> Vec<Bookmark>
```
Gets all bookmarks in a folder.

**Parameters**:
- `folder`: `&str` - Folder name

**Returns**: `Vec<Bookmark>` - Bookmarks in folder

**Example**:
```rust
let reading_bookmarks = bookmarks.get_by_folder("Reading");
```

---

### SettingsManager

Application settings management.

#### Methods

```rust
pub fn new(settings_file: String) -> Result<Self>
```
Creates a new settings manager.

**Parameters**:
- `settings_file`: `String` - Path to settings file

**Returns**: `Result<SettingsManager>` - Settings manager or error

---

```rust
pub fn save(&self) -> Result<()>
```
Saves current settings to file.

**Returns**: `Result<()>` - Success or error

**Example**:
```rust
settings_manager.save()?;
```

---

### PrivateModeManager

Private/incognito mode management.

#### Methods

```rust
pub fn enable_private_mode(&mut self) -> Result<()>
```
Enables private mode.

**Returns**: `Result<()>` - Success or error

**Example**:
```rust
private_mode.enable_private_mode()?;
```

---

```rust
pub fn is_private_mode(&self) -> bool
```
Checks if private mode is active.

**Returns**: `bool` - True if private mode is active

---

## Security Module

### SecurityManager

Central security coordination.

#### Methods

```rust
pub async fn new() -> Result<Self>
```
Creates a new security manager.

**Returns**: `Result<SecurityManager>` - Security manager or error

---

```rust
pub async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>>
```
Encrypts data using post-quantum cryptography.

**Parameters**:
- `data`: `&[u8]` - Data to encrypt

**Returns**: `Result<Vec<u8>>` - Encrypted data

**Example**:
```rust
let encrypted = security.encrypt(b"secret data").await?;
```

---

```rust
pub async fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>>
```
Decrypts data.

**Parameters**:
- `data`: `&[u8]` - Data to decrypt

**Returns**: `Result<Vec<u8>>` - Decrypted data

---

### CryptoEngine

Post-quantum cryptography engine.

#### Methods

```rust
pub fn new() -> Result<Self>
```
Creates a new crypto engine.

**Returns**: `Result<CryptoEngine>` - Crypto engine or error

---

```rust
pub fn hash(&self, data: &[u8]) -> Result<String>
```
Generates a hash using BLAKE3.

**Parameters**:
- `data`: `&[u8]` - Data to hash

**Returns**: `Result<String>` - Hex-encoded hash

**Example**:
```rust
let hash = crypto.hash(b"important data")?;
println!("Hash: {}", hash);
```

---

### DigitalImmuneSystem

Self-healing security system.

#### Methods

```rust
pub async fn scan(&self, path: &str) -> Result<u32>
```
Scans for threats.

**Parameters**:
- `path`: `&str` - Path to scan

**Returns**: `Result<u32>` - Number of threats found

---

### Sandbox

Process isolation system.

#### Methods

```rust
pub fn initialize(&mut self) -> Result<()>
```
Initializes the sandbox system.

**Returns**: `Result<()>` - Success or error

---

```rust
pub fn isolate(&mut self, process_id: String) -> Result<()>
```
Isolates a process.

**Parameters**:
- `process_id`: `String` - Process ID to isolate

**Returns**: `Result<()>` - Success or error

---

## UI Module

### VantisUI

Main UI application.

#### Methods

```rust
pub async fn new(kernel: Arc<VantisKernel>) -> Result<Self>
```
Creates a new VantisUI instance.

**Parameters**:
- `kernel`: `Arc<VantisKernel>` - Kernel reference

**Returns**: `Result<VantisUI>` - UI instance or error

---

```rust
pub async fn run(&mut self) -> Result<()>
```
Runs the main UI event loop.

**Returns**: `Result<()>` - Success or error

---

### BrowserWindow

Browser window with tab support.

#### Methods

```rust
pub async fn new(title: String, width: u32, height: u32, kernel: Arc<VantisKernel>) -> Result<Self>
```
Creates a new browser window.

**Parameters**:
- `title`: `String` - Window title
- `width`: `u32` - Window width
- `height`: `u32` - Window height
- `kernel`: `Arc<VantisKernel>` - Kernel reference

**Returns**: `Result<BrowserWindow>` - Browser window or error

---

```rust
pub async fn create_tab(&mut self, url: String) -> Result<()>
```
Creates a new tab.

**Parameters**:
- `url`: `String` - Initial URL

**Returns**: `Result<()>` - Success or error

---

```rust
pub async fn navigate(&mut self, url: String) -> Result<()>
```
Navigates to URL.

**Parameters**:
- `url`: `String` - URL to navigate to

**Returns**: `Result<()>` - Success or error

---

### ThemeManager

Dynamic theming system.

#### Methods

```rust
pub fn new() -> Result<Self>
```
Creates a new theme manager.

**Returns**: `Result<ThemeManager>` - Theme manager or error

---

```rust
pub fn to_css_variables(&self) -> String
```
Converts current theme to CSS variables.

**Returns**: `String` - CSS variables string

**Example**:
```rust
let css_vars = theme_manager.to_css_variables();
println!("CSS Variables:\n{}", css_vars);
```

---

## Examples

### Complete Browser Initialization

```rust
use vanisweb::core::kernel::VantisKernel;
use vanisweb::security::SecurityManager;
use vanisweb::ui::VantisUI;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize kernel
    let kernel = VantisKernel::new().await?;
    
    // Initialize security
    let security = SecurityManager::new().await?;
    
    // Initialize UI
    let ui = VantisUI::new(kernel.clone()).await?;
    
    // Run browser
    ui.run().await?;
    
    Ok(())
}
```

### Using History Manager

```rust
use vanisweb::core::history::HistoryManager;

fn main() -> anyhow::Result<()> {
    let mut history = HistoryManager::new(1000);
    
    // Add history entries
    history.add_entry(
        "https://example.com".to_string(),
        "Example Site".to_string()
    )?;
    
    history.add_entry(
        "https://vantis.ai".to_string(),
        "Vantis AI".to_string()
    )?;
    
    // Search history
    let results = history.search("example");
    println!("Found {} results", results.len());
    
    Ok(())
}
```

### Using Bookmarks

```rust
use vanisweb::core::bookmarks::BookmarkManager;

fn main() -> anyhow::Result<()> {
    let mut bookmarks = BookmarkManager::new();
    
    // Add bookmark
    bookmarks.add(
        "https://vantis.ai".to_string(),
        "Vantis AI".to_string(),
        Some("Favorites".to_string())
    )?;
    
    // Get bookmarks from folder
    let favorites = bookmarks.get_by_folder("Favorites");
    for bookmark in favorites {
        println!("{} - {}", bookmark.title, bookmark.url);
    }
    
    Ok(())
}
```

### Using Encryption

```rust
use vanisweb::security::crypto::CryptoEngine;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let crypto = CryptoEngine::new()?;
    
    // Encrypt data
    let plaintext = b"Secret message";
    let encrypted = crypto.encrypt(plaintext)?;
    
    // Decrypt data
    let decrypted = crypto.decrypt(&encrypted)?;
    
    assert_eq!(plaintext, decrypted.as_slice());
    
    Ok(())
}
```

### Using Private Mode

```rust
use vanisweb::core::private_mode::PrivateModeManager;

fn main() -> anyhow::Result<()> {
    let mut private_mode = PrivateModeManager::new();
    
    // Enable private mode
    private_mode.enable_private_mode()?;
    
    // Check if private mode is active
    if private_mode.is_private_mode() {
        println!("Private mode is active");
    }
    
    // Disable private mode
    private_mode.disable_private_mode()?;
    
    Ok(())
}
```

---

## Type Definitions

### KernelState

```rust
pub struct KernelState {
    pub initialized: bool,
    pub start_time: DateTime<Utc>,
    pub uptime_seconds: u64,
    pub active_modules: Vec<String>,
    pub system_health: SystemHealth,
}
```

### HistoryEntry

```rust
pub struct HistoryEntry {
    pub id: String,
    pub url: String,
    pub title: String,
    pub visit_time: DateTime<Utc>,
    pub visit_count: u32,
}
```

### Bookmark

```rust
pub struct Bookmark {
    pub id: String,
    pub url: String,
    pub title: String,
    pub folder: String,
    pub created_at: DateTime<Utc>,
    pub favicon: Option<String>,
}
```

### TaskPriority

```rust
pub enum TaskPriority {
    Critical = 0,   // UI rendering, user input
    High = 1,       // Web engine, networking
    Normal = 2,     // Background tasks
    Low = 3,        // Cleanup, indexing
}
```

---

## Error Handling

All functions return `Result<T>` for error handling:

```rust
use anyhow::Result;

fn example() -> Result<()> {
    // Function that may fail
    let kernel = VantisKernel::new().await?;
    Ok(())
}
```

Common error types:
- `anyhow::Error` - Generic errors
- `std::io::Error` - I/O errors
- `serde_json::Error` - Serialization errors

---

## Best Practices

1. **Always use async/await** for kernel operations
2. **Check return values** - Don't ignore errors
3. **Use Arc** for sharing kernel between modules
4. **Clean up resources** - Call shutdown when done
5. **Use private mode** for sensitive operations
6. **Encrypt sensitive data** before storage

---

## Support

- 📖 [Documentation](https://docs.vantis.ai)
- 💬 [Discord](https://discord.gg/vantis)
- 🐛 [Issues](https://github.com/vantisCorp/VantisWeb/issues)

---

*API Documentation for VantisWeb Browser v0.1.0*