//! Web Engine Module
//! 
//! Web rendering engine integration:
//! - WebKit/Blink integration
//! - HTML/CSS/JS support
//! - WebAssembly support
//! - DOM manipulation

pub mod renderer;
pub mod web_renderer;
pub mod dom;
pub mod parser;
pub mod js_runtime;
pub mod wasm;

pub use renderer::WebRenderer;
pub use web_renderer::WebRenderer as VantisWebRenderer;