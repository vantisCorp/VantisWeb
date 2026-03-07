//! Profile Import/Export Module
//!
//! Allows users to import and export profiles to/from files:
//! - Export single or multiple profiles
//! - Import profiles from JSON files
//! - Validate imported profile data
//! - Optional encryption for exported profiles

use anyhow::{Context, Result};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use super::ProfileConfig;

/// Export format for a single profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileExport {
    /// Export format version
    pub version: String,
    /// Export timestamp
    pub exported_at: i64,
    /// Profile data
    pub profile: ProfileConfig,
    /// Encrypted flag
    pub encrypted: bool,
}

/// Export format for multiple profiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilesExport {
    /// Export format version
    pub version: String,
    /// Export timestamp
    pub exported_at: i64,
    /// Profiles data
    pub profiles: Vec<ProfileConfig>,
    /// Encrypted flag
    pub encrypted: bool,
}

/// Import options
#[derive(Debug, Clone, Default)]
pub struct ImportOptions {
    /// Overwrite existing profiles
    pub overwrite: bool,
    /// Include bookmarks
    pub include_bookmarks: bool,
    /// Include history
    pub include_history: bool,
    /// New profile name (for single profile import)
    pub new_name: Option<String>,
}

/// Export options
#[derive(Debug, Clone, Default)]
pub struct ExportOptions {
    /// Include bookmarks
    pub include_bookmarks: bool,
    /// Include history
    pub include_history: bool,
    /// Encrypt export
    pub encrypt: bool,
    /// Encryption password (if encrypt is true)
    pub password: Option<String>,
}

/// Import result
#[derive(Debug, Clone)]
pub struct ImportResult {
    /// Number of profiles imported
    pub imported_count: usize,
    /// Number of profiles skipped
    pub skipped_count: usize,
    /// Number of profiles failed
    pub failed_count: usize,
    /// Import errors
    pub errors: Vec<String>,
    /// Imported profile IDs
    pub imported_ids: Vec<String>,
}

impl ImportResult {
    pub fn new() -> Self {
        Self {
            imported_count: 0,
            skipped_count: 0,
            failed_count: 0,
            errors: Vec::new(),
            imported_ids: Vec::new(),
        }
    }

    pub fn is_success(&self) -> bool {
        self.failed_count == 0
    }
}

/// Validate profile export format
fn validate_export_format(export: &ProfilesExport) -> Result<()> {
    // Check version
    if export.version != "1.0" && export.version != "1.1" {
        return Err(anyhow::anyhow!("Unsupported export format version: {}", export.version));
    }

    // Check profiles
    if export.profiles.is_empty() {
        return Err(anyhow::anyhow!("Export contains no profiles"));
    }

    // Validate each profile
    for profile in &export.profiles {
        if profile.id.is_empty() {
            return Err(anyhow::anyhow!("Profile has empty ID"));
        }
        if profile.name.is_empty() {
            return Err(anyhow::anyhow!("Profile has empty name"));
        }
    }

    Ok(())
}

/// Encrypt profile data using ChaCha20-Poly1305
fn encrypt_data(data: &str, password: &str) -> Result<String> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    use rand::RngCore;
    
    // Generate random salt for key derivation
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);
    
    // Derive key from password
    let key = crate::security::CryptoEngine::derive_key_from_password(password, &salt)?;
    
    // Create cipher with derived key
    let crypto = crate::security::CryptoEngine::with_key(key)?;
    
    // Encrypt the data
    let encrypted = crypto.encrypt(data.as_bytes())?;
    
    // Combine salt + encrypted data and encode as base64
    let mut result = Vec::with_capacity(salt.len() + encrypted.len());
    result.extend_from_slice(&salt);
    result.extend_from_slice(&encrypted);
    
    Ok(STANDARD.encode(&result))
}

/// Decrypt profile data using ChaCha20-Poly1305
fn decrypt_data(encrypted_data: &str, password: &str) -> Result<String> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    
    // Decode base64
    let decoded = STANDARD.decode(encrypted_data)?;
    
    if decoded.len() < 16 + 12 + 16 {
        return Err(anyhow::anyhow!("Encrypted data too short"));
    }
    
    // Extract salt (first 16 bytes)
    let salt = &decoded[..16];
    let data = &decoded[16..];
    
    // Derive key from password with the extracted salt
    let key = crate::security::CryptoEngine::derive_key_from_password(password, salt)?;
    
    // Create cipher with derived key
    let crypto = crate::security::CryptoEngine::with_key(key)?;
    
    // Decrypt the data
    let decrypted = crypto.decrypt(data)?;
    
    Ok(String::from_utf8(decrypted)?)
}

