# WebAssembly Support Documentation

## Overview

VantisWeb provides comprehensive WebAssembly (WASM) support using the `wasmi` runtime, a lightweight WebAssembly interpreter optimized for embedding in browsers and other applications.

## Architecture

### Components

1. **WasmRuntime** - Main runtime component that manages WASM modules
2. **WasmModule** - Represents a loaded WebAssembly module instance
3. **WasmFunction** - Represents an exported function from a WASM module
4. **WasmMemory** - Represents WebAssembly linear memory
5. **WasmRuntimeStats** - Runtime statistics and metrics

### Integration

The WebAssembly runtime is integrated with:
- **WebRenderer** - WASM runtime is available in each renderer instance
- **JavaScript Bridge** - JavaScript can interact with WASM modules
- **Storage API** - WASM modules can access browser storage
- **Fetch API** - WASM modules can make HTTP requests

## Features

### Core Features

✅ **Module Loading**
- Load WASM modules from byte arrays
- Automatic module validation
- Unique module ID generation

✅ **Function Execution**
- Execute exported functions
- Support for multiple parameter types
- Return value handling

✅ **Memory Management**
- Read/write access to WASM memory
- Memory size tracking
- Memory export/import support

✅ **WASI Support**
- WebAssembly System Interface (WASI) integration
- Standard I/O support
- File system access (sandboxed)

✅ **Module Management**
- List loaded modules
- Get module by ID
- Unload modules
- Get exported functions

### Advanced Features

✅ **Type Safety**
- Strong typing with Rust's type system
- Compile-time type checking
- Safe memory access

✅ **Error Handling**
- Comprehensive error reporting
- Graceful failure handling
- Detailed error messages

✅ **Performance**
- Efficient execution
- Minimal overhead
- Optimized memory usage

## Usage

### Basic Usage

```rust
use vantisweb::engine::wasm::WasmRuntime;

// Create a new runtime
let mut runtime = WasmRuntime::new()?;

// Load a WASM module
let wasm_bytes = std::fs::read("module.wasm")?;
let module_id = runtime.load_module(wasm_bytes)?;

// Execute a function
let args = vec![Value::I32(5), Value::I32(3)];
let result = runtime.execute_function(&module_id, "add", args)?;

// Get module info
let module = runtime.get_module(&module_id)?;
let exports = runtime.get_exports(&module_id)?;

// Unload module
runtime.unload_module(&module_id)?;
```

### Memory Operations

```rust
// Read memory
let data = runtime.read_memory(&module_id, offset, length)?;

// Write memory
runtime.write_memory(&module_id, offset, &data)?;

// Get memory info
let memory = runtime.get_memory(&module_id)?;
println!("Memory size: {} bytes", memory.size);
```

### Integration with WebRenderer

```rust
use vantisweb::engine::web_renderer::WebRenderer;

let renderer = WebRenderer::new(kernel)?;

// Get WASM runtime
let wasm_runtime = renderer.get_wasm_runtime();

// Load and execute modules
let module_id = wasm_runtime.load_module(wasm_bytes)?;
let result = wasm_runtime.execute_function(&module_id, "main", vec![])?;
```

## JavaScript Integration

WebAssembly modules can be accessed from JavaScript through the JavaScript Bridge:

```javascript
// Load WASM module
const wasmBytes = await fetch('module.wasm').then(r => r.arrayBuffer());
const module = await WebAssembly.instantiate(wasmBytes);

// Execute function
const result = module.exports.add(5, 3);
console.log(result); // 8
```

## WASI Support

VantisWeb supports the WebAssembly System Interface (WASI) for system-level operations:

```rust
// WASI context is automatically created
let wasi_ctx = WasiCtxBuilder::new()
    .inherit_stdio()
    .build();

// Modules can access standard I/O
// File system access is sandboxed
```

## Testing

### Unit Tests

The WASM implementation includes comprehensive unit tests:

```bash
cargo test wasm
```

### Integration Tests

A test suite is available at `tests/wasm_test.html`:

```bash
# Open in browser
open tests/wasm_test.html

# Or serve with HTTP server
python -m http.server 8000
# Navigate to http://localhost:8000/tests/wasm_test.html
```

