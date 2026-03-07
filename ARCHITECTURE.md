# VantisWeb Architecture

This document provides a comprehensive overview of the VantisWeb browser architecture.

## Table of Contents

- [Overview](#overview)
- [Core Components](#core-components)
- [Extension System](#extension-system)
- [Security Model](#security-model)
- [Data Flow](#data-flow)
- [Performance Considerations](#performance-considerations)

## Overview

VantisWeb is a modern web browser built with Rust, designed for performance, security, and extensibility. The architecture follows a modular design with clear separation of concerns.

## Core Components

### 1. Browser Engine Integration

VantisWeb uses a web rendering engine (like Servo or WebKit) through FFI bindings.

```
src/
├── engine/          # Web engine integration
│   ├── mod.rs
│   ├── bindings.rs  # FFI bindings
│   └── context.rs   # Engine context management
```

### 2. Tab Management

```rust
// src/tabs/mod.rs
pub struct TabManager {
    tabs: Vec<Tab>,
    active_tab: Option<usize>,
}

pub struct Tab {
    id: TabId,
    url: Url,
    title: String,
    history: Vec<HistoryEntry>,
    // ...
}
```

### 3. Extension System

The extension system is a key feature of VantisWeb, supporting WebExtensions standard.

```
src/extensions/
├── mod.rs              # Main extension module
├── manifest.rs         # Manifest V3 parsing
├── storage.rs          # Storage API
├── runtime.rs          # Runtime API
├── tabs.rs             # Tabs API
├── content.rs          # Content script management
└── security.rs         # Security and permissions
```

### 4. Security Layer

```rust
// src/security/mod.rs
pub struct SecurityManager {
    permission_validator: PermissionValidator,
    threat_detector: ThreatDetector,
    csp_manager: ContentSecurityPolicyManager,
}
```

## Extension System Architecture

### Manifest V3 Support

VantisWeb implements the WebExtensions Manifest V3 standard:

```json
{
  "manifest_version": 3,
  "name": "My Extension",
  "version": "1.0.0",
  "permissions": ["storage", "tabs"],
  "background": {
    "service_worker": "background.js"
  }
}
```

### Extension Lifecycle

1. **Loading**: Parse manifest, validate permissions
2. **Activation**: Load background scripts, register APIs
3. **Runtime**: Handle events, execute scripts
4. **Deactivation**: Cleanup resources, save state

### Content Script System

Content scripts run in isolated worlds with controlled access:

```rust
pub struct ContentScript {
    files: Vec<String>,
    matches: Vec<UrlPattern>,
    run_at: RunAt,
    all_frames: bool,
}
```

## Security Model

### Permission System

VantisWeb uses a granular permission system:

- **Host Permissions**: `*://*.example.com/*`
- **API Permissions**: `storage`, `tabs`, `runtime`
- **Optional Permissions**: Requested at runtime

### Sandbox

Content scripts run in a sandboxed environment:
- Isolated JavaScript execution
- Restricted DOM access
- No native code execution

### Threat Detection

```rust
pub struct ThreatDetector {
    malware_db: MalwareDatabase,
    phishing_db: PhishingDatabase,
    behavior_analyzer: BehaviorAnalyzer,
}
```

## Data Flow

### Page Load Flow

```
User Input
    ↓
TabManager
    ↓
Engine Context
    ↓
Network Request
    ↓
Content Renderer
    ↓
Extension Injection
    ↓
Display
```

### Extension Message Flow

```
Extension (Background)
    ↓
Runtime API
    ↓
Message Router
    ↓
Content Script
    ↓
DOM
```

## Performance Considerations

### Memory Management

- Use `Arc<RwLock<>>` for shared state
- Avoid unnecessary clones
- Implement proper cleanup

### Async/Await

VantisWeb uses Tokio for async operations:

```rust
use tokio::sync::RwLock;

pub async fn load_page(&self, url: Url) -> Result<(), Error> {
    let mut tabs = self.tabs.write().await;
    // ...
}
```

### Caching

- HTTP cache for network resources
- Extension manifest cache
- Image and font caching

## Module Dependencies

```
┌─────────────────┐
│   Main Entry    │
└────────┬────────┘
         │
         ├─→ Engine
         │
         ├─→ Tabs
         │   ├─→ History
         │   └─→ Bookmarks
         │
         ├─→ Extensions
         │   ├─→ Manifest
         │   ├─→ Storage
         │   ├─→ Runtime
         │   └─→ Content
         │
         └─→ Security
             ├─→ Permissions
             └─→ CSP
```

## Testing Architecture

### Unit Tests

Test individual modules in isolation:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_manifest_parsing() {
        let manifest = parse_manifest("path/to/manifest.json").unwrap();
        assert_eq!(manifest.name, "Test Extension");
    }
}
```

### Integration Tests

Test interactions between components:

```rust
#[tokio::test]
async fn test_extension_lifecycle() {
    let manager = ExtensionManager::new().await;
    manager.load_extension("test_ext").await.unwrap();
    // Verify extension is active
}
```

## Future Enhancements

### Planned Features

1. **Multi-process Architecture**: Separate renderer processes
2. **GPU Acceleration**: Hardware acceleration for rendering
3. **Profile System**: Multiple user profiles
4. **Sync**: Cross-device synchronization
5. **Privacy Mode**: Enhanced privacy features

### Optimization Goals

- <100ms cold start time
- <50ms tab switch time
- <100MB memory usage per tab
- 60fps scrolling performance

## Contributing

When contributing to VantisWeb architecture:

1. Follow the module structure
2. Document new components
3. Add tests for new features
4. Consider performance implications
5. Maintain security best practices

For more details, see [CONTRIBUTING.md](CONTRIBUTING.md).