/// Export a single profile to a file
pub fn export_profile_to_file(
    profile: &ProfileConfig,
    path: &Path,
    options: &ExportOptions,
) -> Result<()> {
    info!("Exporting profile '{}' to {}", profile.name, path.display());

    // Create export data
    let mut export_profile = profile.clone();
    
    // Apply export options
    if !options.include_bookmarks {
        export_profile.bookmarks.clear();
    }
    if !options.include_history {
        export_profile.history.clear();
    }

    // Create export format
    let export = ProfileExport {
        version: "1.1".to_string(),
        exported_at: chrono::Utc::now().timestamp_millis(),
        profile: export_profile,
        encrypted: options.encrypt,
    };

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&export)
        .context("Failed to serialize profile for export")?;

    // Encrypt if requested
    let final_data = if options.encrypt {
        let password = options.password.as_ref()
            .context("Password required for encryption")?;
        encrypt_data(&json, password)?
    } else {
        json
    };

    // Write to file
    let file = File::create(path)
        .context("Failed to create export file")?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &final_data)
        .context("Failed to write export file")?;

    info!("Profile exported successfully");
    Ok(())
}

/// Export multiple profiles to a file
pub fn export_profiles_to_file(
    profiles: &[ProfileConfig],
    path: &Path,
    options: &ExportOptions,
) -> Result<()> {
    info!("Exporting {} profiles to {}", profiles.len(), path.display());

    // Apply export options to each profile
    let export_profiles: Vec<ProfileConfig> = profiles
        .iter()
        .map(|p| {
            let mut profile = p.clone();
            if !options.include_bookmarks {
                profile.bookmarks.clear();
            }
            if !options.include_history {
                profile.history.clear();
            }
            profile
        })
        .collect();

    // Create export format
    let export = ProfilesExport {
        version: "1.1".to_string(),
        exported_at: chrono::Utc::now().timestamp_millis(),
        profiles: export_profiles,
        encrypted: options.encrypt,
    };

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&export)
        .context("Failed to serialize profiles for export")?;

    // Encrypt if requested
    let final_data = if options.encrypt {
        let password = options.password.as_ref()
            .context("Password required for encryption")?;
        encrypt_data(&json, password)?
    } else {
        json
    };

    // Write to file
    let file = File::create(path)
        .context("Failed to create export file")?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &final_data)
        .context("Failed to write export file")?;

    info!("Profiles exported successfully");
    Ok(())
}

/// Import a single profile from a file
pub fn import_profile_from_file(
    path: &Path,
    password: Option<&str>,
    options: &ImportOptions,
) -> Result<(ProfileConfig, ImportResult)> {
    info!("Importing profile from {}", path.display());

    let mut result = ImportResult::new();

    // Read file
    let file = File::open(path)
        .context("Failed to open import file")?;
    let reader = BufReader::new(file);

    // Determine if encrypted or not
    let content: String = serde_json::from_reader(reader)
        .context("Failed to read import file")?;

    // Decrypt if password provided
    let json_content = if let Some(pwd) = password {
        decrypt_data(&content, pwd)
            .context("Failed to decrypt import file")?
    } else {
        content
    };

    // Try to parse as single profile export first
    if let Ok(export) = serde_json::from_str::<ProfileExport>(&json_content) {
        let mut profile = export.profile;

        // Apply import options
        if !options.include_bookmarks {
            profile.bookmarks.clear();
        }
        if !options.include_history {
            profile.history.clear();
        }

        // Apply new name if provided
        if let Some(new_name) = &options.new_name {
            profile.name = new_name.clone();
        }

        // Generate new ID for imported profile
        profile.id = uuid::Uuid::new_v4().to_string();
        profile.active = false;
        profile.last_used_at = chrono::Utc::now().timestamp_millis();

        result.imported_count = 1;
        result.imported_ids.push(profile.id.clone());

        info!("Profile '{}' imported successfully", profile.name);
        return Ok((profile, result));
    }

    // Try to parse as multiple profiles export
    if let Ok(export) = serde_json::from_str::<ProfilesExport>(&json_content) {
        // Validate export format
        validate_export_format(&export)
            .context("Invalid export format")?;

        // Import first profile
        if let Some(mut profile) = export.profiles.first().cloned() {
            // Apply import options
            if !options.include_bookmarks {
                profile.bookmarks.clear();
            }
            if !options.include_history {
                profile.history.clear();
            }

            // Apply new name if provided
            if let Some(new_name) = &options.new_name {
                profile.name = new_name.clone();
            }

            // Generate new ID for imported profile
            profile.id = uuid::Uuid::new_v4().to_string();
            profile.active = false;
            profile.last_used_at = chrono::Utc::now().timestamp_millis();

            result.imported_count = 1;
            result.imported_ids.push(profile.id.clone());

            info!("Profile '{}' imported successfully", profile.name);
            return Ok((profile, result));
        }
    }

    Err(anyhow::anyhow!("Invalid import file format"))
}

