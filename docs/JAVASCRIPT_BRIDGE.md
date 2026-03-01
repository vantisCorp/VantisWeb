# JavaScript Bridge Implementation

## Overview

The JavaScript Bridge is a critical component of VantisWeb that enables JavaScript code running in web pages to interact with Rust-based Web APIs. This bidirectional bridge allows web applications to use native browser functionality implemented in Rust.

## Architecture

### Components

1. **JsContext** (`src/engine/js_bridge.rs`)
   - Manages JavaScript function and object registration
   - Provides function and method calling capabilities
   - Maintains a registry of callable JavaScript functions

2. **JsBridge** (`src/engine/js_bridge.rs`)
   - Main bridge component integrating all Web APIs
   - Registers Web APIs with the JavaScript context
   - Handles JavaScript-to-Rust and Rust-to-JavaScript communication

3. **WebRenderer Integration**
   - JsBridge is integrated into WebRenderer
   - All Web APIs are accessible through the renderer
   - Provides getter methods for each API

## Implemented Web APIs

### 1. Console API

**JavaScript Object:** `console`

**Methods:**
- `console.log(...args)` - Log informational messages
- `console.warn(...args)` - Log warning messages
- `console.error(...args)` - Log error messages
- `console.info(...args)` - Log info messages
- `console.debug(...args)` - Log debug messages

**Usage:**
```javascript
console.log("Hello, VantisWeb!");
console.warn("This is a warning");
console.error("This is an error");
```

### 2. Storage API

**JavaScript Objects:** `localStorage`, `sessionStorage`

**Methods:**
- `getItem(key)` - Retrieve a value by key
- `setItem(key, value)` - Store a key-value pair
- `removeItem(key)` - Remove a value by key
- `clear()` - Clear all storage

**Usage:**
```javascript
// Local Storage
localStorage.setItem("username", "vantisuser");
const username = localStorage.getItem("username");
localStorage.removeItem("username");
localStorage.clear();

// Session Storage
sessionStorage.setItem("sessionToken", "abc123");
const token = sessionStorage.getItem("sessionToken");
```

### 3. Fetch API

**JavaScript Function:** `fetch(url, options)`

**Features:**
- HTTP GET requests
- JSON response parsing
- Error handling
- Custom URL support

**Usage:**
```javascript
fetch("https://api.example.com/data")
  .then(response => response.json())
  .then(data => console.log(data))
  .catch(error => console.error("Error:", error));
```

### 4. Event Loop API

**JavaScript Functions:**
- `setTimeout(callback, delay)` - Execute callback after delay
- `setInterval(callback, interval)` - Execute callback repeatedly
- `clearTimeout(id)` - Cancel a timeout
- `clearInterval(id)` - Cancel an interval

**Usage:**
```javascript
// Timeout
const timeoutId = setTimeout(() => {
  console.log("Executed after 1 second");
}, 1000);

clearTimeout(timeoutId);

// Interval
const intervalId = setInterval(() => {
  console.log("Executed every 500ms");
}, 500);

clearInterval(intervalId);
```

## Technical Implementation

### Async Function Type

The bridge uses an async function type to handle JavaScript callbacks:

```rust
pub type JsFunction = Box<dyn Fn(Vec<serde_json::Value>) -> Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>> + Send + Sync>;
```

This allows:
- Asynchronous execution of JavaScript callbacks
- Type-safe argument passing
- Error handling through Result types
- Thread-safe operations

### Function Registration

Functions are registered using the `JsContext`:

```rust
context.register_function(
    "fetch".to_string(),
    Box::new(move |args: Vec<serde_json::Value>| {
        // Implementation
        Box::pin(async move {
            // Async logic
            Ok(serde_json::json!(result))
        }) as Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>>
    }) as JsFunction,
).await;
```

### Object Registration

Objects with methods are registered similarly:

```rust
let mut methods = HashMap::new();
methods.insert(
    "getItem".to_string(),
    Box::new(move |args: Vec<serde_json::Value>| {
        // Implementation
    }) as JsFunction,
);

context.register_object("localStorage".to_string(), methods).await;
```

## Integration with WebRenderer

The JsBridge is integrated into WebRenderer:

```rust
pub struct WebRenderer {
    // ... other fields
    js_bridge: Arc<JsBridge>,
}

impl WebRenderer {
    pub fn get_js_bridge(&self) -> Arc<JsBridge> {
        self.js_bridge.clone()
    }
}
```

## Testing

A comprehensive test suite is provided in `tests/web_apis_test.html`:

1. **Console API Tests** - Validates all console methods
2. **Storage API Tests** - Tests localStorage and sessionStorage
3. **Fetch API Tests** - Tests HTTP requests and JSON parsing
4. **Event Loop Tests** - Tests timer functions

### Running Tests

1. Build and run VantisWeb browser
2. Open `tests/web_apis_test.html` in the browser
3. Click "Run All Tests" or test individual APIs
4. Review results in the test sections

## Performance Considerations

### Current Limitations

1. **Fetch API**: Uses placeholder implementation. Full HTTP client integration needed.
2. **Event Loop**: Timer callbacks are not fully integrated with JavaScript execution.
3. **Storage**: Data is stored in memory and lost on browser restart.

### Future Optimizations

1. Implement persistent storage with disk I/O
2. Add request/response caching for Fetch API
3. Optimize timer scheduling for better performance
4. Add Web Workers support for parallel execution

## Security Considerations

### Current Implementation

- All JavaScript code runs in the same process
- No sandboxing for JavaScript execution
- Storage is not encrypted

### Future Enhancements

1. Implement JavaScript sandboxing
2. Add content security policy (CSP) support
3. Encrypt sensitive storage data
4. Add CORS enforcement for Fetch API

## Error Handling

The bridge provides comprehensive error handling:

```rust
try {
    const result = await someApi();
    console.log(result);
} catch (error) {
    console.error("Error:", error.message);
}
```

All errors are propagated through the Result type and converted to JavaScript exceptions.

## Future Enhancements

### Planned Features

1. **WebSocket API** - Real-time communication
2. **Web Workers API** - Parallel execution
3. **IndexedDB API** - Structured storage
4. **Service Worker API** - Offline support
5. **Geolocation API** - Location services
6. **Notification API** - Push notifications
7. **Clipboard API** - Clipboard access
8. **File System API** - File operations

### API Extensions

1. Add more HTTP methods (POST, PUT, DELETE, etc.)
2. Add request/response interceptors
3. Add streaming support for Fetch API
4. Add storage quota management
5. Add timer precision control

## Contributing

When adding new Web APIs:

1. Implement the API in Rust
2. Register it in the JavaScript Bridge
3. Add corresponding tests to `tests/web_apis_test.html`
4. Update this documentation
5. Run all tests to ensure no regressions

## References

- [MDN Web Docs - Console API](https://developer.mozilla.org/en-US/docs/Web/API/Console)
- [MDN Web Docs - Storage API](https://developer.mozilla.org/en-US/docs/Web/API/Storage)
- [MDN Web Docs - Fetch API](https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API)
- [MDN Web Docs - Window Timers](https://developer.mozilla.org/en-US/docs/Web/API/WindowOrWorkerGlobalScope/setTimeout)

## License

MIT License - See LICENSE file for details