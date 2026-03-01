//! Web Engine Module
//! 
//! Web rendering engine integration:
//! - WebKit/Blink integration
//! - HTML/CSS/JS support
//! - WebAssembly support
//! - DOM manipulation
//! - Navigation system
//! - Event loop
//! - Fetch API
//! - Storage API
//! - Console API
//! - JavaScript Bridge

pub mod renderer;
pub mod web_renderer;
pub mod dom;
pub mod parser;
pub mod js_runtime;
pub mod js_bridge;
pub mod wasm;
pub mod navigation;
pub mod event_loop;
pub mod fetch;
pub mod storage;
pub mod console;

pub use wasm::{WasmRuntime, WasmModule, WasmFunction, WasmMemory, WasmRuntimeStats};