/// Import multiple profiles from a file
pub fn import_profiles_from_file(
    path: &Path,
    password: Option<&str>,
    options: &ImportOptions,
) -> Result<(Vec<ProfileConfig>, ImportResult)> {
    info!("Importing profiles from {}", path.display());

    let mut result = ImportResult::new();

    // Read file
    let file = File::open(path)
        .context("Failed to open import file")?;
    let reader = BufReader::new(file);

    // Determine if encrypted or not
    let content: String = serde_json::from_reader(reader)
        .context("Failed to read import file")?;

    // Decrypt if password provided
    let json_content = if let Some(pwd) = password {
        decrypt_data(&content, pwd)
            .context("Failed to decrypt import file")?
    } else {
        content
    };

    // Parse as multiple profiles export
    let export: ProfilesExport = serde_json::from_str(&json_content)
        .context("Failed to parse import file")?;

    // Validate export format
    validate_export_format(&export)
        .context("Invalid export format")?;

    // Import profiles
    let mut imported_profiles = Vec::new();
    for mut profile in export.profiles {
        // Apply import options
        if !options.include_bookmarks {
            profile.bookmarks.clear();
        }
        if !options.include_history {
            profile.history.clear();
        }

        // Generate new ID for imported profile
        profile.id = uuid::Uuid::new_v4().to_string();
        profile.active = false;
        profile.last_used_at = chrono::Utc::now().timestamp_millis();

        imported_profiles.push(profile);
        result.imported_count += 1;
        result.imported_ids.push(imported_profiles.last().unwrap().id.clone());
    }

    info!("{} profiles imported successfully", result.imported_count);
    Ok((imported_profiles, result))
}

/// Validate profile import file without importing
pub fn validate_import_file(
    path: &Path,
    password: Option<&str>,
) -> Result<ProfilesExport> {
    debug!("Validating import file: {}", path.display());

    // Read file
    let file = File::open(path)
        .context("Failed to open import file")?;
    let reader = BufReader::new(file);

    // Determine if encrypted or not
    let content: String = serde_json::from_reader(reader)
        .context("Failed to read import file")?;

    // Decrypt if password provided
    let json_content = if let Some(pwd) = password {
        decrypt_data(&content, pwd)
            .context("Failed to decrypt import file")?
    } else {
        content
    };

    // Try to parse as single profile export
    if let Ok(export) = serde_json::from_str::<ProfileExport>(&json_content) {
        return Ok(ProfilesExport {
            version: export.version,
            exported_at: export.exported_at,
            profiles: vec![export.profile],
            encrypted: export.encrypted,
        });
    }

    // Try to parse as multiple profiles export
    let export: ProfilesExport = serde_json::from_str(&json_content)
        .context("Failed to parse import file")?;

    // Validate export format
    validate_export_format(&export)?;

    debug!("Import file is valid");
    Ok(export)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_export_format() {
        let export = ProfilesExport {
            version: "1.1".to_string(),
            exported_at: chrono::Utc::now().timestamp_millis(),
            profiles: vec![ProfileConfig::new("Test".to_string(), ProfileType::Default)],
            encrypted: false,
        };
        assert!(validate_export_format(&export).is_ok());
    }

    #[test]
    fn test_validate_export_format_invalid_version() {
        let export = ProfilesExport {
            version: "2.0".to_string(),
            exported_at: chrono::Utc::now().timestamp_millis(),
            profiles: vec![ProfileConfig::new("Test".to_string(), ProfileType::Default)],
            encrypted: false,
        };
        assert!(validate_export_format(&export).is_err());
    }

    #[test]
    fn test_import_result() {
        let result = ImportResult::new();
        assert!(result.is_success());
        assert_eq!(result.imported_count, 0);
    }
}