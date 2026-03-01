//! WebAssembly Support
//!
//! WebAssembly runtime integration using wasmi:
//! - WASM runtime with wasmi
//! - WASI support
//! - Memory management
//! - Module loading and execution
//! - JavaScript interop

use anyhow::{anyhow, Result};
use log::{debug, info, warn};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use wasmi::*;
use wasmi_wasi::{WasiCtx, WasiCtxBuilder};

/// WebAssembly module instance
#[derive(Clone)]
pub struct WasmModule {
    pub id: String,
    pub instance: Instance,
    pub store: Arc<Mutex<Store<WasiCtx>>>,
}

/// WebAssembly function export
#[derive(Clone)]
pub struct WasmFunction {
    pub name: String,
    pub module_id: String,
}

/// WebAssembly memory
#[derive(Clone)]
pub struct WasmMemory {
    pub module_id: String,
    pub size: usize,
}

/// WebAssembly Runtime
pub struct WasmRuntime {
    id: String,
    modules: HashMap<String, WasmModule>,
    engine: Engine,
}

impl WasmRuntime {
    /// Create a new WebAssembly runtime
    pub fn new() -> Result<Self> {
        info!("Initializing WebAssembly Runtime with wasmi...");
        
        let engine = Engine::default();
        
        Ok(Self {
            id: uuid::Uuid::new_v4().to_string(),
            modules: HashMap::new(),
            engine,
        })
    }
    
    /// Load a WebAssembly module from bytes
    pub fn load_module(&mut self, wasm_bytes: Vec<u8>) -> Result<String> {
        info!("Loading WebAssembly module ({} bytes)", wasm_bytes.len());
        
        // Create WASI context
        let wasi_ctx = WasiCtxBuilder::new()
            .inherit_stdio()
            .build();
        
        // Create store with WASI context
        let mut store = Store::new(&self.engine, wasi_ctx);
        
        // Parse the module
        let module = Module::new(&self.engine, &wasm_bytes)
            .map_err(|e| anyhow!("Failed to parse WASM module: {}", e))?;
        
        // Create linker for WASI support
        let mut linker = Linker::new(&self.engine);
        wasmi_wasi::add_to_linker(&mut linker, |ctx| ctx)?;
        
        // Instantiate the module
        let instance = linker
            .instantiate(&mut store, &module)?
            .start(&mut store)?;
        
        let module_id = uuid::Uuid::new_v4().to_string();
        
        let wasm_module = WasmModule {
            id: module_id.clone(),
            instance,
            store: Arc::new(Mutex::new(store)),
        };
        
        self.modules.insert(module_id.clone(), wasm_module);
        
        info!("✓ WebAssembly module loaded: {}", module_id);
        
        Ok(module_id)
    }
    
    /// Get module by ID
    pub fn get_module(&self, module_id: &str) -> Option<&WasmModule> {
        self.modules.get(module_id)
    }
    
    /// Execute function in WebAssembly module
    pub fn execute_function(
        &mut self,
        module_id: &str,
        function_name: &str,
        args: Vec<Value>,
    ) -> Result<Vec<Value>> {
        debug!("Executing WASM function: {}::{}", module_id, function_name);
        
        let module = self.modules.get(module_id)
            .ok_or_else(|| anyhow!("Module not found: {}", module_id))?;
        
        let mut store = module.store.lock()
            .map_err(|e| anyhow!("Failed to lock store: {}", e))?;
        
        // Get the function export
        let func = module.instance
            .get_typed_func::<(i32, i32), i32>(&mut *store, function_name)
            .or_else(|_| {
                module.instance.get_typed_func::<(), i32>(&mut *store, function_name)
            });
        
        match func {
            Ok(f) => {
                // Execute based on function signature
                let result = if args.len() == 2 {
                    let a = args[0].unwrap_i32();
                    let b = args[1].unwrap_i32();
                    f.call(&mut *store, (a, b)).map(|v| vec![Value::I32(v)])?
                } else {
                    f.call(&mut *store, ()).map(|v| vec![Value::I32(v)])?
                };
                
                Ok(result)
            }
            Err(e) => {
                // Try generic function call
                let func = module.instance
                    .get_func(&mut *store, function_name)
                    .ok_or_else(|| anyhow!("Function not found: {}", function_name))?;
                
                let result = func.call(&mut *store, &args)
                    .map_err(|e| anyhow!("Function call failed: {}", e))?;
                
                Ok(result)
            }
        }
    }
    
    /// Get memory from module
    pub fn get_memory(&self, module_id: &str) -> Result<WasmMemory> {
        let module = self.modules.get(module_id)
            .ok_or_else(|| anyhow!("Module not found: {}", module_id))?;
        
        let store = module.store.lock()
            .map_err(|e| anyhow!("Failed to lock store: {}", e))?;
        
        let memory = module.instance
            .get_memory(&*store, "memory")
            .ok_or_else(|| anyhow!("Memory export not found"))?;
        
        let size = memory.size(&*store) * 65536; // Convert pages to bytes
        
        Ok(WasmMemory {
            module_id: module_id.to_string(),
            size,
        })
    }
    
