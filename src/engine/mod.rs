//! Web Engine Module
//! 
//! Web rendering engine integration:
//! - WebKit/Blink integration
//! - HTML/CSS/JS support
//! - WebAssembly support
//! - DOM manipulation

pub mod renderer;
pub mod dom;
pub mod wasm;

pub use renderer::WebRenderer;