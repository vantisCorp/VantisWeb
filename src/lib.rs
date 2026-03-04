//! VantisWeb Browser Library
//! 
//! Next-generation web browser with Liquid Core Architecture

pub mod core;
pub mod ui;
pub mod engine;
pub mod extensions;
pub mod security;
pub mod modules;
pub mod profiles;
pub mod network;
pub mod ai;
pub mod utils;
pub mod sync;
pub mod auth;

pub use core::kernel::VantisKernel;
pub use security::SecurityManager;
pub use ui::VantisUI;
pub use sync::{CloudSyncManager, SyncError, SyncProvider, SyncProviderType};
pub use auth::{AuthManager, AuthConfig, AuthError};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = "VantisWeb Browser";