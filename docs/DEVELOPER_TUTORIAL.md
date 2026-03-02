# VantisWeb Browser - Developer Tutorial

## Introduction

Welcome to the VantisWeb developer tutorial! This guide will help you get started with developing for VantisWeb, whether you want to contribute to the core browser, create extensions, or build custom profiles.

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Understanding the Architecture](#understanding-the-architecture)
3. [Building from Source](#building-from-source)
4. [Running Tests](#running-tests)
5. [Creating Extensions](#creating-extensions)
6. [Creating Profile Templates](#creating-profile-templates)
7. [Contributing Guidelines](#contributing-guidelines)
8. [Best Practices](#best-practices)

---

## Getting Started

### Prerequisites

Before you start developing for VantisWeb, ensure you have:

- **Rust 1.75+**: Install from [rustup.rs](https://rustup.rs/)
- **Cargo**: Comes with Rust
- **Git**: For version control
- **System Dependencies**:
  - Linux: `webkit2gtk` development libraries
  - Windows: WebView2 Runtime
  - macOS: Cocoa framework

### Setting Up Your Development Environment

```bash
# Clone the repository
git clone https://github.com/vantisCorp/VantisWeb.git
cd VantisWeb

# Install Rust dependencies
cargo build

# Run the browser
cargo run
```

---

## Understanding the Architecture

### Liquid Core Architecture

VantisWeb uses a revolutionary **Liquid Core Architecture**:

```
┌─────────────────────────────────────────┐
│           Vantis Kernel                 │
│  (Central Orchestration & Management)   │
└──────────────┬──────────────────────────┘
               │
       ┌───────┴───────┐
       │               │
┌──────▼──────┐  ┌────▼─────┐
│   Core      │  │ Security │
│  Modules    │  │  System  │
└──────┬──────┘  └────┬─────┘
       │              │
       └──────┬───────┘
              │
       ┌──────▼──────────┐
       │  Web Engine     │
       │  (WebKitGTK)    │
       └──────┬──────────┘
              │
       ┌──────▼──────────┐
       │     UI Layer    │
       │  (VantisUI)     │
       └─────────────────┘
```

### Key Components

1. **Vantis Kernel**: Central orchestration and management
2. **Micro-Scheduler**: Intelligent CPU thread management
3. **Web Engine**: WebKitGTK integration for rendering
4. **Security System**: Post-quantum cryptography and sandboxing
5. **UI Layer**: WebGPU-based rendering with 144Hz+ support
6. **Extensions System**: Modular extension architecture
7. **Profile Management**: Multiple profiles with templates and sync

---

## Building from Source

### Debug Build

```bash
cargo build
```

### Release Build

```bash
cargo build --release
```

### Running with Debug Logging

```bash
RUST_LOG=debug cargo run
```

### Running Specific Tests

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test core::kernel
cargo test extensions::manager
cargo test profiles::templates

# Run integration tests
cargo test --test integration_test_core
```

---

## Running Tests

### Unit Tests

Unit tests are located in individual module files:

```bash
# Run all unit tests
cargo test --lib

# Run specific test
cargo test test_kernel_initialization
```

### Integration Tests

Integration tests are in the `tests/` directory:

```bash
# Run all integration tests
cargo test --test integration_test_core
cargo test --test integration_test_extensions
cargo test --test integration_test_profiles
cargo test --test integration_test_web_ui

# Run all integration tests at once
./scripts/run_all_tests.sh
```

### Performance Benchmarks

```bash
# Run all benchmarks
./scripts/run_benchmarks.sh

# Run specific benchmark suite
cargo bench --bench core_bench
cargo bench --bench extensions_bench
cargo bench --bench profiles_bench
cargo bench --bench web_bench
```

---

## Creating Extensions

### Extension Structure

```
my-extension/
├── manifest.json          # Extension configuration
├── background.js          # Background script
├── content.js             # Content script
├── popup.html             # Popup UI
├── popup.js               # Popup script
└── icons/                 # Extension icons
    ├── icon16.png
    ├── icon48.png
    └── icon128.png
```

### Manifest File

```json
{
  "name": "My Extension",
  "version": "1.0.0",
  "description": "A sample extension for VantisWeb",
  "author": "Your Name",
  "permissions": [
    "browser",
    "storage",
    "tabs",
    "messaging"
  ],
  "background": {
    "scripts": ["background.js"]
  },
  "content_scripts": [
    {
      "matches": ["<all_urls>"],
      "js": ["content.js"]
    }
  ],
  "browser_action": {
    "default_popup": "popup.html",
    "default_icon": {
      "16": "icons/icon16.png",
      "48": "icons/icon48.png",
      "128": "icons/icon128.png"
    }
  }
}
```

### Background Script

```javascript
// background.js
// Listen for extension installation
browser.runtime.onInstalled.addListener(() => {
  console.log("Extension installed!");
});

// Listen for messages from content scripts
browser.runtime.onMessage.addListener((message, sender, sendResponse) => {
  console.log("Received message:", message);
  sendResponse({ status: "ok" });
});

// Use storage API
browser.storage.local.set({ counter: 0 });

// Use tabs API
browser.tabs.onActivated.addListener((activeInfo) => {
  console.log("Tab activated:", activeInfo.tabId);
});
```

### Content Script

```javascript
// content.js
// Inject content into pages
console.log("Content script loaded!");

// Send message to background script
browser.runtime.sendMessage({ action: "hello" });

// Listen for messages from background script
browser.runtime.onMessage.addListener((message, sender, sendResponse) => {
  console.log("Received message:", message);
});
```

### Loading Extensions

```rust
use vantisweb::extensions::{ExtensionManager, ExtensionLoader};

// Create extension manager
let mut manager = ExtensionManager::new();

// Create extension loader
let loader = ExtensionLoader::new("/path/to/extensions");

// Load all extensions
loader.load_all(&mut manager).await.unwrap();

// Enable specific extension
manager.enable_extension("my-extension").await.unwrap();
```

---

## Creating Profile Templates

### Template Structure

Profile templates are JSON files with predefined settings:

```json
{
  "name": "My Template",
  "description": "A custom profile template",
  "category": "Custom",
  "settings": {
    "search_engine": "https://www.google.com/search?q=",
    "homepage": "https://www.google.com",
    "start_pages": [
      "https://www.google.com",
      "https://github.com"
    ],
    "keyboard_shortcuts": {
      "Ctrl+T": "new_tab",
      "Ctrl+W": "close_tab"
    },
    "privacy": {
      "block_trackers": true,
      "block_ads": true,
      "clear_cookies_on_exit": false,
      "clear_history_on_exit": false,
      "private_mode_by_default": false,
      "disable_javascript": false
    },
    "performance": {
      "hardware_acceleration": true,
      "memory_limit": 4096,
      "cpu_priority": "Normal",
      "cache_size": 512
    }
  },
  "extensions": [
    "adblocker",
    "password-manager"
  ],
  "theme": "dark"
}
```

### Creating a Profile from Template

```rust
use vantisweb::profiles::{ProfileManager, TemplateManager};

// Create profile manager
let manager = ProfileManager::new(kernel).await.unwrap();

// Create template manager
let template_manager = TemplateManager::new();

// Get template
let template = template_manager.get_template("My Template").unwrap();

// Create profile from template
let profile = ProfileConfig::new("My Profile", ProfileType::Custom("My Template".to_string()));

// Apply template settings
profile.settings = template.settings.clone();
profile.extensions = template.extensions.clone();
profile.theme = template.theme.clone();

// Save profile
manager.create_profile(profile).await.unwrap();
```

---

## Contributing Guidelines

### Code Style

- Follow Rust naming conventions
- Use `cargo fmt` for formatting
- Run `cargo clippy` for linting
- Add tests for new features
- Update documentation

### Commit Messages

Use conventional commit messages:

```
feat: add new feature
fix: fix bug
docs: update documentation
test: add tests
refactor: refactor code
perf: performance improvement
```

### Pull Request Process

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Update documentation
6. Run tests: `./scripts/run_all_tests.sh`
7. Run benchmarks: `./scripts/run_benchmarks.sh`
8. Commit your changes
9. Push to your fork
10. Open a Pull Request

### Code Review Checklist

- [ ] Code follows Rust style guidelines
- [ ] All tests pass
- [ ] New features have tests
- [ ] Documentation is updated
- [ ] No compiler warnings
- [ ] No clippy warnings
- [ ] Performance benchmarks pass

---

## Best Practices

### Performance

1. **Use Arc for Shared Ownership**
   ```rust
   // Good
   let data = Arc::new(MyData::new());
   
   // Avoid
   let data = MyData::new();
   let data_clone = data.clone();
   ```

2. **Pre-allocate Collection Capacities**
   ```rust
   // Good
   let mut vec = Vec::with_capacity(100);
   
   // Avoid
   let mut vec = Vec::new();
   ```

3. **Use References Instead of Clones**
   ```rust
   // Good
   fn get_data(&self) -> &Data {
       &self.data
   }
   
   // Avoid
   fn get_data(&self) -> Data {
       self.data.clone()
   }
   ```

### Security

1. **Validate All Inputs**
   ```rust
   fn validate_url(url: &str) -> Result<String> {
       if url.is_empty() {
           return Err(anyhow!("URL cannot be empty"));
       }
       // ... validation logic
   }
   ```

2. **Use Secure Cryptography**
   ```rust
   use vantisweb::security::CryptoEngine;
   
   let crypto = CryptoEngine::new()?;
   let hash = crypto.hash(password)?;
   ```

3. **Sanitize User Input**
   ```rust
   // Always sanitize user input before use
   let sanitized = html_escape::encode_text(&user_input);
   ```

### Testing

1. **Write Tests First** (TDD)
2. **Test Edge Cases**
3. **Use Descriptive Test Names**
4. **Keep Tests Fast**
5. **Mock External Dependencies**

### Documentation

1. **Document Public APIs**
2. **Add Examples**
3. **Keep Documentation Up-to-Date**
4. **Use Clear Language**

---

## Resources

- [API Reference](API_REFERENCE.md)
- [Optimization Guide](OPTIMIZATION.md)
- [Testing Guide](TESTING_GUIDE.md)
- [Extensions Guide](EXTENSIONS.md)
- [Profiles Guide](../profiles/README.md)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Documentation](https://doc.rust-lang.org/std/)

---

## Getting Help

- 📖 [Documentation](https://docs.vantis.ai)
- 💬 [Discord](https://discord.gg/vantis)
- 🐛 [Issue Tracker](https://github.com/vantisCorp/VantisWeb/issues)
- 📧 Email: support@vantis.ai

---

**Happy Coding! 🚀**

---

**Last Updated:** March 2, 2025  
**Version:** 1.0.0