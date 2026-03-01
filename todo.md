# VantisWeb - Next Steps Plan

## Completed Work ✅
- WebAssembly implementation with wasmi runtime
- WASI support for system-level operations
- Complete API reference documentation
- All changes committed and pushed to GitHub

## Next Task: Extensions System

### Overview
Implement a basic extensions system to allow users to add functionality to the browser through plugins/add-ons.

### Implementation Plan

#### 1. Core Extension Infrastructure
- [ ] Create `src/extensions/mod.rs` - Extension module
- [ ] Create `src/extensions/extension.rs` - Extension trait and base types
- [ ] Create `src/extensions/manager.rs` - Extension manager
- [ ] Create `src/extensions/registry.rs` - Extension registry
- [ ] Create `src/extensions/loader.rs` - Extension loader

#### 2. Extension Types
- [ ] Define Extension trait with lifecycle methods
- [ ] Implement ContentScript extensions
- [ ] Implement BackgroundScript extensions
- [ ] Implement Popup extensions
- [ ] Implement Theme extensions

#### 3. Extension Manifest
- [ ] Define manifest format (JSON)
- [ ] Create manifest parser
- [ ] Validate manifest structure
- [ ] Handle version compatibility

#### 4. Extension Loading
- [ ] Load extensions from directory
- [ ] Validate extension permissions
- [ ] Initialize extension contexts
- [ ] Handle extension errors

#### 5. Extension API
- [ ] Browser API for extensions
- [ ] Storage API for extensions
- [ ] Messaging API (extension ↔ browser)
- [ ] Tab API for tab manipulation
- [ ] Request API for network access

#### 6. Security & Sandboxing
- [ ] Permission system
- [ ] Content Security Policy
- [ ] Isolated execution contexts
- [ ] Resource limits

#### 7. UI Integration
- [ ] Extensions settings page
- [ ] Extension manager UI
- [ ] Enable/disable extensions
- [ ] Extension permissions dialog

#### 8. Testing
- [ ] Unit tests for extension manager
- [ ] Integration tests for extension loading
- [ ] Test extension examples
- [ ] Security tests

#### 9. Documentation
- [ ] Extension development guide
- [ ] API reference for extension developers
- [ ] Example extensions
- [ ] Manifest documentation

### File Structure
```
src/extensions/
├── mod.rs              # Module exports
├── extension.rs        # Extension trait and types
├── manager.rs          # Extension manager
├── registry.rs         # Extension registry
├── loader.rs           # Extension loader
├── manifest.rs         # Manifest parser
├── api/
│   ├── mod.rs          # API exports
│   ├── browser.rs      # Browser API
│   ├── storage.rs      # Storage API
│   ├── messaging.rs    # Messaging API
│   ├── tabs.rs         # Tab API
│   └── requests.rs     # Request API
└── types.rs            # Common types

extensions/
├── example-extension/  # Example extension
│   ├── manifest.json
│   ├── background.js
│   ├── content.js
│   └── popup.html
└── README.md
```

### Dependencies to Add
- serde_json (already present)
- Additional validation libraries if needed

### Estimated Complexity
- Medium complexity
- ~1500-2000 lines of code
- 2-3 days of work

### Success Criteria
- [ ] Extensions can be loaded from manifest
- [ ] Extensions can interact with browser APIs
- [ ] Extensions can be enabled/disabled
- [ ] Security sandboxing is in place
- [ ] Documentation is complete
- [ ] Example extension works

## Future Tasks (After Extensions)
1. Profile Management enhancements
2. Code optimization and polishing
3. Additional Web Engine features
4. Advanced security features
5. AI features integration