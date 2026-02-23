//! Utilities Module
//! 
//! Helper functions and utilities:
//! - File operations
//! - String utilities
//! - Date/time helpers
//! - Logging utilities

use std::path::Path;

/// Check if path exists
pub fn path_exists(path: &str) -> bool {
    Path::new(path).exists()
}

/// Create directory if it doesn't exist
pub fn ensure_dir(path: &str) -> anyhow::Result<()> {
    std::fs::create_dir_all(path)?;
    Ok(())
}

/// Generate unique ID
pub fn generate_id() -> String {
    uuid::Uuid::new_v4().to_string()
}