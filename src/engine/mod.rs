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

pub mod renderer;
pub mod web_renderer;
pub mod dom;
pub mod parser;
pub mod js_runtime;
pub mod wasm;
pub mod navigation;
pub mod event_loop;
pub mod fetch;
pub mod storage;
pub mod console;

pub use renderer::WebRenderer;
pub use web_renderer::WebRenderer as VantisWebRenderer;
pub use navigation::{NavigationManager, NavigationEntry, NavigationState, NavigationEvent};
pub use event_loop::{EventLoop, Task, TaskType, TaskStatus, EventLoopStats};
pub use fetch::{FetchApi, HttpRequest, HttpResponse, HttpMethod, HttpHeaders, RequestBody, CorsMode};
pub use storage::{StorageApi, StorageType, StorageEvent, StorageEntry, CookieManager, Cookie};
pub use console::{ConsoleApi, LogLevel, ConsoleEntry, PerformanceMetric};