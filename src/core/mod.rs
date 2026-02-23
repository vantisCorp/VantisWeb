//! VantisWeb Core Module
//! 
//! Contains the fundamental components of VantisWeb:
//! - Kernel: Central orchestration
//! - Micro-Scheduler: CPU thread management
//! - Storage: Data persistence
//! - Config: Configuration management

pub mod kernel;
pub mod scheduler;
pub mod storage;
pub mod config;

pub use kernel::VantisKernel;
pub use scheduler::MicroScheduler;
pub use storage::StorageManager;
pub use config::VantisConfig;