    /// Read memory from module
    pub fn read_memory(&self, module_id: &str, offset: usize, length: usize) -> Result<Vec<u8>> {
        let module = self.modules.get(module_id)
            .ok_or_else(|| anyhow!("Module not found: {}", module_id))?;
        
        let mut store = module.store.lock()
            .map_err(|e| anyhow!("Failed to lock store: {}", e))?;
        
        let memory = module.instance
            .get_memory(&mut *store, "memory")
            .ok_or_else(|| anyhow!("Memory export not found"))?;
        
        let mut buffer = vec![0u8; length];
        memory.read(&mut *store, offset, &mut buffer)?;
        
        Ok(buffer)
    }
    
    /// Write memory to module
    pub fn write_memory(&mut self, module_id: &str, offset: usize, data: &[u8]) -> Result<()> {
        let module = self.modules.get(module_id)
            .ok_or_else(|| anyhow!("Module not found: {}", module_id))?;
        
        let mut store = module.store.lock()
            .map_err(|e| anyhow!("Failed to lock store: {}", e))?;
        
        let memory = module.instance
            .get_memory(&mut *store, "memory")
            .ok_or_else(|| anyhow!("Memory export not found"))?;
        
        memory.write(&mut *store, offset, data)?;
        
        Ok(())
    }
    
    /// Get all exported functions from module
    pub fn get_exports(&self, module_id: &str) -> Result<Vec<String>> {
        let module = self.modules.get(module_id)
            .ok_or_else(|| anyhow!("Module not found: {}", module_id))?;
        
        let store = module.store.lock()
            .map_err(|e| anyhow!("Failed to lock store: {}", e))?;
        
        let mut exports = Vec::new();
        
        for export in module.instance.exports(&*store) {
            if let Extern::Func(_) = export.into_extern() {
                exports.push(export.name().to_string());
            }
        }
        
        Ok(exports)
    }
    
    /// Unload a module
    pub fn unload_module(&mut self, module_id: &str) -> Result<()> {
        self.modules.remove(module_id)
            .ok_or_else(|| anyhow!("Module not found: {}", module_id))?;
        
        info!("✓ WebAssembly module unloaded: {}", module_id);
        
        Ok(())
    }
    
    /// Get runtime statistics
    pub fn get_stats(&self) -> WasmRuntimeStats {
        WasmRuntimeStats {
            id: self.id.clone(),
            module_count: self.modules.len(),
            total_memory: self.modules.values()
                .map(|m| {
                    let store = m.store.lock().unwrap();
                    m.instance.get_memory(&*store, "memory")
                        .map(|mem| mem.size(&*store) * 65536)
                        .unwrap_or(0)
                })
                .sum(),
        }
    }
}

impl Default for WasmRuntime {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| panic!("Failed to create WASM runtime"))
    }
}

/// WebAssembly runtime statistics
#[derive(Clone, Debug)]
pub struct WasmRuntimeStats {
    pub id: String,
    pub module_count: usize,
    pub total_memory: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Simple WASM module that adds two numbers
    const ADD_WASM: &[u8] = &[
        0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x07, 0x01,
        0x60, 0x02, 0x7f, 0x7f, 0x01, 0x7f, 0x03, 0x02, 0x01, 0x00, 0x07,
        0x07, 0x01, 0x03, 0x61, 0x64, 0x64, 0x00, 0x00, 0x0a, 0x09, 0x01,
        0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0x6a, 0x0b,
    ];

    #[test]
    fn test_wasm_runtime_creation() {
        let runtime = WasmRuntime::new();
        assert!(runtime.is_ok());
    }

    #[test]
    fn test_load_module() {
        let mut runtime = WasmRuntime::new().unwrap();
        let result = runtime.load_module(ADD_WASM.to_vec());
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_module() {
        let mut runtime = WasmRuntime::new().unwrap();
        let module_id = runtime.load_module(ADD_WASM.to_vec()).unwrap();
        let module = runtime.get_module(&module_id);
        assert!(module.is_some());
    }

    #[test]
    fn test_execute_function() {
        let mut runtime = WasmRuntime::new().unwrap();
        let module_id = runtime.load_module(ADD_WASM.to_vec()).unwrap();
        
        let args = vec![Value::I32(5), Value::I32(3)];
        let result = runtime.execute_function(&module_id, "add", args);
        
        assert!(result.is_ok());
        let values = result.unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].unwrap_i32(), 8);
    }

    #[test]
    fn test_get_exports() {
        let mut runtime = WasmRuntime::new().unwrap();
        let module_id = runtime.load_module(ADD_WASM.to_vec()).unwrap();
        
        let exports = runtime.get_exports(&module_id);
        assert!(exports.is_ok());
        assert!(exports.unwrap().contains(&"add".to_string()));
    }

    #[test]
    fn test_unload_module() {
        let mut runtime = WasmRuntime::new().unwrap();
        let module_id = runtime.load_module(ADD_WASM.to_vec()).unwrap();
        
        let result = runtime.unload_module(&module_id);
        assert!(result.is_ok());
        
        let module = runtime.get_module(&module_id);
        assert!(module.is_none());
    }

    #[test]
    fn test_get_stats() {
        let mut runtime = WasmRuntime::new().unwrap();
        runtime.load_module(ADD_WASM.to_vec()).unwrap();
        
        let stats = runtime.get_stats();
        assert_eq!(stats.module_count, 1);
    }
}