### Test Coverage

- ✅ Runtime creation
- ✅ Module loading
- ✅ Function execution
- ✅ Memory operations
- ✅ Module management
- ✅ Error handling

## Performance Considerations

### Optimization Tips

1. **Reuse Runtime Instances** - Create one runtime and reuse it
2. **Cache Modules** - Keep frequently used modules loaded
3. **Batch Operations** - Group memory operations together
4. **Use Appropriate Types** - Choose the right WASM types for your data

### Benchmarks

- Module loading: ~1-10ms (depending on size)
- Function execution: ~0.1-1ms (simple operations)
- Memory read/write: ~0.01-0.1ms per KB

## Security

### Sandboxing

- WASM modules run in a sandboxed environment
- Memory access is isolated
- No direct access to host system

### WASI Sandboxing

- File system access is restricted
- Network access is controlled
- Standard I/O is inherited

### Best Practices

1. **Validate Input** - Always validate WASM module bytes
2. **Limit Resources** - Set memory and execution time limits
3. **Use HTTPS** - Load WASM modules from secure sources
4. **Audit Modules** - Review WASM code before loading

## Limitations

### Current Limitations

- No multi-threading support
- Limited SIMD support
- No direct DOM access from WASM
- No WebGPU integration yet

### Future Enhancements

- [ ] Multi-threading support
- [ ] SIMD optimizations
- [ ] WebGPU integration
- [ ] Direct DOM access
- [ ] Shared memory support

## Troubleshooting

### Common Issues

**Module fails to load**
- Check WASM module validity
- Verify module format
- Check for missing dependencies

**Function execution fails**
- Verify function name
- Check parameter types
- Ensure function is exported

**Memory access errors**
- Check memory bounds
- Verify offset and length
- Ensure memory is exported

### Debugging

Enable debug logging:

```rust
env_logger::init();
```

Check runtime statistics:

```rust
let stats = runtime.get_stats();
println!("Modules: {}", stats.module_count);
println!("Total memory: {} bytes", stats.total_memory);
```

## API Reference

### WasmRuntime

```rust
pub struct WasmRuntime {
    id: String,
    modules: HashMap<String, WasmModule>,
    engine: Engine,
}

impl WasmRuntime {
    pub fn new() -> Result<Self>;
    pub fn load_module(&mut self, wasm_bytes: Vec<u8>) -> Result<String>;
    pub fn get_module(&self, module_id: &str) -> Option<&WasmModule>;
    pub fn execute_function(&mut self, module_id: &str, function_name: &str, args: Vec<Value>) -> Result<Vec<Value>>;
    pub fn get_memory(&self, module_id: &str) -> Result<WasmMemory>;
    pub fn read_memory(&self, module_id: &str, offset: usize, length: usize) -> Result<Vec<u8>>;
    pub fn write_memory(&mut self, module_id: &str, offset: usize, data: &[u8]) -> Result<()>;
    pub fn get_exports(&self, module_id: &str) -> Result<Vec<String>>;
    pub fn unload_module(&mut self, module_id: &str) -> Result<()>;
    pub fn get_stats(&self) -> WasmRuntimeStats;
}
```

### WasmModule

```rust
pub struct WasmModule {
    pub id: String,
    pub instance: Instance,
    pub store: Arc<Mutex<Store<WasiCtx>>>,
}
```

### WasmRuntimeStats

```rust
pub struct WasmRuntimeStats {
    pub id: String,
    pub module_count: usize,
    pub total_memory: usize,
}
```

## Contributing

To contribute to the WebAssembly implementation:

1. Follow Rust coding standards
2. Add tests for new features
3. Update documentation
4. Ensure all tests pass
5. Submit pull requests

## Resources

- [WebAssembly Specification](https://webassembly.github.io/spec/)
- [wasmi Documentation](https://docs.rs/wasmi/)
- [WASI Specification](https://wasi.dev/)
- [WebAssembly on MDN](https://developer.mozilla.org/en-US/docs/WebAssembly)

## License

This implementation is part of VantisWeb and follows the same license (MIT).
