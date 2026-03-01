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
pub mod history;
pub mod bookmarks;
pub mod downloads;
pub mod settings;
pub mod private_mode;

