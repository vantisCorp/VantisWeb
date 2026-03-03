//! Tests for profile import/export functionality
//!
//! Comprehensive tests for the import/export module including:
//! - Single profile export/import
//! - Multiple profiles export/import
//! - With and without bookmarks/history
//! - Encryption (placeholder)
//! - Error handling
//! - Validation

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::import_export::*;
    use std::fs;
    use tempfile::NamedTempFile;

    /// Create a test profile
    fn create_test_profile(id: &str, name: &str) -> ProfileConfig {
        ProfileConfig {
            id: id.to_string(),
            name: name.to_string(),
            profile_type: ProfileType::Custom,
            icon: Some("🧪".to_string()),
            color: Some("#6366f1".to_string()),
            created_at: 1234567890000,
            last_used_at: 1234567895000,
            bookmarks: vec![
                Bookmark {
                    id: "bk1".to_string(),
                    url: "https://example.com".to_string(),
                    title: "Example".to_string(),
                    favicon: None,
                    created_at: 1234567890000,
                }
            ],
            history: vec![
                HistoryEntry {
                    url: "https://example.com/page".to_string(),
                    title: "Example Page".to_string(),
                    visited_at: 1234567895000,
                }
            ],
            settings: ProfileSettings::default(),
        }
    }

    #[test]
    fn test_export_single_profile() {
        let profile = create_test_profile("test1", "Test Profile 1");
        let temp_file = NamedTempFile::new().unwrap();
        
        let options = ExportOptions {
            include_bookmarks: true,
            include_history: true,
            encrypt: false,
            password: None,
        };

        let result = export_profile_to_file(&profile, temp_file.path(), &options);
        assert!(result.is_ok(), "Export should succeed");

        // Verify file exists
        assert!(temp_file.path().exists(), "Export file should exist");

        // Read and verify content
        let content = fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.contains("Test Profile 1"), "Export should contain profile name");
        assert!(content.contains("https://example.com"), "Export should contain bookmarks");
    }

    #[test]
    fn test_export_multiple_profiles() {
        let profiles = vec![
            create_test_profile("test1", "Test Profile 1"),
            create_test_profile("test2", "Test Profile 2"),
        ];
        let temp_file = NamedTempFile::new().unwrap();
        
        let options = ExportOptions {
            include_bookmarks: true,
            include_history: false,
            encrypt: false,
            password: None,
        };

        let result = export_profiles_to_file(&profiles, temp_file.path(), &options);
        assert!(result.is_ok(), "Export should succeed");

        // Verify file exists
        assert!(temp_file.path().exists(), "Export file should exist");

        // Read and verify content
        let content = fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.contains("Test Profile 1"), "Export should contain first profile");
        assert!(content.contains("Test Profile 2"), "Export should contain second profile");
    }

    #[test]
    fn test_import_single_profile() {
        let profile = create_test_profile("test1", "Test Profile 1");
        let export_file = NamedTempFile::new().unwrap();
        
        let export_options = ExportOptions {
            include_bookmarks: true,
            include_history: true,
            encrypt: false,
            password: None,
        };

        // First export
        export_profile_to_file(&profile, export_file.path(), &export_options).unwrap();

        // Then import
        let import_options = ImportOptions {
            overwrite: false,
            include_bookmarks: true,
            include_history: true,
            new_name: None,
        };

        let result = import_profile_from_file(export_file.path(), None, &import_options);
        assert!(result.is_ok(), "Import should succeed");

        let imported = result.unwrap();
        assert_eq!(imported.name, "Test Profile 1");
        assert_eq!(imported.bookmarks.len(), 1);
        assert_eq!(imported.history.len(), 1);
    }

    #[test]
    fn test_import_multiple_profiles() {
        let profiles = vec![
            create_test_profile("test1", "Test Profile 1"),
            create_test_profile("test2", "Test Profile 2"),
        ];
        let export_file = NamedTempFile::new().unwrap();
        
        let export_options = ExportOptions {
            include_bookmarks: true,
            include_history: true,
            encrypt: false,
            password: None,
        };

        // First export
        export_profiles_to_file(&profiles, export_file.path(), &export_options).unwrap();

        // Then import
        let import_options = ImportOptions {
            overwrite: false,
            include_bookmarks: true,
            include_history: true,
            new_name: None,
        };

        let result = import_profiles_from_file(export_file.path(), None, &import_options);
        assert!(result.is_ok(), "Import should succeed");

        let imported = result.unwrap();
        assert_eq!(imported.imported_count, 2);
        assert_eq!(imported.failed_count, 0);
    }

    #[test]
    fn test_export_without_bookmarks() {
        let profile = create_test_profile("test1", "Test Profile 1");
        let temp_file = NamedTempFile::new().unwrap();
        
        let options = ExportOptions {
            include_bookmarks: false,
            include_history: true,
            encrypt: false,
            password: None,
        };

        let result = export_profile_to_file(&profile, temp_file.path(), &options);
        assert!(result.is_ok());

        let content = fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.contains("Test Profile 1"), "Should contain profile name");
        // Bookmarks should be empty
        assert!(content.contains(r#""bookmarks":[]"#), "Bookmarks should be empty");
    }

    #[test]
    fn test_import_with_new_name() {
        let profile = create_test_profile("test1", "Test Profile 1");
        let export_file = NamedTempFile::new().unwrap();
        
        let export_options = ExportOptions {
            include_bookmarks: true,
            include_history: true,
            encrypt: false,
            password: None,
        };

        export_profile_to_file(&profile, export_file.path(), &export_options).unwrap();

        let import_options = ImportOptions {
            overwrite: false,
            include_bookmarks: true,
            include_history: true,
            new_name: Some("Renamed Profile".to_string()),
        };

        let result = import_profile_from_file(export_file.path(), None, &import_options);
        assert!(result.is_ok());

        let imported = result.unwrap();
        assert_eq!(imported.name, "Renamed Profile", "Profile name should be changed");
    }

    #[test]
    fn test_validate_import_file() {
        let profile = create_test_profile("test1", "Test Profile 1");
        let export_file = NamedTempFile::new().unwrap();
        
        let options = ExportOptions {
            include_bookmarks: true,
            include_history: true,
            encrypt: false,
            password: None,
        };

        export_profile_to_file(&profile, export_file.path(), &options).unwrap();

        let result = validate_import_file(export_file.path(), None);
        assert!(result.is_ok(), "Validation should succeed");

        let validated = result.unwrap();
        assert_eq!(validated.version, "1.0.0");
        assert_eq!(validated.profiles.len(), 1);
        assert_eq!(validated.profiles[0].name, "Test Profile 1");
    }

    #[test]
    fn test_import_encrypted_placeholder() {
        // This tests the placeholder encryption functionality
        // When real AES-256-GCM is implemented, this will test actual encryption
        
        let profile = create_test_profile("test1", "Test Profile 1");
        let export_file = NamedTempFile::new().unwrap();
        
        let options = ExportOptions {
            include_bookmarks: true,
            include_history: true,
            encrypt: true,
            password: Some("test_password".to_string()),
        };

        let result = export_profile_to_file(&profile, export_file.path(), &options);
        // Placeholder encryption should succeed
        assert!(result.is_ok());

        let content = fs::read_to_string(temp_file.path()).unwrap();
        // In placeholder mode, encryption flag is set but data is base64 encoded
        assert!(content.contains(r#""encrypted":true"#), "Should be marked as encrypted");

        // Import should work with the same password
        let import_options = ImportOptions {
            overwrite: false,
            include_bookmarks: true,
            include_history: true,
            new_name: None,
        };

        let import_result = import_profile_from_file(export_file.path(), Some("test_password"), &import_options);
        assert!(import_result.is_ok(), "Import with correct password should succeed");
    }

    #[test]
    fn test_import_wrong_password() {
        let profile = create_test_profile("test1", "Test Profile 1");
        let export_file = NamedTempFile::new().unwrap();
        
        let options = ExportOptions {
            include_bookmarks: true,
            include_history: true,
            encrypt: true,
            password: Some("correct_password".to_string()),
        };

        export_profile_to_file(&profile, export_file.path(), &options).unwrap();

        let import_options = ImportOptions {
            overwrite: false,
            include_bookmarks: true,
            include_history: true,
            new_name: None,
        };

        // Wrong password should fail
        let result = import_profile_from_file(export_file.path(), Some("wrong_password"), &import_options);
        assert!(result.is_err(), "Import with wrong password should fail");
    }

    #[test]
    fn test_import_invalid_file() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), "invalid json").unwrap();

        let import_options = ImportOptions {
            overwrite: false,
            include_bookmarks: true,
            include_history: true,
            new_name: None,
        };

        let result = import_profile_from_file(temp_file.path(), None, &import_options);
        assert!(result.is_err(), "Import of invalid file should fail");
    }

    #[test]
    fn test_export_import_roundtrip() {
        // Test that data survives export -> import roundtrip
        let original = create_test_profile("test1", "Test Profile 1");
        let export_file = NamedTempFile::new().unwrap();
        
        let export_options = ExportOptions {
            include_bookmarks: true,
            include_history: true,
            encrypt: false,
            password: None,
        };

        // Export
        export_profile_to_file(&original, export_file.path(), &export_options).unwrap();

        // Import
        let import_options = ImportOptions {
            overwrite: false,
            include_bookmarks: true,
            include_history: true,
            new_name: None,
        };

        let imported = import_profile_from_file(export_file.path(), None, &import_options).unwrap();

        // Verify all fields match
        assert_eq!(imported.name, original.name);
        assert_eq!(imported.profile_type, original.profile_type);
        assert_eq!(imported.icon, original.icon);
        assert_eq!(imported.color, original.color);
        assert_eq!(imported.bookmarks.len(), original.bookmarks.len());
        assert_eq!(imported.history.len(), original.history.len());
        
        if let (Some(imported_bk), Some(original_bk)) = (imported.bookmarks.first(), original.bookmarks.first()) {
            assert_eq!(imported_bk.url, original_bk.url);
            assert_eq!(imported_bk.title, original_bk.title);
        }
    }
}