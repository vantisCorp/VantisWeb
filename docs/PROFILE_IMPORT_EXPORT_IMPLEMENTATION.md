# Profile Import/Export Feature Implementation

## Overview
This document describes the implementation of the Profile Import/Export functionality for VantisWeb Browser (Issue #4 from v1.1.0 roadmap).

## Feature Summary
The Profile Import/Export feature allows users to:
- Export single or multiple profiles to JSON files
- Import profiles from previously exported files
- Optionally include/exclude bookmarks and history
- Optionally encrypt exported files (placeholder implementation)
- Validate import files before importing
- Rename profiles during import
- Overwrite or skip existing profiles

## Implementation Details

### Backend Components

#### 1. import_export.rs Module
**Location:** `src/profiles/import_export.rs`

**Key Structures:**
- `ProfileExport` - Single profile export container
- `ProfilesExport` - Multiple profiles export container
- `ExportOptions` - Configuration for export operations
- `ImportOptions` - Configuration for import operations
- `ImportResult` - Results of bulk import operations

**Key Functions:**
- `export_profile_to_file()` - Export a single profile
- `export_profiles_to_file()` - Export multiple profiles
- `import_profile_from_file()` - Import a single profile
- `import_profiles_from_file()` - Import multiple profiles
- `validate_import_file()` - Validate import file structure

**Encryption:**
- Currently using base64 encoding as placeholder
- Planned: AES-256-GCM implementation
- Password-protected exports supported

### Frontend Components

#### 2. ProfileManagerUI Updates
**Location:** `src/ui/profile_ui.rs`

**New Fields:**
```rust
show_export_dialog: bool,
show_import_dialog: bool,
export_selected_profile: Option<String>,
```

**New Methods:**
- `show_export_dialog()` - Display export dialog
- `hide_export_dialog()` - Hide export dialog
- `show_import_dialog()` - Display import dialog
- `hide_import_dialog()` - Hide import dialog
- `render_export_dialog()` - Render export options modal
- `render_import_dialog()` - Render import options modal

**UI Elements:**
- Export button added to each profile card
- Import button in profile manager header
- Export dialog with options:
  - Export name
  - Include bookmarks checkbox
  - Include history checkbox
  - Encrypt checkbox
  - Password field (when encryption enabled)
- Import dialog with options:
  - File selector
  - Password field (for encrypted files)
  - Overwrite existing profiles checkbox
  - Import bookmarks checkbox
  - Import history checkbox
  - New profile name field

#### 3. ProfileManager Integration
**Location:** `src/profiles/mod.rs`

**New Methods Added to ProfileManager:**
```rust
pub fn export_profile(&self, profile_id: &str, path: &Path, options: &ExportOptions) -> Result<()>
pub fn export_all_profiles(&self, path: &Path, options: &ExportOptions) -> Result<()>
pub async fn import_profile(&self, path: &Path, password: Option<&str>, options: &ImportOptions) -> Result<ImportResult>
pub async fn import_profiles(&self, path: &Path, password: Option<&str>, options: &ImportOptions) -> Result<ImportResult>
pub fn validate_import(&self, path: &Path, password: Option<&str>) -> Result<ProfilesExport>
```

### Testing

#### 4. Test Suite
**Location:** `src/profiles/import_export_tests.rs`

**Test Coverage:**
- Single profile export/import
- Multiple profiles export/import
- Export without bookmarks
- Import with name change
- File validation
- Encryption/decryption (placeholder)
- Wrong password handling
- Invalid file handling
- Roundtrip data integrity

**Test Cases:**
1. `test_export_single_profile` - Verify single profile export
2. `test_export_multiple_profiles` - Verify bulk export
3. `test_import_single_profile` - Verify single profile import
4. `test_import_multiple_profiles` - Verify bulk import
5. `test_export_without_bookmarks` - Test selective data export
6. `test_import_with_new_name` - Test profile renaming
7. `test_validate_import_file` - Test file validation
8. `test_import_encrypted_placeholder` - Test encryption (placeholder)
9. `test_import_wrong_password` - Test password validation
10. `test_import_invalid_file` - Test error handling
11. `test_export_import_roundtrip` - Test data integrity

## File Format

### Export File Structure (JSON)
```json
{
  "version": "1.0.0",
  "exported_at": 1234567890000,
  "profiles": [
    {
      "id": "profile_id",
      "name": "Profile Name",
      "profile_type": "Custom",
      "icon": "🧪",
      "color": "#6366f1",
      "created_at": 1234567890000,
      "last_used_at": 1234567895000,
      "bookmarks": [],
      "history": [],
      "settings": {}
    }
  ],
  "encrypted": false
}
```

### Single Profile Export
Same structure but with `profile` field instead of `profiles` array.

## Usage Examples

### Exporting a Profile
```rust
use vantis::profiles::{ProfileManager, ExportOptions};

let manager = ProfileManager::new();
let options = ExportOptions {
    include_bookmarks: true,
    include_history: true,
    encrypt: false,
    password: None,
};

manager.export_profile("profile_id", Path::new("export.json"), &options)?;
```

### Importing a Profile
```rust
use vantis::profiles::{ProfileManager, ImportOptions};

let manager = ProfileManager::new();
let options = ImportOptions {
    overwrite: false,
    include_bookmarks: true,
    include_history: true,
    new_name: None,
};

let result = manager.import_profile(Path::new("export.json"), None, &options).await?;
println!("Imported {} profiles", result.imported_count);
```

### Validating an Import File
```rust
use vantis::profiles::ProfileManager;

let manager = ProfileManager::new();
let validated = manager.validate_import(Path::new("export.json"), None)?;
println!("Valid export with {} profiles", validated.profiles.len());
```

## Error Handling

The implementation includes comprehensive error handling:
- File I/O errors
- JSON parsing errors
- Invalid format errors
- Encryption/decryption errors
- Duplicate profile ID handling
- Missing data validation

All functions return `Result<T>` types with descriptive error messages.

## Future Enhancements

### Planned Features
1. **Real Encryption**: Replace base64 placeholder with AES-256-GCM
2. **Cloud Integration**: Export to cloud storage
3. **Auto-Backup**: Scheduled profile backups
4. **Partial Import**: Select specific bookmarks/history to import
5. **Conflict Resolution**: UI for resolving import conflicts
6. **Version Compatibility**: Handle different export format versions
7. **Compression**: Compress large exports
8. **Verification**: Verify exported file integrity

### Known Limitations
1. Encryption is placeholder (base64 encoding)
2. No conflict resolution UI
3. No incremental updates
4. No merge functionality (only overwrite or skip)

## Security Considerations

### Current Implementation
- Passwords are handled in memory only
- No sensitive data is logged
- Placeholder encryption is not secure
- File permissions not enforced

### Future Improvements
1. Implement proper AES-256-GCM encryption
2. Use secure key derivation (Argon2 or similar)
3. Add file integrity verification (HMAC)
4. Implement secure password entry in UI
5. Add export file signing

## Performance

### Expected Performance
- Single profile export: <100ms
- Bulk export (10 profiles): <500ms
- Single profile import: <100ms
- Bulk import (10 profiles): <500ms
- File validation: <50ms

### Optimization Opportunities
1. Parallel bulk import/export
2. Streaming for large exports
3. Caching of validation results
4. Compression for large histories

## Compatibility

### Version Support
- Current format version: 1.0.0
- Backwards compatibility: Not yet implemented
- Migration path: Planned for v1.2.0

### Platform Support
- Windows: Supported
- macOS: Supported
- Linux: Supported

## Testing Strategy

### Unit Tests
- All core functions tested
- Edge cases covered
- Error scenarios tested

### Integration Tests
- ProfileManager integration tests
- UI component tests
- End-to-end workflow tests

### Manual Testing Checklist
- [ ] Export single profile
- [ ] Export multiple profiles
- [ ] Export without bookmarks
- [ ] Export without history
- [ ] Export with encryption
- [ ] Import single profile
- [ ] Import multiple profiles
- [ ] Import with new name
- [ ] Import with overwrite
- [ ] Import encrypted file
- [ ] Validate import file
- [ ] Handle invalid file
- [ ] Handle wrong password
- [ ] Cancel operations

## Documentation

### User Documentation Needed
- Export profiles guide
- Import profiles guide
- Export format specification
- Troubleshooting guide

### Developer Documentation
- API documentation (rustdoc)
- Architecture diagram
- Data flow diagram
- Error handling guide

## Release Notes

### Version 1.1.0 - Feature Addition
**New Features:**
- Profile export functionality
- Profile import functionality
- Optional encryption
- Import validation
- Export/import UI dialogs

**Improvements:**
- Added export button to profile cards
- Added import button to profile manager
- Comprehensive test coverage

**Known Issues:**
- Encryption is placeholder (base64 only)
- No conflict resolution UI

## Conclusion

The Profile Import/Export feature provides a solid foundation for users to backup, restore, and transfer their browser profiles. The implementation includes:

- ✅ Complete backend functionality
- ✅ UI integration
- ✅ Comprehensive error handling
- ✅ Extensive test coverage
- ✅ Flexible import/export options
- ✅ Validation capabilities

The feature is ready for initial release with placeholder encryption. Future work will focus on implementing real encryption, conflict resolution UI, and additional cloud integration options.